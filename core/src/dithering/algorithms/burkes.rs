use ddot_core_macros::FilterParams;
use serde::{Deserialize, Serialize};

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;
use crate::dithering::DitherAlgorithm;
use crate::dithering::utils::find_closest_color;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BurkesParams {
    #[param(min = 0.0, max = 1.0, default = 1.0)]
    pub amount: f32,
}

pub struct Burkes;

impl Burkes {
    pub const NAME: &'static str = "burkes";

    pub const fn definition() -> FilterDefinition {
        FilterDefinition {
            name: Self::NAME,
            params: <BurkesParams as crate::filter::FilterParams>::PARAMS,
        }
    }
}

impl DitherAlgorithm for Burkes {
    type Params = BurkesParams;

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

                let factor = 1.0 / 32.0;

                // Row 0
                // (x + 1, y) -> 8/32
                if x + 1 < width {
                    let n_idx = y * width + (x + 1);
                    f_pixels[n_idx][0] += err_r * 8.0 * factor;
                    f_pixels[n_idx][1] += err_g * 8.0 * factor;
                    f_pixels[n_idx][2] += err_b * 8.0 * factor;
                }
                // (x + 2, y) -> 4/32
                if x + 2 < width {
                    let n_idx = y * width + (x + 2);
                    f_pixels[n_idx][0] += err_r * 4.0 * factor;
                    f_pixels[n_idx][1] += err_g * 4.0 * factor;
                    f_pixels[n_idx][2] += err_b * 4.0 * factor;
                }

                // Row 1
                if y + 1 < height {
                    // (x - 2, y + 1) -> 2/32
                    if x >= 2 {
                        let n_idx = (y + 1) * width + (x - 2);
                        f_pixels[n_idx][0] += err_r * 2.0 * factor;
                        f_pixels[n_idx][1] += err_g * 2.0 * factor;
                        f_pixels[n_idx][2] += err_b * 2.0 * factor;
                    }
                    // (x - 1, y + 1) -> 4/32
                    if x >= 1 {
                        let n_idx = (y + 1) * width + (x - 1);
                        f_pixels[n_idx][0] += err_r * 4.0 * factor;
                        f_pixels[n_idx][1] += err_g * 4.0 * factor;
                        f_pixels[n_idx][2] += err_b * 4.0 * factor;
                    }
                    // (x, y + 1) -> 8/32
                    {
                        let n_idx = (y + 1) * width + x;
                        f_pixels[n_idx][0] += err_r * 8.0 * factor;
                        f_pixels[n_idx][1] += err_g * 8.0 * factor;
                        f_pixels[n_idx][2] += err_b * 8.0 * factor;
                    }
                    // (x + 1, y + 1) -> 4/32
                    if x + 1 < width {
                        let n_idx = (y + 1) * width + (x + 1);
                        f_pixels[n_idx][0] += err_r * 4.0 * factor;
                        f_pixels[n_idx][1] += err_g * 4.0 * factor;
                        f_pixels[n_idx][2] += err_b * 4.0 * factor;
                    }
                    // (x + 2, y + 1) -> 2/32
                    if x + 2 < width {
                        let n_idx = (y + 1) * width + (x + 2);
                        f_pixels[n_idx][0] += err_r * 2.0 * factor;
                        f_pixels[n_idx][1] += err_g * 2.0 * factor;
                        f_pixels[n_idx][2] += err_b * 2.0 * factor;
                    }
                }

                let output_colors = image.colors_mut();
                output_colors[idx] = closest;
            }
        }
    }
}
