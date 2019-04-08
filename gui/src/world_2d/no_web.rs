use std::default::Default;

use wcs::world::{ComponentMgr};
use wcs::component::{SingleCase, SingleCaseWriteRef};
use world_2d::component::image::*;
use world_2d::component::sdf::*;
use world_2d::component::char_block::*;
use component::math::{Point2};

world!(
    struct World2dMgr{
        #[component]
        sdf: Sdf,

        #[component]
        word: CharBlock,

        #[component]
        image: Image,

        //全局数据
        #[single_component]
        overflow: Overflow,
        width: f32,
        height: f32,
    } 
);

impl World2dMgr {
    pub fn new() -> World2dMgr{
        World2dMgr{
            sdf: SdfGroup::default(),
            word: CharBlockGroup::default(),
            image: ImageGroup::default(),

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