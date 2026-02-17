use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidImageId(i64),
    NonFiniteEditParam(&'static str),
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidImageId(value) => write!(f, "image id must be positive, got {value}"),
            Self::NonFiniteEditParam(name) => write!(f, "edit parameter {name} must be finite"),
        }
    }
}

impl std::error::Error for DomainError {}
