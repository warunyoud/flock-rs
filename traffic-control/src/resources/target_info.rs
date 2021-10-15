use serde::Deserialize;
use redis_async::resp::RespValue;
use crate::types::{Result, Error};

#[derive(Deserialize, Debug)]
pub struct TargetInfo {
    host: String,
    port: u16
}

impl std::fmt::Display for TargetInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl TargetInfo {
    fn from_str(my_str: &str) -> Result<Self> {
        let v: Vec<&str> = my_str.splitn(2, ':').collect();
        if v.len() < 2 {
            Err(Error::InternalError)
        } else {
            Ok(TargetInfo {
                host: v[0].to_string(),
                port: v[1].parse::<u16>().map_err(|_| Error::InternalError)?
            })
        }   
    }
}

impl std::convert::TryFrom<RespValue> for TargetInfo {
    type Error = Error;

    fn try_from(value: RespValue) -> Result<Self> {
        match value {
            RespValue::SimpleString(my_str) => {
                TargetInfo::from_str(&my_str)
            }
            RespValue::BulkString(vec) => {
                let my_str = std::str::from_utf8(&vec).map_err(|_| Error::InternalError)?;
                TargetInfo::from_str(my_str) 
            }
            _ => Err(Error::InternalError)
        }
    }
}