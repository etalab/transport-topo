//! Some utilities wikibase queries to ease tests
use crate::utils::DockerContainerWrapper;
use transitwiki::api_client::ObjectType;

pub struct Wikibase {
    client: transitwiki::Client,
}

impl Wikibase {
    pub fn new(docker: &DockerContainerWrapper) -> Self {
        let config = transitwiki::client::Config {
            api_endpoint: docker.api_endpoint.clone(),
            sparql_endpoint: docker.sparql_endpoint.clone(),
            ..Default::default()
        };
        Self {
            client: transitwiki::Client::new(config).expect("impossible to create wikibase client"),
        }
    }

    pub fn exists(&self, object_type: ObjectType, entity: &str) -> bool {
        self.client
            .api
            .find_entity_id(object_type, entity)
            .expect("invalid wikibase query")
            .is_some()
    }

    /// get all objects with a topo id
    pub fn get_topo_objects(&self) -> std::collections::HashSet<String> {
        let r = self
            .client
            .sparql
            .sparql(
                &["topo_id"],
                &format!(
                    "?x wdt:{topo_id} ?topo_id",
                    topo_id = "P1" //self.client.sparql.config.items.producer TODO remove this hardcoding
                ),
            )
            .expect("invalid sparql query");

        r.into_iter()
            .map(|hashmap| hashmap["topo_id"].clone())
            .collect()
    }
}
