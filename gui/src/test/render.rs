use stdweb::web::html_element::CanvasElement;
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::web::{
    IParentNode,
    document,
};
use stdweb::unstable::TryInto;

use wcs::component::{Builder};
use cg::color::{Color as CgColor};

// use layout::{YGDirection, YgNode};

use world_doc::{create_world };
use world_doc::component::node::{NodeBuilder, InsertType};
use world_doc::component::style::generic::{DecorateBuilder };
// use world_doc::component::style::flex::{LayoutBuilder, Rect as WH};
// use world_doc::component::style::generic::{StyleUnit};
use component::math::{Color as MathColor};
use component::color::Color;

pub fn test(){
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    let mut world = create_world(gl);
    world.component_mgr.set_size(1000.0, 1000.0);

    let node2 = NodeBuilder::new()
    .decorate(
        DecorateBuilder::new()
        .background_color(Color::RGBA(MathColor(CgColor::new(0.0, 1.0, 1.0, 1.0))))
        .build(&mut world.component_mgr.node.decorate))
    .build(&mut world.component_mgr.node);
    node2.yoga.set_width(100.0);
    node2.yoga.set_height(100.0);

    world.component_mgr.set_size(1000.0, 1000.0);
    js!{console.log("cccccccccccccccccc")}
    world.component_mgr.get_root_mut().insert_child(node2, InsertType::Back);
    js!{console.log("zzzzzzzzzzzzzzzzzzzzz")}

    world.run(());

}