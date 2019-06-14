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

#[derive(Debug)]
pub struct Style{
    left: yoga::YGValue,
    right: yoga::YGValue,
    top: yoga::YGValue,
    bottom: yoga::YGValue,
    width: yoga::YGValue,
    height: yoga::YGValue,
    margin_left: yoga::YGValue,
    margin_top: yoga::YGValue,
    margin_right: yoga::YGValue,
    margin_bottom: yoga::YGValue,
    padding_left: yoga::YGValue,
    padding_top: yoga::YGValue,
    padding_right: yoga::YGValue,
    padding_bottom: yoga::YGValue,
    border_left:f32,
    border_top: f32,
    border_right: f32,
    border_bottom: f32,
    align_content: YGAlign,
    align_items: YGAlign,
    justify_content: YGJustify,
    flex_direction: YGFlexDirection,
    flex_wrap: YGWrap,
    align_self: YGAlign,
    position_type: YGPositionType,
    flex_grow: f32,
    flex_shrink: f32,
    flex_basis: yoga::YGValue,
    min_width: yoga::YGValue,
    min_height: yoga::YGValue,
    max_width: yoga::YGValue,
    max_height: yoga::YGValue,
}

// pub type YgNodeP = YgNode;

#[derive(Clone, Debug, Copy, PartialEq, Component)]
pub struct YgNode( pub yoga::YGNodeRef);

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
        self.0.is_null()
    }
    pub fn set_position_type(&self, value: YGPositionType) { 
        yoga::yg_node_style_set_position_type(self.0, value)
    }
    pub fn set_position(&self, edge: YGEdge, position: f32) { 
        yoga::yg_node_style_set_position(self.0, edge, position * 100.0);
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
        yoga::yg_node_style_set_margin(self.0, edge, (value * 100.0).round());
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
        yoga::yg_node_style_set_flex_basis(self.0, (value * 100.0).round()); 
    }
    pub fn set_flex_basis_percent(&self, value: f32) { 
        yoga::yg_node_style_set_flex_basis_percent(self.0, value); 
    }
    pub fn set_flex_basis_auto(&self) { 
        yoga::yg_node_style_set_flex_basis_auto(self.0); 
    }
    pub fn set_flex_grow(&self, value: f32) { 
        yoga::yg_node_style_set_flex_grow(self.0, (value * 100.0).round());
    }
    pub fn set_flex_shrink(&self, value: f32) { 
        yoga::yg_node_style_set_flex_shrink(self.0, (value * 100.0).round()); 
    }

    pub fn set_width(&self, value: f32) { 
        yoga::yg_node_style_set_width(self.0, (value * 100.0).round());
    }

    pub fn set_width_percent(&self, value: f32){ 
        yoga::yg_node_style_set_width_percent(self.0, value);
    }
    pub fn set_width_auto(&self){ 
        yoga::yg_node_style_set_width_auto(self.0);
    }
    pub fn set_height(&self, value: f32){ 
        yoga::yg_node_style_set_height(self.0, (value * 100.0).round());
    }
    pub fn set_height_percent(&self, value: f32){ 
        yoga::yg_node_style_set_height_percent(self.0, value);
    }
    pub fn set_height_auto(&self){ 
        yoga::yg_node_style_set_height_auto(self.0);
    }

    pub fn set_min_width(&self, value: f32){ 
        yoga::yg_node_style_set_min_width(self.0, (value * 100.0).round());
    }
    pub fn set_min_width_percent(&self, value: f32){ 
        yoga::yg_node_style_set_min_width_percent(self.0, value);
    }
    pub fn set_min_height(&self, value: f32){ 
        yoga::yg_node_style_set_min_height(self.0, (value * 100.0).round());
    }
    pub fn set_min_height_percent(&self, value: f32){
        yoga::yg_node_style_set_min_height_percent(self.0, value);
    }

    pub fn set_max_width(&self, value: f32){ 
        yoga::yg_node_style_set_max_width(self.0, (value * 100.0).round());
    }
    pub fn set_max_width_percent(&self, value: f32){
        yoga::yg_node_style_set_max_width_percent(self.0, value);
    }
    pub fn set_max_height(&self, value: f32){ 
        yoga::yg_node_style_set_max_height(self.0, (value * 100.0).round());
    }
    pub fn set_max_height_percent(&self, value: f32){
        yoga::yg_node_style_set_max_height_percent(self.0, value);
    }

    pub fn set_aspect_ratio(&self, value: f32){ 
        yoga::yg_node_style_set_aspect_ratio(self.0, (value * 100.0).round());
    }
    pub fn set_border(&self, edge: YGEdge, value: f32){ 
        yoga::yg_node_style_set_border(self.0, edge, (value * 100.0).round());
    }
    pub fn set_padding(&self, edge: YGEdge, value: f32){ 
        yoga::yg_node_style_set_padding(self.0, edge, (value * 100.0).round());
    }
    pub fn set_padding_percent(&self, edge: YGEdge, value: f32){
        yoga::yg_node_style_set_padding_percent(self.0, edge, value);
    }

    pub fn set_context(&self, context: *mut c_void){
        yoga::yg_node_set_context(self.0, context);
    }
    pub fn set_bind(&self, bind: *mut c_void){
        yoga::yg_node_set_bind(self.0, bind);
    }

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
    pub fn get_top(&self) -> yoga::YGValue {
        yoga::yg_node_style_get_position(self.0, YGEdge::YGEdgeTop)
    }
    pub fn get_left(&self) -> yoga::YGValue {
        yoga::yg_node_style_get_position(self.0, YGEdge::YGEdgeLeft)
    }

    pub fn get_style(&self) -> Style {
        Style{
            left: yoga::yg_node_style_get_position(self.0, YGEdge::YGEdgeLeft),
            right: yoga::yg_node_style_get_position(self.0, YGEdge::YGEdgeRight),
            top: yoga::yg_node_style_get_position(self.0, YGEdge::YGEdgeTop),
            bottom: yoga::yg_node_style_get_position(self.0, YGEdge::YGEdgeBottom),
            width: yoga::yg_node_style_get_width(self.0),
            height: yoga::yg_node_style_get_height(self.0),
            margin_left: yoga::yg_node_style_get_margin(self.0, YGEdge::YGEdgeLeft),
            margin_top: yoga::yg_node_style_get_margin(self.0, YGEdge::YGEdgeTop),
            margin_right: yoga::yg_node_style_get_margin(self.0, YGEdge::YGEdgeRight),
            margin_bottom: yoga::yg_node_style_get_margin(self.0, YGEdge::YGEdgeBottom),
            padding_left: yoga::yg_node_style_get_padding(self.0, YGEdge::YGEdgeLeft),
            padding_top: yoga::yg_node_style_get_padding(self.0, YGEdge::YGEdgeTop),
            padding_right: yoga::yg_node_style_get_padding(self.0, YGEdge::YGEdgeRight),
            padding_bottom: yoga::yg_node_style_get_padding(self.0, YGEdge::YGEdgeBottom),
            border_left: yoga::yg_node_style_get_border(self.0, YGEdge::YGEdgeLeft),
            border_top: yoga::yg_node_style_get_border(self.0, YGEdge::YGEdgeTop),
            border_right: yoga::yg_node_style_get_border(self.0, YGEdge::YGEdgeRight),
            border_bottom: yoga::yg_node_style_get_border(self.0, YGEdge::YGEdgeBottom),
            align_content: yoga::yg_node_style_get_align_content(self.0),
            align_items: yoga::yg_node_style_get_align_items(self.0),
            justify_content: yoga::yg_node_style_get_justify_content(self.0),
            flex_direction: yoga::yg_node_style_get_flex_direction(self.0),
            flex_wrap: yoga::yg_node_style_get_flex_wrap(self.0),
            align_self: yoga::yg_node_style_get_align_self(self.0),
            position_type: yoga::yg_node_style_get_position_type(self.0),
            flex_grow: yoga::yg_node_style_get_flex_grow(self.0),
            flex_shrink: yoga::yg_node_style_get_flex_shrink(self.0),
            flex_basis: yoga::yg_node_style_get_flex_basis(self.0),
            min_width: yoga::yg_node_style_get_min_width(self.0),
            min_height: yoga::yg_node_style_get_min_height(self.0),
            max_width: yoga::yg_node_style_get_max_width(self.0),
            max_height: yoga::yg_node_style_get_max_height(self.0),
        }
    }
    pub fn get_context(&self) -> *mut c_void {
        yoga::yg_node_get_context(self.0)
    }
    pub fn get_bind(&self) -> *mut c_void {
        yoga::yg_node_get_bind(self.0)
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
        yoga::yg_node_calculate_layout(self.0, width * 100.0, height * 100.0, direction);
    }

    pub fn calculate_layout_by_callback(&self, width: f32, height:f32, direction: YGDirection, callback: CallbackFunc, arg: *const c_void) {
        yoga::yg_node_calculate_layout_by_callback(self.0, width * 100.0, height * 100.0, direction, unsafe { std::mem::transmute(callback) }, arg);
    }

    pub fn get_layout(&self) -> Layout {
        Layout{
            left: yoga::yg_node_layout_get_left(self.0)/100.0,
            top: yoga::yg_node_layout_get_top(self.0)/100.0,
            width: yoga::yg_node_layout_get_width(self.0)/100.0,
            height: yoga::yg_node_layout_get_height(self.0)/100.0,
            border_left: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeLeft)/100.0,
            border_top: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeTop)/100.0,
            border_right: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeRight)/100.0,
            border_bottom: yoga::yg_node_layout_get_border(self.0, YGEdge::YGEdgeBottom)/100.0,
            padding_left: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeLeft)/100.0,
            padding_top: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeTop)/100.0,
            padding_right: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeRight)/100.0,
            padding_bottom: yoga::yg_node_layout_get_padding(self.0, YGEdge::YGEdgeBottom)/100.0,
        }
    }

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