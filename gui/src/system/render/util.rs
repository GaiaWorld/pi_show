use map::vecmap::VecMap;

pub struct Item {
    pub index: usize,
    pub position_change: bool,
    pub matrix_change: bool,
    pub pipeline_change: bool,
}

impl Item {
    pub fn new(index: usize) -> Item{
        Item {
            index: index,
            position_change: true,
            matrix_change: true,
            pipeline_change: true,
        }
    }
}

pub struct Items {
    pub geometry_dirtys: Vec<usize>,
    pub matrix_dirtys: Vec<usize>,
    pub pipeline_dirtys: Vec<usize>,
    pub render_map: VecMap<Item>,
}

impl Items {
    pub fn new () -> Items {
        Items{
            geometry_dirtys: Vec::new(),
            matrix_dirtys: Vec::new(),
            pipeline_dirtys: Vec::new(),
            render_map: VecMap::new(), 
        }
    }

    pub  fn set_geometry_dirty(&mut self, id: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(id);
            }
        }
    }

    pub  fn set_matrix_dirty(&mut self, id: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            if item.matrix_change == false {
                item.matrix_change = true;
                self.matrix_dirtys.push(id);
            }
        }
    }

    pub  fn set_pipeline_dirty(&mut self, id: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            if item.pipeline_change == false {
                item.pipeline_change = true;
                self.pipeline_dirtys.push(id);
            }
        }
    }

    pub  fn set_dirty(&mut self, id: usize) {
        if let Some(item) = self.render_map.get_mut(id) {
            if item.matrix_change == false {
                item.matrix_change = true;
                self.matrix_dirtys.push(id);
            }

            if item.position_change == false {
                item.position_change = true;
                self.geometry_dirtys.push(id);
            }
        }
    }
}