use std::mem::{transmute, uninitialized};
use std::sync::Arc;
use std::rc::Rc;

use wcs::world::World;
use wcs::component::{Builder};
use atom::Atom;
use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;
use webgl_rendering_context::{WebGLRenderingContext};

use world_doc::component::node::{NodeBuilder};
use world_doc::component::style::element::{ElementBuilder};
use world_doc::component::style::element::{Text as TextElement, ElementId};
use world_doc::{WorldDocMgr, create_world, LAYOUT_SYS, ALL};
use render::engine::Engine;
use render::res::{ TextureRes, Opacity };
use font::sdf_font::{SdfFont, StaticSdfFont};

pub mod data;
pub mod layout;

pub mod text;
pub mod style;
pub mod transform;
pub mod node;

#[no_mangle]
pub fn create_engine() -> u32{
    debug_println!("create_engine");
    let gl = js!(return __gl;).try_into().unwrap();
    let engine = Engine::new(gl);
    Box::into_raw(Box::new(engine)) as u32
}

// #[no_mangle]
// pub fn get_texture_res(engine: u32, key: String) -> u32{
//     let engine = unsafe {&mut *(engine as usize as *mut Engine)};
//     let key = Atom::from(key);
//     match engine.res_mgr.textures.get(&key) {
//         Some(res) => Box::into_raw(Box::new(res)) as u32,
//         None => 0,
//     }
// }

// #[no_mangle]
// pub fn create_texture_res(engine: u32, key: String, width: u32, height: u32, opacity: u8, compress: u32) -> u32{
//     let engine = unsafe {&mut *(engine as usize as *mut Engine)};
//     let bind = js!(return __jsObj;).try_into().unwrap();
//     let key = Atom::from(key);
//     let r = Box::into_raw(Box::new( engine.res_mgr.textures.create(TextureRes::new(key, width as usize, height as usize, unsafe{transmute(opacity)}, compress as usize, bind, engine.gl.clone()) ))) as u32;
//     // js!{
//     //     console.log("create_texture_res src:", @{r});
//     // };
//     r
// }


// __jsObj1: image_buffer, __jsObj: uv cfg, __jsObj2: name(String)
#[no_mangle]
pub fn create_sdf_font(world: u32) -> u32 {
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let name: String = js!(return __jsObj2;).try_into().unwrap();
    let name = Atom::from(name);

    debug_println!("name1:{:?}", name);
    let mut sdf_font = StaticSdfFont::new(unsafe { uninitialized() } );
    let bind: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let bind = bind.to_vec();
    match sdf_font.parse(bind.as_slice()) {
        Ok(_) => (),
        Err(s) => panic!("{}", s),
    };

    let texture = {
        let gl = world.component_mgr.world_2d.component_mgr.engine.gl.clone();
        let texture = match gl.create_texture() {
            Some(v) => v,
            None => panic!("create_texture is None"),
        };
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));
        let width = sdf_font.atlas_width();
        let height = sdf_font.atlas_height();
        js!{
            @{&gl}.texImage2D(@{&gl}.TEXTURE_2D, 0, @{&gl}.ALPHA, @{width as u32}, @{height as u32}, 0, @{&gl}.ALPHA, @{&gl}.UNSIGNED_BYTE, __jsObj1);
        };
        
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_WRAP_S, WebGLRenderingContext::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D,WebGLRenderingContext::TEXTURE_WRAP_T, WebGLRenderingContext::CLAMP_TO_EDGE as i32);
        world.component_mgr.world_2d.component_mgr.engine.res_mgr.textures.create(TextureRes::new(name, width as usize, height as usize, unsafe{transmute(Opacity::Translucent)}, unsafe{transmute(0 as u8)} , texture, gl.clone()) )
        
    };
    unsafe { (sdf_font.texture() as *const Rc<TextureRes> as usize as *mut Rc<TextureRes>).write(texture)};
    debug_println!("sdf: --------------------------{:?}", sdf_font);
    let sdf_font: Arc<SdfFont> = Arc::new(sdf_font);
    Box::into_raw(Box::new(sdf_font)) as u32
}

#[no_mangle]
pub fn add_sdf_font_res(world: u32, value: u32){
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let res = *unsafe { Box::from_raw(value as usize as *mut Arc<SdfFont>) };
    world.component_mgr.font.set_src(res.name(), res);
}

// __jsObj: family_name(String), __jsObj1: src_name(String, 逗号分隔), 
#[no_mangle]
pub fn add_font_face(world: u32, oblique: f32, size: f32, weight: f32){
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let family: String = js!(return __jsObj;).try_into().unwrap();
    let src: String = js!(return __jsObj1;).try_into().unwrap();
    world.component_mgr.font.set_face(Atom::from(family), oblique, size, weight, src);
}

/**创建一个gui的实例 */
#[no_mangle]
pub fn create_gui(engine: u32, width: f32, height: f32) -> u32{
    debug_println!("create_gui");
    let engine: Engine = *unsafe { Box::from_raw(engine as usize as *mut Engine)}; // 安全隐患， 会消耗Engine的所有权， 一旦gui销毁，Engine也会销毁， 因此Engine无法共享， engine应该改为Rc
    let world = create_world(engine, width, height);
    Box::into_raw(Box::new(world)) as u32
}

/** 设置gui的宽高 */
#[no_mangle]
pub fn set_gui_size(world: u32, width: f32, height: f32) {
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    world.component_mgr.set_size(width, height);
    debug_println!("set_gui_size");
}

//创建文本节点
#[no_mangle]
pub fn create_text_node(world: u32) -> u32 {
    debug_println!("create_text_node");
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
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let mut node = NodeBuilder::new().build(&mut world.component_mgr.node);
    node.element = ElementId::Image(0);
    let node_id = world.component_mgr.node._group.insert(node, 0) as u32;
    debug_println!("create_image_node, node_id:{}", node_id);
    node_id
}

// #[no_mangle]
// pub fn create_background_node(world: u32, node_id: u32) -> u32{
//     #[cfg(feature = "log")] 
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
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    let node = NodeBuilder::new().build(&mut world.component_mgr.node);
    let node_id = world.component_mgr.node._group.insert(node, 0) as u32;
    debug_println!("create_node, node_id:{}", node_id);
    node_id
}

// 运行gui
#[no_mangle]
pub fn run(world: u32){
    debug_println!("gui run");
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    world.run(&ALL, ());
}

// 计算布局
#[no_mangle]
pub fn cal_layout(world: u32){
    debug_println!("cal_layout");
    let world = unsafe {&mut *(world as usize as *mut World<WorldDocMgr, ()>)};
    world.run(&LAYOUT_SYS, ());
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