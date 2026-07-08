//use super::definition::FilterDefinition;
use crate::core::params::ParamDefinition;
use crate::core::image::Image;


pub trait Filter {

    type Params;
    type Input;


    fn name() -> &'static str;

    fn params() -> Vec<ParamDefinition>;


    fn parse_params(
        input: Self::Input
    ) -> Result<Self::Params, String>;


    fn apply(
        image: &mut Image,
        params: Self::Params,
    );
}