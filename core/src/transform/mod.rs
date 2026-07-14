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

/// Crops the image by cutting off pixels from the top, left, right, and bottom edges.
/// Clamps dimensions to at least 1x1.
pub fn crop(image: &Image, top: u32, left: u32, right: u32, bottom: u32) -> Image {
    let target_width = image.width.saturating_sub(left).saturating_sub(right).max(1);
    let target_height = image.height.saturating_sub(top).saturating_sub(bottom).max(1);

    // Secure the starting offsets so they don't go out of bounds
    let x_start = left.min(image.width.saturating_sub(target_width)) as usize;
    let y_start = top.min(image.height.saturating_sub(target_height)) as usize;

    let mut new_pixels = Vec::with_capacity((target_width * target_height * 4) as usize);

    for y in 0..target_height {
        let src_y = y_start + y as usize;
        let src_row_start = src_y * image.width as usize * 4;
        let src_pixel_start = src_row_start + x_start * 4;
        let src_pixel_end = src_pixel_start + target_width as usize * 4;

        new_pixels.extend_from_slice(&image.pixels[src_pixel_start..src_pixel_end]);
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
    fn test_crop() {
        let image = Image {
            width: 4,
            height: 4,
            pixels: vec![
                1, 2, 3, 4,     5, 6, 7, 8,     9, 10, 11, 12,    13, 14, 15, 16,
                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,   29, 30, 31, 32,
                33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,   45, 46, 47, 48,
                49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60,   61, 62, 63, 64,
            ],
        };

        // Crop 1px from each side -> should return the center 2x2 area
        let cropped = crop(&image, 1, 1, 1, 1);
        assert_eq!(cropped.width, 2);
        assert_eq!(cropped.height, 2);

        let expected = vec![
            21, 22, 23, 24, 25, 26, 27, 28,
            37, 38, 39, 40, 41, 42, 43, 44,
        ];
        assert_eq!(cropped.pixels, expected);
    }

    #[test]
    fn test_crop_max_clamp() {
        let image = Image {
            width: 2,
            height: 2,
            pixels: vec![
                1, 2, 3, 4,     5, 6, 7, 8,
                9, 10, 11, 12,  13, 14, 15, 16,
            ],
        };

        // Crop 10px from all sides -> should clamp to 1x1 image at the bottom-rightmost pixel
        let cropped = crop(&image, 10, 10, 10, 10);
        assert_eq!(cropped.width, 1);
        assert_eq!(cropped.height, 1);
        assert_eq!(cropped.pixels, vec![13, 14, 15, 16]);
    }

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
