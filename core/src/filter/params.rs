use super::{ParamDefinition, error::ParamError};

pub trait FilterParams {
    const PARAMS: &'static [ParamDefinition];

    fn validate(&self) -> Result<(), ParamError>;

    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}
