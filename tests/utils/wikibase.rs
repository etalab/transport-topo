//! Some utilities wikibase queries to ease tests
use crate::utils::DockerContainerWrapper;

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

    pub fn exists(&self, entity: &str) -> bool {
        self.client
            .api
            .find_entity_id(entity)
            .expect("invalid wikibase query")
            .is_some()
    }
}
