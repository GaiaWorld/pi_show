pub trait Res {
    type Key;
	// 获得资源的唯一名称
	fn name(&self) -> &Self::Key;
}

pub trait ResMgr{
    fn get<T: Res>(&self, key: &<T as Res>::Key) -> Option<T>;
    fn create<T: Res>(&mut self, value: T) -> T;
}