use gloo_utils::format::JsValueSerdeExt;
use htom_core::home_page;
use polyester::page::wasm;
use polyester::page::Page;
use polyester_macro::impl_wasm_page;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct HomePage(home_page::HomePage);

impl_wasm_page!(HomePage);

#[wasm_bindgen(js_name = "homePage")]
pub fn home_page(js_window_size: &JsValue) -> Result<HomePage, JsValue> {
    let window_size = JsValueSerdeExt::into_serde(js_window_size)
        .map_err(|err| format!("Failed to decode window size: {}", err))?;

    Ok(HomePage(home_page::HomePage {
        window_size: Some(window_size),
    }))
}
