use crate::api_client::{claim_string, ApiClient, ApiError};
use crate::client::{Config, Items, Properties};
use anyhow::Error;

// insert item if not already there, and return its id
fn get_or_create_item(
    client: &ApiClient,
    label: &str,
    claims: &[json::JsonValue],
) -> Result<u64, Error> {
    let r = client.create_item(label, claims);
    let id = if let Err(ApiError::AlreadyExists { label, id }) = r {
        log::info!("object \"{}\" already exists with id {}", label, id);
        id
    } else {
        r?
    };
    Ok(id.trim_start_matches('Q').parse()?)
}

fn get_or_create_property(client: &ApiClient, label: &str) -> Result<String, Error> {
    let r = client.create_property(label, &[]);
    if let Err(ApiError::AlreadyExists { label, id }) = r {
        log::info!("object \"{}\" already exists with id {}", label, id);
        Ok(id)
    } else {
        Ok(r?)
    }
}

pub fn initial_populate(
    api_endpoint: &str,
    sparql_endpoint: &str,
    default_producer: bool,
) -> Result<Config, Error> {
    let client = ApiClient::new(Config {
        api_endpoint: api_endpoint.to_owned(),
        sparql_endpoint: sparql_endpoint.to_owned(),
        ..Default::default()
    })?;

    let producer_class = get_or_create_item(&client, "producer", &[])?;
    let instance_of = get_or_create_property(&client, "instance of")?;
    let config = Config {
        api_endpoint: api_endpoint.to_owned(),
        sparql_endpoint: sparql_endpoint.to_owned(),
        properties: Properties {
            produced_by: get_or_create_property(&client, "produced by")?,
            instance_of: instance_of.clone(),
            physical_mode: get_or_create_property(&client, "physical mode")?,
            gtfs_short_name: get_or_create_property(&client, "gtfs short name")?,
            gtfs_long_name: get_or_create_property(&client, "gtfs long name")?,
            gtfs_id: get_or_create_property(&client, "gtfs id")?,
        },
        items: Items {
            line: get_or_create_item(&client, "line", &[])?,
            producer: producer_class,
            bus: get_or_create_item(&client, "bus", &[])?,
        },
    };
    if default_producer {
        // we create a default producer, useful for tests purposes
        let r = client.create_item(
            "bob the bus mapper",
            &[claim_string(&instance_of, &format!("Q{}", producer_class))],
        )?;
        log::info!("producer created: {:?}", r);
    }
    Ok(config)
}
