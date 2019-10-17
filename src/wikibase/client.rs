//mod structures;

use super::api_structures::*;
use failure::{format_err, Error};
use gtfs_structures;
use json::object;

enum ObjectType {
    Item,
    Property,
}

pub struct Client {
    client: reqwest::Client,
    config: super::Config,
    token: Option<String>,
}

impl Client {
    pub fn new(config: crate::wikibase::Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            token: None,
        }
    }

    fn get(&self) -> reqwest::RequestBuilder {
        self.client
            .get(&self.config.api_endpoint)
            .query(&[("format", "json")])
    }

    pub fn find_entity(&self, label: &str) -> Result<Vec<SearchResultItem>, Error> {
        let res = self
            .get()
            .query(&[
                ("action", "wbsearchentities"),
                ("language", "en"),
                ("search", label),
                ("type", "item"),
            ])
            .send()?
            .json::<SearchResponse>()?;

        Ok(res.search)
    }

    pub fn get_token(&mut self) -> Result<String, Error> {
        if let Some(token) = &self.token {
            Ok(token.to_string())
        } else {
            let res = self
                .get()
                .query(&[("action", "query"), ("meta", "tokens")])
                .send()?
                .json::<TokenResponse>()?;
            let token = res.query.tokens.csrftoken;
            self.token = Some(token.clone());
            Ok(token)
        }
    }

    fn create_object(
        &mut self,
        object_type: ObjectType,
        label: &str,
        extra_claims: &[json::JsonValue],
    ) -> Result<String, Error> {
        let new_type = match object_type {
            ObjectType::Item => "item",
            ObjectType::Property => "property",
        };

        let labels = object! {
            "en" => object!{
                "language" => "en",
                "value" => label
            }
        };

        let claims = json::stringify(match object_type {
            ObjectType::Property => object! {
                "labels" => labels,
                "datatype" => "string",
            },
            ObjectType::Item => object! {
                "labels" => labels,
                "claims" => json::Array::from(extra_claims),
            },
        });

        log::trace!("claims: {}", claims);
        let mut res = self
            .client
            .post(&self.config.api_endpoint)
            .query(&[
                ("action", "wbeditentity"),
                ("new", new_type),
                ("format", "json"),
            ])
            .form(&[("token", self.get_token()?), ("data", claims)])
            .send()?;

        log::trace!("Response headers: {:#?}", res);
        let body = res.text()?;
        log::trace!("Response body: {:#?}", body);
        let res = serde_json::from_str::<ApiResponse>(&body)?;
        match res.content {
            ApiResponseContent::Entity(entity) => Ok(entity.id),
            ApiResponseContent::Error(err) => {
                log::warn!("Error inserting: {:#?}", err);
                Err(format_err!("Error while inserting: {}", err.info))
            }
        }
    }

    pub fn create_property(
        &mut self,
        label: &str,
        extra_claims: &[json::JsonValue],
    ) -> Result<String, Error> {
        self.create_object(ObjectType::Property, label, extra_claims)
    }

    pub fn insert_route(
        &mut self,
        producer: &str,
        producer_name: &str,
        route: &gtfs_structures::Route,
    ) -> Result<String, Error> {
        let route_name = if !route.long_name.is_empty() {
            route.long_name.as_str()
        } else {
            route.short_name.as_str()
        };
        let label = format!("{} – ({})", route_name, producer_name);
        let claims = [
            claim_item(&self.config.properties.instance_of, self.config.items.line),
            claim_string(&self.config.properties.gtfs_id, &route.id),
            claim_item(
                &self.config.properties.produced_by,
                producer.trim_start_matches('Q').parse()?,
            ),
            claim_string(&self.config.properties.gtfs_short_name, &route.short_name),
            claim_string(&self.config.properties.gtfs_long_name, &route.long_name),
            claim_item(
                &self.config.properties.physical_mode,
                self.config.physical_mode(route),
            ),
        ];

        self.create_object(ObjectType::Item, &label, &claims)
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

pub fn claim_item(property: &str, id: u64) -> json::JsonValue {
    claim(
        property,
        object! {
            "value" => object!{ "entity-type" => "item", "numeric-id" => id },
            "type" => "wikibase-entityid",
        },
    )
}
