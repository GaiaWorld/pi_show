use stdweb::unstable::TryInto;

use ecs::LendMut;

use gui::component::calc::StyleType;
use gui::single::{ class::*, style_parse::{Attribute, parse_class_from_string} };
pub use gui::layout::{ YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType };
use GuiWorld;


/**
 * 在指定上下文中创建一个 文本样式表
 * __jsObj: class样式的文本描述
 */
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_class(world: u32, class_id: u32) {
    let value: String = js!(return __jsObj;).try_into().unwrap();

    let r = match parse_class_from_string(value.as_str()) {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            return;
        },
    };

    set_class(world, class_id, r);
}

fn set_class(world: u32, class_id: u32, r: (Vec<Attribute>, Vec<LayoutAttr>)) {
    let class_id = class_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let class_sheet = world.gui.class_sheet.lend_mut();
    // let (mut text_style, mut text_style_change) = (TextStyle::default(), false);
    // let (mut bg_color, mut bg_color_change) = (BackgroundColor::default(), false);
    let mut class = Class::default();
    class.layout =  r.1;

    for attr in r.0.into_iter() {
        match attr {
            Attribute::BGColor(r) => {
                class.background_color = class_sheet.background_color.insert(r);
                class.class_style_mark |= StyleType::BackgroundColor as usize;
            },
            _ => println!("set_class error"),
        }
    }

    let c = class_sheet.class.insert(class);

    class_sheet.class_map.insert(class_id, c);

    let notify = class_sheet.get_notify();
    notify.create_event(c);
}