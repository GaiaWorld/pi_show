use ecs::World;

use component::user::*;
use color::Color as CgColor;
use Node;

#[no_mangle]
pub fn set_backgroud_rgba_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    let box_colors = world.fetch_multi::<Node, BoxColor>().unwrap();
    let box_colors = BorrowMut::borrow_mut(&box_color);
    match box_colors.get_write(node as usize) {
        Some(write) => write.set_backgroud_color(Color::RGBA(CgColor::new(r, g, b, a))),
        None => box_colors.insert(BoxColor{
            border_color: CgColor::new(1.0, 1.0, 1.0, 0.0),
            background_color: Color::RGBA(CgColor::new(r, g, b, a)),
        }),
    };
    debug_println!("set_background_color");
}

// 设置一个线性渐变的背景颜色
#[no_mangle]
pub fn set_backgroud_radial_gradient_color(world: u32, node: u32, center_x: f32, center_y: f32, shape: u8, size: u8 ){
    let box_colors = world.fetch_multi::<Node, BoxColor>().unwrap();
    let box_colors = BorrowMut::borrow_mut(&box_color);
    match box_colors.get_write(node as usize) {
        Some(write) => write.set_backgroud_color(Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size))),
        None => box_colors.insert(BoxColor{
            border_color: CgColor::new(1.0, 1.0, 1.0, 0.0),
            background_color: Color::RadialGradient(to_radial_gradient_color(color_and_positions, center_x, center_y, shape, size)),
        }),
    };
    debug_println!("set_backgroud_radial_gradient_color");
}

// 设置一个径向渐变的背景颜色
#[no_mangle]
pub fn set_backgroud_linear_gradient_color(world: u32, node: u32, direction: f32){
    let box_colors = world.fetch_multi::<Node, BoxColor>().unwrap();
    let box_colors = BorrowMut::borrow_mut(&box_color);
    match box_colors.get_write(node as usize) {
        Some(write) => write.set_backgroud_color(Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction))),
        None => box_colors.insert(BoxColor{
            border_color: CgColor::new(1.0, 1.0, 1.0, 0.0),
            background_color: Color::LinearGradient(to_linear_gradient_color(color_and_positions, direction)),
        }),
    };
    debug_println!("set_backgroud_linear_gradient_color");
}

// 设置边框颜色， 类型为rgba
#[no_mangle]
pub fn set_border_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    let box_colors = world.fetch_multi::<Node, BoxColor>().unwrap();
    let box_colors = BorrowMut::borrow_mut(&box_color);
    match box_colors.get_write(node as usize) {
        Some(write) => write.set_border_color(CgColor::new(r, g, b, a)),
        None => box_colors.insert(BoxColor{
            border_color: CgColor::new(r, g, b, a),
            background_color: Color::RGBA(CgColor::new(1.0, 1.0, 1.0, 0.0)),
        }),
    };
    debug_println!("set_border_color");
}

// 设置边框圆角
#[no_mangle]
pub fn set_border_radius(world: u32, node: u32, value: f32){
    let border_radiuss = world.fetch_multi::<Node, BorderRadius>().unwrap();
    let border_radiuss = BorrowMut::borrow_mut(&border_radius);
    match border_radiuss.get_write(node as usize) {
        Some(write) => write.set_0(LengthUnit::Pixel(value)),
        None => border_radiuss.insert(BorderRadius(value)),
    };
    debug_println!("set_border_radius");
}

// 设置边框圆角
#[no_mangle]
pub fn set_border_radius_percent(world: u32, node: u32, value: f32){
    let border_radiuss = world.fetch_multi::<Node, BorderRadius>().unwrap();
    let border_radiuss = BorrowMut::borrow_mut(&border_radius);
    match border_radiuss.get_write(node as usize) {
        Some(write) => write.set_0(LengthUnit::Percent(value)),
        None => border_radiuss.insert(BorderRadius(value)),
    };
    debug_println!("set_border_radius_percent");
}

// 设置阴影颜色
#[no_mangle]
pub fn set_box_shadow_color(world: u32, node: u32, r: f32, g: f32, b: f32, a: f32){
    let box_shadows = world.fetch_multi::<Node, BoxShadow>().unwrap();
    let box_shadows = BorrowMut::borrow_mut(&box_shadow);
    match box_shadows.get_write(node as usize) {
        Some(write) => write.set_color(CgColor::new(r, g, b, a)),
        None => box_shadows.insert(BoxShadow{
            h: 0.0,
            v: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: CgColor::new(r, g, b, a),
        }),
    };
    debug_println!("set_box_shadow_color");
}


// 设置阴影h
#[no_mangle]
pub fn set_box_shadow_h(world: u32, node: u32, value: f32){
    let box_shadows = world.fetch_multi::<Node, BoxShadow>().unwrap();
    let box_shadows = BorrowMut::borrow_mut(&box_shadow);
    match box_shadows.get_write(node as usize) {
        Some(write) => write.set_h(value),
        None => box_shadows.insert(BoxShadow{
            h: value,
            v: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: CgColor::new(0.0, 0.0, 0.0, 0.0),
        }),
    };
    debug_println!("set_box_shadow_h");
}

// 设置阴影v
#[no_mangle]
pub fn set_box_shadow_v(world: u32, node: u32, v: f32){
    let box_shadows = world.fetch_multi::<Node, BoxShadow>().unwrap();
    let box_shadows = BorrowMut::borrow_mut(&box_shadow);
    match box_shadows.get_write(node as usize) {
        Some(write) => write.set_h(value),
        None => box_shadows.insert(BoxShadow{
            h: 0.0,
            v: 1.0,
            blur: 0.0,
            spread: 0.0,
            color: CgColor::new(0.0, 0.0, 0.0, 0.0),
        }),
    };
    debug_println!("set_box_shadow_v");
}

//设置overflow
#[no_mangle]
pub fn set_overflow(world: u32, node: u32, value: bool){
    let box_shadows = world.fetch_multi::<Node, BoxShadow>().unwrap();
    let box_shadows = BorrowMut::borrow_mut(&box_shadow);
    
    debug_println!("set_overflow");
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.component_mgr.get_node_mut(node).set_overflow(value);
}

//设置不透明度
#[no_mangle]
pub fn set_opacity(world: u32, node: u32, value: f32) {
    debug_println!("set_opacity");
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.component_mgr.get_node_mut(node).set_opacity(value);
}

//设置display
#[no_mangle]
pub fn set_display(world: u32, node: u32, value: u8){
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let mut node_ref = NodeWriteRef::new(node, world.component_mgr.node.to_usize(), &mut world.component_mgr);
    node_ref.get_yoga().set_display(unsafe{ transmute(value as u32)});
    node_ref.set_display( unsafe{ transmute(value) });
    debug_println!("set_display"); 
}

//设置visibility, true: visible, false: hidden,	默认true
#[no_mangle]
pub fn set_visibility(world: u32, node: u32, value: bool) {
    debug_println!("set_visibility");
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.component_mgr.get_node_mut(node).set_visibility(value);
}

#[no_mangle]
pub fn set_zindex(world: u32, node: u32, value: i32) {
    debug_println!("set_z_index");
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    world.component_mgr.get_node_mut(node).set_zindex(value as isize);
}