// 处理RectElem的改变 ， 并将其更新到world上的uniform上

use std::cell::RefCell;
use std::rc::{Rc};

use web_sys::*;
use fnv::FnvHashMap;

use wcs::world::{System};
use wcs::component::{ComponentHandler, Event};

use component::component_def::{GuiComponentMgr, SdfStyle, Border, RectElem, Rect, ElementId, CircleElem };
use component::math::{ Aabb3 };
use render::vector_sdf::Index;

pub struct Sdf(RefCell<SdfRenderImpl>);

impl Sdf {
    pub fn init(component_mgr: &mut GuiComponentMgr) -> Rc<Sdf>{
        let system = Rc::new(Sdf(RefCell::new(SdfRenderImpl::new())));
        component_mgr.node.element.rect._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<RectElem, GuiComponentMgr>>)));
        component_mgr.node.element.rect.shape._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Rect, GuiComponentMgr>>)));
        component_mgr.node.bound_box._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Aabb3, GuiComponentMgr>>)));
        component_mgr.node.border._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<Border, GuiComponentMgr>>)));
        component_mgr.node.element.rect.style._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<SdfStyle, GuiComponentMgr>>)));
        component_mgr.node.element.circle.style._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<SdfStyle, GuiComponentMgr>>)));
        component_mgr.node.element.circle._group.register_handler(Rc::downgrade(&(system.clone() as Rc<ComponentHandler<CircleElem, GuiComponentMgr>>)));
        system
    }
}

impl ComponentHandler<RectElem, GuiComponentMgr> for Sdf{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id: _, parent} => {
                console::log_1(&("RectElem create".into()));
                let mut s = self.0.borrow_mut();
                s.alloc_from_sdf(*parent, component_mgr)
            },
            Event::Delete{id: _, parent} => {
                let mut s = self.0.borrow_mut();
                s.free_from_sdf(*parent, component_mgr)
            },
            _ => ()
        }
    }
}

impl ComponentHandler<CircleElem, GuiComponentMgr> for Sdf{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id: _, parent} => {
                console::log_2(&("CircleElem create".into()), &(parent.to_string().into()));
                let mut s = self.0.borrow_mut();
                s.alloc_from_sdf(*parent, component_mgr)
            },
            Event::Delete{id: _, parent} => {
                let mut s = self.0.borrow_mut();
                s.free_from_sdf(*parent, component_mgr)
            },
            _ => ()
        }
    }
}

impl ComponentHandler<Aabb3, GuiComponentMgr> for Sdf{
    fn handle(&self, event: &Event, _component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id: _, parent} => self.0.borrow_mut().marked_shape_dirty(*parent),
            Event::ModifyField{id: _, parent, field: _} => self.0.borrow_mut().marked_shape_dirty(*parent),
            Event::Delete{id: _, parent} => self.0.borrow_mut().marked_shape_dirty(*parent),
            _ => ()
        }
    }
}

impl ComponentHandler<Rect, GuiComponentMgr> for Sdf{
    fn handle(&self, event: &Event, component_mgr: &mut GuiComponentMgr){
        match event {
            //此处只监听了Rect的radius字段的改变， Rect的其他字段对渲染没有意义， 最终的各点坐标还与布局有关（布局影响的包围盒）， 
            Event::ModifyField{id: _, parent, field} => {
                if *field == "radius" {
                    let parent = component_mgr.node.element.rect._group.get_mut(*parent).parent;
                    self.0.borrow_mut().marked_shape_dirty(parent)
                }
            },
            _ => ()
        }
    }
}

// impl ComponentHandler<Circle, GuiComponentMgr> for Sdf{
//     fn handle(&self, event: &Event, _component_mgr: &mut GuiComponentMgr){
//         match event {
//             //此处只监听了Rect的radius字段的改变， Rect的其他字段对渲染没有意义， 最终的各点坐标还与布局有关（布局影响的包围盒）， 
//             Event::ModifyField{id: _, parent, field} => {
//                 if *field == "radius" {
//                     self.0.borrow_mut().marked_shape_dirty(*parent)
//                 }
//             },
//             _ => ()
//         }
//     }
// }

impl ComponentHandler<Border, GuiComponentMgr> for Sdf{
    fn handle(&self, event: &Event, _component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id: _, parent} => self.0.borrow_mut().marked_style_dirty(*parent),
            Event::ModifyField{id: _, parent, field: _} => self.0.borrow_mut().marked_style_dirty(*parent),
            Event::Delete{id: _, parent} => self.0.borrow_mut().marked_style_dirty(*parent),
            _ => ()
        }
    }
}

impl ComponentHandler<SdfStyle, GuiComponentMgr> for Sdf{
    fn handle(&self, event: &Event, _component_mgr: &mut GuiComponentMgr){
        match event {
            Event::Create{id: _, parent} => self.0.borrow_mut().marked_style_dirty(*parent),
            Event::ModifyField{id: _, parent, field: _} => self.0.borrow_mut().marked_style_dirty(*parent),
            Event::Delete{id: _, parent} => self.0.borrow_mut().marked_style_dirty(*parent),
            _ => ()
        }
    }
}


impl System<(), GuiComponentMgr> for Sdf{
    fn run(&self, _e: &(), component_mgr: &mut GuiComponentMgr){
        let s = self.0.borrow_mut();

        for node_id in s.shape_dirtys.iter() {
            console::log_1(&("sdf_run shape_dirtys".into()));
            let index = s.indexs.get(node_id).unwrap();
            let node = component_mgr.node._group.get(*node_id);
            let bound_box = component_mgr.node.bound_box._group.get(node.bound_box);
            match node.element {
                ElementId::Rect(rect_elem_id) => {
                    let shape_id = component_mgr.node.element.rect._group.get(rect_elem_id).shape;
                    let radius = component_mgr.node.element.rect.shape._group.get(shape_id).radius;
                    unsafe {component_mgr.opaque_vector.update_rect(bound_box.min.x, bound_box.min.y, bound_box.max.x, bound_box.max.y, bound_box.min.z, radius, &index.index)};
                },
                ElementId::Circle(_) => {
                    let radius = (bound_box.max.x - bound_box.min.x)/2.0;
                    let center = (bound_box.min.x + radius, bound_box.min.y + radius);
                    console::log_3(&("Circle index".into()), &(index.index.tex.to_string().into()), &(index.index.attribute.to_string().into()));
                    unsafe {component_mgr.opaque_vector.update_circle(center, radius, bound_box.min.z, &index.index) };
                },
                _ => {}
            }

            
        }

        for node_id in s.style_dirtys.iter() {
            let index = s.indexs.get(node_id).unwrap();
            let node = component_mgr.node._group.get(*node_id);
            let border = component_mgr.node.border._group.get(node.border).value;
            match node.element {
                ElementId::Rect(rect_elem_id) => {
                    let style_id = component_mgr.node.element.rect._group.get(rect_elem_id).style;
                    let style = component_mgr.node.element.rect.style._group.get(style_id);
                    unsafe {component_mgr.opaque_vector.update_style(border, &style.color, index.index.tex) };
                },
                ElementId::Circle(circle_elem_id) => {
                    let style_id = component_mgr.node.element.circle._group.get(circle_elem_id).style;
                    let style = component_mgr.node.element.circle.style._group.get(style_id);
                    unsafe {component_mgr.opaque_vector.update_style(border, &style.color, index.index.tex) };
                },
                _ => {}
            }
        }
    }
}

struct IndexExtent {
    index: Index,
    shape_dirty: bool,
    style_dirty: bool,
}

pub struct SdfRenderImpl {
    indexs: FnvHashMap<usize, IndexExtent>,
    index_map_reverse: FnvHashMap<usize,  usize>, // atrribut index映射表， key为atrribut的index， value为node id
    shape_dirtys: Vec<usize>,
    style_dirtys: Vec<usize>,
}

impl SdfRenderImpl {
    pub fn new() -> SdfRenderImpl{
        SdfRenderImpl{
            indexs: FnvHashMap::default(),
            index_map_reverse: FnvHashMap::default(),
            shape_dirtys: Vec::new(),
            style_dirtys: Vec::new(),
        }
    }

    fn marked_shape_dirty(&mut self, node_id: usize){
        console::log_1(&("sdf_marked_shape_dirty".into()));
        match self.indexs.get_mut(&node_id) {
            Some(i) => {
                if i.shape_dirty == false {
                    i.shape_dirty = true;
                }else {
                    return;
                }
            },
            None => return,
        }

        self.shape_dirtys.push(node_id);
    }

    fn delete_shape_dirty(&mut self, node_id: usize){
        for i in 0..self.shape_dirtys.len(){
            if self.shape_dirtys[i] == node_id{
                self.shape_dirtys.remove(i);
                return;
            }
        }
    }

    fn marked_style_dirty(&mut self, node_id: usize){
        match self.indexs.get_mut(&node_id) {
            Some(i) => {
                if i.style_dirty == false {
                    i.style_dirty = true;
                }else {
                    return;
                }
            },
            None => return,
        }

        self.style_dirtys.push(node_id);
    }

    fn delete_style_dirty(&mut self, node_id: usize){
        for i in 0..self.style_dirtys.len(){
            if self.style_dirtys[i] == node_id{
                self.style_dirtys.remove(i);
                return;
            }
        }
    }

    fn alloc_from_sdf(&mut self, node_id: usize, component_mgr: &mut GuiComponentMgr ){
        let index = component_mgr.opaque_vector.alloc(); // 分配buffer
        self.index_map_reverse.insert(index.attribute, node_id); // 对index的attribute建立反向映射
        self.indexs.insert(node_id, IndexExtent{index: index, shape_dirty: false, style_dirty: false}); // 保存所有分配的buffer的index
    }

    fn free_from_sdf(&mut self, node_id: usize, component_mgr: &mut GuiComponentMgr ){
        let index = self.indexs.remove(&node_id).unwrap(); // 从indexs中删除
        let last_index = unsafe{component_mgr.opaque_vector.swap_delete_rect(&index.index)};
        if index.shape_dirty {
            self.delete_shape_dirty(node_id);
        }

        if index.shape_dirty {
            self.delete_style_dirty(node_id);
        }

        if last_index > index.index.attribute {
            let last_node_id = self.index_map_reverse.remove(&last_index).unwrap(); // 从反向映射表中删除
            //如果当前删除的不是最后一个buffer， 由于使用swap方式删除， 尾部buffer的位置会发生改变
            self.indexs.get_mut(&last_node_id).unwrap().index.attribute = last_index;
            self.index_map_reverse.insert(index.index.attribute, last_node_id); 
        }else {
            self.index_map_reverse.remove(&index.index.attribute);        // 从反向映射表中删除
        }
    }
}

// fn add_rect(r_point: &RectPoint, s_point: &SdfStylePoint, component_mgr: &mut GuiComponentMgr){
//     let rect = component_mgr.node.shape.rect_i_0._group.get(r_point);
//     let style = component_mgr.node.style.vector_i_0._group.get(s_point);
//     component_mgr.opaque_vector.add_rect(rect.left_top, rect.width, rect.height, 0.0, 0.0, style);
// }