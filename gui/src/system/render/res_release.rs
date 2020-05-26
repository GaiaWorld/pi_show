/**
 *  资源释放，每隔一秒对资源管理器进行扫描， 整理资源表缓存中超出的内存， 或超时的对象
 */
use std::marker::PhantomData;
use std::time::SystemTime;

use ecs::{Runner, SingleCaseImpl};

use hal_core::*;
use render::engine::ShareEngine;

pub struct ResReleaseSys<C: HalContext + 'static> {
    system_time: SystemTime,
    collect_time: usize, // 整理时间
    collect_interval: usize,
    marker: PhantomData<C>,
}

impl<C: HalContext + 'static> ResReleaseSys<C> {
    pub fn new() -> Self {
        let system_time = SystemTime::now();
        // let now = system_time.elapsed().unwrap().as_secs() as usize * 1000;
        Self {
            system_time: system_time,
            collect_time: 0,
            collect_interval: 1000, // 1秒钟扫描一次预整理列表
            marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ResReleaseSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<ShareEngine<C>>;
    fn run(&mut self, _: Self::ReadData, engine: Self::WriteData) {
        let now = match self.system_time.elapsed() {
            Ok(r) => r.as_secs() as usize * 1000,
            Err(_) => panic!("system_time elapsed fail"),
        };
        if now >= self.collect_time {
            self.collect_time += self.collect_interval;
            engine.res_mgr.collect(now);
            // println!(
            //     "texture_total=================size:{},max_capacity:{}",
            //     engine.texture_res_map.caches[0].size(),
            //     engine.texture_res_map.caches[0].get_max_capacity()
            // );
        }
    }
}

impl_system! {
    ResReleaseSys<C> where [C: HalContext + 'static],
    true,
    {

    }
}
