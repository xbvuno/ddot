use crate::image::Color;
use crate::palette::Palette;

#[inline]
pub fn find_closest_color(r: f32, g: f32, b: f32, palette: &Palette) -> Color {
    if palette.colors.is_empty() {
        return Color {
            r: r.clamp(0.0, 255.0) as u8,
            g: g.clamp(0.0, 255.0) as u8,
            b: b.clamp(0.0, 255.0) as u8,
            a: 255,
        };
    }

    let mut min_dist_sq = f32::MAX;
    let mut closest = palette.colors[0];

    for &color in &palette.colors {
        let dr = r - color.r as f32;
        let dg = g - color.g as f32;
        let db = b - color.b as f32;
        let dist_sq = dr * dr + dg * dg + db * db;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            closest = color;
        }
    }

    closest
}
