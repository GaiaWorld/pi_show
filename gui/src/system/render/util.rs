use map::vecmap::VecMap;

pub struct Item<T> {
    pub index: T,
    pub dirty: usize,
}

impl<T> Item<T> {
    pub fn new(index: T) -> Item<T>{
        Item {
            index: index,
            dirty: std::usize::MAX,
        }
    }
}

#[derive(Default)]
pub struct Items<T> {
    pub dirtys: Vec<usize>,
    pub render_map: VecMap<Item<T>>,
}

impl<T> Items<T> {

    pub fn set_dirty(&mut self, id: usize, dirty: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            let dirty = item.dirty | (dirty as usize);
            if item.dirty | (dirty as usize) != item.dirty {
                item.dirty = dirty;
                self.dirtys.push(id);
            }
        }
    }

    pub fn set_dirty_no_push(&mut self, id: usize, dirty: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            let dirty = item.dirty | (dirty as usize);
            item.dirty = dirty;
        }
    }

    pub fn create(&mut self, id: usize, index: T) {
        self.render_map.insert(id, Item::new(index));
        self.dirtys.push(id);
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