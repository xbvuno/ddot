use ddot_core_macros::{
    Filter,
    FilterParams
};

use crate::image::Image;


#[derive(FilterParams)]
pub struct AdjustmentParams {

    #[param(min=-100, max=100, default=0)]
    pub brightness:i32,


    #[param(min=-100, max=100, default=0)]
    pub contrast:i32,
}



#[derive(Filter)]
#[filter(params = AdjustmentParams)]
pub struct Adjustment;



impl Adjustment {

    pub fn apply(
        &self,
        image:&mut Image,
        params:&AdjustmentParams
    ){

        apply_brightness(
            image,
            params.brightness
        );

        apply_contrast(
            image,
            params.contrast
        );
    }
}


fn apply_brightness(
    image: &mut Image,
    brightness: i32,
) {
    for pixel in image.pixels.chunks_mut(4) {
        pixel[0] = adjust_channel(pixel[0], brightness);
        pixel[1] = adjust_channel(pixel[1], brightness);
        pixel[2] = adjust_channel(pixel[2], brightness);
    }
}


fn apply_contrast(
    image: &mut Image,
    contrast: i32,
) {
    let factor =
        (259 * (contrast + 255)) as f32
        / (255 * (259 - contrast)) as f32;

    for pixel in image.pixels.chunks_mut(4) {
        pixel[0] = contrast_channel(pixel[0], factor);
        pixel[1] = contrast_channel(pixel[1], factor);
        pixel[2] = contrast_channel(pixel[2], factor);
    }
}


fn adjust_channel(
    channel: u8,
    amount: i32,
) -> u8 {
    (channel as i32 + amount).clamp(0, 255) as u8
}


fn contrast_channel(
    channel: u8,
    factor: f32,
) -> u8 {
    (factor * (channel as f32 - 128.0) + 128.0)
        .round()
        .clamp(0.0, 255.0) as u8
}


#[cfg(test)]
mod tests {
    use super::{
        Adjustment,
        AdjustmentParams,
    };
    use crate::filter::{
        Filter,
        FilterParams,
        ParamError,
    };


    #[test]
    fn derives_snake_case_filter_name() {
        let filter =
            Adjustment;

        assert_eq!(
            filter.name(),
            "adjustment"
        );
    }


    #[test]
    fn validates_generated_param_ranges() {
        let params =
            AdjustmentParams {
                brightness: 101,
                contrast: 0,
            };

        assert_eq!(
            params.validate(),
            Err(ParamError::OutOfRange {
                name: "brightness",
                value: "101".to_string(),
            })
        );
    }
}
