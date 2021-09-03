// use std::default::Default;
// use std::sync::Arc;

// use hal_core::{Context, RenderBeginDesc};
// use atom::Atom;
// use cgmath::One;

// use ecs::{World, SeqDispatcher, Dispatcher};
// use ecs::idtree::IdTree;
// use component::user::*;
// use component::calc::*;
// use component::calc;
// use component::user;
// use bc::YgNode;
// use single::*;
// use entity::Node;
// use render::engine::Engine;
// use system::*;
// use font::font_sheet::FontSheet;
// use Z_MAX;

// pub fn create_world<C: Context + Sync + Send + 'static>(mut engine: Engine<C>, width: f32, height: f32) -> World{
//     let mut world = World::default();

//     let mut font = Font::default();
//     font.family = Atom::from("common");

//     let clip_sys = ClipSys::<C>::new(&mut engine, width as u32, height as u32);

//     //user
//     world.register_entity::<Node>();
//     world.register_multi::<Node, Transform>();;
//     world.register_multi::<Node, user::ZIndex>();
//     world.register_multi::<Node, Overflow>();
//     world.register_multi::<Node, Show>();
//     world.register_multi::<Node, user::Opacity>();
//     world.register_multi::<Node, BackgroundColor>();
//     world.register_multi::<Node, BoxShadow>();
//     world.register_multi::<Node, BorderColor>();
//     world.register_multi::<Node, BorderImage<C>>();
//     world.register_multi::<Node, BorderImageClip>();
//     world.register_multi::<Node, BorderImageSlice>();
//     world.register_multi::<Node, BorderImageRepeat>();
//     world.register_multi::<Node, CharBlock>();
//     world.register_multi::<Node, Text>();
//     world.register_multi::<Node, TextStyle>();
//     world.register_multi::<Node, TextShadow>();
//     world.register_multi::<Node, Font>();
//     world.register_multi::<Node, BorderRadius>();
//     world.register_multi::<Node, Image<C>>();
//     world.register_multi::<Node, ImageClip>();
//     world.register_multi::<Node, ObjectFit>();
//     world.register_multi::<Node, Filter>();

//     //calc
//     world.register_multi::<Node, ZDepth>();
//     world.register_multi::<Node, Enable>();
//     world.register_multi::<Node, Visibility>();
//     world.register_multi::<Node, WorldMatrix>();
//     world.register_multi::<Node, ByOverflow>();
//     world.register_multi::<Node, calc::Opacity>();
//     world.register_multi::<Node, Layout>();
//     world.register_multi::<Node, YgNode>();
//     world.register_multi::<Node, HSV>();
//     world.register_multi::<Node, WorldMatrixRender>();

//     //single
//     world.register_single::<ClipUbo<C>>(ClipUbo(Arc::new(engine.gl.create_uniforms())));
//     world.register_single::<IdTree>(IdTree::default());
//     world.register_single::<Oct>(Oct::new());
//     world.register_single::<OverflowClip>(OverflowClip::default());
//     world.register_single::<RenderObjs<C>>(RenderObjs::<C>::default());
//     world.register_single::<Engine<C>>(engine);
//     world.register_single::<FontSheet<C>>(FontSheet::<C>::default());
//     world.register_single::<ViewMatrix>(ViewMatrix(Matrix4::one()));
//     world.register_single::<ProjectionMatrix>(ProjectionMatrix::new(width, height, -Z_MAX - 1.0, Z_MAX + 1.0));
//     world.register_single::<RenderBegin>(RenderBegin(Arc::new(RenderBeginDesc::new(0, 0, width as i32, height as i32))));
//     world.register_single::<NodeRenderMap>(NodeRenderMap::new());

//     world.register_system(ZINDEX_N.clone(), CellZIndexImpl::new(ZIndexImpl::new()));
//     world.register_system(SHOW_N.clone(), CellShowSys::new(ShowSys::default()));
//     world.register_system(FILTER_N.clone(), CellFilterSys::new(FilterSys::default()));
//     world.register_system(OPCITY_N.clone(), CellOpacitySys::new(OpacitySys::default()));
//     world.register_system(LYOUT_N.clone(), CellLayoutSys::new(LayoutSys));
//     world.register_system(TEXT_LAYOUT_N.clone(), CellLayoutImpl::new(LayoutImpl::<C>::new()));
//     world.register_system(WORLD_MATRIX_N.clone(), CellWorldMatrixSys::new(WorldMatrixSys::default()));
//     world.register_system(OCT_N.clone(), CellOctSys::new(OctSys::default()));
//     world.register_system(OVERFLOW_N.clone(), CellOverflowImpl::new(OverflowImpl));
//     world.register_system(CLIP_N.clone(), CellClipSys::new(clip_sys));
//     world.register_system(IMAGE_N.clone(), CellImageSys::new(ImageSys::<C>::new()));
//     world.register_system(CHAR_BLOCK_N.clone(), CellCharBlockSys::<C>::new(CharBlockSys::new()));
//     world.register_system(CHAR_BLOCK_SHADOW_N.clone(), CellCharBlockShadowSys::<C>::new(CharBlockShadowSys::new()));
//     world.register_system(BG_COLOR_N.clone(), CellBackgroundColorSys::new(BackgroundColorSys::<C>::new()));
//     world.register_system(BR_COLOR_N.clone(), CellBorderColorSys::new(BorderColorSys::<C>::new()));
//     world.register_system(BR_IMAGE_N.clone(), CellBorderImageSys::new(BorderImageSys::<C>::new()));
//     world.register_system(BOX_SHADOW_N.clone(), CellBoxShadowSys::new(BoxShadowSys::<C>::new()));
//     world.register_system(NODE_ATTR_N.clone(), CellNodeAttrSys::new(NodeAttrSys::<C>::new()));
//     world.register_system(RENDER_N.clone(), CellRenderSys::new(RenderSys::<C>::default()));
//     world.register_system(WORLD_MATRIX_RENDER_N.clone(), CellRenderMatrixSys::new(RenderMatrixSys::<C>::new()));

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("z_index_sys, show_sys, filter_sys, opacity_sys, layout_sys, text_layout_sys, world_matrix_sys, oct_sys, overflow_sys, clip_sys, world_matrix_render, background_color_sys, border_color_sys, box_shadow_sys, image_sys, border_image_sys, charblock_sys, charblock_shadow_sys, node_attr_sys, render_sys".to_string(), &world);
//     world.add_dispatcher(RENDER_DISPATCH.clone(), dispatch);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("layout_sys, world_matrix_sys, oct_sys".to_string(), &world);
//     world.add_dispatcher(LAYOUT_DISPATCH.clone(), dispatch);

//     world
// }
use std::collections::hash_map::Entry;
use std::mem::transmute;

use gui::single::DirtyViewRect;
use gui::util::event::RenderObjEvent;
use gui::util::event::send_im_event;
use js_sys::{Object, Function};
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// use ecs::{Lend, LendMut};
use hash::XHashMap;

use hal_core::HalContext;
use hal_webgl::WebglHalContext;

use gui::component::user::{TextStyle, Vector2};
use gui::single::Class;
use gui::world::App;
use gui::font::font_sheet::{TextInfo as TextInfo1, FontSheet};
use share::Share;
use gui::util::cell::StdCell;

#[wasm_bindgen(module = "/js/utils.js")]
extern "C" {
	// #[wasm_bindgen]
	fn fillBackGround(canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d, x: u32, y: u32);
	// #[wasm_bindgen]
    fn setFont(ctx: &CanvasRenderingContext2d, weight: u32, fontSize: u32, font: u32, strokeWidth: u8);
	// #[wasm_bindgen]
	fn drawCharWithStroke(ctx: &CanvasRenderingContext2d, ch_code: u32, x: u32, y: u32);
	// #[wasm_bindgen]
	fn drawChar(ctx: &CanvasRenderingContext2d, ch_code: u32, x: u32, y: u32);
	// #[wasm_bindgen]
	pub fn measureText(ctx: &CanvasRenderingContext2d, ch: u32, font_size: u32, name: u32) -> f32;
	// #[wasm_bindgen]
	pub fn loadImage(image_name: u32, callback: &Function);
	// #[wasm_bindgen]
	pub fn useVao() -> bool;
}

pub struct GuiWorld {
    pub gui: App,
    pub draw_text_sys: DrawTextSys,
    pub default_attr: Class,
	pub max_texture_size: u32,
	pub performance_inspector: usize,
	pub load_image_success: Closure<dyn FnMut(
		u8,
		i32,
		u8, /* 缓存类型，支持0， 1， 2三种类型 */
		u32,
		u32,
		u32,
		Object,
		u32)>,
	pub load_image: Box<dyn Fn(u32, &Function)>,
	pub draw_text: Closure<dyn FnMut(JsValue)>,
	pub old_texture_tex_version: usize, // 上次run时的文字纹理版本
}

pub struct DrawTextSys {
	pub canvas: HtmlCanvasElement,
	pub ctx: CanvasRenderingContext2d,
}

impl DrawTextSys {
    pub fn new() -> Self {
		
		let window = window().expect("no global `window` exists");
		let document = window.document().expect("should have a document on window");
		let canvas = document.create_element("canvas").expect("create canvas fail");
		let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().expect("create canvas fail");
		let ctx = canvas.get_context("2d").expect("").unwrap().dyn_into::<CanvasRenderingContext2d>().expect("create canvas fail");
        // let obj: Object = TryInto::try_into(js! {
		// 	var c = document.createElement("canvas");
		// 	// c.style.position = "absolute";
        //     // document.body.append(c);// 查看效果
        //     var ctx = c.getContext("2d");
        //     return {canvas: c, ctx: ctx, wrap: c};
        // })
		// .unwrap();
        DrawTextSys { canvas: canvas, ctx: ctx  }
    }

    pub fn run(&mut self, world_id: u32) {
		let ptr;
		{
			let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
			let world = &mut world.gui;
			let font_sheet = world.get_resource::<Share<StdCell<FontSheet>>>().unwrap();
			let mut font_sheet = font_sheet.borrow_mut();
			if font_sheet.wait_draw_list.len() == 0 {
				return ;
			}

			let list = std::mem::replace(&mut font_sheet.wait_draw_list, Vec::default());
			ptr = Box::into_raw(Box::new(list)) as usize as u32;

			font_sheet.wait_draw_map.clear();
		}

		
		draw_canvas_text(world_id, ptr);
		
		// js_sys::Promise::resolve(&[world_id, ptr].into()).then(&Closure::once(draw_canvas_text));
		// draw_canvas_text(world_id, ptr)
    }
}

/// 绘制文字
#[allow(unused_unsafe)]
pub fn draw_canvas_text(world_id: u32, data: u32){
    // // let t = std::time::Instant::now();
    // let text_info_list = unsafe { Box::from_raw(data as usize as *mut Vec<TextInfo1>) };
    // let world1 = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    // let canvas = &world1.draw_text_sys.canvas;
	// let ctx = &world1.draw_text_sys.ctx;
    // let world = &mut world1.gui;
	// let font_sheet = world.get_resource::<FontSheet>().unwrap();
	// let font_sheet = &mut font_sheet.borrow_mut();
	// font_sheet.tex_version += 1;
    // let engine = world.engine.lend_mut();
    // let texture = font_sheet.get_font_tex();

    // // 将在绘制在同一行的文字归类在一起， 以便一起绘制，一起更新
    // let mut end_v = 0;
    // let mut map: XHashMap<u32, (Vec<usize>, Vector2)> = XHashMap::default();
    // for i in 0..text_info_list.len() {
    //     let text_info = &text_info_list[i];
    //     let first = &text_info.chars[0];
    //     let h = first.y + text_info.size.y as u32;
    //     if h > end_v {
    //         end_v = h;
    //     }
    //     match map.entry(first.y) {
    //         Entry::Occupied(mut e) => {
    //             let r = e.get_mut();
    //             r.0.push(i);
    //             r.1.x += text_info.size.x;
    //             if text_info.size.y > r.1.y {
    //                 r.1.y = text_info.size.y;
    //             }
    //         }
    //         Entry::Vacant(r) => {
    //             r.insert((vec![i], text_info.size.clone()));
    //         }
    //     };
    // }

    // // 扩展纹理
    // if end_v > texture.height as u32 {
    //     end_v = next_power_of_two(end_v);
    //     if end_v > world1.max_texture_size {
    //         debug_println!("update_canvas_text fail, height overflow");
    //     }
    //     engine
    //         .gl
    //         .texture_extend(&texture.bind, texture.width as u32, end_v);
    //     texture.update_size(texture.width, end_v as usize);
    //     single_font_sheet.get_notify_ref().modify_event(0, "", 0);
    // }

    // for indexs in map.iter() {
	// 	unsafe{fillBackGround(canvas, ctx, (indexs.1).1.x as u32, (indexs.1).1.y as u32)}
    //     let mut start: (i32, i32) = (-1, -1);

    //     for i in (indexs.1).0.iter() {
    //         let text_info = &text_info_list[*i];
    //         let first = &text_info.chars[0];
    //         if start.0 == -1 {
    //             start.0 = first.x as i32;
    //             start.1 = first.y as i32;
    //         }
    //         let hal_stroke_width = text_info.stroke_width / 2;
    //         // let bottom = text_info.size.y as u32 - hal_stroke_width as u32;
	// 		unsafe{
	// 			setFont(
	// 				ctx, 
	// 				text_info.weight as u32, 
	// 				text_info.font_size as u32, 
	// 				text_info.font as u32, 
	// 				text_info.stroke_width as u8);
	// 		};
    //         // js! {

    //         //     var c = @{canvas};
    //         //     var ctx = c.ctx;
    //         //     var weight;
    //         //     if (@{text_info.weight as u32} <= 300 ) {
    //         //         weight = "lighter";
    //         //     } else if (@{text_info.weight as u32} < 700 ) {
    //         //         weight = "normal";
    //         //     } else if (@{text_info.weight as u32} < 900 ) {
    //         //         weight = "bold";
    //         //     } else {
    //         //         weight = "bolder";
    //         //     }
    //         //     ctx.font = weight + " " + @{text_info.font_size as u32} + "px " + @{text_info.font.as_ref()};
    //         //     ctx.fillStyle = "#0f0";
    //         //     ctx.textBaseline = "top";
    //         // }
    //         if text_info.stroke_width > 0 {
    //             for char_info in text_info.chars.iter() {
    //                 let ch_code: u32 = unsafe { transmute(char_info.ch) };
    //                 let x = char_info.x + hal_stroke_width as u32 - start.0 as u32;
	// 				unsafe {
	// 					//fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
	// 					drawCharWithStroke(ctx, ch_code, x, text_info.top as u32);
	// 				}
	// 				// unsafe {useVao111(1);}
    //             }
    //         } else {
    //             for char_info in text_info.chars.iter() {
    //                 let ch_code: u32 = unsafe { transmute(char_info.ch) };
    //                 let x = char_info.x - start.0 as u32;
	// 				unsafe {
	// 					drawChar(ctx, ch_code, x, text_info.top as u32);
	// 				}
    //             }
    //         }
	// 	}

	// 	// // 在华为Mate 20上，将canvas更新到纹理存在bug，因此这里将canvas的数据取到，然后跟新到纹理
	// 	// // 如果在后续迭代的过程中，所有手机都不存在该bug，应该删除该句，以节省性能（getImageData会拷贝数据）
	// 	// js!{
	// 	// 	@{canvas}.wrap = @{canvas}.ctx.getImageData(0, 0, @{canvas}.canvas.width, @{canvas}.canvas.height);
	// 	// }
    //     engine
    //         .gl
    //         .texture_update_webgl(&texture.bind, 0, start.0 as u32, start.1 as u32, &canvas);
    // }

	// world1.old_texture_tex_version = font_sheet.tex_version;
    // set_render_dirty(world_id);
}

/// 调试使用， 设置渲染脏， 使渲染系统在下一帧进行渲染
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_render_dirty(world: u32) {
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let mut dirty_view_rect = world.get_resource_mut::<DirtyViewRect>().unwrap();
	dirty_view_rect.4 = true;

	send_im_event(world, RenderObjEvent::new_modify(usize::max_value(), "", 0));
}


#[derive(Debug, Serialize)]
pub struct TextInfoList {
    list: Vec<TextInfo>,
}

#[derive(Debug, Serialize)]
pub struct TextInfo {
    pub font: String,
    pub font_size: usize,
    pub stroke_width: usize,
    pub weight: usize,
    pub size: (f32, f32),
    pub chars: Vec<WaitChar>,
}

#[derive(Debug, Serialize)]
pub struct WaitChar {
    ch: char,
    width: f32,
    x: u32,
    y: u32,
}

pub fn next_power_of_two(value: u32) -> u32 {
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value += 1;
    value
}