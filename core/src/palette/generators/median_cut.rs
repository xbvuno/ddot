use std::collections::HashMap;
use crate::image::{Color, Image};
use crate::palette::{Palette, PaletteGenerator};

pub struct MedianCut;

#[derive(Debug, Clone)]
pub struct MedianCutParams {
    pub n_of_colors: u32,
}

struct Bucket {
    start: usize,
    end: usize,
    total_weight: u64,
}

impl PaletteGenerator for MedianCut {
    type Params = MedianCutParams;

    fn name(&self) -> &'static str {
        "median_cut"
    }

    fn calculate(&self, image: &Image, params: &Self::Params) -> Palette {
        let colors_slice = image.colors();
        if colors_slice.is_empty() || params.n_of_colors == 0 {
            return Palette { colors: Vec::new() };
        }

        // 1. Gather unique colors and counts
        let mut color_counts: HashMap<[u8; 3], u32> = HashMap::new();
        for color in colors_slice {
            let rgb = [color.r, color.g, color.b];
            *color_counts.entry(rgb).or_insert(0) += 1;
        }

        let mut unique_colors: Vec<([u8; 3], u32)> = color_counts.into_iter().collect();
        let n_colors = params.n_of_colors as usize;

        // 2. Initialize the first bucket
        let mut buckets = Vec::new();
        let initial_weight: u64 = unique_colors.iter().map(|(_, count)| *count as u64).sum();
        buckets.push(Bucket {
            start: 0,
            end: unique_colors.len(),
            total_weight: initial_weight,
        });

        // 3. Iteratively split buckets
        while buckets.len() < n_colors {
            // Find the bucket with the largest weight that has more than 1 color
            let mut best_bucket_idx = None;
            let mut max_weight = 0;

            for (i, bucket) in buckets.iter().enumerate() {
                let len = bucket.end - bucket.start;
                if len > 1 && bucket.total_weight > max_weight {
                    max_weight = bucket.total_weight;
                    best_bucket_idx = Some(i);
                }
            }

            let bucket_idx = match best_bucket_idx {
                Some(idx) => idx,
                None => break, // No more buckets can be split
            };

            let bucket = buckets.remove(bucket_idx);
            let slice = &mut unique_colors[bucket.start..bucket.end];

            // Find min/max values for R, G, B channels
            let mut min_r = u8::MAX;
            let mut max_r = u8::MIN;
            let mut min_g = u8::MAX;
            let mut max_g = u8::MIN;
            let mut min_b = u8::MAX;
            let mut max_b = u8::MIN;

            for &(rgb, _) in slice.iter() {
                min_r = min_r.min(rgb[0]);
                max_r = max_r.max(rgb[0]);
                min_g = min_g.min(rgb[1]);
                max_g = max_g.max(rgb[1]);
                min_b = min_b.min(rgb[2]);
                max_b = max_b.max(rgb[2]);
            }

            let range_r = max_r - min_r;
            let range_g = max_g - min_g;
            let range_b = max_b - min_b;

            // Choose the channel with the largest range
            let channel_idx = if range_r >= range_g && range_r >= range_b {
                0
            } else if range_g >= range_r && range_g >= range_b {
                1
            } else {
                2
            };

            // Sort slice by chosen channel
            slice.sort_by_key(|&(rgb, _)| rgb[channel_idx]);

            // Find median weight split point
            let mut running_weight = 0;
            let mut split_idx = bucket.start;

            for (i, &(_, count)) in slice.iter().enumerate() {
                running_weight += count as u64;
                if running_weight >= bucket.total_weight / 2 {
                    split_idx = bucket.start + i;
                    break;
                }
            }

            // Boundary checks to ensure no empty buckets
            if split_idx == bucket.start {
                split_idx = bucket.start + 1;
            }
            if split_idx >= bucket.end {
                split_idx = bucket.end - 1;
            }

            // Calculate weights for the two new buckets
            let weight_left: u64 = unique_colors[bucket.start..split_idx]
                .iter()
                .map(|(_, count)| *count as u64)
                .sum();
            let weight_right: u64 = unique_colors[split_idx..bucket.end]
                .iter()
                .map(|(_, count)| *count as u64)
                .sum();

            buckets.push(Bucket {
                start: bucket.start,
                end: split_idx,
                total_weight: weight_left,
            });
            buckets.push(Bucket {
                start: split_idx,
                end: bucket.end,
                total_weight: weight_right,
            });
        }

        // 4. Calculate weighted average color for each bucket
        let mut palette_colors = Vec::with_capacity(buckets.len());
        for bucket in buckets {
            let slice = &unique_colors[bucket.start..bucket.end];
            if bucket.total_weight == 0 {
                continue;
            }

            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            for &(rgb, count) in slice {
                let w = count as f64;
                sum_r += rgb[0] as f64 * w;
                sum_g += rgb[1] as f64 * w;
                sum_b += rgb[2] as f64 * w;
            }

            let total_w = bucket.total_weight as f64;
            palette_colors.push(Color {
                r: (sum_r / total_w).round() as u8,
                g: (sum_g / total_w).round() as u8,
                b: (sum_b / total_w).round() as u8,
                a: 255,
            });
        }

        Palette { colors: palette_colors }
    }
}
