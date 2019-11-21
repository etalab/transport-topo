use crate::client::{EntitiesId, Items, Properties};
use anyhow::Context;
use itertools::Itertools;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SparqlError {
    #[error("No entity with topo id {0} found")]
    TopoIdNotFound(String),
    #[error("Several entities with topo id {0}")]
    DuplicatedTopoId(String),
    #[error("Impossible to query: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Invalid json: {0}")]
    InvalidJsonError(#[from] json::Error),
    #[error("Error parsing the id {0} for entity with topo id {1}")]
    TopoInvalidId(String, String),
    #[error("Too many elements {0}")]
    Duplicate(String),
}

pub fn read_id_from_url(url: &str) -> Option<String> {
    url.split('/')
        .collect::<Vec<_>>()
        .last()
        .map(|id| id.to_string())
}

pub struct SparqlClient {
    client: reqwest::Client,
    endpoint: String,
    pub config: crate::client::EntitiesId,
}

impl SparqlClient {
    /// initialize a client without discovering the configuration
    pub fn new_without_config(endpoint: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.to_owned(),
            config: Default::default(),
        }
    }

    /// create a new client and discory all the base entities id
    pub fn new(endpoint: &str, topo_id_id: &str) -> Result<Self, anyhow::Error> {
        let mut client = Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.to_owned(),
            config: EntitiesId {
                properties: Properties {
                    topo_id_id: topo_id_id.to_owned(),
                    ..Default::default()
                },
                ..Default::default()
            },
        };

        client.config = client
            .discover_config()
            .context("impossible to discovery config")?;
        Ok(client)
    }

    pub fn discover_config(&self) -> Result<EntitiesId, SparqlError> {
        Ok(EntitiesId {
            items: Items {
                physical_mode: self.find_entity_by_topo_id("physical_mode")?,
                route: self.find_entity_by_topo_id("route")?,
                producer: self.find_entity_by_topo_id("producer")?,
                tramway: self.find_entity_by_topo_id("tramway")?,
                subway: self.find_entity_by_topo_id("subway")?,
                railway: self.find_entity_by_topo_id("railway")?,
                bus: self.find_entity_by_topo_id("bus")?,
                ferry: self.find_entity_by_topo_id("ferry")?,
                cable_car: self.find_entity_by_topo_id("cable_car")?,
                gondola: self.find_entity_by_topo_id("gondola")?,
                funicular: self.find_entity_by_topo_id("funicular")?,
                stop_point: self.find_entity_by_topo_id("stop_point")?,
                stop_area: self.find_entity_by_topo_id("stop_area")?,
                stop_entrance: self.find_entity_by_topo_id("stop_entrance")?,
                stop_generic_node: self.find_entity_by_topo_id("stop_generic_node")?,
                stop_boarding_area: self.find_entity_by_topo_id("stop_boarding_area")?,
            },
            properties: Properties {
                topo_id_id: self.config.properties.topo_id_id.to_string(),
                produced_by: self.find_entity_by_topo_id("produced_by")?,
                instance_of: self.find_entity_by_topo_id("instance_of")?,
                gtfs_short_name: self.find_entity_by_topo_id("gtfs_short_name")?,
                gtfs_long_name: self.find_entity_by_topo_id("gtfs_long_name")?,
                gtfs_name: self.find_entity_by_topo_id("gtfs_name")?,
                gtfs_id: self.find_entity_by_topo_id("gtfs_id")?,
                first_seen_in: self.find_entity_by_topo_id("first_seen_in")?,
                data_source: self.find_entity_by_topo_id("data_source")?,
                source: self.find_entity_by_topo_id("source")?,
                file_format: self.find_entity_by_topo_id("file_format")?,
                sha_256: self.find_entity_by_topo_id("sha_256")?,
                has_physical_mode: self.find_entity_by_topo_id("has_physical_mode")?,
                tool_version: self.find_entity_by_topo_id("tool_version")?,
                part_of: self.find_entity_by_topo_id("part_of")?,
                connecting_line: self.find_entity_by_topo_id("connecting_line")?,
            },
        })
    }
    fn query(&self, query: &str) -> Result<json::JsonValue, SparqlError> {
        log::debug!("Sparql query: {}", query);
        let response = self
            .client
            .get(&self.endpoint)
            .query(&[("format", "json"), ("query", query)])
            .send()?
            .error_for_status()?
            .text()?;
        log::trace!("Query response: {:?}", response);
        Ok(json::parse(&response)?)
    }

    pub fn sparql(
        &self,
        variables: &[&str],
        where_clause: &str,
    ) -> Result<Vec<HashMap<String, String>>, SparqlError> {
        let vars = variables.iter().format(" ");
        let query = format!("SELECT {} WHERE {{ {} SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }} }}", vars, where_clause);
        let res = self.query(&query)?;

        let mut result = Vec::new();
        for binding in res["results"]["bindings"].members() {
            let values = binding
                .entries()
                .map(|(k, v)| (k.to_string(), v["value"].as_str().unwrap_or("").into()))
                .collect();
            result.push(values);
        }
        Ok(result)
    }

    pub fn find_route(
        &self,
        producer_id: &str,
        gtfs_id: &str,
    ) -> Result<Vec<HashMap<String, String>>, SparqlError> {
        log::trace!("Finding route {} of producer {}", gtfs_id, producer_id);
        self.sparql(
            &[
                "?route",
                "?routeLabel",
                "?route_short_name",
                "?route_long_name",
                "?physical_mode",
                "?gtfs_id",
            ],
            &format!(
                "?route wdt:{instance_of} wd:{route}.
                 ?route wdt:{gtfs_id_prop} \"{gtfs_id}\".
                 ?route wdt:{data_source} ?data_source.
                 ?data_source wdt:{producer_prop} wd:{producer_id}.
                 ",
                instance_of = self.config.properties.instance_of,
                route = self.config.items.route,
                gtfs_id_prop = self.config.properties.gtfs_id,
                producer_prop = self.config.properties.produced_by,
                data_source = self.config.properties.data_source,
                gtfs_id = gtfs_id,
                producer_id = producer_id,
            ),
        )
    }

    pub fn find_stop(
        &self,
        producer_id: &str,
        stop: &gtfs_structures::Stop,
    ) -> Result<Vec<HashMap<String, String>>, SparqlError> {
        log::trace!(
            "Finding stop {} {} of producer {}",
            stop.name,
            stop.id,
            producer_id
        );
        self.sparql(
            &["?stop", "?stopLabel", "?stopName", "?gtfs_id"],
            &format!(
                "?stop wdt:{instance_of} wd:{stop_type}.
                 ?stop wdt:{gtfs_id_prop} \"{gtfs_id}\".
                 ?stop wdt:{data_source} ?data_source.
                 ?data_source wdt:{producer_prop} wd:{producer_id}.
                 ?stop wdt:{gtfs_name} ?stop_name.",
                instance_of = self.config.properties.instance_of,
                stop_type = self.config.location_type(stop),
                gtfs_id_prop = self.config.properties.gtfs_id,
                producer_prop = self.config.properties.produced_by,
                gtfs_name = self.config.properties.gtfs_name,
                data_source = self.config.properties.data_source,
                gtfs_id = stop.id,
                producer_id = producer_id,
            ),
        )
    }

    /// Finds an entity id with a given topo_id
    /// Will fail if no item or strictly more than one is returned
    /// You must provide the id of the `topo tool id` property
    pub fn find_entity_by_topo_id(&self, item_topo_id: &str) -> Result<String, SparqlError> {
        self.sparql(
            &["?item_id"],
            &format!(
                "?item_id wdt:{topo_id_id} '{item_topo_id}'",
                topo_id_id = self.config.properties.topo_id_id,
                item_topo_id = item_topo_id
            ),
        )
        .and_then(|items| match items.as_slice() {
            [] => Err(SparqlError::TopoIdNotFound(item_topo_id.to_string())),
            [item] => Ok(item["item_id"].to_owned()),
            _ => Err(SparqlError::DuplicatedTopoId(item_topo_id.to_string())),
        })
        .and_then(|id| {
            read_id_from_url(&id)
                .ok_or_else(|| SparqlError::TopoInvalidId(id, item_topo_id.to_string()))
        })
    }

    pub fn get_producer_label(&self, producer_id: &str) -> Result<Option<String>, SparqlError> {
        self.sparql(
            &["?label"],
            &format!(
                "wd:{producer_id} wdt:{instance_of} wd:{producer};
                                  rdfs:label ?label.",
                producer_id = producer_id,
                instance_of = self.config.properties.instance_of,
                producer = self.config.items.producer
            ),
        )
        .and_then(|mut items| match items.as_mut_slice() {
            [] => Ok(None),
            [item] => Ok(item.remove("label")),
            _ => Err(SparqlError::Duplicate(producer_id.to_string())),
        })
    }

    pub fn get_producer_id(&self, producer_label: &str) -> Result<Option<String>, SparqlError> {
        self.sparql(
            &["?producer"],
            &format!(
                r#"?producer wdt:{instance_of} wd:{producer};
                           rdfs:label "{label}"@en "#,
                label = producer_label,
                instance_of = self.config.properties.instance_of,
                producer = self.config.items.producer
            ),
        )
        .and_then(|mut items| match items.as_mut_slice() {
            [] => Ok(None),
            [item] => Ok(item.get("producer").and_then(|u| read_id_from_url(u))),
            _ => Err(SparqlError::Duplicate(producer_label.to_string())),
        })
    }

    pub fn get_id_by_topo_id(&self, topo_id: &str) -> Result<Option<String>, SparqlError> {
        self.sparql(
            &["?item"],
            &format!(
                r#"?item wdt:{topo_id_prop} "{topo_id}"."#,
                topo_id = topo_id,
                topo_id_prop = self.config.properties.topo_id_id,
            ),
        )
        .and_then(|mut items| match items.as_mut_slice() {
            [] => Ok(None),
            [item] => Ok(item.get("item").and_then(|u| read_id_from_url(u))),
            _ => Err(SparqlError::Duplicate(topo_id.to_string())),
        })
    }
}
