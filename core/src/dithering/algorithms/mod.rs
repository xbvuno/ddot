mod atkinson;
mod bayer;
mod burkes;
mod floyd_steinberg;
mod sierra;
mod only_palette;
mod stucki;

pub use atkinson::{Atkinson, AtkinsonParams};
pub use bayer::{Bayer, BayerParams};
pub use burkes::{Burkes, BurkesParams};
pub use floyd_steinberg::{FloydSteinberg, FloydSteinbergParams};
pub use only_palette::{OnlyPalette, OnlyPaletteParams};
pub use sierra::{Sierra, SierraParams};
pub use stucki::{Stucki, StuckiParams};

