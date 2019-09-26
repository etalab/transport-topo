use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct EntityResponse {
    pub entities: HashMap<String, Entity>,
}

#[derive(Deserialize, Debug)]
pub struct Value {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct Datavalue {
    pub value: Value,
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
    pub claims: HashMap<String, Vec<Claim>>,
    pub labels: HashMap<String, Label>,
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
pub struct InsertResponse {
    pub entity: InsertEntity,
}

#[derive(Deserialize, Debug)]
pub struct InsertEntity {
    pub id: String,
}
