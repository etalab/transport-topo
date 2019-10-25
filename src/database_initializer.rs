use crate::api_client::{
    claim_item, claim_string, ApiClient, ApiError, ObjectType, PropertyDataType,
};
use crate::client::{Config, Items, Properties};
use anyhow::Error;
use inflector::Inflector;

// insert item if not already there, and return its id
fn get_or_create_item<'a>(
    client: &ApiClient,
    label: &str,
    claims: &[json::JsonValue],
    topo_id_id: impl Into<Option<&'a str>>,
) -> Result<String, Error> {
    let mut claims = Vec::from(claims);
    if let Some(id) = topo_id_id.into() {
        claims.push(claim_string(id, &label.to_snake_case()))
    };

    // for an item, we need to do a separate query to check if the item is already there
    let id = if let Some(id) = client.find_entity_id(ObjectType::Item, label)? {
        log::info!("item \"{}\" already exists with id {}", label, id);
        id
    } else {
        let id = client.create_item(label, &claims)?;
        log::info!("creating item \"{}\" with id {}", label, id);
        id
    };
    Ok(id.to_owned())
}

fn get_or_create_property<'a>(
    client: &ApiClient,
    label: &str,
    prop_type: PropertyDataType,
    topo_id_id: impl Into<Option<&'a str>>,
) -> Result<String, Error> {
    let claims = match topo_id_id.into() {
        Some(id) => vec![claim_string(id, &label.to_snake_case())],
        None => vec![],
    };
    // 2 properties cannot have the same label, so we just try to insert it
    // and get the id of the conflicting property if present
    let r = client.create_object(ObjectType::Property(prop_type), label, &claims);
    if let Err(ApiError::PropertyAlreadyExists { label, id }) = r {
        log::info!("property \"{}\" already exists with id {}", label, id);
        Ok(id)
    } else {
        let id = r?;
        log::info!("creating property \"{}\" with id {}", label, id);
        Ok(id)
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

    let topo_id = get_or_create_property(&client, "Topo tools id", PropertyDataType::String, None)?;

    let producer_class = get_or_create_item(&client, "producer", &[], topo_id.as_str())?;
    let instance_of = get_or_create_property(
        &client,
        "instance of",
        PropertyDataType::Item,
        topo_id.as_str(),
    )?;
    let config = Config {
        api_endpoint: api_endpoint.to_owned(),
        sparql_endpoint: sparql_endpoint.to_owned(),
        properties: Properties {
            produced_by: get_or_create_property(
                &client,
                "Produced by",
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
            instance_of: instance_of.clone(),
            physical_mode: get_or_create_property(
                &client,
                "Physical mode",
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
            gtfs_short_name: get_or_create_property(
                &client,
                "GTFS short name",
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
            gtfs_long_name: get_or_create_property(
                &client,
                "GTFS long name",
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
            gtfs_id: get_or_create_property(
                &client,
                "GTFS id",
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
        },
        items: Items {
            line: get_or_create_item(&client, "line", &[], topo_id.as_str())?,
            producer: producer_class.to_owned(),
            bus: get_or_create_item(&client, "bus", &[], topo_id.as_str())?,
        },
    };
    if default_producer {
        // we create a default producer, useful for tests purposes
        get_or_create_item(
            &client,
            "bob the bus mapper",
            &[claim_item(&instance_of, &producer_class)],
            topo_id.as_str(),
        )?;
    }

    Ok(config)
}
