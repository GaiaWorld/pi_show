use std::ops::{Deref};
use std::fmt::{Debug, Formatter, Result};
use std::sync::Arc;

#[cfg(feature = "web")]
use webgl_rendering_context::{WebGLBuffer};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;
use component::color::{Color};
use component::math::{Matrix4, Point2, Color as MathColor};
use font::sdf_font::SdfFont;
use world_doc::component::style::text::{Shadow};

#[allow(unused_attributes)]
#[derive(Component)]
pub struct CharBlock{
    //world_matrix
    #[listen]
    pub world_matrix: Matrix4,

    pub offset: (f32, f32),
    pub extend: (f32, f32),

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

    pub line_height: f32, //设置行高

    pub shadow: Option<Shadow>, //设置阴影

    #[listen]
    pub sdf_font: Arc<SdfFont>,

    //顏色
    #[listen]
    pub color: Color,
    
    #[listen]
    pub chars: Vec<Char>,
}

impl Debug for CharBlock {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, 
        r#"CharBlock {{ 
            world_matrix: {:?}, 
            alpha: {}, 
            visibility: {}, 
            is_opaque: {}, 
            z_depth: {}, 
            by_overflow: {}, 
            stroke_size: {}, 
            stroke_color: {:?},
            font_size: {:?},
            color: {:?},
            chars: {:?},
            offset: {:?},
            extend: {:?}
        }}"#, self.world_matrix, self.alpha, self.visibility, self.is_opaque, self.z_depth, self.by_overflow, self.stroke_size, self.stroke_color, self.font_size, self.color, self.chars, self.offset, self.extend)
    }
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

    pub font_clamp: f32,
    pub smooth_range: f32,

    pub buffer_dirty: bool,

    pub indeices_len: u16,

    pub is_shadow: bool,
}

#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct CharBlockDefines {
    pub stroke: bool, //描边
    pub clip_plane: bool,//裁剪
    pub linear_color_gradient_2: bool, //线性渐变（两种颜色）
    pub linear_color_gradient_4: bool, // 线性渐变（四种颜色）
    pub color: bool, //单色
}

impl Default for CharBlockDefines {
    fn default() -> CharBlockDefines {
        CharBlockDefines {
            stroke: false,
            clip_plane: false,
            linear_color_gradient_2: false,
            linear_color_gradient_4: false,
            color: true,
        }
    }
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