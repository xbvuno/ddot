pub mod generators;

use serde::Serialize;
use crate::image::Color;

#[derive(Debug, Clone, Serialize)]
pub struct Palette {
    pub colors: Vec<Color>,
}

pub trait PaletteGenerator {
    type Params;

    fn name(&self) -> &'static str;

    fn calculate(&self, image: &crate::image::Image, params: &Self::Params) -> Palette;
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::generators::{MedianCut, MedianCutParams, Octree, OctreeParams, KMeans, KMeansParams};
    use crate::image::Image;

    fn create_test_image() -> Image {
        let mut pixels = Vec::new();
        for _ in 0..10 {
            pixels.extend_from_slice(&[255, 0, 0, 255]); // Red
        }
        for _ in 0..10 {
            pixels.extend_from_slice(&[0, 255, 0, 255]); // Green
        }
        for _ in 0..10 {
            pixels.extend_from_slice(&[0, 0, 255, 255]); // Blue
        }
        for _ in 0..10 {
            pixels.extend_from_slice(&[255, 255, 0, 255]); // Yellow
        }

        Image {
            width: 40,
            height: 1,
            pixels,
        }
    }

    #[test]
    fn test_median_cut() {
        let image = create_test_image();
        let generator = MedianCut;
        let palette = generator.calculate(&image, &MedianCutParams { n_of_colors: 4 });
        assert_eq!(palette.colors.len(), 4);
        
        let mut has_red = false;
        let mut has_green = false;
        let mut has_blue = false;
        let mut has_yellow = false;

        for c in palette.colors {
            if c.r > 200 && c.g < 50 && c.b < 50 { has_red = true; }
            if c.r < 50 && c.g > 200 && c.b < 50 { has_green = true; }
            if c.r < 50 && c.g < 50 && c.b > 200 { has_blue = true; }
            if c.r > 200 && c.g > 200 && c.b < 50 { has_yellow = true; }
        }

        assert!(has_red);
        assert!(has_green);
        assert!(has_blue);
        assert!(has_yellow);
    }

    #[test]
    fn test_octree() {
        let image = create_test_image();
        let generator = Octree;
        let palette = generator.calculate(&image, &OctreeParams { n_of_colors: 4 });
        assert_eq!(palette.colors.len(), 4);

        let mut has_red = false;
        let mut has_green = false;
        let mut has_blue = false;
        let mut has_yellow = false;

        for c in palette.colors {
            if c.r > 200 && c.g < 50 && c.b < 50 { has_red = true; }
            if c.r < 50 && c.g > 200 && c.b < 50 { has_green = true; }
            if c.r < 50 && c.g < 50 && c.b > 200 { has_blue = true; }
            if c.r > 200 && c.g > 200 && c.b < 50 { has_yellow = true; }
        }

        assert!(has_red);
        assert!(has_green);
        assert!(has_blue);
        assert!(has_yellow);
    }

    #[test]
    fn test_kmeans() {
        let image = create_test_image();
        let generator = KMeans;
        let palette = generator.calculate(&image, &KMeansParams {
            n_of_colors: 4,
            max_iterations: 10,
            tolerance: 0.1,
        });
        assert_eq!(palette.colors.len(), 4);

        let mut has_red = false;
        let mut has_green = false;
        let mut has_blue = false;
        let mut has_yellow = false;

        for c in palette.colors {
            if c.r > 200 && c.g < 50 && c.b < 50 { has_red = true; }
            if c.r < 50 && c.g > 200 && c.b < 50 { has_green = true; }
            if c.r < 50 && c.g < 50 && c.b > 200 { has_blue = true; }
            if c.r > 200 && c.g > 200 && c.b < 50 { has_yellow = true; }
        }

        assert!(has_red);
        assert!(has_green);
        assert!(has_blue);
        assert!(has_yellow);
    }
}

