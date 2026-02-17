use std::fmt::{Display, Formatter};

use lite_room_domain::DomainError;

#[derive(Debug)]
pub enum ApplicationError {
    Domain(DomainError),
    InvalidInput(String),
    NotFound(String),
    Io(String),
    Persistence(String),
    Decode(String),
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => write!(f, "{error}"),
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Io(msg) => write!(f, "io error: {msg}"),
            Self::Persistence(msg) => write!(f, "persistence error: {msg}"),
            Self::Decode(msg) => write!(f, "decode error: {msg}"),
        }
    }
}

impl std::error::Error for ApplicationError {}

impl From<DomainError> for ApplicationError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}
