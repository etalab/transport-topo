use crate::api_structures::*;
use anyhow::anyhow;
use json::object;
use regex::Regex;
use thiserror::Error;

const WIKIBASE_LABEL_CONFLICT: &str = "wikibase-validator-label-conflict";

lazy_static::lazy_static! {
    // the message in the api response is in the form "[[Property:P1|P1]]"
    // and in this example we want to extract "P1"
    static ref LABEL_CONFLICT_REGEX: Regex = Regex::new(r#"\[\[.+\|(.+)\]\]"#).unwrap();
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{label} already exists, id = {id}")]
    PropertyAlreadyExists { label: String, id: String },
    #[error("{0} is not a valid producer id")]
    InvalidProducer(String),
    #[error("Several items with label {0}")]
    TooManyItems(String),
    #[error("error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("error: {0}")]
    InvalidJsonError(#[from] serde_json::Error),
    #[error("error: {0}")]
    GenericError(String),
}

pub enum PropertyDataType {
    String,
    Item,
}

impl std::string::ToString for PropertyDataType {
    fn to_string(&self) -> String {
        match self {
            Self::String => "string".to_owned(),
            Self::Item => "wikibase-item".to_owned(),
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
    config: crate::client::EntitiesId,
    endpoint: String,
    token: String,
}

impl ApiClient {
    pub fn new(endpoint: &str, config: crate::client::EntitiesId) -> Result<Self, ApiError> {
        let client = reqwest::Client::new();
        let res = client
            .get(endpoint)
            .query(&[("format", "json"), ("action", "query"), ("meta", "tokens")])
            .send()?
            .json::<TokenResponse>()?;
        Ok(ApiClient {
            client,
            config,
            endpoint: endpoint.to_owned(),
            token: res.query.tokens.csrftoken,
        })
    }

    fn get(&self) -> reqwest::RequestBuilder {
        self.client.get(&self.endpoint).query(&[("format", "json")])
    }

    /// search for all entities for a given english label
    pub fn find_entities(
        &self,
        label: &str,
        obj_type: ObjectType,
    ) -> Result<Vec<SearchResultItem>, ApiError> {
        let res = self
            .get()
            .query(&[
                ("action", "wbsearchentities"),
                ("language", "en"),
                ("search", label),
                ("type", &obj_type.to_string()),
            ])
            .send()?
            .json::<SearchResponse>()?;

        Ok(res.search)
    }

    /// search for an entity by it's english label, and return it if it is uniq
    /// if multiple items match, return an error
    pub fn find_entity_id(
        &self,
        object_type: ObjectType,
        label: &str,
    ) -> Result<Option<String>, ApiError> {
        self.find_entities(label, object_type)
            .and_then(|entries| match entries.as_slice() {
                [] => Ok(None),
                [e] => Ok(Some(e.id.clone())),
                _ => Err(ApiError::TooManyItems(label.to_owned())),
            })
    }

    pub fn create_object(
        &self,
        object_type: ObjectType,
        label: &str,
        extra_claims: &[json::JsonValue],
    ) -> Result<String, ApiError> {
        let labels = object! {
            "en" => object!{
                "language" => "en",
                "value" => label
            }
        };

        let claims = json::stringify(match &object_type {
            ObjectType::Property(datatype) => object! {
                "labels" => labels,
                "datatype" => datatype.to_string(),
                "claims" => json::Array::from(extra_claims),
            },
            ObjectType::Item => object! {
                "labels" => labels,
                "claims" => json::Array::from(extra_claims),
            },
        });

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
            .send()?;

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
        extra_claims: &[json::JsonValue],
    ) -> Result<String, ApiError> {
        self.create_object(ObjectType::Item, label, extra_claims)
    }

    pub fn insert_data_source(
        &self,
        sha256: &Option<String>,
        producer: &str,
        path: &str,
    ) -> Result<String, ApiError> {
        let dt = chrono::Utc::now();
        let label = format!("Data source for {} - imported {}", &producer, dt);

        let mut claims = vec![
            claim_item(&self.config.properties.produced_by, producer),
            claim_string(&self.config.properties.file_link, path),
            claim_string(&self.config.properties.file_format, "GTFS"),
            claim_string(&self.config.properties.tool_version, crate::GIT_VERSION),
        ];
        if let Some(sha) = sha256 {
            claims.push(claim_string(&self.config.properties.content_id, sha));
        }

        self.create_object(ObjectType::Item, &label, &claims)
    }

    pub fn insert_route(
        &self,
        route: &gtfs_structures::Route,
        data_source_id: &str,
        producer_name: &str,
    ) -> Result<String, ApiError> {
        let route_name = if !route.long_name.is_empty() {
            route.long_name.as_str()
        } else {
            route.short_name.as_str()
        };
        let label = format!("{} – ({})", route_name, producer_name);
        let claims = [
            claim_item(
                &self.config.properties.instance_of,
                &self.config.items.route,
            ),
            claim_string(&self.config.properties.gtfs_id, &route.id),
            claim_item(&self.config.properties.data_source, data_source_id),
            claim_string(&self.config.properties.gtfs_short_name, &route.short_name),
            claim_string(&self.config.properties.gtfs_long_name, &route.long_name),
            claim_item(
                &self.config.properties.has_physical_mode,
                self.config.physical_mode(route),
            ),
        ];

        self.create_object(ObjectType::Item, &label, &claims)
    }

    pub fn get_label(&self, id: &str) -> Result<String, anyhow::Error> {
        let res = self
            .get()
            .query(&[("action", "wbgetentities"), ("ids", id), ("format", "json")])
            .send()?
            .json::<serde_json::Value>()?;

        res.pointer(&format!("/entities/{}/labels/en/value", id))
            .and_then(|l| l.as_str())
            .map(|l| l.to_owned())
            .ok_or_else(|| anyhow!("no entitity {}", &id))
    }
}

pub fn claim(property: &str, datavalue: json::JsonValue) -> json::JsonValue {
    object! {
        "mainsnak" => object!{
            "snaktype" => "value",
            "property" => property,
            "datavalue" => datavalue
        },
        "type" => "statement",
        "rank" => "normal"
    }
}
pub fn claim_string(property: &str, value: &str) -> json::JsonValue {
    claim(property, object! { "value" => value, "type" => "string" })
}

pub fn claim_item(property: &str, id: &str) -> json::JsonValue {
    claim(
        property,
        object! {
            "value" => object!{ "entity-type" => "item", "id" => id },
            "type" => "wikibase-entityid",
        },
    )
}
