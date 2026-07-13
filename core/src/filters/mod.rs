mod adjustment;
mod noise;

use crate::{
    filter::{Filter, FilterDefinition, FilterError, FilterParams, BackendSupport},
    image::Image,
};

pub use adjustment::{Adjustment, AdjustmentParams};
pub use noise::{Noise, NoiseParams};

pub const FILTERS: &[FilterDefinition] = &[
    Adjustment::definition(),
    Noise::definition(),
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
        _ => None,
    }
}

async fn dispatch_filter<F: Filter>(
    filter: &F,
    image: &mut Image,
    params: &F::Params,
    backend: &str,
) -> Result<(), FilterError> {
    match backend {
        "cpu" => {
            filter.apply(image, params);
            Ok(())
        }
        "gpu" => {
            if let Some(shader) = filter.gpu_shader() {
                let params_bytes = params.to_bytes();
                match crate::filter::gpu::run_gpu(shader, image, &params_bytes).await {
                    Ok(()) => Ok(()),
                    Err(FilterError::GpuUnavailable) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::warn_1(&format!("Filter '{}' failed to run on GPU: WebGPU is not available, falling back to CPU", filter.name()).into());
                        #[cfg(not(target_arch = "wasm32"))]
                        eprintln!("Warning: Filter '{}' failed to run on GPU: WebGPU is not available, falling back to CPU", filter.name());

                        filter.apply(image, params);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::warn_1(&format!("Filter '{}' does not support GPU backend, falling back to CPU", filter.name()).into());
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("Warning: Filter '{}' does not support GPU backend, falling back to CPU", filter.name());

                filter.apply(image, params);
                Ok(())
            }
        }
        _ => { // "auto"
            if let Some(shader) = filter.gpu_shader() {
                let params_bytes = params.to_bytes();
                match crate::filter::gpu::run_gpu(shader, image, &params_bytes).await {
                    Ok(()) => Ok(()),
                    Err(FilterError::GpuUnavailable) => {
                        // Silent fallback for auto mode
                        filter.apply(image, params);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                filter.apply(image, params);
                Ok(())
            }
        }
    }
}

pub async fn apply_filter(
    image: &mut Image,
    name: &str,
    settings: serde_json::Value,
    backend: &str,
) -> Result<(), FilterError> {
    match name {
        Adjustment::NAME => {
            let params: AdjustmentParams = serde_json::from_value(settings)?;
            params.validate()?;
            dispatch_filter(&Adjustment, image, &params, backend).await
        }

        Noise::NAME => {
            let params: NoiseParams = serde_json::from_value(settings)?;
            params.validate()?;
            dispatch_filter(&Noise, image, &params, backend).await
        }

        _ => Err(FilterError::UnknownFilter(name.to_owned())),
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

        pollster::block_on(apply_filter(
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
            "cpu",
        ))
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

        pollster::block_on(apply_filter(
            &mut image,
            "adjustment",
            json!({
                "saturation": 0.0,
                "hue": 0.0,
            }),
            "cpu",
        ))
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

        let error = pollster::block_on(apply_filter(
            &mut image,
            "adjustment",
            json!({
                "gamma": 3.1,
            }),
            "cpu",
        ))
        .expect_err("invalid params");

        assert!(error.to_string().contains("gamma"));
    }

    #[test]
    fn verifies_filter_backend_support_capabilities() {
        assert_eq!(filter_backend_support("adjustment"), Some(BackendSupport::CpuAndGpu));
        let adj_shader = Adjustment.gpu_shader().expect("adjustment gpu shader");
        assert!(adj_shader.contains("@compute"));

        assert_eq!(filter_backend_support("noise"), Some(BackendSupport::CpuOnly));
        assert!(Noise.gpu_shader().is_none());
    }
}
