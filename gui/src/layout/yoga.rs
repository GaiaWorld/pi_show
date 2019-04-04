#![allow(unused_attributes)]
#![allow(non_camel_case_types)]
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_float, c_void};

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

pub type __builtin_va_list = *mut c_char;
pub type __gnuc_va_list = __builtin_va_list;
pub type va_list = __gnuc_va_list;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct YGSize {
    pub width: c_float,
    pub height: c_float,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct YGValue {
    pub value: c_float,
    pub unit: YGUnit,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct YGConfig {
    _unused: [u8; 0],
}
pub type YGConfigRef = *mut YGConfig;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct YGNode {
    _unused: [u8; 0],
}
pub type YGNodeRef = *mut YGNode;

pub type YGPrintFunc = Option<unsafe extern "C" fn(node: YGNodeRef)>;

pub type YGDirtiedFunc = Option<unsafe extern "C" fn(node: YGNodeRef)>;

pub type YGBaselineFunc =
    Option<unsafe extern "C" fn(node: YGNodeRef, width: c_float, height: c_float) -> c_float>;

pub type YGCloneNodeFunc = Option<
    unsafe extern "C" fn(oldNode: YGNodeRef, owner: YGNodeRef, childIndex: c_int) -> YGNodeRef,
>;

pub type YGCalcCallbackFunc = unsafe extern "C" fn(args: *const c_void, context: *const c_void);

pub type YGMeasureFunc = Option<
    unsafe extern "C" fn(
        node: YGNodeRef,
        width: c_float,
        widthMode: YGMeasureMode,
        height: c_float,
        heightMode: YGMeasureMode,
    ) -> YGSize,
>;

pub type YGLogger = Option<
    unsafe extern "C" fn(
        config: YGConfigRef,
        node: YGNodeRef,
        level: YGLogLevel,
        format: *const c_char,
        args: va_list,
    ) -> c_int,
>;

#[link_args = "yoga.bc"]
extern "C" {
    #[link_name = "\u{1}YGValueUndefined"]
    pub static mut YGValueUndefined: YGValue;
    #[link_name = "\u{1}YGValueAuto"]
    pub static mut YGValueAuto: YGValue;

    fn YGAlignToString(value: YGAlign) -> *const c_char;
    fn YGDimensionToString(value: YGDimension) -> *const c_char;
    fn YGDirectionToString(value: YGDirection) -> *const c_char;
    fn YGDisplayToString(value: YGDisplay) -> *const c_char;
    fn YGEdgeToString(value: YGEdge) -> *const c_char;
    fn YGExperimentalFeatureToString(value: YGExperimentalFeature) -> *const c_char;
    fn YGFlexDirectionToString(value: YGFlexDirection) -> *const c_char;
    fn YGJustifyToString(value: YGJustify) -> *const c_char;
    fn YGLogLevelToString(value: YGLogLevel) -> *const c_char;
    fn YGMeasureModeToString(value: YGMeasureMode) -> *const c_char;
    fn YGNodeTypeToString(value: YGNodeType) -> *const c_char;
    fn YGOverflowToString(value: YGOverflow) -> *const c_char;
    fn YGPositionTypeToString(value: YGPositionType) -> *const c_char;
    fn YGPrintOptionsToString(value: YGPrintOptions) -> *const c_char;
    fn YGUnitToString(value: YGUnit) -> *const c_char;
    fn YGWrapToString(value: YGWrap) -> *const c_char;
    fn YGNodeNew() -> YGNodeRef;
    fn YGNodeNewWithConfig(config: YGConfigRef) -> YGNodeRef;
    fn YGNodeClone(node: YGNodeRef) -> YGNodeRef;
    fn YGNodeFree(node: YGNodeRef);
    fn YGNodeFreeRecursive(node: YGNodeRef);
    fn YGNodeReset(node: YGNodeRef);
    fn YGNodeGetInstanceCount() -> i32;
    fn YGNodeInsertChild(node: YGNodeRef, child: YGNodeRef, index: u32);
    fn YGNodeInsertSharedChild(node: YGNodeRef, child: YGNodeRef, index: u32);
    fn YGNodeRemoveChild(node: YGNodeRef, child: YGNodeRef);
    fn YGNodeRemoveAllChildren(node: YGNodeRef);
    fn YGNodeGetChild(node: YGNodeRef, index: u32) -> YGNodeRef;
    fn YGNodeGetOwner(node: YGNodeRef) -> YGNodeRef;
    fn YGNodeGetParent(node: YGNodeRef) -> YGNodeRef;
    fn YGNodeGetChildCount(node: YGNodeRef) -> u32;
    fn YGNodeSetChildren(owner: YGNodeRef, children: *const YGNodeRef, count: u32);
    fn YGNodeCalculateLayout(
        node: YGNodeRef,
        availableWidth: c_float,
        availableHeight: c_float,
        ownerDirection: YGDirection,
    );
    fn YGNodeCalculateLayoutByCallback(
        node: YGNodeRef,
        availableWidth: c_float,
        availableHeight: c_float,
        ownerDirection: YGDirection,
        calcCallback: YGCalcCallbackFunc,
        callbackArgs: *const c_void,
    );
    fn YGNodeMarkDirty(node: YGNodeRef);
    fn YGNodeMarkDirtyAndPropogateToDescendants(node: YGNodeRef);
    fn YGNodePrint(node: YGNodeRef, options: YGPrintOptions);
    fn YGFloatIsUndefined(value: c_float) -> bool;
    fn YGNodeCanUseCachedMeasurement(
        widthMode: YGMeasureMode,
        width: c_float,
        heightMode: YGMeasureMode,
        height: c_float,
        lastWidthMode: YGMeasureMode,
        lastWidth: c_float,
        lastHeightMode: YGMeasureMode,
        lastHeight: c_float,
        lastComputedWidth: c_float,
        lastComputedHeight: c_float,
        marginRow: c_float,
        marginColumn: c_float,
        config: YGConfigRef,
    ) -> bool;
    fn YGNodeCopyStyle(dstNode: YGNodeRef, srcNode: YGNodeRef);
    fn YGNodeGetContext(node: YGNodeRef) -> *mut c_void;
    fn YGNodeSetContext(node: YGNodeRef, context: *mut c_void);
    fn YGConfigSetPrintTreeFlag(config: YGConfigRef, enabled: bool);
    fn YGNodeGetMeasureFunc(node: YGNodeRef) -> YGMeasureFunc;
    fn YGNodeSetMeasureFunc(node: YGNodeRef, measureFunc: YGMeasureFunc);
    fn YGNodeGetBaselineFunc(node: YGNodeRef) -> YGBaselineFunc;
    fn YGNodeSetBaselineFunc(node: YGNodeRef, baselineFunc: YGBaselineFunc);
    fn YGNodeGetDirtiedFunc(node: YGNodeRef) -> YGDirtiedFunc;
    fn YGNodeSetDirtiedFunc(node: YGNodeRef, dirtiedFunc: YGDirtiedFunc);
    fn YGNodeGetPrintFunc(node: YGNodeRef) -> YGPrintFunc;
    fn YGNodeSetPrintFunc(node: YGNodeRef, printFunc: YGPrintFunc);
    fn YGNodeGetHasNewLayout(node: YGNodeRef) -> bool;
    fn YGNodeSetHasNewLayout(node: YGNodeRef, hasNewLayout: bool);
    fn YGNodeGetNodeType(node: YGNodeRef) -> YGNodeType;
    fn YGNodeSetNodeType(node: YGNodeRef, nodeType: YGNodeType);
    fn YGNodeIsDirty(node: YGNodeRef) -> bool;
    fn YGNodeLayoutGetDidUseLegacyFlag(node: YGNodeRef) -> bool;
    fn YGNodeStyleGetDirection(node: YGNodeRef) -> YGDirection;
    fn YGNodeStyleSetDirection(node: YGNodeRef, direction: YGDirection);
    fn YGNodeStyleGetFlexDirection(node: YGNodeRef) -> YGFlexDirection;
    fn YGNodeStyleSetFlexDirection(node: YGNodeRef, flexDirection: YGFlexDirection);
    fn YGNodeStyleGetJustifyContent(node: YGNodeRef) -> YGJustify;
    fn YGNodeStyleSetJustifyContent(node: YGNodeRef, justifyContent: YGJustify);
    fn YGNodeStyleGetAlignContent(node: YGNodeRef) -> YGAlign;
    fn YGNodeStyleSetAlignContent(node: YGNodeRef, alignContent: YGAlign);
    fn YGNodeStyleGetAlignItems(node: YGNodeRef) -> YGAlign;
    fn YGNodeStyleSetAlignItems(node: YGNodeRef, alignItems: YGAlign);
    fn YGNodeStyleGetAlignSelf(node: YGNodeRef) -> YGAlign;
    fn YGNodeStyleSetAlignSelf(node: YGNodeRef, alignSelf: YGAlign);
    fn YGNodeStyleGetPositionType(node: YGNodeRef) -> YGPositionType;
    fn YGNodeStyleSetPositionType(node: YGNodeRef, positionType: YGPositionType);
    fn YGNodeStyleGetFlexWrap(node: YGNodeRef) -> YGWrap;
    fn YGNodeStyleSetFlexWrap(node: YGNodeRef, flexWrap: YGWrap);
    fn YGNodeStyleGetOverflow(node: YGNodeRef) -> YGOverflow;
    fn YGNodeStyleSetOverflow(node: YGNodeRef, overflow: YGOverflow);
    fn YGNodeStyleGetDisplay(node: YGNodeRef) -> YGDisplay;
    fn YGNodeStyleSetDisplay(node: YGNodeRef, display: YGDisplay);
    fn YGNodeStyleGetFlex(node: YGNodeRef) -> c_float;
    fn YGNodeStyleSetFlex(node: YGNodeRef, flex: c_float);
    fn YGNodeStyleGetFlexGrow(node: YGNodeRef) -> c_float;
    fn YGNodeStyleSetFlexGrow(node: YGNodeRef, flexGrow: c_float);
    fn YGNodeStyleGetFlexShrink(node: YGNodeRef) -> c_float;
    fn YGNodeStyleSetFlexShrink(node: YGNodeRef, flexShrink: c_float);
    fn YGNodeStyleGetFlexBasis(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetFlexBasis(node: YGNodeRef, flexBasis: c_float);
    fn YGNodeStyleSetFlexBasisPercent(node: YGNodeRef, flexBasis: c_float);
    fn YGNodeStyleSetFlexBasisAuto(node: YGNodeRef);
    fn YGNodeStyleGetPosition(node: YGNodeRef, edge: YGEdge) -> YGValue;
    fn YGNodeStyleSetPosition(node: YGNodeRef, edge: YGEdge, position: c_float);
    fn YGNodeStyleSetPositionPercent(node: YGNodeRef, edge: YGEdge, position: c_float);
    fn YGNodeStyleGetMargin(node: YGNodeRef, edge: YGEdge) -> YGValue;
    fn YGNodeStyleSetMargin(node: YGNodeRef, edge: YGEdge, margin: c_float);
    fn YGNodeStyleSetMarginPercent(node: YGNodeRef, edge: YGEdge, margin: c_float);
    fn YGNodeStyleSetMarginAuto(node: YGNodeRef, edge: YGEdge);
    fn YGNodeStyleGetPadding(node: YGNodeRef, edge: YGEdge) -> YGValue;
    fn YGNodeStyleSetPadding(node: YGNodeRef, edge: YGEdge, padding: c_float);
    fn YGNodeStyleSetPaddingPercent(node: YGNodeRef, edge: YGEdge, padding: c_float);
    fn YGNodeStyleGetBorder(node: YGNodeRef, edge: YGEdge) -> c_float;
    fn YGNodeStyleSetBorder(node: YGNodeRef, edge: YGEdge, border: c_float);
    fn YGNodeStyleGetWidth(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetWidth(node: YGNodeRef, width: c_float);
    fn YGNodeStyleSetWidthPercent(node: YGNodeRef, width: c_float);
    fn YGNodeStyleSetWidthAuto(node: YGNodeRef);
    fn YGNodeStyleGetHeight(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetHeight(node: YGNodeRef, height: c_float);
    fn YGNodeStyleSetHeightPercent(node: YGNodeRef, height: c_float);
    fn YGNodeStyleSetHeightAuto(node: YGNodeRef);
    fn YGNodeStyleGetMinWidth(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetMinWidth(node: YGNodeRef, minWidth: c_float);
    fn YGNodeStyleSetMinWidthPercent(node: YGNodeRef, minWidth: c_float);
    fn YGNodeStyleGetMinHeight(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetMinHeight(node: YGNodeRef, minHeight: c_float);
    fn YGNodeStyleSetMinHeightPercent(node: YGNodeRef, minHeight: c_float);
    fn YGNodeStyleGetMaxWidth(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetMaxWidth(node: YGNodeRef, maxWidth: c_float);
    fn YGNodeStyleSetMaxWidthPercent(node: YGNodeRef, maxWidth: c_float);
    fn YGNodeStyleGetMaxHeight(node: YGNodeRef) -> YGValue;
    fn YGNodeStyleSetMaxHeight(node: YGNodeRef, maxHeight: c_float);
    fn YGNodeStyleSetMaxHeightPercent(node: YGNodeRef, maxHeight: c_float);
    fn YGNodeStyleGetAspectRatio(node: YGNodeRef) -> c_float;
    fn YGNodeStyleSetAspectRatio(node: YGNodeRef, aspectRatio: c_float);
    fn YGNodeLayoutGetLeft(node: YGNodeRef) -> c_float;
    fn YGNodeLayoutGetTop(node: YGNodeRef) -> c_float;
    fn YGNodeLayoutGetRight(node: YGNodeRef) -> c_float;
    fn YGNodeLayoutGetBottom(node: YGNodeRef) -> c_float;
    fn YGNodeLayoutGetWidth(node: YGNodeRef) -> c_float;
    fn YGNodeLayoutGetHeight(node: YGNodeRef) -> c_float;
    fn YGNodeLayoutGetDirection(node: YGNodeRef) -> YGDirection;
    fn YGNodeLayoutGetHadOverflow(node: YGNodeRef) -> bool;
    fn YGNodeLayoutGetDidLegacyStretchFlagAffectLayout(node: YGNodeRef) -> bool;
    fn YGNodeLayoutGetMargin(node: YGNodeRef, edge: YGEdge) -> c_float;
    fn YGNodeLayoutGetBorder(node: YGNodeRef, edge: YGEdge) -> c_float;
    fn YGNodeLayoutGetPadding(node: YGNodeRef, edge: YGEdge) -> c_float;
    fn YGConfigSetLogger(config: YGConfigRef, logger: YGLogger);
    fn YGLog(node: YGNodeRef, level: YGLogLevel, message: *const c_char, ...);
    fn YGLogWithConfig(config: YGConfigRef, level: YGLogLevel, format: *const c_char, ...);
    fn YGAssert(condition: bool, message: *const c_char);
    fn YGAssertWithNode(node: YGNodeRef, condition: bool, message: *const c_char);
    fn YGAssertWithConfig(config: YGConfigRef, condition: bool, message: *const c_char);
    fn YGConfigSetPointScaleFactor(config: YGConfigRef, pixelsInPoint: c_float);
    fn YGConfigSetShouldDiffLayoutWithoutLegacyStretchBehaviour(
        config: YGConfigRef,
        shouldDiffLayout: bool,
    );
    fn YGConfigSetUseLegacyStretchBehaviour(
        config: YGConfigRef,
        useLegacyStretchBehaviour: bool,
    );
    fn YGConfigNew() -> YGConfigRef;
    fn YGConfigFree(config: YGConfigRef);
    fn YGConfigCopy(dest: YGConfigRef, src: YGConfigRef);
    fn YGConfigGetInstanceCount() -> i32;
    fn YGConfigSetExperimentalFeatureEnabled(
        config: YGConfigRef,
        feature: YGExperimentalFeature,
        enabled: bool,
    );
    fn YGConfigIsExperimentalFeatureEnabled(
        config: YGConfigRef,
        feature: YGExperimentalFeature,
    ) -> bool;
    fn YGConfigSetUseWebDefaults(config: YGConfigRef, enabled: bool);
    fn YGConfigGetUseWebDefaults(config: YGConfigRef) -> bool;
    fn YGConfigSetCloneNodeFunc(config: YGConfigRef, callback: YGCloneNodeFunc);
    fn YGConfigGetDefault() -> YGConfigRef;
    fn YGConfigSetContext(config: YGConfigRef, context: *mut c_void);
    fn YGConfigGetContext(config: YGConfigRef) -> *mut c_void;
    fn YGRoundValueToPixelGrid(
        value: c_float,
        pointScaleFactor: c_float,
        forceCeil: bool,
        forceFloor: bool,
    ) -> c_float;
}
        
pub fn yg_align_to_string(value: YGAlign) -> String {
    unsafe {
        CStr::from_ptr(YGAlignToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_dimension_to_string(value: YGDimension) -> String {
	unsafe {
        CStr::from_ptr(YGDimensionToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_direction_to_string(value: YGDirection) -> String {
	unsafe {
        CStr::from_ptr(YGDirectionToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_display_to_string(value: YGDisplay) -> String {
	unsafe {
        CStr::from_ptr(YGDisplayToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_edge_to_string(value: YGEdge) -> String {
	unsafe {
        CStr::from_ptr(YGEdgeToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_experimental_feature_to_string(value: YGExperimentalFeature) -> String {
	unsafe {
        CStr::from_ptr(YGExperimentalFeatureToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_flex_direction_to_string(value: YGFlexDirection) -> String {
	unsafe {
        CStr::from_ptr(YGFlexDirectionToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_justify_to_string(value: YGJustify) -> String {
	unsafe {
        CStr::from_ptr(YGJustifyToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_log_level_to_string(value: YGLogLevel) -> String {
	unsafe {
        CStr::from_ptr(YGLogLevelToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_measure_mode_to_string(value: YGMeasureMode) -> String {
	unsafe {
        CStr::from_ptr(YGMeasureModeToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_node_type_to_string(value: YGNodeType) -> String {
	unsafe {
        CStr::from_ptr(YGNodeTypeToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_overflow_to_string(value: YGOverflow) -> String {
	unsafe {
        CStr::from_ptr(YGOverflowToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_position_type_to_string(value: YGPositionType) -> String {
	unsafe {
        CStr::from_ptr(YGPositionTypeToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_print_options_to_string(value: YGPrintOptions) -> String {
	unsafe {
        CStr::from_ptr(YGPrintOptionsToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_unit_to_string(value: YGUnit) -> String {
	unsafe {
        CStr::from_ptr(YGUnitToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_wrap_to_string(value: YGWrap) -> String {
	unsafe {
        CStr::from_ptr(YGWrapToString(value)).to_string_lossy().into_owned()
	}
}

pub fn yg_node_new() -> YGNodeRef {
	unsafe {
        YGNodeNew()
	}
}

pub fn yg_node_new_with_config(config: YGConfigRef) -> YGNodeRef {
	unsafe {
        YGNodeNewWithConfig(config)
	}
}

pub fn yg_node_clone(node: YGNodeRef) -> YGNodeRef {
	unsafe {
        YGNodeClone(node)
	}
}

pub fn yg_node_free(node: YGNodeRef) {
	unsafe {
        YGNodeFree(node);
	}
}

pub fn yg_node_free_recursive(node: YGNodeRef) {
	unsafe {
        YGNodeFreeRecursive(node);
	}
}

pub fn yg_node_reset(node: YGNodeRef) {
	unsafe {
        YGNodeReset(node);
	}
}

pub fn yg_node_get_instance_count() -> i32 {
	unsafe {
        YGNodeGetInstanceCount()
	}
}

pub fn yg_node_insert_child(node: YGNodeRef, child: YGNodeRef, index: u32) {
	unsafe {
        YGNodeInsertChild(node, child, index);
	}
}

pub fn yg_node_insert_shared_child(node: YGNodeRef, child: YGNodeRef, index: u32) {
	unsafe {
        YGNodeInsertSharedChild(node, child, index);
	}
}

pub fn yg_node_remove_child(node: YGNodeRef, child: YGNodeRef) {
	unsafe {
        YGNodeRemoveChild(node, child);
	}
}

pub fn yg_node_remove_all_children(node: YGNodeRef) {
	unsafe {
        YGNodeRemoveAllChildren(node);
	}
}

pub fn yg_node_get_child(node: YGNodeRef, index: u32) -> YGNodeRef {
	unsafe {
        YGNodeGetChild(node, index)
	}
}

pub fn yg_node_get_owner(node: YGNodeRef) -> YGNodeRef {
	unsafe {
        YGNodeGetOwner(node)
	}
}

pub fn yg_node_get_parent(node: YGNodeRef) -> YGNodeRef {
	unsafe {
        YGNodeGetParent(node)
	}
}

pub fn yg_node_get_child_count(node: YGNodeRef) -> u32 {
	unsafe {
        YGNodeGetChildCount(node)
	}
}

pub fn yg_node_set_children(owner: YGNodeRef, children: *const YGNodeRef, count: u32) {
	unsafe {
        YGNodeSetChildren(owner, children, count);
	}
}

pub fn yg_node_calculate_layout(
    node: YGNodeRef,
    available_width: f32,
    available_height: f32,
    owner_direction: YGDirection,
) {
	unsafe {
        YGNodeCalculateLayout(node, available_width, available_height, owner_direction)
	}
}

pub fn yg_node_calculate_layout_by_callback(
    node: YGNodeRef,
    available_width: f32,
    available_height: f32,
    owner_direction: YGDirection,
    callback: YGCalcCallbackFunc,
    args: *const c_void,
) {
    unsafe {
        YGNodeCalculateLayoutByCallback(node, available_width, available_height, owner_direction, callback, args);
    }
}

pub fn yg_node_mark_dirty(node: YGNodeRef) {
	unsafe {
        YGNodeMarkDirty(node);
	}
}

pub fn yg_node_mark_dirty_and_propogate_to_descendants(node: YGNodeRef) {
	unsafe {
        YGNodeMarkDirtyAndPropogateToDescendants(node);
	}
}

pub fn yg_node_print(node: YGNodeRef, options: YGPrintOptions) {
	unsafe {
        YGNodePrint(node, options);
	}
}

pub fn yg_float_is_undefined(value: f32) -> bool {
	unsafe {
        YGFloatIsUndefined(value)
	}
}

pub fn yg_node_can_use_cached_measurement(
    width_mode: YGMeasureMode,
    width: f32,
    height_mode: YGMeasureMode,
    height: f32,
    last_width_mode: YGMeasureMode,
    last_width: f32,
    last_height_mode: YGMeasureMode,
    last_height: f32,
    last_computed_width: f32,
    last_computed_height: f32,
    margin_row: f32,
    margin_column: f32,
    config: YGConfigRef,
) -> bool {
	unsafe {
        YGNodeCanUseCachedMeasurement(width_mode, 
                                    width, 
                                    height_mode, 
                                    height,
                                    last_width_mode, 
                                    last_width, 
                                    last_height_mode, 
                                    last_height, 
                                    last_computed_width, 
                                    last_computed_height, 
                                    margin_row, 
                                    margin_column, 
                                    config)
	}
}

pub fn yg_node_copy_style(dst_node: YGNodeRef, src_node: YGNodeRef) {
	unsafe {
        YGNodeCopyStyle(dst_node, src_node);
	}
}

pub fn yg_node_get_context(node: YGNodeRef) -> *mut c_void {
	unsafe {
        YGNodeGetContext(node)
	}
}

pub fn yg_node_set_context(node: YGNodeRef, context: *mut c_void) {
	unsafe {
        YGNodeSetContext(node, context);
	}
}

pub fn yg_config_set_print_tree_flag(config: YGConfigRef, enabled: bool) {
	unsafe {
        YGConfigSetPrintTreeFlag(config, enabled);
	}
}

pub fn yg_node_get_measure_func(node: YGNodeRef) -> YGMeasureFunc {
	unsafe {
        YGNodeGetMeasureFunc(node)
	}
}

pub fn yg_node_set_measure_func(node: YGNodeRef, measure_func: YGMeasureFunc) {
	unsafe {
        YGNodeSetMeasureFunc(node, measure_func);
	}
}

pub fn yg_node_get_baseline_func(node: YGNodeRef) -> YGBaselineFunc {
	unsafe {
        YGNodeGetBaselineFunc(node)
	}
}

pub fn yg_node_set_baseline_func(node: YGNodeRef, baseline_func: YGBaselineFunc) {
	unsafe {
        YGNodeSetBaselineFunc(node, baseline_func);
	}
}

pub fn yg_node_get_dirtied_func(node: YGNodeRef) -> YGDirtiedFunc {
	unsafe {
        YGNodeGetDirtiedFunc(node)
	}
}

pub fn yg_node_set_dirtied_func(node: YGNodeRef, dirtied_func: YGDirtiedFunc) {
	unsafe {
        YGNodeSetDirtiedFunc(node, dirtied_func);
	}
}

pub fn yg_node_get_print_func(node: YGNodeRef) -> YGPrintFunc {
	unsafe {
        YGNodeGetPrintFunc(node)
	}
}

pub fn yg_node_set_print_func(node: YGNodeRef, print_func: YGPrintFunc) {
	unsafe {
        YGNodeSetPrintFunc(node, print_func);
	}
}

pub fn yg_node_get_has_new_layout(node: YGNodeRef) -> bool {
	unsafe {
        YGNodeGetHasNewLayout(node)
	}
}

pub fn yg_node_set_has_new_layout(node: YGNodeRef, has_new_layout: bool) {
	unsafe {
        YGNodeSetHasNewLayout(node, has_new_layout);
	}
}

pub fn yg_node_get_node_type(node: YGNodeRef) -> YGNodeType {
	unsafe {
        YGNodeGetNodeType(node)
	}
}

pub fn yg_node_set_node_type(node: YGNodeRef, node_type: YGNodeType) {
	unsafe {
        YGNodeSetNodeType(node, node_type);
	}
}

pub fn yg_node_is_dirty(node: YGNodeRef) -> bool {
	unsafe {
        YGNodeIsDirty(node)
	}
}

pub fn yg_node_layout_get_did_use_legacy_flag(node: YGNodeRef) -> bool {
	unsafe {
        YGNodeLayoutGetDidUseLegacyFlag(node)
	}
}

pub fn yg_node_style_get_direction(node: YGNodeRef) -> YGDirection {
	unsafe {
        YGNodeStyleGetDirection(node)
	}
}

pub fn yg_node_style_set_direction(node: YGNodeRef, direction: YGDirection) {
	unsafe {
        YGNodeStyleSetDirection(node, direction);
	}
}

pub fn yg_node_style_get_flex_direction(node: YGNodeRef) -> YGFlexDirection {
	unsafe {
        YGNodeStyleGetFlexDirection(node)
	}
}

pub fn yg_node_style_set_flex_direction(node: YGNodeRef, flex_direction: YGFlexDirection) {
	unsafe {
        YGNodeStyleSetFlexDirection(node, flex_direction);
	}
}

pub fn yg_node_style_get_justify_content(node: YGNodeRef) -> YGJustify {
	unsafe {
        YGNodeStyleGetJustifyContent(node)
	}
}

pub fn yg_node_style_set_justify_content(node: YGNodeRef, justify_content: YGJustify) {
	unsafe {
        YGNodeStyleSetJustifyContent(node, justify_content);
	}
}

pub fn yg_node_style_get_align_content(node: YGNodeRef) -> YGAlign {
	unsafe {
        YGNodeStyleGetAlignContent(node)
	}
}

pub fn yg_node_style_set_align_content(node: YGNodeRef, align_content: YGAlign) {
	unsafe {
        YGNodeStyleSetAlignContent(node, align_content);
	}
}

pub fn yg_node_style_get_align_items(node: YGNodeRef) -> YGAlign {
	unsafe {
        YGNodeStyleGetAlignItems(node)
	}
}

pub fn yg_node_style_set_align_items(node: YGNodeRef, align_items: YGAlign) {
	unsafe {
        YGNodeStyleSetAlignItems(node, align_items);
	}
}

pub fn yg_node_style_get_align_self(node: YGNodeRef) -> YGAlign {
	unsafe {
        YGNodeStyleGetAlignSelf(node)
	}
}

pub fn yg_node_style_set_align_self(node: YGNodeRef, align_self: YGAlign) {
	unsafe {
        YGNodeStyleSetAlignSelf(node, align_self);
	}
}

pub fn yg_node_style_get_position_type(node: YGNodeRef) -> YGPositionType {
	unsafe {
        YGNodeStyleGetPositionType(node)
	}
}

pub fn yg_node_style_set_position_type(node: YGNodeRef, position_type: YGPositionType) {
	unsafe {
        YGNodeStyleSetPositionType(node, position_type);
	}
}

pub fn yg_node_style_get_flex_wrap(node: YGNodeRef) -> YGWrap {
	unsafe {
        YGNodeStyleGetFlexWrap(node)
	}
}

pub fn yg_node_style_set_flex_wrap(node: YGNodeRef, flex_wrap: YGWrap) {
	unsafe {
        YGNodeStyleSetFlexWrap(node, flex_wrap);
	}
}

pub fn yg_node_style_get_overflow(node: YGNodeRef) -> YGOverflow {
	unsafe {
        YGNodeStyleGetOverflow(node)
	}
}

pub fn yg_node_style_set_overflow(node: YGNodeRef, overflow: YGOverflow) {
	unsafe {
        YGNodeStyleSetOverflow(node, overflow);
	}
}

pub fn yg_node_style_get_display(node: YGNodeRef) -> YGDisplay {
	unsafe {
        YGNodeStyleGetDisplay(node)
	}
}

pub fn yg_node_style_set_display(node: YGNodeRef, display: YGDisplay) {
	unsafe {
        YGNodeStyleSetDisplay(node, display);
	}
}

pub fn yg_node_style_get_flex(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeStyleGetFlex(node)
	}
}

pub fn yg_node_style_set_flex(node: YGNodeRef, flex: f32) {
	unsafe {
        YGNodeStyleSetFlex(node, flex);
	}
}

pub fn yg_node_style_get_flex_grow(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeStyleGetFlexGrow(node)
	}
}

pub fn yg_node_style_set_flex_grow(node: YGNodeRef, flex_grow: f32) {
	unsafe {
        YGNodeStyleSetFlexGrow(node, flex_grow);
	}
}

pub fn yg_node_style_get_flex_shrink(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeStyleGetFlexShrink(node)
	}
}

pub fn yg_node_style_set_flex_shrink(node: YGNodeRef, flex_shrink: f32) {
	unsafe {
        YGNodeStyleSetFlexShrink(node, flex_shrink);
	}
}

pub fn yg_node_style_get_flex_basis(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetFlexBasis(node)
	}
}

pub fn yg_node_style_set_flex_basis(node: YGNodeRef, flex_basis: f32) {
	unsafe {
        YGNodeStyleSetFlexBasis(node, flex_basis);
	}
}

pub fn yg_node_style_set_flex_basis_percent(node: YGNodeRef, flex_basis: f32) {
	unsafe {
        YGNodeStyleSetFlexBasisPercent(node, flex_basis);
	}
}

pub fn yg_node_style_set_flex_basis_auto(node: YGNodeRef) {
	unsafe {
        YGNodeStyleSetFlexBasisAuto(node);
	}
}

pub fn yg_node_style_get_position(node: YGNodeRef, edge: YGEdge) -> YGValue {
	unsafe {
        YGNodeStyleGetPosition(node, edge)
	}
}

pub fn yg_node_style_set_position(node: YGNodeRef, edge: YGEdge, position: f32) {
	unsafe {
        YGNodeStyleSetPosition(node, edge, position);
	}
}

pub fn yg_node_style_set_position_percent(node: YGNodeRef, edge: YGEdge, position: f32) {
	unsafe {
        YGNodeStyleSetPositionPercent(node, edge, position);
	}
}

pub fn yg_node_style_get_margin(node: YGNodeRef, edge: YGEdge) -> YGValue {
	unsafe {
        YGNodeStyleGetMargin(node, edge)
	}
}

pub fn yg_node_style_set_margin(node: YGNodeRef, edge: YGEdge, margin: f32) {
	unsafe {
        YGNodeStyleSetMargin(node, edge, margin);
	}
}

pub fn yg_node_style_set_margin_percent(node: YGNodeRef, edge: YGEdge, margin: f32) {
	unsafe {
        YGNodeStyleSetMarginPercent(node, edge, margin);
	}
}

pub fn yg_node_style_set_margin_auto(node: YGNodeRef, edge: YGEdge) {
	unsafe {
        YGNodeStyleSetMarginAuto(node, edge);
	}
}

pub fn yg_node_style_get_padding(node: YGNodeRef, edge: YGEdge) -> YGValue {
	unsafe {
        YGNodeStyleGetPadding(node, edge)
	}
}

pub fn yg_node_style_set_padding(node: YGNodeRef, edge: YGEdge, padding: f32) {
	unsafe {
        YGNodeStyleSetPadding(node, edge, padding);
	}
}

pub fn yg_node_style_set_padding_percent(node: YGNodeRef, edge: YGEdge, padding: f32) {
	unsafe {
        YGNodeStyleSetPaddingPercent(node, edge, padding);
	}
}

pub fn yg_node_style_get_border(node: YGNodeRef, edge: YGEdge) -> f32 {
	unsafe {
        YGNodeStyleGetBorder(node, edge)
	}
}

pub fn yg_node_style_set_border(node: YGNodeRef, edge: YGEdge, border: f32) {
	unsafe {
        YGNodeStyleSetBorder(node, edge, border);
	}
}

pub fn yg_node_style_get_width(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetWidth(node)
	}
}

pub fn yg_node_style_set_width(node: YGNodeRef, width: f32) {
	unsafe {
        YGNodeStyleSetWidth(node, width);
	}
}

pub fn yg_node_style_set_width_percent(node: YGNodeRef, width: f32) {
	unsafe {
        YGNodeStyleSetWidthPercent(node, width);
	}
}

pub fn yg_node_style_set_width_auto(node: YGNodeRef) {
	unsafe {
        YGNodeStyleSetWidthAuto(node);
	}
}

pub fn yg_node_style_get_height(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetHeight(node)
	}
}

pub fn yg_node_style_set_height(node: YGNodeRef, height: f32) {
	unsafe {
        YGNodeStyleSetHeight(node, height);
	}
}

pub fn yg_node_style_set_height_percent(node: YGNodeRef, height: f32) {
	unsafe {
        YGNodeStyleSetHeightPercent(node, height);
	}
}

pub fn yg_node_style_set_height_auto(node: YGNodeRef) {
	unsafe {
        YGNodeStyleSetHeightAuto(node);
	}
}

pub fn yg_node_style_get_min_width(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetMinWidth(node)
	}
}

pub fn yg_node_style_set_min_width(node: YGNodeRef, min_width: f32) {
	unsafe {
        YGNodeStyleSetMinWidth(node, min_width);
	}
}

pub fn yg_node_style_set_min_width_percent(node: YGNodeRef, min_width: f32) {
	unsafe {
        YGNodeStyleSetMinWidthPercent(node, min_width);
	}
}

pub fn yg_node_style_get_min_height(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetMinHeight(node)
	}
}

pub fn yg_node_style_set_min_height(node: YGNodeRef, min_height: f32) {
	unsafe {
        YGNodeStyleSetMinHeight(node, min_height);
	}
}

pub fn yg_node_style_set_min_height_percent(node: YGNodeRef, min_height: f32) {
	unsafe {
        YGNodeStyleSetMinHeightPercent(node, min_height);
	}
}

pub fn yg_node_style_get_max_width(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetMaxWidth(node)
	}
}

pub fn yg_node_style_set_max_width(node: YGNodeRef, max_width: f32) {
	unsafe {
        YGNodeStyleSetMaxWidth(node, max_width);
	}
}

pub fn yg_node_style_set_max_width_percent(node: YGNodeRef, max_width: f32) {
	unsafe {
        YGNodeStyleSetMaxWidthPercent(node, max_width);
	}
}

pub fn yg_node_style_get_max_height(node: YGNodeRef) -> YGValue {
	unsafe {
        YGNodeStyleGetMaxHeight(node)
	}
}

pub fn yg_node_style_set_max_height(node: YGNodeRef, max_height: f32) {
	unsafe {
        YGNodeStyleSetMaxHeight(node, max_height);
	}
}

pub fn yg_node_style_set_max_height_percent(node: YGNodeRef, max_height: f32) {
	unsafe {
        YGNodeStyleSetMaxHeightPercent(node, max_height);
	}
}

pub fn yg_node_style_get_aspect_ratio(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeStyleGetAspectRatio(node)
	}
}

pub fn yg_node_style_set_aspect_ratio(node: YGNodeRef, aspect_ratio: f32) {
	unsafe {
        YGNodeStyleSetAspectRatio(node, aspect_ratio);
	}
}

pub fn yg_node_layout_get_left(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeLayoutGetLeft(node)
	}
}

pub fn yg_node_layout_get_top(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeLayoutGetTop(node)
	}
}

pub fn yg_node_layout_get_right(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeLayoutGetRight(node)
	}
}

pub fn yg_node_layout_get_bottom(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeLayoutGetBottom(node)
	}
}

pub fn yg_node_layout_get_width(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeLayoutGetWidth(node)
	}
}

pub fn yg_node_layout_get_height(node: YGNodeRef) -> f32 {
	unsafe {
        YGNodeLayoutGetHeight(node)
	}
}

pub fn yg_node_layout_get_direction(node: YGNodeRef) -> YGDirection {
	unsafe {
        YGNodeLayoutGetDirection(node)
	}
}

pub fn yg_node_layout_get_had_overflow(node: YGNodeRef) -> bool {
	unsafe {
        YGNodeLayoutGetHadOverflow(node)
	}
}

pub fn yg_node_layout_get_did_legacy_stretch_flag_affect_layout(node: YGNodeRef) -> bool {
	unsafe {
        YGNodeLayoutGetDidLegacyStretchFlagAffectLayout(node)
	}
}

pub fn yg_node_layout_get_margin(node: YGNodeRef, edge: YGEdge) -> f32 {
	unsafe {
        YGNodeLayoutGetMargin(node, edge)
	}
}

pub fn yg_node_layout_get_border(node: YGNodeRef, edge: YGEdge) -> f32 {
	unsafe {
        YGNodeLayoutGetBorder(node, edge)
	}
}

pub fn yg_node_layout_get_padding(node: YGNodeRef, edge: YGEdge) -> f32 {
	unsafe {
        YGNodeLayoutGetPadding(node, edge)
	}
}

pub fn yg_config_set_logger(config: YGConfigRef, logger: YGLogger) {
	unsafe {
        YGConfigSetLogger(config, logger);
	}
}

pub fn yg_log(node: YGNodeRef, level: YGLogLevel, message: &str) {
	unsafe {
        YGLog(node, level, CString::new(message).unwrap().as_ptr());
	}
}

pub fn yg_log_with_config(config: YGConfigRef, level: YGLogLevel, format: &str) {
	unsafe {
        YGLogWithConfig(config, level, CString::new(format).unwrap().as_ptr());
	}
}

pub fn yg_assert(condition: bool, message: &str) {
	unsafe {
        YGAssert(condition, CString::new(message).unwrap().as_ptr());
	}
}

pub fn yg_assert_with_node(node: YGNodeRef, condition: bool, message: &str) {
	unsafe {
        YGAssertWithNode(node, condition, CString::new(message).unwrap().as_ptr());
	}
}

pub fn yg_assert_with_config(config: YGConfigRef, condition: bool, message: &str) {
	unsafe {
        YGAssertWithConfig(config, condition, CString::new(message).unwrap().as_ptr());
	}
}

pub fn yg_config_set_point_scale_factor(config: YGConfigRef, pixels_in_point: f32) {
	unsafe {
        YGConfigSetPointScaleFactor(config, pixels_in_point);
	}
}

pub fn yg_config_set_should_diff_layout_without_legacy_stretch_behaviour(
    config: YGConfigRef,
    should_diff_layout: bool,
) {
	unsafe {
        YGConfigSetShouldDiffLayoutWithoutLegacyStretchBehaviour(config, should_diff_layout);
	}
}

pub fn yg_config_set_use_legacy_stretch_behaviour(
    config: YGConfigRef,
    use_legacy_stretch_behaviour: bool,
) {
	unsafe {
        YGConfigSetUseLegacyStretchBehaviour(config, use_legacy_stretch_behaviour);
	}
}

pub fn yg_config_new() -> YGConfigRef {
	unsafe {
        YGConfigNew()
	}
}

pub fn yg_config_free(config: YGConfigRef) {
	unsafe {
        YGConfigFree(config);
	}
}

pub fn yg_config_copy(dest: YGConfigRef, src: YGConfigRef) {
	unsafe {
        YGConfigCopy(dest, src);
	}
}

pub fn yg_config_get_instance_count() -> i32 {
	unsafe {
        YGConfigGetInstanceCount()
	}
}

pub fn yg_config_set_experimental_feature_enabled(
    config: YGConfigRef,
    feature: YGExperimentalFeature,
    enabled: bool,
) {
	unsafe {
        YGConfigSetExperimentalFeatureEnabled(config, feature, enabled);
	}
}

pub fn yg_config_is_experimental_feature_enabled(
    config: YGConfigRef,
    feature: YGExperimentalFeature,
) -> bool {
	unsafe {
        YGConfigIsExperimentalFeatureEnabled(config, feature)
	}
}

pub fn yg_config_set_use_web_defaults(config: YGConfigRef, enabled: bool) {
	unsafe {
        YGConfigSetUseWebDefaults(config, enabled);
	}
}

pub fn yg_config_get_use_web_defaults(config: YGConfigRef) -> bool {
	unsafe {
        YGConfigGetUseWebDefaults(config)
	}
}

pub fn yg_config_set_clone_node_func(config: YGConfigRef, callback: YGCloneNodeFunc) {
	unsafe {
        YGConfigSetCloneNodeFunc(config, callback);
	}
}

pub fn yg_config_get_default() -> YGConfigRef {
	unsafe {
        YGConfigGetDefault()
	}
}

pub fn yg_config_set_context(config: YGConfigRef, context: *mut c_void) {
	unsafe {
        YGConfigSetContext(config, context);
	}
}

pub fn yg_config_get_context(config: YGConfigRef) -> *mut c_void {
	unsafe {
        YGConfigGetContext(config)
	}
}

pub fn yg_round_value_to_pixel_grid(
    value: f32,
    point_scale_factor: f32,
    force_ceil: bool,
    force_floor: bool,
) -> f32 {
	unsafe {
        YGRoundValueToPixelGrid(value, point_scale_factor, force_ceil, force_floor)
	}
}
