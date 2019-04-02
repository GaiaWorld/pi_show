pub mod component;
pub mod system;

use std::default::Default;

use wcs::world::{ComponentMgr};
use object2d::component::image::*;
use object2d::component::sdf::*;
use object2d::component::word::*;

world!(
    struct Object2dMgr{
        #[component]
        sdfs: Sdf,

        #[component]
        words: Word,

        #[component]
        images: Image,
    } 
);

impl Default for Object2dMgr {
    fn default() -> Object2dMgr{
        Object2dMgr{
            sdfs: SdfGroup::default(),
            words: WordGroup::default(),
            images: ImageGroup::default(),
        }
    }
}