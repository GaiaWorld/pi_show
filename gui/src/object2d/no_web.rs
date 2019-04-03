use std::default::Default;

use wcs::world::{ComponentMgr};
use wcs::component::{SingleCase, SingleCaseWriteRef};
use object2d::component::image::*;
use object2d::component::sdf::*;
use object2d::component::char_block::*;
use generic_component::math::{Point2};

world!(
    struct Object2dMgr{
        #[component]
        sdfs: Sdf,

        #[component]
        words: CharBlock,

        #[component]
        images: Image,

        //全局数据
        #[single_component]
        overflow: Overflow,
        width: f32,
        height: f32,
    } 
);

impl Object2dMgr {
    pub fn new() -> Object2dMgr{
        Object2dMgr{
            sdfs: SdfGroup::default(),
            words: CharBlockGroup::default(),
            images: ImageGroup::default(),

            overflow: SingleCase::new(Overflow([0;8],[[Point2::default();4];8])),
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}

#[derive(Debug)]
pub struct Overflow(pub [usize;8], pub [[Point2;4];8]);