use crate::clients::api_client;
use crate::topo_query::TopoQuery;
use crate::topo_writer::TopoWriter;
use anyhow::Context;
use anyhow::Error;
use log::info;
use std::collections::{HashMap, HashSet};

pub struct GtfsImporter {
    pub writer: TopoWriter,
    pub query: TopoQuery,
}

impl GtfsImporter {
    pub fn new(api_endpoint: &str, sparql_enpoint: &str, topo_id_id: &str) -> Result<Self, Error> {
        let query = TopoQuery::new(sparql_enpoint, topo_id_id)
            .context("impossible to create query client")?;
        Ok(Self {
            writer: TopoWriter::new(api_endpoint, query.known_entities.clone())?,
            query,
        })
    }

    pub fn import_gtfs(
        &self,
        gtfs_filename: &str,
        producer_id: &str,
        producer_name: &str,
        override_existing: bool,
    ) -> Result<(), anyhow::Error> {
        let raw_gtfs = gtfs_structures::RawGtfs::new(gtfs_filename).map_err(|e| e.compat())?;

        log::info!("import gtfs version {}", crate::GIT_VERSION);
        let data_source_id =
            self.writer
                .insert_data_source(&raw_gtfs.sha256, &producer_id, gtfs_filename)?;

        let gtfs = gtfs_structures::Gtfs::try_from(raw_gtfs).map_err(|e| e.compat())?;

        let route_mapping =
            self.import_routes(&gtfs.routes, &data_source_id, producer_id, producer_name)?;
        let stop_mapping =
            self.import_stops(&gtfs.stops, &data_source_id, producer_id, override_existing)?;
        self.insert_stop_relations(&gtfs.stops, &stop_mapping)?;
        self.insert_stop_route_relations(&gtfs.trips, &stop_mapping, &route_mapping)?;

        Ok(())
    }

    pub fn import_routes(
        &self,
        routes: &HashMap<String, gtfs_structures::Route>,
        data_source_id: &str,
        producer_id: &str,
        producer_name: &str,
    ) -> Result<std::collections::HashMap<String, String>, anyhow::Error> {
        routes
            .values()
            .map(|route| {
                let r = self.query.find_route(&producer_id, &route.id)?;
                match r {
                    None => {
                        info!(
                            "Route “{}” ({}) does not exist, inserting",
                            route.long_name, route.short_name
                        );
                        let wikibase_id =
                            self.writer
                                .insert_route(&route, &data_source_id, producer_name)?;
                        Ok((route.id.to_owned(), wikibase_id))
                    }
                    Some(route_id) => {
                        info!(
                            "Route “{}” ({}) already exists with id {}, skipping",
                            route.long_name, route.short_name, route_id
                        );
                        Ok((route.id.to_owned(), route_id.to_owned()))
                    }
                }
            })
            .collect()
    }

    pub fn import_stops(
        &self,
        stops: &HashMap<String, std::sync::Arc<gtfs_structures::Stop>>,
        data_source_id: &str,
        producer_id: &str,
        override_existing: bool,
    ) -> Result<std::collections::HashMap<String, String>, anyhow::Error> {
        stops
            .values()
            .map(|stop| {
                let s = self.query.find_stop(&producer_id, &stop)?;
                match s {
                    None => {
                        info!(
                            "Stop “{}” ({}) does not exist, inserting",
                            stop.name, stop.id
                        );
                        let wikibase_id = self.writer.insert_stop(&stop, &data_source_id)?;
                        Ok((stop.id.to_owned(), wikibase_id))
                    }
                    Some(stop_id) => {
                        if override_existing {
                            info!(
                            "Stop “{}” ({}) already exists with id {}, updating it with new claims",
                            stop.name, stop.id, stop_id
                        );
                            self.writer.update_stop(&stop_id, stop, data_source_id)?;
                        } else {
                            info!(
                                "Stop “{}” ({}) already exists with id {}, skipping",
                                stop.name, stop.id, stop_id
                            );
                        }
                        Ok((stop.id.to_owned(), stop_id.to_owned()))
                    }
                }
            })
            .collect()
    }

    pub fn insert_stop_relations(
        &self,
        stops: &HashMap<String, std::sync::Arc<gtfs_structures::Stop>>,
        id_mapping: &std::collections::HashMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        for stop in stops.values() {
            if let Some(parent_gtfs_id) = &stop.parent_station {
                let parent_wikibase_id = match id_mapping.get(parent_gtfs_id) {
                    Some(id) => id,
                    None => {
                        log::warn!("Could not find wikibase id for gtfs id: {}", parent_gtfs_id);
                        continue;
                    }
                };
                let child_wikibase_id = match id_mapping.get(&stop.id) {
                    Some(id) => id,
                    None => {
                        log::warn!("Could not find wikibase id for gtfs id: {}", parent_gtfs_id);
                        continue;
                    }
                };
                let claim = api_client::claim_item(
                    &self.query.known_entities.properties.part_of,
                    parent_wikibase_id,
                );
                self.writer
                    .client
                    .add_claims(child_wikibase_id, vec![claim])?;
            }
        }
        Ok(())
    }

    pub fn insert_stop_route_relations(
        &self,
        trips: &HashMap<String, gtfs_structures::Trip>,
        stop_mapping: &std::collections::HashMap<String, String>,
        route_mapping: &std::collections::HashMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        log::info!("inserting stop/routes relations");
        let mut stops_by_routes: HashMap<String, HashSet<String>> = HashMap::new();
        for trip in trips.values() {
            let stops = stops_by_routes
                .entry(trip.route_id.clone())
                .or_insert_with(HashSet::new);
            for s in &trip.stop_times {
                stops.insert(s.stop.id.clone());
            }
        }
        for (route_id, stops) in stops_by_routes.iter() {
            let route_topo_id = match route_mapping.get(route_id) {
                Some(id) => id,
                None => {
                    log::warn!("Could not find wikibase id for gtfs route id: {}", route_id);
                    continue;
                }
            };
            for stop_id in stops.iter() {
                let stop_topo_id = match stop_mapping.get(stop_id) {
                    Some(id) => id,
                    None => {
                        log::warn!("Could not find wikibase id for gtfs id: {}", stop_id);
                        continue;
                    }
                };
                let claim = api_client::claim_item(
                    &self.query.known_entities.properties.part_of,
                    &route_topo_id,
                );

                self.writer.client.add_claims(stop_topo_id, vec![claim])?;
            }
        }

        Ok(())
    }
}
