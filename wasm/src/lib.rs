use ddot_core::{
    filters,
    image::Image as CoreImage,
    palette::{
        generators::{KMeans, KMeansParams, MedianCut, MedianCutParams, Octree, OctreeParams},
        Palette as CorePalette, PaletteGenerator,
    },
};
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

#[wasm_bindgen(js_name = Palette)]
pub struct WasmPalette {
    inner: CorePalette,
}

#[wasm_bindgen(js_class = Palette)]
impl WasmPalette {
    #[wasm_bindgen(getter)]
    pub fn colors(&self) -> Array {
        let array = Array::new();
        for color in &self.inner.colors {
            let obj = js_sys::Object::new();
            Reflect::set(&obj, &JsValue::from_str("r"), &JsValue::from(color.r)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("g"), &JsValue::from(color.g)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("b"), &JsValue::from(color.b)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("a"), &JsValue::from(color.a)).unwrap();
            array.push(&obj.into());
        }
        array
    }
}

#[wasm_bindgen]
pub struct Palettes;

#[wasm_bindgen]
impl Palettes {
    #[wasm_bindgen(getter, js_name = Generators)]
    pub fn generators() -> Result<JsValue, JsValue> {
        let generators_obj = js_sys::Object::new();

        let median_cut = WasmMedianCutGenerator::new();
        Reflect::set(&generators_obj, &JsValue::from_str("MedianCut"), &JsValue::from(median_cut))?;

        let octree = WasmOctreeGenerator::new();
        Reflect::set(&generators_obj, &JsValue::from_str("Octree"), &JsValue::from(octree))?;

        let kmeans = WasmKMeansGenerator::new();
        Reflect::set(&generators_obj, &JsValue::from_str("Kmeans"), &JsValue::from(kmeans))?;

        Ok(generators_obj.into())
    }
}

#[wasm_bindgen(js_name = MedianCutGenerator)]
pub struct WasmMedianCutGenerator;

#[wasm_bindgen(js_class = MedianCutGenerator)]
impl WasmMedianCutGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMedianCutGenerator {
        WasmMedianCutGenerator
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "median_cut".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_n_colors = js_sys::Object::new();
        Reflect::set(&p_n_colors, &JsValue::from_str("name"), &JsValue::from_str("n_of_colors"))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("type"), &JsValue::from_str("int"))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("min"), &JsValue::from(2))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("max"), &JsValue::from(256))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("default"), &JsValue::from(16))?;
        params.push(&p_n_colors.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn calculate(&self, image: &WasmImage, params: JsValue) -> Result<WasmPalette, JsValue> {
        let n_of_colors = if let Some(n) = params.as_f64() {
            n as u32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(params)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("n_of_colors")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(16)
        };

        let generator = MedianCut;
        let core_params = MedianCutParams { n_of_colors };
        let palette = generator.calculate(&image.inner, &core_params);

        Ok(WasmPalette { inner: palette })
    }
}

#[wasm_bindgen(js_name = OctreeGenerator)]
pub struct WasmOctreeGenerator;

#[wasm_bindgen(js_class = OctreeGenerator)]
impl WasmOctreeGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmOctreeGenerator {
        WasmOctreeGenerator
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "octree".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_n_colors = js_sys::Object::new();
        Reflect::set(&p_n_colors, &JsValue::from_str("name"), &JsValue::from_str("n_of_colors"))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("type"), &JsValue::from_str("int"))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("min"), &JsValue::from(2))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("max"), &JsValue::from(256))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("default"), &JsValue::from(16))?;
        params.push(&p_n_colors.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn calculate(&self, image: &WasmImage, params: JsValue) -> Result<WasmPalette, JsValue> {
        let n_of_colors = if let Some(n) = params.as_f64() {
            n as u32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(params)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("n_of_colors")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(16)
        };

        let generator = Octree;
        let core_params = OctreeParams { n_of_colors };
        let palette = generator.calculate(&image.inner, &core_params);

        Ok(WasmPalette { inner: palette })
    }
}

#[wasm_bindgen(js_name = KMeansGenerator)]
pub struct WasmKMeansGenerator;

#[wasm_bindgen(js_class = KMeansGenerator)]
impl WasmKMeansGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmKMeansGenerator {
        WasmKMeansGenerator
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "kmeans".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        
        let p_n_colors = js_sys::Object::new();
        Reflect::set(&p_n_colors, &JsValue::from_str("name"), &JsValue::from_str("n_of_colors"))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("type"), &JsValue::from_str("int"))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("min"), &JsValue::from(2))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("max"), &JsValue::from(256))?;
        Reflect::set(&p_n_colors, &JsValue::from_str("default"), &JsValue::from(16))?;
        params.push(&p_n_colors.into());

        let p_max_iter = js_sys::Object::new();
        Reflect::set(&p_max_iter, &JsValue::from_str("name"), &JsValue::from_str("max_iterations"))?;
        Reflect::set(&p_max_iter, &JsValue::from_str("type"), &JsValue::from_str("int"))?;
        Reflect::set(&p_max_iter, &JsValue::from_str("min"), &JsValue::from(1))?;
        Reflect::set(&p_max_iter, &JsValue::from_str("max"), &JsValue::from(100))?;
        Reflect::set(&p_max_iter, &JsValue::from_str("default"), &JsValue::from(10))?;
        params.push(&p_max_iter.into());

        let p_tolerance = js_sys::Object::new();
        Reflect::set(&p_tolerance, &JsValue::from_str("name"), &JsValue::from_str("tolerance"))?;
        Reflect::set(&p_tolerance, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_tolerance, &JsValue::from_str("min"), &JsValue::from(0.00001))?;
        Reflect::set(&p_tolerance, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_tolerance, &JsValue::from_str("default"), &JsValue::from(0.1))?;
        params.push(&p_tolerance.into());

        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn calculate(&self, image: &WasmImage, params: JsValue) -> Result<WasmPalette, JsValue> {
        let (n_of_colors, max_iterations, tolerance) = if let Some(n) = params.as_f64() {
            (n as u32, 10, 0.1f32)
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(params)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let n = val.get("n_of_colors")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(16);
            let max_iter = val.get("max_iterations")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(10);
            let tol = val.get("tolerance")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(0.1);
            (n, max_iter, tol)
        };

        let generator = KMeans;
        let core_params = KMeansParams {
            n_of_colors,
            max_iterations,
            tolerance,
        };
        let palette = generator.calculate(&image.inner, &core_params);

        Ok(WasmPalette { inner: palette })
    }
}

