use crate::client::{Config, Properties, Items};
use failure::Error;

pub fn initial_populate(api_endpoint: String, sparql_endpoint: String) -> Result<Config, Error> {

    let mut client = crate::api_client::ApiClient::new(Config {
        api_endpoint: api_endpoint.clone(),
        sparql_endpoint: sparql_endpoint.clone(),
        ..Default::default()
    });
    Ok(Config {
        api_endpoint,
        sparql_endpoint,
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
