use crate::error::TrafficControlClientError;
pub type Error = TrafficControlClientError;
pub type Result<T> = std::result::Result<T, Error>;
