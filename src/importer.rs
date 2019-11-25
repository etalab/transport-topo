use crate::clients::api_client;
use crate::topo_query::TopoQuery;
use crate::topo_writer::TopoWriter;
use anyhow::Context;
use anyhow::Error;
use log::{info, warn};

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
    ) -> Result<(), anyhow::Error> {
        let gtfs = gtfs_structures::RawGtfs::new(gtfs_filename).map_err(|e| e.compat())?;

        log::info!("import gtfs version {}", crate::GIT_VERSION);
        let data_source_id =
            self.writer
                .insert_data_source(&gtfs.sha256, &producer_id, gtfs_filename)?;

        let routes = gtfs.routes.map_err(|e| e.compat())?;
        self.import_routes(&routes, &data_source_id, producer_id, producer_name)?;
        let stops = gtfs.stops.map_err(|e| e.compat())?;
        let id_mapping = self.import_stops(&stops, &data_source_id, producer_id)?;
        self.insert_stop_relations(&stops, id_mapping)?;

        Ok(())
    }

    pub fn import_routes(
        &self,
        routes: &[gtfs_structures::Route],
        data_source_id: &str,
        producer_id: &str,
        producer_name: &str,
    ) -> Result<(), anyhow::Error> {
        for route in routes {
            let r = self.query.find_route(&producer_id, &route.id)?;
            match r.as_slice() {
                [] => {
                    info!(
                        "Route “{}” ({}) does not exist, inserting",
                        route.long_name, route.short_name
                    );
                    self.writer
                        .insert_route(&route, &data_source_id, producer_name)?;
                }
                [e] => {
                    info!(
                        "Route “{}” ({}) already exists with id {}, skipping",
                        route.long_name, route.short_name, e["route"]
                    );
                }
                _ => warn!(
                    "Route “{}” ({}) exists many times. Something is not right",
                    route.long_name, route.short_name
                ),
            }
        }
        Ok(())
    }

    pub fn import_stops(
        &self,
        stops: &[gtfs_structures::Stop],
        data_source_id: &str,
        producer_id: &str,
    ) -> Result<std::collections::HashMap<String, String>, anyhow::Error> {
        stops
            .iter()
            .map(|stop| {
                let s = self.query.find_stop(&producer_id, &stop)?;
                match s.as_slice() {
                    [] => {
                        info!(
                            "Stop “{}” ({}) does not exist, inserting",
                            stop.name, stop.id
                        );
                        let wikibase_id = self.writer.insert_stop(&stop, &data_source_id)?;
                        Ok((stop.id.to_owned(), wikibase_id))
                    }
                    [e] => {
                        info!(
                            "Stop “{}” ({}) already exists with id {}, skipping",
                            stop.name, stop.id, e["stop"]
                        );
                        Ok((stop.id.to_owned(), e["stop"].to_owned()))
                    }
                    _ => Err(anyhow::anyhow!(
                        "Stop “{}” ({}) exists many times. Something is not right",
                        stop.name,
                        stop.id
                    )),
                }
            })
            .collect()
    }

    pub fn insert_stop_relations(
        &self,
        stops: &[gtfs_structures::Stop],
        id_mapping: std::collections::HashMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        for stop in stops {
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
                self.writer.client.add_claims(child_wikibase_id, vec![claim])?;
            }
        }
        Ok(())
    }
}
