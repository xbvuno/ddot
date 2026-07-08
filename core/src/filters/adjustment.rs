use ddot_core_macros::{Filter, FilterParams};

use crate::image::Image;

#[derive(FilterParams)]
pub struct AdjustmentParams {
    #[param(min=-100, max=100, default=0)]
    pub brightness: i32,

    #[param(min=-100, max=100, default=0)]
    pub contrast: i32,
}

#[derive(Filter)]
#[filter(params = AdjustmentParams)]
pub struct Adjustment;

impl Adjustment {
    pub fn apply(&self, image: &mut Image, params: &AdjustmentParams) {
        apply_brightness(image, params.brightness);

        apply_contrast(image, params.contrast);
    }
}

fn apply_brightness(image: &mut Image, brightness: i32) {
    let offset = brightness * 255 / 100;

    for color in image.colors_mut() {
        color.r = offset_channel(color.r, offset);
        color.g = offset_channel(color.g, offset);
        color.b = offset_channel(color.b, offset);
    }
}

fn apply_contrast(image: &mut Image, contrast: i32) {
    let contrast = contrast as f32 * 255.0 / 100.0;

    let factor = 259.0 * (contrast + 255.0) / (255.0 * (259.0 - contrast));

    for color in image.colors_mut() {
        color.r = contrast_channel(color.r, factor);
        color.g = contrast_channel(color.g, factor);
        color.b = contrast_channel(color.b, factor);
    }
}

fn offset_channel(channel: u8, amount: i32) -> u8 {
    (channel as i32 + amount).clamp(0, 255) as u8
}

fn contrast_channel(channel: u8, factor: f32) -> u8 {
    (factor * (channel as f32 - 128.0) + 128.0)
        .round()
        .clamp(0.0, 255.0) as u8
}

