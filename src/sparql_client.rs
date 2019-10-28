use anyhow::Error;
use itertools::Itertools;
use json;
use log::{debug, trace};
use std::collections::HashMap;

pub struct SparqlClient {
    client: reqwest::Client,
    config: crate::client::Config,
}

impl SparqlClient {
    pub fn new(config: crate::client::Config) -> Self {
        SparqlClient {
            client: reqwest::Client::new(),
            config,
        }
    }
    fn query(&self, query: &str) -> Result<json::JsonValue, Error> {
        debug!("Sparql query: {}", query);
        let response = self
            .client
            .get(&self.config.sparql_endpoint)
            .query(&[("format", "json"), ("query", query)])
            .send()?
            .text()?;
        debug!("Query response: {:?}", response);
        Ok(json::parse(&response)?)
    }

    pub fn sparql(
        &self,
        variables: &[&str],
        where_clause: &str,
    ) -> Result<Vec<HashMap<String, String>>, Error> {
        let vars = variables.iter().map(|var| format!("?{}", var)).format(" ");
        let query = format!("SELECT {} WHERE {{ {} SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }} }}", vars, where_clause);
        let res = self.query(&query)?;

        let mut result = Vec::new();
        for binding in res["results"]["bindings"].members() {
            let mut values = HashMap::new();
            for &var in variables {
                values.insert(
                    var.to_string(),
                    binding[var]["value"].as_str().unwrap_or("").into(),
                );
            }
            result.push(values);
        }
        Ok(result)
    }

    pub fn find_line(
        &self,
        producer_id: &str,
        gtfs_id: &str,
    ) -> Result<Vec<HashMap<String, String>>, Error> {
        trace!("Finding line {} of producer {}", gtfs_id, producer_id);
        self.sparql(
            &[
                "line",
                "lineLabel",
                "route_short_name",
                "route_long_name",
                "physical_mode",
                "gtfs_id",
            ],
            &format!(
                "?line wdt:{instance_of} wd:{line}.
    ?line wdt:{gtfs_id_prop} \"{gtfs_id}\".
    ?line wdt:{producer_prop} wd:{producer_id}.
    ?line wdt:{route_short_name} ?route_short_name.
    ?line wdt:{route_long_name} ?route_long_name.
    ?line wdt:{physical_mode} ?physical_mode.",
                instance_of = self.config.properties.instance_of,
                line = self.config.items.line,
                gtfs_id_prop = self.config.properties.gtfs_id,
                producer_prop = self.config.properties.produced_by,
                route_short_name = self.config.properties.gtfs_short_name,
                route_long_name = self.config.properties.gtfs_long_name,
                physical_mode = self.config.properties.physical_mode,
                gtfs_id = gtfs_id,
                producer_id = producer_id
            ),
        )
    }

    pub fn find_producer(&self, producer_id: &str) -> Result<Vec<HashMap<String, String>>, Error> {
        trace!("Finding producer {}", producer_id);
        self.sparql(
            &["producer", "producerLabel"],
            &format!(
                "?producer wdt:{instance_of} wd:{producer}.
                FILTER(?producer = wd:{producer_id})",
                producer_id = producer_id,
                instance_of = self.config.properties.instance_of,
                producer = self.config.items.producer
            ),
        )
    }
}
