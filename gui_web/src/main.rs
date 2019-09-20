#![feature(nll)] 
#![feature(proc_macro_hygiene)]
#![feature(rustc_const_unstable)] 
#![feature(core_intrinsics)]
#![feature(custom_attribute)] 
#![feature(type_ascription)]
#![feature(link_args)]
#![feature(vec_remove_item)]
#![allow(unused_attributes)]
#![allow(dead_code)]
#![feature(rustc_private)]
#![recursion_limit="512"]

#[macro_use]
extern crate serde;
extern crate stdweb_derive;
extern crate webgl_rendering_context;
#[macro_use]
extern crate stdweb;
extern crate lazy_static;
extern crate paste;
extern crate gui;
extern crate ecs;
#[macro_use]
extern crate ecs_derive;
extern crate map;
#[macro_use]
extern crate debug_info;
extern crate hal_core;
extern crate hal_webgl;
extern crate atom;
extern crate octree;
extern crate cg2d;
extern crate share;
extern crate hash;
extern crate ordered_float;
extern crate data_view;
extern crate bincode;
extern crate res;

use std::mem::transmute;
use std::cell::RefCell;

use stdweb::unstable::TryInto;
use stdweb::Object;
use webgl_rendering_context::{WebGLRenderingContext};
use ordered_float::OrderedFloat;
use res::ResMgr;

use share::Share;
use atom::Atom;
use hal_webgl::*;
use hal_core::*;
use ecs::{ LendMut, Lend};
use gui::layout::{ YGAlign, FlexNode };
use gui::world::{ create_world, create_res_mgr, RENDER_DISPATCH, LAYOUT_DISPATCH };
use gui::component::user::*;
use gui::component::calc::Visibility;
use gui::single::Class;
use gui::single::RenderBegin;
use gui::render::engine::{ShareEngine, Engine, UnsafeMut};
use gui::render::res::Opacity as ROpacity;
use gui::world::GuiWorld as GuiWorld1;
use gui::render::res::TextureRes;

pub mod class;
pub mod style;
pub mod node;
pub mod text;
pub mod layout;
pub mod transform;
#[cfg(not(feature = "no_debug"))]
pub mod debug;
pub mod yoga;
pub mod bc;
pub mod world;
#[cfg(not(feature = "no_define_js"))]
pub mod rs_call_js;

use bc::{YgNode};
use text::{ DrawTextSys};
#[cfg(not(feature = "no_define_js"))]
use rs_call_js::define_js;
use node::define_set_class;

pub struct GuiWorld {
	pub gui: GuiWorld1<YgNode, WebglHalContext>,
	pub draw_text_sys: DrawTextSys,
	pub default_text_style: TextStyle,
	pub default_attr: Class,
}

// 设置纹理的缓存配置
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_texture_catch_cfg(
	res_mgr: u32/* res_mgr指针 */,
	min_c1: u32,
	max_c1: u32,
	timeout1: u32,
	min_c2: u32,
	max_c2: u32,
	timeout2: u32,
	min_c3: u32,
	max_c3: u32,
	timeout3: u32 ) {
	let res_mgr = unsafe {&mut *(res_mgr as *mut  Share<RefCell<ResMgr>>)};
	res_mgr.borrow_mut().register::<TextureRes>([
		min_c1 as usize, max_c1 as usize, timeout1 as usize,
		min_c2 as usize, max_c2 as usize, timeout2 as usize,
		min_c3 as usize, max_c3 as usize, timeout3 as usize,
	]);
}

// total_capacity: 资源管理器总容量, 如果为0， 将使用默认的容量设置
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_engine(total_capacity: u32/* 资源管理器总容量 */) -> u32 {
	let gl: WebGLRenderingContext = js!(return __gl;).try_into().unwrap();
	let use_vao = TryInto::<bool>::try_into(js!(var u = navigator.userAgent.toLowerCase(); return u.indexOf("ipad") < 0 && u.indexOf("iphone") < 0;)).unwrap();

	// let gl = WebglHalContext::new(gl, fbo, false);
	let gl = WebglHalContext::new(gl, use_vao);
	let engine = Engine::new(gl, create_res_mgr(total_capacity as usize));
	let r = Box::into_raw(Box::new(UnsafeMut::new(Share::new(engine)))) as u32;
	r
}

// 创建渲染目标， 返回渲染目标的指针， 必须要高层调用destroy_render_target接口， 该渲染目标才能得到释放
#[allow(unused_attributes)]
#[no_mangle]
pub fn create_render_target(world: u32) -> u32 {
	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
	let fbo = TryInto::<Object>::try_into(js!(return {wrap: __fbo};)).unwrap();
	let engine = world.engine.lend_mut();
	let rt = Share::new(engine.gl.rt_create_webgl(fbo)); // 创建渲染目标
	Box::into_raw(Box::new(rt)) as u32
}

// 销毁渲染目标
#[allow(unused_attributes)]
#[no_mangle]
pub fn destroy_render_target(render_target: u32) {
	unsafe{Box::from_raw(&mut *(render_target as usize as *mut Share<HalRenderTarget>))};
}

// 绑定rendertarget
// render_target为0时， 表示绑定gl默认的渲染目标， 当大于0时， render_target必须是一个RenderTarget的指针
#[allow(unused_attributes)]
#[no_mangle]
pub fn bind_render_target(world: u32, render_target: u32) {
	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
	let engine = world.engine.lend_mut();
	if render_target == 0 {
		engine.render_target = None;
	}else {
		engine.render_target = Some(unsafe{&*(render_target as usize as *const Share<HalRenderTarget>)}.clone());
	}
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn clone_engine(engine: u32) -> u32 {
	Box::into_raw(Box::new(unsafe {&*(engine as usize as *mut ShareEngine<WebglHalContext>)}.clone())) as u32
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn create_gui(engine: u32, width: f32, height: f32) -> u32 {
	let mut engine = *unsafe { Box::from_raw(engine as usize as *mut ShareEngine<WebglHalContext>)}; 
	let draw_text_sys = DrawTextSys::new();
	let c = draw_text_sys.canvas.clone();
	let f = Box::new(move |name: &Atom, font_size: usize, ch: char| -> f32 {
		let ch = ch as u32;
		let font_size = font_size as u32;
		TryInto::<f64>::try_into( js!{
			let c = @{&c};
			c.ctx.font = @{font_size} + "px " + @{name.as_ref()};
			return c.ctx.measureText(String.fromCharCode(@{ch})).width;
		}).unwrap() as f32
	});
	let texture = engine.gl.texture_create_2d(0, 2048, 32, PixelFormat::RGBA, DataFormat::UnsignedByte, false, None).unwrap();
	let res = engine.create_texture_res(
		Atom::from("__$text".to_string()),
		TextureRes::new(2048, 32, PixelFormat::RGBA, DataFormat::UnsignedByte, unsafe{transmute(1 as u8)}, unsafe{transmute(0 as u8)}, texture), 0);

	let world = create_world::<YgNode, WebglHalContext>(engine, width, height, f, res);
	let world =  GuiWorld1::<YgNode, WebglHalContext>::new(world);
	let idtree = world.idtree.lend_mut();
	let node = world.node.lend_mut().create();
	let border_radius = world.border_radius.lend_mut();
	border_radius.insert(node, BorderRadius{x: LengthUnit::Pixel(0.0), y: LengthUnit::Pixel(0.0)});

	let visibilitys = world.visibility.lend_mut();
	visibilitys.insert(node, Visibility(true));

	let ygnode = world.yoga.lend_mut();
	let ygnode = unsafe { ygnode.get_unchecked_mut(node) };

	// let config = YgConfig::new();
	// config.set_point_scale_factor(0.0);
	// let ygnode1 = YgNode::new_with_config(config);
	// let ygnode1 = YgNode::default();
	ygnode.set_width(width);
	ygnode.set_height(height);
	ygnode.set_align_items(YGAlign::YGAlignFlexStart);
	// *ygnode = ygnode1;

	idtree.create(node);
	idtree.insert_child(node, 0, 0, None);
	let world = GuiWorld{
		gui: world,
		draw_text_sys: draw_text_sys,
		default_text_style: TextStyle::default(),
		default_attr: Class::default(),
	};
	Box::into_raw(Box::new(world)) as u32
}

// 设置gui渲染的清屏颜色
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_clear_color(world: u32, r: f32, g: f32, b: f32, a: f32){
	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
	let render_begin = world.world.fetch_single::<RenderBegin>().unwrap();
	let render_begin = render_begin.lend_mut();
	Share::make_mut(&mut render_begin.0).clear_color = Some((OrderedFloat(r), OrderedFloat(g), OrderedFloat(b), OrderedFloat(a))); 
}

// 渲染gui， 通常每帧调用
#[allow(unused_attributes)]
#[no_mangle]
pub fn render(world_id: u32){
	let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
	world.draw_text_sys.run(world_id);
	let world = &mut world.gui;
	load_image(world_id);
	world.world.run(&RENDER_DISPATCH);;
}

// 强制计算一次布局
#[allow(unused_attributes)]
#[no_mangle]
pub fn cal_layout(world_id: u32){
	let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
	let world = &mut world.gui;
	world.world.run(&LAYOUT_DISPATCH);
}

//设置shader
#[allow(unused_attributes)]
#[no_mangle]
pub fn set_shader(engine: u32){
	let shader_name: String = js!(return __jsObj;).try_into().unwrap();
	let shader_code: String = js!(return __jsObj1;).try_into().unwrap();
	let engine = unsafe { &mut *(engine as usize as *mut ShareEngine<WebglHalContext>)};
	engine.gl.render_set_shader_code(&shader_name, &shader_code);
}

// 加载图片成功后调用
// image_name可以使用hash值与高层交换 TODO
// __jsObj: image, __jsObj1: image_name(String)
#[no_mangle]
pub fn load_image_success(world_id: u32, opacity: u8, compress: u8, mut r_type: u8/* 缓存类型，支持0， 1， 2三种类型 */){
	if r_type > 2 {
		r_type = 0;
	}
	let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};
	let world = &mut world.gui;

	let name: String = js!{return __jsObj1}.try_into().unwrap();
	let name = Atom::from(name);
	let engine = world.engine.lend_mut();
	let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
	let height: u32 = js!{return __jsObj.height}.try_into().unwrap();
	let opacity = unsafe{transmute(opacity)};

	let pformate = match opacity {
		ROpacity::Opaque => PixelFormat::RGB,
		ROpacity::Translucent | ROpacity::Transparent => PixelFormat::RGBA,
	};

	let texture = match TryInto::<Object>::try_into(js!{return {wrap: __jsObj};}) {
		Ok(image_obj) => engine.gl.texture_create_2d_webgl(width, height, 0, pformate, DataFormat::UnsignedByte, false, &image_obj).unwrap(),
		Err(s) => panic!("set_src error, {:?}", s),
	};
	let res = engine.create_texture_res(name.clone(), TextureRes::new(width as usize, height as usize, pformate, DataFormat::UnsignedByte, opacity, unsafe{transmute(compress)}, texture), r_type as usize);
	
	let image_wait_sheet = world.image_wait_sheet.lend_mut();
	match image_wait_sheet.wait.remove(&name) {
		Some(r) => {
			image_wait_sheet.finish.push((name, res, r));
		},
		None => (),
	};
	image_wait_sheet.get_notify().modify_event(0, "", 0);
}

// 加载图片，调用高层接口，加载所有等待中的图片
fn load_image(world_id: u32) {
	let world = unsafe {&mut *(world_id as usize as *mut GuiWorld)};

	let image_wait_sheet = &mut world.gui.image_wait_sheet.lend_mut();
	for img_name in image_wait_sheet.loads.iter() {
		js!{
			if (window.__load_image) {
				window.__load_image(@{world_id}, @{img_name.as_ref()});
			} else {
				console.log("__load_image is undefined");
			}
		}
	}
	image_wait_sheet.loads.clear();
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


fn main(){
	// 定义图片加载函数， canvas文字纹理绘制函数（使用feature: “no_define_js”, 将不会有这两个接口， 外部可根据需求自己实现 ）
	#[cfg(not(feature = "no_define_js"))]
	define_js();
	define_set_class();
}
