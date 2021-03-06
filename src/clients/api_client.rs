use crate::clients::api_structures::*;
use crate::clients::ApiError;
use crate::entity;
use anyhow::anyhow;
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;

const WIKIBASE_LABEL_CONFLICT: &str = "wikibase-validator-label-conflict";

lazy_static::lazy_static! {
    // the message in the api response is in the form "[[Property:P1|P1]]"
    // and in this example we want to extract "P1"
    static ref LABEL_CONFLICT_REGEX: Regex = Regex::new(r#"\[\[.+\|(.+)\]\]"#).unwrap();
}

pub enum PropertyDataType {
    String,
    Url,
    Item,
    Coord,
}

impl std::string::ToString for PropertyDataType {
    fn to_string(&self) -> String {
        match self {
            Self::String => "string".to_owned(),
            Self::Item => "wikibase-item".to_owned(),
            Self::Url => "url".to_owned(),
            Self::Coord => "globe-coordinate".to_owned(),
        }
    }
}

pub enum ObjectType {
    Item,
    Property(PropertyDataType),
}

impl std::string::ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            Self::Item => "item".to_owned(),
            Self::Property(_) => "property".to_owned(),
        }
    }
}

pub struct ApiClient {
    client: reqwest::Client,
    endpoint: String,
    token: String,
}

impl ApiClient {
    pub fn new(endpoint: &str) -> Result<Self, ApiError> {
        let client = reqwest::Client::new();
        let res = client
            .get(endpoint)
            .query(&[("format", "json"), ("action", "query"), ("meta", "tokens")])
            .send()?
            .json::<TokenResponse>()?;
        Ok(ApiClient {
            client,
            endpoint: endpoint.to_owned(),
            token: res.query.tokens.csrftoken,
        })
    }

    fn get(&self) -> reqwest::RequestBuilder {
        self.client.get(&self.endpoint).query(&[("format", "json")])
    }

    /// search for all entities for a given english label
    pub fn get_entity(&self, id: &str) -> Result<entity::Entity, ApiError> {
        let mut res: EntityResponse = self
            .get()
            .query(&[("action", "wbgetentities"), ("ids", id)])
            .send()?
            .json()?;

        // the id is always here is the api response (even if the object does not exists)
        let r = res.entities.remove(id).ok_or_else(|| {
            ApiError::GenericError("invalid response format, no id in resposne".to_owned())
        })?;

        if r.missing.is_some() {
            Err(ApiError::EntityNotFound(id.to_owned()))
        } else {
            Ok(entity::Entity {
                id: r.id,
                label: r
                    .labels
                    .and_then(|mut l| l.remove("en"))
                    .map(|l| l.value)
                    .ok_or_else(|| ApiError::GenericError("invalid api response".to_owned()))?,
                properties: r
                    .claims
                    .unwrap()
                    .into_iter()
                    .map(|(prop_id, claims)| {
                        let vals = claims
                            .into_iter()
                            .map(|claim| {
                                let data_value = claim.mainsnak.datavalue;
                                match data_value {
                                    Datavalue::String(s) => entity::PropertyValue::String(s),
                                    Datavalue::Item { id } => entity::PropertyValue::Item(id),
                                    Datavalue::Coord {
                                        latitude,
                                        longitude,
                                    } => entity::PropertyValue::Coord {
                                        latitude,
                                        longitude,
                                    },
                                }
                            })
                            .collect();
                        Ok((prop_id, vals))
                    })
                    .collect::<Result<HashMap<String, Vec<entity::PropertyValue>>, ApiError>>()?,
            })
        }
    }

    pub fn create_object(
        &self,
        object_type: ObjectType,
        label: &str,
        extra_claims: Vec<Option<serde_json::Value>>,
    ) -> Result<String, ApiError> {
        let labels = json!({
            "en": {
                "language": "en",
                "value": label
            }
        });
        let extra_claims: Vec<_> = extra_claims.into_iter().filter_map(|v| v).collect();

        let json_claims = match &object_type {
            ObjectType::Property(datatype) => json!({
                "labels": labels,
                "datatype": datatype.to_string(),
                "claims": extra_claims,
            }),
            ObjectType::Item => json!( {
                "labels": labels,
                "claims": extra_claims,
            }),
        };
        let claims = serde_json::to_string(&json_claims)?;

        log::trace!("claims: {}", claims);
        let mut res = self
            .client
            .post(&self.endpoint)
            .query(&[
                ("action", "wbeditentity"),
                ("new", &object_type.to_string()),
                ("format", "json"),
            ])
            .form(&[("token", &self.token), ("data", &claims)])
            .send()?
            .error_for_status()?;

        log::trace!("Response headers: {:#?}", res);
        let body = res.text()?;
        log::trace!("Response body: {:#?}", body);
        let res = serde_json::from_str::<ApiResponse>(&body)?;
        match res.content {
            ApiResponseContent::Entity(entity) => Ok(entity.id),
            ApiResponseContent::Error(err) => {
                if let Some(message) = err
                    .messages
                    .iter()
                    .find(|m| m.name == WIKIBASE_LABEL_CONFLICT)
                {
                    // it seems to be the way to check that the write was rejected
                    // because something has already this label
                    // To get the id of the existing object,
                    // there does not seems to be a better way to parse the badly organised response

                    let existing_id = message.parameters.last().and_then(|p| {
                        LABEL_CONFLICT_REGEX
                            .captures(p)
                            .and_then(|r| r.get(1))
                            .map(|c| c.as_str())
                    });
                    if let Some(existing_id) = existing_id {
                        Err(ApiError::PropertyAlreadyExists {
                            label: label.to_owned(),
                            id: existing_id.to_owned(),
                        })
                    } else {
                        // we were not able to get the existing id, falling back to a generic error
                        log::warn!("impossible to parse conflict message: {:#?}", message);
                        Err(ApiError::GenericError(format!(
                            "conflict while inserting: {}",
                            err.info
                        )))
                    }
                } else {
                    log::warn!("Error inserting: {:#?}", err);
                    Err(ApiError::GenericError(format!(
                        "Error while inserting: {}",
                        err.info
                    )))
                }
            }
        }
    }

    pub fn create_item(
        &self,
        label: &str,
        extra_claims: Vec<Option<serde_json::Value>>,
    ) -> Result<String, ApiError> {
        self.create_object(ObjectType::Item, label, extra_claims)
    }

    pub fn get_label(&self, id: &str) -> Result<String, anyhow::Error> {
        let res = self
            .get()
            .query(&[("action", "wbgetentities"), ("ids", id), ("format", "json")])
            .send()?
            .error_for_status()?
            .json::<serde_json::Value>()?;

        res.pointer(&format!("/entities/{}/labels/en/value", id))
            .and_then(|l| l.as_str())
            .map(|l| l.to_owned())
            .ok_or_else(|| anyhow!("no entitity {}", &id))
    }

    pub fn add_claims(
        &self,
        entity_id: &str,
        claims: Vec<Option<serde_json::Value>>,
    ) -> Result<(), ApiError> {
        self.update_object_claims(entity_id, claims, false)
    }

    pub fn override_object_claims(
        &self,
        entity_id: &str,
        claims: Vec<Option<serde_json::Value>>,
    ) -> Result<(), ApiError> {
        self.update_object_claims(entity_id, claims, true)
    }

    fn update_object_claims(
        &self,
        entity_id: &str,
        claims: Vec<Option<serde_json::Value>>,
        override_claims: bool,
    ) -> Result<(), ApiError> {
        let claims: Vec<_> = claims.into_iter().filter_map(|v| v).collect();
        let claims = serde_json::to_string(&json!({ "claims": claims }))?;
        log::trace!("claims: {}", claims);
        let mut params = vec![
            ("action", "wbeditentity"),
            ("id", entity_id),
            ("format", "json"),
        ];
        if override_claims {
            params.push(("clear", "true"));
        }
        let mut res = self
            .client
            .post(&self.endpoint)
            .query(&params)
            .form(&[("token", &self.token), ("data", &claims)])
            .send()?
            .error_for_status()?;

        log::trace!("Response headers: {:#?}", res);
        let body = res.text()?;
        log::trace!("Response body: {:#?}", body);
        serde_json::from_str::<ApiResponse>(&body)?.error_for_status()?;
        Ok(())
    }
}

pub fn claim(property: &str, datavalue: serde_json::Value) -> Option<serde_json::Value> {
    Some(json!({
        "mainsnak": {
            "snaktype": "value",
            "property": property,
            "datavalue": datavalue
        },
        "type": "statement",
        "rank": "normal"
    }))
}
pub fn claim_string(property: &str, value: &str) -> Option<serde_json::Value> {
    let value = value.trim();
    if value.is_empty() {
        // it's impossible to add a claim with an empty value, so we skip it
        None
    } else {
        claim(property, json!({ "value": value, "type": "string" }))
    }
}

pub fn claim_item(property: &str, id: &str) -> Option<serde_json::Value> {
    claim(
        property,
        json!({
            "value":{ "entity-type": "item", "id": id },
            "type": "wikibase-entityid",
        }),
    )
}

pub fn claim_coord(property: &str, lon: f64, lat: f64) -> Option<serde_json::Value> {
    claim(
        property,
        json!({
            "value": {
                "latitude": lat,
                "longitude": lon,
                "precision": 0.000_001,
                "globe": "http://www.wikidata.org/entity/Q2"
            },
            "type": "globecoordinate",
        }),
    )
}
