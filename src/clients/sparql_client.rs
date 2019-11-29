use itertools::Itertools;
use std::collections::HashMap;

pub fn read_id_from_url(url: &str) -> Option<String> {
    url.split('/')
        .collect::<Vec<_>>()
        .last()
        .map(|id| id.to_string())
}

pub struct SparqlClient {
    client: reqwest::Client,
    endpoint: String,
}

impl SparqlClient {
    /// create a new client and discover all the base entities id
    pub fn new(endpoint: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.to_owned(),
        }
    }

    fn query(&self, query: &str) -> Result<serde_json::Value, anyhow::Error> {
        log::debug!("Sparql query: {}", query);
        let response = self
            .client
            .get(&self.endpoint)
            .query(&[("format", "json"), ("query", query)])
            .send()?
            .error_for_status()?
            .text()?;
        log::trace!("Query response: {:?}", response);
        Ok(serde_json::from_str(&response)?)
    }

    pub fn sparql(
        &self,
        variables: &[&str],
        where_clause: &str,
    ) -> Result<Vec<HashMap<String, String>>, anyhow::Error> {
        let vars = variables.iter().format(" ");
        let query = format!("SELECT {} WHERE {{ {} SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }} }}", vars, where_clause);
        let res = self.query(&query)?;

        let mut result = Vec::new();
        for binding in res
            .pointer("/results/bindings")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("invalid json, no bindings"))?
        {
            let values = binding
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("invalid json, bindings badly formated"))?
                .iter()
                .map(|(k, v)| (k.to_string(), v["value"].as_str().unwrap_or("").into()))
                .collect();
            result.push(values);
        }
        Ok(result)
    }
}
