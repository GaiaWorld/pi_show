use ecs::Runner;
use ecs::SingleCaseImpl;
use crate::single::DirtyList;

#[derive(Default)]
pub struct StyleVersion;

impl<'a> Runner<'a> for StyleVersion {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<DirtyList>;
    fn run(&mut self, _read: Self::ReadData, mut dirty_list: Self::WriteData) {
        // 清理后， 版本加1
        dirty_list.2 += 1;
    }
}

#[derive(Default)]
pub struct DirtyCount;

impl<'a> Runner<'a> for DirtyCount {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<DirtyList>;
    fn run(&mut self, _read: Self::ReadData, mut dirty_list: Self::WriteData) {
        dirty_list.3 = dirty_list.0.len();
    }
}

impl_system! {
    DirtyCount,
    true,
    {
    }
}

impl_system! {
    StyleVersion,
    true,
    {
    }
}
