// 布局系统， 同时负责布局文本。
// 文本节点的布局判断：
// 情况一： 图文混排布局。如果该节点的父节点是无布局的节点，则将清空父yoga节点的子yoga节点，将其子yoga节点重新插入，将文本拆成每个字（英文为单词）的yaga节点加入父yoga节点。这样可以支持图文混排。
// 情况二： 普通布局。如果该节点的父节点是有布局的节点，则将该文本节点作为父节点，将字yaga节点加入该文本的yoga节点。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。

use std::cell::RefCell;
use std::rc::{Rc};
use std::os::raw::{c_void};
use std::mem::forget;
use std::collections::hash_map::Entry;
use std::ops::Deref;

use fnv::FnvHashMap;

use slab::{Slab};
use wcs::world::{System};
use wcs::component::{ComponentHandler, CreateEvent, ModifyFieldEvent, DeleteEvent};

use world_doc::component::style::border::Border;
use world_doc::component::style::element::{Element, Text};
use world_doc::component::node::{Node};
use world_doc::WorldDocMgr;
use component::math::{ Vector3 };
use layout::{YGEdge, YGDirection, YgNode};

pub struct Layout(RefCell<LayoutImpl>);

impl Layout {
    pub fn init(mgr: &mut WorldDocMgr) -> Rc<Layout>{
        let r = Rc::new(Layout(RefCell::new(LayoutImpl::new(mgr))));
        // 文本元素的监听
        mgr.node.element.text._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Text, CreateEvent, WorldDocMgr>>)));
        mgr.node.element.text._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Text, DeleteEvent, WorldDocMgr>>)));
        mgr.node.element.text._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Text, ModifyFieldEvent, WorldDocMgr>>)));
        r
    }
}

//监听文本创建事件
impl ComponentHandler<Text, CreateEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &CreateEvent, mgr: &mut WorldDocMgr){
        let CreateEvent {id, parent} = event;
        self.0.borrow_mut().create_text(*id, *parent);
    }
}
//监听文本删除事件
impl ComponentHandler<Text, DeleteEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &DeleteEvent, mgr: &mut WorldDocMgr){
        let DeleteEvent {id, parent} = event;
        self.0.borrow_mut().delete_text(*id, *parent);
    }
}
//监听文本修改事件
impl ComponentHandler<Text, ModifyFieldEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &ModifyFieldEvent, _component_mgr: &mut WorldDocMgr){
        let ModifyFieldEvent {id, parent, field: _} = event; // TODO 其他要判断样式是否影响布局
        self.0.borrow_mut().modify_text(*id, *parent);
    }
}

impl System<(), WorldDocMgr> for Layout{
    fn run(&self, _e: &(), mgr: &mut WorldDocMgr){
        let width = mgr.world_2d.component_mgr.width;
        let height = mgr.world_2d.component_mgr.height;
        let layoutImpl = self.0.borrow_mut().deref() as *const LayoutImpl;
        //计算布局，如果布局更改， 调用回调来设置layout属性
        mgr.node._group.get(mgr.get_root_id()).yoga.calculate_layout_by_callback(width, height, YGDirection::YGDirectionLTR, callback, layoutImpl as *const c_void);
    }
}


pub struct TextImpl {
  pub parent_ref: usize, // 初始化时，获得的对应的yoga节点。该节点取决于是否为图文混排布局。 每次计算要检查该节点
  pub height: usize, // 字体高度
  pub chars: Vec<usize>, // 字符集合
}
pub struct CharImpl {
  pub ch: char, // 字符
  pub width: usize, // 字符宽度
  pub parent: isize, // 对应的父节点，如果为正数表示Dom文本节点，负数为yoga节点（如果字节点是字符容器会出现这种情况）
  pub node: YgNode, // 对应的yoga节点
}
pub struct LayoutImpl{
    node_map: FnvHashMap<usize,TextImpl>,
    //char_slab: Slab<CharImpl>,
    mgr: *mut WorldDocMgr,
}

impl LayoutImpl {
    pub fn new(mgr: &mut WorldDocMgr) -> LayoutImpl{
        LayoutImpl{
            node_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            //char_slab: Slab::new(),
            mgr: mgr as *mut WorldDocMgr,
        }
    }
    // 立即生成yoga节点并加入
    pub fn create_text(&mut self, text_id: usize, node_id: usize) {
        // TODO 计算字体高度，根据父节点的布局，获取对应的parent_ref
        // self.node_map.insert(node_id, TextImpl {
        //     action: action,
        //     chars: Vec::new(),
        // });
    }
    // 如果是图文混排布局，立即删除yoga节点。 如果是普通布局，则由节点进行yaga节点的删除
    pub fn delete_text(&mut self, text_id: usize, node_id: usize) {
        // let text = component_mgr.node.element.text._group.get(text_id);
        // let font = component_mgr.node.element.text._group.get(text.font);
        // let text_style = component_mgr.node.element.text._group.get(text.text_style);
        // match self.node_map.entry(node_id) {
        //     Entry::Occupied(mut e) => {
        //         let v = e.get_mut();
        //         if v.action != ActionType::Delete {
        //             v.action = action;
        //         }
        //     },
        //     Entry::Vacant(e) => {
        //         e.insert(TextImpl{
        //             action: action,
        //             chars: Vec::new(),
        //         });
        //     }
        // }
    }
    // 更新文字， 先删除yoga节点，再生成yoga节点并加入
    pub fn modify_text(&mut self, text_id: usize, node_id: usize) {
        
    }
    // 文本布局改变
    pub fn update(&mut self, mgr: &mut WorldDocMgr, char_id: usize){
        // let mut char_map = self.char_map;
        // for (key, val) in self.node_map.iter_mut() {
        //     match val.dirty {
        //         ActionType::Delete => {

        //         },
        //         ActionType::Update => {

        //         },
        //         _ => ()
        //     }
        // }
    }
}

// 节点布局更新
fn update(mgr: &mut WorldDocMgr, node_id: usize) {
    let layout = {
        let yoga = &mgr.node._group.get(node_id).yoga;
        println!("update_layout, layout: {:?}, node_id:{}",  yoga.get_layout(), node_id);
        yoga.get_layout()
    };
    let mut node_ref = mgr.get_node_mut(node_id);
    //修改position size
    node_ref.set_layout(layout);
}

//回调函数
#[no_mangle]
extern "C" fn callback(callback_context: *const c_void, context: *const c_void) {
    //更新布局
    let node_id = unsafe { context as isize};
    let layoutImpl = unsafe{ &mut *(callback_context as usize as *mut LayoutImpl) };
    let mgr = unsafe{ &mut *(layoutImpl.mgr) };
    if node_id > 0 {
        update(mgr, node_id as usize);
    }else if node_id < 0 {
        layoutImpl.update(mgr, (-node_id) as usize);
    }
    forget(mgr);
    forget(layoutImpl);
}