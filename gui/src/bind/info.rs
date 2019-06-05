use std::{
  sync::Arc,
  usize::MAX as UMAX,
  f32::INFINITY as FMAX,
  mem::transmute,
};

use stdweb::unstable::TryInto;
use stdweb::web::html_element::{ImageElement, CanvasElement};

use ecs::{World, Lend, LendMut, MultiCaseImpl, SingleCaseImpl};
use ecs::idtree::{IdTree, InsertType};
use hal_core::*;
use hal_webgl::*;
use atom::Atom;
use octree::intersects;
use cg2d::{include_quad2, InnOuter};

use component::user::*;
use component::calc::*;
use component::calc::Opacity as COpacity;
use single::oct::Oct;
use single::{ OverflowClip};
use entity::Node;


#[allow(unused_attributes)]
#[no_mangle]
pub fn node_info(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.borrow();

    let z_depth = world.fetch_multi::<Node, ZDepth>().unwrap();
    let z_depth = unsafe { z_depth.lend().get_unchecked(node) }.0;

    let enable = world.fetch_multi::<Node, Enable>().unwrap();
    let enable = unsafe { enable.lend().get_unchecked(node) }.0;

    let visibility = world.fetch_multi::<Node, Visibility>().unwrap();
    let visibility = unsafe { visibility.lend().get_unchecked(node) }.0;

    let by_overflow = world.fetch_multi::<Node, ByOverflow>().unwrap();
    let by_overflow = unsafe { by_overflow.lend().get_unchecked(node) }.0;

    let opacity = world.fetch_multi::<Node, COpacity>().unwrap();
    let opacity = unsafe { opacity.lend().get_unchecked(node) }.0;

    let hsv = world.fetch_multi::<Node, HSV>().unwrap();
    let hsv = unsafe { hsv.lend().get_unchecked(node) };

    let layout = world.fetch_multi::<Node, Layout>().unwrap();
    let layout = layout.lend();

    let world_matrix = world.fetch_multi::<Node, WorldMatrix>().unwrap();
    let world_matrix = world_matrix.lend();

    let transform = world.fetch_multi::<Node, Transform>().unwrap();
    let transform = transform.lend();

    let world_matrix1 = cal_matrix(node, world_matrix, transform, layout);
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
        layout: layout.clone(),
        border_box: absolute_b_box,
        padding_box: absolute_p_box,
        content_box: absolute_c_box,
    };

    js!{
        console.log("node_info:", @{format!("{:?}", info)});
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn overflow_clip(world: u32) {
    let world = unsafe {&mut *(world as usize as *mut World)};
    let overflow_clip = world.fetch_single::<OverflowClip>().unwrap();
    let overflow_clip = overflow_clip.lend();
    js!{
        console.log("overflow_clip:", @{format!("{:?}", **overflow_clip)});
    }
}

// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn bound_box(world: u32, node: u32) {
//     let node = node as usize
//     let world = unsafe {&mut *(world as usize as *mut World)};
//     let overflow_clip = world.fetch_single::<OverflowClip>().unwrap();
//     js!{
//         console.log("overflow_clip:", @{format!("{:?}", &overflow_clip.value)});
//     }
// }

fn cal_matrix(
    id: usize,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    layouts: &MultiCaseImpl<Node, Layout>,
) -> Matrix4 {
    let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
    let transform = unsafe { transforms.get_unchecked(id) };
    let layout = unsafe { layouts.get_unchecked(id) };

    let origin = transform.origin.to_value(layout.width, layout.height);

    if origin.x != 0.0 || origin.y != 0.0 {
        return world_matrix.0 * Matrix4::from_translation(Vector3::new(-origin.x, -origin.y, 0.0));
    }
    
    world_matrix.0.clone()
}

#[derive(Debug)]
struct Quad{
    left_top: Point2,
    left_bottom: Point2,
    right_bottom: Point2,
    right_top: Point2,
}

#[derive(Debug)]
struct Info{
    by_overflow: usize,
    visibility: bool,
    enable: bool,
    opacity: f32,
    zindex: f32,
    layout: Layout,
    border_box: Quad,
    padding_box: Quad,
    content_box: Quad,
}