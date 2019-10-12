use std::mem::transmute;

use serde::{Serialize};
use hash::XHashMap;
use hal_core::*;
use hal_webgl::*;

use ecs::{Lend, LendMut};
use gui::component::user::*;
use gui::component::calc::*;
use gui::render::res::*;
use gui::system::util::cal_matrix;
use gui::entity::Node;
// use gui::single::Oct;
use gui::layout::FlexNode;
use gui::single::*;
use GuiWorld;
use gui::render::engine::ShareEngine;
use bc::YgNode;

// 打印节点信息
#[allow(unused_attributes)]
#[no_mangle]
pub fn node_info(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;

    let z_depth = unsafe { world.z_depth.lend().get_unchecked(node) }.0;

    let enable = unsafe { world.enable.lend().get_unchecked(node) }.0;
    let visibility = unsafe { world.visibility.lend().get_unchecked(node) }.0;

    let by_overflow = unsafe { world.by_overflow.lend().get_unchecked(node) }.0;

    let opacity = unsafe { world.opacity.lend().get_unchecked(node) }.0;

    let layout = world.layout.lend();

    let world_matrix = world.world_matrix.lend();

    let transform = world.transform.lend();

    let world_matrix1 = cal_matrix(node, world_matrix, transform, layout, &Transform::default());
    let layout = unsafe { layout.get_unchecked(node) };
    
    // border box
    let b_left_top = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    let b_left_bottom = world_matrix1 * Vector4::new(0.0, layout.height, 1.0, 1.0);
    let b_right_bottom = world_matrix1 * Vector4::new(layout.width, layout.height, 1.0, 1.0);
    let b_right_top = world_matrix1 * Vector4::new(layout.width, 0.0, 1.0, 1.0);
    
    // border box
    let absolute_b_box = Quad {
        left_top: Point2::new(b_left_top.x, b_left_top.y),
        left_bottom: Point2::new(b_left_bottom.x, b_left_bottom.y),
        right_bottom: Point2::new(b_right_bottom.x, b_right_bottom.y),
        right_top: Point2::new(b_right_top.x, b_right_top.y),
    };

    // padding box
    let p_left_top = world_matrix1 * Vector4::new(layout.border_left, layout.border_top, 1.0, 1.0);
    let p_left_bottom = world_matrix1 * Vector4::new(layout.border_left, layout.height - layout.border_bottom, 1.0, 1.0);
    let p_right_bottom = world_matrix1 * Vector4::new(layout.width - layout.border_right, layout.height - layout.border_bottom, 1.0, 1.0);
    let p_right_top = world_matrix1 * Vector4::new(layout.width - layout.border_right, layout.border_top, 1.0, 1.0);

    let absolute_p_box = Quad {
        left_top: Point2::new(p_left_top.x, p_left_top.y),
        left_bottom: Point2::new(p_left_bottom.x, p_left_bottom.y),
        right_bottom: Point2::new(p_right_bottom.x, p_right_bottom.y),
        right_top: Point2::new(p_right_top.x, p_right_top.y),
    };

    // content box
    let c_left_top = world_matrix1 * Vector4::new(layout.border_left + layout.padding_left, layout.border_top + layout.padding_top, 1.0, 1.0);
    let c_left_bottom = world_matrix1 * Vector4::new(layout.border_left + layout.padding_left, layout.height - layout.border_bottom - layout.padding_bottom, 1.0, 1.0);
    let c_right_bottom = world_matrix1 * Vector4::new(layout.width - layout.border_right - layout.padding_right, layout.height - layout.border_bottom - layout.padding_bottom, 1.0, 1.0);
    let c_right_top = world_matrix1 * Vector4::new(layout.width - layout.border_right - layout.padding_right, layout.border_top + layout.padding_top, 1.0, 1.0);
    
    let absolute_c_box = Quad {
        left_top: Point2::new(c_left_top.x, c_left_top.y),
        left_bottom: Point2::new(c_left_bottom.x, c_left_bottom.y),
        right_bottom: Point2::new(c_right_bottom.x, c_right_bottom.y),
        right_top: Point2::new(c_right_top.x, c_right_top.y),
    };

    // let yogas = world.yoga.lend();
    // let yoga = unsafe { yogas.get_unchecked(node) };

    // let octs = world.oct.lend();
    // let oct = unsafe { octs.get_unchecked(node) };
	
	let mut render_map = Vec::new();
	let map = world.world.fetch_single::<NodeRenderMap>().unwrap();
	let map = map.lend();
	let render_objs = world.world.fetch_single::<RenderObjs>().unwrap();
	let render_objs = render_objs.lend();
	let engine = world.world.fetch_single::<ShareEngine<WebglHalContext>>().unwrap();
	let engine = engine.lend();
	if let Some(arr) = map.get(node) {
		for id in arr.iter() {
			let v = match render_objs.get(*id) {
				Some(r) => r,
				None => continue,
			};
			let mut paramter = XHashMap::default();
			// let pt = v.paramter.get_texture_layout();

			let val = v.paramter.get_values();
			let vals = v.paramter.get_single_uniforms();
			// let valt = v.paramter.get_textures();
			let mut i = 0;
			for name in v.paramter.get_layout() {
				let mut ubo = XHashMap::default();
				let ubo_val = val[i].get_values();
				let mut j = 0;
				for n in val[i].get_layout(){
					ubo.insert(n.to_string(), ubo_val[j].clone());
					j += 1;
				}
				paramter.insert(name.to_string(), Paramter::Ubo(ubo) );
				i += 1;
			}

			i = 0;
			for name in v.paramter.get_single_uniform_layout() {
				paramter.insert(name.to_string(), Paramter::Uniform(vals[i].clone()));
				i += 1;
			}

			let rs = engine.gl.rs_get_desc(&v.state.rs);
			let bs = engine.gl.bs_get_desc(&v.state.bs);

			let mut vs_defines = Vec::new();
			for n in v.vs_defines.list().iter() {
				if let Some(r) = n {
					vs_defines.push(r.to_string())
				}
			}

			let mut fs_defines = Vec::new();
			for n in v.fs_defines.list().iter() {
				if let Some(r) = n {
					fs_defines.push(r.to_string())
				}
			}
			
			let obj = RenderObject {
				depth: v.depth,
				depth_diff: v.depth_diff,
				visibility: v.visibility,
				is_opacity: v.is_opacity,
				vs_name: v.vs_name.as_ref().to_string(),
				fs_name: v.fs_name.as_ref().to_string(),
				vs_defines: vs_defines,
				fs_defines: fs_defines,
				paramter: paramter,
				program_dirty: v.program_dirty,

				program: v.program.is_some(),
				geometry: v.geometry.is_some(),
				state: State {
					rs: unsafe {transmute(rs.clone())},
					bs: unsafe {transmute(bs.clone())},
					ss: engine.gl.ss_get_desc(&v.state.ss).clone(),
					ds: engine.gl.ds_get_desc(&v.state.ds).clone(),
				},

				context: v.context,
			};
			render_map.push(obj);
		}
	}

	let char_block = world.world.fetch_multi::<Node, CharBlock<YgNode>>().unwrap();
	let char_block = char_block.lend();
	let char_block = match char_block.get(node) {
		Some(r) => {
			let mut c = CharBlock1 {
				font_size: r.font_size,
				font_height: r.font_height,
				stroke_width: r.stroke_width,
				line_height: r.line_height,
				chars: Vec::default(),
				lines: r.lines.clone(),
				last_line: r.last_line,
				size: r.size,
				wrap_size: r.wrap_size,
				pos: r.pos,
				line_count: r.line_count,
				fix_width: r.fix_width,
				style_class: r.style_class,
				is_pixel: r.is_pixel,
			};
			for i in r.chars.iter(){
				c.chars.push(CharNode{
					ch: i.ch,
					width: i.width,
					pos: i.pos,
					ch_id_or_count: i.ch_id_or_count,
					base_width: i.base_width,
				});
			}
			Some(c)
		},
		None => None,
	};

	let info = Info {
		char_block: char_block,
		overflow:  unsafe { world.overflow.lend().get_unchecked(node)}.0,
        by_overflow: by_overflow,
        visibility: visibility,
        enable: enable,
        opacity: opacity,
        zindex: z_depth,
        layout: unsafe { transmute(layout.clone()) },
        border_box: absolute_b_box,
        padding_box: absolute_p_box,
        content_box: absolute_c_box,
		culling: unsafe { world.culling.lend().get_unchecked(node) }.0,
		text: match world.text_style.lend().get(node) {
			Some(r) => Some(r.clone()),
			None => None,
		},
		text_content: match world.text_content.lend().get(node){
			Some(r) => Some(r.clone()),
			None => None,
		},
		render_obj: render_map,
    };

    js!{
        window.__jsObj = @{info};
        // window.__jsObj1 = window.__jsObj;
        console.log("node_info:", window.__jsObj);
        // console.log("style:", @{format!( "{:?}", yoga.get_style() )});
        // console.log("layout:", @{format!( "{:?}", yoga.get_layout() )});
        // console.log("boundBox:", @{format!( "{:?}", oct )});
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn overflow_clip(world: u32) {
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let overflow_clip = world.overflow_clip.lend();

	let mut clips: Vec<(usize, Clip)> = Vec::new();
	for (index, v) in overflow_clip.clip.iter(){
		clips.push((index, v.clone()));
	}

	let mut clip_map = XHashMap::default();
	for (k, v) in overflow_clip.clip_map.iter(){
		clip_map.insert(*k, v.0.clone());
	}
	let c = OverflowClip {
		id_map: overflow_clip.id_map.clone(),
		clip: clips,
		clip_map: clip_map,
	};
    js!{
        console.log("overflow_clip:", @{c});
    }
}

#[allow(unused_attributes)]
#[no_mangle]
fn res_size(world: u32){
	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
	let engine = world.engine.lend();
	let mut size = ResMgrSize::default();

	let texture = engine.texture_res_map.all_res();
	for i in texture.0.iter() {
		size.texture += i.1;
		size.count_texture += 1;
	}
	for i in texture.1.iter() {
		size.catch_texture += i.1.elem.cost;
		size.count_catch_texture += 1;
	}

	let geometry = engine.geometry_res_map.all_res();
	for i in geometry.0.iter() {
		size.geometry += i.1;
		size.count_geometry += 1;
	}
	for i in geometry.1.iter() {
		size.catch_geometry += i.1.elem.cost;
		size.count_catch_geometry += 1;
	}

	let buffer = engine.buffer_res_map.all_res();
	for i in buffer.0.iter() {
		size.buffer += i.1;
		size.count_buffer += 1;
	}
	for i in buffer.1.iter() {
		size.catch_buffer += i.1.elem.cost;
		size.count_catch_buffer += 1;
	}

	let rs = engine.rs_res_map.all_res();
	for i in rs.0.iter() {
		size.rs += i.1;
		size.count_rs += 1;
	}
	for i in rs.1.iter() {
		size.catch_rs += i.1.elem.cost;
		size.count_catch_rs += 1;
	}

	let bs = engine.bs_res_map.all_res();
	for i in bs.0.iter() {
		size.bs += i.1;
		size.count_bs += 1;
	}
	for i in bs.1.iter() {
		size.catch_bs += i.1.elem.cost;
		size.count_catch_bs += 1;
	}

	let ss = engine.ss_res_map.all_res();
	for i in ss.0.iter() {
		size.ss += i.1;
		size.count_ss += 1;
	}
	for i in ss.1.iter() {
		size.catch_ss += i.1.elem.cost;
		size.count_catch_ss += 1;
	}

	let ds = engine.ds_res_map.all_res();
	for i in ds.0.iter() {
		size.ds += i.1;
		size.count_ds += 1;
	}
	for i in ds.1.iter() {
		size.catch_ds += i.1.elem.cost;
		size.count_catch_ds += 1;
	}

	let sampler = engine.sampler_res_map.all_res();
	for i in sampler.0.iter() {
		size.sampler += i.1;
		size.count_sampler += 1;
	}
	for i in sampler.1.iter() {
		size.catch_sampler += i.1.elem.cost;
		size.count_catch_sampler += 1;
	}

	let ucolor = engine.res_mgr.fetch_map::<UColorUbo>().unwrap();
	let ucolor = ucolor.all_res();
	for i in ucolor.0.iter() {
		size.ucolor += i.1;
		size.count_ucolor += 1;
	}
	for i in ucolor.1.iter() {
		size.catch_ucolor+= i.1.elem.cost;
		size.count_catch_ucolor += 1;
	}

	let hsv = engine.res_mgr.fetch_map::<HsvUbo>().unwrap();
	let hsv = hsv.all_res();
	for i in hsv.0.iter() {
		size.hsv += i.1;
		size.count_hsv += 1;
	}
	for i in hsv.1.iter() {
		size.catch_hsv += i.1.elem.cost;
		size.count_catch_hsv += 1;
	}

	let msdf_stroke = engine.res_mgr.fetch_map::<MsdfStrokeUbo>().unwrap();
	let msdf_stroke = msdf_stroke.all_res();
	for i in msdf_stroke.0.iter() {
		size.msdf_stroke += i.1;
		size.count_msdf_stroke += 1;
	}
	for i in msdf_stroke.1.iter() {
		size.catch_msdf_stroke += i.1.elem.cost;
		size.count_catch_msdf_stroke += 1;
	}

	let canvas_stroke = engine.res_mgr.fetch_map::<CanvasTextStrokeColorUbo>().unwrap();
	let canvas_stroke = canvas_stroke.all_res();
	for i in canvas_stroke.0.iter() {
		size.canvas_stroke += i.1;
		size.count_canvas_stroke += 1;
	}
	for i in canvas_stroke.1.iter() {
		size.catch_canvas_stroke += i.1.elem.cost;
		size.count_catch_canvas_stroke += 1;
	}

	js!{
		console.log("res_mgr_size: ", @{size});
	}
}

#[derive(Serialize, Debug, Default)]
struct ResMgrSize{
	count_texture: usize,
	count_geometry: usize,
	count_buffer: usize,
	count_sampler: usize,
	count_rs: usize,
	count_bs: usize,
	count_ss: usize,
	count_ds: usize,
	count_ucolor: usize,
	count_hsv: usize,
	count_msdf_stroke: usize,
	count_canvas_stroke: usize,

	count_catch_texture: usize,
	count_catch_geometry: usize,
	count_catch_buffer: usize,
	count_catch_sampler: usize,
	count_catch_rs: usize,
	count_catch_bs: usize,
	count_catch_ss: usize,
	count_catch_ds: usize,
	count_catch_ucolor: usize,
	count_catch_hsv: usize,
	count_catch_msdf_stroke: usize,
	count_catch_canvas_stroke: usize,

    texture: usize,
	geometry: usize,
	buffer: usize,
	sampler: usize,
	rs: usize,
	bs: usize,
	ss: usize,
	ds: usize,
	ucolor: usize,
	hsv: usize,
	msdf_stroke: usize,
	canvas_stroke: usize,

	
	catch_texture: usize,
	catch_geometry: usize,
	catch_buffer: usize,
	catch_sampler: usize,
	catch_rs: usize,
	catch_bs: usize,
	catch_ss: usize,
	catch_ds: usize,
	catch_ucolor: usize,
	catch_hsv: usize,
	catch_msdf_stroke: usize,
	catch_canvas_stroke: usize,
}
js_serializable!( ResMgrSize );





// #[allow(unused_attributes)]
// #[no_mangle]
// pub fn bound_box(world: u32, node: u32) {
//     let node = node as usize
//     let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	// let world = &mut world.gui;
//     let overflow_clip = world.fetch_single::<OverflowClip>().unwrap();
//     js!{
//         console.log("overflow_clip:", @{format!("{:?}", &overflow_clip.value)});
//     }
// }

#[allow(unused_attributes)]
#[no_mangle]
pub fn get_world_matrix(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let world_matrixs = world.world_matrix.lend();
	let world_matrix = match world_matrixs.get(node) {
		Some(r) => r,
		None => return,
	};
    js!{
        console.log("world_matrix:", @{format!("{:?}", &world_matrix)});
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn get_transform(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let transforms = world.transform.lend();
	let transform = match transforms.get(node) {
		Some(r) => r,
		None => return,
	};
    js!{
        console.log("transform:", @{format!("{:?}", &transform)});
    }
}

#[allow(unused_attributes)]
#[no_mangle]
pub fn get_yoga(world: u32, node: u32) {
    let node = node as usize;
    let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;
    let yogas = world.yoga.lend();
	let (style, layout) = match yogas.get(node) {
		Some(r) => (r.get_style(), r.get_layout()),
		None => return,
	};
	let layout1 = match world.layout.lend().get(node) {
		Some(r) => r,
		None => return,
	};
    js!{
        console.log("style:", @{format!("{:?}", &style)});
		console.log("layout:", @{format!("{:?}", &layout)});
		console.log("layout1:", @{format!("{:?}", layout1)});
    }
}

// #[derive(Serialize, Debug)]
// struct Point2{
//     x: f32, 
//     y: f32,
// }
// js_serializable!( Point2 );

// impl Point2 {
//     fn new(x: f32, y: f32) -> Self {
//         Self {x, y}
//     }
// }

#[derive(Serialize, Debug)]
struct Quad{
    left_top: Point2,
    left_bottom: Point2,
    right_bottom: Point2,
    right_top: Point2,
}
js_serializable!( Quad );

#[derive(Serialize, Debug)]
struct Layout1{
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    border_left: f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    padding_left: f32,
    padding_top: f32,
    padding_right: f32,
    padding_bottom: f32,
}
js_serializable!( Layout1 );

#[derive(Serialize, Debug)]
struct Info{
	overflow: bool,
    by_overflow: usize,
    visibility: bool,
    enable: bool,
    opacity: f32,
    zindex: f32,
    layout: Layout1,
    border_box: Quad,
    padding_box: Quad,
    content_box: Quad,
	culling: bool,
	text: Option<TextStyle>,
	text_content: Option<TextContent>,
	render_obj: Vec<RenderObject>,
	char_block: Option<CharBlock1>,
}
js_serializable!( Info );

#[derive(Serialize, Debug)]
struct RenderObject {
	pub depth: f32,
    pub depth_diff: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub vs_name: String,
    pub fs_name: String,
    pub vs_defines: Vec<String>,
    pub fs_defines: Vec<String>,
    pub paramter: XHashMap<String, Paramter>,
    pub program_dirty: bool,

    pub program: bool,
    pub geometry: bool,
    pub state: State,

    pub context: usize,
}
js_serializable!( RenderObject );

#[derive(Serialize, Debug)]
enum Paramter {
	Uniform(UniformValue),
	Ubo(XHashMap<String, UniformValue>),
}
js_serializable!( Paramter );

#[derive(Serialize, Debug)]
struct State {
	pub rs: RasterStateDesc,
    pub bs: BlendStateDesc,
    pub ss: StencilStateDesc,
    pub ds: DepthStateDesc,
}
js_serializable!( State );

#[derive(Serialize, Debug)]
pub struct RasterStateDesc {
    pub cull_mode: Option<CullMode>,
    pub is_front_face_ccw: bool,
    pub polygon_offset: (f32, f32),
}
js_serializable!( RasterStateDesc );

#[derive(Serialize, Debug)]
pub struct BlendStateDesc {
    pub rgb_equation: BlendFunc,
    pub alpha_equation: BlendFunc,
    
    pub src_rgb_factor: BlendFactor,
    pub dst_rgb_factor: BlendFactor,
    
    pub src_alpha_factor: BlendFactor,
    pub dst_alpha_factor: BlendFactor,

    pub const_rgba: (f32, f32, f32, f32),
}
js_serializable!( BlendStateDesc );

#[derive(Serialize, Debug)]
pub struct OverflowClip{
    pub id_map: XHashMap<usize, usize>,
    pub clip: Vec<(usize, Clip)>,
    pub clip_map: XHashMap<usize, Aabb3>,
}

js_serializable!( OverflowClip );

#[derive(Serialize, Debug)]
pub struct CharBlock1 {
  pub font_size: f32, // 字体高度
  pub font_height: f32, // 字体高度
  pub stroke_width: f32, //描边宽度
  pub line_height: f32,
  pub chars: Vec<CharNode>, // 字符集合
  pub lines: Vec<(usize, usize, f32)>, // 不折行下的每行的起始字符位置、单词数量和总宽度。 自动折行不会影响该值
  pub last_line: (usize, usize, f32), // 最后一行的起始字符位置、单词数量和总宽度
  pub size: Vector2,
  pub wrap_size: Vector2,
  pub pos: Point2,
  pub line_count: usize, // 行数，
  pub fix_width: bool, // 如果有字宽不等于font_size
  pub style_class: usize, // 使用的那个样式类
  pub is_pixel: bool,
}

// 字符节点， 对应一个字符的
#[derive(Serialize, Debug, Clone)]
pub struct CharNode{
  pub ch: char, // 字符
  pub width: f32, // 字符宽度
  pub pos: Point2, // 位置
  pub ch_id_or_count: usize, // 字符id或单词的字符数量
  pub base_width: f32, // font_size 为32 的字符宽度
}




#[allow(unused_attributes)]
#[no_mangle]
pub fn test_create_render_obj(world: u32, count: u32) {
	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	let world = &mut world.gui;

	let default_state = world.world.fetch_single::<gui::single::DefaultState>().unwrap();
	let default_state = default_state.lend();
	let render_objs = world.world.fetch_single::<RenderObjs>().unwrap();
	let render_objs = render_objs.lend_mut();
	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj(default_state);
	}
	println!("create_render_obj: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj1(default_state);
	}
	println!("create_render_obj1: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj3(default_state);
	}
	println!("create_render_obj3: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj4(default_state);
	}
	println!("create_render_obj4: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj5(default_state);
	}
	println!("create_render_obj5: {:?}", std::time::Instant::now() - time);

	let mut m = map::vecmap::VecMap::default();
	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj6(&mut m, 2, render_objs, default_state);
	}
	println!("create_render_obj6: {:?}", std::time::Instant::now() - time);

	let mut m = map::vecmap::VecMap::default();
	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj7(&mut m, 2, render_objs, default_state);
	}
	println!("create_render_obj7: {:?}", std::time::Instant::now() - time);

	let p: share::Share<dyn hal_core::ProgramParamter> = share::Share::new(ImageParamter::default());
	let time = std::time::Instant::now();
	for i in 0..count {
		create_render_obj13(&mut m, 2, render_objs, default_state, &p);
	}
	println!("create_render_obj13: {:?}", std::time::Instant::now() - time);


	let read = (world.copacity.lend(), world.visibility.lend(), world.hsv.lend(), world.z_depth.lend(), world.culling.lend());
	let render_objs = world.world.fetch_single::<gui::single::RenderObjs>().unwrap();
	let node_render_map = world.world.fetch_single::<gui::single::NodeRenderMap>().unwrap();
	let write = (render_objs.lend_mut(), node_render_map.lend_mut());
	let v:Option<share::Share<dyn UniformBuffer>> = Some(share::Share::new(gui::component::calc::ViewMatrixUbo::new(hal_core::UniformValue::MatrixV4(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0]))));
	let p:Option<share::Share<dyn UniformBuffer>> = Some(share::Share::new(gui::component::calc::ProjectMatrixUbo::new(hal_core::UniformValue::MatrixV4(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0]))));

	// let mut m = map::vecmap::VecMap::default();
	let time = std::time::Instant::now();
	for i in 0..count {
		render_objs_create8((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
	}
	println!("create_render_obj8: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		render_objs_create9((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
	}
	println!("render_objs_create9: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		render_objs_create10((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
	}
	println!("render_objs_create10: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		render_objs_create11((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
	}
	println!("render_objs_create11: {:?}", std::time::Instant::now() - time);

	let time = std::time::Instant::now();
	for i in 0..count {
		render_objs_create12((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
	}
	println!("render_objs_create12: {:?}", std::time::Instant::now() - time);

	

	

}

//  RenderObj {
//         depth: 0.0,
//         program_dirty: true,
//         visibility: false,
//         vs_defines: Box::new(VsDefines::default()),
//         fs_defines: Box::new(FsDefines::default()),
//         program: None,
//         geometry: None,
//         depth_diff,
//         is_opacity,
//         vs_name,
//         fs_name,
//         paramter,
//         state,
//         context,
//     }

#[inline]
pub fn create_render_obj(
    default_state: &gui::single::DefaultState,
){
    let state = gui::single::State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	gui::system::util::new_render_obj(1, 2.0, true, gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(), gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(), share::Share::new(gui::component::calc::ImageParamter::default()), state);
}


#[inline]
pub fn create_render_obj1(
    default_state: &gui::single::DefaultState,
){
    let state = gui::single::State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
	let notify = default_state.df_ds.clone();
}

#[inline]
pub fn create_render_obj3(
    default_state: &gui::single::DefaultState,
){
    let state = gui::single::State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    let vs = gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone();
	let fs = gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone();
	let p = share::Share::new(gui::component::calc::ImageParamter::default());

}

#[inline]
pub fn create_render_obj4(
    default_state: &gui::single::DefaultState,
){
    let state = gui::single::State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
	let p = share::Share::new(gui::component::calc::ImageParamter::default());

}

#[inline]
pub fn create_render_obj5(
    default_state: &gui::single::DefaultState,
){
    let state = gui::single::State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
	share::Share::new(1);
	share::Share::new(1);
	share::Share::new(1);
	share::Share::new(1);
	share::Share::new(1);
	share::Share::new(1);
	share::Share::new(1);
}

#[inline]
fn create_render_obj6(
	render_map: &mut map::vecmap::VecMap<usize>,
	id: usize,
	render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
	default_state: &DefaultState,
) -> usize{
	gui::system::util::create_render_obj(
		id,
		-0.1,
		true,
		gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(),
		gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(),
		share::Share::new(ImageParamter::default()),
		default_state, render_objs,
		render_map
	)
}

#[inline]
fn create_render_obj7(
	render_map: &mut map::vecmap::VecMap<usize>,
	id: usize,
	render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
	default_state: &DefaultState,
) -> usize{
	create_render_obj_(
		id,
		-0.1,
		true,
		gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(),
		gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(),
		share::Share::new(ImageParamter::default()),
		default_state, render_objs,
		render_map
	)
}

#[inline]
pub fn create_render_obj_(
    context: usize,
    depth_diff: f32,
    is_opacity: bool,
    vs_name: atom::Atom,
    fs_name: atom::Atom,
    paramter: share::Share<dyn ProgramParamter>,
    default_state: &DefaultState,
    render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
    render_map: &mut map::vecmap::VecMap<usize>,
) -> usize{
    let state = gui::single::State {
        bs: default_state.df_bs.clone(),
        rs: default_state.df_rs.clone(),
        ss: default_state.df_ss.clone(),
        ds: default_state.df_ds.clone(),
    };
    let notify = render_objs.get_notify();
    let render_index = render_objs.insert(
        gui::system::util::new_render_obj(context, depth_diff, is_opacity, vs_name, fs_name, paramter, state),
        None
    );
    render_map.insert(context, render_index);
    render_index
}

fn render_objs_create8<'a>(read: (
        &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
        &'a ecs::MultiCaseImpl<Node, Visibility>,
        &'a ecs::MultiCaseImpl<Node, HSV>,
        &'a ecs::MultiCaseImpl<Node, ZDepth>,
        &'a ecs::MultiCaseImpl<Node, Culling>,
    ), 
	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
) {
	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
	let (render_objs, node_render_map) = write;
	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
	let notify = node_render_map.get_notify();
	unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };
	
	let paramter = &mut render_obj.paramter;

	paramter.set_value("viewMatrix", view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
	paramter.set_value("projectMatrix", project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

	let z_depth = unsafe { z_depths.get_unchecked(render_obj.context) }.0;
	let opacity = unsafe { opacitys.get_unchecked(render_obj.context) }.0;
	paramter.set_single_uniform("alpha", UniformValue::Float1(opacity)); // alpha
	debug_println!("id: {}, alpha: {:?}", render_obj.context, opacity);

	let visibility = unsafe { visibilitys.get_unchecked(render_obj.context) }.0;
	let culling = unsafe { cullings.get_unchecked(render_obj.context) }.0;
	render_obj.visibility = visibility & !culling;

	render_obj.depth = z_depth + render_obj.depth_diff;

	let hsv = unsafe { hsvs.get_unchecked(render_obj.context) };
	if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
		render_obj.fs_defines.add("HSV");
		// paramter.set_value("hsvValue", self.create_hsv_ubo(hsv)); // hsv
	}
}

fn render_objs_create9<'a>(read: (
        &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
        &'a ecs::MultiCaseImpl<Node, Visibility>,
        &'a ecs::MultiCaseImpl<Node, HSV>,
        &'a ecs::MultiCaseImpl<Node, ZDepth>,
        &'a ecs::MultiCaseImpl<Node, Culling>,
    ), 
	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
) {
	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
	let (render_objs, node_render_map) = write;
	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
	let notify = node_render_map.get_notify();
	unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };
	
	let paramter = &mut render_obj.paramter;

	paramter.set_value("viewMatrix", view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
	paramter.set_value("projectMatrix", project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

}

fn render_objs_create10<'a>(read: (
        &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
        &'a ecs::MultiCaseImpl<Node, Visibility>,
        &'a ecs::MultiCaseImpl<Node, HSV>,
        &'a ecs::MultiCaseImpl<Node, ZDepth>,
        &'a ecs::MultiCaseImpl<Node, Culling>,
    ), 
	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
) {
	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
	let (render_objs, node_render_map) = write;
	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
	let notify = node_render_map.get_notify();
	unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };
}

fn render_objs_create11<'a>(read: (
        &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
        &'a ecs::MultiCaseImpl<Node, Visibility>,
        &'a ecs::MultiCaseImpl<Node, HSV>,
        &'a ecs::MultiCaseImpl<Node, ZDepth>,
        &'a ecs::MultiCaseImpl<Node, Culling>,
    ), 
	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
) {
	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
	let (render_objs, node_render_map) = write;
	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
	let notify = node_render_map.get_notify();
	// unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };
}

fn render_objs_create12<'a>(read: (
        &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
        &'a ecs::MultiCaseImpl<Node, Visibility>,
        &'a ecs::MultiCaseImpl<Node, HSV>,
        &'a ecs::MultiCaseImpl<Node, ZDepth>,
        &'a ecs::MultiCaseImpl<Node, Culling>,
    ), 
	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
) {
	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
	let (render_objs, node_render_map) = write;
	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
	let notify = node_render_map.get_notify();
	// unsafe{ node_render_map.add_unchecked(render_obj.context, 3, notify) };
}

#[inline]
fn create_render_obj13(
	render_map: &mut map::vecmap::VecMap<usize>,
	id: usize,
	render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
	default_state: &DefaultState,
	p: &share::Share<dyn hal_core::ProgramParamter>
) -> usize{
	create_render_obj_(
		id,
		-0.1,
		true,
		gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(),
		gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(),
		p.clone(),
		default_state, render_objs,
		render_map
	)
}