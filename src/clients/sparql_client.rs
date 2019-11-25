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
}

impl SparqlClient {
    /// create a new client and discory all the base entities id
    pub fn new(endpoint: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.to_owned(),
        }
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
}
