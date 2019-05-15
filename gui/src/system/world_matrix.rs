/**
 * 监听transform和position组件， 利用transform和position递归计算节点的世界矩阵（worldmatrix组件）
 */

use std::cell::RefCell;
use std::rc::{Rc};

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::idtree::{ IdTree, Node as IdTreeNode};
use dirty::LayerDirty;

use component::{
    user::{ Transform},
    calc::{ WorldMatrix},
};
use map::vecmap::{VecMap};
use layout::Layout;
use entity::{Node};

#[derive(Default)]
pub struct WorldMatrixSys{
    dirty_mark_list: VecMap<bool>,
    dirty: LayerDirty,
}

impl WorldMatrixSys{
    fn marked_dirty(&mut self, id: usize, id_tree: &SingleCaseImpl<IdTree>){
        match id_tree.get(id) {
            Some(r) => {
                *unsafe {self.dirty_mark_list.get_unchecked_mut(id)} = true;
                self.dirty.mark(id, r.layer)
            },
            _ => ()
        };
    }

    fn recursive_delete_dirty(&mut self, id: usize, node: &IdTreeNode, id_tree: &SingleCaseImpl<IdTree>){
        if unsafe {self.dirty_mark_list.remove_unchecked(id)} {
            self.dirty.mark(id, node.layer)
        }

        let first = unsafe { id_tree.get_unchecked(id).children.head };
        for child in id_tree.iter(first) {
            self.recursive_delete_dirty(child.0, &child.1, id_tree);
        }
    }
}

impl<'a> EntityListener<'a, Node, CreateEvent> for WorldMatrixSys{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Transform>, &'a mut MultiCaseImpl<Node, Layout>);
    fn listen(&mut self, event: &CreateEvent, _read: Self::ReadData, write: Self::WriteData){
        write.0.insert(event.id, Transform::default());
        write.1.insert(event.id, Layout::default());
        self.dirty_mark_list.insert(event.id, false);
    }
}

impl<'a> EntityListener<'a, Node, DeleteEvent> for WorldMatrixSys{
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, Transform>, &'a mut MultiCaseImpl<Node, Layout>);
    fn listen(&mut self, event: &DeleteEvent, _read: Self::ReadData, write: Self::WriteData){
        write.0.delete(event.id);
        write.1.delete(event.id);
        unsafe { self.dirty_mark_list.remove_unchecked(event.id) } ;
    }
}

impl<'a> MultiCaseListener<'a, Node, Transform, ModifyEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
        self.marked_dirty(event.id, read);
    }
}

impl<'a> MultiCaseListener<'a, Node, Layout, ModifyEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, _write: Self::WriteData){
        self.marked_dirty(event.id, read);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        self.marked_dirty(event.id, read);
    }
}

impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let node = unsafe { read.get_unchecked(event.id) };
        self.recursive_delete_dirty(event.id, &node, read);
    }
}

impl<'a> Runner<'a> for WorldMatrixSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, WorldMatrix>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        // cal_matrix(&mut self.0.borrow_mut(), component_mgr);
    }
}

// //计算世界矩阵
// pub fn cal_matrix(dirty_marks: &mut LayerDirty, component_mgr: &mut WorldDocMgr){
//     for d1 in dirty_marks.dirtys.iter() {
//         for node_id in d1.iter() {
//             let dirty_mark = unsafe{*dirty_marks.dirty_mark_list.get_unchecked(*node_id)};
//             if dirty_mark == false {
//                 continue;
//             }

//             //修改节点世界矩阵及子节点的世界矩阵
//             modify_matrix(&mut dirty_marks.dirty_mark_list, *node_id, component_mgr);
//         }
//     }
//     dirty_marks.dirtys.clear();
// }

// //取lefttop相对于父节点的变换原点的位置
// #[inline]
// fn get_lefttop_offset(layout: &Layout, parent_origin: &cg::Point2<f32>, parent_layout: &Layout) -> cg::Point2<f32>{
//     cg::Point2::new(
//         layout.left - parent_origin.x + parent_layout.border + parent_layout.padding_left,
//         layout.top - parent_origin.y + parent_layout.border + parent_layout.padding_top
//     )  
// }

// //计算世界矩阵
// fn modify_matrix(dirty_mark_list: &mut VecMap<bool>, node_id: usize, component_mgr: &mut WorldDocMgr) {
//     let world_matrix = {
//         let (transform_id, l, parent) = {
//             let node = component_mgr.node._group.get(node_id);
//             (node.transform, &node.layout, node.parent)
//         };
//         if parent == 0 {
//             if transform_id == 0 {
//                 Matrix4::default().0
//             }else {
//                 component_mgr.node.transform._group.get(transform_id).matrix(l.width, l.height, &cg::Point2::new(l.left, l.top))
//             }
//         }else {
//             let parent_node = component_mgr.node._group.get(parent);
//             let parent_world_matrix = &component_mgr.node.world_matrix._group.get(parent_node.world_matrix).owner.0;  
//             let parent_transform_origin = {
//                 if parent_node.transform == 0 {
//                     cg::Point2::new(0.0, 0.0)
//                 }else {
//                     component_mgr.node.transform._group.get(parent_node.transform).origin.to_value(parent_node.layout.width, parent_node.layout.height)
//                 }
//             };
//             let offset = get_lefttop_offset(l, &parent_transform_origin, &parent_node.layout);
//             if transform_id == 0 {
//                 parent_world_matrix * cg::Matrix4::from_translation(cg::Vector3::new(offset.x, offset.y, 0.0))
//             }else {
//                 let transform_matrix = component_mgr.node.transform._group.get(transform_id).matrix(l.width, l.height, &offset);
//                 parent_world_matrix * transform_matrix
//             }
//         }

//         // if parent_id != 0 {
//         //     let parent_world_matrix = {
//         //         let parent_world_matrix_id = component_mgr.node._group.get(parent_id).world_matrix;
//         //         ***component_mgr.node.world_matrix._group.get(parent_world_matrix_id)
//         //     };
//         //     world_matrix = parent_world_matrix * world_matrix;
//         // }
        
//         // let center = if parent > 0 {
//         //     //parent_layout
//         //     let pl = &component_mgr.node._group.get(parent).layout;
//         //     cg::Vector3::new(
//         //         l.width/2.0 + l.left - pl.width/2.0,
//         //         l.height/2.0 + l.top - pl.height/2.0,
//         //         0.0,
//         //     )
//         // }else {
//         //     cg::Vector3::new(l.width/2.0, l.height/2.0, 0.0)
//         // };
        
//         // // let mut matrix = cg::Matrix4::from_translation(center.clone()); // center_matrix
//         // if transform_id != 0 {
//         //     matrix = matrix * component_mgr.node.transform._group.get(transform_id).matrix(cg::Vector4::new(l.width, l.height, 0.0, 0.0));
//         // }
//         // (matrix, parent)
//         // let transform = match transform_id == 0 {
//         //     true => Transform::default().matrix(), // 优化？ 默认的matrix可以从全局取到 TODO
//         //     false => component_mgr.node.transform._group.get(transform_id).matrix(),
//         // };

        
//         // (center_matrix * transform, parent)
//     };

//     let mut child = {
//         let mut node_ref = component_mgr.get_node_mut(node_id);
//         node_ref.get_world_matrix_mut().modify(|matrix: &mut Matrix4|{
//             matrix.x = world_matrix.x;
//             matrix.y = world_matrix.y;
//             matrix.z = world_matrix.z;
//             matrix.w = world_matrix.w;
//             true
//         });

//         node_ref.get_childs_mut().get_first()
//     };
//     unsafe{*dirty_mark_list.get_unchecked_mut(node_id) = false}
//     //递归计算子节点的世界矩阵
//     loop {
//         if child == 0 {
//             return;
//         }
//         let node_id = {
//             let v = unsafe{ component_mgr.node_container.get_unchecked(child) };
//             child = v.next;
//             v.elem.clone()
//         };
//         modify_matrix(dirty_mark_list, node_id, component_mgr);
//     }
// }

// // #[cfg(test)]
// // #[cfg(not(feature = "web"))]
// // mod test{
// //     use std::rc::Rc;

// //     use wcs::component::Builder;
// //     use wcs::world::{World, System};

// //     use world_doc::WorldDocMgr;
// //     use world_doc::component::node::{NodeBuilder, InsertType};
// //     use world_doc::component::style::transform::Transform;
// //     use component::math::{Vector3};
// //     use world_doc::system::world_matrix::WorldMatrix;


// //     #[test]
// //     fn test(){
// //         let mut world = new_world();
// //         let node2 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node3 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node4 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node5 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node6 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node7 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node8 = NodeBuilder::new().build(&mut world.component_mgr.node);
// //         let node9 = NodeBuilder::new().build(&mut world.component_mgr.node);

// //         world.component_mgr.set_size(500.0, 500.0);
// //         let (root, node_ids) = {
// //             let root = NodeBuilder::new().build(&mut world.component_mgr.node);
// //             let root_id = world.component_mgr.add_node(root).id;
// //             let mgr = &mut world.component_mgr;
            
// //             //root的直接子节点
// //             let node2 = mgr.get_node_mut(root_id).insert_child(node2, InsertType::Back).id;
// //             let node3 = mgr.get_node_mut(root_id).insert_child(node3, InsertType::Back).id;

// //             //node2的直接子节点
// //             let node4 = mgr.get_node_mut(node2).insert_child(node4, InsertType::Back).id;
// //             let node5 = mgr.get_node_mut(node2).insert_child(node5, InsertType::Back).id;

// //             //node3的直接子节点
// //             let node6 = mgr.get_node_mut(node3).insert_child(node6, InsertType::Back).id;
// //             let node7 = mgr.get_node_mut(node3).insert_child(node7, InsertType::Back).id;

// //             //node4的直接子节点
// //             let node8 = mgr.get_node_mut(node4).insert_child(node8, InsertType::Back).id;
// //             let node9 = mgr.get_node_mut(node4).insert_child(node9, InsertType::Back).id;

// //             (
// //                 root_id,
// //                 vec![node2, node3, node4, node5, node6, node7, node8, node9]
// //             )
// //         };

// //         //  mgr.get_node_mut(root).
// //         world.run(());
// //         for i in node_ids.iter(){
// //             {
// //                 let world_matrix_id = world.component_mgr.node._group.get(*i).world_matrix;
// //                 let world_matrix = world.component_mgr.node.world_matrix._group.get(world_matrix_id);
// //                 // println!("test_world_matrix1, node{} , world_matrix:{:?}", i, world_matrix);
// //             }
// //         }
// //         {
// //             world.component_mgr.get_node(root);
// //             let transform_id = *(world.component_mgr.get_node(root).get_transform());
// //             if transform_id == 0 {
// //                 let mut transform = Transform::default();
// //                 transform.position = Vector3(cg::Vector3::new(1.0, 2.0, 3.0));
// //                 world.component_mgr.get_node_mut(root).set_transform(transform);
// //             }else {
// //                 world.component_mgr.get_node_mut(root).get_transform_mut().modify(|t: &mut Transform| {
// //                     t.position = Vector3(cg::Vector3::new(1.0, 2.0, 3.0));
// //                     true
// //                 });
// //             }
// //         }
        
// //         world.run(());
// //         // println!("-----------------------------------------------------------------");
// //         for i in node_ids.iter(){
// //             {
// //                 let world_matrix_id = world.component_mgr.node._group.get(*i).world_matrix;
// //                 let world_matrix = world.component_mgr.node.world_matrix._group.get(world_matrix_id);
// //                 // println!("test_world_matrix2, node{} , world_matrix:{:?}", i, world_matrix);
// //             }
// //         }

// //         // //修改node2的position
// //         // world.component_mgr.get_node_mut(node_ids[0]).get_position_mut().modify(|t: &mut Vector3| {
// //         //     t.x = 1.0;
// //         //     t.y = 2.0;
// //         //     t.z = 3.0;
// //         //     true
// //         // });
// //         world.run(());
// //         // println!("-----------------------------------------------------------------");
// //         for i in node_ids.iter(){
// //             {
// //                 let world_matrix_id = world.component_mgr.node._group.get(*i).world_matrix;
// //                 let world_matrix = world.component_mgr.node.world_matrix._group.get(world_matrix_id);
// //                 // println!("test_world_matrix3, node{} , world_matrix:{:?}", i, world_matrix);
// //             }
// //         }
// //     }

// //     fn new_world() -> World<WorldDocMgr, ()>{
// //         let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new());
// //         let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr)];
// //         world.set_systems(systems);
// //         world
// //     }
// // }
