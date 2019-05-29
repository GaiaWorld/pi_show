use std::default::Default;
use std::os::raw::{c_void};

use ecs::component::Component;
use map::vecmap::VecMap;

use component::user::Layout;
use layout::yoga;
pub use layout::yoga::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType, YGUnit};

// pub struct YgNode(YgNodeP);

// impl YgNode{
//     pub fn create() -> YgNode {
//         YgNode(YgNodeP::create())
//     }

//     pub fn remove_child(&mut self, node: YgNode){
//         self.0.remove_child( &node.0 );
//     }

//     pub fn remove_child_unfree(&mut self, node: YgNodeP) {
//         self.0.remove_child( &node );
//     }
// }

// impl Drop for YgNode {
//     fn drop (&mut self) {
//         (self.0).free();
//     }
// }

// impl Deref for YgNode {
//     type Target = YgNodeP;
//     fn deref (&self) -> &YgNodeP {
//         &self.0
//     }
// }

// impl DerefMut for YgNode {
//     fn deref_mut (&mut self) -> &mut YgNodeP {
//         &mut self.0
//     }
// }

// impl Default for YgNode{
//     fn default() -> YgNode{
//         YgNode::create()
//     }
// }

// impl fmt::Debug for YgNode {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let layout = self.get_layout();
//         write!(f, "YgNode {{ left: {}, top: {}, width:{}, height:{} }}", layout.left, layout.top, layout.width, layout.height)
//     }
// }


// pub type YgNodeP = YgNode;

#[derive(Clone, Debug, Copy, PartialEq, Component)]
pub struct YgNode( yoga::YGNodeRef);

unsafe impl Sync for YgNode{}
unsafe impl Send for YgNode{}

impl Default for YgNode{
    fn default() -> YgNode{
        let y = YgNode::new();
        y.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
        y
    }
}

impl YgNode {
    pub fn new() -> YgNode {
        YgNode(yoga::yg_node_new())
    }
    pub fn new_null() -> YgNode {
        YgNode( 0 as yoga::YGNodeRef)
    }
    pub fn is_null(&self) -> bool {
        self.is_null()
    }
    pub fn set_position_type(&self, value: YGPositionType) { 
        yoga::yg_node_style_set_position_type(self.0, value)
    }
    pub fn set_position(&self, edge: YGEdge, position: f32) { 
        yoga::yg_node_style_set_position(self.0, edge, position);
    }
    pub fn set_position_percent(&self, edge: YGEdge, position: f32) {
        yoga::yg_node_style_set_position_percent(self.0, edge, position);
    }

    pub fn set_align_content(&self, value: YGAlign) {
        yoga::yg_node_style_set_align_content(self.0, value);
    }

    pub fn set_align_items(&self, value: YGAlign) { 
        yoga::yg_node_style_set_align_items(self.0, value);
    }

    pub fn set_align_self(&self, value: YGAlign) { 
        yoga::yg_node_style_set_align_self(self.0, value);
    }
    pub fn set_flex_direction(&self, value: YGFlexDirection) { 
        yoga::yg_node_style_set_flex_direction(self.0, value);
    }
    pub fn set_flex_wrap(&self, value: YGWrap) { 
        yoga::yg_node_style_set_flex_wrap(self.0, value);
    }
    pub fn set_justify_content(&self, value: YGJustify) {
        yoga::yg_node_style_set_justify_content(self.0, value);
    }
    pub fn set_margin(&self, edge: YGEdge, value: f32) { 
        yoga::yg_node_style_set_margin(self.0, edge, value);
    }
    pub fn set_margin_percent(&self, edge: YGEdge, value: f32) { 
        yoga::yg_node_style_set_margin_percent(self.0, edge, value);
    }
    pub fn set_margin_auto(&self, edge: YGEdge) { 
        yoga::yg_node_style_set_margin_auto(self.0, edge);
    }

    pub fn set_overflow(&self, value: YGOverflow) { 
        yoga::yg_node_style_set_overflow(self.0, value);
    }

    pub fn set_display(&self, value: YGDisplay) { 
        yoga::yg_node_style_set_display(self.0, value);
    }

    pub fn set_flex(&self, value: f32) { 
        yoga::yg_node_style_set_flex(self.0, value);
    }
    pub fn set_flex_basis(&self, value: f32) { 
        yoga::yg_node_style_set_flex_basis(self.0, value); 
    }
    pub fn set_flex_basis_percent(&self, value: f32) { 
        yoga::yg_node_style_set_flex_basis_percent(self.0, value); 
    }
    pub fn set_flex_basis_auto(&self) { 
        yoga::yg_node_style_set_flex_basis_auto(self.0); 
    }
    pub fn set_flex_grow(&self, value: f32) { 
        yoga::yg_node_style_set_flex_grow(self.0, value);
    }
    pub fn set_flex_shrink(&self, value: f32) { 
        yoga::yg_node_style_set_flex_shrink(self.0, value); 
    }

    pub fn set_width(&self, value: f32) { 
        yoga::yg_node_style_set_width(self.0, value);
    }

    pub fn set_width_percent(&self, value: f32){ 
        yoga::yg_node_style_set_width_percent(self.0, value);
    }
    pub fn set_width_auto(&self){ 
        yoga::yg_node_style_set_width_auto(self.0);
    }
    pub fn set_height(&self, value: f32){ 
        yoga::yg_node_style_set_height(self.0, value);
    }
    pub fn set_height_percent(&self, value: f32){ 
        yoga::yg_node_style_set_height_percent(self.0, value);
    }
    pub fn set_height_auto(&self){ 
        yoga::yg_node_style_set_height_auto(self.0);
    }

    pub fn set_min_width(&self, value: f32){ 
        yoga::yg_node_style_set_min_width(self.0, value);
    }
    pub fn set_min_width_percent(&self, value: f32){ 
        yoga::yg_node_style_set_min_width_percent(self.0, value);
    }
    pub fn set_min_height(&self, value: f32){ 
        yoga::yg_node_style_set_min_height(self.0, value);
    }
    pub fn set_min_height_percent(&self, value: f32){
        yoga::yg_node_style_set_min_height_percent(self.0, value);
    }

    pub fn set_max_width(&self, value: f32){ 
        yoga::yg_node_style_set_max_width(self.0, value);
    }
    pub fn set_max_width_percent(&self, value: f32){
        yoga::yg_node_style_set_max_width_percent(self.0, value);
    }
    pub fn set_max_height(&self, value: f32){ 
        yoga::yg_node_style_set_max_height(self.0, value);
    }
    pub fn set_max_height_percent(&self, value: f32){
        yoga::yg_node_style_set_max_height_percent(self.0, value);
    }

    pub fn set_aspect_ratio(&self, value: f32){ 
        yoga::yg_node_style_set_aspect_ratio(self.0, value);
    }
    pub fn set_border(&self, edge: YGEdge, value: f32){ 
        yoga::yg_node_style_set_border(self.0, edge, value);
    }
    pub fn set_padding(&self, edge: YGEdge, value: f32){ 
        yoga::yg_node_style_set_padding(self.0, edge, value);
    }
    pub fn set_padding_percent(&self, edge: YGEdge, value: f32){
        yoga::yg_node_style_set_padding_percent(self.0, edge, value);
    }

    pub fn set_context(&self, context: *mut c_void){
        yoga::yg_node_set_context(self.0, context);
    }

    // pub fn get_position_type(&self) -> YGPositionType {
    //    yoga::yg_node_style_get_position_type(self.0)
    // }
    // pub fn get_position(&self, edge: YGEdge) -> f32 { 
    //     yoga::yg_node_style_get_position(self.0, edge)
    // }

    // pub fn get_align_content(&self) -> YGAlign {
    //     yoga::yg_node_style_get_align_content(self.0) 
    // }
    // pub fn get_align_items(&self) -> YGAlign{ 
    //     yoga::yg_node_style_get_align_items(self.0)
    // }
    // pub fn get_align_self(&self) -> YGAlign { 
    //     yoga::yg_node_style_get_align_self(self.0)
    // }
    // pub fn get_flex_wrap(&self) -> YGWrap {
    //     yoga::yg_node_style_get_flex_wrap(self.0)
    // }
    // pub fn get_justify_content(&self) -> YGJustify { 
    //     yoga::yg_node_style_get_position_type(self.0)
    // }

    // pub fn get_margin(&self, edge: YGEdge) -> f32 { 
    //     yoga::yg_node_style_get_margin(self.0)
    // }

    // pub fn get_flex_basis(&self) -> f32 {
    //     yoga::yg_node_style_get_flex_basis(self.0)
    // }
    // pub fn get_flex_grow(&self) -> f32 { 
    //     yoga::yg_node_style_get_flex_grow(self.0)
    // }
    // pub fn get_flex_shrink(&self) -> f32 { 
    //     yoga::yg_node_style_get_flex_shrink(self.0)
    // }

    // pub fn get_width(&self) -> f32 { 
    //     yoga::yg_node_style_get_width(self.0)
    // }
    // pub fn get_height(&self) -> f32 { 
    //     yoga::yg_node_style_get_height(self.0)
    // }

    // pub fn get_min_width(&self) -> f32 { 
    //     yoga::yg_node_style_get_min_width(self.0)
    // }
    // pub fn get_min_height(&self) -> f32 { 
    //     yoga::yg_node_style_get_min_height(self.0)
    // }

    // pub fn get_max_width(&self) -> f32 { 
    //     yoga::yg_node_style_get_max_width(self.0)
    // }
    // pub fn get_max_height(&self) -> f32 { 
    //     yoga::yg_node_style_get_max_height(self.0)
    // }

    // pub fn get_aspect_ratio(&self) -> f32 { 
    //     yoga::yg_node_style_get_aspect_ratio(self.0)
    // }

    // pub fn get_border(&self, edge: YGEdge) -> f32 {
    //     yoga::yg_node_style_get_border(self.0, edge)
    // }

    // pub fn get_overflow(&self) -> YGOverflow { 
    //     yoga::yg_node_style_get_overflow(self.0)
    // }
    // pub fn get_display(&self) -> YGDisplay { 
    //     yoga::yg_node_style_get_display(self.0)
    // }

    // pub fn get_padding(&self, edge: YGEdge) -> f32 { 
    //     yoga::yg_node_style_get_padding(self.0)
    // }

    pub fn insert_child(&self, node: YgNode, index: u32){
        yoga::yg_node_insert_child(self.0, node.0, index)
    }

    pub fn remove_child(&self, node: YgNode){
        yoga::yg_node_remove_child(self.0, node.0)
    }

    pub fn get_child(&self, index: u32) -> YgNode {
        YgNode(yoga::yg_node_get_child(self.0, index))
    }

    pub fn get_parent(&self) -> YgNode {
        YgNode(yoga::yg_node_get_parent(self.0))
    }

    pub fn get_child_count(&self) -> u32 {
        yoga::yg_node_get_child_count(self.0)
    }

    pub fn get_width(&self) -> yoga::YGValue {
        yoga::yg_node_style_get_width(self.0)
    }

    pub fn get_height(&self) -> yoga::YGValue {
        yoga::yg_node_style_get_height(self.0)
    }
    pub fn get_context(&self) -> *mut c_void {
        yoga::yg_node_get_context(self.0)
    }
    // pub fn is_reference_baseline(&self) -> bool { 
    //     yoga::yg_node_style_get_position_type(self.0)
    // }
    // pub fn set_is_reference_baseline(&self, value: bool) { 
    //     yoga::yg_node_style_get_position_type(self.0)
    // }

    pub fn mark_dirty(&self) { 
        yoga::yg_node_mark_dirty(self.0)
    }
    pub fn is_dirty(&self) -> bool { 
        yoga::yg_node_is_dirty(self.0)
    }

    pub fn calculate_layout(&self, width: f32, height:f32, direction: YGDirection){
        yoga::yg_node_calculate_layout(self.0, width, height, direction);
    }

    pub fn calculate_layout_by_callback(&self, width: f32, height:f32, direction: YGDirection, callback: CallbackFunc, arg: *const c_void) {
        yoga::yg_node_calculate_layout_by_callback(self.0, width, height, direction, unsafe { std::mem::transmute(callback) }, arg);
    }

//     为指定节点设置上下文
// yoga::yg_node_set_context(node, Box::into_raw(Box::new(0u8)) as *mut c_void);
// 计算布局并回调
// yoga::yg_node_calculate_layout_by_callback(node, 1000.0, 1000.0, YGDirection::YGDirectionLTR, callback);

// //回调函数
// #[no_mangle]
// extern "C" fn callback(context: *const c_void) {
//     //更新布局
//     let node = unsafe { Box::from_raw(context as *mut u8) };
//     Box::into_raw(Box::new(node));
// }

    pub fn get_layout(&self) -> Layout {
        Layout{
            left: yoga::yg_node_layout_get_left(self.0),
            top: yoga::yg_node_layout_get_top(self.0),
            width: yoga::yg_node_layout_get_width(self.0),
            height: yoga::yg_node_layout_get_height(self.0),
            border_left: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeLeft),
            border_top: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeTop),
            border_right: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeRight),
            border_bottom: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeBottom),
            padding_left: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeLeft),
            padding_top: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeTop),
            padding_right: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeRight),
            padding_bottom: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeBottom),
        }
    }

    // pub fn get_computed_size(&self) -> Vector2 {
    //     Vector2{
    //         x: self.0.getComputedWidth(), 
    //         y:self.0.getComputedHeight()
    //     }
    // }

    pub fn get_layout_margin(&self, edge: YGEdge) -> f32 { 
        yoga::yg_node_layout_get_margin(self.0, edge)
    }
    pub fn get_layout_border(&self, edge: YGEdge) -> f32 { 
        yoga::yg_node_layout_get_border(self.0, edge)
    }
    pub fn get_layout_padding(&self, edge: YGEdge) -> f32 { 
        yoga::yg_node_layout_get_padding(self.0, edge)
    }

    pub fn free(&self){ 
        yoga::yg_node_free(self.0)
    }

    pub fn free_recursive(&self) {
        yoga::yg_node_free_recursive(self.0)
    }

    pub fn reset(&self) {
        yoga::yg_node_reset(self.0)
    }

    pub fn get_index(&self) -> usize {
        let parent_yoga = self.get_parent();
        let mut index = parent_yoga.get_child_count();
        while index > 0 && parent_yoga.get_child(index-1) != *self {
            index-=1;
        }
        index -= 1;
        index as usize
    }
}

pub type CallbackFunc = unsafe extern "C" fn(node: YgNode, args: *const c_void);


// #[derive(Clone, Debug, PartialEq, Eq, ReferenceType)]
// #[reference(instance_of = "YgNode")]
// pub struct YgNode( Reference );

// impl YgNode {
//     pub fn new () -> YgNode {
//         js! (return new YgNode();).try_into().unwrap()
//     }

//     pub fn set_position_type (&self, ) {
//         js! (
//             return @{self}.setPositionType();
//         );
//     }
// }


// extern "C" {
//     pub type Node;
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> Node;

//     #[wasm_bindgen(method)]
//     pub fn setPositionType(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setPosition(this: &Node, edge: u8, position: f32);
//     #[wasm_bindgen(method)]
//     pub fn setPositionPercent(this: &Node, edge: u8, position: f32);

//     #[wasm_bindgen(method)]
//     pub fn setAlignContent(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setAlignItems(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setAlignSelf(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setFlexDirection(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setFlexWrap(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setJustifyContent(this: &Node, value: u8);

//     #[wasm_bindgen(method)]
//     pub fn setMargin(this: &Node, edge: u8, position: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMarginPercent(this: &Node, edge: u8, position: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMarginAuto(this: &Node, edge: u8);

//     #[wasm_bindgen(method)]
//     pub fn setOverflow(this: &Node, value: u8);
//     #[wasm_bindgen(method)]
//     pub fn setDisplay(this: &Node, value: u8);

//     #[wasm_bindgen(method)]
//     pub fn setFlex(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setFlexBasis(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setFlexBasisPercent(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setFlexGrow(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setFlexShrink(this: &Node, value: f32);

//     #[wasm_bindgen(method)]
//     pub fn setWidth(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setWidthPercent(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setWidthAuto(this: &Node);
//     #[wasm_bindgen(method)]
//     pub fn setHeight(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setHeightPercent(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setHeightAuto(this: &Node);

//     #[wasm_bindgen(method)]
//     pub fn setMinWidth(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMinWidthPercent(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMinHeight(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMinHeightPercent(this: &Node, value: f32);

//     #[wasm_bindgen(method)]
//     pub fn setMaxWidth(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMaxWidthPercent(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMaxHeight(this: &Node, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setMaxHeightPercent(this: &Node, value: f32);

//     #[wasm_bindgen(method)]
//     pub fn setAspectRatio(this: &Node, value: f32);

//     #[wasm_bindgen(method)]
//     pub fn setBorder(this: &Node, edge: u8, value: f32);
//     #[wasm_bindgen(method)]

//     #[wasm_bindgen(method)]
//     pub fn setPadding(this: &Node, edge: u8, value: f32);
//     #[wasm_bindgen(method)]
//     pub fn setPaddingPercent(this: &Node, edge: u8, value: f32);

//     #[wasm_bindgen(method)]
//     pub fn getPositionType(this: &Node) -> u8;
//     #[wasm_bindgen(method)]
//     pub fn getPosition(this: &Node, edge: u8) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getAlignContent(this: &Node) -> u8;
//     #[wasm_bindgen(method)]
//     pub fn getAlignItems(this: &Node) -> u8;
//     #[wasm_bindgen(method)]
//     pub fn getAlignSelf(this: &Node) -> u8;
//     #[wasm_bindgen(method)]
//     pub fn getFlexWrap(this: &Node) -> u8;
//     #[wasm_bindgen(method)]
//     pub fn getJustifyContent(this: &Node) -> u8;

//     #[wasm_bindgen(method)]
//     pub fn getMargin(this: &Node, edge: u8) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getFlexBasis(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getFlexGrow(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getFlexShrink(this: &Node) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getWidth(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getHeight(this: &Node) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getMinWidth(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getMinHeight(this: &Node) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getMaxWidth(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getMaxHeight(this: &Node) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getAspectRatio(this: &Node) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getBorder(this: &Node, edge: u8) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getOverflow(this: &Node) -> u8;
//     #[wasm_bindgen(method)]
//     pub fn getDisplay(this: &Node) -> u8;

//     #[wasm_bindgen(method)]
//     pub fn getPadding(this: &Node, edge: u8) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn insertChild(this: &Node, node: Node, index: usize);
//     #[wasm_bindgen(method)]
//     pub fn removeChild(this: &Node, node: &Node);

//     #[wasm_bindgen(method)]
//     pub fn getChildCount(this: &Node) -> usize;

//     #[wasm_bindgen(method)]
//     pub fn getParent(this: &Node) -> Node;
//     #[wasm_bindgen(method)]
//     pub fn getChild(this: &Node, index: usize) -> Node;

//     #[wasm_bindgen(method)]
//     pub fn isReferenceBaseline(this: &Node) -> bool;
//     #[wasm_bindgen(method)]
//     pub fn setIsReferenceBaseline(this: &Node, value: bool);

//     #[wasm_bindgen(method)]
//     pub fn markDirty(this: &Node);
//     #[wasm_bindgen(method)]
//     pub fn isDirty(this: &Node) -> bool;

//     #[wasm_bindgen(method)]
//     pub fn calculateLayout(this: &Node, width: f32, height:f32, direction: u8);
    

//     #[wasm_bindgen(method)]
//     pub fn getComputedLeft(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedRight(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedTop(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedBottom(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedWidth(this: &Node) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedHeight(this: &Node) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn getComputedMargin(this: &Node, edge: u8) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedBorder(this: &Node, edge: u8) -> f32;
//     #[wasm_bindgen(method)]
//     pub fn getComputedPadding(this: &Node, edge: u8) -> f32;

//     #[wasm_bindgen(method)]
//     pub fn clone_node(this: &Node) -> Node;

//     #[wasm_bindgen(method)]
//     pub fn free(this: &Node);
// }

// //定义横轴方向， 当主轴为横轴是， 会与FlexDirection的值相会影响
// #[derive(Debug, Copy, Clone)]
// pub enum Direction {
//     Inherit,
//     LTR,
//     RTL,
// }

// //主轴
// #[derive(Debug, Copy, Clone)]
// pub enum FlexDirection {
//     Column, //主轴为垂直方向，起点在上沿。(默认)
//     ColumnReverse,//主轴为垂直方向，起点在下沿。
//     Row,//主轴为水平方向，起点在左端
//     RowReverse,//主轴为水平方向，起点在右端。
// }

// //flex-wrap属性定义，如果一条轴线排不下，如何换行
// #[derive(Debug, Copy, Clone)]
// pub enum FlexWrap {
//     NoWrap, //不换行
//     Wrap, //下一行在下方
//     WrapReverse, //下一行在上方
// }

// //定义了项目在主轴上的对齐方式
// #[derive(Debug, Copy, Clone)]
// pub enum JustifyContent {
//     Start, //主轴方向起点对齐
//     Center, //主轴方向居中对齐对齐
//     End, //主轴方向终点对齐
//     SpaceBetween, // 两端对齐，项目之间的间隔都相等
//     SpaceAround, // 每个项目两侧的间隔相等。所以，项目之间的间隔比项目与边框的间隔大一倍
// }

// //定义项目在交叉轴上如何对齐
// #[derive(Debug, Copy, Clone)]
// pub enum AlignItems {
//     Start, //交叉轴方向起点对齐
//     Center, //交叉轴方向居中对齐
//     End, //交叉轴方向终点对齐
//     BaseLine, // 项目的第一行文字的基线对齐
//     Stretch, // 如果项目未设置高度或设为auto，将占满整个容器的高度
// }

// // 定义了多根轴线的对齐方式。如果项目只有一根轴线，该属性不起作用
// #[derive(Debug, Copy, Clone)]
// pub enum AlignContent {
//     Start, //与交叉轴的起点对齐
//     Center, // 与交叉轴的中点对齐
//     End, // 与交叉轴的终点对齐
//     SpaceBetween, // 与交叉轴两端对齐，轴线之间的间隔平均分布
//     SpaceAround, // 每根轴线两侧的间隔都相等。所以，轴线之间的间隔比轴线与边框的间隔大一倍
//     Stretch, // 轴线占满整个交叉轴
// }

// //align-self属性允许单个项目有与其他项目不一样的对齐方式，可覆盖align-items属性。默认值为auto，表示继承父元素的align-items属性，如果没有父元素，则等同于stretch
// #[derive(Debug, Copy, Clone)]
// pub enum AlignSelf {
//     Auto,
//     Start,
//     Center,
//     End,
//     BaseLine,
//     Stretch,
// }

// //定位类型
// #[derive(Debug, Copy, Clone)]
// pub enum PositionType {
//     Relative,
//     Absolute,
// }

// #[derive(Debug, Copy, Clone)]
// pub enum YGEdge {
//     Left,
//     Top,
//     Right,
//     Bottom,
//     Start,
//     End,
//     Horizontal,
//     Vertical,
//     All,
// }

// #[derive(Debug, Copy, Clone)]
// pub enum Overflow {
//     YGOverflowVisible,
//     YGOverflowHidden,
//     YGOverflowScroll
// }

// #[derive(Debug, Copy, Clone)]
// pub enum Display {
//     Flex,
//     None
// }
