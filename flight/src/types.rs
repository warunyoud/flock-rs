use actix::{Addr, Message};
use serde::Deserialize;
use crate::{
    error::FlockError,
    resources::Dispatcher
};

pub use crate::utils::auth::AuthorizedReq;

#[derive(Deserialize)]
pub struct Event {
    pub topic: String,
    pub message: String
}

impl Message for Event {
    type Result = Result<bool>;
}

pub type Error = FlockError;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct AppState {
    pub dispatcher: Addr<Dispatcher>,
}
