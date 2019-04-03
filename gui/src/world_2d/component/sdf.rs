use std::ops::{Deref};

#[cfg(feature = "web")]
use webgl_rendering_context::{WebGLBuffer};
use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;


use component::math::{Vector2};
use component::color::{Color};
use component::math::{Color as CgColor, Aabb3};

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct Sdf{

    //alpha
    #[listen]
    pub alpha: f32,

    #[listen]
    pub is_opaque: bool,

    // z深度
    #[listen]
    pub z_depth: f32,

    // 被裁剪
    #[listen]
    pub by_overflow: usize,

    //圓角
    #[listen]
    pub radius: f32,

    // blur
    pub blur: f32,

    // 中心點
    pub center: Vector2,

    // extend
    pub extend: Vector2,

    // 旋轉角度
    pub rotate: f32,

    #[listen]
    pub bound_box: Aabb3,

    //顏色
    #[listen]
    pub color: Color,

    #[listen]
    pub border_size: f32,

    // 边框
    #[listen]
    pub border_color: CgColor,
}


#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct SdfEffect {
    pub program: u64,

    #[component(SdfDefines)]
    pub defines: usize,

    pub positions_buffer: WebGLBuffer,
    pub indeices_buffer: WebGLBuffer,

    pub positions_dirty: bool,

    pub sdf_id: usize,
}

#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct SdfDefines {
    pub sdf_rect: bool, //圆角
    pub stroke: bool, //描边
    pub clip_plane: bool,//裁剪
    pub linear_color_gradient_2: bool, //线性渐变（两种颜色）
    pub linear_color_gradient_4: bool, // 线性渐变（四种颜色）
    pub ellipse_color_gradient: bool, // 放射渐变 （四种颜色）
    pub color: bool, //单色
}

#[cfg(feature = "web")]
impl SdfDefines {
    pub fn list(&self) -> Vec<Atom> {
        let mut arr = Vec::new();
        if self.sdf_rect {
            arr.push(SDF_RECT.clone());
        }
        if self.stroke {
            arr.push(SDF_STROKE.clone());
        }
        if self.clip_plane {
            arr.push(SDF_CLIP_PLANE.clone());
        }
        if self.color {
            arr.push(SDF_COLOR.clone());
        }else if self.linear_color_gradient_2 {
            arr.push(SDF_LINEAR_COLOR_GRADIENT_2.clone());
        }else if self.linear_color_gradient_4 {
            arr.push(SDF_LINEAR_COLOR_GRADIENT_4.clone());
        }else if self.ellipse_color_gradient {
            arr.push(SDF_ELLIPSE_COLOR_GRADIENT.clone());
        }
        arr
    }
}

// defines
lazy_static! {
	static ref SDF_RECT: Atom = Atom::from("SDF_RECT");
    static ref SDF_STROKE: Atom = Atom::from("STROKE");
    static ref SDF_CLIP_PLANE: Atom = Atom::from("CLIP_PLANE");
    static ref SDF_LINEAR_COLOR_GRADIENT_2: Atom = Atom::from("LINEAR_COLOR_GRADIENT_2");
    static ref SDF_LINEAR_COLOR_GRADIENT_4: Atom = Atom::from("LINEAR_COLOR_GRADIENT_4");
    static ref SDF_ELLIPSE_COLOR_GRADIENT: Atom = Atom::from("ELLIPSE_COLOR_GRADIENT");
    static ref SDF_COLOR: Atom = Atom::from("COLOR");
}