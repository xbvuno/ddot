use ddot_core_macros::{Filter, FilterParams};
use serde::{Deserialize, Serialize};

use crate::image::Image;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NoiseParams {
    #[param(min = 0.0, max = 1.0, default = 0.0)]
    pub coverage: f32,

    #[param(min = 0.0, max = 1.0, default = 0.0)]
    pub intensity: f32,

    #[param(min = 0.0, max = 1.0, default = 0.0)]
    pub saturation: f32,

    #[param(min = 0.0, max = 100.0, default = 0.0)]
    pub phase: f32,
}

#[derive(Filter)]
#[filter(params = NoiseParams)]
pub struct Noise;

impl crate::filter::GpuFilter for Noise {
    fn gpu_shader(&self) -> &'static str {
        include_str!("gpu/noise.wgsl")
    }
}

#[inline(always)]
fn hash_2d(x: f32, y: f32, seed: f32) -> f32 {
    let t = ((x + seed * 0.17) * 12.9898 + (y + seed * 1.31) * 78.233).sin() * 43758.5453;
    t - t.floor()
}

impl Noise {
    pub fn apply(&self, image: &mut Image, params: &NoiseParams) {
        if params.coverage == 0.0 || params.intensity == 0.0 {
            return;
        }

        let width = image.width as usize;
        let height = image.height as usize;
        let seed = params.phase + 1.0;
        let colors = image.colors_mut();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                
                // Coordinate-based noise matching the GPU shader
                if hash_2d(x as f32, y as f32, seed) < params.coverage {
                    let mono = hash_2d(x as f32, y as f32, seed + 1.0) * 2.0 - 1.0;
                    let nr = hash_2d(x as f32, y as f32, seed + 2.0) * 2.0 - 1.0;
                    let ng = hash_2d(x as f32, y as f32, seed + 3.0) * 2.0 - 1.0;
                    let nb = hash_2d(x as f32, y as f32, seed + 4.0) * 2.0 - 1.0;

                    let sat = params.saturation;
                    let noise_r = (mono * (1.0 - sat) + nr * sat) * params.intensity;
                    let noise_g = (mono * (1.0 - sat) + ng * sat) * params.intensity;
                    let noise_b = (mono * (1.0 - sat) + nb * sat) * params.intensity;

                    let color = &mut colors[idx];
                    color.r = ((color.r as f32 + noise_r * 255.0).clamp(0.0, 255.0)) as u8;
                    color.g = ((color.g as f32 + noise_g * 255.0).clamp(0.0, 255.0)) as u8;
                    color.b = ((color.b as f32 + noise_b * 255.0).clamp(0.0, 255.0)) as u8;
                }
            }
        }
    }
}
