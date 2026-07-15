use ddot_core_macros::{Filter, FilterParams};
use serde::{Deserialize, Serialize};

use crate::image::{Color, Image};

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct GaussianBlurParams {
    #[param(min = 0.0, max = 20.0, default = 0.0)]
    pub sigma: f32,
}

#[derive(Filter)]
#[filter(params = GaussianBlurParams)]
pub struct GaussianBlur;

impl crate::filter::GpuFilter for GaussianBlur {
    fn gpu_shader(&self) -> &'static str {
        include_str!("gpu/gaussian_blur.wgsl")
    }
}

impl GaussianBlur {
    pub fn apply(&self, image: &mut Image, params: &GaussianBlurParams) {
        if params.sigma <= 0.0 {
            return;
        }

        let width = image.width as usize;
        let height = image.height as usize;
        if width == 0 || height == 0 {
            return;
        }

        let sigma = params.sigma;
        let radius = (3.0 * sigma).ceil() as isize;

        // Precompute 1D Gaussian kernel weights
        let mut kernel = Vec::with_capacity((radius * 2 + 1) as usize);
        let mut weight_sum = 0.0;
        let sigma2 = 2.0 * sigma * sigma;

        for x in -radius..=radius {
            let weight = (-((x * x) as f32) / sigma2).exp();
            kernel.push(weight);
            weight_sum += weight;
        }

        // Normalize weights
        for w in &mut kernel {
            *w /= weight_sum;
        }

        // 1. Horizontal Blur Pass: src -> temp
        let mut temp = vec![Color::default(); width * height];
        let src_colors = image.colors();

        for y in 0..height {
            let y_offset = y * width;
            for x in 0..width {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut a = 0.0;
                let mut w_total = 0.0;

                for dx in -radius..=radius {
                    let px = (x as isize + dx).clamp(0, width as isize - 1) as usize;
                    let color = src_colors[y_offset + px];
                    let weight = kernel[(dx + radius) as usize];

                    r += color.r as f32 * weight;
                    g += color.g as f32 * weight;
                    b += color.b as f32 * weight;
                    a += color.a as f32 * weight;
                    w_total += weight;
                }

                temp[y_offset + x] = Color {
                    r: (r / w_total).clamp(0.0, 255.0) as u8,
                    g: (g / w_total).clamp(0.0, 255.0) as u8,
                    b: (b / w_total).clamp(0.0, 255.0) as u8,
                    a: (a / w_total).clamp(0.0, 255.0) as u8,
                };
            }
        }

        // 2. Vertical Blur Pass: temp -> src
        let dest_colors = image.colors_mut();

        for y in 0..height {
            let y_offset = y * width;
            for x in 0..width {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut a = 0.0;
                let mut w_total = 0.0;

                for dy in -radius..=radius {
                    let py = (y as isize + dy).clamp(0, height as isize - 1) as usize;
                    let color = temp[py * width + x];
                    let weight = kernel[(dy + radius) as usize];

                    r += color.r as f32 * weight;
                    g += color.g as f32 * weight;
                    b += color.b as f32 * weight;
                    a += color.a as f32 * weight;
                    w_total += weight;
                }

                dest_colors[y_offset + x] = Color {
                    r: (r / w_total).clamp(0.0, 255.0) as u8,
                    g: (g / w_total).clamp(0.0, 255.0) as u8,
                    b: (b / w_total).clamp(0.0, 255.0) as u8,
                    a: (a / w_total).clamp(0.0, 255.0) as u8,
                };
            }
        }
    }
}
