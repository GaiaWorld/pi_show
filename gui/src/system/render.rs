// use std::cell::RefCell;
// use std::rc::{Rc};

// use wcs::world::{System};
// use wcs::component::{ComponentHandler, Event};

// use component::style::transform::{Transform};
// use world::GuiComponentMgr;
// use component::math::{Matrix4, Vector3};
// // use alert;

// pub struct Renderer(RefCell<RendererImpl>);

// impl WorldMatrix {
//     pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<WorldMatrix>{
//         let system = Rc::new(WorldMatrix(RefCell::new(WorldMatrixImpl::new())));
//         component_mgr.node.transform._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Transform, GuiComponentMgr>>)));
//         component_mgr.node.position._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Vector3, GuiComponentMgr>>)));
//         system
//     }
// }

// impl ComponentHandler<Transform, GuiComponentMgr> for WorldMatrix{
//     fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
//         match event {
//             Event::ModifyField{id: _, parent, field: _} => {
//                 self.0.borrow_mut().marked_dirty(*parent, component_mgr);
//             },
//             Event::Create{id: _, parent} => {
//                 self.0.borrow_mut().marked_dirty(*parent, component_mgr);
//             },
//             Event::Delete{id, parent: _} => {
//                 self.0.borrow_mut().delete_dirty(*id, component_mgr);
//             },
//             _ => ()
//         }
//     }
// }

// impl ComponentHandler<Vector3, GuiComponentMgr> for WorldMatrix{
//     fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){

//         match event {
//             Event::ModifyField{id: _, parent, field: _} => {
//                 self.0.borrow_mut().marked_dirty(*parent, component_mgr);
//             },
//             _ => ()
//         }
//     }
// }

// impl System<(), GuiComponentMgr> for WorldMatrix{
//     fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
//         self.0.borrow_mut().cal_matrix(component_mgr);
//     }
// }

// pub struct WorldMatrixImpl {
//     dirtys: Vec<Vec<usize>>, //Vec<Vec<node_id>>
// }

// impl WorldMatrixImpl {
//     pub fn new() -> WorldMatrixImpl{
//         WorldMatrixImpl{
//             dirtys: Vec::new()
//         }
//     }

//     //计算世界矩阵
//     pub fn cal_matrix(&mut self, component_mgr: &mut GuiComponentMgr){
//         for d1 in self.dirtys.iter() {
//             for node_id in d1.iter() {
//                 //修改节点世界矩阵及子节点的世界矩阵
//                 modify_matrix(*node_id, component_mgr);
//             }
//         }

//         //处理完脏列表，需要清空， 此处暂时不清空， TODO
//     }

//     pub fn marked_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
//         let layer = {
//             let node = mgr.node._group.get_mut(node_id);
//             if node.world_matrix_dirty == true {
//                 return;
//             }
//             node.world_matrix_dirty = true;
//             node.layer
//         };

//         if self.dirtys.len() <= layer{
//             for _i in 0..(layer + 1 - self.dirtys.len()){
//                 self.dirtys.push(Vec::new());
//             }
//         }
//         self.dirtys[layer].push(node_id);
//     }

//     pub fn delete_dirty(&mut self, node_id: usize, mgr: &mut GuiComponentMgr){
//         let node = mgr.node._group.get_mut(node_id);
//         if node.world_matrix_dirty == true {
//             let layer = node.layer;
//             for i in 0..self.dirtys[layer].len() {
//                 if self.dirtys[layer][i] == node_id {
//                     self.dirtys[layer].swap_remove(i);
//                     return;
//                 }
//             }
//         }
//     }
// }

// fn modify_matrix(node_id: usize, component_mgr: &mut GuiComponentMgr) {
//     // 设置脏标志
//     {
//         let node = component_mgr.node._group.get_mut(node_id);
//         if node.world_matrix_dirty == false {
//             return;
//         }
//         node.world_matrix_dirty = false;
//     }

//     //计算世界矩阵(应该递归计算并修改子节点的世界矩阵， TODO)

//     let world_matrix = {
//         let (transform_id, position_id) = {
//             let node = component_mgr.node._group.get(node_id);
//             (node.transform, node.position)
//         };
//         let transform = match transform_id == 0 {
//             true => Transform::default().matrix(),
//             false => component_mgr.node.transform._group.get(transform_id).matrix(),
//         };
//         let p = component_mgr.node.position._group.get(position_id).owner.0;
//         let position = cg::Matrix4::from_translation(p.clone());
//         transform * position

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
//         modify_matrix(node_id, component_mgr);
//     }
// }

// // #[cfg(test)]
// // use wcs::world::{World};
// // #[cfg(test)]
// // use component::math_component::{Vector3};

// // #[test]
// // fn test(){
// //     let mut world: World<GuiComponentMgr, ()> = World::new();
// //     let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![WorldMatrix::init(&mut world.component_mgr)];
// //     world.set_systems(systems);
// //     test_world_matrix(&mut world);
// // }

// // #[cfg(test)]
// // fn test_world_matrix(world: &mut World<GuiComponentMgr, ()>){
// //     let (root, node1, node2, node3, node4, node5) = {
// //         let component_mgr = &mut world.component_mgr;
// //         {
            
// //             let (root, node1, node2, node3, node4, node5) = {
// //                 let mut root = component_mgr.create_node(0);
// //                 root.set_layer(1);
// //                 (   
// //                     root.id.0.clone(),
// //                     root.create_child_back().id.0.clone(), 
// //                     root.create_child_back().id.0.clone(),
// //                     root.create_child_back().id.0.clone(),
// //                     root.create_child_back().id.0.clone(),
// //                     root.create_child_back().id.0.clone(),
// //                 )
// //            };
// //             print_node(component_mgr, &node1);
// //             print_node(component_mgr, &node2);
// //             print_node(component_mgr, &node3);
// //             print_node(component_mgr, &node4);
// //             print_node(component_mgr, &node5);

// //             {
// //                 let mut node = component_mgr.get_node_mut(&root);
// //                 let mut size = node.get_size_mut();
// //                 size.set_width(500.0);
// //                 size.set_height(500.0);
// //             }

// //             {
// //                 let mut node = component_mgr.get_node_mut(&node1);
// //                 let mut size = node.get_size_mut();
// //                 size.set_width(100.0);
// //                 size.set_height(100.0);
// //             }

// //             {
// //                 let mut node = component_mgr.get_node_mut(&node2);
// //                 {
// //                     let mut size = node.get_size_mut();
// //                     size.set_width(100.0);
// //                     size.set_height(100.0);
// //                 }
// //                 {
// //                     let mut transform = node.get_transform_mut();
// //                     transform.set_position(Vector3::new(100.0, 0.0, 0.0));
// //                 }
                
// //             }

// //             {
// //                 let mut node = component_mgr.get_node_mut(&node3);
// //                 {
// //                     let mut size = node.get_size_mut();
// //                     size.set_width(100.0);
// //                     size.set_height(100.0);
// //                 }
// //                 {
// //                     let mut transform = node.get_transform_mut();
// //                     transform.set_position(Vector3::new(200.0, 0.0, 0.0));
// //                 }
                
// //             }

// //             {
// //                 let mut node = component_mgr.get_node_mut(&node4);
// //                 {
// //                     let mut size = node.get_size_mut();
// //                     size.set_width(100.0);
// //                     size.set_height(100.0);
// //                 }
// //                 {
// //                     let mut transform = node.get_transform_mut();
// //                     transform.set_position(Vector3::new(100.0, 0.0, 0.0));
// //                     transform.set_position(Vector3::new(400.0, 0.0, 0.0));
// //                 }
                
// //             }

// //             {
// //                 let mut node = component_mgr.get_node_mut(&node5);
// //                 {
// //                     let mut size = node.get_size_mut();
// //                     size.set_width(100.0);
// //                     size.set_height(100.0);
// //                 }
// //                 {
// //                     let mut transform = node.get_transform_mut();
// //                     transform.set_position(Vector3::new(0.0, 100.0, 0.0));
// //                 }
                
// //             }
// //             println!("modify-----------------------------------------");
// //             print_node(component_mgr, &node1);
// //             print_node(component_mgr, &node2);
// //             print_node(component_mgr, &node3);
// //             print_node(component_mgr, &node4);
// //             print_node(component_mgr, &node5);

// //             let node2_qid = component_mgr.get_node_mut(&node2).get_qid().clone();
// //             component_mgr.get_node_mut(&root).remove_child(node2_qid);
// //             (root, node1, node2, node3, node4, node5)
// //         }
// //     };

// //     println!("modify run-----------------------------------------");
// //     world.run(());
// //     print_node(&world.component_mgr, &root);
// //     print_node(&world.component_mgr, &node1);
// //     print_node(&world.component_mgr, &node2);
// //     print_node(&world.component_mgr, &node3);
// //     print_node(&world.component_mgr, &node4);
// //     print_node(&world.component_mgr, &node5);
// // }

// // #[cfg(test)]
// // fn print_node(mgr: &GuiComponentMgr, id: &usize) {
// //     let node = mgr.node._group.get(&id);
// //     let transform = mgr.node.transform._group.get(&node.transform);
// //     let matrix = mgr.node.world_matrix._group.get(&node.world_matrix);

// //     println!("nodeid: {}, transform:{:?}, world_matrix: {:?}, matrix_dirty: {}", id, transform, matrix, node.world_matrix_dirty);
// // }