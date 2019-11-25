pub mod client;
pub mod clients;
pub mod database_initializer;
pub mod entity;
pub mod known_entities;
pub mod log;

pub use client::Client;
pub use clients::ObjectType;

pub const GIT_VERSION: &str = git_version::git_describe!("--dirty");
