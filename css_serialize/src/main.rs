#![feature(link_args)]

#[macro_use]
extern crate stdweb;
extern crate bincode;
extern crate fx_hashmap;
extern crate gui;

use fx_hashmap::FxHashMap32;
use gui::single::style_parse::parse_class_map_from_string;
use gui::single::Class;
use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
#[allow(unused_attributes)]
#[no_mangle]
pub fn serialize_class_map() {
    let value: String = js!(return __jsObj;).try_into().unwrap();
    match parse_class_map_from_string(value.as_str()) {
        Ok(r) => match bincode::serialize(&r) {
            Ok(bin) => {
                let bin = TypedArray::<u8>::from(bin.as_slice());
                js! {
                    __jsObj = @{bin};
                }
                return;
            }
            Err(r) => {
                js! {__jsObj = @{r.to_string()};};
            }
        },
        Err(r) => {
            js! {__jsObj = @{r};};
        }
    };
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn deserialize_class_map() {
    let value: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let value = value.to_vec();
    let r: FxHashMap32<usize, Class> = match bincode::deserialize(value.as_slice()) {
        Ok(r) => r,
        Err(e) => {
            println!("deserialize_class_map error: {:?}", e);
            return;
        }
    };
    // println!("r: {:?}", r);
}

fn main() {}
