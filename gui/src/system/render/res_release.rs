/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::marker::PhantomData;
use share::Share;
use std::hash::{ Hasher, Hash };

use ordered_float::NotNan;
use fxhash::FxHasher32;
use map::vecmap::VecMap;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use hal_core::*;
use atom::Atom;
use polygon::*;

use component::user::*;
use component::calc::*;
use component::calc::{Opacity};
use entity::*;
use single::*;
use render::engine::Engine;
use render::res::*;
use system::util::*;
use system::render::shaders::color::{COLOR_FS_SHADER_NAME, COLOR_VS_SHADER_NAME};
use system::render::util::*;

pub struct ResReleaseSys<C: HalContext + 'static>{
    items: Items,
    share_ucolor_ubo: VecMap<Share<dyn UniformBuffer>>, // 如果存在BackgroundClass， 也存在对应的ubo
    mark: PhantomData<C>,
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ResReleaseSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let res_mgr = engine.res_mgr;
    }
}

impl_system!{
    ResReleaseSys<C> where [C: HalContext + 'static],
    true,
    {

    }
}