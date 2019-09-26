//mod structures;

use super::structures::*;
use failure::Error;
use gtfs_structures;
use json::{array, object};

pub struct Client {
    client: reqwest::Client,
    config: super::Config,
    token: Option<String>,
}

pub fn new(config: super::Config) -> Client {
    Client {
        client: reqwest::Client::new(),
        config,
        token: None,
    }
}

impl Client {
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
        let claims = object! {
            "labels" => object!{
                "en" => object!{
                    "language" => "en",
                    "value" => format!("{} – ({})", route_name, producer_name)
                }
            },
            "claims" => array![
                claim_item(&self.config.properties.instance_of, self.config.items.line),
                claim_string(&self.config.properties.gtfs_id, &route.id), // has <gtfs id>
                claim_item(&self.config.properties.produced_by, producer.trim_start_matches('Q').parse()?), // <produced by>
                claim_string(&self.config.properties.gtfs_short_name, &route.short_name),
                claim_string(&self.config.properties.gtfs_long_name, &route.long_name),
                claim_item(&self.config.properties.physical_mode, self.config.physical_mode(route)) // has <physical mode> <bus>
        ]};

        let res = self
            .client
            .post(&self.config.api_endpoint)
            .query(&[
                ("action", "wbeditentity"),
                ("new", "item"),
                ("format", "json"),
            ])
            .form(&[
                ("token", self.get_token()?),
                ("data", json::stringify(claims)),
            ])
            .send()?
            .json::<InsertResponse>()?;
        Ok(res.entity.id)
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
