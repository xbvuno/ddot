use crate::image::Image;

fn luma(r: u8, g: u8, b: u8) -> u8 {
    (0.2126 * r as f32 +
     0.7152 * g as f32 +
     0.0722 * b as f32) as u8
}

pub fn grayscale(image: &mut Image) {
    for pixel in image.pixels.chunks_mut(4) {
        let gray = luma(pixel[0], pixel[1], pixel[2]);

        pixel[0] = gray;
        pixel[1] = gray;
        pixel[2] = gray;
    }
}