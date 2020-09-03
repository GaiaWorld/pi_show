use std::mem::transmute;
use std::time::SystemTime;

use atom::Atom;
use ecs::LendMut;
use ecs::RunTime;
use ecs::{MultiCaseImpl, Runner, SingleCaseImpl, World};
use gui::component::calc::*;
use gui::component::user::*;
use gui::entity::Node;
// use gui::layout::FlexNode;
use gui::world::GuiWorld;
use hal_core::*;
use share::Share;

lazy_static! {
    pub static ref PERFORMANCE_STATISTICIANS: Atom = Atom::from("performance_statisticians_sys");
    pub static ref LAYOUT_PERFORMANCE_STATISTICIANS: Atom =
        Atom::from("layout_performance_statisticians_sys");
}

pub struct PerformanceStatisticians {
    last_time: f32,
    runtime: Share<Vec<RunTime>>,
    // ui: UiNodes,
}

// 统计布局dispatcher时间
pub struct LayoutPerformanceStatisticians {
    runtime: Share<Vec<RunTime>>,
}

impl<'a> Runner<'a> for LayoutPerformanceStatisticians {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<Performance>;
    fn run(&mut self, _: Self::ReadData, performance: Self::WriteData) {
        // 累计运行时间
        for t1 in self.runtime.iter() {
            performance.sum_run_time += t1.cost_time;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Performance {
    run_count: usize,
    avg_cost_time: f32, //单位s
    max_cost_time: f32, //单位s
    sum_run_time: f32,  // 单位s
}

// // Arial
// // pub struct Text

// impl<'a> Runner<'a> for PerformanceStatisticians {
//     type ReadData = ();
//     type WriteData = (
//         &'a mut SingleCaseImpl<Performance>,
//         &'a mut MultiCaseImpl<Node, TextContent>,
//     );
//     fn run(&mut self, _: Self::ReadData, write: Self::WriteData) {
//         let (performance, text_content) = write;
//         if performance.run_count == 0 {
//             performance.max_cost_time = 0.0;
//         }

//         let mut sum = 0.0;
//         // 累计运行时间
//         for t1 in self.runtime.iter() {
//             sum += t1.cost_time.as_secs_f32();
//         }
//         performance.sum_run_time += sum;
//         performance.run_count += 1;
//         if sum > performance.max_cost_time {
//             performance.max_cost_time = sum
//         }

//         let time = SystemTime::now();
//         let now = time.elapsed().unwrap().as_secs_f32();
//         let time_diff = now - self.last_time;

//         if time_diff >= 1.0 {
//             self.last_time = now;
//             // let fps = performance.run_count as f32/1000.0*self.sum_run_time;
//             performance.run_count = 0; //
//             performance.avg_cost_time = performance.sum_run_time / performance.run_count as f32; // 计算平均时间
//             performance.sum_run_time = 0.0; // 重置运行时间总和
//             text_content.insert(
//                 self.ui.avg_time,
//                 TextContent(performance.avg_cost_time.to_string(), Atom::from("")),
//             );
//             text_content.insert(
//                 self.ui.run_count,
//                 TextContent(performance.run_count.to_string(), Atom::from("")),
//             );
//             text_content.insert(
//                 self.ui.max_time,
//                 TextContent(performance.max_cost_time.to_string(), Atom::from("")),
//             );
//         }
//     }
// }

// impl PerformanceStatisticians {
//     pub fn new<L: FlexNode, C: HalContext>(world: &mut GuiWorld<L, C>) -> PerformanceStatisticians {
//         let time = SystemTime::now();
//         PerformanceStatisticians {
//             last_time: time.elapsed().unwrap().as_secs_f32(),
//             runtime: world.world.runtime.clone(),
//             ui: creat_ui(world),
//         }
//     }

//     pub fn register_to_world<L: FlexNode, C: HalContext>(
//         world: &mut GuiWorld<L, C>,
//         performance_sys: PerformanceStatisticians,
//     ) {
//         let layout_performance_sys = LayoutPerformanceStatisticians {
//             runtime: world.world.runtime.clone(),
//         };
//         world.world.register_single(Performance::default());
//         world.world.register_system(
//             PERFORMANCE_STATISTICIANS.clone(),
//             CellPerformanceStatisticians::new(performance_sys),
//         );
//         world.world.register_system(
//             LAYOUT_PERFORMANCE_STATISTICIANS.clone(),
//             CellLayoutPerformanceStatisticians::new(layout_performance_sys),
//         );

//         // // 统计render时间， 并渲染Performance检视面板
//         // Arc::make_mut(world.world.get_dispatcher_mut(&gui::world::RENDER_DISPATCH).unwrap()).build("performance_statisticians_sys");
//         // // 统计布局时间
//         // world.world.get_dispatcher_mut(&gui::world::LAYOUT_DISPATCH).unwrap().make_mut().build("layout_performance_statisticians_sys");
//     }

//     pub fn unregister_to_world(world: &mut World) {
//         world.unregister_system(&PERFORMANCE_STATISTICIANS);
//     }
// }

// #[derive(Debug, Clone)]
// pub struct MemoryState {
//     cpu_all: usize,
//     node_capacity: usize,
//     node_max_count: usize,
//     node_count: usize,
//     gpu_all: usize,
//     gpu_details: GpuMemoryDetails,
//     // lru_
// }

// #[derive(Debug, Clone)]
// pub struct GpuMemoryDetails {
//     texture_count: usize,
//     texture_memory: usize,
//     geo_count: usize,
//     buffer_memory: usize,
// }

// pub fn create_node<L: FlexNode, C: HalContext>(gui: &GuiWorld<L, C>) -> usize {
//     let idtree = gui.idtree.lend_mut();
//     let node = gui.node.lend_mut().create();
//     let border_radius = gui.border_radius.lend_mut();
//     border_radius.insert(
//         node,
//         BorderRadius {
//             x: LengthUnit::Pixel(0.0),
//             y: LengthUnit::Pixel(0.0),
//         },
//     );
//     idtree.create(node);
//     node
// }

// pub fn create_text_node<L: FlexNode, C: HalContext>(gui: &GuiWorld<L, C>) -> usize {
//     let node = create_node(gui);
//     gui.text_content
//         .lend_mut()
//         .insert(node, TextContent("".to_string(), Atom::from("")));
//     node
// }

// pub fn append_child<L: FlexNode, C: HalContext>(gui: &GuiWorld<L, C>, child: usize, parent: usize) {
//     let idtree = gui.idtree.lend_mut();
//     let notify = idtree.get_notify();
//     // 如果child在树上， 则会从树上移除节点， 但不会发出事件
//     // idtree.remove(child);
//     idtree.insert_child_with_notify(child, parent, std::usize::MAX, &notify);
// }

// pub fn set_text_rgba_color<L: FlexNode, C: HalContext>(
//     gui: &GuiWorld<L, C>,
//     node_id: usize,
//     r: f32,
//     g: f32,
//     b: f32,
//     a: f32,
// ) {
//     let text_style = gui.text_style.lend_mut();
//     let text_style1 = unsafe { text_style.get_unchecked_mut(node_id) };
//     text_style1.text.color = Color::RGBA(CgColor::new(r, g, b, a));
//     text_style
//         .get_notify_ref()
//         .modify_event(node_id, "color", 0)
// }

// pub fn set_text_content<L: FlexNode, C: HalContext>(
//     gui: &GuiWorld<L, C>,
//     node_id: usize,
//     value: String,
// ) {
//     gui.text_content
//         .lend_mut()
//         .insert(node_id, TextContent(value, Atom::from("")));
// }

// #[macro_use()]
// macro_rules! func_enum {
//     ($func:ident, $ty:ident) => {
//         pub fn $func<L: FlexNode, C: HalContext>(
//             world: &GuiWorld<L, C>,
//             node_id: usize,
//             value: u8,
//         ) {
//             let value = unsafe { transmute(value) };
//             unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
//                 StyleType1::$ty as usize;
//             let yoga = world.yoga.lend_mut();
//             unsafe { yoga.get_unchecked_mut(node_id) }.$func(value);
//             yoga.get_notify_ref().modify_event(node_id, "", 0);
//         }
//     };
// }
// #[macro_use()]
// macro_rules! func_value {
//     ($func:ident, $ty:ident) => {
//         pub fn $func<L: FlexNode, C: HalContext>(
//             world: &GuiWorld<L, C>,
//             node_id: usize,
//             value: f32,
//         ) {
//             unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 |=
//                 StyleType1::$ty as usize;
//             let yoga = world.yoga.lend_mut();
//             unsafe { yoga.get_unchecked_mut(node_id) }.$func(value);
//             yoga.get_notify_ref().modify_event(node_id, "", 0);
//         }
//     };
// }
// // #[macro_use()]
// // macro_rules! func_enum_value {
// //     ($func:ident, $ty:ident) => {
// //         pub fn $func<L: FlexNode, C: HalContext>(world: &GuiWorld<L, C>, node_id: usize, edge: u8, value: f32){
// //             let edge = unsafe{transmute(edge)};
// //             unsafe{world.style_mark.lend_mut().get_unchecked_mut(node_id)}.local_style1 |= StyleType1::$ty as usize;
// //             let yoga = world.yoga.lend_mut();
// // 			unsafe { yoga.get_unchecked_mut(node_id) }.$func(edge, value);
// // 			yoga.get_notify_ref().modify_event(node_id, "", 0);
// //         }
// //     };
// // }

// // func_enum_value!(set_position, Position);
// func_value!(set_width, Width);
// func_value!(set_height, Height);
// // func_value!(set_width_percent, Width);
// func_enum!(set_position_type, PositionType);

// fn creat_ui<L: FlexNode, C: HalContext>(gui: &GuiWorld<L, C>) -> UiNodes {
//     let root = create_node(gui);
//     set_width(gui, root, 200.0);
//     set_height(gui, root, 200.0);
//     set_position_type(gui, root, 1);

//     // avg_name
//     let avg_name = create_node(gui);
//     set_width(gui, avg_name, 100.0);
//     set_height(gui, avg_name, 30.0);
//     append_child(gui, avg_name, root);

//     let avg_name_text = create_text_node(gui);
//     set_text_content(gui, avg_name_text, "avg_time:".to_string());
//     set_text_rgba_color(gui, avg_name_text, 0.0, 1.0, 0.0, 1.0);
//     append_child(gui, avg_name_text, avg_name);

//     let avg_time = create_node(gui);
//     set_width(gui, avg_time, 100.0);
//     set_height(gui, avg_time, 30.0);
//     append_child(gui, avg_time, root);

//     let avg_time_text = create_text_node(gui);
//     set_text_rgba_color(gui, avg_time_text, 0.0, 1.0, 0.0, 1.0);
//     append_child(gui, avg_time_text, avg_time);

//     // max_time
//     let max_name = create_node(gui);
//     set_width(gui, max_name, 100.0);
//     set_height(gui, max_name, 30.0);
//     append_child(gui, max_name, root);

//     let max_name_text = create_text_node(gui);
//     set_text_rgba_color(gui, max_name_text, 0.0, 1.0, 0.0, 1.0);
//     set_text_content(gui, max_name_text, "max_time:".to_string());
//     append_child(gui, max_name_text, max_name);

//     let max_time = create_node(gui);
//     set_width(gui, max_time, 100.0);
//     set_height(gui, max_time, 30.0);
//     append_child(gui, max_time, root);

//     let max_time_text = create_text_node(gui);
//     set_text_rgba_color(gui, max_time_text, 0.0, 1.0, 0.0, 1.0);
//     append_child(gui, max_time_text, max_time);

//     // run_count
//     let count_name = create_node(gui);
//     set_width(gui, count_name, 100.0);
//     set_height(gui, count_name, 30.0);
//     append_child(gui, count_name, root);

//     let count_name_text = create_text_node(gui);
//     set_text_rgba_color(gui, count_name_text, 0.0, 1.0, 0.0, 1.0);
//     set_text_content(gui, count_name_text, "run_count:".to_string());
//     append_child(gui, count_name_text, count_name);

//     let count_time = create_node(gui);
//     set_width(gui, count_time, 100.0);
//     set_height(gui, count_time, 30.0);
//     append_child(gui, count_time, root);

//     let run_count_text = create_text_node(gui);
//     set_text_rgba_color(gui, run_count_text, 0.0, 1.0, 0.0, 1.0);
//     append_child(gui, run_count_text, count_time);

//     append_child(gui, root, 1);

//     UiNodes {
//         avg_time: avg_time_text,
//         max_time: max_time_text,
//         run_count: run_count_text,
//     }
// }

// struct UiNodes {
//     avg_time: usize,
//     max_time: usize,
//     run_count: usize,
// }

// impl_system! {
//     PerformanceStatisticians,
//     true,
//     {
//     }
// }

// impl_system! {
//     LayoutPerformanceStatisticians,
//     true,
//     {
//     }
// }

// #[cfg(test)]
// use ecs::{World, LendMut, SeqDispatcher, Dispatcher};
// #[cfg(test)]
// use atom::Atom;
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

//     unsafe { transforms.get_unchecked_write(e0)}.modify(|transform: &mut Transform|{
//         transform.funcs.push(TransformFunc::TranslateX(50.0));
//         true
//     });
//     world.run(&Atom::from("test_oct_sys"));
//     debug_println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         unsafe{oct.get_unchecked(e0)},
//         unsafe{oct.get_unchecked(e00)},
//         unsafe{oct.get_unchecked(e01)},
//         unsafe{oct.get_unchecked(e02)},
//         unsafe{oct.get_unchecked(e000)},
//         unsafe{oct.get_unchecked(e001)},
//         unsafe{oct.get_unchecked(e002)},
//         unsafe{oct.get_unchecked(e010)},
//         unsafe{oct.get_unchecked(e011)},
//         unsafe{oct.get_unchecked(e012)},
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
