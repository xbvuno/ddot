use crate::image::{Color, Image};
use crate::palette::{Palette, PaletteGenerator};

pub struct KMeans;

#[derive(Debug, Clone)]
pub struct KMeansParams {
    pub n_of_colors: u32,
    pub max_iterations: u32,
    pub tolerance: f32,
}

impl PaletteGenerator for KMeans {
    type Params = KMeansParams;

    fn name(&self) -> &'static str {
        "kmeans"
    }

    fn calculate(&self, image: &Image, params: &Self::Params) -> Palette {
        let colors_slice = image.colors();
        if colors_slice.is_empty() || params.n_of_colors == 0 {
            return Palette { colors: Vec::new() };
        }

        // Subsample pixels for speed (limit to 10,000)
        let step = (colors_slice.len() / 10000).max(1);
        let sampled_pixels: Vec<[u8; 3]> = colors_slice
            .iter()
            .step_by(step)
            .map(|c| [c.r, c.g, c.b])
            .collect();

        let k = (params.n_of_colors as usize).min(sampled_pixels.len());
        if k == 0 {
            return Palette { colors: Vec::new() };
        }

        // Initialize centroids by choosing pixels at equal spacing
        let mut centroids = Vec::with_capacity(k);
        for i in 0..k {
            let idx = i * sampled_pixels.len() / k;
            centroids.push([
                sampled_pixels[idx][0] as f32,
                sampled_pixels[idx][1] as f32,
                sampled_pixels[idx][2] as f32,
            ]);
        }

        let max_iter = params.max_iterations.max(1) as usize;
        let tolerance_sq = params.tolerance * params.tolerance;

        for _iter in 0..max_iter {
            let mut sums = vec![[0.0f32; 3]; k];
            let mut counts = vec![0u32; k];

            // Assignment step
            for pixel in &sampled_pixels {
                let mut min_dist_sq = f32::MAX;
                let mut min_idx = 0;

                for (ci, centroid) in centroids.iter().enumerate() {
                    let dr = pixel[0] as f32 - centroid[0];
                    let dg = pixel[1] as f32 - centroid[1];
                    let db = pixel[2] as f32 - centroid[2];
                    let dist_sq = dr * dr + dg * dg + db * db;
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                        min_idx = ci;
                    }
                }

                sums[min_idx][0] += pixel[0] as f32;
                sums[min_idx][1] += pixel[1] as f32;
                sums[min_idx][2] += pixel[2] as f32;
                counts[min_idx] += 1;
            }

            // Update step
            let mut max_movement_sq = 0.0f32;
            for ci in 0..k {
                if counts[ci] > 0 {
                    let new_r = sums[ci][0] / counts[ci] as f32;
                    let new_g = sums[ci][1] / counts[ci] as f32;
                    let new_b = sums[ci][2] / counts[ci] as f32;

                    let dr = new_r - centroids[ci][0];
                    let dg = new_g - centroids[ci][1];
                    let db = new_b - centroids[ci][2];
                    let movement_sq = dr * dr + dg * dg + db * db;
                    max_movement_sq = max_movement_sq.max(movement_sq);

                    centroids[ci] = [new_r, new_g, new_b];
                } else {
                    // Empty cluster: reinitialize centroid to a pixel (pseudo-randomly)
                    let p_idx = (ci * 13) % sampled_pixels.len();
                    centroids[ci] = [
                        sampled_pixels[p_idx][0] as f32,
                        sampled_pixels[p_idx][1] as f32,
                        sampled_pixels[p_idx][2] as f32,
                    ];
                }
            }

            if max_movement_sq < tolerance_sq {
                break;
            }
        }

        let colors = centroids
            .into_iter()
            .map(|c| Color {
                r: c[0].clamp(0.0, 255.0).round() as u8,
                g: c[1].clamp(0.0, 255.0).round() as u8,
                b: c[2].clamp(0.0, 255.0).round() as u8,
                a: 255,
            })
            .collect();

        Palette { colors }
    }
}
