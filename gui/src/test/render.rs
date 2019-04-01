use std::rc::Rc;
use std::os::raw::{c_void};

use stdweb::web::html_element::CanvasElement;
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::web::{
    IParentNode,
    document,
};
use stdweb::unstable::TryInto;

use wcs::world::{World, System};
use wcs::component::{Builder};
use cg::{Aabb3, Point3};
use cg::color::{Color as CgColor};

// use layout::{YGDirection, YgNode};
use system::{world_matrix::WorldMatrix, oct::Oct};

use system::{layout::Layout as LayoutSys, rect, create_program, render};
use world::GuiComponentMgr;
use component::node::{NodeBuilder, InsertType, YogaContex};
use component::style::element::{ElementBuilder, RectBuilder};
use component::style::color::{Color};
// use component::style::flex::{LayoutBuilder, Rect as WH};
// use component::style::generic::{StyleUnit};
use component::math::{Color as MathColor};


pub fn test(){
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    let mut world: World<GuiComponentMgr, ()> = World::new(GuiComponentMgr::new(gl));

    let _rect_set = rect::RectSet::init(&mut world.component_mgr);
    let _radius_set = rect::RadiusSet::init(&mut world.component_mgr);
    let _color_set = rect::ColorSet::init(&mut world.component_mgr);
    let _border_set = rect::BorderColorSet::init(&mut world.component_mgr);

    let layout_sys = LayoutSys::init(&mut world.component_mgr);
    let world_matrix_sys = WorldMatrix::init(&mut world.component_mgr);
    let oct_sys = Oct::init(&mut world.component_mgr);
    let create_program_sys = create_program::CreateProgram::init(&mut world.component_mgr);
    let render_sys = render::Render::init(&mut world.component_mgr);

    let systems: Vec<Rc<System<(), GuiComponentMgr>>> = vec![layout_sys, world_matrix_sys, oct_sys, create_program_sys, render_sys];
    world.set_systems(systems);

    let root = NodeBuilder::new()
        .build(&mut world.component_mgr.node);

    let root_yoga = root.yoga;

    world.component_mgr.root_id = world.component_mgr.add_node(root).id;
    let yoga_context = Box::into_raw(Box::new(YogaContex {
        node_id: world.component_mgr.root_id,
        mgr: &world.component_mgr as *const GuiComponentMgr as usize,
    })) as usize;
    root_yoga.set_context(yoga_context as *mut c_void);

    let node2 = NodeBuilder::new()
    .element(
        ElementBuilder::new()
        .rect(
            RectBuilder::new()
            .color(Color::RGBA(MathColor(CgColor::new(0.0, 0.0, 1.0, 1.0))))
            .build(&mut world.component_mgr.node.element.rect))
        .build(&mut world.component_mgr.node.element))
    .build(&mut world.component_mgr.node);
    node2.yoga.set_width(100.0);
    node2.yoga.set_height(100.0);

    world.component_mgr.get_root_mut().insert_child(node2, InsertType::Back);
    world.component_mgr.set_size(1000.0, 1000.0);

    world.run(());

}