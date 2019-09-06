use stdweb::unstable::TryInto;

use ecs::LendMut;
use gui::single::{style_parse::{parse_class_from_string} };

use GuiWorld;


/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
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