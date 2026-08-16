use std::fmt;

/// Errors that can occur in Drive contract operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveContractError {
    /// Invalid ID format.
    InvalidId { kind: String, value: String },
    /// Invalid URI format.
    InvalidUri { reason: String },
    /// Validation error.
    Validation { field: String, message: String },
    /// Not found.
    NotFound { kind: String, id: String },
    /// Conflict (e.g., duplicate name).
    Conflict { kind: String, message: String },
}

impl fmt::Display for DriveContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId { kind, value } => {
                write!(f, "invalid {kind} id: {value}")
            }
            Self::InvalidUri { reason } => {
                write!(f, "invalid drive uri: {reason}")
            }
            Self::Validation { field, message } => {
                write!(f, "validation error on {field}: {message}")
            }
            Self::NotFound { kind, id } => {
                write!(f, "{kind} not found: {id}")
            }
            Self::Conflict { kind, message } => {
                write!(f, "{kind} conflict: {message}")
            }
        }
    }
}

impl std::error::Error for DriveContractError {}
