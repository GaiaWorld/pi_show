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
#![recursion_limit = "512"]

#[macro_use]
extern crate serde;
extern crate stdweb_derive;
extern crate webgl_rendering_context;
#[macro_use]
extern crate stdweb;
extern crate ecs;
extern crate gui;
extern crate lazy_static;
extern crate paste;
extern crate map;
#[macro_use]
extern crate debug_info;
extern crate atom;
extern crate bincode;
extern crate cg2d;
extern crate data_view;
extern crate gui_tool;
extern crate hal_core;
extern crate hal_webgl;
extern crate hash;
extern crate octree;
extern crate ordered_float;
extern crate res;
extern crate share;
extern crate idtree;
extern crate flex_layout;

// use std::cell::RefCell;
use std::mem::transmute;

use ordered_float::OrderedFloat;
// use res::ResMgr;
use stdweb::unstable::TryInto;
use stdweb::Object;
use webgl_rendering_context::WebGLRenderingContext;
use flex_layout::{Size, Dimension, PositionType, Rect};

use atom::Atom;
use ecs::{Lend, LendMut};
use gui::component::calc::Visibility;
use gui::component::user::*;
use gui::render::engine::{Engine, ShareEngine, UnsafeMut};
use gui::render::res::Opacity as ROpacity;
use gui::render::res::TextureRes;
use gui::single::Class;
use gui::single::RenderBegin;
use gui::world::GuiWorld as GuiWorld1;
use gui::Z_MAX;
use gui::world::{create_res_mgr, create_world, LAYOUT_DISPATCH, RENDER_DISPATCH, CALC_DISPATCH};
use hal_core::*;
use hal_webgl::*;
use share::Share;

// // pub mod bc;
pub mod class;
#[cfg(not(feature = "no_debug"))]
pub mod debug;
pub mod layout;
pub mod node;
// // pub mod reset_style;
#[cfg(not(feature = "no_define_js"))]
pub mod rs_call_js;
pub mod style;
pub mod text;
pub mod transform;
pub mod world;
pub mod yoga;

// use bc::YgNode;
use node::define_set_class;
#[cfg(not(feature = "no_define_js"))]
use rs_call_js::define_js;
use text::DrawTextSys;

pub struct GuiWorld {
    pub gui: GuiWorld1<WebglHalContext>,
    pub draw_text_sys: DrawTextSys,
    pub default_text_style: TextStyle,
    pub default_attr: Class,
    pub performance_inspector: usize,
}

// /// 设置纹理的缓存配置
// #[allow(unused_attributes)]
// #[no_mangle]
// #[js_export]
// pub fn set_texture_catch_cfg(
//     res_mgr: u32, /* res_mgr指针 */
//     min_c1: u32,
//     max_c1: u32,
//     timeout1: u32,
//     min_c2: u32,
//     max_c2: u32,
//     timeout2: u32,
//     min_c3: u32,
//     max_c3: u32,
//     timeout3: u32,
// ) {
//     let res_mgr = unsafe { &mut *(res_mgr as *mut Share<RefCell<ResMgr>>) };
//     res_mgr.borrow_mut().register::<TextureRes>(
//         [
//             min_c1 as usize,
//             max_c1 as usize,
//             timeout1 as usize,
//             min_c2 as usize,
//             max_c2 as usize,
//             timeout2 as usize,
//             min_c3 as usize,
//             max_c3 as usize,
//             timeout3 as usize,
//         ],
//         "TextureRes".to_string(),
//     );
// }

/// total_capacity: 资源管理器总容量, 如果为0， 将使用默认的容量设置
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn create_engine(total_capacity: u32 /* 资源管理器总容量 */) -> u32 {
    let gl: WebGLRenderingContext = js!(return __gl;).try_into().unwrap();
    let use_vao = TryInto::<bool>::try_into(js!(var u = navigator.userAgent.toLowerCase(); return u.indexOf("ipad") < 0 && u.indexOf("iphone") < 0;)).unwrap();

    // let gl = WebglHalContext::new(gl, fbo, false);
    let gl = WebglHalContext::new(gl, use_vao);
    let engine = Engine::new(gl, create_res_mgr(total_capacity as usize));
    let r = Box::into_raw(Box::new(UnsafeMut::new(Share::new(engine)))) as u32;
    r
}

/// 创建渲染目标， 返回渲染目标的指针， 必须要高层调用destroy_render_target接口， 该渲染目标才能得到释放
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn create_render_target(world: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let fbo = TryInto::<Object>::try_into(js!(return {wrap: __fbo};)).unwrap();
    let engine = world.engine.lend_mut();
    let rt = Share::new(engine.gl.rt_create_webgl(fbo)); // 创建渲染目标
    Box::into_raw(Box::new(rt)) as u32
}

/// 销毁渲染目标
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn destroy_render_target(render_target: u32) {
    unsafe { Box::from_raw(&mut *(render_target as usize as *mut Share<HalRenderTarget>)) };
}

/// 绑定rendertarget
/// render_target为0时， 表示绑定gl默认的渲染目标， 当大于0时， render_target必须是一个RenderTarget的指针
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
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
#[no_mangle]
#[js_export]
pub fn clone_engine(engine: u32) -> u32 {
    let engine: ShareEngine<WebglHalContext> =
        ShareEngine::clone(unsafe { &*(engine as usize as *const ShareEngine<WebglHalContext>) });
    Box::into_raw(Box::new(engine)) as u32
}

/// 创建gui实例
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn create_gui(engine: u32, width: f32, height: f32) -> u32 {
    let mut engine =
        *unsafe { Box::from_raw(engine as usize as *mut ShareEngine<WebglHalContext>) };
    let draw_text_sys = DrawTextSys::new();
    let c = draw_text_sys.canvas.clone();
    let f = Box::new(move |name: &Atom, font_size: usize, ch: char| -> f32 {
        let ch = ch as u32;
        let font_size = font_size as u32;
        TryInto::<f64>::try_into(js! {
            var c = @{&c};
            c.ctx.font = @{font_size} + "px " + @{name.as_ref()};
            return c.ctx.measureText(String.fromCharCode(@{ch})).width;
        })
        .unwrap() as f32
    });
    let texture = engine
        .gl
        .texture_create_2d(
            0,
            2048,
            32,
            PixelFormat::RGBA,
            DataFormat::UnsignedByte,
            false,
            None,
        )
        .unwrap();
    let res = engine.create_texture_res(
        Atom::from("__$text".to_string()).get_hash(),
        TextureRes::new(
            2048,
            32,
            PixelFormat::RGBA,
            DataFormat::UnsignedByte,
            unsafe { transmute(1 as u8) },
            None, //unsafe { transmute(0 as usize) },
            texture,
        ),
        0,
    );
	let cur_time: u64 = js!{return Date.now()}.try_into().unwrap();
    let world = create_world::<WebglHalContext>(engine, width, height, f, res, cur_time);
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

    idtree.insert_child(node, 0, 0);
    let world = GuiWorld {
        gui: world,
        draw_text_sys: draw_text_sys,
        default_text_style: TextStyle::default(),
        default_attr: Class::default(),
        performance_inspector: 0,
	};
	
	
	// {
	// 	let runtime_ref = unsafe { &mut *( world.gui.world.runtime.as_ref() as *const Vec<ecs::RunTime> as *mut Vec<ecs::RunTime>)};
	// 	push_runtime(runtime_ref, Atom::from("draw_text"));
	// 	push_runtime(runtime_ref, Atom::from("all_run_time"));
	// }
	

    Box::into_raw(Box::new(world)) as u32
}

// fn push_runtime(runtime_ref: &mut Vec<ecs::RunTime>, name: Atom) -> usize {
// 	let runtime_index = runtime_ref.len();
// 	runtime_ref.push(ecs::RunTime{sys_name: name, cost_time: std::time::Duration::from_millis(0)});
// 	runtime_index
// }

/// 设置gui渲染的清屏颜色
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
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
#[no_mangle]
#[js_export]
pub fn nullify_clear_color(world: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let render_begin = world.world.fetch_single::<RenderBegin>().unwrap();
    let render_begin = render_begin.lend_mut();
	render_begin.0.clear_color = None;
}

/// 设置视口
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
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
}

/// 设置视口
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
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
#[no_mangle]
#[js_export]
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
	
	let rect_layout_style = world.gui.rect_layout_style.lend_mut();
    let rect_layout_style = unsafe { rect_layout_style.get_unchecked_mut(1) };

    // let config = YgConfig::new();
    // config.set_point_scale_factor(0.0);
    // let ygnode1 = YgNode::new_with_config(config);
    // let ygnode1 = YgNode::default();
	rect_layout_style.size.width = Dimension::Points(width as f32);
	rect_layout_style.size.height = Dimension::Points(width as f32);
	
	let layout = world.gui.layout.lend_mut();
	let layout = unsafe { layout.get_unchecked_mut(1) };
	layout.rect.end = width as f32;
	layout.rect.bottom = height as f32;
}

/**
 * 文字纹理绘制时， 总是按照第一次取到的节点缩放值以及文字本身的样式来绘制， 以后更改缩放值， 无法重绘纹理
 * 如果
 */
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
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
#[no_mangle]
#[js_export]
pub fn render(world_id: u32) {
    let gui_world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    // #[cfg(feature = "debug")]
	// let time = std::time::Instant::now();

    gui_world.draw_text_sys.run(world_id);
    // #[cfg(feature = "debug")]
    // let draw_text_sys_time = std::time::Instant::now() - time;

    // #[cfg(feature = "debug")]
    // let time = std::time::Instant::now();
    let world = &mut gui_world.gui;
	load_image(world_id);
    // #[cfg(feature = "debug")]
	// let load_image_time = std::time::Instant::now() - time;
	let cur_time: u64 = js!{return Date.now()}.try_into().unwrap();
	let sys_time = world.system_time.lend_mut();
	let cur_time = cur_time - sys_time.start_time;
	sys_time.cur_time = cur_time as usize;
    // #[cfg(feature = "debug")]
    // let time = std::time::Instant::now();
    world.world.run(&RENDER_DISPATCH);
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

#[cfg(feature = "debug")]
js_serializable!(RunTime);

/// 强制计算一次
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn calc(world_id: u32) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    world.world.run(&CALC_DISPATCH);
}

/// 强制计算一次布局
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
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
#[no_mangle]
#[js_export]
pub fn set_shader(engine: u32) {
    let shader_name: String = js!(return __jsObj;).try_into().unwrap();
    let shader_code: String = js!(return __jsObj1;).try_into().unwrap();
    let engine = unsafe { &mut *(engine as usize as *mut ShareEngine<WebglHalContext>) };
    engine.gl.render_set_shader_code(&shader_name, &shader_code);
}

/// 加载图片成功后调用
/// image_name可以使用hash值与高层交互 TODO
/// __jsObj: image, __jsObj1: image_name(String)
#[no_mangle]
#[js_export]
pub fn load_image_success(
    world_id: u32,
    opacity: u8,
    compress: i32,
    r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
) {
    let (res, name) = create_texture(world_id, opacity, compress, r_type);
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
/// __jsObj: image, __jsObj1: image_name(String)
#[no_mangle]
#[js_export]
pub fn create_texture_res(
    world_id: u32,
    opacity: u8,
    compress: i32,
    r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
) -> u32 {
    Share::into_raw(create_texture(world_id, opacity, compress, r_type).0) as u32
}

pub fn create_texture(
    world_id: u32,
    opacity: u8,
    compress: i32,
    mut r_type: u8, /* 缓存类型，支持0， 1， 2三种类型 */
) -> (Share<TextureRes>, usize) {
    if r_type > 2 {
        r_type = 0;
    }
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    let world = &mut world.gui;

	// let name: String = js! {return __jsObj1}.try_into().unwrap();
	// let name = Atom::from(name);
	let name: usize = js! {return __jsObj1}.try_into().unwrap();

    let engine = world.engine.lend_mut();

    let res = match engine.texture_res_map.get(&name) {
        Some(r) => return (r, name),
        None => {
            let width: u32 = js! {return __jsObj.width}.try_into().unwrap();
            let height: u32 = js! {return __jsObj.height}.try_into().unwrap();
            let opacity = unsafe { transmute(opacity) };

            let pformate = match opacity {
                ROpacity::Opaque => PixelFormat::RGB,
                ROpacity::Translucent | ROpacity::Transparent => PixelFormat::RGBA,
            };

            let texture = match TryInto::<Object>::try_into(js! {return {wrap: __jsObj};}) {
                Ok(obj) => {
                    if compress < 0 {
                        engine
                            .gl
                            .texture_create_2d_webgl(
                                0,
                                width,
                                height,
                                pformate,
                                DataFormat::UnsignedByte,
                                false,
                                Some(&obj), // obj = {wrap: Cnavas} | obj = {wrap: Image}
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
                                Some(&obj), // obj = {wrap: Uint8Array} | obj = {wrap: float32Array}
                            )
                            .unwrap()
                    }
                }
                Err(s) => panic!("set_src error, {:?}", s),
            };
            let compress = if compress < 0 {
                None
            } else {
                Some(CompressedTexFormat(compress as isize))
                // Some(unsafe { transmute::<u8, CompressedTexFormat>(compress as u8) })
            };
            engine.create_texture_res(
                name.clone(),
                TextureRes::new(
                    width as usize,
                    height as usize,
                    pformate,
                    DataFormat::UnsignedByte,
                    opacity,
                    compress,
                    texture,
                ),
                r_type as usize,
            )
        }
    };
    return (res, name);
}

/// 加载图片，调用高层接口，加载所有等待中的图片
fn load_image(world_id: u32) {
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };

    let image_wait_sheet = &mut world.gui.image_wait_sheet.lend_mut();
    for img_name in image_wait_sheet.loads.iter() {
        js! {
            if (window.__load_image) {
                window.__load_image(@{world_id}, @{*img_name as u32});
            } else {
                console.log("__load_image is undefined");
            }
        }
    }
    image_wait_sheet.loads.clear();
}

/// 调试使用， 设置渲染脏， 使渲染系统在下一帧进行渲染
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn set_render_dirty(world: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let render_objs = world.render_objs.lend();

    render_objs.get_notify_ref().modify_event(1, "", 0);
}

/// 纹理是否存在, 返回0表示不存在
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn texture_is_exist(world: u32, group_i: usize) -> bool {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    // let name: String = js! {return __jsObj1}.try_into().unwrap();
	// let name = Atom::from(name);
	let name: usize = js! {return __jsObj1}.try_into().unwrap();

    let engine = world.gui.engine.lend();
    match engine.res_mgr.get::<TextureRes>(&name, group_i) {
        Some(_) => true,
        None => false,
    }
}

fn main() {
    // 定义图片加载函数， canvas文字纹理绘制函数（使用feature: “no_define_js”, 将不会有这两个接口， 外部可根据需求自己实现 ）
    #[cfg(not(feature = "no_define_js"))]
    define_js();
    define_set_class();
}
