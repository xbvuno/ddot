use serde::Serialize;
use crate::core::params::ParamDefinition;


#[derive(Serialize)]
pub struct FilterDefinition {

    pub name: &'static str,

    pub params: Vec<ParamDefinition>,
}