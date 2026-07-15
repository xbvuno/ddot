use ddot_core_macros::FilterParams;
use serde::{Deserialize, Serialize};

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;
use crate::dithering::DitherAlgorithm;
use crate::dithering::utils::find_closest_color;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AtkinsonParams {
    #[param(min = 0.0, max = 1.0, default = 1.0)]
    pub amount: f32,
}

pub struct Atkinson;

impl Atkinson {
    pub const NAME: &'static str = "atkinson";

    pub const fn definition() -> FilterDefinition {
        FilterDefinition {
            name: Self::NAME,
            params: <AtkinsonParams as crate::filter::FilterParams>::PARAMS,
        }
    }
}

impl DitherAlgorithm for Atkinson {
    type Params = AtkinsonParams;

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

        let colors = image.colors();
        let mut f_pixels: Vec<[f32; 3]> = colors
            .iter()
            .map(|c| [c.r as f32, c.g as f32, c.b as f32])
            .collect();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let [old_r, old_g, old_b] = f_pixels[idx];

                let closest = find_closest_color(old_r, old_g, old_b, palette);

                let err_r = (old_r - closest.r as f32) * amount;
                let err_g = (old_g - closest.g as f32) * amount;
                let err_b = (old_b - closest.b as f32) * amount;

                let factor = 1.0 / 8.0;

                // Diffuse to neighbors
                // (x + 1, y)
                if x + 1 < width {
                    let n_idx = y * width + (x + 1);
                    f_pixels[n_idx][0] += err_r * factor;
                    f_pixels[n_idx][1] += err_g * factor;
                    f_pixels[n_idx][2] += err_b * factor;
                }
                // (x + 2, y)
                if x + 2 < width {
                    let n_idx = y * width + (x + 2);
                    f_pixels[n_idx][0] += err_r * factor;
                    f_pixels[n_idx][1] += err_g * factor;
                    f_pixels[n_idx][2] += err_b * factor;
                }
                // (x - 1, y + 1)
                if x > 0 && y + 1 < height {
                    let n_idx = (y + 1) * width + (x - 1);
                    f_pixels[n_idx][0] += err_r * factor;
                    f_pixels[n_idx][1] += err_g * factor;
                    f_pixels[n_idx][2] += err_b * factor;
                }
                // (x, y + 1)
                if y + 1 < height {
                    let n_idx = (y + 1) * width + x;
                    f_pixels[n_idx][0] += err_r * factor;
                    f_pixels[n_idx][1] += err_g * factor;
                    f_pixels[n_idx][2] += err_b * factor;
                }
                // (x + 1, y + 1)
                if x + 1 < width && y + 1 < height {
                    let n_idx = (y + 1) * width + (x + 1);
                    f_pixels[n_idx][0] += err_r * factor;
                    f_pixels[n_idx][1] += err_g * factor;
                    f_pixels[n_idx][2] += err_b * factor;
                }
                // (x, y + 2)
                if y + 2 < height {
                    let n_idx = (y + 2) * width + x;
                    f_pixels[n_idx][0] += err_r * factor;
                    f_pixels[n_idx][1] += err_g * factor;
                    f_pixels[n_idx][2] += err_b * factor;
                }

                let output_colors = image.colors_mut();
                output_colors[idx] = closest;
            }
        }
    }
}
