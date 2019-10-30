use crate::api_client::ApiClient;
use crate::sparql_client::SparqlClient;
use anyhow::Error;
use log::{info, warn};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct EntitiesId {
    pub properties: Properties,
    pub items: Items,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Properties {
    pub topo_id_id: String,
    pub produced_by: String,
    pub instance_of: String,
    pub physical_mode: String,
    pub gtfs_short_name: String,
    pub gtfs_long_name: String,
    pub gtfs_id: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Items {
    pub line: String,
    pub producer: String,
    pub bus: String,
}

pub struct Client {
    pub api: ApiClient,
    pub sparql: SparqlClient,
}

impl EntitiesId {
    pub fn physical_mode(&self, route: &gtfs_structures::Route) -> &str {
        use gtfs_structures::RouteType::*;
        match route.route_type {
            Bus => &self.items.bus,
            _ => &self.items.bus,
        }
    }
}

impl Client {
    pub fn new(api_endpoint: &str, sparql_enpoint: &str, topo_id_id: &str) -> Result<Self, Error> {
        let sparql = SparqlClient::new(sparql_enpoint, topo_id_id)?;
        Ok(Self {
            api: ApiClient::new(api_endpoint, sparql.config.clone())?,
            sparql,
        })
    }

    pub fn import_gtfs(
        &self,
        gtfs_filename: &str,
        producer_id: &str,
        producer_name: &str,
    ) -> Result<(), anyhow::Error> {
        let gtfs = gtfs_structures::RawGtfs::from_zip(gtfs_filename).map_err(|e| e.compat())?;
        let routes = gtfs.routes.map_err(|e| e.compat())?;

        for route in routes {
            let r = self.sparql.find_line(producer_id, &route.id)?;
            match r.as_slice() {
                [] => {
                    info!(
                        "Line “{}” ({}) does not exist, inserting",
                        route.long_name, route.short_name
                    );
                    self.api.insert_route(producer_id, producer_name, &route)?;
                }
                [e] => {
                    info!(
                        "Line “{}” ({}) already exists with id {}, skipping",
                        route.long_name, route.short_name, e["line"]
                    );
                }
                _ => warn!(
                    "Line “{}” ({}) exists many times. Something is not right",
                    route.long_name, route.short_name
                ),
            }
        }
        Ok(())
    }
}
