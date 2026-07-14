use ddot_core_macros::FilterParams;
use serde::{Deserialize, Serialize};

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;
use crate::dithering::DitherAlgorithm;
use crate::dithering::utils::find_closest_color;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RandomParams {
    #[param(min = 0.0, max = 1.0, default = 0.65)]
    pub amount: f32,

    #[param(min = 0.0, max = 100.0, default = 1.0)]
    pub seed: f32,
}

pub struct Random;

impl Random {
    pub const NAME: &'static str = "random";

    pub const fn definition() -> FilterDefinition {
        FilterDefinition {
            name: Self::NAME,
            params: <RandomParams as crate::filter::FilterParams>::PARAMS,
        }
    }
}

#[inline]
fn hash_2d(x: f32, y: f32, seed: f32) -> f32 {
    let t = ((x + seed * 0.17) * 12.9898 + (y + seed * 1.31) * 78.233).sin() * 43758.5453;
    t - t.floor()
}

impl DitherAlgorithm for Random {
    type Params = RandomParams;

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
        let seed = params.seed;

        let output_colors = image.colors_mut();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let color = output_colors[idx];

                // Noise ranges in -0.5 .. 0.5
                let noise = hash_2d(x as f32, y as f32, seed) - 0.5;
                let delta = noise * amount * 255.0;

                let r = (color.r as f32 + delta).clamp(0.0, 255.0);
                let g = (color.g as f32 + delta).clamp(0.0, 255.0);
                let b = (color.b as f32 + delta).clamp(0.0, 255.0);

                output_colors[idx] = find_closest_color(r, g, b, palette);
            }
        }
    }
}
