/**
 *  
 */
use std::marker::PhantomData;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseImpl, MultiCaseImpl, Runner};
use hal_core::*;
use map::vecmap::VecMap;

use component::user::*;
use component::calc::{WorldMatrix, WorldMatrixRender};
use entity::{Node};
use single::*;
use system::util::*;

pub struct RenderMatrixSys<C: Context>{
    dirtys: Vec<usize>,
    dirty_mark: VecMap<bool>,
    
}

impl<'a, C: Context> RenderMatrixSys {
    pub fn new() -> Self{
        RenderMatrixSys {
            dirty_mark: VecMap::default(),
            dirtys: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<'a, C: Context> Runner<'a> for RenderMatrixSys{
    type ReadData = (
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<NodeRenderMap>
    );
    type WriteData = &'a mut MultiCaseImpl<Node, WorldMatrixRender>;
    fn run(&mut self, read: Self::ReadData, world_matrix_render: Self::WriteData){
        let (world_matrixs, transforms, layouts, default_table, _node_render_map) = read;
        let default_transform = default_table.get_unchecked::<Transform>();
        for i in self.dirtys.iter() {
            unsafe { *(self.dirty_mark.get_unchecked_mut(*i)) = false; }
            // if unsafe { node_render_map.get_unchecked(*i) }.len() > 0 {
                let r = cal_matrix(*i, world_matrixs, transforms, layouts, default_transform);
                world_matrix_render.insert(*i, WorldMatrixRender(r));
            // }
        }
        self.dirtys.clear();
    }
}

//Node创建 设脏
impl<'a, C: Context> EntityListener<'a, Node, CreateEvent> for RenderMatrixSys{
    type ReadData = ();
    type WriteData = () ;
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, _: Self::WriteData){
        self.dirty_mark.insert(event.id, true);
        self.dirtys.push(event.id);
    }
}

//Node删除 设脏
impl<'a, C: Context> EntityListener<'a, Node, DeleteEvent> for RenderMatrixSys{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, _: Self::WriteData){
        if unsafe { self.dirty_mark.remove_unchecked(event.id) } {
            self.dirtys.remove_item(&event.id);
        }
    }
}

//世界矩阵变化， 设脏
impl<'a, C: Context> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for RenderMatrixSys{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, _read: Self::ReadData, _write: Self::WriteData){
        let mark = unsafe { self.dirty_mark.get_unchecked_mut(event.id) };
        if *mark == false {
            *mark = true;
            self.dirtys.push(event.id);
        }
    }
}

unsafe impl<'a, C: Context> Sync for RenderMatrixSys{}
unsafe impl<'a, C: Context> Send for RenderMatrixSys{}

impl_system!{
    RenderMatrixSys where [C: Context],
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, DeleteEvent>
        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
    }
}