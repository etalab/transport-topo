use crate::clients::sparql_client::{read_id_from_url, SparqlClient};
use crate::known_entities::{EntitiesId, Items, Properties};
use anyhow::Context;
use thiserror::Error;

pub struct TopoQuery {
    pub client: SparqlClient,
    pub known_entities: EntitiesId,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("No entity with topo id {0} found")]
    TopoIdNotFound(String),
    #[error("Several entities with topo id {0}")]
    DuplicatedTopoId(String),
    #[error("sparql error: {0}")]
    ReqwestError(#[from] anyhow::Error),
    #[error("Error parsing the id {0} for entity with topo id {1}")]
    TopoInvalidId(String, String),
    #[error("Too many elements {0}")]
    Duplicate(String),
}

impl TopoQuery {
    /// create a new TopoQuery and discover all the known entities id
    pub fn new(endpoint: &str, topo_id_id: &str) -> Result<Self, anyhow::Error> {
        let client = SparqlClient::new(endpoint);
        let known_entities = discover_known_entities(&client, topo_id_id)
            .context("impossible to discover config")?;
        Ok(Self {
            client,
            known_entities,
        })
    }

    pub fn find_route(
        &self,
        producer_id: &str,
        gtfs_id: &str,
    ) -> Result<Option<String>, QueryError> {
        log::trace!("Finding route {} of producer {}", gtfs_id, producer_id);
        let items = self.client.sparql(
            &["?route", "?routeLabel", "?gtfs_id"],
            &format!(
                "?route wdt:{instance_of} wd:{route}.
                 ?route wdt:{gtfs_id_prop} \"{gtfs_id}\".
                 ?route wdt:{data_source} ?data_source.
                 ?data_source wdt:{producer_prop} wd:{producer_id}.
                 ",
                instance_of = self.known_entities.properties.instance_of,
                route = self.known_entities.items.route,
                gtfs_id_prop = self.known_entities.properties.gtfs_id,
                producer_prop = self.known_entities.properties.produced_by,
                data_source = self.known_entities.properties.data_source,
                gtfs_id = gtfs_id,
                producer_id = producer_id,
            ),
        )?;

        match items.as_slice() {
            [] => Ok(None),
            [item] => Ok(item.get("route").and_then(|u| read_id_from_url(u))),
            _ => Err(QueryError::Duplicate(format!(
                "Route “{}” exists many times. Something is not right",
                gtfs_id
            ))),
        }
    }

    pub fn find_stop(
        &self,
        producer_id: &str,
        stop: &gtfs_structures::Stop,
    ) -> Result<Option<String>, QueryError> {
        log::trace!(
            "Finding stop {} {} of producer {}",
            stop.name,
            stop.id,
            producer_id
        );
        let items = self.client.sparql(
            &["?stop", "?stopLabel", "?stopName", "?gtfs_id"],
            &format!(
                "?stop wdt:{instance_of} wd:{stop_type}.
                 ?stop wdt:{gtfs_id_prop} \"{gtfs_id}\".
                 ?stop wdt:{data_source} ?data_source.
                 ?data_source wdt:{producer_prop} wd:{producer_id}.
                 ?stop wdt:{gtfs_name} ?stop_name.",
                instance_of = self.known_entities.properties.instance_of,
                stop_type = self.known_entities.location_type(stop),
                gtfs_id_prop = self.known_entities.properties.gtfs_id,
                producer_prop = self.known_entities.properties.produced_by,
                gtfs_name = self.known_entities.properties.gtfs_name,
                data_source = self.known_entities.properties.data_source,
                gtfs_id = stop.id,
                producer_id = producer_id,
            ),
        )?;

        match items.as_slice() {
            [] => Ok(None),
            [item] => Ok(item.get("stop").and_then(|u| read_id_from_url(u))),
            _ => Err(QueryError::Duplicate(format!(
                "Stop “{}” exists many times. Something is not right",
                stop.id
            ))),
        }
    }

    pub fn get_producer_label(&self, producer_id: &str) -> Result<Option<String>, QueryError> {
        let mut items = self.client.sparql(
            &["?label"],
            &format!(
                "wd:{producer_id} wdt:{instance_of} wd:{producer};
                                  rdfs:label ?label.",
                producer_id = producer_id,
                instance_of = self.known_entities.properties.instance_of,
                producer = self.known_entities.items.producer
            ),
        )?;
        match items.as_mut_slice() {
            [] => Ok(None),
            [item] => Ok(item.remove("label")),
            _ => Err(QueryError::Duplicate(producer_id.to_string())),
        }
    }
}

/// Finds an entity id with a given topo_id
/// Will fail if no item or strictly more than one is returned
/// You must provide the id of the `topo tool id` property
fn find_entity_by_topo_id(
    client: &SparqlClient,
    item_topo_id: &str,
    topo_id_id: &str,
) -> Result<String, QueryError> {
    let items = client.sparql(
        &["?item_id"],
        &format!(
            "?item_id wdt:{topo_id_id} '{item_topo_id}'",
            topo_id_id = topo_id_id,
            item_topo_id = item_topo_id
        ),
    )?;

    let id = match items.as_slice() {
        [] => Err(QueryError::TopoIdNotFound(item_topo_id.to_string())),
        [item] => Ok(item["item_id"].to_owned()),
        _ => Err(QueryError::DuplicatedTopoId(item_topo_id.to_string())),
    }?;
    read_id_from_url(&id).ok_or_else(|| QueryError::TopoInvalidId(id, item_topo_id.to_string()))
}

fn discover_known_entities(
    client: &SparqlClient,
    topo_id_id: &str,
) -> Result<EntitiesId, anyhow::Error> {
    Ok(EntitiesId {
        items: Items {
            physical_mode: find_entity_by_topo_id(client, "physical_mode", topo_id_id)?,
            route: find_entity_by_topo_id(client, "route", topo_id_id)?,
            producer: find_entity_by_topo_id(client, "producer", topo_id_id)?,
            tramway: find_entity_by_topo_id(client, "tramway", topo_id_id)?,
            subway: find_entity_by_topo_id(client, "subway", topo_id_id)?,
            railway: find_entity_by_topo_id(client, "railway", topo_id_id)?,
            bus: find_entity_by_topo_id(client, "bus", topo_id_id)?,
            ferry: find_entity_by_topo_id(client, "ferry", topo_id_id)?,
            cable_car: find_entity_by_topo_id(client, "cable_car", topo_id_id)?,
            gondola: find_entity_by_topo_id(client, "gondola", topo_id_id)?,
            funicular: find_entity_by_topo_id(client, "funicular", topo_id_id)?,
            stop_point: find_entity_by_topo_id(client, "stop_point", topo_id_id)?,
            stop_area: find_entity_by_topo_id(client, "stop_area", topo_id_id)?,
            stop_entrance: find_entity_by_topo_id(client, "stop_entrance", topo_id_id)?,
            stop_generic_node: find_entity_by_topo_id(client, "stop_generic_node", topo_id_id)?,
            stop_boarding_area: find_entity_by_topo_id(client, "stop_boarding_area", topo_id_id)?,
        },
        properties: Properties {
            topo_id_id: topo_id_id.to_string(),
            produced_by: find_entity_by_topo_id(client, "produced_by", topo_id_id)?,
            instance_of: find_entity_by_topo_id(client, "instance_of", topo_id_id)?,
            gtfs_short_name: find_entity_by_topo_id(client, "gtfs_short_name", topo_id_id)?,
            gtfs_long_name: find_entity_by_topo_id(client, "gtfs_long_name", topo_id_id)?,
            gtfs_name: find_entity_by_topo_id(client, "gtfs_name", topo_id_id)?,
            gtfs_id: find_entity_by_topo_id(client, "gtfs_id", topo_id_id)?,
            first_seen_in: find_entity_by_topo_id(client, "first_seen_in", topo_id_id)?,
            data_source: find_entity_by_topo_id(client, "data_source", topo_id_id)?,
            source: find_entity_by_topo_id(client, "source", topo_id_id)?,
            file_format: find_entity_by_topo_id(client, "file_format", topo_id_id)?,
            sha_256: find_entity_by_topo_id(client, "sha_256", topo_id_id)?,
            has_physical_mode: find_entity_by_topo_id(client, "has_physical_mode", topo_id_id)?,
            tool_version: find_entity_by_topo_id(client, "tool_version", topo_id_id)?,
            part_of: find_entity_by_topo_id(client, "part_of", topo_id_id)?,
            connecting_line: find_entity_by_topo_id(client, "connecting_line", topo_id_id)?,
            coordinate_location: find_entity_by_topo_id(client, "coordinate_location", topo_id_id)?,
        },
    })
}
