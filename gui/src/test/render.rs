use std::rc::Rc;

use stdweb::web::html_element::CanvasElement;
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::web::{
    IParentNode,
    world_doc,
};
use stdweb::unstable::TryInto;

use wcs::world::{World, System};
use wcs::component::{Builder};
use cg::color::{Color as CgColor};

// use layout::{YGDirection, YgNode};
use world_doc::system::{world_matrix::WorldMatrix, oct::Oct};

use world_doc::system::{layout::Layout as LayoutSys, rect, create_program, render};
use world_doc::WorldDocMgr;
use world_doc::component::node::{NodeBuilder, InsertType};
use world_doc::component::style::element::{ElementBuilder, RectBuilder};
use world_doc::component::style::color::{Color};
// use world_doc::component::style::flex::{LayoutBuilder, Rect as WH};
// use world_doc::component::style::generic::{StyleUnit};
use component::math::{Color as MathColor};


pub fn test(){
    let canvas: CanvasElement = world_doc().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    let mut world: World<WorldDocMgr, ()> = World::new(WorldDocMgr::new(gl));

    let _rect_set = rect::RectSet::init(&mut world.component_mgr);
    let _radius_set = rect::RadiusSet::init(&mut world.component_mgr);
    let _color_set = rect::ColorSet::init(&mut world.component_mgr);
    let _border_set = rect::BorderColorSet::init(&mut world.component_mgr);

    let layout_sys = LayoutSys::init(&mut world.component_mgr);
    let world_matrix_sys = WorldMatrix::init(&mut world.component_mgr);
    let oct_sys = Oct::init(&mut world.component_mgr);
    let create_program_sys = create_program::CreateProgram::init(&mut world.component_mgr);
    let render_sys = render::Render::init(&mut world.component_mgr);

    let systems: Vec<Rc<System<(), WorldDocMgr>>> = vec![layout_sys, world_matrix_sys, oct_sys, create_program_sys, render_sys];
    world.set_systems(systems);

    let node2 = NodeBuilder::new()
    .element(
        ElementBuilder::new()
        .rect(
            RectBuilder::new()
            .color(Color::RGBA(MathColor(CgColor::new(0.0, 1.0, 1.0, 1.0))))
            .build(&mut world.component_mgr.node.element.rect))
        .build(&mut world.component_mgr.node.element))
    .build(&mut world.component_mgr.node);
    node2.yoga.set_width(100.0);
    node2.yoga.set_height(100.0);

    world.component_mgr.set_size(1000.0, 1000.0);
    js!{console.log("cccccccccccccccccc")}
    world.component_mgr.get_root_mut().insert_child(node2, InsertType::Back);
    js!{console.log("zzzzzzzzzzzzzzzzzzzzz")}

    world.run(());

}