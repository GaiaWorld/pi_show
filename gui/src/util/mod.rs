use dirty::LayerDirty;

use self::vecmap_default::VecMapWithDefault;
use ecs::SingleCaseImpl;
use map::Map;
use crate::single::IdTree;

pub mod vecmap_default;
pub mod hashmap_default;


#[derive(Default)]
pub struct Dirty {
	pub dirty_mark_list: VecMapWithDefault<DirtyMark>,
	pub dirty: LayerDirty<usize>,
}

#[derive(Default, Clone)]
pub struct DirtyMark {
	pub layer: usize,
	pub ty: usize,
}

impl Dirty {
	pub fn with_capacity(capacity: usize) -> Dirty {
		Dirty{
			dirty_mark_list: VecMapWithDefault::with_capacity(capacity), // VecMap<layer>
    		dirty: LayerDirty::default(),
		}
	}

    pub fn marked_dirty(&mut self, id: usize, id_tree: &IdTree, ty: usize) {
        match id_tree.get(id) {
            Some(r) => {
                if r.layer() != 0 {
					let d = &mut self.dirty_mark_list[id];
                    if d.layer != r.layer() {
                        if d.layer != 0 {
                            self.dirty.delete(id, d.layer);
                        }
                        d.layer = r.layer();
                        self.dirty.mark(id, r.layer());
                    }
					d.ty |= ty;
                }
            }
            _ => (),
        };
    }
}