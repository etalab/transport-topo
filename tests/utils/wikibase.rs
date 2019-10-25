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

            // hardcode the id for the moment
            // TODO remove this
            properties: transitwiki::client::Properties {
                instance_of: "P2".to_owned(),
                produced_by: "P3".to_owned(),
                physical_mode: "P4".to_owned(),
                gtfs_short_name: "P5".to_owned(),
                gtfs_long_name: "P6".to_owned(),
                gtfs_id: "P7".to_owned(),
            },
            items: transitwiki::client::Items {
                producer: "Q1".to_owned(),
                line: "Q2".to_owned(),
                bus: "Q3".to_owned(),
            },
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
