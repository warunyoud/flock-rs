use actix_web::{
  dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum FlockError {
    #[display(fmt = "Internal Server Error")]
    InternalError,

    #[display(fmt = "Bad Request")]
    BadRequest
}

impl From<reqwest::Error> for FlockError {
    fn from(error: reqwest::Error) -> Self {
        println!("Reqwest Error: {:?}", error);
        FlockError::InternalError
    }
}

impl From<actix::MailboxError> for FlockError {
    fn from(error: actix::MailboxError) -> Self {
        println!("Actix Mailbox Error: {:?}", error);
        FlockError::InternalError
    }
}

impl From<actix_redis::Error> for FlockError {
    fn from(error: actix_redis::Error) -> Self {
        println!("Redis Error: {:?}", error);
        FlockError::InternalError
    }
}

impl From<std::time::SystemTimeError> for FlockError {
    fn from(error: std::time::SystemTimeError) -> Self {
        println!("SystemTime Error: {:?}", error);
        FlockError::InternalError
    }
}

impl From<serde_json::Error> for FlockError {
    fn from(error: serde_json::Error) -> Self {
        println!("Serde Error: {:?}", error);
        FlockError::BadRequest
    }
}

impl From<actix_web::Error> for FlockError {
    fn from(error: actix_web::Error) -> Self {
        println!("Actix Web: {:?}", error);
        FlockError::InternalError
    }    
}

impl error::ResponseError for FlockError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            FlockError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            FlockError::BadRequest => StatusCode::BAD_REQUEST
        }
    }
}