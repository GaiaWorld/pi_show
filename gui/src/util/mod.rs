use std::hash::{Hash, Hasher};

use dirty::LayerDirty;
use hash::DefaultHasher;
use ordered_float::NotNan;

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

pub fn f32_4_hash(r: f32, g: f32, b: f32, a: f32) -> u64 {
    let mut hasher = DefaultHasher::default();
    if let Err(_r) = NotNan::new(r) {
        log::info!("r=============={}", r);
    }
    if let Err(g) = NotNan::new(g) {
        log::info!("g=============={}", g);
    }
    if let Err(r) = NotNan::new(b) {
        log::info!("b=============={}", b);
    }
    if let Err(r) = NotNan::new(a) {
        log::info!("a=============={}", a);
    }
    NotNan::new(r).unwrap().hash(&mut hasher);
    NotNan::new(g).unwrap().hash(&mut hasher);
    NotNan::new(b).unwrap().hash(&mut hasher);
    NotNan::new(a).unwrap().hash(&mut hasher);
    hasher.finish()
}