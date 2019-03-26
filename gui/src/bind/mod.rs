#![feature(custom_attribute)]

// #[macro_use]
// extern crate stdweb_derive;

use std::rc::Rc;
use std::cell::RefCell;

use webgl_rendering_context::{WebGLRenderingContext};

use wcs::world::World;
use wcs::component::{Builder};

use world::GuiComponentMgr;
use layout::{YgNode};
use component::node::{NodeBuilder};
use component::style::element::{ElementBuilder};
use component::style::generic::{Display};
use component::style::element::{Text as TextElement, Image as ImageElement};

pub mod data;
pub mod layout;

pub mod text;
pub mod style;
pub mod transform;
pub mod node;

pub struct Pointer{
    id: usize,
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

/**创建一个gui的实例 */
#[no_mangle]
pub fn create_gui(gl: WebGLRenderingContext) -> u32{
    js!{console.log("create_gui");}
    let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::new(gl));

    // 创建一个宽，高与canvase相等的根节点
    let mut root = NodeBuilder::new()
    .display(Some(Display::Flex))
    .build(&mut world.component_mgr.node);
    let root_yoga = YgNode::new();
    root.yoga = root_yoga;
    // 默认是指根的宽高为10.0
    root.yoga.set_width(10.0);
    root.yoga.set_height(10.0);

    let root_id = world.component_mgr.add_node(root).id;
    let r = Pointer {
        id: root_id, 
        world: Rc::new(RefCell::new(world)),
    };

    to_raw(r)
}

/** 设置gui的宽高 */
#[no_mangle]
pub fn set_gui_size(own: u32, width: f32, height: f32) {
    let gui = unsafe {&*(own as *const Pointer)};
    let world = gui.world.borrow_mut();

    let root = world.component_mgr.node._group.get(gui.id);
    root.yoga.set_width(width);
    root.yoga.set_height(height);
    js!{console.log("set_gui_size");}
}

//创建文本节点
#[no_mangle]
pub fn create_text_node(own: u32) -> u32{
    js!{console.log("create_text_node");}
    let gui = unsafe {&*(own as *const Pointer)};
    let mut world = gui.world.borrow_mut();
    let node = NodeBuilder::new()
    .element(
        ElementBuilder::new()
        .text(TextElement::default())
        .build(&mut world.component_mgr.node.element))
    .build(&mut world.component_mgr.node);
    let node_id = world.component_mgr.node._group.insert(node, 0);
    let node_pointer = Pointer {
        id: node_id,
        world: gui.world.clone(),
    };
    to_raw(node_pointer)
}

//创建图片节点
#[no_mangle]
pub fn create_image_node(own: u32) -> u32{
    js!{console.log("create_image_node");}
    let gui = unsafe {&*(own as *const Pointer)};
    let mut world = gui.world.borrow_mut();
    let node = NodeBuilder::new()
    .element(
        ElementBuilder::new()
        .image(ImageElement::default())
        .build(&mut world.component_mgr.node.element))
    .build(&mut world.component_mgr.node);
    let node_id = world.component_mgr.node._group.insert(node, 0);
    let node_pointer = Pointer {
        id: node_id,
        world: gui.world.clone(),
    };
    to_raw(node_pointer)
}

// #[no_mangle]
// pub fn create_background_node(own: u32) -> u32{
//     js!{console.log("create_background_node");}
//     let gui = unsafe {&*(own as *const Pointer)};
//     let mut world = gui.world.borrow_mut();
//     let node = NodeBuilder::new()
//     .element(
//         ElementBuilder::new()
//         .rect(RectElement::default())
//         .build(&mut world.component_mgr.node.element))
//     .build(&mut world.component_mgr.node);
//     let node_id = world.component_mgr.node._group.insert(node, 0);
//     let node_pointer = Pointer {
//         id: node_id,
//         world: gui.world.clone(),
//     };
//     to_raw(node_pointer)
// }

//创建容器节点， 容器节点可设置背景颜色
#[no_mangle]
pub fn create_container_node(own: u32) -> u32{
    js!{console.log("create_container_node");}
    let gui = unsafe {&*(own as *const Pointer)};
    let mut world = gui.world.borrow_mut();
    let node = NodeBuilder::new().build(&mut world.component_mgr.node);
    let node_id = world.component_mgr.node._group.insert(node, 0);
    let node_pointer = Pointer {
        id: node_id,
        world: gui.world.clone(),
    };
    to_raw(node_pointer)
}

// 运行gui
#[no_mangle]
pub fn run(own: u32){
    js!{console.log("gui run");}
    let gui = unsafe {&*(own as *const Pointer)};
    let mut world = gui.world.borrow_mut();
    world.run(());
}

#[inline]
pub fn to_raw<T>(t: T) -> u32{
    Box::into_raw(Box::new(t)) as u32
}

#[inline]
pub fn from_raw_mut<T>(ptr: u32) -> &'static T {
    unsafe {&*(ptr as *const T)}
}


//相当于document
// #[no_mangle]
// fn root(_own: u32) -> u32 {
//     1
// }

// fn main(){
// }

// #[wasm_bindgen]
// impl Gui{
//     #[js_export] pub fn new(&self, width: f32, height: f32) -> Gui{
//         let mut world: World<GuiComponentMgr, ()> = World::new();
//         let root = {
//             let mgr = &mut world.component_mgr;
//             let node = NodeBuilder::new().style(StyleBuilder::new().layout(LayoutBuilder::new().wh(Rect::new(Some(StyleUnit::Length(width)), Some(StyleUnit::Length(height)))).build(&mut mgr.node.style.layout)).build(&mut mgr.node.style)).build(&mut mgr.node);
//             let r = mgr.add_node(node);
//             r.id
//         };
//         Gui{
//             root: root,
//             world: Rc::new(RefCell::new(world))
//         }
//     }

//     #[js_export] pub fn run(&self){
//         self.world.borrow_mut().run(());
//     }

//     #[js_export] pub fn root(&self) -> node::Node{
//         node::Node::new(self.root, self.world.clone())
//     }
// }