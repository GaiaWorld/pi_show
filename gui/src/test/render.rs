// use std::rc::Rc;

// use stdweb::web::html_element::CanvasElement;
// use stdweb::web::{
//     IParentNode,
//     document,
// };
// use stdweb::unstable::TryInto;

// use wcs::world::{World, System};
// use wcs::component::{Builder};
// use cg::{Aabb3, Point3};
// use cg::color::{Color as CgColor};

// use layout::{YGDirection, YgNode};
// use component::math::{Color as MathColor};
// use system::{layout::Layout as LayoutSys, world_matrix::WorldMatrix, oct::Oct, render::Render, sdf::Sdf};
// use world::GuiComponentMgr;
// use component::node::{NodeBuilder, InsertType};
// use component::style::element::{ElementBuilder, RectBuilder};
// use component::style::color::{Color};
// use component::style::style::{StyleBuilder, Display};
// use component::style::flex::{LayoutBuilder, Rect as WH};
// use component::style::generic::{StyleUnit};

// pub fn test(){
//     let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();

//     let mut world: World<GuiComponentMgr, ()> = World::new();

//     let layout_system = LayoutSys::init(&mut world.component_mgr);
//     let world_matrix_system = WorldMatrix::init(&mut world.component_mgr);
//     let oct_system = Oct::init(&mut world.component_mgr, Aabb3::new(Point3::new(-200.0, -200.0, 0.0), Point3::new(2000.0, -2000.0, 1.0)));
//     let sdf_system = Sdf::init(&mut world.component_mgr);
//     let render_system = Render::init(&mut world.component_mgr, &canvas).unwrap();

//     let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![layout_system, world_matrix_system, oct_system, sdf_system, render_system];
//     world.set_systems(systems);

//     let mut root = NodeBuilder::new()
//     .style(
//         StyleBuilder::new()
//         .display(Some(Display::Flex))
//         .build(&mut world.component_mgr.node.style))
//     .build(&mut world.component_mgr.node);
//     root.yoga = Some(YgNode::create());

//     let node0 = NodeBuilder::new()
//     .element(
//         ElementBuilder::new()
//         .rect(
//             RectBuilder::new()
//             .color(Color::RGBA(MathColor(CgColor::new(0.0, 0.0, 1.0, 1.0))))
//             .build(&mut world.component_mgr.node.element.rect))
//         .build(&mut world.component_mgr.node.element))
//     .style(
//         StyleBuilder::new()
//         .display(Some(Display::Flex))
//         .layout(
//             LayoutBuilder::new()
//             .wh(WH::new(Some(StyleUnit::Length(100.0)), Some(StyleUnit::Length(100.0))))
//             .build(&mut world.component_mgr.node.style.layout))
//         .build(&mut world.component_mgr.node.style))
//     .build(&mut world.component_mgr.node);

//     // let node1 = NodeBuilder::new()
//     // .element(
//     //     ElementBuilder::new()
//     //     .rect(
//     //         RectBuilder::new()
//     //         .color(Color::RGBA(MathColor(CgColor::new(1.0, 0.0, 1.0, 1.0))))
//     //         .build(&mut world.component_mgr.node.element.rect))
//     //     .build(&mut world.component_mgr.node.element))
//     // .style(
//     //     StyleBuilder::new()
//     //     .display(Some(Display::Flex))
//     //     .layout(
//     //         LayoutBuilder::new()
//     //         .wh(WH::new(Some(StyleUnit::Length(100.0)), Some(StyleUnit::Length(100.0))))
//     //         .build(&mut world.component_mgr.node.style.layout))
//     //     .build(&mut world.component_mgr.node.style))
//     // .build(&mut world.component_mgr.node);

//     let root_yoga = (*(root.yoga.as_ref().unwrap())).clone();
//     {
//         let mut root_ref = world.component_mgr.add_node(root);
//         root_ref.insert_child(node0, InsertType::Back);
//         // root_ref.insert_child(node1, InsertType::Back);
//     }

//     root_yoga.calculate_layout(800.0, 800.0, Direction::LTR);
//     world.run(());
// }