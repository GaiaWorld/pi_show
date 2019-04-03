use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Handlers};
use wcs::world::{ComponentMgr};
use component::math::{Vector2};
use component::color::{Color};

#[allow(unused_attributes)]
#[derive(Debug, Component, Default)]
pub struct CharBlock{
    //alpha
    #[listen]
    pub alpha: f32,

    pub is_opaque: bool,

    // z深度
    #[listen]
    pub z_depth: f32,

    // 被裁剪
    #[listen]
    pub by_overflow: usize,

    // 中心點
    pub center: Vector2,

    // extend
    pub extend: Vector2,

    // 旋轉角度
    pub rotate: f32,

    //顏色
    pub color: Color,

    pub value: Vec<Char>,
}


#[derive(Debug)]
pub struct Char {
    value: char,
    pos: (f32, f32)
}

// pub struct WordBlockEffect {
//     program: u64,

//     #[component(ImageDefines)]
//     defines: usize,
// }

// #[allow(unused_attributes)]
// #[derive(Debug, Component, Default)]
// pub struct WordBlockDefines {

// }

// impl WordBlockDefines {
//     pub fn list(&self) -> Vec<Atom> {
//         Vec::new()
//     }
// }

// // // defines
// // lazy_static! {
// // 	static ref SDF_RECT: Atom = Atom::from("SDF_RECT");
// // }