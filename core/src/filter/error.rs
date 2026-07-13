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

#[derive(Debug)]
pub enum FilterError {
    UnknownFilter(String),

    InvalidSettings(serde_json::Error),

    InvalidParams(ParamError),

    GpuUnavailable,

    GpuExecutionFailed,

    UnsupportedBackend,
}

impl fmt::Display for FilterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterError::UnknownFilter(name) => {
                write!(formatter, "unknown filter '{name}'")
            }

            FilterError::InvalidSettings(error) => {
                write!(formatter, "invalid filter settings: {error}")
            }

            FilterError::InvalidParams(error) => {
                write!(formatter, "{error}")
            }

            FilterError::GpuUnavailable => {
                write!(formatter, "GPU backend is not available")
            }

            FilterError::GpuExecutionFailed => {
                write!(formatter, "GPU pipeline execution failed")
            }

            FilterError::UnsupportedBackend => {
                write!(formatter, "filter does not support the requested backend")
            }
        }
    }
}

impl Error for FilterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FilterError::UnknownFilter(_) => None,
            FilterError::InvalidSettings(error) => Some(error),
            FilterError::InvalidParams(error) => Some(error),
            FilterError::GpuUnavailable => None,
            FilterError::GpuExecutionFailed => None,
            FilterError::UnsupportedBackend => None,
        }
    }
}

impl From<serde_json::Error> for FilterError {
    fn from(error: serde_json::Error) -> Self {
        FilterError::InvalidSettings(error)
    }
}

impl From<ParamError> for FilterError {
    fn from(error: ParamError) -> Self {
        FilterError::InvalidParams(error)
    }
}
