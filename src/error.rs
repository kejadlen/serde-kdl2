use std::fmt;

/// Error type for serde-kdl serialization and deserialization.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A KDL parsing error.
    #[error("KDL parse error: {0}")]
    Parse(#[from] kdl::KdlError),

    /// A serde (de)serialization error.
    #[error("{0}")]
    Message(String),

    /// The top-level type must be a struct or map.
    #[error("top-level type must be a struct or map")]
    TopLevelNotStruct,

    /// Expected a specific type but got something else.
    #[error("type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: &'static str, got: String },

    /// A required field was missing.
    #[error("missing field: {0}")]
    MissingField(String),

    /// Integer overflow during conversion.
    #[error("integer {0} out of range for target type")]
    IntegerOutOfRange(i128),

    /// Enum variant not found or ambiguous.
    #[error("unknown enum variant: {0}")]
    UnknownVariant(String),

    /// Unsupported operation.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
