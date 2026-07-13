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
}

struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }
}

#[derive(Filter)]
#[filter(params = NoiseParams)]
pub struct Noise;

impl Noise {
    pub fn apply(&self, image: &mut Image, params: &NoiseParams) {
        if params.coverage == 0.0 || params.intensity == 0.0 {
            return;
        }

        // Initialize LCG with a fixed seed so noise is deterministic for the same image
        let mut lcg = Lcg::new(42);

        for color in image.colors_mut() {
            if lcg.next_f32() < params.coverage {
                let mono = lcg.next_f32() * 2.0 - 1.0;
                let nr = lcg.next_f32() * 2.0 - 1.0;
                let ng = lcg.next_f32() * 2.0 - 1.0;
                let nb = lcg.next_f32() * 2.0 - 1.0;

                let sat = params.saturation;
                let noise_r = (mono * (1.0 - sat) + nr * sat) * params.intensity;
                let noise_g = (mono * (1.0 - sat) + ng * sat) * params.intensity;
                let noise_b = (mono * (1.0 - sat) + nb * sat) * params.intensity;

                color.r = ((color.r as f32 + noise_r * 255.0).clamp(0.0, 255.0)) as u8;
                color.g = ((color.g as f32 + noise_g * 255.0).clamp(0.0, 255.0)) as u8;
                color.b = ((color.b as f32 + noise_b * 255.0).clamp(0.0, 255.0)) as u8;
            }
        }
    }
}
