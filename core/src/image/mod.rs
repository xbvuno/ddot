mod color;

pub use color::Color;

use bytemuck::{cast_slice, cast_slice_mut};

#[derive(Clone)]
pub struct Image {
    pub width: u32,

    pub height: u32,

    pub pixels: Vec<u8>,
}

impl Image {
    /// Immutable view of the pixels as colors.
    #[inline]
    pub fn colors(&self) -> &[Color] {
        cast_slice(&self.pixels)
    }

    /// Mutable view of the pixels as colors.
    #[inline]
    pub fn colors_mut(&mut self) -> &mut [Color] {
        cast_slice_mut(&mut self.pixels)
    }
}
