/**
 * FlexNode的空实现， 可用于测试
*/
use std::default::Default;
use std::os::raw::{c_void};

use ecs::component::Component;
use map::vecmap::VecMap;

use component::user::Layout;

#[derive(Clone, Debug, Copy, PartialEq, Component)]
pub struct YgNode();

impl Default for YgNode{
    fn default() -> YgNode{
        YgNode::new()
    }
}

impl YgNode {
    pub fn new() -> YgNode {
        YgNode()
    }
    pub fn new_null() -> YgNode {
        YgNode()
    }
    pub fn is_null(&self) -> bool {
        true
    }
    pub fn set_position_type(&self, _value: YGPositionType) { 
        
    }
    pub fn set_position(&self, _edge: YGEdge, _position: f32) { 
        
    }
    pub fn set_position_percent(&self, _edge: YGEdge, _position: f32) {
        
    }

    pub fn set_align_content(&self, _value: YGAlign) {
        
    }

    pub fn set_align_items(&self, _value: YGAlign) { 
        
    }

    pub fn set_align_self(&self, _value: YGAlign) { 
        
    }
    pub fn set_flex_direction(&self, _value: YGFlexDirection) { 
        
    }
    pub fn set_flex_wrap(&self, _value: YGWrap) { 
        
    }
    pub fn set_justify_content(&self, _value: YGJustify) {
        
    }
    pub fn set_margin(&self, _edge: YGEdge, _value: f32) { 
        
    }
    pub fn set_margin_percent(&self, _edge: YGEdge, _value: f32) { 
        
    }
    pub fn set_margin_auto(&self, _edge: YGEdge) { 
        
    }
    pub fn set_flex_basis_auto(&self) { 
        
    }
    pub fn set_overflow(&self, _value: YGOverflow) { 
        
    }

    pub fn set_display(&self, _value: YGDisplay) { 
        
    }

    pub fn set_flex(&self, _value: f32) { 
        
    }
    pub fn set_flex_basis(&self, _value: f32) { 
        
    }
    pub fn set_flex_basis_percent(&self, _value: f32) { 
        
    }
    pub fn set_flex_grow(&self, _value: f32) { 
        
    }
    pub fn set_flex_shrink(&self, _value: f32) { 
        
    }

    pub fn set_width(&self, _value: f32) { 
        
    }

    pub fn set_width_percent(&self, _value: f32){ 
        
    }
    pub fn set_width_auto(&self){ 
        
    }
    pub fn set_height(&self, _value: f32){ 
        
    }
    pub fn set_height_percent(&self, _value: f32){ 
        
    }
    pub fn set_height_auto(&self){ 
        
    }

    pub fn set_min_width(&self, _value: f32){ 
        
    }
    pub fn set_min_width_percent(&self, _value: f32){ 
        
    }
    pub fn set_min_height(&self, _value: f32){ 
        
    }
    pub fn set_min_height_percent(&self, _value: f32){
        
    }

    pub fn set_max_width(&self, _value: f32){ 
        
    }
    pub fn set_max_width_percent(&self, _value: f32){
        
    }
    pub fn set_max_height(&self, _value: f32){ 
        
    }
    pub fn set_max_height_percent(&self, _value: f32){
        
    }

    pub fn set_aspect_ratio(&self, _value: f32){ 
        
    }
    pub fn set_border(&self, _edge: YGEdge, _value: f32){ 
        
    }
    pub fn set_padding(&self, _edge: YGEdge, _value: f32){ 
        
    }
    pub fn set_padding_percent(&self, _edge: YGEdge, _value: f32){
        
    }

    pub fn set_context(&self, _context: *mut c_void){
        
    }
    pub fn set_bind(&self, _bind: *mut c_void){
    }
    pub fn insert_child(&self, _node: YgNode, _index: u32){

    }

    pub fn remove_child(&self, _node: YgNode){

    }

    pub fn get_child(&self, _index: u32) -> YgNode {
        YgNode::default()
    }

    pub fn get_parent(&self) -> YgNode {
        YgNode::default()
    }

    pub fn get_child_count(&self) -> u32 {
        0
    }
    pub fn get_context(&self) -> *mut c_void {
        1 as *mut c_void
    }
    pub fn get_bind(&self) -> *mut c_void {
        1 as *mut c_void
    }
    pub fn mark_dirty(&self) { 
        
    }
    pub fn is_dirty(&self) -> bool { 
        false
    }

    pub fn calculate_layout(&self, _width: f32, _height:f32, _direction: YGDirection){
        
    }

    pub fn calculate_layout_by_callback(&self, _width: f32, _height:f32, _direction: YGDirection, _callback: YGCalcCallbackFunc, _callback_args: *const c_void) {
        
    }


    pub fn get_layout(&self) -> Layout {
        Layout::default()
    }

    pub fn get_layout_margin(&self, _edge: YGEdge) -> f32 { 
        0.0
    }
    pub fn get_layout_border(&self, _edge: YGEdge) -> f32 { 
        0.0
    }
    pub fn get_layout_padding(&self, _edge: YGEdge) -> f32 { 
        0.0
    }

    pub fn free(&self){ 
        
    }

    pub fn free_recursive(&self) {
        
    }
}