use std::sync::Arc;
use log::error;
use pyo3::prelude::*;

mod error;
mod types;

use crate::{
    types::{Result, Error},
};

#[pyclass]
pub struct PyTrafficControlClient {
    client: Arc<TrafficControlClient>
}

#[pymethods]
impl PyTrafficControlClient {
    #[new]
    pub fn new(username: &str, password: &str) -> Self {
        let client = Arc::new(TrafficControlClient::new(
            username,
            password
        ));
        Self {
            client
        }
    }

    pub fn publish(self_: PyRef<Self>, base_url: &str, topic: &str, payload: &str) -> PyResult<()> {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let client = self_.client.clone();
        rt.block_on(async {
            client.publish(base_url, topic, payload).await
        })?;
        Ok(())
    }
}

#[pymodule]
fn traffic_control_client(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTrafficControlClient>()?;
    Ok(())
}

pub struct TrafficControlClient {
    client: reqwest::Client,
    username: String,
    password: String,
}

impl TrafficControlClient {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub async fn publish(&self, base_url: &str, topic: &str, payload: &str) -> Result<()> {
        let url = format!("{}/api/v1/event/{}", base_url, topic);
        match self.client.post(&url).body(payload.to_string())
            .basic_auth(&self.username, Some(&self.password))
            .header("content-type", "application/json")
            .send().await {
            Ok(res) => {
                if res.status() == reqwest::StatusCode::NO_CONTENT {
                    Ok(())
                } else if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                    Err(Error::Unauthorized)
                } else {
                    error!("Unexpected status code {}", res.status());
                    Err(Error::InternalError)
                }
            }
            Err(err) => Err(err.into())
        }
    }
}
