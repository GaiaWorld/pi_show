/**
 *  资源释放， 
 */
use std::marker::PhantomData;
use std::time::SystemTime;

use ecs::{SingleCaseImpl, Runner};

use hal_core::*;
use render::engine::ShareEngine;

pub struct ResReleaseSys<C: HalContext + 'static>{
    system_time: SystemTime,
    collect_time: usize, // 整理时间
    collect_interval: usize,
	marker: PhantomData<C>,
}

impl<C: HalContext + 'static> ResReleaseSys<C> {
    pub fn new() -> Self {
        let system_time = SystemTime::now();
        let now = system_time.elapsed().unwrap().as_secs() as usize * 1000;
        Self{
            system_time: system_time,
            collect_time: now,
            collect_interval: 1000, // 3秒钟扫描一次预整理列表
			marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ResReleaseSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<ShareEngine<C>>;
    fn run(&mut self, _: Self::ReadData, engine: Self::WriteData){
        let now = self.system_time.elapsed().unwrap().as_secs() as usize * 1000;
        if now >= self.collect_time {
            self.collect_time += self.collect_interval;
            engine.res_mgr.collect(now);
        }
    }
}

impl_system!{
	ResReleaseSys<C> where [C: HalContext + 'static],
    true,
    {

    }
}