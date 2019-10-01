use std::mem::transmute;

use serde::{Serialize};
use hash::XHashMap;
use hal_core::*;
use hal_webgl::*;

use ecs::{Lend};
use gui::component::user::*;
use gui::system::util::cal_matrix;
// use gui::single::Oct;
use gui::layout::FlexNode;
use gui::single::*;
use GuiWorld;
use gui::render::engine::ShareEngine;

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

    let octs = world.oct.lend();
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

	let info = Info {
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
pub fn overflow_clip(_world: u32) {
    // let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
	// let world = &mut world.gui;
    // let overflow_clip = world.overflow_clip.lend();
    // js!{
    //     console.log("overflow_clip:", @{format!("{:?}", **overflow_clip)});
    // }
}



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

#[derive(Serialize, Debug)]
struct Point2{
    x: f32, 
    y: f32,
}
js_serializable!( Point2 );

impl Point2 {
    fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
}

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