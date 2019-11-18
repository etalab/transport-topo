use crate::api_client::{
    claim_item, claim_string, ApiClient, ApiError, ObjectType, PropertyDataType,
};
use crate::client::Client;
use crate::known_entities::{EntitiesId, Items, Properties};
use anyhow::Error;
use inflector::Inflector;

// insert item if not already there, and return its id
fn get_or_create_item(
    client: &Client,
    label: &str,
    claims: &[Option<json::JsonValue>],
) -> Result<String, Error> {
    let mut claims = Vec::from(claims);
    let topo_id = label.to_snake_case();
    claims.push(claim_string(
        client.api.known_entities.properties.topo_id_id.as_str(),
        &topo_id,
    ));

    // for an item, we need to do a separate query to check if the item is already there
    let id = if let Some(id) = client.sparql.get_id_by_topo_id(&topo_id)? {
        log::info!("item \"{}\" already exists with id {}", label, id);
        id
    } else {
        let id = client.api.create_item(label, claims)?;
        log::info!("creating item \"{}\" with id {}", label, id);
        id
    };
    Ok(id.to_owned())
}

fn get_or_create_property(
    api: &ApiClient,
    label: &str,
    prop_type: PropertyDataType,
) -> Result<String, Error> {
    get_or_create_property_impl(
        api,
        label,
        prop_type,
        api.known_entities.properties.topo_id_id.as_str(),
    )
}

fn get_or_create_topo_id_id(client: &mut Client) -> Result<String, Error> {
    let topo_id =
        get_or_create_property_impl(&client.api, "Topo tools id", PropertyDataType::String, None)?;

    // the topo id is a bit special, once we get its id, we set it in the known_entitiesuration as it will be needed
    // for the creation of all the other properties/items
    client.sparql.known_entities.properties.topo_id_id = topo_id.clone();
    client.api.known_entities.properties.topo_id_id = topo_id.clone();

    Ok(topo_id)
}

fn get_or_create_property_impl<'a>(
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
    let r = client.create_object(ObjectType::Property(prop_type), label, claims);
    if let Err(ApiError::PropertyAlreadyExists { label, id }) = r {
        log::info!("property \"{}\" already exists with id {}", label, id);
        Ok(id)
    } else {
        let id = r?;
        log::info!("creating property \"{}\" with id {}", label, id);
        Ok(id)
    }
}

pub fn initial_populate(api_endpoint: &str, sparql_endpoint: &str) -> Result<EntitiesId, Error> {
    let mut client = Client::new_without_known_entities(api_endpoint, sparql_endpoint)?;
    let topo_id = get_or_create_topo_id_id(&mut client)?;

    let create_prop = |label, prop_type| get_or_create_property(&client.api, label, prop_type);

    let gtfs_id = create_prop("GTFS id", PropertyDataType::String)?;
    let instance_of = create_prop("Instance of", PropertyDataType::Item)?;

    let producer_class = get_or_create_item(&client, "Producer", &[])?;
    let physical_mode = get_or_create_item(&client, "Physical mode", &[])?;

    let create_mode = |label, id| {
        get_or_create_item(
            &client,
            label,
            &[
                claim_item(&instance_of, &physical_mode),
                claim_string(&gtfs_id, id),
            ],
        )
    };
    let create_stop = |label, id| get_or_create_item(&client, label, &[claim_string(&gtfs_id, id)]);

    let known_entities = EntitiesId {
        properties: Properties {
            topo_id_id: topo_id.to_owned(),
            produced_by: create_prop("Produced by", PropertyDataType::Item)?,
            instance_of: instance_of.clone(),
            gtfs_short_name: create_prop("GTFS short name", PropertyDataType::String)?,
            gtfs_long_name: create_prop("GTFS long name", PropertyDataType::String)?,
            gtfs_name: create_prop("GTFS name", PropertyDataType::String)?,
            gtfs_id: gtfs_id.to_owned(),
            has_physical_mode: create_prop("Has physical mode", PropertyDataType::Item)?,
            first_seen_in: create_prop("First seen in", PropertyDataType::Item)?,
            data_source: create_prop("Data source", PropertyDataType::Item)?,
            source: create_prop("Source", PropertyDataType::String)?, //Link to the raw file
            file_format: create_prop("File format", PropertyDataType::String)?,
            sha_256: create_prop("sha_256", PropertyDataType::String)?,
            tool_version: create_prop("Tool version", PropertyDataType::String)?,
            part_of: create_prop("Part of", PropertyDataType::Item)?,
            connecting_line: create_prop("Connecting line", PropertyDataType::Item)?,
        },
        items: Items {
            physical_mode: physical_mode.to_owned(),
            route: get_or_create_item(&client, "Route", &[])?,
            producer: producer_class.to_owned(),
            tramway: create_mode("Tramway", "0")?,
            subway: create_mode("Subway", "1")?,
            railway: create_mode("Railway", "2")?,
            bus: create_mode("Bus", "3")?,
            ferry: create_mode("Ferry", "4")?,
            cable_car: create_mode("Cable car", "5")?,
            gondola: create_mode("Gondola", "6")?,
            funicular: create_mode("Funicular", "7")?,
            stop_point: create_stop("Stop point", "0")?,
            stop_area: create_stop("Stop area", "1")?,
            stop_entrance: create_stop("Stop entrance", "2")?,
            stop_generic_node: create_stop("Stop generic node", "3")?,
            stop_boarding_area: create_stop("Stop boarding area", "4")?,
        },
    };

    Ok(known_entities)
}
