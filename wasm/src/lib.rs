use ddot_core::{filters, image::Image as CoreImage};
use js_sys::{Array, Reflect};
use wasm_bindgen::{Clamped, prelude::*};
use web_sys::ImageData;

#[wasm_bindgen(js_name = Image)]
pub struct WasmImage {
    inner: CoreImage,
}

#[wasm_bindgen(js_class = Image)]
impl WasmImage {
    /// Creates an image from ImageData.
    #[wasm_bindgen(constructor)]
    pub fn new(image: ImageData) -> WasmImage {
        WasmImage {
            inner: CoreImage {
                width: image.width(),
                height: image.height(),
                pixels: image.data().0,
            },
        }
    }

    /// Deep copy.
    #[wasm_bindgen]
    pub fn clone(&self) -> WasmImage {
        WasmImage {
            inner: self.inner.clone(),
        }
    }

    /// Width.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Height.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Returns a copy of the pixel buffer.
    #[wasm_bindgen(getter)]
    pub fn pixels(&self) -> Vec<u8> {
        self.inner.pixels.clone()
    }

    /// Converts back to browser ImageData.
    #[wasm_bindgen(js_name = toImageData)]
    pub fn to_image_data(&self) -> Result<ImageData, JsValue> {
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.inner.pixels),
            self.inner.width,
            self.inner.height,
        )
    }
}

#[wasm_bindgen]
pub struct Filters;

#[wasm_bindgen]
impl Filters {
    #[wasm_bindgen(js_name = getFilters)]
    pub fn get_filters() -> Result<Array, JsValue> {
        let output = Array::new();

        for filter in filters::FILTERS {
            let handle = FilterHandle::new(filter.name);
            let value = JsValue::from(handle);

            output.push(&value);

            Reflect::set(&output, &JsValue::from_str(filter.name), &value)?;
        }

        Ok(output)
    }

    #[wasm_bindgen(js_name = getFilterNames)]
    pub fn get_filter_names() -> Array {
        filters::filter_names().map(JsValue::from_str).collect()
    }
}

#[wasm_bindgen]
pub struct FilterHandle {
    name: String,
}

#[wasm_bindgen]
impl FilterHandle {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> FilterHandle {
        FilterHandle {
            name: name.to_owned(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let definition = filters::filter_definition(&self.name)
            .ok_or_else(|| JsValue::from_str(&format!("unknown filter '{}'", self.name)))?;

        serde_wasm_bindgen::to_value(&definition.params)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    #[wasm_bindgen(js_name = apply)]
    pub fn apply(&self, image: &mut WasmImage, settings: JsValue) -> Result<(), JsValue> {
        let settings = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        filters::apply_filter(&mut image.inner, &self.name, settings)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }
}
