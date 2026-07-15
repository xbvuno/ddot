use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Color {
    pub fn to_hsl(self) -> Hsl {
        let r = channel_to_unit(self.r);
        let g = channel_to_unit(self.g);
        let b = channel_to_unit(self.b);

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        let l = (max + min) * 0.5;

        if chroma == 0.0 {
            return Hsl { h: 0.0, s: 0.0, l };
        }

        let s = chroma / (1.0 - (2.0 * l - 1.0).abs());

        let hue_turns = if max == r {
            ((g - b) / chroma).rem_euclid(6.0)
        } else if max == g {
            (b - r) / chroma + 2.0
        } else {
            (r - g) / chroma + 4.0
        };

        Hsl {
            h: normalize_hue(hue_turns * TAU / 6.0),
            s,
            l,
        }
    }

    pub fn from_hsl(hsl: Hsl, a: u8) -> Self {
        let h = hsl.h.rem_euclid(TAU);
        let s = hsl.s.clamp(0.0, 1.0);
        let l = hsl.l.clamp(0.0, 1.0);

        let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let hue_sector = h / (PI / 3.0);
        let x = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
        let m = l - chroma * 0.5;

        let (r, g, b) = match hue_sector as u32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };

        Self {
            r: unit_to_channel(r + m),
            g: unit_to_channel(g + m),
            b: unit_to_channel(b + m),
            a,
        }
    }
}

fn channel_to_unit(channel: u8) -> f32 {
    channel as f32 / 255.0
}

fn unit_to_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn normalize_hue(hue: f32) -> f32 {
    (hue + PI).rem_euclid(TAU) - PI
}
