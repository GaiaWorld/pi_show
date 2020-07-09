/**
 *  资源释放，每隔一秒对资源管理器进行扫描， 整理资源表缓存中超出的内存， 或超时的对象
 */
use std::marker::PhantomData;

use ecs::{Runner, SingleCaseImpl};

use single::SystemTime;
use hal_core::*;
use render::engine::ShareEngine;

pub struct ResReleaseSys<C: HalContext + 'static> {
    collect_time: usize, // 整理时间
    collect_interval: usize,
    marker: PhantomData<C>,
}

impl<C: HalContext + 'static> ResReleaseSys<C> {
    pub fn new() -> Self {
        Self {
            collect_time: 0,
            collect_interval: 1000, // 1秒钟扫描一次预整理列表
            marker: PhantomData,
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a, C: HalContext + 'static> Runner<'a> for ResReleaseSys<C> {
    type ReadData = &'a SingleCaseImpl<SystemTime>;
    type WriteData = &'a mut SingleCaseImpl<ShareEngine<C>>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData) {
        if read.cur_time >= self.collect_time {
            self.collect_time += self.collect_interval;
            engine.res_mgr.collect(read.cur_time as usize);
        }
	}
}

impl_system! {
    ResReleaseSys<C> where [C: HalContext + 'static],
    true,
    {

    }
}
