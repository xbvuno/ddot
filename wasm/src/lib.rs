use ddot_core::{
    filters,
    image::Image as CoreImage,
    palette::{
        generators::{KMeans, KMeansParams, MedianCut, MedianCutParams, Octree, OctreeParams},
        Palette as CorePalette, PaletteGenerator,
    },
    dithering::{
        DitherAlgorithm,
        algorithms::{
            FloydSteinberg, FloydSteinbergParams,
            Atkinson, AtkinsonParams,
            Stucki, StuckiParams,
            Burkes, BurkesParams,
            Sierra, SierraParams,
            Bayer, BayerParams,
            OnlyPalette, OnlyPaletteParams,
        },
    },
    transform,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Auto = 0,
    Cpu = 1,
    Gpu = 2,
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

    #[wasm_bindgen(getter, js_name = backendSupport)]
    pub fn backend_support(&self) -> Result<JsValue, JsValue> {
        let support = filters::filter_backend_support(&self.name)
            .ok_or_else(|| JsValue::from_str(&format!("unknown filter '{}'", self.name)))?;

        serde_wasm_bindgen::to_value(&support)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let definition = filters::filter_definition(&self.name)
            .ok_or_else(|| JsValue::from_str(&format!("unknown filter '{}'", self.name)))?;

        serde_wasm_bindgen::to_value(&definition.params)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    #[wasm_bindgen(js_name = apply)]
    pub async fn apply(
        &self,
        image: &mut WasmImage,
        settings: JsValue,
        backend: JsValue,
    ) -> Result<(), JsValue> {
        let settings = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        let backend_str = if backend.is_undefined() || backend.is_null() {
            "auto"
        } else if let Some(s) = backend.as_string() {
            match s.as_str() {
                "cpu" => "cpu",
                "gpu" => "gpu",
                _ => "auto",
            }
        } else if let Some(n) = backend.as_f64() {
            match n as i32 {
                1 => "cpu",
                2 => "gpu",
                _ => "auto",
            }
        } else {
            "auto"
        };

        filters::apply_filter(&mut image.inner, &self.name, settings, backend_str)
            .await
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

#[wasm_bindgen]
pub struct Dithering;

#[wasm_bindgen]
impl Dithering {
    #[wasm_bindgen(js_name = getAlgorithms)]
    pub fn get_algorithms() -> Result<Array, JsValue> {
        let output = Array::new();

        let f = JsValue::from(WasmFloydSteinberg::new());
        let a = JsValue::from(WasmAtkinson::new());
        let s = JsValue::from(WasmStucki::new());
        let b = JsValue::from(WasmBurkes::new());
        let si = JsValue::from(WasmSierra::new());
        let ba = JsValue::from(WasmBayer::new());
        let op = JsValue::from(WasmOnlyPalette::new());

        output.push(&f);
        output.push(&a);
        output.push(&s);
        output.push(&b);
        output.push(&si);
        output.push(&ba);
        output.push(&op);

        Reflect::set(&output, &JsValue::from_str("FloydSteinberg"), &f)?;
        Reflect::set(&output, &JsValue::from_str("Atkinson"), &a)?;
        Reflect::set(&output, &JsValue::from_str("Stucki"), &s)?;
        Reflect::set(&output, &JsValue::from_str("Burkes"), &b)?;
        Reflect::set(&output, &JsValue::from_str("Sierra"), &si)?;
        Reflect::set(&output, &JsValue::from_str("Bayer"), &ba)?;
        Reflect::set(&output, &JsValue::from_str("OnlyPalette"), &op)?;

        Ok(output)
    }

    #[wasm_bindgen(getter, js_name = Algorithms)]
    pub fn algorithms() -> Result<JsValue, JsValue> {
        let obj = js_sys::Object::new();
        Reflect::set(&obj, &JsValue::from_str("FloydSteinberg"), &JsValue::from(WasmFloydSteinberg::new()))?;
        Reflect::set(&obj, &JsValue::from_str("Atkinson"), &JsValue::from(WasmAtkinson::new()))?;
        Reflect::set(&obj, &JsValue::from_str("Stucki"), &JsValue::from(WasmStucki::new()))?;
        Reflect::set(&obj, &JsValue::from_str("Burkes"), &JsValue::from(WasmBurkes::new()))?;
        Reflect::set(&obj, &JsValue::from_str("Sierra"), &JsValue::from(WasmSierra::new()))?;
        Reflect::set(&obj, &JsValue::from_str("Bayer"), &JsValue::from(WasmBayer::new()))?;
        Reflect::set(&obj, &JsValue::from_str("OnlyPalette"), &JsValue::from(WasmOnlyPalette::new()))?;
        Ok(obj.into())
    }
}

#[wasm_bindgen(js_name = OnlyPaletteDither)]
pub struct WasmOnlyPalette;

#[wasm_bindgen(js_class = OnlyPaletteDither)]
impl WasmOnlyPalette {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "only_palette".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, _settings: JsValue) -> Result<(), JsValue> {
        let alg = OnlyPalette;
        let core_params = OnlyPaletteParams {};
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}


#[wasm_bindgen(js_name = FloydSteinbergDither)]
pub struct WasmFloydSteinberg;

#[wasm_bindgen(js_class = FloydSteinbergDither)]
impl WasmFloydSteinberg {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "floyd_steinberg".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_amount = js_sys::Object::new();
        Reflect::set(&p_amount, &JsValue::from_str("name"), &JsValue::from_str("amount"))?;
        Reflect::set(&p_amount, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_amount, &JsValue::from_str("min"), &JsValue::from(0.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("default"), &JsValue::from(1.0))?;
        params.push(&p_amount.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, settings: JsValue) -> Result<(), JsValue> {
        let amount = if let Some(n) = settings.as_f64() {
            n as f32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("amount")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0)
        };

        let alg = FloydSteinberg;
        let core_params = FloydSteinbergParams { amount };
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}

#[wasm_bindgen(js_name = AtkinsonDither)]
pub struct WasmAtkinson;

#[wasm_bindgen(js_class = AtkinsonDither)]
impl WasmAtkinson {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "atkinson".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_amount = js_sys::Object::new();
        Reflect::set(&p_amount, &JsValue::from_str("name"), &JsValue::from_str("amount"))?;
        Reflect::set(&p_amount, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_amount, &JsValue::from_str("min"), &JsValue::from(0.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("default"), &JsValue::from(1.0))?;
        params.push(&p_amount.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, settings: JsValue) -> Result<(), JsValue> {
        let amount = if let Some(n) = settings.as_f64() {
            n as f32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("amount")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0)
        };

        let alg = Atkinson;
        let core_params = AtkinsonParams { amount };
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}

#[wasm_bindgen(js_name = StuckiDither)]
pub struct WasmStucki;

#[wasm_bindgen(js_class = StuckiDither)]
impl WasmStucki {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "stucki".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_amount = js_sys::Object::new();
        Reflect::set(&p_amount, &JsValue::from_str("name"), &JsValue::from_str("amount"))?;
        Reflect::set(&p_amount, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_amount, &JsValue::from_str("min"), &JsValue::from(0.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("default"), &JsValue::from(1.0))?;
        params.push(&p_amount.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, settings: JsValue) -> Result<(), JsValue> {
        let amount = if let Some(n) = settings.as_f64() {
            n as f32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("amount")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0)
        };

        let alg = Stucki;
        let core_params = StuckiParams { amount };
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}

#[wasm_bindgen(js_name = BurkesDither)]
pub struct WasmBurkes;

#[wasm_bindgen(js_class = BurkesDither)]
impl WasmBurkes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "burkes".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_amount = js_sys::Object::new();
        Reflect::set(&p_amount, &JsValue::from_str("name"), &JsValue::from_str("amount"))?;
        Reflect::set(&p_amount, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_amount, &JsValue::from_str("min"), &JsValue::from(0.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("default"), &JsValue::from(1.0))?;
        params.push(&p_amount.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, settings: JsValue) -> Result<(), JsValue> {
        let amount = if let Some(n) = settings.as_f64() {
            n as f32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("amount")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0)
        };

        let alg = Burkes;
        let core_params = BurkesParams { amount };
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}

#[wasm_bindgen(js_name = SierraDither)]
pub struct WasmSierra;

#[wasm_bindgen(js_class = SierraDither)]
impl WasmSierra {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "sierra".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_amount = js_sys::Object::new();
        Reflect::set(&p_amount, &JsValue::from_str("name"), &JsValue::from_str("amount"))?;
        Reflect::set(&p_amount, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_amount, &JsValue::from_str("min"), &JsValue::from(0.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("default"), &JsValue::from(1.0))?;
        params.push(&p_amount.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, settings: JsValue) -> Result<(), JsValue> {
        let amount = if let Some(n) = settings.as_f64() {
            n as f32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("amount")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0)
        };

        let alg = Sierra;
        let core_params = SierraParams { amount };
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}

#[wasm_bindgen(js_name = BayerDither)]
pub struct WasmBayer;

#[wasm_bindgen(js_class = BayerDither)]
impl WasmBayer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        "bayer".to_owned()
    }

    #[wasm_bindgen(js_name = getParams)]
    pub fn get_params(&self) -> Result<JsValue, JsValue> {
        let params = Array::new();
        let p_amount = js_sys::Object::new();
        Reflect::set(&p_amount, &JsValue::from_str("name"), &JsValue::from_str("amount"))?;
        Reflect::set(&p_amount, &JsValue::from_str("type"), &JsValue::from_str("float"))?;
        Reflect::set(&p_amount, &JsValue::from_str("min"), &JsValue::from(0.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("max"), &JsValue::from(1.0))?;
        Reflect::set(&p_amount, &JsValue::from_str("default"), &JsValue::from(1.0))?;
        params.push(&p_amount.into());
        Ok(params.into())
    }

    #[wasm_bindgen]
    pub fn apply(&self, image: &mut WasmImage, palette: &WasmPalette, settings: JsValue) -> Result<(), JsValue> {
        let amount = if let Some(n) = settings.as_f64() {
            n as f32
        } else {
            let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            val.get("amount")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0)
        };

        let alg = Bayer;
        let core_params = BayerParams { amount };
        alg.apply(&mut image.inner, &palette.inner, &core_params);
        Ok(())
    }
}

#[wasm_bindgen]
pub struct Transform;

#[wasm_bindgen]
impl Transform {
    #[wasm_bindgen(js_name = Resize)]
    pub fn resize(image: &WasmImage, settings: JsValue) -> Result<WasmImage, JsValue> {
        let val = serde_wasm_bindgen::from_value::<serde_json::Value>(settings)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let width = val.get("width")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| JsValue::from_str("missing width parameter"))?;

        let height = val.get("height")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| JsValue::from_str("missing height parameter"))?;

        let scaled = transform::resize(&image.inner, width, height);
        Ok(WasmImage { inner: scaled })
    }
}



