use std::{
  usize::MAX as UMAX,
  f32::INFINITY as FMAX,
  mem::transmute,
};

use stdweb::unstable::TryInto;
use stdweb::Object;

use ecs::{Lend, LendMut, MultiCaseImpl, SingleCaseImpl};
use ecs::idtree::{InsertType};
use hal_core::*;
use atom::Atom;
use octree::intersects;
use cg2d::{include_quad2, InnOuter};
use share::Share;

use gui::component::user::*;
use gui::component::calc::*;
use gui::single::*;
use gui::entity::Node;
use gui::system::util::get_or_default;
use gui::render::res::{TextureRes};
use gui::layout::*;
use gui::Z_MAX;

use GuiWorld;
use yoga as yoga1;


fn create(world: &GuiWorld) -> usize {
    let world = &world.gui;
    let idtree = world.idtree.lend_mut();
    let node = world.node.lend_mut().create();
    // println!("!!!!!!create----{}", node);
    let border_radius = world.border_radius.lend_mut();
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});
    world.class_name.lend_mut().insert(node, ClassName(0));
    idtree.create(node);
    node
}

#[allow(unused_attributes)]
#[no_mangle]
fn insert_child(world: u32, child: u32, parent: u32, index: usize){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, index, Some(&notify));
}

//创建容器节点， 容器节点可设置背景颜色
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let node = create(world);
    debug_println!("create_node, node:{}", node);
    node as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_text_node(world: u32) -> u32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let node = create(world);
    debug_println!("create_text_node, node:{}", node);
    node as u32
}

//创建图片节点
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_image_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let node = create(world);
    debug_println!("create_image_node, node:{}", node);
    node as u32
}

// 在尾部插入子节点
#[allow(unused_attributes)]
#[no_mangle]
pub fn append_child(world: u32, child: u32, parent: u32){
    // println!("!!!!!!append----parent: {}, child:{}", parent, child);
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_child(child as usize, parent as usize, UMAX, Some(&notify));
    // println!("xxxxxxxxxxxxxxxxx, append_child, child: {}, parent: {}", child, parent);
    debug_println!("append_child"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn insert_before(world: u32, child: u32, brother: u32){
    // println!("!!!!!!insert before----brother: {}, child:{}", brother, child);
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.insert_brother(child as usize, brother as usize, InsertType::Front, Some(&notify));
    // println!("xxxxxxxxxxxxxxxxx, insert_before, child: {}, brother: {}", child, brother);
    debug_println!("insert_before"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn remove_child(world: u32, node_id: u32){
    // println!("!!!!!!remove_child----{}", node_id);
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
    let node = world.node.lend_mut();
    // idtree.remove(node as usize, Some(&notify));
    idtree.destroy(node_id as usize, true, Some(&notify));
    node.delete(node_id as usize);
    // println!("xxxxxxxxxxxxxxxxx, remove: {}", node);
    debug_println!("remove_child: {}", node_id);  
}

// __jsObj: image, __jsObj1: image_name(String)
// 设置图片的src
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_src(world: u32, node: u32, opacity: u8, compress: u8){
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let yg_nodes = world.yoga.lend_mut();
    let yoga = match yg_nodes.get(node) {
        Some(r) => r,
        None => return,
    };

    let name: String = js!{return __jsObj1}.try_into().unwrap();
    let name = Atom::from(name);
    let engine = world.engine.lend_mut();
    let (width, height, texture) = match engine.res_mgr.get::<TextureRes>(&name) {
        Some(res) => {
            (res.width as u32, res.height as u32, res.clone())
        },
        None => {
            let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
            let height: u32 = js!{return __jsObj.height}.try_into().unwrap();

            let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}) {
                Ok(image_obj) => engine.gl.texture_create_2d_webgl(width, height, 0, PixelFormat::RGBA, DataFormat::UnsignedByte, false, &image_obj).unwrap(),
                Err(s) => panic!("set_src error, {:?}", s),
            };
            // let texture = match TryInto::<ImageElement>::try_into(js!{return __jsObj}) {
            //   Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Image(r)),
            //   Err(_s) => match TryInto::<CanvasElement>::try_into(js!{return __jsObj}){
            //     Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Canvas(r)),
            //     Err(s) => panic!("set_src error, {:?}", s),
            //   },
            // };
            // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
            // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
            let res = engine.res_mgr.create::<TextureRes>(name, TextureRes::new(width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, texture) );
            (width, height, res)
        },
    };

    match yoga.get_width().unit {
        yoga1::YGUnit::YGUnitUndefined | yoga1::YGUnit::YGUnitAuto => yoga.set_width(width as f32),
        _ => (),
    };
    match yoga.get_height().unit {
        yoga1::YGUnit::YGUnitUndefined | yoga1::YGUnit::YGUnitAuto => yoga.set_height(height as f32),
        _ => (),
    };
    
    let image = world.image.lend_mut();
    image.insert(node, Image{src: texture});

    debug_println!("set_src"); 
}


// 设置图片的src
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_src_with_texture(world: u32, node: u32, texture: u32){
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let yg_nodes = world.yoga.lend_mut();
    let yoga = match yg_nodes.get(node) {
        Some(r) => r,
        None => return,
    };

    let texture = unsafe {&*(texture as usize as *const Share<TextureRes>) }.clone();

    match yoga.get_width().unit {
        yoga1::YGUnit::YGUnitUndefined | yoga1::YGUnit::YGUnitAuto => yoga.set_width(texture.width as f32),
        _ => (),
    };
    match yoga.get_height().unit {
        yoga1::YGUnit::YGUnitUndefined | yoga1::YGUnit::YGUnitAuto => yoga.set_height(texture.height as f32),
        _ => (),
    };
    
    let image = world.image.lend_mut();
    image.insert(node, Image{src: texture});

    debug_println!("set_src_with_texture"); 
}
// __jsObj: image, __jsObj1: image_name(String)
// 设置图片的src

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_src(world: u32, node: u32, opacity: u8, compress: u8){
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    if !world.node.lend().is_exist(node){
        return;
    }

    let name: String = js!{return __jsObj1}.try_into().unwrap();
    let name = Atom::from(name);
    let engine = world.engine.lend_mut();
    let texture = match engine.res_mgr.get::<TextureRes>(&name) {
        Some(res) => {
            res.clone()
        },
        None => {
            let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
            let height: u32 = js!{return __jsObj.height}.try_into().unwrap();

            let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}) {
                Ok(image_obj) => engine.gl.texture_create_2d_webgl(width, height, 0, PixelFormat::RGBA, DataFormat::UnsignedByte, false, &image_obj).unwrap(),
                Err(_) => panic!("set_src error"),
            };
            // let texture = match TryInto::<ImageElement>::try_into(js!{return __jsObj}) {
            //   Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Image(r)),
            //   Err(_s) => match TryInto::<CanvasElement>::try_into(js!{return __jsObj}){
            //     Ok(r) => engine.gl.create_texture_2d_webgl(0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &WebGLTextureData::Canvas(r)),
            //     Err(s) => panic!("set_src error, {:?}", s),
            //   },
            // };
            // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
            // gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
            let res = engine.res_mgr.create::<TextureRes>(name, TextureRes::new(width as usize, height as usize, unsafe{transmute(opacity)}, unsafe{transmute(compress)}, texture) );
            res
        },
    };
    
    let image = world.border_image.lend_mut();
    image.insert(node, BorderImage{src: texture});

    debug_println!("set_border_src"); 
}

// 设置图片的src
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_border_src_with_texture(world: u32, node: u32, texture: u32){
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let yg_nodes = world.yoga.lend_mut();
    let yoga = match yg_nodes.get(node) {
        Some(r) => r,
        None => return,
    };

    let texture = unsafe {&*(texture as usize as *mut Share<TextureRes>) }.clone();

    match yoga.get_width().unit {
        yoga1::YGUnit::YGUnitUndefined | yoga1::YGUnit::YGUnitAuto => yoga.set_width(texture.width as f32),
        _ => (),
    };
    match yoga.get_height().unit {
        yoga1::YGUnit::YGUnitUndefined | yoga1::YGUnit::YGUnitAuto => yoga.set_height(texture.height as f32),
        _ => (),
    };
    
    let image = world.border_image.lend_mut();
    image.insert(node, BorderImage{src: texture});

    debug_println!("set_border_src_with_texture"); 
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_top(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    unsafe {world.layout.lend().get_unchecked(node as usize)}.top
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_left(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    unsafe {world.layout.lend().get_unchecked(node as usize)}.left
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_width(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    unsafe {world.layout.lend().get_unchecked(node as usize)}.width
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn offset_height(world: u32, node: u32) -> f32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    unsafe {world.layout.lend().get_unchecked(node as usize)}.height
}

#[no_mangle]
pub fn offset_document(world: u32, node_id: u32) {
  let node_id = node_id as usize;
  let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
  let layouts = world.layout.lend();
  let world_matrixs = world.world_matrix.lend();
  let transforms = world.transform.lend();
  let default_table = world.default_table.lend();

  let transform = get_or_default(node_id, transforms, default_table);
  let layout = unsafe {layouts.get_unchecked(node_id)};
  let origin = transform.origin.to_value(layout.width, layout.height);

  let world_matrix = unsafe {world_matrixs.get_unchecked(node_id)};
  let point = Vector4::new(-origin.x + layout.border_left + layout.padding_left, -origin.y + layout.border_top+ layout.padding_top, 1.0, 1.0);
  let left_top = world_matrix.0 * point;

  js!{
    __jsObj.left = @{left_top.x};
    __jsObj.top = @{left_top.y};
    __jsObj.width = @{layout.width - layout.border_left - layout.padding_left};
    __jsObj.height = @{layout.height - layout.border_top- layout.padding_top};
  }
}

// #[no_mangle]
// pub fn offset_document(world: u32, node_id: u32) {
//   let mut node_id = node_id as usize;
//   let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	// let world = &mut world.gui;
//   let idtree = world.fetch_single::<IdTree>().unwrap();
//   let idtree = idtree.lend();
//   let layouts = world.fetch_multi::<Node, Layout>().unwrap();
//   let layouts = layouts.lend();
  
//   let mut x: f32 = 0.0;
//   let mut y: f32 = 0.0;
//   let layout = unsafe{layouts.get_unchecked(node_id)};
//   x += layout.left;
//   y += layout.top;

//   loop {
//     let node = unsafe {idtree.get_unchecked(node_id)};
//     if node.parent == 0 {
//       break;
//     }
//     let layout = unsafe{layouts.get_unchecked(node.parent)};
//     x += layout.left;
//     y += layout.top;
//     node_id = node.parent;
//   }

//   js!{
//     __jsObj.left = @{x};
//     __jsObj.top = @{y};
//     __jsObj.width = @{layout.width};
//     __jsObj.height = @{layout.height};
//   }
// }

// #[no_mangle]
// pub fn set_event_type(world: u32, node: u32, ty: u8) {
//   let node = node as usize;
//   let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	// let world = &mut world.gui;
//   let node_ref = world.component_mgr.get_node_mut(node);
//   node_ref.set_event_type(ty);
// }

//content宽高的累加值
#[allow(unused_attributes)]
#[no_mangle]
pub fn content_box(world: u32, node: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let layout = world.layout.lend();
    let idtree = world.idtree.borrow();
    let (mut left, mut right, mut top, mut bottom) = (FMAX, 0.0, FMAX, 0.0);
    for (id, _) in idtree.iter(unsafe {idtree.get_unchecked(node as usize)}.children.head) {
      let l = unsafe {layout.get_unchecked(id)};
      let r = l.left + l.width;
      let b = l.top + l.height;
      if l.left < left {
        left = l.left;
      }
      if r > right {
        right = r;
      }
      if b > bottom {
        bottom = b;
      }
      if l.top < top {
        top = l.top;
      }
    }
    js!{
      __jsObj.width = @{right - left};
      __jsObj.height = @{bottom - top}
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn query(world: u32, x: f32, y: f32)-> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;    

    let octree = world.oct.lend();
    let enables = world.enable.lend();
    let overflow_clip = world.overflow_clip.lend();
    let by_overflows = world.by_overflow.lend();
    let z_depths = world.z_depth.lend();

    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(enables, by_overflows, z_depths, overflow_clip, aabb.clone(), 0);
    octree.query(&aabb, intersects, &mut args, ab_query_func);
    args.result as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn iter_query(world: u32, x: f32, y: f32)-> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;

    let entitys = world.node.lend();
    let octree = world.oct.lend();
    let enables = world.enable.lend();
    let overflow_clip = world.overflow_clip.lend();
    let by_overflows = world.by_overflow.lend();
    let z_depths = world.z_depth.lend();

    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(enables, by_overflows, z_depths, overflow_clip, aabb.clone(), 0);

    for e in entitys.iter() {
        let oct = unsafe { octree.get_unchecked(e) };
        ab_query_func(&mut args, e, oct.0, &e);
    }
    args.result as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn set_class(world: u32, node_id: u32, key: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    

    let class_sheet = world.class_sheet.lend();
    match class_sheet.class_map.get(&(key as usize)) {
        Some(class_id) => {
            let class = unsafe { class_sheet.class.get_unchecked(*class_id) };
            let yoga = world.yoga.lend_mut();
            let yoga = unsafe {yoga.get_unchecked_mut(node_id)};
            for layout_attr in class.layout.iter() {
                match layout_attr.clone() {
                    LayoutAttr::AlignContent(r) => yoga.set_align_content(r),
                    LayoutAttr::AlignItems(r) => yoga.set_align_items(r),
                    LayoutAttr::AlignSelf(r) => yoga.set_align_self(r),
                    LayoutAttr::JustifyContent(r) => yoga.set_justify_content(r),
                    LayoutAttr::FlexDirection(r) => yoga.set_flex_direction(r),
                    LayoutAttr::FlexWrap(r) => yoga.set_flex_wrap(r),
                    LayoutAttr::PositionType(r) => yoga.set_position_type(r),
                    LayoutAttr::Width(r) => match r {
                        ValueUnit::Auto => yoga.set_width_auto(),
                        ValueUnit::Undefined => yoga.set_width_auto(),
                        ValueUnit::Pixel(r) => yoga.set_width(r),
                        ValueUnit::Percent(r) => yoga.set_width_percent(r),
                    },
                    LayoutAttr::Height(r) => match r {
                        ValueUnit::Auto => yoga.set_height_auto(),
                        ValueUnit::Undefined => yoga.set_height_auto(),
                        ValueUnit::Pixel(r) => yoga.set_height(r),
                        ValueUnit::Percent(r) => yoga.set_height_percent(r),
                    },
                    LayoutAttr::MarginLeft(r) => match r {
                        ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeLeft),
                        ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeLeft),
                        ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeLeft, r),
                        ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeLeft, r),
                    },
                    LayoutAttr::MarginTop(r) => match r {
                        ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                        ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                        ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeTop, r),
                        ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeTop, r),
                    },
                    LayoutAttr::MarginBottom(r) => match r {
                        ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                        ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeTop),
                        ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeTop, r),
                        ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeTop, r),
                    },
                    LayoutAttr::MarginRight(r) => match r {
                        ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeRight),
                        ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeRight),
                        ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeRight, r),
                        ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeRight, r),
                    },
                    LayoutAttr::Margin(r) => match r {
                        ValueUnit::Auto => yoga.set_margin_auto(YGEdge::YGEdgeAll),
                        ValueUnit::Undefined => yoga.set_margin_auto(YGEdge::YGEdgeAll),
                        ValueUnit::Pixel(r) => yoga.set_margin(YGEdge::YGEdgeAll, r),
                        ValueUnit::Percent(r) => yoga.set_margin_percent(YGEdge::YGEdgeAll, r),
                    },
                   LayoutAttr::PaddingLeft(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeLeft, r),
                        ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeLeft, r),
                        _ => (),
                    },
                    LayoutAttr::PaddingTop(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeTop, r),
                        ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeTop, r),
                        _ => (),
                    },
                    LayoutAttr::PaddingBottom(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeTop, r),
                        ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeTop, r),
                        _ => (),
                    },
                    LayoutAttr::PaddingRight(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeRight, r),
                        ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeRight, r),
                        _ => (),
                    },
                    LayoutAttr::Padding(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_padding(YGEdge::YGEdgeAll, r),
                        ValueUnit::Percent(r) => yoga.set_padding_percent(YGEdge::YGEdgeAll, r),
                        _ => (),
                    },
                    LayoutAttr::BorderLeft(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeLeft, r),
                        // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeLeft, r),
                        _ => (),
                    },
                    LayoutAttr::BorderTop(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeTop, r),
                        // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeTop, r),
                        _ => (),
                    },
                    LayoutAttr::BorderBottom(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeTop, r),
                        // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeTop, r),
                        _ => (),
                    },
                    LayoutAttr::BorderRight(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeRight, r),
                        // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeRight, r),
                        _ => (),
                    },
                    LayoutAttr::Border(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_border(YGEdge::YGEdgeAll, r),
                        // ValueUnit::Percent(r) => yoga.set_border_percent(YGEdge::YGEdgeAll, r),
                        _ => (),
                    },
                    LayoutAttr::MinWidth(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_min_width(r),
                        ValueUnit::Percent(r) => yoga.set_min_width_percent(r),
                        _ => (),
                    },
                    LayoutAttr::MinHeight(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_min_height(r),
                        ValueUnit::Percent(r) => yoga.set_min_height_percent(r),
                        _ => (),
                    },
                    LayoutAttr::MaxHeight(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_max_height(r),
                        ValueUnit::Percent(r) => yoga.set_max_height_percent(r),
                        _ => (),
                    },
                    LayoutAttr::MaxWidth(r) => match r {
                        ValueUnit::Pixel(r) => yoga.set_max_width(r),
                        ValueUnit::Percent(r) => yoga.set_max_width_percent(r),
                        _ => (),
                    },
                    LayoutAttr::FlexBasis(r) => match r {
                        ValueUnit::Auto => yoga.set_flex_basis_auto(),
                        ValueUnit::Undefined => yoga.set_flex_basis_auto(),
                        ValueUnit::Pixel(r) => yoga.set_flex_basis(r),
                        ValueUnit::Percent(r) => yoga.set_flex_basis_percent(r),
                    },
                    LayoutAttr::FlexShrink(r) => yoga.set_flex_shrink(r),
                    LayoutAttr::FlexGrow(r) => yoga.set_flex_grow(r),
                }
            }
            // 插入className
            world.class_name.lend_mut().insert(node_id, ClassName(*class_id) );
        },
        None => (),
    };
    debug_println!("set_class"); 
}

/// aabb的查询函数的参数
struct AbQueryArgs<'a> {
  enables: &'a MultiCaseImpl<Node, Enable>,
  by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
  z_depths: &'a MultiCaseImpl<Node, ZDepth>,
  overflow_clip: &'a SingleCaseImpl<OverflowClip>,
  aabb: Aabb3,
  ev_type: u32,
  max_z: f32,
  result: usize,
}
impl<'a> AbQueryArgs<'a> {
  pub fn new(
    enables: &'a MultiCaseImpl<Node, Enable>,
    by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
    z_depths: &'a MultiCaseImpl<Node, ZDepth>,
    overflow_clip: &'a SingleCaseImpl<OverflowClip>,
    aabb: Aabb3,
    ev_type: u32,
  ) -> AbQueryArgs<'a> {
    AbQueryArgs{
      enables,
      by_overflows,
      z_depths,
      overflow_clip,
      aabb: aabb,
      ev_type: ev_type,
      max_z: -Z_MAX,
      result: 0,
    }
  }
}
/// aabb的ab查询函数, aabb的oct查询函数应该使用intersects
fn ab_query_func(arg: &mut AbQueryArgs, _id: usize, aabb: &Aabb3, bind: &usize) {
  // debug_println!("ab_query_func----------------------------{}, {:?}, {:?}", *bind, aabb, arg.aabb);
  if intersects(&arg.aabb, aabb) {
    // debug_println!("bind----------------------------{}", *bind);
    let enable = unsafe { arg.enables.get_unchecked(*bind) }.0;
    // debug_println!("enable----------------------------{}", enable);
    // println!("enable----------id: {}, enable: {}", bind, enable);
    //如果enable true 表示不接收事件
    match enable {
      true => (),
      false => return,
    };

    let z_depth = unsafe { arg.z_depths.get_unchecked(*bind) }.0;
    // println!("z_depth----------id: {}, z_depth: {}, arg.max_z:{}", bind, z_depth, arg.max_z);
    // debug_println!("----------------------------z_depth: {}, arg.max_z: {}", z_depth, arg.max_z);
    // 取最大z的node
    if z_depth > arg.max_z {
      let by_overflow = unsafe { arg.by_overflows.get_unchecked(*bind) }.0;
    //   println!("by_overflow1---------------------------bind: {},  by: {}, clip: {:?}, id_vec: {:?}, x: {}, y: {}", bind, by_overflow, &arg.overflow_clip.clip, &arg.overflow_clip.id_vec, arg.aabb.min.x, arg.aabb.min.y);
      // 检查是否有裁剪，及是否在裁剪范围内
      if by_overflow == 0 || in_overflow(&arg.overflow_clip, by_overflow, arg.aabb.min.x, arg.aabb.min.y) {
        // println!("in_overflow------------------by: {}, bind: {}, ", by_overflow, bind);
        // println!("result----------id: {}", bind);
        arg.result = *bind;
        arg.max_z = z_depth;
      }
    }
    // // 判断类型是否有相交
    // if (node.event_type as u32) & arg.ev_type != 0 {
    //     // 取最大z的node
    //     if node.z_depth > arg.max_z {
    //       // 检查是否有裁剪，及是否在裁剪范围内
    //       if node.by_overflow == 0 || in_overflow(&arg.mgr, node.by_overflow, aabb.min.x, aabb.min.y) {
    //         arg.result = *bind;
    //         arg.max_z = node.z_depth;
    //        }
    //     }
    // }
  }
}

/// 检查坐标是否在裁剪范围内， 直接在裁剪面上检查
fn in_overflow(overflow_clip: &SingleCaseImpl<OverflowClip>, by_overflow: usize, x: f32, y: f32) -> bool{
  let xy = Point2::new(x, y);
  for i in 0..overflow_clip.id_vec.len() {
    // debug_println!("i + 1---------------------------{}",i + 1);
    // debug_println!("overflow_clip.id_vec[i]---------------------------{}",overflow_clip.id_vec[i]);
    if (by_overflow & (1<<i)) != 0 && overflow_clip.id_vec[i] > 0 {
      let p = &overflow_clip.clip[i];
      match include_quad2(&xy, &p[0], &p[1], &p[2], &p[3]) {
        InnOuter::Inner => (),
        _ => {
            // println!("overflow----------clip: {:?},x: {}, y: {}", p[0], x, y);
            return false
        }
      }
    }
  }
  return true
}
