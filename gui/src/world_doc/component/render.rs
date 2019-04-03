use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;

pub trait DefinesList {
    fn list(&self) -> Vec<Atom>{
        return Vec::new();
    }
}

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

impl DefinesList for SdfDefines {
    fn list(&self) -> Vec<Atom> {
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

#[derive(Component, Default, Debug, Clone)]
pub struct TextDefines{
}

impl DefinesList for TextDefines{

}

#[derive(Component, Default, Debug, Clone)]
pub struct ImageDefines{

}

impl DefinesList for ImageDefines{
    
}

#[allow(unused_attributes)]
#[derive(Component)]
pub struct RenderObj {

    #[enum_component(Defines)]
    pub defines: DefinesId,

    pub program: u64,

    #[listen]
    pub is_opaque: bool, //是否不透明

    #[listen]
    pub z_index: f32,

    pub bind: Box<Bind>,
    
}

#[derive(EnumComponent)]
pub enum Defines {
    Sdf(SdfDefines),
    Text(TextDefines),
    Image(ImageDefines),
}

pub trait Bind {
    unsafe fn bind(&self, context: usize, id: usize);
    unsafe fn init_locations(&self, context: usize, id: usize);
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