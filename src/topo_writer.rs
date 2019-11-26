use crate::clients::api_client::{claim_item, claim_string, ApiClient};
use crate::clients::ObjectType;
use crate::known_entities::EntitiesId;
use anyhow::Context;

pub struct TopoWriter {
    pub client: ApiClient,
    pub known_entities: EntitiesId,
}

impl TopoWriter {
    pub fn new(endpoint: &str, known_entities: EntitiesId) -> Result<Self, anyhow::Error> {
        Ok(Self {
            client: ApiClient::new(endpoint).context("impossible to create api client")?,
            known_entities,
        })
    }

    pub fn insert_data_source(
        &self,
        sha_256: &Option<String>,
        producer: &str,
        path: &str,
    ) -> Result<String, anyhow::Error> {
        let dt = chrono::Utc::now();
        let label = format!("Data source for {} - imported {}", &producer, dt);

        let mut claims = vec![
            claim_item(&self.known_entities.properties.produced_by, producer),
            claim_string(&self.known_entities.properties.source, path),
            claim_string(&self.known_entities.properties.file_format, "GTFS"),
            claim_string(
                &self.known_entities.properties.tool_version,
                crate::GIT_VERSION,
            ),
        ];
        if let Some(sha) = sha_256 {
            claims.push(claim_string(&self.known_entities.properties.sha_256, sha));
        }

        self.client
            .create_object(ObjectType::Item, &label, claims)
            .context("impossible to insert data source")
    }

    pub fn insert_route(
        &self,
        route: &gtfs_structures::Route,
        data_source_id: &str,
        producer_name: &str,
    ) -> Result<String, anyhow::Error> {
        let route_name = if !route.long_name.is_empty() {
            route.long_name.as_str()
        } else {
            route.short_name.as_str()
        };

        let label = format!("{:?} {} ({})", route.route_type, route_name, producer_name);
        let claims = vec![
            claim_item(
                &self.known_entities.properties.instance_of,
                &self.known_entities.items.route,
            ),
            claim_string(&self.known_entities.properties.gtfs_id, &route.id),
            claim_item(&self.known_entities.properties.data_source, data_source_id),
            claim_string(
                &self.known_entities.properties.gtfs_short_name,
                &route.short_name,
            ),
            claim_string(
                &self.known_entities.properties.gtfs_long_name,
                &route.long_name,
            ),
            claim_item(
                &self.known_entities.properties.has_physical_mode,
                self.known_entities.physical_mode(route),
            ),
        ];

        self.client
            .create_object(ObjectType::Item, &label, claims)
            .context("impossible to insert route")
    }

    pub fn insert_stop(
        &self,
        stop: &gtfs_structures::Stop,
        data_source_id: &str,
    ) -> Result<String, anyhow::Error> {
        let claims = vec![
            claim_item(
                &self.known_entities.properties.instance_of,
                &self.known_entities.location_type(stop),
            ),
            claim_string(&self.known_entities.properties.gtfs_id, &stop.id),
            claim_item(&self.known_entities.properties.data_source, data_source_id),
            claim_string(&self.known_entities.properties.gtfs_name, &stop.name),
        ];

        self.client
            .create_object(ObjectType::Item, &stop.name, claims)
            .context("impossible to insert stop")
    }
}
