
// use std::cell::RefCell;
use std::mem::{transmute, MaybeUninit,};
use std::ptr::write;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use web_sys::{WebGlFramebuffer, WebGlRenderingContext as RawWebGlRenderingContext, console};
use js_sys::{Date, Object, Function};
use wasm_bindgen::JsCast;

use ordered_float::OrderedFloat;
use flex_layout::{Size, Dimension, PositionType, Rect};

use atom::Atom;
use ecs::{Lend, LendMut, StdCell};
use gui::component::calc::Visibility;
use gui::font::font_sheet::FontSheet;
use gui::component::user::*;
use gui::render::engine::{Engine, ShareEngine, UnsafeMut};
use gui::render::res::Opacity as ROpacity;
use gui::render::res::TextureRes as TextureResRaw;
use gui::single::Class;
use gui::single::{RenderBegin, ClassSheet};
use gui::world::GuiWorld as GuiWorld1;
use gui::Z_MAX;
use gui::world::{seting_res_mgr, create_world, LAYOUT_DISPATCH, RENDER_DISPATCH, CALC_DISPATCH};
use hal_webgl::*;
use hal_core::*;
use share::Share;
use res_mgr_web::{ResMgr};
use res::Res;

use crate::world::{DrawTextSys, GuiWorld, measureText, set_render_dirty, loadImage, useVao};


#[wasm_bindgen]
/// 资源包装
pub struct TextureRes(TextureResRaw);

#[wasm_bindgen]
impl TextureRes {
	pub fn new(res: usize) -> TextureRes {
		TextureRes(*unsafe{ Box::from_raw(res as *mut TextureResRaw) })
	}
}
#[wasm_bindgen]
pub struct NativeResRef{
	inner: Share<dyn Res<Key = usize>>
}

#[wasm_bindgen]
impl TextureRes {
	/// 创建一个资源， 如果资源已经存在，则会修改资源的配置
	pub fn register_to_resmgr(mgr: &mut ResMgr, ty: usize, min_capacity: usize, max_capacity: usize, time_out: usize) {
		mgr.get_inner_mut().borrow_mut().register::<TextureResRaw>(min_capacity, max_capacity, time_out, ty, "".to_string());
	}

	/// 创建一个资源， 如果资源已经存在，旧的资源将被覆盖
	/// 如果创建的资源类型未注册，将崩溃
	pub fn create_res(self, mgr: &mut ResMgr, ty: usize, key: usize, cost: usize) -> NativeResRef {
		NativeResRef{inner: mgr.get_inner_mut().borrow_mut().create::<TextureResRaw>(key, ty,self.0, cost, )}
	}

	/// 获取资源
	pub fn get_res(mgr: &ResMgr, ty: usize, key: usize) -> Option<NativeResRef> {
		// return None;
		match mgr.get_inner().borrow().get::<TextureResRaw>(&key, ty) {
			Some(r) => Some(NativeResRef{inner: r}),
			None => None
		}
	}
}

/// total_capacity: 资源管理器总容量, 如果为0， 将使用默认的容量设置
#[allow(unused_unsafe)]
#[wasm_bindgen]
pub fn create_engine(gl: WebGlRenderingContext, res_mgr: &ResMgr) -> u32 {
    let use_vao = unsafe { useVao() };
	// let use_vao = false;
    // let gl = WebglHalContext::new(gl, fbo, false);
	let gl = WebglHalContext::new(gl, use_vao);
	let res_mgr = res_mgr.get_inner().clone();
	seting_res_mgr(&mut res_mgr.borrow_mut());
	let engine = Engine::new(gl, res_mgr);
	let r = Box::into_raw(Box::new(UnsafeMut::new(Share::new(engine)))) as u32;
    r
}

#[wasm_bindgen]
pub fn get_font_sheet(world: u32) -> u32{
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let font_sheet = (*world.gui.font_sheet.lend()).clone();
	Box::into_raw(Box::new(font_sheet)) as u32
}

#[wasm_bindgen]
pub fn get_class_sheet(world: u32) -> u32{
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let class_sheet = (*world.gui.class_sheet.lend()).clone();
	Box::into_raw(Box::new(class_sheet)) as u32
}

/// 创建渲染目标， 返回渲染目标的指针， 必须要高层调用destroy_render_target接口， 该渲染目标才能得到释放
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_render_target(world: u32, fbo: WebGlFramebuffer) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let engine = world.engine.lend_mut();
    let rt = Share::new(engine.gl.rt_create_webgl(fbo)); // 创建渲染目标
    Box::into_raw(Box::new(rt)) as u32
}

/// 销毁渲染目标
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn destroy_render_target(render_target: u32) {
    unsafe { Box::from_raw(&mut *(render_target as usize as *mut Share<HalRenderTarget>)) };
}

/// 绑定rendertarget
/// render_target为0时， 表示绑定gl默认的渲染目标， 当大于0时， render_target必须是一个RenderTarget的指针
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn bind_render_target(world_id: u32, render_target: u32) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    // let engine = world.engine.lend_mut();
    let begin = world.world.fetch_single::<RenderBegin>().unwrap();
    let begin = begin.lend_mut();
    if render_target == 0 {
        begin.1 = None;
    } else {
        begin.1 =
			Some(unsafe { &*(render_target as usize as *const Share<HalRenderTarget>) }.clone());
		set_render_dirty(world_id);
    }
}

/// 克隆渲染引擎（某些情况下， 需要多个gui实例共享同一个渲染引擎）
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn clone_engine(engine: u32) -> u32 {
    let engine: ShareEngine<WebglHalContext> =
        ShareEngine::clone(unsafe { &*(engine as usize as *const ShareEngine<WebglHalContext>) });
    Box::into_raw(Box::new(engine)) as u32
}

/// 创建gui实例
#[allow(unused_attributes)]
#[allow(unused_unsafe)]
#[wasm_bindgen]
pub fn create_gui(engine: u32, width: f32, height: f32, load_image_fun: Option<Function>, class_sheet: u32, font_sheet: u32) -> u32 {
	// unsafe{ console::log_1(&JsValue::from("create_gui0================================="))};
	// println!("create_gui 1============================");
    let mut engine =
        *unsafe { Box::from_raw(engine as usize as *mut ShareEngine<WebglHalContext>) };
    let draw_text_sys = DrawTextSys::new();
	let ctx = draw_text_sys.ctx.clone();
    let f = Box::new(move |name: usize, font_size: usize, ch: char| -> f32 {
		return unsafe { measureText(&ctx, ch as u32, font_size as u32, name as u32) };
	});
	// unsafe{ console::log_1(&JsValue::from("create_gui01================================="))};
	let mut max_texture_size = match engine.gl.get_raw_gl().get_parameter(RawWebGlRenderingContext::MAX_TEXTURE_SIZE) {
		Ok(r) => r.as_f64().unwrap() as u32,
		Err(_r) => 1024,
	};
	// let mut max_texture_size = 1024;
	if max_texture_size > 4096 {
		max_texture_size = 4096;
	}
    let texture = engine
        .gl
        .texture_create_2d(
            0,
            max_texture_size,
            32,
            PixelFormat::RGBA,
            DataFormat::UnsignedByte,
            false,
            None,
        )
		.unwrap();
		// unsafe{ console::log_1(&JsValue::from("create_gui2================================="))};
		// unsafe{ console::log_1(&JsValue::from(Atom::from("__$text".to_string()).get_hash() as u32))};
	
    let res = engine.create_texture_res(
        Atom::from("__$text".to_string()).get_hash(),
        TextureResRaw::new(
            max_texture_size as usize,
            32,
            PixelFormat::RGBA,
            DataFormat::UnsignedByte,
            unsafe { transmute(1 as u8) },
            None, //unsafe { transmute(0 as usize) },
            texture,
			None
        ),
        0,
    );
	let cur_time: usize = (Date::now() as u64 /1000) as usize;
	let mut class_sheet_option = None;
	let mut font_sheet_option = None;
	if class_sheet > 0 {
		class_sheet_option =
        Some(*unsafe { Box::from_raw(class_sheet as usize as *mut Share<StdCell<ClassSheet>>) });
	}
	if font_sheet > 0 {
		font_sheet_option =
        Some(*unsafe { Box::from_raw(font_sheet as usize as *mut Share<StdCell<FontSheet>>) });
	}
	// unsafe{ console::log_1(&JsValue::from("create_gui3================================="))};
    let world = create_world::<WebglHalContext>(engine, width, height, f, res, cur_time, class_sheet_option, font_sheet_option);
    // unsafe{ console::log_1(&JsValue::from("create_gui4================================="))};
	let world = GuiWorld1::<WebglHalContext>::new(world);
    let idtree = world.idtree.lend_mut();
    let node = world.node.lend_mut().create();
    idtree.create(node);

    let border_radius = world.border_radius.lend_mut();
    border_radius.insert(
        node,
        BorderRadius {
            x: LengthUnit::Pixel(0.0),
            y: LengthUnit::Pixel(0.0),
        },
    );
	// unsafe{ console::log_1(&JsValue::from("create_gui5================================="))};

    let visibilitys = world.visibility.lend_mut();
    visibilitys.insert(node, Visibility(true));

	let rect_layout_styles = world.rect_layout_style.lend_mut();
	let other_layout_styles = world.other_layout_style.lend_mut();
	let rect_layout_style = &mut rect_layout_styles[node];
	let other_layout_style = &mut other_layout_styles[node];

    // let config = YgConfig::new();
    // config.set_point_scale_factor(0.0);
    // let ygnode1 = YgNode::new_with_config(config);
	// let ygnode1 = YgNode::default();
	rect_layout_style.size = Size{width: Dimension::Points(width), height: Dimension::Points(height)};
	other_layout_style.position_type = PositionType::Absolute;
	other_layout_style.position = Rect::default();
	rect_layout_styles.get_notify_ref().modify_event(node, "width", 0);
	other_layout_styles.get_notify_ref().modify_event(node, "position_type", 0);
	// ygnode.align_items = AlignItems::FlexStart;
    // ygnode.set_align_items(AlignItems::FlexStart);
    // *ygnode = ygnode1;
	let font_sheet_version = world.font_sheet.lend().borrow().tex_version;

	idtree.insert_child(node, 0, 0);
    let world = GuiWorld {
        gui: world,
        draw_text_sys: draw_text_sys,
		max_texture_size: 4096,
        default_attr: Class::default(),
		performance_inspector: 0,
		load_image_success: unsafe {MaybeUninit::uninit().assume_init()},
		load_image: unsafe {MaybeUninit::uninit().assume_init()},
		draw_text: Closure::wrap(Box::new(move |world_id: JsValue| {
			let world_id = world_id.as_f64().unwrap() as u32;
			let gui_world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
			gui_world.draw_text_sys.run(world_id);
		})),
		old_texture_tex_version: font_sheet_version,
	};
	// unsafe{ console::log_1(&JsValue::from("create_gui6================================="))};

	let world_id = Box::into_raw(Box::new(world)) as u32;
	let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
	
	unsafe{write(&mut world.load_image_success, Closure::wrap(Box::new(move |
		opacity: u8,
		compress: i32,
		r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
		name: u32,
		width: u32,
		height: u32,
		data: Object,
		cost: u32| {
			let (res, name) = create_texture(world_id, opacity, compress, r_type, name, width, height, data, cost);
			let world = &mut *(world_id as usize as *mut GuiWorld);
			let world = &mut world.gui;
		
			let image_wait_sheet = world.image_wait_sheet.lend_mut();
			match image_wait_sheet.wait.remove(&name) {
				Some(r) => {
					image_wait_sheet.finish.push((name, res, r));
					image_wait_sheet.get_notify().modify_event(0, "", 0);
				}
				None => (),
			};
		}
	)))};
	// unsafe{ console::log_1(&JsValue::from("create_gui7================================="))};
	match load_image_fun {
		Some(r) => unsafe { write(&mut world.load_image, Box::new(move |image_name, callback: &Function| {
			r.call2(&JsValue::from(None::<u8>), &JsValue::from(image_name), callback).expect("call load_image fail!!!");
		})) },
		None => unsafe {write(&mut world.load_image, Box::new(|image_name, callback: &Function| {
			loadImage(image_name, callback);
		}) )},
	}
	world_id
}

// fn push_runtime(runtime_ref: &mut Vec<ecs::RunTime>, name: Atom) -> usize {
// 	let runtime_index = runtime_ref.len();
// 	runtime_ref.push(ecs::RunTime{sys_name: name, cost_time: std::time::Duration::from_millis(0)});
// 	runtime_index
// }

/// 设置gui渲染的清屏颜色
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_clear_color(world: u32, r: f32, g: f32, b: f32, a: f32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let render_begin = world.world.fetch_single::<RenderBegin>().unwrap();
    let render_begin = render_begin.lend_mut();
	render_begin.0.clear_color = Some((
		OrderedFloat(r),
		OrderedFloat(g),
		OrderedFloat(b),
		OrderedFloat(a),
	));
}

/// 使gui渲染不清屏
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn nullify_clear_color(world: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let render_begin = world.world.fetch_single::<RenderBegin>().unwrap();
    let render_begin = render_begin.lend_mut();
	render_begin.0.clear_color = None;
}

/// 设置视口
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_view_port(world_id: u32, x: i32, y: i32, width: i32, height: i32) {
    set_render_dirty(world_id);
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let rb_decs = world
        .gui
        .world
        .fetch_single::<gui::single::RenderBegin>()
        .unwrap();
    let rb_decs = rb_decs.lend_mut();
	rb_decs.0.viewport = (x, y, width, height);
	rb_decs.get_notify_ref().modify_event(0, "", 0);
}

/// 设置视口
#[wasm_bindgen]
pub fn set_scissor(world_id: u32, x: i32, y: i32, width: i32, height: i32) {
    set_render_dirty(world_id);
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let rb_decs = world
        .gui
        .world
        .fetch_single::<gui::single::RenderBegin>()
        .unwrap();
    let rb_decs = rb_decs.lend_mut();
	rb_decs.0.scissor = (x, y, width, height);
}

/// 设置投影变换
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_project_transfrom(
    world_id: u32,
    scale_x: f32,
    scale_y: f32,
    translate_x: f32,
    translate_y: f32,
	rotate: f32,
	width: u32,
	height: u32,
) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let mut m = Matrix4::default();

    if scale_x != 1.0 || scale_y != 1.0 {
        m = m * Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0);
    }

    if translate_x != 0.0 || translate_y != 0.0 {
        m = m * Matrix4::from_translation(Vector3::new(translate_x, translate_y, 0.0));
    }

    if rotate != 0.0 {
        m = m * Matrix4::from_angle_z(cgmath::Deg(rotate))
    }

    let project_matrix = world
        .gui
        .world
        .fetch_single::<gui::single::ProjectionMatrix>()
        .unwrap();
    let project_matrix = project_matrix.lend_mut();
    project_matrix.0 = gui::component::calc::WorldMatrix(m, true) * gui::single::ProjectionMatrix::new(
        width as f32,
        height as f32,
        -Z_MAX - 1.0,
        Z_MAX + 1.0,
    ).0;
	project_matrix.get_notify_ref().modify_event(0, "", 0);
	
	let rect_layout_style1 = world.gui.rect_layout_style.lend_mut();
    let rect_layout_style = &mut rect_layout_style1[1];

    // let config = YgConfig::new();
    // config.set_point_scale_factor(0.0);
    // let ygnode1 = YgNode::new_with_config(config);
    // let ygnode1 = YgNode::default();
	rect_layout_style.size.width = Dimension::Points(width as f32);
	rect_layout_style.size.height = Dimension::Points(height as f32);

	// println!("project_matrix============={:?}, {}, {}, {}, {}", &project_matrix.0, scale_x, scale_y, width, height);
	
	// let layouts = world.gui.layout.lend_mut();
	// let layout = &mut layouts[1];
	// layout.rect.end = width as f32;
	// layout.rect.bottom = height as f32;
	rect_layout_style1.get_notify_ref().modify_event(1, "width", 0);
	// debug_println!("layout change, width: {}, height:{}", width, height);
}

/**
 * 文字纹理绘制时， 总是按照第一次取到的节点缩放值以及文字本身的样式来绘制， 以后更改缩放值， 无法重绘纹理
 * 调用此犯法，可强制更新一次缩放值
 */
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn force_update_text(world_id: u32, node_id: u32) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let idtree = world.gui.idtree.lend();
    let text_contents = world.gui.text_content.lend();
    let node = match idtree.get(node_id as usize) {
        Some(r) => r,
        None => return,
    };

    let notify = text_contents.get_notify_ref();
    if let Some(_r) = text_contents.get(node_id as usize) {
        notify.modify_event(node_id as usize, "", 0);
    }

    for (id, _n) in idtree.recursive_iter(node.children().head) {
        if let Some(_r) = text_contents.get(id) {
            notify.modify_event(id, "", 0);
        }
    }
}


/// 渲染gui， 通常每帧调用
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn render(world_id: u32) -> js_sys::Promise {
    let gui_world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    // #[cfg(feature = "debug")]
	// let time = std::time::Instant::now();

	let r = js_sys::Promise::resolve(&world_id.into()).then(&gui_world.draw_text);

	{
		// 纹理更新了, 设置脏
		let font_sheet = gui_world.gui.font_sheet.lend_mut();
		let font_sheet = &mut font_sheet.borrow_mut();
		if gui_world.old_texture_tex_version != font_sheet.tex_version {
			gui_world.old_texture_tex_version = font_sheet.tex_version;
			set_render_dirty(world_id);
		}
	}
    // gui_world.draw_text_sys.run(world_id);
    // #[cfg(feature = "debug")]
    // let draw_text_sys_time = std::time::Instant::now() - time;

    // #[cfg(feature = "debug")]
    // let time = std::time::Instant::now();
    let world = &mut gui_world.gui;
	load_image(world_id);

	let dirty_list_len = world.dirty_list.lend().0.len();
	
    // #[cfg(feature = "debug")]
	// let load_image_time = std::time::Instant::now() - time;
	let cur_time: usize = (Date::now() as u64/1000) as usize;
	let sys_time = world.system_time.lend_mut();
	sys_time.cur_time = cur_time;

    // #[cfg(feature = "debug")]
    // let time = std::time::Instant::now();
	world.world.run(&RENDER_DISPATCH);
	
	// #[cfg(feature = "debug")]
	if dirty_list_len > 0 {
		// console::log_1(&JsValue::from(format!("runtime======={:?}", world.world.runtime)));
	}
    // #[cfg(feature = "debug")]
    // let run_all_time = std::time::Instant::now() - time;

    // // 如果打开了性能检视面板， 应该渲染检视面板
    // if gui_world.performance_inspector > 0 {
    //     let performance_world = unsafe { &mut *(gui_world.performance_inspector as *mut GuiWorld) };
    //     performance_world.gui.world.run(&RENDER_DISPATCH);
    // }

    // #[cfg(feature = "debug")]
    // {
    //     let mut t = RunTime {
    //         draw_text_sys_time: draw_text_sys_time.as_secs_f64() * 1000.0,
    //         load_image_time: load_image_time.as_secs_f64() * 1000.0,
    //         run_all_time: run_all_time.as_secs_f64() * 1000.0,
    //         run_sum_time: 0.0,
    //         sys_time: Vec::with_capacity(world.world.runtime.len()),
    //     };

    //     if unsafe { gui::DIRTY } {
    //         for t1 in world.world.runtime.iter() {
    //             let time = t1.cost_time.as_secs_f64() * 1000.0;
    //             t.sys_time.push((t1.sys_name.as_ref().to_string(), time));
    //             t.run_sum_time += time;
    //         }

    //         #[cfg(feature = "debug")]
    //         js! {
    //             console.log("render", @{t});
    //         }
    //     }
    // }
    // unsafe { gui::DIRTY = false };
	return r;
}

#[cfg(feature = "debug")]
#[derive(Serialize, Debug)]
pub struct RunTime {
    pub draw_text_sys_time: f64,
    pub load_image_time: f64,
    pub run_all_time: f64,
    pub run_sum_time: f64,
    pub sys_time: Vec<(String, f64)>,
}

// #[cfg(feature = "debug")]
// js_serializable!(RunTime);

/// 强制计算一次
#[wasm_bindgen]
pub fn calc(world_id: u32) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    world.world.run(&CALC_DISPATCH);
}

/// 强制计算一次布局
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn cal_layout(world_id: u32) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    world.world.run(&LAYOUT_DISPATCH);

    // #[cfg(feature = "debug")]
    // {
    //     let mut t = RunTime {
    //         draw_text_sys_time: 0.0,
    //         load_image_time: 0.0,
    //         run_all_time: 0.0,
    //         run_sum_time: 0.0,
    //         sys_time: Vec::with_capacity(world.world.runtime.len()),
    //     };
    //     for t1 in world.world.runtime.iter() {
    //         let time = t1.cost_time.as_secs_f64() * 1000.0;
    //         t.sys_time.push((t1.sys_name.as_ref().to_string(), time));
    //         t.run_sum_time += time;
    //     }

    //     #[cfg(feature = "debug")]
    //     js! {
    //         console.log("layout", @{t});
    //     }
    // }
}

//设置shader
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_shader(engine: u32, shader_name: String, shader_code: String) {
    let engine = unsafe { &mut *(engine as usize as *mut ShareEngine<WebglHalContext>) };
    engine.gl.render_set_shader_code(&shader_name, &shader_code);
}

/// 加载图片成功后调用
/// image_name可以使用hash值与高层交互 TODO
/// __jsObj: image, __jsObj1: image_name(String)
#[wasm_bindgen]
pub fn load_image_success(
    world_id: u32,
    opacity: u8,
    compress: i32,
	r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
	name: u32,
	width: u32,
	height: u32,
	data: Object,
	cost: u32
) {
    let (res, name) = create_texture(world_id, opacity, compress, r_type, name, width, height, data, cost);
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let image_wait_sheet = world.image_wait_sheet.lend_mut();
    match image_wait_sheet.wait.remove(&name) {
        Some(r) => {
            image_wait_sheet.finish.push((name, res, r));
            image_wait_sheet.get_notify_ref().modify_event(0, "", 0);
        }
        None => (),
    };
}

/// 创建纹理资源
/// image_name可以使用hash值与高层交互 TODO
#[wasm_bindgen]
pub fn create_texture_res(
    world_id: u32,
    opacity: u8,
    compress: i32,
	r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
	name: u32,
	width: u32,
	height: u32,
	data: Object,
	cost: u32,
) -> u32 {
    Share::into_raw(create_texture(world_id, opacity, compress, r_type, name, width, height, data, cost).0) as u32
}

// 释放纹理资源
#[wasm_bindgen]
pub fn destroy_texture_res(texture: u32) {
	unsafe { Share::from_raw(texture as usize as *const (Share<TextureResRaw>, usize))};
}

pub fn create_texture(
    world_id: u32,
    opacity: u8,
    compress: i32,
	mut r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
	name: u32,
	width: u32,
	height: u32,
	data: Object,
	cost: u32,
) -> (Share<TextureResRaw>, usize) {
    if r_type > 2 {
        r_type = 0;
    }
	let name = name as usize;
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let engine = world.engine.lend_mut();

    let res = match engine.texture_res_map.get(&name) {
        Some(r) => return (r, name),
        None => {
            let opacity = unsafe { transmute(opacity) };

            let pformate = match opacity {
                ROpacity::Opaque => PixelFormat::RGB,
                ROpacity::Translucent | ROpacity::Transparent => PixelFormat::RGBA,
            };

            let texture = if compress < 0 {
				engine
					.gl
					.texture_create_2d_webgl(
						0,
						width,
						height,
						pformate,
						DataFormat::UnsignedByte,
						false,
						Some(&data), // obj = {wrap: Cnavas} | obj = {wrap: Image}
					)
					.unwrap()
			} else {
				engine
					.gl
					.compressed_texture_create_2d_webgl(
						0,
						width,
						height,
						CompressedTexFormat(compress as isize),
						// unsafe { transmute::<u8, CompressedTexFormat>(compress as u8) },
						false,
						Some(&data), // obj = {wrap: Uint8Array} | obj = {wrap: float32Array}
					)
					.unwrap()
			};
            let compress = if compress < 0 {
                None
            } else {
                Some(CompressedTexFormat(compress as isize))
                // Some(unsafe { transmute::<u8, CompressedTexFormat>(compress as u8) })
            };
            engine.create_texture_res(
                name,
                TextureResRaw::new(
                    width as usize,
                    height as usize,
                    pformate,
                    DataFormat::UnsignedByte,
                    opacity,
                    compress,
                    texture,
					Some(cost as usize)
                ),
                r_type as usize,
            )
        }
    };
    return (res, name);
}

/// 加载图片，调用高层接口，加载所有等待中的图片
fn load_image(world_id: u32) {
	// let mut clicks = 0;
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };

    let image_wait_sheet = &mut world.gui.image_wait_sheet.lend_mut();
    for img_name in image_wait_sheet.loads.iter() {
		(world.load_image)(*img_name as u32, world.load_image_success.as_ref().unchecked_ref());
		//  load_image(img_name.as_ref().to_string(), world.load_image_success.as_ref().unchecked_ref());
		// unsafe{loadImage(img_name.as_ref().to_string(),
		// 	world.load_image_success.as_ref().unchecked_ref()
		// 	)};
    }
    image_wait_sheet.loads.clear();
}

// /// 调试使用， 设置渲染脏， 使渲染系统在下一帧进行渲染
// #[wasm_bindgen]
// pub fn set_render_dirty(world: u32) {
// 	// println!("set_render_dirty============={}", world);
//     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//     let world = &mut world.gui;
//     let render_objs = world.render_objs.lend();
// 	let dirty_view_rect = world.dirty_view_rect.lend_mut();
// 	dirty_view_rect.4 = true;


	
//     render_objs.get_notify_ref().modify_event(1, "", 0);
// }

/// 纹理是否存在, 返回0表示不存在
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn texture_is_exist(world: u32, group_i: usize, name: usize) -> bool {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };

    let engine = world.gui.engine.lend();
    match engine.res_mgr.borrow().get::<TextureResRaw>(&name, group_i) {
        Some(_) => true,
        None => false,
    }
}

