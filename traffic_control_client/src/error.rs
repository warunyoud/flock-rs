use log::error;
use pyo3::{
    exceptions::{PyException},
    PyErr
};

#[derive(Debug, PartialEq)]
pub enum TrafficControlClientError {
    Unauthorized,
    InternalError
}

impl From<reqwest::Error> for TrafficControlClientError {
    fn from(error: reqwest::Error) -> Self {
        error!("Reqwest Error: {:?}", error);
        TrafficControlClientError::InternalError
    }
}

impl From<TrafficControlClientError> for PyErr {
    fn from(err: TrafficControlClientError) -> PyErr {
        match err {
            TrafficControlClientError::Unauthorized => PyException::new_err("Unauthorized"),
            _ => PyException::new_err("Something went wrong")
        }
    }
}