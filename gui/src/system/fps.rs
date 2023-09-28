// //八叉树系统
// use ecs::{CreateEvent, DeleteEvent, EntityListener, SingleCaseImpl, MultiCaseImpl, Runner};
// use ecs::idtree::{ IdTree};

// use std::time::SystemTime;


// #[derive(Default)]
// pub struct PerformanceStatisticians{
// 	last_time: f32,
// 	sum_run_time: f32,
// };

// #[derive(Component, Debug, Clone)]
// pub struct Performance {
// 	fps: u32,
// 	run_count: usize,
// 	cost_time: f32, // 单位ms
// };

// #[derive(Debug, Clone)]
// pub struct MemoryState {
// 	cpu_all: usize,
// 	node_capacity: usize,
// 	node_max_count: usize,
// 	node_count: usize,
// 	gpu_all: usize,
// 	gpu_details: GpuMemoryDetails,
// 	lru_
// };

// #[derive(Debug, Clone)]
// pub struct GpuMemoryDetails {
// 	texture_count: usize,
// 	texture_memory: usize,
// 	geo_count: usize,
// 	buffer_memory: usize,
// };

// impl<'a> Runner<'a> for PerformanceStatisticians {
//     type ReadData = ();
//     type WriteData = &'a mut SingleCaseImpl<Fps>;
//     fn run(&mut self, read: Self::ReadData, fps: Self::WriteData){
// 		// 累计运行时间
// 		for t1 in world.world.runtime.iter(){
// 			t.run_sum_time += t1.cost_time.as_secs_f32() * 1000.0;
// 		}
// 		run_count += 1;

// 		let time = SystemTime::now();
// 		let now = system_time.elapsed().unwrap().as_secs_f32() * 1000.0;
// 		let time_diff = now - self.last_time;
// 		if time_diff > 500.0 ||  {
// 			self.last_time = now;
// 			let fps = self.run_count/1000.0*self.sum_run_time;
// 			if fps > 60.0
// 			fps.fps = ;
// 		}
// 	}
// }

// impl_system!{
//     FpsMeasure,
//     true,
//     {
//     }
// }

// #[cfg(test)]
// use ecs::{World, LendMut, SeqDispatcher, Dispatcher};
// #[cfg(test)]
// use pi_atom::Atom;
// #[cfg(test)]
// use component::user::{TransformWrite, TransformFunc};
// #[cfg(test)]
// use component::calc::{ZDepth};
// #[cfg(test)]
// use system::world_matrix::{WorldMatrixSys, CellWorldMatrixSys};

// #[test]
// fn test(){
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = LendMut::lend_mut(&idtree);
//     let oct = world.fetch_single::<Oct>().unwrap();
//     let oct = LendMut::lend_mut(&oct);
//     let notify = idtree.get_notify();
//     let transforms = world.fetch_multi::<Node, Transform>().unwrap();
//     let transforms = LendMut::lend_mut(&transforms);
//     let layouts = world.fetch_multi::<Node, Layout>().unwrap();
//     let layouts = LendMut::lend_mut(&layouts);
//     let world_matrixs = world.fetch_multi::<Node, WorldMatrix>().unwrap();
//     let _world_matrixs = LendMut::lend_mut(&world_matrixs);
//     let zdepths = world.fetch_multi::<Node, ZDepth>().unwrap();
//     let zdepths = LendMut::lend_mut(&zdepths);

//     let e0 = world.create_entity::<Node>();
    
//     idtree.create(e0);
//     transforms.insert(e0, Transform::default());
//     zdepths.insert(e0, ZDepth::default());
//     layouts.insert(e0, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 900.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e0, 0, 0, Some(&notify)); //根
    
//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     transforms.insert(e00, Transform::default());
//     zdepths.insert(e00, ZDepth::default());
//     layouts.insert(e00, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 300.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e00, e0, 1, Some(&notify));

//     idtree.create(e01);
//     layouts.insert(e01, Layout{
//         left: 300.0,
//         top: 0.0,
//         width: 300.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     transforms.insert(e01, Transform::default());
//     zdepths.insert(e01, ZDepth::default());
//     idtree.insert_child(e01, e0, 2, Some(&notify));

//     idtree.create(e02);
//     transforms.insert(e02, Transform::default());
//     zdepths.insert(e02, ZDepth::default());
//     layouts.insert(e02, Layout{
//         left: 600.0,
//         top: 0.0,
//         width: 300.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e02, e0, 3, Some(&notify));

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     layouts.insert(e000, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     transforms.insert(e000, Transform::default());
//     zdepths.insert(e000, ZDepth::default());
//     idtree.insert_child(e000, e00, 1, Some(&notify));

//     idtree.create(e001);
//     transforms.insert(e001, Transform::default());
//     zdepths.insert(e001, ZDepth::default());
//     layouts.insert(e001, Layout{
//         left: 100.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e001, e00, 2, Some(&notify));

//     idtree.create(e002);
//     transforms.insert(e002, Transform::default());
//     zdepths.insert(e002, ZDepth::default());
//     layouts.insert(e002, Layout{
//         left: 200.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e002, e00, 3, Some(&notify));

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     layouts.insert(e010, Layout{
//         left: 0.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     transforms.insert(e010, Transform::default());
//     zdepths.insert(e010, ZDepth::default());
//     idtree.insert_child(e010, e01, 1, Some(&notify));

//     idtree.create(e011);
//     transforms.insert(e011, Transform::default());
//     zdepths.insert(e011, ZDepth::default());
//     layouts.insert(e011, Layout{
//         left: 100.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e011, e01, 2, Some(&notify));

//     idtree.create(e012);
//     transforms.insert(e012, Transform::default());
//     zdepths.insert(e012, ZDepth::default());
//     layouts.insert(e012, Layout{
//         left: 200.0,
//         top: 0.0,
//         width: 100.0,
//         height: 900.0,
//         border_left: 0.0,
//         border_top: 0.0,
//         border_right: 0.0,
//         border_bottom: 0.0,
//         padding_left: 0.0,
//         padding_top: 0.0,
//         padding_right: 0.0,
//         padding_bottom: 0.0,
//     });
//     idtree.insert_child(e012, e01, 3, Some(&notify));
    
//     transforms.get_write(e0).unwrap().modify(|transform: &mut Transform|{
//         transform.funcs.push(TransformFunc::TranslateX(50.0));
//         true
//     });
//     world.run(&Atom::from("test_oct_sys"));
//     debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         oct[e0],
//         oct[e00],
//         oct[e01],
//         oct[e02],
//         oct[e000],
//         oct[e001],
//         oct[e002],
//         oct[e010],
//         oct[e011],
//         oct[e012],
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, Layout>();
//     world.register_multi::<Node, Transform>();
//     world.register_multi::<Node, ZDepth>();
//     world.register_multi::<Node, WorldMatrix>();
//     world.register_single::<IdTree>(IdTree::default());
//     world.register_single::<Oct>(Oct::new());
     
//     let system = CellOctSys::new(OctSys::default());
//     world.register_system(Atom::from("oct_system"), system);
//     let system = CellWorldMatrixSys::new(WorldMatrixSys::default());
//     world.register_system(Atom::from("world_matrix_system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("oct_system, world_matrix_system".to_string(), &world);

//     world.add_dispatcher( Atom::from("test_oct_sys"), dispatch);
//     world
// }

