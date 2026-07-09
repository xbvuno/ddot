pub mod algorithms;
pub mod utils;

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;

pub trait DitherAlgorithm {
    type Params;

    fn name(&self) -> &'static str;

    fn definition(&self) -> FilterDefinition;

    fn apply(&self, image: &mut Image, palette: &Palette, params: &Self::Params);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::algorithms::*;
    use crate::image::Color;

    fn test_gradient_image() -> Image {
        // Create a 10x1 gradient image from black to white
        let mut pixels = Vec::new();
        for i in 0..10 {
            let v = (i * 28) as u8; // from 0 to 252
            pixels.extend_from_slice(&[v, v, v, 255]);
        }
        Image {
            width: 10,
            height: 1,
            pixels,
        }
    }

    fn black_white_palette() -> Palette {
        Palette {
            colors: vec![
                Color { r: 0, g: 0, b: 0, a: 255 },
                Color { r: 255, g: 255, b: 255, a: 255 },
            ],
        }
    }

    #[test]
    fn test_floyd_steinberg() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        FloydSteinberg.apply(&mut img, &pal, &FloydSteinbergParams { amount: 1.0 });
        // Check that all pixels are quantized to either black or white
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }

    #[test]
    fn test_atkinson() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        Atkinson.apply(&mut img, &pal, &AtkinsonParams { amount: 1.0 });
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }

    #[test]
    fn test_stucki() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        Stucki.apply(&mut img, &pal, &StuckiParams { amount: 1.0 });
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }

    #[test]
    fn test_burkes() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        Burkes.apply(&mut img, &pal, &BurkesParams { amount: 1.0 });
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }

    #[test]
    fn test_sierra() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        Sierra.apply(&mut img, &pal, &SierraParams { amount: 1.0 });
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }

    #[test]
    fn test_bayer() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        Bayer.apply(&mut img, &pal, &BayerParams { amount: 1.0 });
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }

    #[test]
    fn test_only_palette() {
        let mut img = test_gradient_image();
        let pal = black_white_palette();
        OnlyPalette.apply(&mut img, &pal, &OnlyPaletteParams {});
        for color in img.colors() {
            assert!(color.r == 0 || color.r == 255);
        }
    }
}


