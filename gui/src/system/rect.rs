use std::rc::{Rc};

use wcs::world::{System};
use wcs::component::{ComponentHandler, ModifyFieldEvent, CreateEvent, DeleteEvent};

use component::style::element::{Rect};
use component::style::color::Color;
use component::render::{SdfDefinesWriteRef, SdfProgram, SdfDefines, Bind};
use world::GuiComponentMgr;

pub struct RectProgramSet;

impl RectProgramSet {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<RectProgramSet>{
        let r = Rc::new(RectProgramSet);
        component_mgr.node.element.rect._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Rect, CreateEvent, GuiComponentMgr>>)));
        component_mgr.node.element.rect._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Rect, DeleteEvent, GuiComponentMgr>>)));
        r
    }
}

impl System<(), GuiComponentMgr> for RectProgramSet{
    fn run(&self, _e: &(), _component_mgr: &mut GuiComponentMgr){
    }
}

//监听Rect的创建， 创建对应的program
impl ComponentHandler<Rect, CreateEvent, GuiComponentMgr> for RectProgramSet{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id, parent} = event;
        //创建SdfProgram组件
        let sdf_program = SdfProgram{
            defines: 0,
            program: 0,
            is_opaque: true,
            bind: Box::new(RectBind(*parent)),
        };
        let program_id = {
            let mut sdf_program_ref = component_mgr.add_sdf_program_with_context(sdf_program, *parent);
            sdf_program_ref.set_defines(SdfDefines::default());
            sdf_program_ref.id
        };
        component_mgr.node.element.rect._group.get_mut(*id).program = program_id;
    }
}

//监听Rect的销毁， 删除对应的program
impl ComponentHandler<Rect, DeleteEvent, GuiComponentMgr> for RectProgramSet{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr){
        let DeleteEvent{id, parent: _} = event;
        let program_id = component_mgr.node.element.rect._group.get(*id).program;
        if program_id > 0 {
            component_mgr.sdf_program._group.remove(program_id);
            component_mgr.sdf_program._group.get_handlers().notify_delete(DeleteEvent{id: program_id, parent: *id}, component_mgr);
            component_mgr.node.element.rect._group.get_mut(*id).program = 0;;
        }
    }
}

// 监听color的变化， 修改COLOR宏
pub struct ColorSet;

impl ColorSet {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<ColorSet>{
        let r = Rc::new(ColorSet);
        component_mgr.node.element.rect.color._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, CreateEvent, GuiComponentMgr>>)));
        component_mgr.node.element.rect.color._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, ModifyFieldEvent, GuiComponentMgr>>)));
        component_mgr.node.element.rect.color._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, DeleteEvent, GuiComponentMgr>>)));
        r
    }
}
 
impl ComponentHandler<Color, CreateEvent, GuiComponentMgr> for ColorSet{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id, parent} = event;
        let program_id = component_mgr.node.element.rect._group.get(*parent).program;
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        modify_color_defines(defines_id, &unsafe{&mut *(component_mgr as *mut GuiComponentMgr)}.node.element.rect.color._group.get(*id).owner, component_mgr);
    }
}

impl ComponentHandler<Color, ModifyFieldEvent, GuiComponentMgr> for ColorSet{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id, parent, field: _} = event;
        let program_id = component_mgr.node.element.rect._group.get(*parent).program;
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        modify_color_defines(defines_id, &unsafe{&mut *(component_mgr as *mut GuiComponentMgr)}.node.element.rect.color._group.get(*id).owner, component_mgr);
    }
}

impl ComponentHandler<Color, DeleteEvent, GuiComponentMgr> for ColorSet{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr){
        let DeleteEvent{id: _, parent} = event;
        let program_id = component_mgr.node.element.rect._group.get(*parent).program;
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        let mut defines_ref = SdfDefinesWriteRef::new(defines_id, component_mgr.sdf_program.defines.to_usize(), component_mgr);
        //修改STROKE宏
        defines_ref.set_color(false);
        defines_ref.set_linear_color_gradient_2(false);
        defines_ref.set_linear_color_gradient_4(false);
        defines_ref.set_ellipse_color_gradient(false);
    }
}

fn modify_color_defines(defines_id: usize, color: &Color, mgr: &mut GuiComponentMgr){
    let mut defines_ref = SdfDefinesWriteRef::new(defines_id, mgr.sdf_program.defines.to_usize(), unsafe{&mut *(mgr as *mut GuiComponentMgr)});
    match color {
        Color::RGB(_) | Color::RGBA(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(false);
        },
        Color::LinearGradient(v) => {
            //修改COLOR宏
            defines_ref.set_color(false);
            if v.list.len() == 2 {
                defines_ref.set_linear_color_gradient_2(true);
                defines_ref.set_linear_color_gradient_4(false);
            }else {
                defines_ref.set_linear_color_gradient_2(false);
                defines_ref.set_linear_color_gradient_4(true);
            }
            defines_ref.set_ellipse_color_gradient(false);
        },
        Color::RadialGradient(_) => {
            //修改COLOR宏
            defines_ref.set_color(true);
            defines_ref.set_linear_color_gradient_2(false);
            defines_ref.set_linear_color_gradient_4(false);
            defines_ref.set_ellipse_color_gradient(true);
        },
    }
}

// 监听border_color的变化， 修改STROKE宏
pub struct BorderColorSet;

impl BorderColorSet {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<BorderColorSet>{
        let r = Rc::new(BorderColorSet);
        component_mgr.node.element.rect.border_color._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, CreateEvent, GuiComponentMgr>>)));
        component_mgr.node.element.rect.border_color._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Color, DeleteEvent, GuiComponentMgr>>)));
        r
    }
}

impl System<(), GuiComponentMgr> for BorderColorSet{
    fn run(&self, _e: &(), _component_mgr: &mut GuiComponentMgr){
    }
}
 
impl ComponentHandler<Color, CreateEvent, GuiComponentMgr> for BorderColorSet{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id: _, parent} = event;
        let program_id = component_mgr.node.element.rect._group.get(*parent).program;
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        let mut defines_ref = SdfDefinesWriteRef::new(defines_id, component_mgr.sdf_program.defines.to_usize(), component_mgr);
        //修改STROKE宏
        defines_ref.set_stroke(true);
    }
}

 
impl ComponentHandler<Color, DeleteEvent, GuiComponentMgr> for BorderColorSet{
    fn handle(&self, event: &DeleteEvent, component_mgr: &mut GuiComponentMgr){
        let DeleteEvent{id: _, parent} = event;
        let program_id = component_mgr.node.element.rect._group.get(*parent).program;
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        let mut defines_ref = SdfDefinesWriteRef::new(defines_id, component_mgr.sdf_program.defines.to_usize(), component_mgr);
        //修改STROKE宏
        defines_ref.set_stroke(false);
    }
}

// 监听radius的变化， 修改SDF_RECT宏
pub struct RadiusSet;

impl RadiusSet {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<RadiusSet>{
        let r = Rc::new(RadiusSet);
        component_mgr.node.element.rect._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Rect, CreateEvent, GuiComponentMgr>>)));
        component_mgr.node.element.rect._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Rect, ModifyFieldEvent, GuiComponentMgr>>)));
        r
    }
}

impl System<(), GuiComponentMgr> for RadiusSet{
    fn run(&self, _e: &(), _component_mgr: &mut GuiComponentMgr){
    }
}
 
impl ComponentHandler<Rect, CreateEvent, GuiComponentMgr> for RadiusSet{
    fn handle(&self, event: &CreateEvent, component_mgr: &mut GuiComponentMgr){
        let CreateEvent{id, parent: _} = event;
        let (program_id, radius) = {
            let rect = component_mgr.node.element.rect._group.get(*id);
            (rect.program, rect.radius)
        };
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        let mut defines_ref = SdfDefinesWriteRef::new(defines_id, component_mgr.sdf_program.defines.to_usize(), component_mgr);
        if radius == 0.0 {
            //修改SDF_RECT宏
            defines_ref.set_sdf_rect(false);
        }else {
            defines_ref.set_sdf_rect(true);
        }
    }
}

 
impl ComponentHandler<Rect, ModifyFieldEvent, GuiComponentMgr> for RadiusSet{
    fn handle(&self, event: &ModifyFieldEvent, component_mgr: &mut GuiComponentMgr){
        let ModifyFieldEvent{id, parent: _, field: _} = event;
        let (program_id, radius) = {
            let rect = component_mgr.node.element.rect._group.get(*id);
            (rect.program, rect.radius)
        };
        let defines_id = component_mgr.sdf_program._group.get(program_id).defines;
        let mut defines_ref = SdfDefinesWriteRef::new(defines_id, component_mgr.sdf_program.defines.to_usize(), component_mgr);
        if radius == 0.0 {
            //修改SDF_RECT宏
            defines_ref.set_sdf_rect(false);
        }else {
            defines_ref.set_sdf_rect(true);
        }
    }
}

// usize为node_id
pub struct RectBind(usize);

impl Bind for RectBind {
    // context 是一个裸指针
    unsafe fn bind(&self, context: usize){
        let _mgr = &mut *(context as *mut GuiComponentMgr);
        //let node
        //绑定uniform, buffer等
    }
}