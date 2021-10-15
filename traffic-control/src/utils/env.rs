use serde::Serialize;

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: u16 = 8080;

const DEFAULT_USERNAME: &str = "admin";
const DEFAULT_PASSWORD: &str = "password";

const DEFAULT_REDIS_HOST: &str = "localhost";
const DEFAULT_REDIS_PORT: u16 = 6379;

const DEFAULT_WATCHTOWER_URLS: &str = "http://localhost:8088";
const DEFAULT_WATCHTOWER_USERNAME: &str = "admin";
const DEFAULT_WATCHTOWER_PASSWORD: &str = "password";

#[derive(Serialize)]
pub struct InstanceInfo {
    pub host: String,
    pub port: u16
}

pub fn get_instance_info() -> InstanceInfo {
    InstanceInfo {
        host: std::env::var("TRAFFIC_CONTROL_HOST").unwrap_or(DEFAULT_HOST.to_string()),
        port: match std::env::var("TRAFFIC_CONTROL_PORT") {
            Ok(port) => port.parse::<u16>().unwrap_or(DEFAULT_PORT),
            _ => DEFAULT_PORT
        }
    }
}

pub fn get_redis_info() -> InstanceInfo {
    InstanceInfo {
        host: std::env::var("REDIS_HOST").unwrap_or(DEFAULT_REDIS_HOST.to_string()),
        port: match std::env::var("REDIS_PORT") {
            Ok(port) => port.parse::<u16>().unwrap_or(DEFAULT_REDIS_PORT),
            _ => DEFAULT_REDIS_PORT
        }
    }
}

pub struct WatchTowerConfig {
    pub urls: Vec<String>,
    pub username: String,
    pub password: String
}

pub fn get_watchtower_config() -> WatchTowerConfig {
    WatchTowerConfig {
        urls: std::env::var("WATCHTOWER_URLS").unwrap_or(DEFAULT_WATCHTOWER_URLS.to_string()).split(',').map(|item| item.to_string()).collect(),
        username: std::env::var("WATCHTOWER_USERNAME").unwrap_or(DEFAULT_WATCHTOWER_USERNAME.to_string()),
        password: std::env::var("WATCHTOWER_PASSWORD").unwrap_or(DEFAULT_WATCHTOWER_PASSWORD.to_string())
    }
}

#[derive(Clone)]
pub struct AuthInfo {
    pub username: String,
    pub password: String
}

pub fn get_auth_info() -> AuthInfo {
    AuthInfo {
        username: std::env::var("FLOCK_USERNAME").unwrap_or(DEFAULT_USERNAME.to_string()),
        password: std::env::var("FLOCK_PASSWORD").unwrap_or(DEFAULT_PASSWORD.to_string()), 
    }
}
