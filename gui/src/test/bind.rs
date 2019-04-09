use stdweb::web::html_element::CanvasElement;
use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::web::{
    IParentNode,
    document,
};
use stdweb::unstable::TryInto;

use bind::*;
use bind::node::*;
use bind::layout::*;
use bind::style::*;

pub fn test(){
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let gl: WebGLRenderingContext = canvas.get_context().unwrap();

    js!{
        window.__gl = @{&gl};
    }

    let mut world = create_gui();
    set_gui_size(world: u32, 1000.0, 700.0);

    let node2 = create_node(world);
    set_width(world, node2, 1000.0);
    set_height(world, node2, 700.0);
    set_backgroud_rgba_color(world, node2, 1.0, 0.0, 0.0, 1.0);
    append_child(world, 1, node2);

    let node3 = create_node(world);
    set_width(world, node3, 500.0);
    set_height(world, node3, 500.0);
    set_backgroud_rgba_color(world, node3, 0.0, 0.0, 1.0, 1.0);
    append_child(world, node2, node3);

    let node4 = create_node(world);
    set_width(world, node4, 400.0);
    set_height(world, node4, 400.0);
    set_backgroud_rgba_color(world, node4, 0.0, 1.0, 0.0, 1.0);
    append_child(world, node2, node4);

    run(world);
}