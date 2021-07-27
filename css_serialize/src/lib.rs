#![feature(link_args)]

extern crate wasm_bindgen;
extern crate bincode;
extern crate fx_hashmap;
extern crate gui;
#[macro_use]
extern crate serde;

use fx_hashmap::FxHashMap32;
use gui::single::style_parse::parse_class_map_from_string;
use gui::single::Class;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize)]
pub struct Result {
	pub err: Option<String>,
	pub bin: Option<Vec<u8>>,
}


/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn serialize_class_map(value: &str) -> JsValue {
    let r = match parse_class_map_from_string(value) {
        Ok(r) => match bincode::serialize(&r) {
            Ok(bin) => Result{err: None, bin: Some(bin)},
            Err(r) => Result{err: Some(r.to_string()), bin: None},
        },
        Err(r) => Result{err: Some(r), bin: None}
    };

	JsValue::from_serde(&r).unwrap()
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn deserialize_class_map(bin: &[u8]) {
    let r: FxHashMap32<usize, Class> = match bincode::deserialize(bin) {
        Ok(r) => r,
        Err(e) => {
            println!("deserialize_class_map error: {:?}", e);
            return;
        }
    };
    // println!("r: {:?}", r);
}

#[wasm_bindgen]
pub fn init_log() {
    pi_web_logger::init_with_level(pi_web_logger::Level::Info);
	log::info!("init_logger ok!");
}
