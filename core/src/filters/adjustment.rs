use ddot_core_macros::{Filter, FilterParams};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::image::{Color, Image};

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdjustmentParams {
    #[param(min = 0.3, max = 3.0, default = 1.0)]
    pub gamma: f32,

    #[param(min = -0.5, max = 0.5, default = 0.0)]
    pub blacks: f32,

    #[param(min = -0.5, max = 0.5, default = 0.0)]
    pub whites: f32,

    #[param(min = -100, max = 500, default = 0)]
    pub contrast: i32,

    #[param(min = 0.0, max = 10.0, default = 1.0)]
    pub saturation: f32,

    #[param(min = -PI, max = PI, default = 0.0)]
    pub hue: f32,
}

#[derive(Filter)]
#[filter(params = AdjustmentParams)]
pub struct Adjustment;

impl crate::filter::GpuFilter for Adjustment {
    fn gpu_shader(&self) -> &'static str {
        include_str!("gpu/adjustment.wgsl")
    }
}

impl Adjustment {
    pub fn apply(&self, image: &mut Image, params: &AdjustmentParams) {
        let do_lut = params.gamma != 1.0
            || params.blacks != 0.0
            || params.whites != 0.0
            || params.contrast != 0;

        let do_sat = params.saturation != 1.0;
        let do_hue = params.hue != 0.0;

        let lut = if do_lut {
            Some(Self::build_lut(params))
        } else {
            None
        };

        let cos_h = params.hue.cos();
        let sin_h = params.hue.sin();

        for color in image.colors_mut() {
            if let Some(lut) = &lut {
                Self::apply_lut(color, lut);
            }

            if do_sat {
                Self::apply_saturation(color, params.saturation);
            }

            if do_hue {
                Self::apply_hue(color, cos_h, sin_h);
            }
        }
    }

    #[inline(always)]
    fn smoothstep(x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    }

    #[inline]
    fn build_lut(params: &AdjustmentParams) -> [u8; 256] {
        let gamma_exp = 1.0 / params.gamma;

        let contrast_val = (params.contrast as f32).clamp(-255.0, 258.0);
        let contrast_factor = (259.0 * (contrast_val + 255.0))
            / (255.0 * (259.0 - contrast_val));

        let mut lut = [0u8; 256];

        for i in 0..256 {
            let mut v = i as f32 * (1.0 / 255.0);

            if params.gamma != 1.0 {
                v = v.powf(gamma_exp);
            }

             // Blacks
            if params.blacks != 0.0 {
                let shadow = Self::smoothstep(1.0 - v);
                v += params.blacks * shadow;
            }

            // Whites
            if params.whites != 0.0 {
                let highlight = Self::smoothstep(v);
                v += params.whites * highlight;
            }

            if params.contrast != 0 {
                v = (v - 0.5) * contrast_factor + 0.5;
            }

            lut[i] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        }

        lut
    }

    #[inline(always)]
    fn apply_lut(color: &mut Color, lut: &[u8; 256]) {
        color.r = lut[color.r as usize];
        color.g = lut[color.g as usize];
        color.b = lut[color.b as usize];
    }

    #[inline(always)]
    fn apply_saturation(color: &mut Color, saturation: f32) {
        let mut r = color.r as f32 * (1.0 / 255.0);
        let mut g = color.g as f32 * (1.0 / 255.0);
        let mut b = color.b as f32 * (1.0 / 255.0);

        let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        if saturation == 0.0 {
            let gray = (gray * 255.0).round() as u8;
            color.r = gray;
            color.g = gray;
            color.b = gray;
            return;
        }

        r = gray + (r - gray) * saturation;
        g = gray + (g - gray) * saturation;
        b = gray + (b - gray) * saturation;

        color.r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        color.g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        color.b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    }

    #[inline(always)]
    fn apply_hue(color: &mut Color, cos_h: f32, sin_h: f32) {
        let mut r = color.r as f32 * (1.0 / 255.0);
        let mut g = color.g as f32 * (1.0 / 255.0);
        let mut b = color.b as f32 * (1.0 / 255.0);

        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let i = 0.596 * r - 0.274 * g - 0.322 * b;
        let q = 0.211 * r - 0.523 * g + 0.312 * b;

        let i2 = i * cos_h - q * sin_h;
        let q2 = i * sin_h + q * cos_h;

        r = y + 0.956 * i2 + 0.621 * q2;
        g = y - 0.272 * i2 - 0.647 * q2;
        b = y - 1.106 * i2 + 1.703 * q2;

        color.r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        color.g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        color.b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

