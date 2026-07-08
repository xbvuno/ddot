use super::filter::Filter;
use crate::core::{
    image::Image,
    params::ParamDefinition,
};


pub struct Adjustment;

pub struct AdjustmentInput {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

pub struct AdjustmentParams {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}


impl Adjustment {

    pub fn apply(
        image: &mut Image,
        params: AdjustmentParams
    ) {
        // solo algoritmo
    }
}