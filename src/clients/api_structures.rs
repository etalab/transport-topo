use crate::clients::ApiError;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct EntityResponse {
    pub entities: HashMap<String, Entity>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Datavalue {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "wikibase-entityid")]
    Item { id: String },
    #[serde(rename = "globecoordinate")]
    Coord { latitude: f64, longitude: f64 },
}

#[derive(Deserialize, Debug)]
pub struct Snak {
    pub datavalue: Datavalue,
}

#[derive(Deserialize, Debug)]
pub struct Claim {
    pub mainsnak: Snak,
}

#[derive(Deserialize, Debug)]
pub struct Label {
    pub language: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct Entity {
    pub id: String,
    pub claims: Option<HashMap<String, Vec<Claim>>>,
    pub labels: Option<HashMap<String, Label>>,
    pub missing: Option<String>, // if not None, the object does not exists
}

#[derive(Deserialize, Debug)]
pub struct SearchResultItem {
    pub id: String,
    pub label: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub search: Vec<SearchResultItem>,
}

#[derive(Deserialize, Debug)]
pub struct Tokens {
    pub csrftoken: String,
}

#[derive(Deserialize, Debug)]
pub struct TokenQuery {
    pub tokens: Tokens,
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub query: TokenQuery,
}

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    #[serde(flatten)]
    pub content: ApiResponseContent,
}

impl ApiResponse {
    /// the wikibase api can return 200 status code but containing an error
    /// this checks that the api response is not an error
    /// and if it is convert return an error
    pub fn error_for_status(self) -> Result<ApiResponse, ApiError> {
        match self.content {
            ApiResponseContent::Entity(_) => Ok(self),
            ApiResponseContent::Error(e) => {
                log::info!(
                    "api error '{}': {}, message: {:?}",
                    &e.code,
                    &e.info,
                    &e.messages
                );
                Err(ApiError::GenericError(format!(
                    "Api error '{}': {} ",
                    &e.code, &e.info
                )))
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ApiResponseContent {
    Entity(InsertEntity),
    Error(Error),
}

#[derive(Deserialize, Debug)]
pub struct InsertEntity {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct Error {
    pub code: String,
    pub info: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub name: String,
    pub parameters: Vec<String>,
}
