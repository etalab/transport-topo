//! Some utilities wikibase queries to ease tests
use crate::utils::DockerContainerWrapper;
use transitwiki::api_client::ObjectType;

pub struct Wikibase {
    client: transitwiki::Client,
}

impl Wikibase {
    pub fn new(docker: &DockerContainerWrapper) -> Self {
        Self {
            client: transitwiki::Client::new(&docker.api_endpoint, &docker.sparql_endpoint, "P1")
                .expect("impossible to create wikibase client"),
        }
    }

    pub fn exists(&self, object_type: ObjectType, entity: &str) -> bool {
        self.client
            .api
            .find_entity_id(object_type, entity)
            .expect("invalid wikibase query")
            .is_some()
    }

    pub fn count_by_producer(&self, producer_id: &str) -> usize {
        let r = self
            .client
            .sparql
            .sparql(
                &["(COUNT(?x) as ?count)"],
                &format!(
                    "?x wdt:P3 wd:{}", // TODO remove the P3 hardcoding
                    producer_id
                ),
            )
            .expect("invalid sparql query");

        r[0]["count"].parse().expect("impossible to parse value")
    }

    pub fn gtfs_id_by_producer(&self, producer_id: &str) -> std::collections::HashSet<String> {
        let r = self
            .client
            .sparql
            .sparql(
                &["?gtfs_id"],
                &format!(
                    // TODO remove the property id hardcoding
                    r#"?x wdt:P3 wd:{}.
                    ?x wdt:P7 ?gtfs_id"#, producer_id
                ),
            )
            .expect("invalid sparql query");

        r.into_iter()
            .map(|hashmap| {
                hashmap["gtfs_id"]
                    .split("/")
                    .collect::<Vec<_>>()
                    .last()
                    .expect("invalid id")
                    .to_string()
            })
            .collect()
    }

    /// get all objects with a topo id
    pub fn get_topo_objects(&self) -> std::collections::HashSet<String> {
        let r = self
            .client
            .sparql
            .sparql(
                &["?topo_id"],
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
