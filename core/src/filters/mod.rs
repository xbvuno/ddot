mod adjustment;
mod noise;
mod gaussian_blur;
mod kawase_blur;

use crate::{
    filter::{Filter, FilterDefinition, FilterError, FilterParams, BackendSupport},
    image::Image,
};

pub use adjustment::{Adjustment, AdjustmentParams};
pub use noise::{Noise, NoiseParams};
pub use gaussian_blur::{GaussianBlur, GaussianBlurParams};
pub use kawase_blur::{KawaseBlur, KawaseBlurParams};

pub const FILTERS: &[FilterDefinition] = &[
    Adjustment::definition(),
    Noise::definition(),
    GaussianBlur::definition(),
    KawaseBlur::definition(),
];

pub fn filter_names() -> impl Iterator<Item = &'static str> {
    FILTERS.iter().map(|filter| filter.name)
}

pub fn filter_definition(name: &str) -> Option<&'static FilterDefinition> {
    FILTERS.iter().find(|filter| filter.name == name)
}

pub fn filter_backend_support(name: &str) -> Option<BackendSupport> {
    match name {
        Adjustment::NAME => Some(Adjustment.backend_support()),
        Noise::NAME => Some(Noise.backend_support()),
        GaussianBlur::NAME => Some(GaussianBlur.backend_support()),
        KawaseBlur::NAME => Some(KawaseBlur.backend_support()),
        _ => None,
    }
}

pub fn apply_filter(
    image: &mut Image,
    name: &str,
    settings: serde_json::Value,
) -> Result<(), FilterError> {
    match name {
        Adjustment::NAME => {
            let params: AdjustmentParams = serde_json::from_value(settings)?;
            params.validate()?;
            Adjustment.apply(image, &params);
            Ok(())
        }

        Noise::NAME => {
            let params: NoiseParams = serde_json::from_value(settings)?;
            params.validate()?;
            Noise.apply(image, &params);
            Ok(())
        }

        GaussianBlur::NAME => {
            let params: GaussianBlurParams = serde_json::from_value(settings)?;
            params.validate()?;
            GaussianBlur.apply(image, &params);
            Ok(())
        }

        KawaseBlur::NAME => {
            let params: KawaseBlurParams = serde_json::from_value(settings)?;
            params.validate()?;
            KawaseBlur.apply(image, &params);
            Ok(())
        }

        _ => Err(FilterError::UnknownFilter(name.to_owned())),
    }
}

pub fn filter_gpu_shader(name: &str) -> Option<&'static str> {
    match name {
        Adjustment::NAME => Adjustment.gpu_shader(),
        Noise::NAME => Noise.gpu_shader(),
        GaussianBlur::NAME => GaussianBlur.gpu_shader(),
        KawaseBlur::NAME => KawaseBlur.gpu_shader(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::filter::BackendSupport;
    use super::*;

    #[test]
    fn exposes_adjustment_metadata() {
        let definition = filter_definition("adjustment").expect("adjustment filter");

        assert_eq!(definition.params.len(), 6);
        assert_eq!(definition.params[0].name, "gamma");
        assert_eq!(definition.params[0].min, Some(0.3));
        assert_eq!(definition.params[0].max, Some(3.0));
        assert_eq!(definition.params[0].default, 1.0);
        assert_eq!(definition.params[5].name, "hue");
    }

    #[test]
    fn applies_filter_from_serialized_settings() {
        let mut image = Image {
            width: 1,
            height: 1,
            pixels: vec![10, 20, 30, 255],
        };

        apply_filter(
            &mut image,
            "adjustment",
            json!({
                "gamma": 1.0,
                "blacks": 0.0,
                "whites": 0.0,
                "contrast": 0,
                "saturation": 1.0,
                "hue": 0.0,
            }),
        )
        .expect("apply filter");

        assert_eq!(image.pixels, vec![10, 20, 30, 255]);
    }

    #[test]
    fn applies_hsl_adjustments() {
        let mut image = Image {
            width: 1,
            height: 1,
            pixels: vec![255, 0, 0, 255],
        };

        apply_filter(
            &mut image,
            "adjustment",
            json!({
                "saturation": 0.0,
                "hue": 0.0,
            }),
        )
        .expect("apply filter");

        assert_eq!(image.pixels, vec![54, 54, 54, 255]);
    }

    #[test]
    fn rejects_out_of_range_serialized_settings() {
        let mut image = Image {
            width: 1,
            height: 1,
            pixels: vec![10, 20, 30, 255],
        };

        let error = apply_filter(
            &mut image,
            "adjustment",
            json!({
                "gamma": 3.1,
            }),
        )
        .expect_err("invalid params");

        assert!(error.to_string().contains("gamma"));
    }

    #[test]
    fn verifies_filter_backend_support_capabilities() {
        assert_eq!(filter_backend_support("adjustment"), Some(BackendSupport::CpuAndGpu));
        let adj_shader = Adjustment.gpu_shader().expect("adjustment gpu shader");
        assert!(adj_shader.contains("@compute"));

        // Noise now supports GPU
        assert_eq!(filter_backend_support("noise"), Some(BackendSupport::CpuAndGpu));
        let noise_shader = Noise.gpu_shader().expect("noise gpu shader");
        assert!(noise_shader.contains("@compute"));

        // KawaseBlur supports GPU
        assert_eq!(filter_backend_support("kawase_blur"), Some(BackendSupport::CpuAndGpu));
        let kawase_shader = KawaseBlur.gpu_shader().expect("kawase blur gpu shader");
        assert!(kawase_shader.contains("@compute"));
    }
}
