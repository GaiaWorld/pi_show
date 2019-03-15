use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::default::Default;
use std::fmt;

use web_sys::*;

use wasm_bindgen::prelude::*;

pub struct YgNodeP(Node);

impl YgNodeP {
    pub fn create() -> YgNodeP {
        YgNodeP(Node::new())
    }

    pub fn new(node: Node) -> YgNodeP {
        YgNodeP(node)
    }

    pub fn set_position_type(&self, value: PositionType) { self.0.setPositionType(value as u8); }
    pub fn set_position(&self, edge: Edge, position: f32) { self.0.setPosition(edge as u8, position); }
    pub fn set_position_percent(&self, edge: Edge, position: f32) { self.0.setPositionPercent(edge as u8, position); }

    pub fn set_align_content(&self, value: AlignContent) { self.0.setAlignContent(value as u8); }
    pub fn set_align_items(&self, value: AlignItems) { self.0.setAlignItems(value as u8); }
    pub fn set_align_self(&self, value: AlignSelf) { self.0.setAlignSelf(value as u8); }
    pub fn set_flex_direction(&self, value: FlexDirection) { self.0.setFlexDirection(value as u8); }
    pub fn set_flex_wrap(&self, value: FlexWrap) { self.0.setFlexWrap(value as u8); }
    pub fn set_justify_content(&self, value: JustifyContent) { self.0.setJustifyContent(value as u8); }

    pub fn set_margin(&self, edge: Edge, position: f32) { self.0.setMargin(edge as u8, position); }
    pub fn set_margin_percent(&self, edge: Edge, position: f32) { self.0.setMarginPercent(edge as u8, position); }
    pub fn set_margin_auto(&self, edge: Edge) { self.0.setMarginAuto(edge as u8); }

    pub fn set_overflow(&self, value: Overflow) { self.0.setOverflow(value as u8); }
    pub fn set_display(&self, value: Display) { self.0.setDisplay(value as u8); }

    pub fn set_flex(&self, value: f32) { self.0.setFlex(value); }
    pub fn set_flex_basis(&self, value: f32) { self.0.setFlexBasis(value); }
    pub fn set_flex_basis_percent(&self, value: f32) { self.0.setFlexBasisPercent(value); }
    pub fn set_flex_grow(&self, value: f32) { self.0.setFlexGrow(value); }
    pub fn set_flex_shrink(&self, value: f32) { self.0.setFlexShrink(value); }

    pub fn set_width(&mut self, width: f32){ 
        
        self.0.setWidth(width); }
    pub fn set_width_percent(&mut self, width: f32){ self.0.setWidthPercent(width); }
    pub fn set_width_auto(&mut self){ self.0.setWidthAuto(); }
    pub fn set_height(&mut self, height: f32){ 
        console::log_2(&("set_height".into()), &(height.to_string().into()));
        self.0.setHeight(height); }
    pub fn set_height_percent(&mut self, width: f32){ self.0.setHeightPercent(width); }
    pub fn set_height_auto(&mut self){ self.0.setHeightAuto(); }

    pub fn set_min_width(&mut self, value: f32){ self.0.setMinWidth(value); }
    pub fn set_min_width_percent(&mut self, value: f32){ self.0.setMinWidthPercent(value); }
    pub fn set_min_height(&mut self, value: f32){ self.0.setMinHeight(value); }
    pub fn set_min_height_percent(&mut self, value: f32){ self.0.setMinHeightPercent(value); }

    pub fn set_max_width(&mut self, value: f32){ self.0.setMaxWidth(value); }
    pub fn set_max_width_percent(&mut self, value: f32){ self.0.setMaxWidthPercent(value); }
    pub fn set_max_height(&mut self, value: f32){ self.0.setMaxHeight(value); }
    pub fn set_max_height_percent(&mut self, value: f32){ self.0.setMaxHeightPercent(value); }

    pub fn set_aspect_ratio(&mut self, value: f32){ self.0.setAspectRatio(value); }
    pub fn set_border(&mut self, edge: Edge, value: f32){ self.0.setBorder(edge as u8, value); }
    pub fn set_padding(&mut self, edge: Edge, value: f32){ self.0.setPadding(edge as u8, value); }
    pub fn set_padding_percent(&mut self, edge: Edge, value: f32){ self.0.setPaddingPercent(edge as u8, value); }

    pub fn get_position_type(&self) -> PositionType { unsafe{ transmute(self.0.getPositionType()) } }
    pub fn get_position(&self, edge: Edge) -> f32 { self.0.getPosition(edge as u8) }

    pub fn get_align_content(&self) -> AlignContent { unsafe{ transmute(self.0.getAlignContent()) } }
    pub fn get_align_items(&self) -> AlignItems { unsafe{ transmute(self.0.getAlignItems()) } }
    pub fn get_align_self(&self) -> AlignSelf { unsafe{ transmute(self.0.getAlignSelf()) } }
    pub fn get_flex_wrap(&self) -> FlexWrap { unsafe{ transmute(self.0.getFlexWrap()) } }
    pub fn get_justify_content(&self) -> JustifyContent { unsafe{ transmute(self.0.getJustifyContent()) } }

    pub fn get_margin(&self, edge: Edge) -> f32 { self.0.getMargin(edge as u8) }

    pub fn get_flex_basis(&self) -> f32 { self.0.getFlexBasis() }
    pub fn get_flex_grow(&self) -> f32 { self.0.getFlexGrow() }
    pub fn get_flex_ghrink(&self) -> f32 { self.0.getFlexShrink() }

    pub fn get_width(&self) -> f32 { self.0.getWidth() }
    pub fn get_height(&self) -> f32 { self.0.getHeight() }

    pub fn get_min_width(&self) -> f32 { self.0.getMinWidth() }
    pub fn get_min_height(&self) -> f32 { self.0.getMinHeight() }

    pub fn get_max_width(&self) -> f32 { self.0.getMaxWidth() }
    pub fn get_max_height(&self) -> f32 { self.0.getMaxHeight() }

    pub fn get_aspect_ratio(&self) -> f32 { self.0.getAspectRatio() }

    pub fn get_border(&self, edge: Edge) -> f32 { self.0.getBorder(edge as u8) }

    pub fn get_overflow(&self) -> Overflow { unsafe{ transmute(self.0.getOverflow()) } }
    pub fn get_display(&self) -> Display { unsafe{ transmute(self.0.getDisplay()) } }

    pub fn get_padding(&self, edge: Edge) -> f32 { self.0.getPadding(edge as u8) }

    pub fn insert_child(&self, node: YgNodeP, index: usize){ self.0.insertChild(unsafe{transmute(node)}, index); }
    pub fn remove_child(&self, node: &YgNodeP){ self.0.removeChild(&node.0); }

    pub fn get_child(&self, index: usize) -> YgNodeP { YgNodeP::new(self.0.getChild(index)) }
    pub fn get_parent(&self) -> YgNodeP { YgNodeP::new(self.0.getParent()) }

    pub fn get_child_count(&self) -> usize { self.0.getChildCount() }

    pub fn is_reference_baseline(&self) -> bool { self.0.isReferenceBaseline() }
    pub fn set_is_reference_baseline(&self, value: bool) { self.0.setIsReferenceBaseline(value) }

    pub fn mark_dirty(&self) { self.0.markDirty() }
    pub fn is_dirty(&self) -> bool { self.0.isDirty() }

    pub fn calculate_layout(&mut self, width: f32, height:f32, direction: Direction){
        self.0.calculateLayout(width, height, direction as u8);
    }

    pub fn get_computed_layout(&self) -> Layout {
        Layout{
            left: self.0.getComputedLeft(),
            top: self.0.getComputedTop(),
            width: self.0.getComputedWidth(),
            height: self.0.getComputedHeight(),
        }
    }

    // pub fn get_computed_size(&self) -> Vector2 {
    //     Vector2{
    //         x: self.0.getComputedWidth(), 
    //         y:self.0.getComputedHeight()
    //     }
    // }

    pub fn get_computed_margin(&self, edge: Edge) -> f32 { self.0.getComputedMargin(edge as u8) }
    pub fn get_computed_border(&self, edge: Edge) -> f32 { self.0.getComputedBorder(edge as u8) }
    pub fn get_computed_padding(&self, edge: Edge) -> f32 { self.0.getComputedPadding(edge as u8) }

    // pub fn clone_node(&self) -> YgNodeP { YgNodeP::new(self.0.clone_node()) }
}

impl Clone for YgNodeP {
    fn clone(&self) -> YgNodeP {
        YgNodeP::new(self.0.clone_node())
    }
}

pub struct YgNode(YgNodeP);

impl YgNode{
    pub fn create() -> YgNode {
        YgNode(YgNodeP::create())
    }

    pub fn remove_child(&mut self, node: YgNode){
        self.0.remove_child( &node.0 );
    }

    pub fn remove_child_unfree(&mut self, node: YgNodeP) {
        self.0.remove_child( &node );
    }
}

impl Drop for YgNode {
    fn drop (&mut self) {
        (self.0).0.free();
    }
}

impl Deref for YgNode {
    type Target = YgNodeP;
    fn deref (&self) -> &YgNodeP {
        &self.0
    }
}

impl DerefMut for YgNode {
    fn deref_mut (&mut self) -> &mut YgNodeP {
        &mut self.0
    }
}

impl Default for YgNode{
    fn default() -> YgNode{
        YgNode::create()
    }
}

impl fmt::Debug for YgNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let layout = self.get_computed_layout();
        write!(f, "YgNode {{ left: {}, top: {}, width:{}, height:{} }}", layout.left, layout.top, layout.width, layout.height)
    }
}

pub struct Layout{
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[wasm_bindgen(module = "./yoga")]
extern "C" {
    pub type Node;
    #[wasm_bindgen(constructor)]
    pub fn new() -> Node;

    #[wasm_bindgen(method)]
    pub fn setPositionType(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setPosition(this: &Node, edge: u8, position: f32);
    #[wasm_bindgen(method)]
    pub fn setPositionPercent(this: &Node, edge: u8, position: f32);

    #[wasm_bindgen(method)]
    pub fn setAlignContent(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setAlignItems(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setAlignSelf(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setFlexDirection(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setFlexWrap(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setJustifyContent(this: &Node, value: u8);

    #[wasm_bindgen(method)]
    pub fn setMargin(this: &Node, edge: u8, position: f32);
    #[wasm_bindgen(method)]
    pub fn setMarginPercent(this: &Node, edge: u8, position: f32);
    #[wasm_bindgen(method)]
    pub fn setMarginAuto(this: &Node, edge: u8);

    #[wasm_bindgen(method)]
    pub fn setOverflow(this: &Node, value: u8);
    #[wasm_bindgen(method)]
    pub fn setDisplay(this: &Node, value: u8);

    #[wasm_bindgen(method)]
    pub fn setFlex(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setFlexBasis(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setFlexBasisPercent(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setFlexGrow(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setFlexShrink(this: &Node, value: f32);

    #[wasm_bindgen(method)]
    pub fn setWidth(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setWidthPercent(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setWidthAuto(this: &Node);
    #[wasm_bindgen(method)]
    pub fn setHeight(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setHeightPercent(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setHeightAuto(this: &Node);

    #[wasm_bindgen(method)]
    pub fn setMinWidth(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setMinWidthPercent(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setMinHeight(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setMinHeightPercent(this: &Node, value: f32);

    #[wasm_bindgen(method)]
    pub fn setMaxWidth(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setMaxWidthPercent(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setMaxHeight(this: &Node, value: f32);
    #[wasm_bindgen(method)]
    pub fn setMaxHeightPercent(this: &Node, value: f32);

    #[wasm_bindgen(method)]
    pub fn setAspectRatio(this: &Node, value: f32);

    #[wasm_bindgen(method)]
    pub fn setBorder(this: &Node, edge: u8, value: f32);
    #[wasm_bindgen(method)]

    #[wasm_bindgen(method)]
    pub fn setPadding(this: &Node, edge: u8, value: f32);
    #[wasm_bindgen(method)]
    pub fn setPaddingPercent(this: &Node, edge: u8, value: f32);

    #[wasm_bindgen(method)]
    pub fn getPositionType(this: &Node) -> u8;
    #[wasm_bindgen(method)]
    pub fn getPosition(this: &Node, edge: u8) -> f32;

    #[wasm_bindgen(method)]
    pub fn getAlignContent(this: &Node) -> u8;
    #[wasm_bindgen(method)]
    pub fn getAlignItems(this: &Node) -> u8;
    #[wasm_bindgen(method)]
    pub fn getAlignSelf(this: &Node) -> u8;
    #[wasm_bindgen(method)]
    pub fn getFlexWrap(this: &Node) -> u8;
    #[wasm_bindgen(method)]
    pub fn getJustifyContent(this: &Node) -> u8;

    #[wasm_bindgen(method)]
    pub fn getMargin(this: &Node, edge: u8) -> f32;

    #[wasm_bindgen(method)]
    pub fn getFlexBasis(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getFlexGrow(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getFlexShrink(this: &Node) -> f32;

    #[wasm_bindgen(method)]
    pub fn getWidth(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getHeight(this: &Node) -> f32;

    #[wasm_bindgen(method)]
    pub fn getMinWidth(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getMinHeight(this: &Node) -> f32;

    #[wasm_bindgen(method)]
    pub fn getMaxWidth(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getMaxHeight(this: &Node) -> f32;

    #[wasm_bindgen(method)]
    pub fn getAspectRatio(this: &Node) -> f32;

    #[wasm_bindgen(method)]
    pub fn getBorder(this: &Node, edge: u8) -> f32;

    #[wasm_bindgen(method)]
    pub fn getOverflow(this: &Node) -> u8;
    #[wasm_bindgen(method)]
    pub fn getDisplay(this: &Node) -> u8;

    #[wasm_bindgen(method)]
    pub fn getPadding(this: &Node, edge: u8) -> f32;

    #[wasm_bindgen(method)]
    pub fn insertChild(this: &Node, node: Node, index: usize);
    #[wasm_bindgen(method)]
    pub fn removeChild(this: &Node, node: &Node);

    #[wasm_bindgen(method)]
    pub fn getChildCount(this: &Node) -> usize;

    #[wasm_bindgen(method)]
    pub fn getParent(this: &Node) -> Node;
    #[wasm_bindgen(method)]
    pub fn getChild(this: &Node, index: usize) -> Node;

    #[wasm_bindgen(method)]
    pub fn isReferenceBaseline(this: &Node) -> bool;
    #[wasm_bindgen(method)]
    pub fn setIsReferenceBaseline(this: &Node, value: bool);

    #[wasm_bindgen(method)]
    pub fn markDirty(this: &Node);
    #[wasm_bindgen(method)]
    pub fn isDirty(this: &Node) -> bool;

    #[wasm_bindgen(method)]
    pub fn calculateLayout(this: &Node, width: f32, height:f32, direction: u8);
    

    #[wasm_bindgen(method)]
    pub fn getComputedLeft(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedRight(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedTop(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedBottom(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedWidth(this: &Node) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedHeight(this: &Node) -> f32;

    #[wasm_bindgen(method)]
    pub fn getComputedMargin(this: &Node, edge: u8) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedBorder(this: &Node, edge: u8) -> f32;
    #[wasm_bindgen(method)]
    pub fn getComputedPadding(this: &Node, edge: u8) -> f32;

    #[wasm_bindgen(method)]
    pub fn clone_node(this: &Node) -> Node;

    #[wasm_bindgen(method)]
    pub fn free(this: &Node);
}

//定义横轴方向， 当主轴为横轴是， 会与FlexDirection的值相会影响
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Inherit,
    LTR,
    RTL,
}

//主轴
#[derive(Debug, Copy, Clone)]
pub enum FlexDirection {
    Column, //主轴为垂直方向，起点在上沿。(默认)
    ColumnReverse,//主轴为垂直方向，起点在下沿。
    Row,//主轴为水平方向，起点在左端
    RowReverse,//主轴为水平方向，起点在右端。
}

//flex-wrap属性定义，如果一条轴线排不下，如何换行
#[derive(Debug, Copy, Clone)]
pub enum FlexWrap {
    NoWrap, //不换行
    Wrap, //下一行在下方
    WrapReverse, //下一行在上方
}

//定义了项目在主轴上的对齐方式
#[derive(Debug, Copy, Clone)]
pub enum JustifyContent {
    Start, //主轴方向起点对齐
    Center, //主轴方向居中对齐对齐
    End, //主轴方向终点对齐
    SpaceBetween, // 两端对齐，项目之间的间隔都相等
    SpaceAround, // 每个项目两侧的间隔相等。所以，项目之间的间隔比项目与边框的间隔大一倍
}

//定义项目在交叉轴上如何对齐
#[derive(Debug, Copy, Clone)]
pub enum AlignItems {
    Start, //交叉轴方向起点对齐
    Center, //交叉轴方向居中对齐
    End, //交叉轴方向终点对齐
    BaseLine, // 项目的第一行文字的基线对齐
    Stretch, // 如果项目未设置高度或设为auto，将占满整个容器的高度
}

// 定义了多根轴线的对齐方式。如果项目只有一根轴线，该属性不起作用
#[derive(Debug, Copy, Clone)]
pub enum AlignContent {
    Start, //与交叉轴的起点对齐
    Center, // 与交叉轴的中点对齐
    End, // 与交叉轴的终点对齐
    SpaceBetween, // 与交叉轴两端对齐，轴线之间的间隔平均分布
    SpaceAround, // 每根轴线两侧的间隔都相等。所以，轴线之间的间隔比轴线与边框的间隔大一倍
    Stretch, // 轴线占满整个交叉轴
}

//align-self属性允许单个项目有与其他项目不一样的对齐方式，可覆盖align-items属性。默认值为auto，表示继承父元素的align-items属性，如果没有父元素，则等同于stretch
#[derive(Debug, Copy, Clone)]
pub enum AlignSelf {
    Auto,
    Start,
    Center,
    End,
    BaseLine,
    Stretch,
}

//定位类型
#[derive(Debug, Copy, Clone)]
pub enum PositionType {
    Relative,
    Absolute,
}

#[derive(Debug, Copy, Clone)]
pub enum Edge {
    Left,
    Top,
    Right,
    Bottom,
    Start,
    End,
    Horizontal,
    Vertical,
    All,
}

#[derive(Debug, Copy, Clone)]
pub enum Overflow {
    YGOverflowVisible,
    YGOverflowHidden,
    YGOverflowScroll
}

#[derive(Debug, Copy, Clone)]
pub enum Display {
    Flex,
    None
}
