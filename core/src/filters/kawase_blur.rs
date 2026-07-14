use ddot_core_macros::{Filter, FilterParams};
use serde::{Deserialize, Serialize};

use crate::image::Image;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct KawaseBlurParams {
    #[serde(rename = "blurStrength")]
    #[param(min = 0.0, max = 4.0, default = 0.0)]
    pub blur_strength: f32,

    #[serde(rename = "edgeStrength")]
    #[param(min = 0.0, max = 30.0, default = 12.0)]
    pub edge_strength: f32,

    #[param(min = 1.0, max = 4.0, default = 2.0)]
    pub passes: f32,
}

#[derive(Filter)]
#[filter(params = KawaseBlurParams)]
pub struct KawaseBlur;

impl crate::filter::GpuFilter for KawaseBlur {
    fn gpu_shader(&self) -> &'static str {
        include_str!("gpu/kawase_blur.wgsl")
    }
}

#[inline(always)]
fn edge_weight(a: &[f32; 3], b: &[f32; 3], strength: f32) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    let diff = (dr * dr + dg * dg + db * db).sqrt();
    (-diff * strength).exp()
}

// Bilinear sampling helper on f32 pixels
fn sample_bilinear(pixels: &[[f32; 3]], width: usize, height: usize, px: f32, py: f32) -> [f32; 3] {
    let w = width as f32;
    let h = height as f32;
    let x = px.clamp(0.0, w - 1.0);
    let y = py.clamp(0.0, h - 1.0);

    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let tx = x - x.floor();
    let ty = y - y.floor();

    let c00 = pixels[y0 * width + x0];
    let c10 = pixels[y0 * width + x1];
    let c01 = pixels[y1 * width + x0];
    let c11 = pixels[y1 * width + x1];

    let lerp_ch = |a: f32, b: f32, t: f32| a + (b - a) * t;

    [
        lerp_ch(lerp_ch(c00[0], c10[0], tx), lerp_ch(c01[0], c11[0], tx), ty),
        lerp_ch(lerp_ch(c00[1], c10[1], tx), lerp_ch(c01[1], c11[1], tx), ty),
        lerp_ch(lerp_ch(c00[2], c10[2], tx), lerp_ch(c01[2], c11[2], tx), ty),
    ]
}

impl KawaseBlur {
    pub fn apply(&self, image: &mut Image, params: &KawaseBlurParams) {
        if params.blur_strength <= 0.0 || params.passes <= 0.0 {
            return;
        }

        let width = image.width as usize;
        let height = image.height as usize;
        if width == 0 || height == 0 {
            return;
        }

        let passes = params.passes.clamp(1.0, 10.0) as usize;
        let offset = params.blur_strength;
        let edge_strength = params.edge_strength;

        // Convert image to f32 colors for calculation
        let mut src_pixels: Vec<[f32; 3]> = image.colors()
            .iter()
            .map(|c| [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0])
            .collect();
        
        let mut dest_pixels = src_pixels.clone();

        for _pass in 0..passes {
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let center = src_pixels[idx];

                    // Sample 4 corner offsets
                    let c1 = sample_bilinear(&src_pixels, width, height, x as f32 + offset, y as f32 + offset);
                    let c2 = sample_bilinear(&src_pixels, width, height, x as f32 - offset, y as f32 + offset);
                    let c3 = sample_bilinear(&src_pixels, width, height, x as f32 + offset, y as f32 - offset);
                    let c4 = sample_bilinear(&src_pixels, width, height, x as f32 - offset, y as f32 - offset);

                    let w1 = edge_weight(&center, &c1, edge_strength);
                    let w2 = edge_weight(&center, &c2, edge_strength);
                    let w3 = edge_weight(&center, &c3, edge_strength);
                    let w4 = edge_weight(&center, &c4, edge_strength);

                    let sum_r = center[0] + c1[0] * w1 + c2[0] * w2 + c3[0] * w3 + c4[0] * w4;
                    let sum_g = center[1] + c1[1] * w1 + c2[1] * w2 + c3[1] * w3 + c4[1] * w4;
                    let sum_b = center[2] + c1[2] * w1 + c2[2] * w2 + c3[2] * w3 + c4[2] * w4;

                    let total_w = 1.0 + w1 + w2 + w3 + w4;

                    dest_pixels[idx] = [
                        sum_r / total_w,
                        sum_g / total_w,
                        sum_b / total_w,
                    ];
                }
            }
            // Swap buffers for the next pass
            src_pixels.copy_from_slice(&dest_pixels);
        }

        // Write back to image
        let output_colors = image.colors_mut();
        for i in 0..output_colors.len() {
            output_colors[i].r = (dest_pixels[i][0] * 255.0).clamp(0.0, 255.0).round() as u8;
            output_colors[i].g = (dest_pixels[i][1] * 255.0).clamp(0.0, 255.0).round() as u8;
            output_colors[i].b = (dest_pixels[i][2] * 255.0).clamp(0.0, 255.0).round() as u8;
        }
    }
}
