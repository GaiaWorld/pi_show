use wasm_bindgen::prelude::*;

use hash::XHashMap;
// use stdweb::unstable::TryInto;
// use stdweb::web::TypedArray;

use bincode;
use debug_info::debug_println;
use ecs::LendMut;
#[cfg(feature = "create_class_by_str")]
use gui::single::style_parse::parse_class_from_string;
use gui::single::Class;
use crate::world::GuiWorld;

/// 在指定上下文中创建一个 文本样式表
///__jsObj: class样式的文本描述
#[cfg(feature = "create_class_by_str")]
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_class(world: u32, class_id: u32, css: &str) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };

    let r = match parse_class_from_string(css) {
        Ok(r) => r,
        Err(e) => {
            debug_println!("{:?}", e);
            return;
        }
    };

    let class_sheet = world.gui.class_sheet.lend_mut();
    class_sheet.borrow_mut().class_map.insert(class_id as usize, r);
}

/// 添加二进制格式的css表
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_class_by_bin(world: u32, bin: &[u8]) {
    let map: XHashMap<usize, Class> = match bincode::deserialize(bin) {
        Ok(r) => r,
        Err(e) => {
            debug_println!("deserialize_class_map error: {:?}", e);
            return;
        }
    };

    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };

    let class_sheet = world.gui.class_sheet.lend_mut();

    class_sheet.borrow_mut().class_map.extend(map.into_iter());
}
