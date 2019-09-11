use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
use hash::XHashMap;

use bincode;
use ecs::LendMut;
#[cfg(feature="create_class_by_str")]
use gui::single::{style_parse::{parse_class_from_string} };
use gui::single::Class;

use GuiWorld;

/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
#[cfg(feature="create_class_by_str")]
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_class(world: u32, class_id: u32) {
    let value: String = js!(return __jsObj;).try_into().unwrap();
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};

    let r = match parse_class_from_string(value.as_str()) {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            return;
        },
    };

    let class_sheet = world.gui.class_sheet.lend_mut();
    class_sheet.class_map.insert(class_id as usize, r);
}

// 添加二进制格式的css表
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_class_by_bin(world: u32) {
    let value: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let value = value.to_vec();
    let map: XHashMap<usize, Class> = match bincode::deserialize(value.as_slice()) {
        Ok(r) => r,
        Err(e) => {
            println!("deserialize_class_map error: {:?}", e);
            return;
        },
    };

    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};

    let class_sheet = world.gui.class_sheet.lend_mut();

    class_sheet.class_map.extend(map.into_iter());
}