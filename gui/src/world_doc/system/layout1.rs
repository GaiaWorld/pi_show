// 布局系统， 同时负责布局文本。
// 文本节点的布局算法： 文本节点本身所对应的yoga节点总是一个0大小的节点。文本节点的父节点才是进行文本布局的节点，称为P节点。P节点如果没有设置布局，则默认用flex布局模拟文档流布局。会将文本拆成每个字（英文为单词）的yaga节点加入P节点上。这样可以支持图文混排。P节点如果有flex布局，则遵循该布局。
// 字节点，根据字符是否为单字决定是需要字符容器还是单字。
// 文字根据样式，会处理：缩进，是否合并空白符，是否自动换行，是否允许换行符。来设置相应的flex布局。 换行符采用高度为0, 宽度100%的yaga节点来模拟。

use std::cell::RefCell;
use std::rc::{Rc};
use std::sync::Arc;
use std::os::raw::{c_void};
use std::collections::hash_map::Entry;
use std::ops::Deref;

use fnv::FnvHashMap;

use slab::{Slab};
use wcs::world::{System};
use wcs::component::{ComponentHandler, CreateEvent, ModifyFieldEvent, DeleteEvent};

use world_doc::font::{split, SplitResult};
use world_doc::component::style::border::Border;
use world_doc::component::style::element::{Element, Text};
use world_doc::component::node::{Node};
use world_doc::WorldDocMgr;
use component::math::{ Vector3, Matrix4, Color as MathColor, Point2 };
use component::color::{Color};
use world_2d::component::char_block::CharBlock;
use layout::{YGEdge, YGDirection, YgNode, Layout as LV};
use font::sdf_font::{SdfFont, StaticSdfFont};
use font::font_sheet::{get_size, get_line_height};
use render::res::{TextureRes};
use text_layout::layout::{TextAlign};
use world_doc::component::style::text::TextStyle;

pub struct Layout(RefCell<LayoutImpl>);

impl Layout {
    pub fn init(mgr: &mut WorldDocMgr) -> Rc<Layout>{
        let r = Rc::new(Layout(RefCell::new(LayoutImpl::new(mgr))));
        // 文本元素的监听
        mgr.node.element.text._group.register_create_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Text, CreateEvent, WorldDocMgr>>)));
        mgr.node.element.text._group.register_delete_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Text, DeleteEvent, WorldDocMgr>>)));
        mgr.node.element.text._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Text, ModifyFieldEvent, WorldDocMgr>>)));
        // 世界矩阵的修改监听
        mgr.node.world_matrix._group.register_modify_field_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr>>)));
        // 监听 各属性的变动
        mgr.node.z_depth.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        mgr.node.opacity.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        mgr.node.visibility.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        mgr.node.by_overflow.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr>>)));
        r
    }
}

//监听文本创建事件
impl ComponentHandler<Text, CreateEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &CreateEvent, mgr: &mut WorldDocMgr){
        let CreateEvent {id, parent} = event;
        self.0.borrow_mut().create_text(mgr, *id, *parent);
    }
}
//监听文本删除事件
impl ComponentHandler<Text, DeleteEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &DeleteEvent, mgr: &mut WorldDocMgr){
        let DeleteEvent {id, parent} = event;
        self.0.borrow_mut().delete_text(mgr, *id, *parent);
    }
}
//监听文本修改事件
impl ComponentHandler<Text, ModifyFieldEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr){
        let ModifyFieldEvent {id, parent, field} = event;
        //self.0.borrow_mut().modify_text(mgr, *id, *parent);
        // 其他要判断样式是否影响布局
        let mut sys = self.0.borrow_mut();
        match sys.node_map.get(parent) {
            Some(text) => {
                match *field {
                    "value" => sys.modify_text(mgr, *id, *parent),
                    "text_style" => {
                        let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                        let style= mgr.node.element.text.text_style._group.get(*id);
                        let align = if let Some(r) = style.text_align {
                            r
                        }else{
                            TextAlign::Left
                        };
                        let ls = if let Some(r) = style.letter_spacing {
                            r
                        }else{
                            0.0
                        };
                        char_block.set_letter_spacing(ls);
                        char_block.set_line_height(get_line_height(text.font_size, &style.line_height));
                        //char_block.set_color(style.stroke_size);
                        //char_block.set_stroke_color(style.stroke_size);
                        //char_block.set_stroke_size(style.stroke_size);
                        // TODO shadow
                    },
                    "font" => {
                        let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                        //char_block.set_line_height(mgr.node.element.text._group.get(*id).line_height)
                    },
                    _ => ()
                }
            }
            _ => ()
        }
    }
}

//监听世界矩阵修改事件
impl ComponentHandler<Matrix4, ModifyFieldEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr){
        let ModifyFieldEvent {id, parent, field: _} = event;
        match self.0.borrow_mut().node_map.get(parent) {
            Some(text) => {
                let world_matrix = mgr.node.world_matrix._group.get(*id);
                let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                char_block.set_world_matrix(world_matrix.owner.clone())
            },
            _ => ()
        }
    }
}
//监听node各属性的修改事件
impl ComponentHandler<Node, ModifyFieldEvent, WorldDocMgr> for Layout{
    fn handle(&self, event: &ModifyFieldEvent, mgr: &mut WorldDocMgr){
        let ModifyFieldEvent {id, parent:_, field} = event;
        match self.0.borrow_mut().node_map.get(id) {
            Some(text) => {
                match *field {
                    "z_depth" => {
                        let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                        char_block.set_z_depth(mgr.node._group.get(*id).z_depth)
                    },
                    "opacity" => {
                        let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                        char_block.set_alpha(mgr.node._group.get(*id).opacity)
                    },
                    "visibility" => {
                        let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                        char_block.set_visibility(mgr.node._group.get(*id).visibility)
                    },
                    "by_overflow" => {
                        let mut char_block = mgr.world_2d.component_mgr.get_char_block_mut(text.rid);
                        char_block.set_by_overflow(mgr.node._group.get(*id).by_overflow)
                    },
                    _ => ()
                }
            }
            _ => ()
        }
    }
}
impl System<(), WorldDocMgr> for Layout{
    fn run(&self, _e: &(), mgr: &mut WorldDocMgr){
        let width = mgr.world_2d.component_mgr.width;
        let height = mgr.world_2d.component_mgr.height;
        let layout_impl = self.0.borrow_mut();
        //计算布局，如果布局更改， 调用回调来设置layout属性，及字符的位置
        mgr.node._group.get(mgr.get_root_id()).yoga.calculate_layout_by_callback(width, height, YGDirection::YGDirectionLTR, callback, layout_impl.deref() as *const LayoutImpl as *const c_void);
    }
}


pub struct TextImpl {
  pub font_size: f32, // 字体高度
  pub chars: Vec<usize>, // 字符集合, CharImpl的id
  pub rid: usize, // 渲染节点的id
}
pub struct CharImpl {
  pub ch: char, // 字符, 0为容器节点
  pub width: usize, // 字符宽度
  pub parent: isize, // 对应的父节点，如果为正数表示Dom文本节点，负数为yoga节点（如果字节点是字符容器会出现这种情况）
  pub node: YgNode, // 对应的yoga节点
  pub rid: usize, // 渲染节点的id
  pub rindex: usize, // 渲染节点的偏移量
}
pub struct LayoutImpl{
    node_map: FnvHashMap<usize,TextImpl>,
    char_slab: Slab<CharImpl>,
    mgr: *mut WorldDocMgr,
}

impl LayoutImpl {
    pub fn new(mgr: &mut WorldDocMgr) -> LayoutImpl{
        LayoutImpl{
            node_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            char_slab: Slab::default(),
            mgr: mgr as *mut WorldDocMgr,
        }
    }
    // 文本立即生成yoga节点并加入
    pub fn create_text(&mut self, mgr: &mut WorldDocMgr, text_id: usize, node_id: usize) {
        // 获得字体高度
        let text = mgr.node.element.text._group.get(text_id);
        let font = mgr.node.element.text.font._group.get(text.font);
        let font_size = mgr.font.get_size(&font.family, &font.size);
        if font_size == 0.0 {
            return;
        }
        let text_style = mgr.node.element.text.text_style._group.get(text.text_style);
        let mut vec = Vec::new();
        let node = mgr.node._group.get(node_id);
        let yaga = node.yoga;
        let parent_yaga = &mgr.node._group.get(node.parent).yoga;
        let mut word: Option<YgNode> = None;
        let merge_whitespace = match text_style.white_space {
            Some(w) => w.preserve_spaces(),
            _ => true
        };
        let letter_spacing = match text_style.white_space {
            Some(w) => w.preserve_spaces(),
            _ => true
        };
        // 设置char_block
        let mut char_block = CharBlock {
        world_matrix: Matrix4::default(),
            alpha: 1.0,
            visibility: true,
            is_opaque: true,
            z_depth: 1.0,
            by_overflow: 0,
            stroke_size: 0.0,
            stroke_color: MathColor::default(),
            font_size: 16.0,
            text_align: TextAlign::Left, //对齐方式
            letter_spacing: 2.0, //字符间距， 单位：像素
            line_height: 18.0, //设置行高
            sdf_font: create_sdf_font(0),
            color: Color::RGBA(MathColor::default()),
            chars: Vec::new(),
        };
        let cb = mgr.world_2d.component_mgr.add_char_block(char_block);
        let rid = cb.id;
        let mut rindex = 0;
        // 计算节点的yaga节点在父节点的yaga节点的位置
        let mut index = parent_yaga.get_child_count();
        while index > 0 && parent_yaga.get_child(index) != yaga {
            index-=1;
        }
        // 根据每个字符, 创建对应的yoga节点, 加入父容器或字容器中
        for cr in split(&text.value, true, merge_whitespace) {
            match cr {
                SplitResult::Newline =>{
                    let yg = YgNode::default();
                    // 设置成宽度100%, 高度0
                    // 如果有缩进, 则添加制表符的空节点, 宽度为缩进值
                    vec.push(self.char_slab.insert(CharImpl{
                        ch: '\n',
                        width: 0,
                        parent: node_id as isize,
                        node: yg,
                        rid: rid,
                        rindex: rindex,
                    }));
                    // TODO 将Char加入vec中
                    rindex +=1;
                    parent_yaga.insert_child(yg.clone(), index);
                },
                SplitResult::Whitespace =>{
                    let yg = YgNode::default();
                    // 设置成宽度为半高, 高度0
                },
                SplitResult::Word(c) =>{
                    let yg = YgNode::default();
                    // 设置成宽高为字符大小, 
                    
                },
                SplitResult::WordStart(c) =>{
                    // 设置word节点成宽高为自适应内容, 字符为0
                    word = Some(YgNode::default());
                },
                SplitResult::WordNext(c) =>{
                    let yg = YgNode::default();
                },
                SplitResult::WordEnd(c) =>{
                    if c != char::from(0) {
                        let yg = YgNode::default();
                    }
                },
            }
        }
        self.node_map.insert(node_id, TextImpl {
            font_size: font_size,
            chars: vec,
            rid: rid,
        });
    }
    // 立即删除自己增加的yoga节点
    pub fn delete_text(&mut self, mgr: &mut WorldDocMgr, _text_id: usize, node_id: usize) {
        match self.node_map.remove(&node_id) {
            Some(t) => {
                // 删除所有yaga节点
                for id in t.chars {
                    self.char_slab.remove(id).node.free();
                }
                // 移除渲染节点
                mgr.world_2d.component_mgr.get_char_block_mut(t.rid).destroy();
            },
            _ => ()
        }
    }
    // 更新文字， 先删除yoga节点，再生成yoga节点并加入
    pub fn modify_text(&mut self, mgr: &mut WorldDocMgr, text_id: usize, node_id: usize) {
        
    }
    // 文本布局改变
    pub fn update(&mut self, mgr: &mut WorldDocMgr, char_id: usize){
        let text = unsafe {self.char_slab.get_unchecked_mut(char_id)};
        let rnode = mgr.world_2d.component_mgr.char_block._group.get_mut(text.rid);
        let ch = unsafe {rnode.chars.get_unchecked_mut(text.rindex)};
        let layout = text.node.get_layout();
        ch.pos.x = layout.left;
        ch.pos.y = layout.top;
        // TODO 发监听
    }
}

// 节点布局更新
fn update(mgr: &mut WorldDocMgr, node_id: usize) {
    let layout = {
        let yoga = &mgr.node._group.get(node_id).yoga;
        yoga.get_layout()
    };
    let mut node_ref = mgr.get_node_mut(node_id);
    //修改position size
    node_ref.set_layout(layout);
}

//回调函数
extern "C" fn callback(callback_context: *const c_void, context: *const c_void) {
    //更新布局
    let node_id = context as isize;
    let layout_impl = unsafe{ &mut *(callback_context as usize as *mut LayoutImpl) };
    let mgr = unsafe{ &mut *(layout_impl.mgr) };
    if node_id > 0 {
        update(mgr, node_id as usize);
    }else if node_id < 0 {
        layout_impl.update(mgr, (-node_id) as usize);
    }
}

pub fn create_sdf_font(texture: u32) -> Arc<SdfFont>{
    let mut sdf_font = StaticSdfFont::new(unsafe { &*(texture as usize as *const Rc<TextureRes>)}.clone());
    Arc::new(sdf_font)
}