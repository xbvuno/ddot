use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamError {
    OutOfRange {
        name: &'static str,
        value: String,
        min: String,
        max: String,
    },

    InvalidValue {
        name: &'static str,
        value: String,
    },
}

impl fmt::Display for ParamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::OutOfRange {
                name,
                value,
                min,
                max,
            } => {
                write!(
                    formatter,
                    "parameter '{name}' is out of range: {value} (expected {min}..{max})"
                )
            }

            ParamError::InvalidValue { name, value } => {
                write!(
                    formatter,
                    "parameter '{name}' has an invalid value: {value}"
                )
            }
        }
    }
}

impl Error for ParamError {}
