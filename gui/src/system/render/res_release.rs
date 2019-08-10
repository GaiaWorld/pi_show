/**
 *  sdf物体（背景色， 边框颜色， 阴影）渲染管线的创建销毁， ubo的设置， attribute的设置
 */
use std::time::SystemTime;
use std::marker::PhantomData;

use ecs::{SingleCaseImpl, Runner};

use render::engine::Engine;
use hal_core::*;

pub struct ResReleaseSys<C>{
    system_time: SystemTime,
    conllect_time: u32, // 整理时间
    prepare_conllect: u32, // 预整理时间
    marker: PhantomData<C>,
    
}

impl<C: HalContext + 'static
> ResReleaseSys<C> {
    pub fn new() -> Self {
        Self{
            system_time: SystemTime::now(),
            conllect_time: std::u32::MAX,
            prepare_conllect: 3000, // 3秒钟扫描一次预整理列表
            marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static>  Runner<'a> for ResReleaseSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, _: Self::ReadData, engine: Self::WriteData){
        let now = self.system_time.elapsed().unwrap().as_secs() as u32 * 1000;

        if now >= self.prepare_conllect {
            engine.res_mgr.prepare_conllect(now, &mut self.conllect_time);
            self.prepare_conllect += 3000;
        }

        if now >= self.conllect_time {
            let mut conllect_time = std::u32::MAX;
            let engine1 = unsafe{&mut *( engine as *const SingleCaseImpl<Engine<C>> as usize as *mut SingleCaseImpl<Engine<C>>)};
            engine1.res_mgr.conllect(now, &mut conllect_time);
            self.conllect_time = conllect_time;
        }
    }
}

impl_system!{
    ResReleaseSys<C> where [C: HalContext + 'static],
    true,
    {

    }
}