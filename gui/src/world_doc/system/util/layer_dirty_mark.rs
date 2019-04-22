/**
 * 按照层级关系记录node的脏标志
 */
use vecmap::{ VecMap, IndexMap};

use world_doc::WorldDocMgr;


pub struct LayerDirtyMark {
    pub dirtys: Vec<Vec<usize>>, //Vec<Vec<node_id>>
    pub dirty_mark_list: VecMap<bool>,
}

impl LayerDirtyMark {
    pub fn new() -> LayerDirtyMark{
        // 默认id为1的node为根， 根的创建没有事件， 因此默认插入根的脏
        let mut dirty_mark_list = VecMap::new();
        let mut dirtys = Vec::new();
        dirtys.push(Vec::new());
        dirtys[0].push(1);
        dirty_mark_list.insert(1, true);

        LayerDirtyMark{
            dirtys,
            dirty_mark_list,
        }
    }

    pub fn marked_dirty(&mut self, node_id: usize, mgr: &mut WorldDocMgr){
        let layer = {
            let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
            if *dirty_mark == true {
                return;
            }
            *dirty_mark = true;

            mgr.node._group.get(node_id).layer
        };

        if self.dirtys.len() <= layer{
            for _i in 0..(layer + 1 - self.dirtys.len()){
                self.dirtys.push(Vec::new());
            }
        }
        self.dirtys[layer].push(node_id);
    }

    pub fn delete_dirty(&mut self, node_id: usize, mgr: &mut WorldDocMgr){
        let node = mgr.node._group.get_mut(node_id);
        let dirty_mark = unsafe{self.dirty_mark_list.get_unchecked_mut(node_id)};
        if *dirty_mark == true {
            let layer = node.layer;
            for i in 0..self.dirtys[layer].len() {
                if self.dirtys[layer][i] == node_id {
                    self.dirtys[layer].swap_remove(i);
                    return;
                }
            }
        }
    }
}