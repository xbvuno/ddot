use super::{ParamDefinition, error::ParamError};

pub trait FilterParams {
    const PARAMS: &'static [ParamDefinition];

    fn validate(&self) -> Result<(), ParamError>;
}
