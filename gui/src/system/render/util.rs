use map::vecmap::VecMap;

pub struct Item {
    pub index: usize,
    pub dirty: usize,
}

impl Item {
    pub fn new(index: usize) -> Item{
        Item {
            index: index,
            dirty: std::usize::MAX,
        }
    }
}

pub struct Items {
    pub dirtys: Vec<usize>,
    pub render_map: VecMap<Item>,
}

impl Items {
    pub fn new () -> Items {
        Items{
            dirtys: Vec::new(),
            render_map: VecMap::new(), 
        }
    }

    pub  fn set_dirty(&mut self, id: usize, dirty: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            let dirty = item.dirty | (dirty as usize);
            if item.dirty | (dirty as usize) != item.dirty {
                item.dirty = dirty;
                self.dirtys.push(id);
            }
        }
    }
}

pub enum DrityType {
    Create = 1,
    VsProgram = 2,
    FsProGram = 4,
    Geometry = 8,
    Matrix = 16,
    Opacity = 32,
}