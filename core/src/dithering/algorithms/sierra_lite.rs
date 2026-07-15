use ddot_core_macros::FilterParams;
use serde::{Deserialize, Serialize};

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;
use crate::dithering::DitherAlgorithm;
use crate::dithering::utils::find_closest_color;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SierraLiteParams {
    #[param(min = 0.0, max = 1.0, default = 1.0)]
    pub amount: f32,
}

pub struct SierraLite;

impl SierraLite {
    pub const NAME: &'static str = "sierra_lite";

    pub const fn definition() -> FilterDefinition {
        FilterDefinition {
            name: Self::NAME,
            params: <SierraLiteParams as crate::filter::FilterParams>::PARAMS,
        }
    }
}

impl DitherAlgorithm for SierraLite {
    type Params = SierraLiteParams;

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

        // Convert pixels to f32 for error diffusion precision
        let colors = image.colors();
        let mut f_pixels: Vec<[f32; 3]> = colors
            .iter()
            .map(|c| [c.r as f32, c.g as f32, c.b as f32])
            .collect();

        // Sierra Lite taps relative offset (dx, dy) and weight
        const TAPS: &[(isize, isize, f32)] = &[
            (1, 0, 2.0),
            (-1, 1, 1.0), (0, 1, 1.0),
        ];

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let [old_r, old_g, old_b] = f_pixels[idx];

                let closest = find_closest_color(old_r, old_g, old_b, palette);

                let err_r = (old_r - closest.r as f32) * amount;
                let err_g = (old_g - closest.g as f32) * amount;
                let err_b = (old_b - closest.b as f32) * amount;

                // Diffuse errors
                for &(dx, dy, weight) in TAPS {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                        let n_idx = (ny as usize) * width + (nx as usize);
                        f_pixels[n_idx][0] += err_r * (weight / 4.0);
                        f_pixels[n_idx][1] += err_g * (weight / 4.0);
                        f_pixels[n_idx][2] += err_b * (weight / 4.0);
                    }
                }

                // Apply quantized color
                let output_colors = image.colors_mut();
                output_colors[idx] = closest;
            }
        }
    }
}
