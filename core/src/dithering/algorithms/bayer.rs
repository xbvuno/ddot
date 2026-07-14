use ddot_core_macros::FilterParams;
use serde::{Deserialize, Serialize};

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;
use crate::dithering::DitherAlgorithm;
use crate::dithering::utils::find_closest_color;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BayerParams {
    #[param(min = 0.0, max = 1.0, default = 1.0)]
    pub amount: f32,

    #[param(min = 1.0, max = 8.0, default = 1.0)]
    pub matrix_scale: f32,
}

pub struct Bayer;

impl Bayer {
    pub const NAME: &'static str = "bayer";

    pub const fn definition() -> FilterDefinition {
        FilterDefinition {
            name: Self::NAME,
            params: <BayerParams as crate::filter::FilterParams>::PARAMS,
        }
    }
}

impl DitherAlgorithm for Bayer {
    type Params = BayerParams;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn definition(&self) -> FilterDefinition {
        Self::definition()
    }

    fn apply(&self, image: &mut Image, palette: &Palette, params: &Self::Params) {
        if image.width == 0 || image.height == 0 {
            return;
        }

        let width = image.width as usize;
        let height = image.height as usize;
        let amount = params.amount;
        let scale = params.matrix_scale.max(1.0) as usize;

        // 4x4 Bayer threshold matrix
        const BAYER_MATRIX: [[f32; 4]; 4] = [
            [ 0.0,  8.0,  2.0, 10.0],
            [12.0,  4.0, 14.0,  6.0],
            [ 3.0, 11.0,  1.0,  9.0],
            [15.0,  7.0, 13.0,  5.0]
        ];

        let output_colors = image.colors_mut();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let color = output_colors[idx];

                // Scale the coordinates correctly by integer division, then modulo 4 (or & 3)
                let bx = (x / scale) & 3;
                let by = (y / scale) & 3;
                let val = BAYER_MATRIX[by][bx];
                
                // Map threshold value to normalized range centered around 0
                let threshold = (val / 16.0 - 0.5) * amount * 255.0;

                let r = (color.r as f32 + threshold).clamp(0.0, 255.0);
                let g = (color.g as f32 + threshold * 0.9).clamp(0.0, 255.0);
                let b = (color.b as f32 + threshold * 0.8).clamp(0.0, 255.0);

                output_colors[idx] = find_closest_color(r, g, b, palette);
            }
        }
    }
}
