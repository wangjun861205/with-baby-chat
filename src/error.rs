use std::fmt::Display;

use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder, ResponseError};

#[derive(Debug)]
pub struct Error(pub String);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error,
{
    fn from(e: E) -> Self {
        Self(format!("{}", e))
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(self.0.clone())
    }
}
