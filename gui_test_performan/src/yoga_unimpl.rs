use std::default::Default;
use std::os::raw::{c_void};

use ecs::component::Component;
use map::vecmap::VecMap;

use gui::layout:: {FlexNode, YGCalcCallbackFunc};
pub use gui::layout::{YGAlign, YGDirection, YGDisplay, YGEdge, YGJustify, YGWrap, YGFlexDirection, YGOverflow, YGPositionType, YGUnit};
use gui::component::user::Layout;

#[derive(Clone, Debug, Copy, PartialEq, Component)]
pub struct YgNode();

impl Default for YgNode{
    fn default() -> YgNode{
        YgNode::new()
    }
}

impl FlexNode for YgNode {
    fn new() -> YgNode {
        YgNode()
    }
    fn new_null() -> YgNode {
        YgNode()
    }
    fn is_null(&self) -> bool {
        true
    }
    fn set_position_type(&self, _value: YGPositionType) { 
        
    }
    fn set_position(&self, _edge: YGEdge, _position: f32) { 
        
    }
    fn set_position_percent(&self, _edge: YGEdge, _position: f32) {
        
    }

    fn set_align_content(&self, _value: YGAlign) {
        
    }

    fn set_align_items(&self, _value: YGAlign) { 
        
    }

    fn set_align_self(&self, _value: YGAlign) { 
        
    }
    fn set_flex_direction(&self, _value: YGFlexDirection) { 
        
    }
    fn set_flex_wrap(&self, _value: YGWrap) { 
        
    }
    fn set_justify_content(&self, _value: YGJustify) {
        
    }
    fn set_margin(&self, _edge: YGEdge, _value: f32) { 
        
    }
    fn set_margin_percent(&self, _edge: YGEdge, _value: f32) { 
        
    }
    fn set_margin_auto(&self, _edge: YGEdge) { 
        
    }
    fn set_flex_basis_auto(&self) { 
        
    }
    fn set_overflow(&self, _value: YGOverflow) { 
        
    }

    fn set_display(&self, _value: YGDisplay) { 
        
    }

    fn set_flex(&self, _value: f32) { 
        
    }
    fn set_flex_basis(&self, _value: f32) { 
        
    }
    fn set_flex_basis_percent(&self, _value: f32) { 
        
    }
    fn set_flex_grow(&self, _value: f32) { 
        
    }
    fn set_flex_shrink(&self, _value: f32) { 
        
    }

    fn set_width(&self, _value: f32) { 
        
    }

    fn set_width_percent(&self, _value: f32){ 
        
    }
    fn set_width_auto(&self){ 
        
    }
    fn set_height(&self, _value: f32){ 
        
    }
    fn set_height_percent(&self, _value: f32){ 
        
    }
    fn set_height_auto(&self){ 
        
    }

    fn set_min_width(&self, _value: f32){ 
        
    }
    fn set_min_width_percent(&self, _value: f32){ 
        
    }
    fn set_min_height(&self, _value: f32){ 
        
    }
    fn set_min_height_percent(&self, _value: f32){
        
    }

    fn set_max_width(&self, _value: f32){ 
        
    }
    fn set_max_width_percent(&self, _value: f32){
        
    }
    fn set_max_height(&self, _value: f32){ 
        
    }
    fn set_max_height_percent(&self, _value: f32){
        
    }

    fn set_aspect_ratio(&self, _value: f32){ 
        
    }
    fn set_border(&self, _edge: YGEdge, _value: f32){ 
        
    }
    fn set_padding(&self, _edge: YGEdge, _value: f32){ 
        
    }
    fn set_padding_percent(&self, _edge: YGEdge, _value: f32){
        
    }

    fn set_context(&self, _context: *mut c_void){
        
    }
    fn set_bind(&self, _bind: *mut c_void){
    }
    fn insert_child(&self, _node: YgNode, _index: u32){

    }

    fn remove_child(&self, _node: YgNode){

    }

    fn get_child(&self, _index: u32) -> YgNode {
        YgNode::default()
    }

    fn get_parent(&self) -> YgNode {
        YgNode::default()
    }

    fn get_child_count(&self) -> u32 {
        1
    }
    fn get_context(&self) -> *mut c_void {
        1 as *mut c_void
    }
    fn get_bind(&self) -> *mut c_void {
        1 as *mut c_void
    }
    fn mark_dirty(&self) { 
        
    }
    fn is_dirty(&self) -> bool { 
        false
    }

    fn calculate_layout(&self, _width: f32, _height:f32, _direction: YGDirection){
        
    }

    fn calculate_layout_by_callback(&self, _width: f32, _height:f32, _direction: YGDirection, _callback: YGCalcCallbackFunc<Self>, _callback_args: *const c_void) {
        
    }


    fn get_layout(&self) -> Layout {
        Layout::default()
    }

    fn get_layout_margin(&self, _edge: YGEdge) -> f32 { 
        0.0
    }
    fn get_layout_border(&self, _edge: YGEdge) -> f32 { 
        0.0
    }
    fn get_layout_padding(&self, _edge: YGEdge) -> f32 { 
        0.0
    }

    fn free(&self){ 
        
    }

    fn free_recursive(&self) {
        
    }
}