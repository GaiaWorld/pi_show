use std::ops::{Deref};
use std::rc::Rc;

#[cfg(feature = "web")]
use webgl_rendering_context::{WebGLBuffer};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;
use component::color::{Color};
use component::math::{Matrix4, Point2, Color as MathColor};
use text_layout::font::SdfFont;

#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct CharBlock{
    //world_matrix
    #[listen]
    pub world_matrix: Matrix4,

    //alpha
    #[listen]
    pub alpha: f32,

    //visibility
    #[listen]
    pub visibility: bool,

    #[listen]
    pub is_opaque: bool,

    // z深度
    #[listen]
    pub z_depth: f32,

    // 被裁剪
    #[listen]
    pub by_overflow: usize,
    
    #[listen]
    pub stroke_size: f32,

    #[listen]
    pub stroke_color: MathColor,

    #[listen]
    pub font_size: f32,

    #[listen]
    pub sdf_font: Rc<SdfFont>,

    //顏色
    #[listen]
    pub color: Color,
    
    #[listen]
    pub chars: Vec<Char>,
}


#[derive(Debug)]
pub struct Char {
    pub value: char,
    pub pos: Point2,
}

#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct CharBlockEffect {
    pub program: u64,

    #[component(CharBlockDefines)]
    pub defines: usize,

    pub positions_buffer: WebGLBuffer,
    pub uvs_buffer: WebGLBuffer,
    pub indeices_buffer: WebGLBuffer,

    pub extend: Point2,

    pub font_clamp: f32,
    pub smooth_range: f32,

    pub buffer_dirty: bool,

    pub indeices_len: u16,
}

#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct CharBlockDefines {
    pub sdf_rect: bool, //圆角
    pub stroke: bool, //描边
    pub clip_plane: bool,//裁剪
    pub linear_color_gradient_2: bool, //线性渐变（两种颜色）
    pub linear_color_gradient_4: bool, // 线性渐变（四种颜色）
    pub color: bool, //单色
}

#[cfg(feature = "web")]
impl CharBlockDefines {
    pub fn list(&self) -> Vec<Atom> {
        let mut arr = Vec::new();
        if self.stroke {
            arr.push(CHAR_STROKE.clone());
        }
        if self.clip_plane {
            arr.push(CHAR_CLIP_PLANE.clone());
        }
        if self.color {
            arr.push(CHAR_COLOR.clone());
        }else if self.linear_color_gradient_2 {
            arr.push(CHAR_LINEAR_COLOR_GRADIENT_2.clone());
        }else if self.linear_color_gradient_4 {
            arr.push(CHAR_LINEAR_COLOR_GRADIENT_4.clone());
        }
        arr
    }
}

// defines
lazy_static! {
    static ref CHAR_STROKE: Atom = Atom::from("STROKE");
    static ref CHAR_CLIP_PLANE: Atom = Atom::from("CLIP_PLANE");
    static ref CHAR_LINEAR_COLOR_GRADIENT_2: Atom = Atom::from("LINEAR_COLOR_GRADIENT_2");
    static ref CHAR_LINEAR_COLOR_GRADIENT_4: Atom = Atom::from("LINEAR_COLOR_GRADIENT_4");
    static ref CHAR_ELLIPSE_COLOR_GRADIENT: Atom = Atom::from("ELLIPSE_COLOR_GRADIENT");
    static ref CHAR_COLOR: Atom = Atom::from("COLOR");
}