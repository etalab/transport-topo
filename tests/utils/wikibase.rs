//! Some utilities wikibase queries to ease tests
use crate::utils::DockerContainerWrapper;
use std::collections::BTreeSet;
use transit_topo::clients::sparql_client::read_id_from_url;

pub struct Wikibase {
    pub client: transit_topo::Client,
}

#[derive(Hash, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DataSourceItem {
    pub id: String,
    pub label: String,
    pub gtfs_id: Option<String>,
    pub instance_of: String,
}

impl Wikibase {
    pub fn new(docker: &DockerContainerWrapper) -> Self {
        Self {
            client: transit_topo::Client::new(&docker.api_endpoint, &docker.sparql_endpoint, "P1")
                .expect("impossible to create wikibase client"),
        }
    }

    pub fn properties(&self) -> &transit_topo::known_entities::Properties {
        &self.client.sparql.known_entities.properties
    }

    pub fn items(&self) -> &transit_topo::known_entities::Items {
        &self.client.sparql.known_entities.items
    }

    pub fn get_entity_id(&self, label: &str) -> Option<String> {
        match self
            .client
            .sparql
            .sparql(
                &["?item"],
                &format!(r#"?item rdfs:label "{label}"@en"#, label = label,),
            )
            .expect("impossible to call sparql")
            .as_mut_slice()
        {
            [] => None,
            [e] => read_id_from_url(&e["item"]),
            val => panic!("too many value for entity {} : {:?} ", label, val),
        }
    }
    pub fn exists(&self, entity: &str) -> bool {
        self.get_entity_id(entity).is_some()
    }

    pub fn get_entity(&self, item: &str) -> transit_topo::entity::Entity {
        self.client
            .api
            .get_entity(item)
            .expect("impossible to find entity")
    }

    pub fn get_all_items_for_datasource(&self, data_source_id: &str) -> BTreeSet<DataSourceItem> {
        let prop = &self.client.sparql.known_entities.properties;
        let r = self
            .client
            .sparql
            .sparql(
                &["?gtfs_id ?item ?item_label ?type_label"],
                &format!(
                    r#"
                        ?item wdt:{from} wd:{data_source};
                              rdfs:label ?item_label.
                        OPTIONAL {{
                            ?item wdt:{gtfs_id} ?gtfs_id;
                                  wdt:{instance_of} ?type.
                            ?type rdfs:label ?type_label.
                        }}
                    "#,
                    data_source = data_source_id,
                    from = prop.data_source,
                    gtfs_id = prop.gtfs_id,
                    instance_of = prop.instance_of,
                ),
            )
            .expect("invalid sparql query");

        r.into_iter()
            .map(|res| DataSourceItem {
                id: read_id_from_url(&res["item"]).expect("no id"),
                label: res["item_label"].clone(),
                gtfs_id: res.get("gtfs_id").cloned(),
                instance_of: res["type_label"].clone(),
            })
            .collect()
    }

    pub fn get_producer_datasources_id(&self, producer_id: &str) -> BTreeSet<String> {
        let prop = &self.client.sparql.known_entities.properties;
        let r = self
            .client
            .sparql
            .sparql(
                &["?data_source"],
                &format!(
                    "?data_source wdt:{produced_by} wd:{producer}.",
                    produced_by = prop.produced_by,
                    producer = producer_id
                ),
            )
            .expect("invalid sparql query");

        r.into_iter()
            .map(|hashmap| read_id_from_url(&hashmap["data_source"]).expect("invalid id"))
            .collect()
    }

    /// get all objects with a topo id
    pub fn get_topo_objects(&self) -> BTreeSet<String> {
        let r = self
            .client
            .sparql
            .sparql(
                &["?topo_id"],
                &format!(
                    "?x wdt:{topo_id} ?topo_id",
                    topo_id = "P1" //self.client.sparql.known_entities.items.producer TODO remove this hardcoding
                ),
            )
            .expect("invalid sparql query");

        r.into_iter()
            .map(|hashmap| hashmap["topo_id"].clone())
            .collect()
    }
}
