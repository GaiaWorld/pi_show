/// 将设置节点属性的接口导出到js

use std::{
  usize::MAX as UMAX,
  f32::INFINITY as FMAX,
};

use stdweb::unstable::TryInto;

use ecs::{Lend, LendMut, MultiCaseImpl, SingleCaseImpl};
use ecs::idtree::{InsertType, IdTree};
use atom::Atom;
use octree::intersects;
use cg2d::{include_quad2, InnOuter};

use gui::component::user::*;
use gui::component::calc::*;
use gui::single::*;
use gui::entity::Node;
// use gui::
// use gui::system::set_layout_style;
use gui::Z_MAX;

use GuiWorld;

fn create(world: &GuiWorld) -> usize {
    let gui = &world.gui;
    let idtree = gui.idtree.lend_mut();
    let node = gui.node.lend_mut().create();
    let border_radius = gui.border_radius.lend_mut();
    border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});
    idtree.create(node);

    // set_layout_style(&world.default_attr, unsafe {gui.yoga.lend_mut().get_unchecked(node)}, &mut StyleMark::default());
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
    node as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_text_node(world: u32) -> u32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let node = create(world);
    world.gui.text_content.lend_mut().insert(node, TextContent("".to_string(), Atom::from("")));
    node as u32
}

//创建图片节点
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_image_node(world: u32) -> u32{
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    let node = create(world);
    node as u32
}

// 在尾部插入子节点，如果该节点已经存在父节点， 会移除原有父节点对本节点的引用
#[allow(unused_attributes)]
#[no_mangle]
pub fn append_child(world: u32, child: u32, parent: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();

	// 如果child在树上， 则会从树上移除节点， 但不会发出事件
	idtree.remove(child as usize, None);
    idtree.insert_child(child as usize, parent as usize, UMAX, Some(&notify));
}

// 在某个节点之前插入节点，如果该节点已经存在父节点， 会移除原有父节点对本节点的引用
#[allow(unused_attributes)]
#[no_mangle]
pub fn insert_before(world: u32, child: u32, brother: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
	// 如果child在树上， 则会从树上移除节点， 但不会发出事件
	idtree.remove(child as usize, None);
    idtree.insert_brother(child as usize, brother as usize, InsertType::Front, Some(&notify));
}

// 移除节点， 但不会销毁， 如果节点树中存在图片资源， 将会释放其对纹理的引用， 如果节点树中overflow=hidden， 将会删除对应的裁剪平面，
#[allow(unused_attributes)]
#[no_mangle]
pub fn remove_node(world: u32, node_id: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
    idtree.remove(node_id as usize, Some(&notify)); 
}

// 销毁节点
#[allow(unused_attributes)]
#[no_mangle]
pub fn destroy_node(world: u32, node_id: u32){
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    // let notify = idtree.get_notify();
    let node = world.node.lend_mut();
    idtree.destroy(node_id as usize, false, None);
    node.delete(node_id as usize); 
}

// __jsObj: image_name(String)
// 设置图片的src
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_src(world: u32, node: u32){
	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let name: String = js!{return __jsObj}.try_into().unwrap();

	let images = world.gui.image.lend_mut();
	images.insert(node as usize, Image{src: None, url: Atom::from(name)});
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

    let transform;
    let transform1;
    match transforms.get(node_id) {
        Some(r) => transform = r,
        None => {
            transform1 = Transform::default();
            transform = &transform1;
        },
    };
    
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
	let idtree = world.idtree.lend();

    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(enables, by_overflows, z_depths, overflow_clip, idtree, aabb.clone(), 0);
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
	let idtree = world.idtree.lend();

    let aabb = Aabb3::new(Point3::new(x,y,-Z_MAX), Point3::new(x,y,Z_MAX));
    let mut args = AbQueryArgs::new(enables, by_overflows, z_depths, overflow_clip, idtree, aabb.clone(), 0);

    for e in entitys.iter() {
		let oct = match octree.get(e) {
			Some(r) => r,
			None => {
				println!("query fail, id: {}", e);
				return 0;
			},
		};
        // let oct = unsafe { octree.get_unchecked(e) };
        ab_query_func(&mut args, e, oct.0, &e);
    }
    args.result as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn add_class(world: u32, node_id: u32, key: u32, index: u32){
    let node_id = node_id as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};

    if index == 0 {
        unsafe { world.gui.class_name.lend_mut().get_unchecked_mut(node_id as usize) } .one = key as usize;
    } else if index == 1 {
        unsafe { world.gui.class_name.lend_mut().get_unchecked_mut(node_id as usize) } .two = key as usize;
    }if index > 2 {
        unsafe { world.gui.class_name.lend_mut().get_unchecked_mut(node_id as usize) } .other.push(key as usize);
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn add_class_end(world: u32, node_id: u32, old: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    world.gui.class_name.lend_mut().get_notify_ref().modify_event(node_id as usize, "", old as usize);
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn add_class_start(world: u32, node_id: u32) -> u32 {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
    Box::into_raw(Box::new(world.gui.class_name.lend_mut().insert_no_notify(node_id as usize, ClassName::default()).unwrap())) as u32
}

pub fn define_set_class(){
    js! {
        Module._set_class = function(world, node, class_arr){
            var old = Module._add_class_start(world, node);
            for (var i = 0; i < class_arr.length; i++) {
                Module._add_class(world, node, class_arr[i], i);
            }
            Module._add_class_end(world, node, old);
        }
    }
}

/// aabb的查询函数的参数
struct AbQueryArgs<'a> {
  enables: &'a MultiCaseImpl<Node, Enable>,
  by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
  z_depths: &'a MultiCaseImpl<Node, ZDepth>,
  overflow_clip: &'a SingleCaseImpl<OverflowClip>,
  id_tree: &'a SingleCaseImpl<IdTree>,
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
	id_tree: &'a SingleCaseImpl<IdTree>,
    aabb: Aabb3,
    ev_type: u32,
  ) -> AbQueryArgs<'a> {
    AbQueryArgs{
      enables,
      by_overflows,
      z_depths,
      overflow_clip,
	  id_tree,
      aabb: aabb,
      ev_type: ev_type,
      max_z: -Z_MAX,
      result: 0,
    }
  }
}
/// aabb的ab查询函数, aabb的oct查询函数应该使用intersects
fn ab_query_func(arg: &mut AbQueryArgs, _id: usize, aabb: &Aabb3, bind: &usize) {
	match arg.id_tree.get(*bind) {
		Some(node) => if node.layer == 0 {return},
		None => return,
	};
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
  for (i, c) in overflow_clip.clip.iter() {
    // debug_println!("i + 1---------------------------{}",i + 1);
    // debug_println!("overflow_clip.id_vec[i]---------------------------{}",overflow_clip.id_vec[i]);
    if (by_overflow & (1<<(i -1))) != 0  {
      let p = &c.view;
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

// /// 检查坐标是否在裁剪范围内， 直接在裁剪面上检查
// fn in_overflow(overflow_clip: &SingleCaseImpl<OverflowClip>, by_overflow: usize, x: f32, y: f32) -> bool{
//   let xy = Point2::new(x, y);
//   for i in 0..overflow_clip.id_vec.len() {
//     // debug_println!("i + 1---------------------------{}",i + 1);
//     // debug_println!("overflow_clip.id_vec[i]---------------------------{}",overflow_clip.id_vec[i]);
//     if (by_overflow & (1<<i)) != 0 && overflow_clip.id_vec[i] > 0 {
//       let p = &overflow_clip.clip[i];
//       match include_quad2(&xy, &p[0], &p[1], &p[2], &p[3]) {
//         InnOuter::Inner => (),
//         _ => {
//             // println!("overflow----------clip: {:?},x: {}, y: {}", p[0], x, y);
//             return false
//         }
//       }
//     }
//   }
//   return true
// }
