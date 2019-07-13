use std::default::Default;
use std::fmt::Debug;
use std::os::raw::{c_void};
use ecs::component::Component;
use component::user::Layout;

pub trait FlexNode: Default + Clone + Debug + Copy + PartialEq + Component {
    fn new() -> Self;
    fn new_null() -> Self; 
    fn is_null(&self) -> bool;
    fn set_position_type(&self, value: YGPositionType);
    fn set_position(&self, edge: YGEdge, position: f32);
    fn set_position_percent(&self, edge: YGEdge, position: f32);
    fn set_align_content(&self, value: YGAlign);
    fn set_align_items(&self, value: YGAlign);
    fn set_align_self(&self, value: YGAlign);
    fn set_flex_direction(&self, value: YGFlexDirection);
    fn set_flex_wrap(&self, value: YGWrap);
    fn set_justify_content(&self, value: YGJustify);
    fn set_margin(&self, edge: YGEdge, value: f32);
    fn set_margin_percent(&self, edge: YGEdge, value: f32);
    fn set_margin_auto(&self, edge: YGEdge);
    fn set_flex_basis_auto(&self);
    fn set_overflow(&self, value: YGOverflow);
    fn set_display(&self, value: YGDisplay);
    fn set_flex(&self, value: f32);
    fn set_flex_basis(&self, value: f32);
    fn set_flex_basis_percent(&self, value: f32);
    fn set_flex_grow(&self, value: f32);
    fn set_flex_shrink(&self, value: f32);
    fn set_width(&self, value: f32);
    fn set_width_percent(&self, value: f32);
    fn set_width_auto(&self);
    fn set_height(&self, value: f32);
    fn set_height_percent(&self, value: f32);
    fn set_height_auto(&self);
    fn set_min_width(&self, value: f32);
    fn set_min_width_percent(&self, value: f32);
    fn set_min_height(&self, value: f32);
    fn set_min_height_percent(&self, value: f32);
    fn set_max_width(&self, value: f32);
    fn set_max_width_percent(&self, value: f32);
    fn set_max_height(&self, value: f32);
    fn set_max_height_percent(&self, value: f32);
    fn set_aspect_ratio(&self, value: f32);
    fn set_border(&self, edge: YGEdge, value: f32);
    fn set_padding(&self, edge: YGEdge, value: f32);
    fn set_padding_percent(&self, edge: YGEdge, value: f32);
    fn set_context(&self, _context: *mut c_void);
    fn set_bind(&self, _bind: *mut c_void);
    fn insert_child(&self, _node: Self, _index: u32);
    fn remove_child(&self, _node: Self);
    fn get_child(&self, _index: u32) -> Self;
    fn get_parent(&self) -> Self;
    fn get_child_count(&self) -> u32;
    fn get_context(&self) -> *mut c_void;
    fn get_bind(&self) -> *mut c_void;
    fn mark_dirty(&self);
    fn is_dirty(&self) -> bool;
    fn calculate_layout(&self, _width: f32, _height:f32, _direction: YGDirection);
    fn calculate_layout_by_callback(&self, _width: f32, _height:f32, _direction: YGDirection, _callback: YGCalcCallbackFunc<Self>, _callback_args: *const c_void);
    fn get_layout(&self) -> Layout;
    fn get_layout_margin(&self, edge: YGEdge) -> f32;
    fn get_layout_border(&self, edge: YGEdge) -> f32;
    fn get_layout_padding(&self, edge: YGEdge) -> f32;
    fn get_style_width_unit(&self) -> YGUnit;
    fn get_style_justify(&self) -> YGJustify;
    fn get_style_align_content(&self) -> YGAlign;
    fn get_style_align_items(&self) -> YGAlign;
    fn get_style_width_value(&self) -> f32;
    fn free(&self);
    fn free_recursive(&self);
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
pub enum FlexNodeType {
    FlexNodeTypeDefault = 0,
    FlexNodeTypeText = 1,
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

pub type YGCalcCallbackFunc<T> = unsafe extern "C" fn(node: T, args: *const c_void);