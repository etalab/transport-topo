use crate::api_client::{
    claim_item, claim_string, ApiClient, ApiError, ObjectType, PropertyDataType,
};
use crate::client::{EntitiesId, Items, Properties};
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

pub fn initial_populate(api_endpoint: &str, default_producer: bool) -> Result<EntitiesId, Error> {
    let client = ApiClient::new(api_endpoint, Default::default())?;

    let topo_id = get_or_create_property(&client, "Topo tools id", PropertyDataType::String, None)?;

    let producer_class = get_or_create_item(&client, "Producer", &[], topo_id.as_str())?;
    let gtfs_id = get_or_create_property(
        &client,
        "GTFS id",
        PropertyDataType::String,
        topo_id.as_str(),
    )?;
    let physical_mode = get_or_create_item(&client, "Physical mode", &[], topo_id.as_str())?;
    let instance_of = get_or_create_property(
        &client,
        "Instance of",
        PropertyDataType::Item,
        topo_id.as_str(),
    )?;
    let is_physical_mode_claim = claim_item(&instance_of, &physical_mode);
    let config = EntitiesId {
        properties: Properties {
            topo_id_id: topo_id.to_owned(),
            produced_by: get_or_create_property(
                &client,
                "Produced by",
                PropertyDataType::Item,
                topo_id.as_str(),
            )?,
            instance_of: instance_of.clone(),
            physical_mode: physical_mode.to_owned(),
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
            gtfs_id: gtfs_id.to_owned(),
            has_physical_mode: get_or_create_property(
                &client,
                "Has physical mode",
                PropertyDataType::Item,
                topo_id.as_str(),
            )?,
            first_seen_in: get_or_create_property(
                &client,
                "First seen in",
                PropertyDataType::Item,
                topo_id.as_str(),
            )?,
            data_source: get_or_create_property(
                &client,
                "Data source",
                PropertyDataType::Item,
                topo_id.as_str(),
            )?,

            file_link: get_or_create_property(
                &client,
                "file link", //Link to the raw file
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
            file_format: get_or_create_property(
                &client,
                "file format",
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
            content_id: get_or_create_property(
                &client,
                "content id", //Checksum of the file
                PropertyDataType::String,
                topo_id.as_str(),
            )?,
        },
        items: Items {
            route: get_or_create_item(&client, "route", &[], topo_id.as_str())?,
            tramway: get_or_create_item(
                &client,
                "Tramway",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "0")],
                topo_id.as_str(),
            )?,
            subway: get_or_create_item(
                &client,
                "Subway",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "1")],
                topo_id.as_str(),
            )?,
            railway: get_or_create_item(
                &client,
                "Railway",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "2")],
                topo_id.as_str(),
            )?,
            bus: get_or_create_item(
                &client,
                "Bus",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "3")],
                topo_id.as_str(),
            )?,
            ferry: get_or_create_item(
                &client,
                "Ferry",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "4")],
                topo_id.as_str(),
            )?,
            cable_car: get_or_create_item(
                &client,
                "Cable car",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "5")],
                topo_id.as_str(),
            )?,
            gondola: get_or_create_item(
                &client,
                "Gondola",
                &[is_physical_mode_claim.clone(), claim_string(&gtfs_id, "6")],
                topo_id.as_str(),
            )?,
            funicular: get_or_create_item(
                &client,
                "Funicular",
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
