use std::default::Default;
use std::os::raw::{c_void};

#[derive(Clone, Debug, Default)]
pub struct Layout{
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub border: f32,
    pub padding_left: f32,
    pub padding_top: f32,
}
// pub type YgNodeP = YgNode;

#[derive(Clone, Debug, Copy)]
pub struct YgNode;

impl Default for YgNode{
    fn default() -> YgNode{
        YgNode::new()
    }
}

impl YgNode {
    pub fn new() -> YgNode {
        YgNode
    }

    pub fn set_position_type(&self, _value: YGPositionType) { 
        unimplemented!()
    }
    pub fn set_position(&self, _edge: YGEdge, _position: f32) { 
        unimplemented!()
    }
    pub fn set_position_percent(&self, _edge: YGEdge, _position: f32) {
        unimplemented!()
    }

    pub fn set_align_content(&self, _value: YGAlign) {
        unimplemented!()
    }

    pub fn set_align_items(&self, _value: YGAlign) { 
        unimplemented!()
    }

    pub fn set_align_self(&self, _value: YGAlign) { 
        unimplemented!()
    }
    pub fn set_flex_direction(&self, _value: YGFlexDirection) { 
        unimplemented!()
    }
    pub fn set_flex_wrap(&self, _value: YGWrap) { 
        unimplemented!()
    }
    pub fn set_justify_content(&self, _value: YGJustify) {
        unimplemented!()
    }
    pub fn set_margin(&self, _edge: YGEdge, _value: f32) { 
        unimplemented!()
    }
    pub fn set_margin_percent(&self, _edge: YGEdge, _value: f32) { 
        unimplemented!()
    }
    pub fn set_margin_auto(&self, _edge: YGEdge) { 
        unimplemented!()
    }

    pub fn set_overflow(&self, _value: YGOverflow) { 
        unimplemented!()
    }

    pub fn set_display(&self, _value: YGDisplay) { 
        unimplemented!()
    }

    pub fn set_flex(&self, _value: f32) { 
        unimplemented!()
    }
    pub fn set_flex_basis(&self, _value: f32) { 
        unimplemented!()
    }
    pub fn set_flex_basis_percent(&self, _value: f32) { 
        unimplemented!()
    }
    pub fn set_flex_grow(&self, _value: f32) { 
        unimplemented!()
    }
    pub fn set_flex_shrink(&self, _value: f32) { 
        unimplemented!()
    }

    pub fn set_width(&self, _value: f32) { 
        unimplemented!()
    }

    pub fn set_width_percent(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_width_auto(&self){ 
        unimplemented!()
    }
    pub fn set_height(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_height_percent(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_height_auto(&self){ 
        unimplemented!()
    }

    pub fn set_min_width(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_min_width_percent(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_min_height(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_min_height_percent(&self, _value: f32){
        unimplemented!()
    }

    pub fn set_max_width(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_max_width_percent(&self, _value: f32){
        unimplemented!()
    }
    pub fn set_max_height(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_max_height_percent(&self, _value: f32){
        unimplemented!()
    }

    pub fn set_aspect_ratio(&self, _value: f32){ 
        unimplemented!()
    }
    pub fn set_border(&self, _edge: YGEdge, _value: f32){ 
        unimplemented!()
    }
    pub fn set_padding(&self, _edge: YGEdge, _value: f32){ 
        unimplemented!()
    }
    pub fn set_padding_percent(&self, _edge: YGEdge, _value: f32){
        unimplemented!()
    }

    pub fn set_context(&self, _context: *mut c_void){
        
    }

    pub fn insert_child(&self, _node: YgNode, _index: u32){

    }

    pub fn remove_child(&self, _node: YgNode){

    }

    pub fn get_child(&self, _index: u32) -> YgNode {
        unimplemented!()
    }

    pub fn get_parent(&self) -> YgNode {
        unimplemented!()
    }

    pub fn get_child_count(&self) -> u32 {
        unimplemented!()
    }

    pub fn mark_dirty(&self) { 
        unimplemented!()
    }
    pub fn is_dirty(&self) -> bool { 
        unimplemented!()
    }

    pub fn calculate_layout(&self, _width: f32, _height:f32, _direction: YGDirection){
        unimplemented!()
    }

    pub fn calculate_layout_by_callback(&self, _width: f32, _height:f32, _direction: YGDirection, _callback: YGCalcCallbackFunc, callbackArgs: *const c_void) {
        unimplemented!()
    }


    pub fn get_layout(&self) -> Layout {
        unimplemented!()
    }

    pub fn get_layout_margin(&self, _edge: YGEdge) -> f32 { 
        unimplemented!()
    }
    pub fn get_layout_border(&self, _edge: YGEdge) -> f32 { 
        unimplemented!()
    }
    pub fn get_layout_padding(&self, _edge: YGEdge) -> f32 { 
        unimplemented!()
    }

    pub fn free(&self){ 
        unimplemented!()
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGAlign {
    YGAlignAuto = 0,
    YGAlignFlexStart = 1,
    YGAlignCenter = 2,
    YGAlignFlexEnd = 3,
    YGAlignStretch = 4,
    YGAlignBaseline = 5,
    YGAlignSpaceBetween = 6,
    YGAlignSpaceAround = 7,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGDimension {
    YGDimensionWidth = 0,
    YGDimensionHeight = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGDirection {
    YGDirectionInherit = 0,
    YGDirectionLTR = 1,
    YGDirectionRTL = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGDisplay {
    YGDisplayFlex = 0,
    YGDisplayNone = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGEdge {
    YGEdgeLeft = 0,
    YGEdgeTop = 1,
    YGEdgeRight = 2,
    YGEdgeBottom = 3,
    YGEdgeStart = 4,
    YGEdgeEnd = 5,
    YGEdgeHorizontal = 6,
    YGEdgeVertical = 7,
    YGEdgeAll = 8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGExperimentalFeature {
    YGExperimentalFeatureWebFlexBasis = 0,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGFlexDirection {
    YGFlexDirectionColumn = 0,
    YGFlexDirectionColumnReverse = 1,
    YGFlexDirectionRow = 2,
    YGFlexDirectionRowReverse = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGJustify {
    YGJustifyFlexStart = 0,
    YGJustifyCenter = 1,
    YGJustifyFlexEnd = 2,
    YGJustifySpaceBetween = 3,
    YGJustifySpaceAround = 4,
    YGJustifySpaceEvenly = 5,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGLogLevel {
    YGLogLevelError = 0,
    YGLogLevelWarn = 1,
    YGLogLevelInfo = 2,
    YGLogLevelDebug = 3,
    YGLogLevelVerbose = 4,
    YGLogLevelFatal = 5,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGMeasureMode {
    YGMeasureModeUndefined = 0,
    YGMeasureModeExactly = 1,
    YGMeasureModeAtMost = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGNodeType {
    YGNodeTypeDefault = 0,
    YGNodeTypeText = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGOverflow {
    YGOverflowVisible = 0,
    YGOverflowHidden = 1,
    YGOverflowScroll = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGPositionType {
    YGPositionTypeRelative = 0,
    YGPositionTypeAbsolute = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGPrintOptions {
    YGPrintOptionsLayout = 1,
    YGPrintOptionsStyle = 2,
    YGPrintOptionsChildren = 4,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGUnit {
    YGUnitUndefined = 0,
    YGUnitPoint = 1,
    YGUnitPercent = 2,
    YGUnitAuto = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum YGWrap {
    YGWrapNoWrap = 0,
    YGWrapWrap = 1,
    YGWrapWrapReverse = 2,
}

pub type YGCalcCallbackFunc = unsafe extern "C" fn(args: *const c_void, context: *const c_void);