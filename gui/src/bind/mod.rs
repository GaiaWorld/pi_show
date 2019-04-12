use stdweb::unstable::TryInto;

use wcs::world::World;
use wcs::component::{Builder};
use atom::Atom;

use world_doc::component::node::{NodeBuilder};
use world_doc::component::style::element::{ElementBuilder};
use world_doc::component::style::element::{Text as TextElement, Image as ImageElement};
use world_doc::{WorldDocMgr, create_world};
use render::engine::Engine;
use render::res::TextureRes;

pub mod data;
pub mod layout;

pub mod text;
pub mod style;
pub mod transform;
pub mod node;

#[no_mangle]
pub fn create_engine() -> u32{
    js!{console.log("create_engine");}
    let gl = js!(return __gl;).try_into().unwrap();
    let engine = Engine::new(gl);
    Box::into_raw(Box::new(engine)) as u32
}

#[no_mangle]
pub fn get_texture_res(engine: u32, key: String) -> u32{
    let engine = unsafe {&mut *(engine as usize as *mut Engine)};
    let key = Atom::from(key);
    match engine.res_mgr.textures.get(&key) {
        Some(res) => Box::into_raw(Box::new(res)) as u32,
        None => 0,
    }
}

#[no_mangle]
pub fn create_texture_res(engine: u32, key: String, width: u32, height: u32, opacity: f32, compress: u32, bind: u32) -> u32{
    let engine = unsafe {&mut *(engine as usize as *mut Engine)};
    let bind = js!(return __rsObjMap.get(@{bind});).try_into().unwrap();
    let key = Atom::from(key);
    Box::into_raw(Box::new( engine.res_mgr.textures.create(TextureRes::new(key, width as usize, height as usize, opacity, compress as usize, bind, engine.gl.clone()) ))) as u32
}

/**创建一个gui的实例 */
#[no_mangle]
pub fn create_gui(engine: u32) -> u32{
    js!{console.log("create_gui");}
    let engine: Engine = *unsafe { Box::from_raw(engine as usize as *mut Engine)}; // 安全隐患， 会消耗Engine的所有权， 一旦gui销毁，Engine也会销毁， 因此Engine无法共享， engine应该改为Rc
    let world = create_world(engine);
    Box::into_raw(Box::new(world)) as u32
}

/** 设置gui的宽高 */
#[no_mangle]
pub fn set_gui_size(world: u32, width: f32, height: f32) {
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    world.component_mgr.set_size(width, height);
    js!{console.log("set_gui_size");}
}

//创建文本节点
#[no_mangle]
pub fn create_text_node(world: u32) -> u32{
    js!{console.log("create_text_node");}
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};

    let node = NodeBuilder::new()
    .element(
        ElementBuilder::new()
        .text(TextElement::default())
        .build(&mut world.component_mgr.node.element))
    .build(&mut world.component_mgr.node);
    world.component_mgr.node._group.insert(node, 0) as u32
}

//创建图片节点
#[no_mangle]
pub fn create_image_node(world: u32) -> u32{
    js!{console.log("create_image_node");}
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};

    let node = NodeBuilder::new()
    .element(
        ElementBuilder::new()
        .image(ImageElement::default())
        .build(&mut world.component_mgr.node.element))
    .build(&mut world.component_mgr.node);
    world.component_mgr.node._group.insert(node, 0) as u32
}

// #[no_mangle]
// pub fn create_background_node(world: u32, node_id: u32) -> u32{
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
pub fn create_node(world: u32) -> u32{
    js!{console.log("create_container_node");}
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};

    let node = NodeBuilder::new().build(&mut world.component_mgr.node);
    world.component_mgr.node._group.insert(node, 0) as u32
}

// 运行gui
#[no_mangle]
pub fn run(world: u32){
    js!{console.log("gui run");}
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    world.run(());
}


//相当于world_doc
// #[no_mangle]
// fn root(_world: u32, node_id: u32) -> u32 {
//     1
// }

// fn main(){
// }

// #[wasm_bindgen]
// impl Gui{
//     #[js_export] pub fn new(&self, width: f32, height: f32) -> Gui{
//         let mut world: World<WorldDocMgr, ()> = World::new();
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