use wasm_bindgen::prelude::*;
use pi_style::style_parse::parse_class_map_from_string;
// use stdweb::unstable::TryInto;
// use stdweb::web::TypedArray;

// #[cfg(feature = "create_class_by_str")]
// use pi_style::style_parse::parse_class_map_from_string;
// use gui::single::Class;
use crate::world::GuiWorld;

/// 在指定上下文中创建一个 文本样式表
///__jsObj: class样式的文本描述
#[cfg(feature = "create_class_by_str")]
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_class(world: u32, class_id: usize, css: &str) {

    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };

	let s = ".c".to_string()
		+ class_id.to_string().as_str() 
		+ "{" 
		+ css
		+ "}";
    let r = match parse_class_map_from_string(s.as_str(), 0) {
        Ok(r) => r,
        Err(_e) => {
            debug_println!("{:?}", e);
            return;
        }
    };
	world.gui.create_class(r);
}

/// 添加二进制格式的css表
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_class_by_bin(world: u32, bin: &[u8]) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	world.gui.create_class_by_bin(bin);
}
