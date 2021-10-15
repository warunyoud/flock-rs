use actix::{
    Message,
    prelude::Addr
};
use actix_redis::RedisActor;
use serde::Deserialize;
use crate::error::FlockError;

pub use crate::resources::target_info::TargetInfo;
pub use crate::utils::auth::AuthorizedReq;
pub use crate::utils::env::AuthInfo;

#[derive(Deserialize)]
pub struct Event {
    pub topic: String,
    pub message: String
}

impl Message for Event {
    type Result = Result<bool>;
}

pub struct AppState {
    pub redis_addr: Addr<RedisActor>,
    pub http_client: reqwest::Client,
    pub auth: AuthInfo
}

pub type Error = FlockError;
pub type Result<T> = std::result::Result<T, Error>;

