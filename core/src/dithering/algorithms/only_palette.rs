use ddot_core_macros::FilterParams;
use serde::{Deserialize, Serialize};

use crate::filter::FilterDefinition;
use crate::image::Image;
use crate::palette::Palette;
use crate::dithering::DitherAlgorithm;
use crate::dithering::utils::find_closest_color;

#[derive(Clone, Debug, Deserialize, FilterParams, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct OnlyPaletteParams {}

pub struct OnlyPalette;

impl OnlyPalette {
    pub const NAME: &'static str = "only_palette";

    pub const fn definition() -> FilterDefinition {
        FilterDefinition {
            name: Self::NAME,
            params: <OnlyPaletteParams as crate::filter::FilterParams>::PARAMS,
        }
    }
}

impl DitherAlgorithm for OnlyPalette {
    type Params = OnlyPaletteParams;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn definition(&self) -> FilterDefinition {
        Self::definition()
    }

    fn apply(&self, image: &mut Image, palette: &Palette, _params: &Self::Params) {
        let output_colors = image.colors_mut();
        for color in output_colors.iter_mut() {
            *color = find_closest_color(color.r as f32, color.g as f32, color.b as f32, palette);
        }
    }
}
