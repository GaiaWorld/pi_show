use std::rc::Rc;

use stdweb::UnsafeTypedArray;
use webgl_rendering_context::{WebGLBuffer, WebGLRenderingContext};

use atom::Atom;
use wcs::component::{ComponentHandler, CreateEvent, DeleteEvent, ModifyFieldEvent};
use wcs::world::System;

use generic_component::math::{Color as MathColor, Aabb3 as MathAabb3};
use document::component::render::{Bind, Defines, DefinesId, RenderObj, SdfDefines, SdfDefinesWriteRef};
use generic_component::color::Color;
use document::component::style::element::{Rect, ElementId, RectWriteRef};
use document::component::node::{Node};
use document::DocumentMgr;
use render::engine::get_uniform_location;

pub struct RectChange {
    _rect_set:  Rc<RectSet>,
    _color_set: Rc<ColorSet>,
    _border_color_set : Rc<BorderColorSet>,
    _shape_dirty: Rc<ShapeDirty>,
    _radius_set: Rc<RadiusSet>,
}

impl RectChange{
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<RectChange> {
        let r = RectChange{
            _rect_set: RectSet::init(component_mgr),
            _color_set: ColorSet::init(component_mgr),
            _border_color_set: BorderColorSet::init(component_mgr),
            _shape_dirty: ShapeDirty::init(component_mgr),
            _radius_set: RadiusSet::init(component_mgr),
        };
        let r = Rc::new(r);
        r
    }
}

impl System<(), DocumentMgr> for RectChange {
    fn run(&self, _e: &(), _component_mgr: &mut DocumentMgr) {}
}

// 监听MathAabb3， z_depth的改变， 将rect及其阴影的shape_dirty设为true， 渲染时如发现该值为true，应该更新gl中的attribut: position, 并将该值设回false
pub struct ShapeDirty;

impl ShapeDirty {
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<ShapeDirty> {
        let r = Rc::new(ShapeDirty);
        component_mgr.node.z_depth.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, DocumentMgr>>),));
        component_mgr.node.bound_box._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<MathAabb3, ModifyFieldEvent, DocumentMgr>>),));
        r
    }
}

impl ComponentHandler<MathAabb3, ModifyFieldEvent, DocumentMgr> for ShapeDirty {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut DocumentMgr) {
        let ModifyFieldEvent { id: _, parent, field: _ } = event;
        mark_rect_shape_dirty(*parent, component_mgr);
    }
}

impl ComponentHandler<Node, ModifyFieldEvent, DocumentMgr> for ShapeDirty {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut DocumentMgr) {
        let ModifyFieldEvent { id, parent: _, field: _ } = event;
        mark_rect_shape_dirty(*id, component_mgr);
    }
}

//param(id: node_id)
fn mark_rect_shape_dirty(id: usize, mgr: &mut DocumentMgr){
    let element = mgr.node._group.get(id).element.clone();
    match element {
        ElementId::Rect(id) => {
            let mut rect_ref = RectWriteRef::new(id, mgr.node.element.rect.to_usize(), mgr);
            rect_ref.set_shape_dirty(true);
            let mut shadow_ref = rect_ref.get_shadow_mut();
            match shadow_ref.id > 0 {
                true => shadow_ref.set_shape_dirty(true),
                false => (),
            };
        },
        _ => (),
    }
}
// z_depth

//监听rect的创建和销毁事件， 创建或删除对应的RenderObj
pub struct RectSet;

impl RectSet {
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<RectSet> {
        let r = Rc::new(RectSet);
        component_mgr
            .node
            .element
            .rect
            ._group
            .register_create_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Rect, CreateEvent, DocumentMgr>>),
            ));
        component_mgr
            .node
            .element
            .rect
            ._group
            .register_delete_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Rect, DeleteEvent, DocumentMgr>>),
            ));
        r
    }
}

impl System<(), DocumentMgr> for RectSet {
    fn run(&self, _e: &(), _component_mgr: &mut DocumentMgr) {}
}

//监听Rect的创建， 创建对应的program
impl ComponentHandler<Rect, CreateEvent, DocumentMgr> for RectSet {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut DocumentMgr) {
        let CreateEvent { id, parent } = event;
        //创建SdfProgram组件
        let render_obj = RenderObj {
            defines: DefinesId::None,
            program: 0,
            is_opaque: true, //TODO 根据节点的opaque值进行设置
            z_index: 0.0,    //TODO 根据节点的z_index值进行设置
            bind: Box::new(RectBind::new(*id, component_mgr)),
        };
        let render_obj_id = {
            let mut render_obj_ref = component_mgr.add_render_obj_with_context(render_obj, *parent);
            render_obj_ref.set_defines(Defines::Sdf(SdfDefines::default()));
            render_obj_ref.id
        };
        component_mgr
            .node
            .element
            .rect
            ._group
            .get_mut(*id)
            .render_obj = render_obj_id;
    }
}

//监听Rect的销毁， 删除对应的program
impl ComponentHandler<Rect, DeleteEvent, DocumentMgr> for RectSet {
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut DocumentMgr) {
        let DeleteEvent { id, parent: _ } = event;
        let render_obj_id = component_mgr.node.element.rect._group.get(*id).render_obj;
        if render_obj_id > 0 {
            component_mgr.render_obj._group.remove(render_obj_id);
            component_mgr
                .render_obj
                ._group
                .get_handlers()
                .notify_delete(
                    DeleteEvent {
                        id: render_obj_id,
                        parent: *id,
                    },
                    component_mgr,
                );
            component_mgr
                .node
                .element
                .rect
                ._group
                .get_mut(*id)
                .render_obj = 0;;
        }
    }
}

// 监听color的变化， 修改COLOR宏
pub struct ColorSet;

impl ColorSet {
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<ColorSet> {
        let r = Rc::new(ColorSet);
        component_mgr
            .node
            .element
            .rect
            .color
            ._group
            .register_create_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Color, CreateEvent, DocumentMgr>>),
            ));
        component_mgr
            .node
            .element
            .rect
            .color
            ._group
            .register_modify_field_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Color, ModifyFieldEvent, DocumentMgr>>),
            ));
        component_mgr
            .node
            .element
            .rect
            .color
            ._group
            .register_delete_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Color, DeleteEvent, DocumentMgr>>),
            ));
        r
    }
}

impl ComponentHandler<Color, CreateEvent, DocumentMgr> for ColorSet {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut DocumentMgr) {
        let CreateEvent { id, parent } = event;
        let render_obj_id = component_mgr
            .node
            .element
            .rect
            ._group
            .get(*parent)
            .render_obj;
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            modify_color_defines(
                defines_id,
                &unsafe { &mut *(component_mgr as *mut DocumentMgr) }
                    .node
                    .element
                    .rect
                    .color
                    ._group
                    .get(*id)
                    .owner,
                component_mgr,
            );
        }
    }
}

impl ComponentHandler<Color, ModifyFieldEvent, DocumentMgr> for ColorSet {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut DocumentMgr) {
        let ModifyFieldEvent {
            id,
            parent,
            field: _,
        } = event;
        let render_obj_id = component_mgr
            .node
            .element
            .rect
            ._group
            .get(*parent)
            .render_obj;
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            modify_color_defines(
                defines_id,
                &unsafe { &mut *(component_mgr as *mut DocumentMgr) }
                    .node
                    .element
                    .rect
                    .color
                    ._group
                    .get(*id)
                    .owner,
                component_mgr,
            );
        }
    }
}

impl ComponentHandler<Color, DeleteEvent, DocumentMgr> for ColorSet {
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut DocumentMgr) {
        let DeleteEvent { id: _, parent } = event;
        let render_obj_id = component_mgr
            .node
            .element
            .rect
            ._group
            .get(*parent)
            .render_obj;
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            let mut defines_ref = SdfDefinesWriteRef::new(
                defines_id,
                component_mgr.render_obj.defines.to_usize(),
                component_mgr,
            );
            //修改STROKE宏
            defines_ref.set_color(false);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(false);
        }
    }
}

impl System<(), DocumentMgr> for ColorSet {
    fn run(&self, _e: &(), _component_mgr: &mut DocumentMgr) {}
}

fn modify_color_defines(defines_id: usize, color: &Color, mgr: &mut DocumentMgr) {
    let mut defines_ref =
        SdfDefinesWriteRef::new(defines_id, mgr.render_obj.defines.to_usize(), unsafe {
            &mut *(mgr as *mut DocumentMgr)
        });
    match color {
        Color::RGB(_) | Color::RGBA(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(false);
        }
        Color::LinearGradient(v) => {
            //修改COLOR宏
            defines_ref.set_color(false);
            if v.list.len() == 2 {
                defines_ref.set_linear_color_gradient_2(true);
                defines_ref.set_linear_color_gradient_4(false);
            } else {
                defines_ref.set_linear_color_gradient_2(false);
                defines_ref.set_linear_color_gradient_4(true);
            }
            defines_ref.set_ellipse_color_gradient(false);
        }
        Color::RadialGradient(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(true);
        }
    }
}

// 监听border_color的变化， 修改STROKE宏
pub struct BorderColorSet;

impl BorderColorSet {
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<BorderColorSet> {
        let r = Rc::new(BorderColorSet);
        component_mgr
            .node
            .element
            .rect
            .border_color
            ._group
            .register_create_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Color, CreateEvent, DocumentMgr>>),
            ));
        component_mgr
            .node
            .element
            .rect
            .border_color
            ._group
            .register_delete_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Color, DeleteEvent, DocumentMgr>>),
            ));
        r
    }
}

impl System<(), DocumentMgr> for BorderColorSet {
    fn run(&self, _e: &(), _component_mgr: &mut DocumentMgr) {}
}

impl ComponentHandler<Color, CreateEvent, DocumentMgr> for BorderColorSet {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut DocumentMgr) {
        let CreateEvent { id: _, parent } = event;
        let render_obj_id = component_mgr
            .node
            .element
            .rect
            ._group
            .get(*parent)
            .render_obj;
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            let mut defines_ref = SdfDefinesWriteRef::new(
                defines_id,
                component_mgr.render_obj.defines.to_usize(),
                component_mgr,
            );
            //修改STROKE宏
            defines_ref.set_stroke(true);
        }
    }
}

impl ComponentHandler<Color, DeleteEvent, DocumentMgr> for BorderColorSet {
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut DocumentMgr) {
        let DeleteEvent { id: _, parent } = event;
        let render_obj_id = component_mgr
            .node
            .element
            .rect
            ._group
            .get(*parent)
            .render_obj;
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            let mut defines_ref = SdfDefinesWriteRef::new(
                defines_id,
                component_mgr.render_obj.defines.to_usize(),
                component_mgr,
            );
            //修改STROKE宏
            defines_ref.set_stroke(false);
        }
    }
}

// 监听radius的变化， 修改SDF_RECT宏
pub struct RadiusSet;

impl RadiusSet {
    pub fn init(component_mgr: &mut DocumentMgr) -> Rc<RadiusSet> {
        let r = Rc::new(RadiusSet);
        component_mgr
            .node
            .element
            .rect
            ._group
            .register_create_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Rect, CreateEvent, DocumentMgr>>),
            ));
        component_mgr
            .node
            .element
            .rect
            ._group
            .register_modify_field_handler(Rc::downgrade(
                &(r.clone() as Rc<ComponentHandler<Rect, ModifyFieldEvent, DocumentMgr>>),
            ));
        r
    }
}

impl System<(), DocumentMgr> for RadiusSet {
    fn run(&self, _e: &(), _component_mgr: &mut DocumentMgr) {}
}

impl ComponentHandler<Rect, CreateEvent, DocumentMgr> for RadiusSet {
    fn handle(&self, event: &CreateEvent, component_mgr: &mut DocumentMgr) {
        let CreateEvent { id, parent: _ } = event;
        let (render_obj_id, radius) = {
            let rect = component_mgr.node.element.rect._group.get(*id);
            (rect.render_obj, rect.radius)
        };
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            let mut defines_ref = SdfDefinesWriteRef::new(
                defines_id,
                component_mgr.render_obj.defines.to_usize(),
                component_mgr,
            );
            if radius == 0.0 {
                //修改SDF_RECT宏
                defines_ref.set_sdf_rect(false);
            } else {
                defines_ref.set_sdf_rect(true);
            }
        }
    }
}

impl ComponentHandler<Rect, ModifyFieldEvent, DocumentMgr> for RadiusSet {
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut DocumentMgr) {
        let ModifyFieldEvent {
            id,
            parent: _,
            field: _,
        } = event;
        let (render_obj_id, radius) = {
            let rect = component_mgr.node.element.rect._group.get(*id);
            (rect.render_obj, rect.radius)
        };
        if let DefinesId::Sdf(defines_id) =
            component_mgr.render_obj._group.get(render_obj_id).defines
        {
            let mut defines_ref = SdfDefinesWriteRef::new(
                defines_id,
                component_mgr.render_obj.defines.to_usize(),
                component_mgr,
            );
            if radius == 0.0 {
                //修改SDF_RECT宏
                defines_ref.set_sdf_rect(false);
            } else {
                defines_ref.set_sdf_rect(true);
            }
        }
    }
}

// usize为node_id
pub struct RectBind {
    id: usize,
    indices_buffer: WebGLBuffer,
    positions_buffer: WebGLBuffer,
}

impl RectBind {
    pub fn new(id: usize, mgr: &mut DocumentMgr) -> RectBind {
        let indices_buffer = mgr.engine.gl.create_buffer().unwrap();
        let buffer: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let buffer = unsafe { UnsafeTypedArray::new(&buffer) };
        mgr.engine.gl.bind_buffer(
            WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&indices_buffer),
        );
        js! {
            @{&mgr.engine.gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{&buffer}, @{WebGLRenderingContext::STATIC_DRAW});
            console.log("indeices", @{&buffer});
        }
        RectBind {
            id: id,
            positions_buffer: mgr.engine.gl.create_buffer().unwrap(),
            indices_buffer: indices_buffer,
        }
    }
}

lazy_static! {
    static ref POSITION: Atom = Atom::from("position");
    static ref WORLD_VIEW_PROJECTION: Atom = Atom::from("worldViewProjection");
    static ref CENTER: Atom = Atom::from("center");
    static ref BLUR: Atom = Atom::from("blur");
    static ref EXTEND: Atom = Atom::from("extend");
    static ref ALPHA: Atom = Atom::from("alpha");
    static ref SCREEN_SIZE: Atom = Atom::from("screenSize");
    static ref ANGLE: Atom = Atom::from("angle");
    static ref RADIUS: Atom = Atom::from("radius");
    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref COLOR: Atom = Atom::from("color");
    static ref COLOR_ANGLE: Atom = Atom::from("colorAngle");
    static ref DISTANCE: Atom = Atom::from("distance");
    static ref COLOR1: Atom = Atom::from("color1");
    static ref COLOR2: Atom = Atom::from("color2");
    static ref COLOR3: Atom = Atom::from("color3");
    static ref COLOR4: Atom = Atom::from("color4");
    static ref SIZE_TYPE: Atom = Atom::from("sizeType");
    static ref CLIP_INDEICES: Atom = Atom::from("clipIndices");
    static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
}

impl Bind for RectBind {
    unsafe fn init_locations(&self, context: usize, renderobj_id: usize) {
        let mgr = &mut *(context as *mut DocumentMgr);
        let render_obj = mgr.render_obj._group.get(renderobj_id);
        let defines = match render_obj.defines {
            DefinesId::Sdf(id) => mgr.render_obj.defines.sdf._group.get(id),
            _ => panic!("defines type error!"),
        };
        let gl = mgr.engine.gl.clone();

        let program = mgr.engine.lookup_program_mut(render_obj.program).unwrap();
        let uniform_locations = &mut program.uniform_locations;
        let attr_locations = &mut program.attr_locations;
        let program = &program.program;

        if uniform_locations.len() > 0 {
            return;
        }
        
        let position_location = gl.get_attrib_location(program, &POSITION) as u32;
        attr_locations.insert(
            POSITION.clone(),
            position_location,
        );
        gl.vertex_attrib_pointer(position_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0);

        uniform_locations.insert(
            WORLD_VIEW_PROJECTION.clone(),
            get_uniform_location(&gl,program, &WORLD_VIEW_PROJECTION),
        );
        uniform_locations.insert(
            CENTER.clone(),
            get_uniform_location(&gl,program, &CENTER),
        );
        uniform_locations.insert(
            BLUR.clone(),
            get_uniform_location(&gl,program, &BLUR),
        );
        uniform_locations.insert(
            EXTEND.clone(),
            get_uniform_location(&gl,program, &EXTEND),
        );
        uniform_locations.insert(
            ALPHA.clone(),
            get_uniform_location(&gl,program, &ALPHA),
        );
        uniform_locations.insert(
            SCREEN_SIZE.clone(),
            get_uniform_location(&gl,program, &SCREEN_SIZE),
        );
        uniform_locations.insert(
            ANGLE.clone(),
            get_uniform_location(&gl,program, &ANGLE),
        );

        if defines.sdf_rect {
            uniform_locations.insert(
                RADIUS.clone(),
                get_uniform_location(&gl,program, &RADIUS),
            );
        }
        if defines.stroke {
            uniform_locations.insert(
                STROKE_SIZE.clone(),
                get_uniform_location(&gl,program, &STROKE_SIZE),
            );
            uniform_locations.insert(
                STROKE_COLOR.clone(),
                get_uniform_location(&gl,program, &STROKE_COLOR),
            );
        }
        if defines.clip_plane {
            uniform_locations.insert(
                CLIP_INDEICES.clone(),
                get_uniform_location(&gl,program, &CLIP_INDEICES),
            );

            uniform_locations.insert(
                CLIP_TEXTURE.clone(),
                get_uniform_location(&gl,program, &CLIP_TEXTURE),
            );
            uniform_locations.insert(
                CLIP_INDEICES_SIZE.clone(),
                get_uniform_location(&gl,program, &CLIP_INDEICES_SIZE)
                    ,
            );
        }

        if defines.color {
            uniform_locations.insert(
                COLOR.clone(),
                get_uniform_location(&gl,program, &COLOR),
            );
        } else if defines.linear_color_gradient_2 {
            uniform_locations.insert(
                COLOR_ANGLE.clone(),
                get_uniform_location(&gl,program, &COLOR_ANGLE),
            );
            uniform_locations.insert(
                DISTANCE.clone(),
                get_uniform_location(&gl,program, &DISTANCE),
            );
            uniform_locations.insert(
                COLOR1.clone(),
                get_uniform_location(&gl,program, &COLOR1),
            );
            uniform_locations.insert(
                COLOR2.clone(),
                get_uniform_location(&gl,program, &COLOR2),
            );
        } else if defines.linear_color_gradient_4 {
            uniform_locations.insert(
                COLOR_ANGLE.clone(),
                get_uniform_location(&gl,program, &COLOR_ANGLE),
            );
            uniform_locations.insert(
                DISTANCE.clone(),
                get_uniform_location(&gl,program, &DISTANCE),
            );
            uniform_locations.insert(
                COLOR1.clone(),
                get_uniform_location(&gl,program, &COLOR1),
            );
            uniform_locations.insert(
                COLOR2.clone(),
                get_uniform_location(&gl,program, &COLOR2),
            );
            uniform_locations.insert(
                COLOR3.clone(),
                get_uniform_location(&gl,program, &COLOR3),
            );
            uniform_locations.insert(
                COLOR4.clone(),
                get_uniform_location(&gl,program, &COLOR4),
            );
        } else if defines.ellipse_color_gradient {
            uniform_locations.insert(
                SIZE_TYPE.clone(),
                get_uniform_location(&gl,program, &SIZE_TYPE),
            );
            uniform_locations.insert(
                DISTANCE.clone(),
                get_uniform_location(&gl,program, &DISTANCE),
            );
            uniform_locations.insert(
                COLOR1.clone(),
                get_uniform_location(&gl,program, &COLOR1),
            );
            uniform_locations.insert(
                COLOR2.clone(),
                get_uniform_location(&gl,program, &COLOR2),
            );
            uniform_locations.insert(
                COLOR3.clone(),
                get_uniform_location(&gl,program, &COLOR3),
            );
            uniform_locations.insert(
                COLOR4.clone(),
                get_uniform_location(&gl,program, &COLOR4),
            );
        }
    }
    // context 是一个裸指针
    unsafe fn bind(&self, context: usize, renderobj_id: usize) {
        let mgr = &mut *(context as *mut DocumentMgr);
        let (extent_id, border_id, bound_box_id, z_depth) = {
            let node_id = mgr.node.element.rect._group.get(self.id).parent;
            let node = mgr.node._group.get(node_id);
            (
                node.extent,
                node.border,
                node.bound_box,
                node.z_depth,
            )
        };
        let rect = mgr.node.element.rect._group.get(self.id).owner.clone();
        // let position = mgr.node.position._group.get(position_id);
        let extent = mgr.node.extent._group.get(extent_id);
        let border = mgr.node.border._group.get(border_id);
        let bound_box = mgr.node.bound_box._group.get(bound_box_id);

        let render_obj = mgr.render_obj._group.get(renderobj_id);
        let defines = match render_obj.defines {
            DefinesId::Sdf(id) => mgr.render_obj.defines.sdf._group.get(id),
            _ => panic!("defines type error!"),
        };
        let gl = &mgr.engine.gl;

        let program = mgr.engine.lookup_program(render_obj.program).unwrap();
        let uniform_locations = &program.uniform_locations;
        let attr_locations = &program.attr_locations;
        let program = &program.program;

        // use_program
        gl.use_program(Some(program));
        gl.viewport(0,0,1000,1000);
        

        //设置worldViewProjection
        js! {
            console.log("world_view", @{&(*mgr.world_view)});
        }
        gl.uniform_matrix4fv(
            uniform_locations.get(&WORLD_VIEW_PROJECTION),
            false,
            &(*mgr.world_view),
        );

        //blur
        js! {
            console.log("blur", 1.0);
        }
        // gl.uniform1f(uniform_locations.get(&BLUR), 1.0);

        //extend
        js! {
            console.log("extent", @{extent.width}, @{extent.height});
        }
        gl.uniform2f(uniform_locations.get(&EXTEND), extent.width, extent.height);

        // alpha
        gl.uniform1f(uniform_locations.get(&ALPHA), 1.0);

        // screenSize
        gl.uniform2f(
            uniform_locations.get(&SCREEN_SIZE),
            mgr.root_width,
            mgr.root_height,
        );
        js!{console.log("SCREEN_SIZE", @{mgr.root_width}, @{mgr.root_height})}

        //angle
        js!{console.log("ANGLE", 0)}
        gl.uniform1f(uniform_locations.get(&ANGLE), 0.0); //TODO

        //set_uniforms
        if defines.sdf_rect {
            //设置radius
            gl.uniform1f(uniform_locations.get(&RADIUS), rect.radius);
        }
        if defines.stroke {
            //设置strokeSize
            gl.uniform1f(uniform_locations.get(&STROKE_SIZE), border.value);

            //设置strokeColor
            let border_color = &mgr
                .node
                .element
                .rect
                .border_color
                ._group
                .get(rect.border_color)
                .owner;
            if let Color::RGBA(color) = border_color {
                gl.uniform4f(
                    uniform_locations.get(&STROKE_COLOR),
                    color.r,
                    color.g,
                    color.b,
                    color.a,
                );
            } else {
                panic!("border_color error");
            }
        }
        if defines.clip_plane {
            //TODO
            panic!("ccccccccccccccccccccccccc");
            // uniform float clipIndices;
            // uniform sampler2D clipTexture;
            // uniform float clipTextureSize;

            // arr.push(SDF_CLIP_PLANE.clone());
        }
        let color = &mgr
            .node
            .element
            .rect
            .color
            ._group
            .get(rect.color)
            .owner;
        match color {
            Color::RGB(color) | Color::RGBA(color) => {
                js!{console.log("color", @{color.r}, @{color.g}, @{color.b}, @{color.a})}
                // color
                gl.uniform4f(
                    uniform_locations.get(&COLOR),
                    color.r,
                    color.g,
                    color.b,
                    color.a,
                );
            }
            Color::LinearGradient(color) => {
                //colorAngle
                gl.uniform1f(uniform_locations.get(&COLOR_ANGLE), color.direction);

                if defines.linear_color_gradient_2 {
                    //distance
                    gl.uniform2f(
                        uniform_locations.get(&DISTANCE),
                        color.list[0].position,
                        color.list[1].position,
                    );

                    //color1
                    let color1 = &color.list[0].rgba;
                    gl.uniform4f(
                        uniform_locations.get(&COLOR1),
                        color1.r,
                        color1.g,
                        color1.b,
                        color1.a,
                    );

                    //color2
                    let color2 = &color.list[1].rgba;
                    gl.uniform4f(
                        uniform_locations.get(&COLOR2),
                        color2.r,
                        color2.g,
                        color2.b,
                        color2.a,
                    );
                } else {
                    let mut distances = [0.0, 100.0, 100.0, 100.0];
                    let default_color = MathColor(cg::color::Color::new(1.0, 1.0, 1.0, 1.0));
                    let mut colors = [
                        &default_color,
                        &default_color,
                        &default_color,
                        &default_color,
                    ];
                    let mut i = 0;
                    for k in color.list.iter() {
                        if i > 3 {
                            break;
                        }
                        distances[i] = k.position;
                        colors[i] = &k.rgba;
                        i += 1;
                    }
                    gl.uniform4f(
                        uniform_locations.get(&DISTANCE),
                        distances[0],
                        distances[1],
                        distances[2],
                        distances[3],
                    );

                    //color1
                    gl.uniform4f(
                        uniform_locations.get(&COLOR1),
                        colors[0].r,
                        colors[0].g,
                        colors[0].b,
                        colors[0].a,
                    );

                    //color2
                    gl.uniform4f(
                        uniform_locations.get(&COLOR2),
                        colors[1].r,
                        colors[1].g,
                        colors[1].b,
                        colors[1].a,
                    );

                    //color3
                    gl.uniform4f(
                        uniform_locations.get(&COLOR3),
                        colors[2].r,
                        colors[2].g,
                        colors[2].b,
                        colors[2].a,
                    );

                    //color4
                    gl.uniform4f(
                        uniform_locations.get(&COLOR4),
                        colors[3].r,
                        colors[3].g,
                        colors[3].b,
                        colors[3].a,
                    );
                }
            }
            Color::RadialGradient(_color) => {
                //TODO
                panic!("color type error");
                // uniform float sizeType;
                // uniform vec4 distance;
                // uniform vec4 color1;
                // uniform vec4 color2;
                // uniform vec4 color3;
                // uniform vec4 color4;
            }
        }

        gl.uniform2f(
            uniform_locations.get(&CENTER),
            bound_box.max.x - bound_box.min.x,
            bound_box.max.y - bound_box.min.y,
        );

        //position
        gl.bind_buffer(
            WebGLRenderingContext::ARRAY_BUFFER,
            Some(&self.positions_buffer),
        );
        //如果shape_dirty， 更新定点顶点数据
        if rect.shape_dirty {
            let buffer = [
                bound_box.min.x,
                bound_box.min.y,
                z_depth, // left_top
                bound_box.min.x,
                bound_box.max.y,
                z_depth, // left_bootom
                bound_box.max.x,
                bound_box.max.y,
                z_depth, // right_bootom
                bound_box.max.x,
                bound_box.min.y,
                z_depth, // right_top
            ];

            let buffer = UnsafeTypedArray::new(&buffer);
            js! {
                console.log("position", @{&buffer});
                @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
            }
        }
        
        let position_location = *(attr_locations.get(&POSITION).unwrap()) ;

        gl.vertex_attrib_pointer(
            position_location,
            3,
            WebGLRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(position_location);

        //index
        gl.bind_buffer(
            WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.indices_buffer),
        );

        js! {
            console.log("draw_elements-------------------");
        }
        //draw
        gl.draw_elements(
            WebGLRenderingContext::TRIANGLES,
            6,
            WebGLRenderingContext::UNSIGNED_SHORT,
            0,
        );

        js! {
            console.log("draw_elements-------------------end");
        }

        gl.disable_vertex_attrib_array(position_location);
    }
}
