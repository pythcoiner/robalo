use std::fmt::Display;

use axum::http::header::ToStrError;

#[derive(Debug)]
pub enum Error {
    MissingHeaderEntry(&'static str),
    ToStr(&'static str, ToStrError),
    MissingField(&'static str),
    FieldType(&'static str),
    InvalidSecret,
    NotAction(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingHeaderEntry(e) => write!(f, "missing header entry {}", e),
            Error::ToStr(field, e) => write!(f, "failed to convert field {}: {}", field, e),
            Error::MissingField(field) => write!(f, "field {} is missing", field),
            Error::FieldType(field) => write!(f, "field {} is of wrong type", field),
            Error::InvalidSecret => write!(f, "sentry secret is not valid"),
            Error::NotAction(action) => write!(f, "action is not of type {}", action),
        }
    }
}
