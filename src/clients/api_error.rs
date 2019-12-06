use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{label} already exists, id = {id}")]
    PropertyAlreadyExists { label: String, id: String },
    #[error("Several items with label {0}")]
    TooManyItems(String),
    #[error("Cannot find entiy {0}")]
    EntityNotFound(String),
    #[error("error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("error: {0}")]
    InvalidJsonError(#[from] serde_json::Error),
    #[error("error: {0}")]
    GenericError(String),
}
