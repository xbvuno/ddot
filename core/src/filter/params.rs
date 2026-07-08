use super::error::ParamError;


pub trait FilterParams {

    fn validate(
        &self
    ) -> Result<(), ParamError>;

}
