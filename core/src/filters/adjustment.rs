use ddot_core_macros::{Filter, FilterParams};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::image::Image;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdjustmentParams {
    #[param(min = 0.3, max = 3.0, default = 1.0)]
    pub gamma: f32,

    #[param(min = -0.5, max = 0.5, default = 0.0)]
    pub blacks: f32,

    #[param(min = -0.5, max = 0.5, default = 0.0)]
    pub whites: f32,

    #[param(min = -100, max = 100, default = 0)]
    pub contrast: i32,

    #[param(min = 0.0, max = 2.0, default = 1.0)]
    pub saturation: f32,

    #[param(min = -PI, max = PI, default = 0.0)]
    pub hue: f32,
}

#[derive(Filter)]
#[filter(params = AdjustmentParams)]
pub struct Adjustment;
impl Adjustment {
    pub fn apply(&self, image: &mut Image, params: &AdjustmentParams) {
        if params.gamma != 1.0
            || params.blacks != 0.0
            || params.whites != 0.0
            || params.contrast != 0
        {
            let lut = Self::build_lut(params);
            Self::apply_lut(image, &lut);
        }

        if params.saturation != 1.0 {
            Self::apply_saturation(image, params.saturation);
        }

        if params.hue != 0.0 {
            Self::apply_hue(image, params.hue);
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

        let contrast_factor = (259.0 * (params.contrast as f32 + 255.0))
            / (255.0 * (259.0 - params.contrast as f32));

        let mut lut = [0u8; 256];

        for i in 0..256 {
            let mut v = i as f32 / 255.0;

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

    #[inline]
    fn apply_lut(image: &mut Image, lut: &[u8; 256]) {
        for color in image.colors_mut() {
            color.r = lut[color.r as usize];
            color.g = lut[color.g as usize];
            color.b = lut[color.b as usize];
        }
    }

    #[inline]
    fn apply_saturation(image: &mut Image, saturation: f32) {
        for color in image.colors_mut() {
            let mut r = color.r as f32 / 255.0;
            let mut g = color.g as f32 / 255.0;
            let mut b = color.b as f32 / 255.0;

            let gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;

            r = gray + (r - gray) * saturation;
            g = gray + (g - gray) * saturation;
            b = gray + (b - gray) * saturation;

            color.r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            color.g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            color.b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }

    #[inline]
    fn apply_hue(image: &mut Image, hue: f32) {
        let cos_h = hue.cos();
        let sin_h = hue.sin();

        for color in image.colors_mut() {
            let mut r = color.r as f32 / 255.0;
            let mut g = color.g as f32 / 255.0;
            let mut b = color.b as f32 / 255.0;

            // RGB -> YIQ
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            let i = 0.596 * r - 0.274 * g - 0.322 * b;
            let q = 0.211 * r - 0.523 * g + 0.312 * b;

            // Rotate hue
            let i2 = i * cos_h - q * sin_h;
            let q2 = i * sin_h + q * cos_h;

            // YIQ -> RGB
            r = y + 0.956 * i2 + 0.621 * q2;
            g = y - 0.272 * i2 - 0.647 * q2;
            b = y - 1.106 * i2 + 1.703 * q2;

            color.r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            color.g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            color.b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}

