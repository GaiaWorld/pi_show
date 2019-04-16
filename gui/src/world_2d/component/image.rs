use std::ops::{Deref};
use std::rc::Rc;

#[cfg(feature = "web")]
use webgl_rendering_context::{WebGLBuffer};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;

use component::math::{Vector2, Matrix4, Color};
use render::res::TextureRes;

#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct Image{
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

    // extend
    #[listen]
    pub extend: Vector2,

    #[listen]
    pub src: Rc<TextureRes>,

     // z深度
    #[listen]
    pub color: Color,
}

impl Image {
    pub fn new(src: Rc<TextureRes>) -> Image{
        Image {
            world_matrix: Matrix4::default(),
            alpha: 1.0,
            is_opaque: true,
            z_depth: 0.0,
            by_overflow: 0,
            extend: Vector2::default(),
            visibility: true,
            src: src,
            color: Color(cg::color::Color::new(1.0, 1.0, 1.0, 1.0)),
        }
    }
}


#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component)]
pub struct ImageEffect {
    pub program: u64,

    #[component(ImageDefines)]
    pub defines: usize,

    pub positions_buffer: WebGLBuffer,
    pub uvs_buffer: WebGLBuffer,
    pub indeices_buffer: WebGLBuffer,

    pub positions_dirty: bool,

    pub image_id: usize,
}

#[cfg(feature = "web")]
#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct ImageDefines {
    pub clip_plane: bool,//裁剪
}

#[cfg(feature = "web")]
impl ImageDefines {
    pub fn list(&self) -> Vec<Atom> {
        let mut arr = Vec::new();
        if self.clip_plane {
            arr.push(SDF_CLIP_PLANE.clone());
        }
        arr
    }
}

// defines
lazy_static! {
    static ref SDF_CLIP_PLANE: Atom = Atom::from("CLIP_PLANE");
}