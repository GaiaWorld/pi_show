/**
 *  资源释放， 
 */
use std::time::SystemTime;
use std::cell::RefCell;

use ecs::{SingleCaseImpl, Runner};
use res::ResMgr;
use share::Share;

pub struct ResReleaseSys{
    system_time: SystemTime,
    collect_time: usize, // 整理时间
    collect_interval: usize,
}

impl ResReleaseSys {
    pub fn new() -> Self {
        let system_time = SystemTime::now();
        let now = system_time.elapsed().unwrap().as_secs() as usize * 1000;
        Self{
            system_time: system_time,
            collect_time: now,
            collect_interval: 1000, // 3秒钟扫描一次预整理列表
        }
    }
}

// 将顶点数据改变的渲染对象重新设置索引流和顶点流
impl<'a>  Runner<'a> for ResReleaseSys{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Share<RefCell<ResMgr>>>;
    fn run(&mut self, _: Self::ReadData, res_mgr: Self::WriteData){
        let now = self.system_time.elapsed().unwrap().as_secs() as usize * 1000;
        if now >= self.collect_time {
            self.collect_time += self.collect_interval;
            res_mgr.borrow_mut().collect(now);
        }
    }
}

impl_system!{
    ResReleaseSys,
    true,
    {

    }
}