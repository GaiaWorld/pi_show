use vecmap::VecMap;

pub struct DirtyMark {
    pub dirtys: Vec<usize>,
    pub dirty_mark_list: VecMap<bool>,
}

impl DirtyMark {
    pub fn new() -> DirtyMark{
        DirtyMark{
            dirtys: Vec::new(),
            dirty_mark_list: VecMap::new(),
        }
    }

    pub fn marked_dirty(&mut self, id: usize){
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(id)};
        if *dirty_mark == true {
            return;
        }
        *dirty_mark = true;

        self.dirtys.push(id);
    }

    pub fn delete_dirty(&mut self, id: usize){
        for i in 0..self.dirtys.len(){
            if self.dirtys[i] == id{
                self.dirtys.remove(i);
                return;
            }
        }
    }
}