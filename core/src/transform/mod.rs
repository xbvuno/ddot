use crate::image::Image;

/// Resizes the image using a nearest-neighbor algorithm.
/// Clamps target dimensions to a maximum of the original dimensions to guarantee downscaling only.
pub fn resize(image: &Image, width: u32, height: u32) -> Image {
    let target_width = width.min(image.width).max(1);
    let target_height = height.min(image.height).max(1);

    if target_width == image.width && target_height == image.height {
        return image.clone();
    }

    let mut new_pixels = Vec::with_capacity((target_width * target_height * 4) as usize);

    for y in 0..target_height {
        // Map target y to source y
        let src_y = (y * image.height) / target_height;
        let src_row_start = (src_y * image.width * 4) as usize;

        for x in 0..target_width {
            // Map target x to source x
            let src_x = (x * image.width) / target_width;
            let src_pixel_start = src_row_start + (src_x * 4) as usize;

            new_pixels.push(image.pixels[src_pixel_start]);     // R
            new_pixels.push(image.pixels[src_pixel_start + 1]); // G
            new_pixels.push(image.pixels[src_pixel_start + 2]); // B
            new_pixels.push(image.pixels[src_pixel_start + 3]); // A
        }
    }

    Image {
        width: target_width,
        height: target_height,
        pixels: new_pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downscale() {
        let image = Image {
            width: 4,
            height: 4,
            pixels: vec![
                255, 0, 0, 255,   0, 255, 0, 255,   0, 0, 255, 255,   255, 255, 255, 255,
                255, 0, 0, 255,   0, 255, 0, 255,   0, 0, 255, 255,   255, 255, 255, 255,
                255, 0, 0, 255,   0, 255, 0, 255,   0, 0, 255, 255,   255, 255, 255, 255,
                255, 0, 0, 255,   0, 255, 0, 255,   0, 0, 255, 255,   255, 255, 255, 255,
            ],
        };

        // Resize down to 2x2
        let scaled = resize(&image, 2, 2);
        assert_eq!(scaled.width, 2);
        assert_eq!(scaled.height, 2);

        // Verify pixels mapped correctly:
        // row 0: col 0 mapped from src col 0 (red), col 1 mapped from src col 2 (blue)
        // row 1: col 0 mapped from src col 0 (red), col 1 mapped from src col 2 (blue)
        assert_eq!(scaled.pixels[0..4], [255, 0, 0, 255]);
        assert_eq!(scaled.pixels[4..8], [0, 0, 255, 255]);
        assert_eq!(scaled.pixels[8..12], [255, 0, 0, 255]);
        assert_eq!(scaled.pixels[12..16], [0, 0, 255, 255]);
    }

    #[test]
    fn test_clamps_to_max_dimensions() {
        let image = Image {
            width: 2,
            height: 2,
            pixels: vec![
                255, 0, 0, 255, 0, 255, 0, 255,
                255, 0, 0, 255, 0, 255, 0, 255,
            ],
        };

        // Requesting a larger size should clamp to original size
        let scaled = resize(&image, 10, 10);
        assert_eq!(scaled.width, 2);
        assert_eq!(scaled.height, 2);
        assert_eq!(scaled.pixels, image.pixels);
    }
}
