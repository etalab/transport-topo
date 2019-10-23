use crate::client::{Config, Properties, Items};
use failure::Error;

pub fn initial_populate(api_endpoint: &str, sparql_endpoint: &str) -> Result<Config, Error> {
    let client = crate::api_client::ApiClient::new(Config {
        api_endpoint: api_endpoint.to_owned(),
        sparql_endpoint: sparql_endpoint.to_owned(),
        ..Default::default()
    })?;
    Ok(Config {
        api_endpoint: api_endpoint.to_owned(),
        sparql_endpoint: sparql_endpoint.to_owned(),
        properties: Properties {
            produced_by: client.create_property("produced by", &[])?,
            instance_of: client.create_property("instance of", &[])?,
            physical_mode: client.create_property("physical mode", &[])?,
            gtfs_short_name: client.create_property("gtfs short name", &[])?,
            gtfs_long_name: client.create_property("gtfs long name", &[])?,
            gtfs_id: client.create_property("gtfs id", &[])?,
        },
        items: Items {
            ..Default::default() //TODO
        },
    })
}
