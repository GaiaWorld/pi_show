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
use render::engine::Engine;
use render::res::{TextureRes};
use layout::*;
use Z_MAX;


#[allow(unused_attributes)]
#[no_mangle]
pub fn node_info(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut World)};
    let idtree = world.fetch_single::<IdTree>().unwrap();
    let idtree = idtree.borrow();

    let z_depth = world.fetch_multi::<Node, ZDepth>().unwrap();
    let z_depth = unsafe { z_depth.lend().get_unchecked(node) }.0 as u32;

    let enable = world.fetch_multi::<Node, Enable>().unwrap();
    let enable = unsafe { enable.lend().get_unchecked(node) }.0;

    let visibility = world.fetch_multi::<Node, Visibility>().unwrap();
    let visibility = unsafe { visibility.lend().get_unchecked(node) }.0;

    let by_overflow = world.fetch_multi::<Node, ByOverflow>().unwrap();
    let by_overflow = unsafe { by_overflow.lend().get_unchecked(node) }.0 as u32;

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
    


    let b_left_top = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    let b_left_bottom = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    let b_left_top = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    let b_left_top = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    
    js!{
        console.log(@{format!("{:?}", layout)});
    };
}

// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn node_layout(world: u32, node: u32) {
//     let node = node as usize;
//     let world = unsafe {&mut *(world as usize as *mut World)};

//     let layout = world.fetch_multi::<Node, Layout>().unwrap();
//     let layout = unsafe { layout.lend().get_unchecked(node) };

//     js!{
//       window._____info = layout: {
//           width: @{layout.width},
//           height: @{layout.height},
//           border_left: @{layout.border_left},
//           border_top: @{layout.border_top},
//           border_right: @{layout.border_right},
//           border_bottom: @{layout.border_bottom},
//           padding_left: @{layout.padding_left},
//           padding_top: @{layout.padding_top},
//           padding_right: @{layout.padding_right},
//           padding_bottom: @{layout.padding_bottom},
//           left: @{layout.left},
//           top: @{layout.top}
//         },
//       };
//       console.log(_____info);
//       window._____info = null;
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