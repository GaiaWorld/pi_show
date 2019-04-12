use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use atom::Atom;

use component::math::{Vector2, Matrix4};
use render::res::TextureRes;

//  // Attributes
//     attribute vec2 uv;
//     attribute vec3 position;
    
//     // Uniforms
//     uniform vec4 uvOffsetScale;
//     uniform mat4 worldViewProjection;
   
//     // Varyings
//     varying vec2 vuv;

 // Uniforms
    // uniform vec4 color;
    // uniform sampler2D texture;

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct Image{
    //world_matrix
    #[listen]
    pub world_matrix: Matrix4,

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

    // extend
    pub extend: Vector2,

    // pub src: Rc<TextureRes>,
    pub src: u32
}


#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct ImageEffect {
    pub program: u64,

    #[component(ImageDefines)]
    pub defines: usize,

    pub image_id: usize,
}

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct ImageDefines {

}

impl ImageDefines {
    pub fn list(&self) -> Vec<Atom> {
        Vec::new()
    }
}

// // defines
// lazy_static! {
// 	static ref SDF_RECT: Atom = Atom::from("SDF_RECT");
// }