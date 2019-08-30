use std::mem::transmute;

use serde::{Serialize};

use ecs::{Lend};
use gui::component::user::*;
use gui::system::util::cal_matrix;
// use gui::single::Oct;
// use gui::layout::FlexNode;
use GuiWorld;

// 打印节点信息
#[allow(unused_attributes)]
#[no_mangle]
pub fn node_info(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;

    let z_depth = unsafe { world.z_depth.lend().get_unchecked(node) }.0;

    let enable = unsafe { world.enable.lend().get_unchecked(node) }.0;
    let visibility = unsafe { world.visibility.lend().get_unchecked(node) }.0;

    let by_overflow = unsafe { world.by_overflow.lend().get_unchecked(node) }.0;

    let opacity = unsafe { world.opacity.lend().get_unchecked(node) }.0;

    let layout = world.layout.lend();

    let world_matrix = world.world_matrix.lend();

    let transform = world.transform.lend();

    let world_matrix1 = cal_matrix(node, world_matrix, transform, layout, &Transform::default());
    let layout = unsafe { layout.get_unchecked(node) };
    
    // border box
    let b_left_top = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    let b_left_bottom = world_matrix1 * Vector4::new(0.0, layout.height, 1.0, 1.0);
    let b_right_bottom = world_matrix1 * Vector4::new(layout.width, layout.height, 1.0, 1.0);
    let b_right_top = world_matrix1 * Vector4::new(layout.width, 0.0, 1.0, 1.0);
    
    // border box
    let absolute_b_box = Quad {
        left_top: Point2::new(b_left_top.x, b_left_top.y),
        left_bottom: Point2::new(b_left_bottom.x, b_left_bottom.y),
        right_bottom: Point2::new(b_right_bottom.x, b_right_bottom.y),
        right_top: Point2::new(b_right_top.x, b_right_top.y),
    };

    // padding box
    let p_left_top = world_matrix1 * Vector4::new(layout.border_left, layout.border_top, 1.0, 1.0);
    let p_left_bottom = world_matrix1 * Vector4::new(layout.border_left, layout.height - layout.border_bottom, 1.0, 1.0);
    let p_right_bottom = world_matrix1 * Vector4::new(layout.width - layout.border_right, layout.height - layout.border_bottom, 1.0, 1.0);
    let p_right_top = world_matrix1 * Vector4::new(layout.width - layout.border_right, layout.border_top, 1.0, 1.0);

    let absolute_p_box = Quad {
        left_top: Point2::new(p_left_top.x, p_left_top.y),
        left_bottom: Point2::new(p_left_bottom.x, p_left_bottom.y),
        right_bottom: Point2::new(p_right_bottom.x, p_right_bottom.y),
        right_top: Point2::new(p_right_top.x, p_right_top.y),
    };

    // content box
    let c_left_top = world_matrix1 * Vector4::new(layout.border_left + layout.padding_left, layout.border_top + layout.padding_top, 1.0, 1.0);
    let c_left_bottom = world_matrix1 * Vector4::new(layout.border_left + layout.padding_left, layout.height - layout.border_bottom - layout.padding_bottom, 1.0, 1.0);
    let c_right_bottom = world_matrix1 * Vector4::new(layout.width - layout.border_right - layout.padding_right, layout.height - layout.border_bottom - layout.padding_bottom, 1.0, 1.0);
    let c_right_top = world_matrix1 * Vector4::new(layout.width - layout.border_right - layout.padding_right, layout.border_top + layout.padding_top, 1.0, 1.0);
    
    let absolute_c_box = Quad {
        left_top: Point2::new(c_left_top.x, c_left_top.y),
        left_bottom: Point2::new(c_left_bottom.x, c_left_bottom.y),
        right_bottom: Point2::new(c_right_bottom.x, c_right_bottom.y),
        right_top: Point2::new(c_right_top.x, c_right_top.y),
    };

    let info = Info {
        by_overflow: by_overflow,
        visibility: visibility,
        enable: enable,
        opacity: opacity,
        zindex: z_depth,
        layout: unsafe { transmute(layout.clone()) },
        border_box: absolute_b_box,
        padding_box: absolute_p_box,
        content_box: absolute_c_box,
    };


    // let yogas = world.yoga.lend();
    // let yoga = unsafe { yogas.get_unchecked(node) };

    let octs = world.oct.lend();
    let oct = unsafe { octs.get_unchecked(node) };

    js!{
        window.__jsObj = @{info};
        // window.__jsObj1 = window.__jsObj;
        // console.log("node_info:", window.__jsObj);
        // console.log("style:", @{format!( "{:?}", yoga.get_style() )});
        // console.log("layout:", @{format!( "{:?}", yoga.get_layout() )});
        console.log("boundBox:", @{format!( "{:?}", oct )});
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn overflow_clip(_world: u32) {
    // let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	// let world = &mut world.gui;
    // let overflow_clip = world.overflow_clip.lend();
    // js!{
    //     console.log("overflow_clip:", @{format!("{:?}", **overflow_clip)});
    // }
}

// 调试使用， 设置渲染脏， 使渲染系统在下一帧进行渲染
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_render_dirty(world: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let render_objs = world.render_objs.lend();
    
    render_objs.get_notify().modify_event(1, "", 0); 
}

// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn bound_box(world: u32, node: u32) {
//     let node = node as usize
//     let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	// let world = &mut world.gui;
//     let overflow_clip = world.fetch_single::<OverflowClip>().unwrap();
//     js!{
//         console.log("overflow_clip:", @{format!("{:?}", &overflow_clip.value)});
//     }
// }

#[derive(Serialize)]
struct Point2{
    x: f32, 
    y: f32,
}
js_serializable!( Point2 );

impl Point2 {
    fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
}

#[derive(Serialize)]
struct Quad{
    left_top: Point2,
    left_bottom: Point2,
    right_bottom: Point2,
    right_top: Point2,
}
js_serializable!( Quad );

#[derive(Serialize)]
struct Layout1{
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    border_left: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    padding_left: f32,
    padding_top: f32,
    padding_right: f32,
    padding_bottom: f32,
}
js_serializable!( Layout1 );

#[derive(Serialize)]
struct Info{
    by_overflow: usize,
    visibility: bool,
    enable: bool,
    opacity: f32,
    zindex: f32,
    layout: Layout1,
    border_box: Quad,
    padding_box: Quad,
    content_box: Quad,
}
js_serializable!( Info );