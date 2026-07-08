mod core;
mod filters;


use wasm_bindgen::prelude::*;

use filters::{
    Adjustment,
    filter::Filter,
};


#[wasm_bindgen]
pub fn get_filters() -> JsValue {

    let filters = vec![
        Adjustment::definition(),
    ];

    serde_wasm_bindgen::to_value(&filters)
        .unwrap()
}