use std::mem::transmute;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use serde::{Serialize, Deserialize};

use hal_core::*;
use hal_webgl::*;
use hash::XHashMap;
use flex_layout::style::*;

use ecs::{Lend, LendMut};
use gui::component::calc::*;
use gui::component::user::*;
// use gui::render::res::*;
use gui::system::util::cal_matrix;
// use gui::single::Oct;
// use gui::component::user::*;
// use gui::layout::*;

use gui::render::engine::ShareEngine;
use gui::single::*;
use GuiWorld;

#[derive(Serialize, Deserialize, Debug)]
struct Quad {
    pub left_top: Point2,
    pub left_bottom: Point2,
    pub right_bottom: Point2,
    pub right_top: Point2,
}

#[derive(Serialize, Deserialize, Debug)]
struct Layout1 {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub border_left: f32,
    pub border_top: f32,
    pub border_end: f32,
    pub border_bottom: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_end: f32,
    pub padding_bottom: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Info {
    pub overflow: bool,
    pub by_overflow: usize,
    pub visibility: bool,
    pub enable: bool,
    pub opacity: f32,
    pub zindex: u32,
    pub zdepth: f32,
    pub layout: Layout1,
    pub border_box: Quad,
    pub padding_box: Quad,
    pub content_box: Quad,
    pub culling: bool,
    pub render_obj: Vec<RenderObject>,
    // char_block: Option<CharBlock1>,
    pub class_name: Option<ClassName>,
    pub image: Option<String>,
    pub border_image: Option<String>,
    pub background_color: Option<BackgroundColor>,
    pub border_color: Option<BorderColor>,
    pub transform: Option<Transform>,
    pub box_shadow: Option<BoxShadow>,
    pub border_image_clip: Option<BorderImageClip>,
    pub border_image_slice: Option<BorderImageSlice>,
    pub border_image_repeat: Option<BorderImageRepeat>,
    pub image_clip: Option<ImageClip>,
    pub border_radius: Option<BorderRadius>,
    pub object_fit: Option<ObjectFit>,
    pub filter: Option<Filter>,
    pub transform_will_change: Option<TransformWillChange>,
    pub parent_id: Option<u32>,

    pub text: Option<TextStyle>,
	pub text_content: Option<TextContent>,
	pub children: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RenderObject {
    pub depth: f32,
    pub depth_diff: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub vs_name: String,
    pub fs_name: String,
    pub vs_defines: Vec<String>,
    pub fs_defines: Vec<String>,
    pub paramter: XHashMap<String, Paramter>,
    pub program_dirty: bool,

    pub program: bool,
    pub geometry: bool,
    pub state: State,

    pub context: usize,
}

#[derive(Serialize, Deserialize, Debug)]
enum Paramter {
    Uniform(UniformValue),
    Ubo(XHashMap<String, UniformValue>),
}

#[derive(Serialize, Deserialize, Debug)]
struct State {
    pub rs: RasterStateDesc,
    pub bs: BlendStateDesc,
    pub ss: StencilStateDesc,
    pub ds: DepthStateDesc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RasterStateDesc {
    pub cull_mode: Option<CullMode>,
    pub is_front_face_ccw: bool,
    pub polygon_offset: (f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlendStateDesc {
    pub rgb_equation: BlendFunc,
    pub alpha_equation: BlendFunc,

    pub src_rgb_factor: BlendFactor,
    pub dst_rgb_factor: BlendFactor,

    pub src_alpha_factor: BlendFactor,
    pub dst_alpha_factor: BlendFactor,

    pub const_rgba: (f32, f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverflowClip {
    pub id_map: XHashMap<usize, usize>,
    pub clip: Vec<(usize, Clip)>,
    pub clip_map: XHashMap<usize, Aabb3>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CharBlock1 {
    pub font_size: f32,    // 字体高度
    pub font_height: f32,  // 字体高度
    pub stroke_width: f32, //描边宽度
    pub line_height: f32,
    pub chars: Vec<CharNode>,            // 字符集合
    pub lines: Vec<(usize, usize, f32)>, // 不折行下的每行的起始字符位置、单词数量和总宽度。 自动折行不会影响该值
    pub last_line: (usize, usize, f32),  // 最后一行的起始字符位置、单词数量和总宽度
    pub size: Vector2,
    pub wrap_size: Vector2,
    pub pos: Point2,
    pub line_count: usize,  // 行数，
    pub fix_width: bool,    // 如果有字宽不等于font_size
    pub style_class: usize, // 使用的那个样式类
    pub is_pixel: bool,
}

// 字符节点， 对应一个字符的
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharNode {
    pub ch: char,              // 字符
    pub width: f32,            // 字符宽度
    pub pos: Point2,           // 位置
    pub ch_id_or_count: usize, // 字符id或单词的字符数量
    pub base_width: f32,       // font_size 为32 的字符宽度
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clazz(pub Class);

#[wasm_bindgen]
pub fn list_class(world: u32) -> JsValue {
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;

	let class_map = &world
        .class_sheet
        .lend()
		.class_map;
	let mut r = Vec::new();
	for ci in class_map.iter() {
		r.push(ci.0);
	}
	JsValue::from_serde(&r).unwrap()
}

#[wasm_bindgen]
pub fn get_class_name(world: u32, node: u32) -> JsValue {
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;
	
	let class_name = world.class_name.lend();

	JsValue::from_serde(&class_name.get(node as usize)).unwrap()
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn get_class(world: u32, class_name: u32) -> JsValue {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let class = match world
        .class_sheet
        .lend()
        .class_map
        .get(&(class_name as usize))
    {
        Some(r) => {
            let mut ret = "".to_string();
            for attr in r.attrs1.iter() {
                let s = to_css_str(Attr::Attr1(attr));
                if s.as_str() != "" {
                    ret += (s + ";").as_str();
                }
            }
            for attr in r.attrs2.iter() {
                let s = to_css_str(Attr::Attr2(attr));
                if s.as_str() != "" {
                    ret += (s + ";").as_str();
                }
            }
            for attr in r.attrs3.iter() {
                let s = to_css_str(Attr::Attr3(attr));
                if s.as_str() != "" {
                    ret += (s + ";").as_str();
                }
            }
            Some(ret)
        }
        None => None,
    };

	JsValue::from_serde(&class).unwrap()
}

enum Attr<'a> {
    Attr1(&'a Attribute1),
    Attr2(&'a Attribute2),
    Attr3(&'a Attribute3),
}

fn to_css_str(attr: Attr) -> String {
    match attr {
        Attr::Attr1(attr) => match attr {
            Attribute1::PositionType(r) => match r {
                PositionType::Relative => "position:relative".to_string(),
                PositionType::Absolute => "position:absolute".to_string(),
            },
            Attribute1::FlexWrap(r) => match r {
                FlexWrap::NoWrap => "flex-wrap:nowrap".to_string(),
                FlexWrap::Wrap => "flex-wrap:wrap".to_string(),
                FlexWrap::WrapReverse => "flex-wrap:wrapreverse".to_string(),
            },
            Attribute1::FlexDirection(r) => match r {
                FlexDirection::Column => "flex-direction:column".to_string(),
                FlexDirection::ColumnReverse => {
                    "flex-direction:columnreverse".to_string()
                }
                FlexDirection::Row => "flex-direction:row".to_string(),
                FlexDirection::RowReverse => {
                    "flex-direction:rowreverse".to_string()
                }
            },
            Attribute1::AlignContent(r) => match r {
                // AlignContent::Auto => "align-content:auto".to_string(),
                AlignContent::FlexStart => "align-content:flex-start".to_string(),
                AlignContent::Center => "align-content:center".to_string(),
                AlignContent::FlexEnd => "align-content:flex-end".to_string(),
                AlignContent::Stretch => "align-content:stretch".to_string(),
                // AlignContent::Baseline => "align-content:baseline".to_string(),
                AlignContent::SpaceBetween => "align-content:space-between".to_string(),
                AlignContent::SpaceAround => "align-content:space-around".to_string(),
            },
            Attribute1::AlignItems(r) => match r {
                // AlignItems::Auto => "align-items:auto".to_string(),
                AlignItems::FlexStart => "align-items:flex-start".to_string(),
                AlignItems::Center => "align-items:center".to_string(),
                AlignItems::FlexEnd => "align-items:flex-end".to_string(),
                AlignItems::Stretch => "align-items:stretch".to_string(),
                AlignItems::Baseline => "align-items:baseline".to_string(),
                // AlignItems::SpaceBetween => "align-items:space-between".to_string(),
                // AlignItems::SpaceAround => "align-items:space-around".to_string(),
            },
            Attribute1::AlignSelf(r) => match r {
                AlignSelf::Auto => "align-self:auto".to_string(),
                AlignSelf::FlexStart => "align-self:flex-start".to_string(),
                AlignSelf::Center => "align-self:center".to_string(),
                AlignSelf::FlexEnd => "align-self:flex-end".to_string(),
                AlignSelf::Stretch => "align-self:stretch".to_string(),
                AlignSelf::Baseline => "align-self:baseline".to_string(),
                // AlignSelf::SpaceBetween => "align-self:space-between".to_string(),
                // AlignSelf::SpaceAround => "align-self:space-around".to_string(),
            },
            Attribute1::JustifyContent(r) => match r {
                JustifyContent::FlexStart => "justify-content:flex-start".to_string(),
                JustifyContent::Center => "justify-content:center".to_string(),
                JustifyContent::FlexEnd => "justify-content:flex-end".to_string(),
                JustifyContent::SpaceBetween => "justify-content:space-between".to_string(),
                JustifyContent::SpaceAround => "justify-content:space-around".to_string(),
                JustifyContent::SpaceEvenly => "justify-content:space-evenly".to_string(),
            },

            Attribute1::ObjectFit(r) => match r.0 {
                FitType::None => "object-fit:none".to_string(),
                FitType::Fill => "object-fit:fill".to_string(),
                FitType::Contain => "object-fit:contain".to_string(),
                FitType::Cover => "object-fit:cover".to_string(),
                FitType::ScaleDown => "object-fit:scale-down".to_string(),
                FitType::Repeat => "object-fit:repeat".to_string(),
                FitType::RepeatX => "object-fit:repeat-x".to_string(),
                FitType::RepeatY => "object-fit:repeat-y".to_string(),
            },
            Attribute1::TextAlign(r) => match r {
                TextAlign::Left => "text-align:left".to_string(),
                TextAlign::Right => "text-align:right".to_string(),
                TextAlign::Center => "text-align:center".to_string(),
                TextAlign::Justify => "text-align:justify".to_string(),
            },
            Attribute1::VerticalAlign(r) => match r {
                VerticalAlign::Top => "vertical-align:top".to_string(),
                VerticalAlign::Middle => "vertical-align:middle".to_string(),
                VerticalAlign::Bottom => "vertical-align:bottom".to_string(),
            },
            Attribute1::WhiteSpace(r) => match r {
                WhiteSpace::Normal => "white-space:normal".to_string(),
                WhiteSpace::Nowrap => "white-space:nowrap".to_string(),
                WhiteSpace::PreWrap => "white-space:pre-wrap".to_string(),
                WhiteSpace::Pre => "white-space:pre".to_string(),
                WhiteSpace::PreLine => "white-space:pre-line".to_string(),
            },
            Attribute1::FontStyle(r) => match r {
                FontStyle::Normal => "font-style:normal".to_string(),
                FontStyle::Ttalic => "font-style:ttalic".to_string(),
                FontStyle::Oblique => "font-style:oblique".to_string(),
            },
            Attribute1::Enable(r) => match r {
                EnableType::Auto => "enable:auto".to_string(),
                EnableType::None => "enable:none".to_string(),
                EnableType::Visible => "enable:visible".to_string(),
            },
            Attribute1::Display(r) => match r {
                Display::Flex => "display:flex".to_string(),
                Display::None => "display:none".to_string(),
            },
            Attribute1::Visibility(r) => match r {
                true => "visibility:visible".to_string(),
                false => "visibility:hidden".to_string(),
            },
            Attribute1::Overflow(r) => match r {
                true => "overflow:hidden".to_string(),
                false => "overflow:visible".to_string(),
            },
        },
        Attr::Attr2(attr) => match attr {
            Attribute2::LetterSpacing(r) => "letter-spacing:".to_string() + r.to_string().as_str(),
            Attribute2::LineHeight(r) => match r {
                LineHeight::Normal => "line-height:normal".to_string(),
                LineHeight::Length(r) => "line-height:".to_string() + r.to_string().as_str() + "px",
                LineHeight::Number(r) => "line-height:".to_string() + r.to_string().as_str(),
                LineHeight::Percent(r) => "line-height:".to_string() + (r * 100.0).to_string().as_str() + "%",
            },
            Attribute2::TextIndent(r) => "text-indent:".to_string() + r.to_string().as_str() + "px",
            Attribute2::WordSpacing(r) => "word-space:".to_string() + r.to_string().as_str() + "px",
            Attribute2::FontWeight(r) => "font-weight:".to_string() + r.to_string().as_str(),
            Attribute2::FontSize(_r) => "".to_string(), // TODO
            Attribute2::FontFamily(r) => "font-family:".to_string() + r.to_string().as_str(),
            Attribute2::ZIndex(r) => "z-index:".to_string() + r.to_string().as_str(),
            Attribute2::Opacity(r) => "opacity:".to_string() + r.0.to_string().as_str(),
            // Attribute2::BorderImageRepeat(BorderImageRepeat)(x, y) => "border-image-repeat:" + r.to_string().as_str() + " " +,
            Attribute2::ImageUrl(r) => "src:".to_string() + r.to_string().as_str(),
            Attribute2::BorderImageUrl(r) => {
                "border-image-src:".to_string() + r.to_string().as_str()
            }

            Attribute2::FlexShrink(r) => "flex-shrink:".to_string() + r.to_string().as_str(),
            Attribute2::FlexGrow(r) => "flex-grow:".to_string() + r.to_string().as_str(),
            Attribute2::Width(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "width:auto".to_string(),
                Dimension::Points(r) => "width:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "width:".to_string() + (r * 100.0).to_string().as_str() + "%",
            },
            Attribute2::Height(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "height:auto".to_string(),
                Dimension::Points(r) => "height:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "height:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MarginLeft(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "margin-left:auto".to_string(),
                Dimension::Points(r) => "margin-left:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "margin-left:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MarginTop(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "margin-top:auto".to_string(),
                Dimension::Points(r) => "margin-top:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "margin-top:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MarginBottom(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "margin-bottom:auto".to_string(),
                Dimension::Points(r) => "margin-bottom:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => {
                    "margin-bottom:".to_string() + (r*100.0).to_string().as_str() + "%"
                }
            },
            Attribute2::MarginRight(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "margin-right:auto".to_string(),
                Dimension::Points(r) => "margin-right:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "margin-right:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::Margin(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "margin:auto".to_string(),
                Dimension::Points(r) => "margin:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "margin:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::PaddingLeft(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "padding-left:auto".to_string(),
                Dimension::Points(r) => "padding-left:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "padding-left:".to_string() + (r * 100.0).to_string().as_str() + "%",
            },
            Attribute2::PaddingTop(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "padding-top:auto".to_string(),
                Dimension::Points(r) => "padding-top:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "padding-top:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::PaddingBottom(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "padding-bottom:auto".to_string(),
                Dimension::Points(r) => {
                    "padding-bottom:".to_string() + r.to_string().as_str() + "px"
                }
                Dimension::Percent(r) => {
                    "padding-bottom:".to_string() + (r*100.0).to_string().as_str() + "%"
                }
            },
            Attribute2::PaddingRight(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "padding-right:auto".to_string(),
                Dimension::Points(r) => "padding-right:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => {
                    "padding-right:".to_string() + (r*100.0).to_string().as_str() + "%"
                }
            },
            Attribute2::Padding(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "padding:auto".to_string(),
                Dimension::Points(r) => "padding:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "padding:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::BorderLeft(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "border-left:auto".to_string(),
                Dimension::Points(r) => "borderleft:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "borderleft:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::BorderTop(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "border-top:auto".to_string(),
                Dimension::Points(r) => "border-top:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "border-top:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::BorderBottom(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "border-bottom:auto".to_string(),
                Dimension::Points(r) => "border-bottom:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => {
                    "border-bottom:".to_string() + (r*100.0).to_string().as_str() + "%"
                }
            },
            Attribute2::BorderRight(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "border-right:auto".to_string(),
                Dimension::Points(r) => "border-right:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "border-right:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::Border(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "width:auto".to_string(),
                Dimension::Points(r) => "width:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "width:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MinWidth(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "min-width:auto".to_string(),
                Dimension::Points(r) => "min-width:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "min-width:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MinHeight(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "min-height:auto".to_string(),
                Dimension::Points(r) => "min-height:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "min-height:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MaxHeight(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "max-height:auto".to_string(),
                Dimension::Points(r) => "max-height:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "max-height:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::MaxWidth(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "max-width:auto".to_string(),
                Dimension::Points(r) => "max-width:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "max-width:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::FlexBasis(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "flex-basis:auto".to_string(),
                Dimension::Points(r) => "flex-basis:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "flex-basis:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::PositionLeft(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "left:auto".to_string(),
                Dimension::Points(r) => "left:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "left:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::PositionTop(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "top:auto".to_string(),
                Dimension::Points(r) => "top:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "top:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::PositionRight(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "right:auto".to_string(),
                Dimension::Points(r) => "right:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "right:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            Attribute2::PositionBottom(r) => match r {
                Dimension::Undefined => "".to_string(),
                Dimension::Auto => "bottom:auto".to_string(),
                Dimension::Points(r) => "bottom:".to_string() + r.to_string().as_str() + "px",
                Dimension::Percent(r) => "bottom:".to_string() + (r*100.0).to_string().as_str() + "%",
            },
            _ => "".to_string(),
        },
        Attr::Attr3(attr) => match attr {
            Attribute3::BGColor(color) => match &color.0 {
                Color::RGBA(r) => {
                    "background-color:rgba(".to_string()
                        + r.r.to_string().as_str()
                        + ","
                        + r.g.to_string().as_str()
                        + ","
                        + r.b.to_string().as_str()
                        + ","
                        + r.a.to_string().as_str()
                        + ")"
                }
                Color::LinearGradient(_r) => "background-color:linear-gradient".to_string(),
            },
            Attribute3::BorderColor(r) => {
                let r = r.0;
                "border-color:rgba(".to_string()
                    + r.r.to_string().as_str()
                    + ","
                    + r.g.to_string().as_str()
                    + ","
                    + r.b.to_string().as_str()
                    + ","
                    + r.a.to_string().as_str()
                    + ")"
            }
            Attribute3::BoxShadow(r) => {
                "box-shadow:".to_string()
                    + r.h.to_string().as_str()
                    + " "
                    + r.v.to_string().as_str()
                    + " "
                    + r.blur.to_string().as_str()
                    + " "
                    + r.spread.to_string().as_str()
                    + " rgba("
                    + r.color.r.to_string().as_str()
                    + ","
                    + r.color.g.to_string().as_str()
                    + ","
                    + r.color.b.to_string().as_str()
                    + ","
                    + r.color.a.to_string().as_str()
                    + ")"
                // pub h: f32,         // 水平偏移，正右负左
                // pub v: f32,         // 垂直偏移，正下负上
                // pub blur: f32,      // 模糊半径，0代表不模糊，
                // pub spread: f32,    // 阴影扩展，上下左右各加上这个值
                // pub color: CgColor, // 阴影颜色
            }

            Attribute3::ImageClip(r) => {
                "image-clip:".to_string()
                    + (r.min.y * 100.0).to_string().as_str()
                    + "% "
                    + (r.max.x * 100.0).to_string().as_str()
                    + "% "
                    + (r.max.y * 100.0).to_string().as_str()
                    + "% "
                    + (r.min.x * 100.0).to_string().as_str()
                    + "%"
            }

            Attribute3::BorderImageClip(r) => {
                "border-image-clip:".to_string()
                    + (r.min.y * 100.0).to_string().as_str()
                    + "% "
                    + (r.max.x * 100.0).to_string().as_str()
                    + "% "
                    + (r.max.y * 100.0).to_string().as_str()
                    + "% "
                    + (r.min.x * 100.0).to_string().as_str()
                    + "%"
            }
            Attribute3::BorderImageSlice(r) => {
                let mut f = "";
                if r.fill {
                    f = " fill";
                }
                "border-image-slice:".to_string()
                    + (r.top * 100.0).to_string().as_str()
                    + "% "
                    + (r.right * 100.0).to_string().as_str()
                    + "% "
                    + (r.bottom * 100.0).to_string().as_str()
                    + "% "
                    + (r.left * 100.0).to_string().as_str()
                    + "%"
                    + f
            }

            Attribute3::Color(r) => match r {
                Color::RGBA(r) => {
                    "color:rgba(".to_string()
                        + r.r.to_string().as_str()
                        + ","
                        + r.g.to_string().as_str()
                        + ","
                        + r.b.to_string().as_str()
                        + ","
                        + r.a.to_string().as_str()
                        + ")"
                }
                Color::LinearGradient(_r) => "background-color:linear-gradient".to_string(),
            },
            Attribute3::TextShadow(r) => {
                "text-shadow:".to_string()
                    + r.h.to_string().as_str()
                    + " "
                    + r.v.to_string().as_str()
                    + " "
                    + r.blur.to_string().as_str()
                    + " rgba("
                    + r.color.r.to_string().as_str()
                    + ","
                    + r.color.g.to_string().as_str()
                    + ","
                    + r.color.b.to_string().as_str()
                    + ","
                    + r.color.a.to_string().as_str()
                    + ")"
            }
            Attribute3::TextStroke(r) => {
                "text-stroke:".to_string()
                    + " rgba("
                    + r.color.r.to_string().as_str()
                    + ","
                    + r.color.g.to_string().as_str()
                    + ","
                    + r.color.b.to_string().as_str()
                    + ","
                    + r.color.a.to_string().as_str()
                    + ")"
            }

            Attribute3::BorderRadius(_r) => "".to_string(), // TODO
            Attribute3::TransformFunc(_r) => "".to_string(), // TODO
            Attribute3::TransformOrigin(_r) => "".to_string(), // TODO
            Attribute3::Filter(_r) => "".to_string(),       // TODO
        },
    }
}

// 打印节点信息
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn node_info(world: u32, node: u32) -> JsValue {
    let node = node as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;
	let idtree = world.idtree.lend();

    // let z_depth = unsafe { world.z_depth.lend().get_unchecked(node) }.0;

    let parent = unsafe { idtree.get_unchecked(node) }.parent();

    let enable = unsafe { world.enable.lend().get_unchecked(node) }.0;
    let visibility = unsafe { world.visibility.lend().get_unchecked(node) }.0;

    let by_overflow = unsafe { world.by_overflow.lend().get_unchecked(node) }.0;

    let opacity = unsafe { world.opacity.lend().get_unchecked(node) }.0;

    let layout = world.layout.lend();

    let world_matrix = world.world_matrix.lend();

    let transform = world.transform.lend();

    let world_matrix1 = cal_matrix(node, world_matrix, transform, layout, &Transform::default());
    let layout = unsafe { layout.get_unchecked(node) };

	let width = layout.rect.end - layout.rect.start;
	let height = layout.rect.bottom - layout.rect.top;
    // border box
    let b_left_top = world_matrix1 * Vector4::new(0.0, 0.0, 1.0, 1.0);
    let b_left_bottom = world_matrix1 * Vector4::new(0.0, height, 1.0, 1.0);
    let b_right_bottom = world_matrix1 * Vector4::new(width, height, 1.0, 1.0);
    let b_right_top = world_matrix1 * Vector4::new(width, 0.0, 1.0, 1.0);

    // border box
    let absolute_b_box = Quad {
        left_top: Point2::new(b_left_top.x, b_left_top.y),
        left_bottom: Point2::new(b_left_bottom.x, b_left_bottom.y),
        right_bottom: Point2::new(b_right_bottom.x, b_right_bottom.y),
        right_top: Point2::new(b_right_top.x, b_right_top.y),
    };

    // padding box
    let p_left_top = world_matrix1 * Vector4::new(layout.border.start, layout.border.top, 1.0, 1.0);
    let p_left_bottom = world_matrix1
        * Vector4::new(
            layout.border.start,
            height - layout.border.bottom,
            1.0,
            1.0,
        );
    let p_right_bottom = world_matrix1
        * Vector4::new(
            width - layout.border.end,
            height - layout.border.bottom,
            1.0,
            1.0,
        );
    let p_right_top = world_matrix1
        * Vector4::new(
            width - layout.border.end,
            layout.border.top,
            1.0,
            1.0,
        );

    let absolute_p_box = Quad {
        left_top: Point2::new(p_left_top.x, p_left_top.y),
        left_bottom: Point2::new(p_left_bottom.x, p_left_bottom.y),
        right_bottom: Point2::new(p_right_bottom.x, p_right_bottom.y),
        right_top: Point2::new(p_right_top.x, p_right_top.y),
    };

    // content box
    let c_left_top = world_matrix1
        * Vector4::new(
            layout.border.start + layout.padding.start,
            layout.border.top + layout.padding.top,
            1.0,
            1.0,
        );
    let c_left_bottom = world_matrix1
        * Vector4::new(
            layout.border.start + layout.padding.start,
            height - layout.border.bottom - layout.padding.bottom,
            1.0,
            1.0,
        );
    let c_right_bottom = world_matrix1
        * Vector4::new(
            width - layout.border.end - layout.padding.end,
            height - layout.border.bottom - layout.padding.bottom,
            1.0,
            1.0,
        );
    let c_right_top = world_matrix1
        * Vector4::new(
            width - layout.border.end - layout.padding.end,
            layout.border.top + layout.padding.top,
            1.0,
            1.0,
        );

    let absolute_c_box = Quad {
        left_top: Point2::new(c_left_top.x, c_left_top.y),
        left_bottom: Point2::new(c_left_bottom.x, c_left_bottom.y),
        right_bottom: Point2::new(c_right_bottom.x, c_right_bottom.y),
        right_top: Point2::new(c_right_top.x, c_right_top.y),
    };

    // let yogas = world.yoga.lend();
    // let yoga = unsafe { yogas.get_unchecked(node) };

    // let octs = world.oct.lend();
    // let oct = unsafe { octs.get_unchecked(node) };

    let mut render_map = Vec::new();
    let map = world.world.fetch_single::<NodeRenderMap>().unwrap();
    let map = map.lend();
    let render_objs = world.world.fetch_single::<RenderObjs>().unwrap();
    let render_objs = render_objs.lend();
    let engine = world
        .world
        .fetch_single::<ShareEngine<WebglHalContext>>()
        .unwrap();
    let engine = engine.lend();
    if let Some(arr) = map.get(node) {
        for id in arr.iter() {
            let v = match render_objs.get(*id) {
                Some(r) => r,
                None => continue,
            };
            let mut paramter = XHashMap::default();
            // let pt = v.paramter.get_texture_layout();

            let val = v.paramter.get_values();
            let vals = v.paramter.get_single_uniforms();
            // let valt = v.paramter.get_textures();
            let mut i = 0;
            for name in v.paramter.get_layout() {
                let mut ubo = XHashMap::default();
                let ubo_val = val[i].get_values();
                let mut j = 0;
                for n in val[i].get_layout() {
                    ubo.insert(n.to_string(), ubo_val[j].clone());
                    j += 1;
                }
                paramter.insert(name.to_string(), Paramter::Ubo(ubo));
                i += 1;
            }

            i = 0;
            for name in v.paramter.get_single_uniform_layout() {
                paramter.insert(name.to_string(), Paramter::Uniform(vals[i].clone()));
                i += 1;
            }

            let rs = engine.gl.rs_get_desc(&v.state.rs);
            let bs = engine.gl.bs_get_desc(&v.state.bs);

            let mut vs_defines = Vec::new();
            for n in v.vs_defines.list().iter() {
                if let Some(r) = n {
                    vs_defines.push(r.to_string())
                }
            }

            let mut fs_defines = Vec::new();
            for n in v.fs_defines.list().iter() {
                if let Some(r) = n {
                    fs_defines.push(r.to_string())
                }
            }

			// let projectMatrix = world.world.fetch_single::<ProjectionMatrix>();
			// let viewMatrux = world.world.fetch_single::<ViewMatrix>();
			// let worldMatrix = world.world.fetch_single::<ViewMatrix>();
			
            let obj = RenderObject {
                depth: v.depth,
                depth_diff: v.depth_diff,
                visibility: v.visibility,
                is_opacity: v.is_opacity,
                vs_name: v.vs_name.as_ref().to_string(),
                fs_name: v.fs_name.as_ref().to_string(),
                vs_defines: vs_defines,
                fs_defines: fs_defines,
                paramter: paramter,
                program_dirty: v.program_dirty,

                program: v.program.is_some(),
                geometry: v.geometry.is_some(),
                state: State {
                    rs: unsafe { transmute(rs.clone()) },
                    bs: unsafe { transmute(bs.clone()) },
                    ss: engine.gl.ss_get_desc(&v.state.ss).clone(),
                    ds: engine.gl.ds_get_desc(&v.state.ds).clone(),
                },

                context: v.context,
            };
            render_map.push(obj);
        }
    }

    // let char_block = world
    //     .world
    //     .fetch_multi::<Node, CharBlock<YgNode>>()
    //     .unwrap();
    // let char_block = char_block.lend();
    // let char_block = match char_block.get(node) {
    //     Some(r) => {
    //         let mut c = CharBlock1 {
    //             font_size: r.font_size,
    //             font_height: r.font_height,
    //             stroke_width: r.stroke_width,
    //             line_height: r.line_height,
    //             chars: Vec::default(),
    //             lines: r.lines.clone(),
    //             last_line: r.last_line,
    //             size: r.size,
    //             wrap_size: r.wrap_size,
    //             pos: r.pos,
    //             line_count: r.line_count,
    //             fix_width: r.fix_width,
    //             style_class: r.style_class,
    //             is_pixel: r.is_pixel,
    //         };
    //         for i in r.chars.iter() {
    //             c.chars.push(CharNode {
    //                 ch: i.ch,
    //                 width: i.width,
    //                 pos: i.pos,
    //                 ch_id_or_count: i.ch_id_or_count,
    //                 base_width: i.base_width,
    //             });
    //         }
    //         Some(c)
    //     }
    //     None => None,
	// };
	let mut children = Vec::new();
	for id in idtree.iter(idtree[node].children().head) {
		children.push(id.0);
	}

    let info = Info {
        // char_block: char_block,
        overflow: unsafe { world.overflow.lend().get_unchecked(node) }.0,
        by_overflow: by_overflow,
        visibility: visibility,
        enable: enable,
        opacity: opacity,
        zindex: unsafe { world.z_index.lend().get_unchecked(node) }.0 as u32,
        zdepth: unsafe { world.z_depth.lend().get_unchecked(node) }.0,
        layout: unsafe { transmute(layout.clone()) },
        border_box: absolute_b_box,
        padding_box: absolute_p_box,
        content_box: absolute_c_box,
        culling: unsafe { world.culling.lend().get_unchecked(node) }.0,
        text: match world.text_style.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        text_content: match world.text_content.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        render_obj: render_map,
        class_name: match world.class_name.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        image: match world.image.lend().get(node) {
            Some(r) => Some(r.url.as_ref().to_string()),
            None => None,
        },
        border_image: match world.border_image.lend().get(node) {
            Some(r) => Some(r.0.url.as_ref().to_string()),
            None => None,
        },
        background_color: match world.background_color.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        border_color: match world.border_color.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        transform: match world.transform.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        box_shadow: match world.box_shadow.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        border_image_clip: match world.border_image_clip.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        border_image_slice: match world.border_image_slice.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        border_image_repeat: match world.border_image_repeat.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        image_clip: match world.image_clip.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        border_radius: match world.border_radius.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        object_fit: match world.object_fit.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        filter: match world.filter.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
        transform_will_change: match world.transform_will_change.lend().get(node) {
            Some(r) => Some(r.clone()),
            None => None,
        },
		parent_id: Some(parent as u32),
		children: children,
    };

    return JsValue::from_serde(&info).unwrap();
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn overflow_clip(world: u32) -> JsValue {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let overflow_clip = world.overflow_clip.lend();

    let mut clips: Vec<(usize, Clip)> = Vec::new();
    for (index, v) in overflow_clip.clip.iter() {
        clips.push((index, v.clone()));
    }

    let mut clip_map = XHashMap::default();
    for (k, v) in overflow_clip.clip_map.iter() {
        clip_map.insert(*k, v.0.clone());
    }
    let c = OverflowClip {
        id_map: overflow_clip.id_map.clone(),
        clip: clips,
        clip_map: clip_map,
	};
	return JsValue::from_serde(&c).unwrap();
}

// pub fn create_gui(engine: u32, width: f32, height: f32) -> u32 {

// // 打开性能检视面板
// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn open_performance_inspector(world: u32, width: f32, height: f32) -> u32 {
// 	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
// 	if world.performance_inspector == 0 {
// 		let world = &mut world.gui;
// 		let performance_gui = create_gui(Box::into_raw(Box::new((*world.engine).clone()) as u32, width, height);
// 		let performance_gui = unsafe {&mut *(performance_gui as usize as *mut GuiWorld)};
// 		gui_tool::open_performance_inspection(world, PerformanceStatisticians::new(&mut performance_gui.gui));
// 		world.performance_inspector = performance_gui;
// 		performance_gui as u32
// 	}
// }

// // 关闭性能检视面板
// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn close_performance_inspector(world: u32) {
// 	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
// 	if world.performance_inspector > 0 {
// 		let world = Box::from_raw(unsafe {world as usize as *mut GuiWorld});
// 		let world = &mut world.gui;
// 		gui_tool::close_performance_inspection(world);
// 	}
// }

#[wasm_bindgen]
pub fn res_size(world: u32) -> JsValue {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let engine = world.engine.lend();
    let mut size = ResMgrSize::default();

    let texture = engine.texture_res_map.all_res();
    for i in texture.0.iter() {
        size.texture += i.1;
        size.count_texture += 1;
    }
    for i in texture.1.iter() {
        size.catch_texture += i.1.elem.cost;
        size.count_catch_texture += 1;
    }

    let geometry = engine.geometry_res_map.all_res();
    for i in geometry.0.iter() {
        size.geometry += i.1;
        size.count_geometry += 1;
    }
    for i in geometry.1.iter() {
        size.catch_geometry += i.1.elem.cost;
        size.count_catch_geometry += 1;
    }

    let buffer = engine.buffer_res_map.all_res();
    for i in buffer.0.iter() {
        size.buffer += i.1;
        size.count_buffer += 1;
    }
    for i in buffer.1.iter() {
        size.catch_buffer += i.1.elem.cost;
        size.count_catch_buffer += 1;
    }

    let rs = engine.rs_res_map.all_res();
    for i in rs.0.iter() {
        size.rs += i.1;
        size.count_rs += 1;
    }
    for i in rs.1.iter() {
        size.catch_rs += i.1.elem.cost;
        size.count_catch_rs += 1;
    }

    let bs = engine.bs_res_map.all_res();
    for i in bs.0.iter() {
        size.bs += i.1;
        size.count_bs += 1;
    }
    for i in bs.1.iter() {
        size.catch_bs += i.1.elem.cost;
        size.count_catch_bs += 1;
    }

    let ss = engine.ss_res_map.all_res();
    for i in ss.0.iter() {
        size.ss += i.1;
        size.count_ss += 1;
    }
    for i in ss.1.iter() {
        size.catch_ss += i.1.elem.cost;
        size.count_catch_ss += 1;
    }

    let ds = engine.ds_res_map.all_res();
    for i in ds.0.iter() {
        size.ds += i.1;
        size.count_ds += 1;
    }
    for i in ds.1.iter() {
        size.catch_ds += i.1.elem.cost;
        size.count_catch_ds += 1;
    }

    let sampler = engine.sampler_res_map.all_res();
    for i in sampler.0.iter() {
        size.sampler += i.1;
        size.count_sampler += 1;
    }
    for i in sampler.1.iter() {
        size.catch_sampler += i.1.elem.cost;
        size.count_catch_sampler += 1;
    }

    let ucolor = engine.res_mgr.fetch_map::<UColorUbo>(0).unwrap();
    let ucolor = ucolor.all_res();
    for i in ucolor.0.iter() {
        size.ucolor += i.1;
        size.count_ucolor += 1;
    }
    for i in ucolor.1.iter() {
        size.catch_ucolor += i.1.elem.cost;
        size.count_catch_ucolor += 1;
    }

    let hsv = engine.res_mgr.fetch_map::<HsvUbo>(0).unwrap();
    let hsv = hsv.all_res();
    for i in hsv.0.iter() {
        size.hsv += i.1;
        size.count_hsv += 1;
    }
    for i in hsv.1.iter() {
        size.catch_hsv += i.1.elem.cost;
        size.count_catch_hsv += 1;
    }

    let msdf_stroke = engine.res_mgr.fetch_map::<MsdfStrokeUbo>(0).unwrap();
    let msdf_stroke = msdf_stroke.all_res();
    for i in msdf_stroke.0.iter() {
        size.msdf_stroke += i.1;
        size.count_msdf_stroke += 1;
    }
    for i in msdf_stroke.1.iter() {
        size.catch_msdf_stroke += i.1.elem.cost;
        size.count_catch_msdf_stroke += 1;
    }

    let canvas_stroke = engine
        .res_mgr
        .fetch_map::<CanvasTextStrokeColorUbo>(0)
        .unwrap();
    let canvas_stroke = canvas_stroke.all_res();
    for i in canvas_stroke.0.iter() {
        size.canvas_stroke += i.1;
        size.count_canvas_stroke += 1;
    }
    for i in canvas_stroke.1.iter() {
        size.catch_canvas_stroke += i.1.elem.cost;
        size.count_catch_canvas_stroke += 1;
    }

    size.total_capacity = engine.res_mgr.total_capacity;

    size.texture_max_capacity = engine.texture_res_map.cache.get_max_capacity();

    return JsValue::from_serde(&size).unwrap();
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn common_statistics(world: u32) -> JsValue {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui.world;

	let mut all_run_time = 0.0;
	let mut renderTime = 0.0;
	let mut layoutTime = 0.0;
    for t in world.runtime.iter() {
        // println!("t.sys_name:{:?}", t.sys_name);
        if t.sys_name.as_ref() == "render_sys" {
            renderTime = t.cost_time;
        } else if t.sys_name.as_ref() == "text_layout_sys" {
            layoutTime = t.cost_time;
        }
        all_run_time += t.cost_time;
    }

    let statistics = world.fetch_single::<Statistics>().unwrap();
	let statistics = statistics.lend_mut();
	return JsValue::from_serde(&CommonStatistics{
		renderTime,
		layoutTime,
		runTotalTimes: all_run_time,
		drawCallTimes: statistics.drawcall_times as u32,
	}).unwrap();
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CommonStatistics {
	pub renderTime: f32,
	pub layoutTime: f32,
	pub runTotalTimes: f32,
	pub drawCallTimes: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MemStatistics {
	pub textureTotalCount: u32,
	pub textureTotalMemory: u32,
}

#[wasm_bindgen]
pub fn mem_statistics(world: u32) -> JsValue {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let engine = world.gui.engine.lend();

    let texture = engine.texture_res_map.all_res();

    let mut texture_size = 0;
    let mut texture_count = 0;
    let mut catch_texture_count = 0;
    let mut catch_texture_size = 0;
    for i in texture.0.iter() {
        texture_size += i.1;
        texture_count += 1;
    }
    for i in texture.1.iter() {
        catch_texture_size += i.1.elem.cost;
        catch_texture_count += 1;
	}
	return JsValue::from_serde(&MemStatistics{
		textureTotalCount: (catch_texture_count + texture_count) as u32, 
		textureTotalMemory: (catch_texture_size + texture_size) as u32,
	}).unwrap();
}

/// 打印内存情况
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn print_memory(world: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    println!("print_memory begin");

    let mut total = 0;

    let r = world.node.lend().mem_size();
    total += r;
    println!("    world::node = {:?}", r);
    let r = world.transform.lend().mem_size();
    total += r;
    println!("    world::transform = {:?}", r);
    let r = world.z_index.lend().mem_size();
    total += r;
    println!("    world::z_index = {:?}", r);
    let r = world.overflow.lend().mem_size();
    total += r;
    println!("    world::overflow = {:?}", r);
    let r = world.show.lend().mem_size();
    total += r;
    println!("    world::show = {:?}", r);
    let r = world.opacity.lend().mem_size();
    total += r;
    println!("    world::opacity = {:?}", r);
    let r = world.background_color.lend().mem_size();
    total += r;
    println!("    world::background_color = {:?}", r);
    let r = world.box_shadow.lend().mem_size();
    total += r;
    println!("    world::box_shadow = {:?}", r);
    let r = world.border_color.lend().mem_size();
    total += r;
    println!("    world::border_color = {:?}", r);
    let r = world.border_image.lend().mem_size();
    total += r;
    println!("    world::border_image = {:?}", r);
    let r = world.border_image_clip.lend().mem_size();
    total += r;
    println!("    world::border_image_clip = {:?}", r);
    let r = world.border_image_slice.lend().mem_size();
    total += r;
    println!("    world::border_image_slice = {:?}", r);
    let r = world.border_image_repeat.lend().mem_size();
    total += r;
    println!("    world::border_image_repeat = {:?}", r);
    let r = world.text_style.lend().mem_size();
    total += r;
    println!("    world::text_style = {:?}", r);
    let r = world.text_content.lend().mem_size();
    total += r;
    println!("    world::text_content = {:?}", r);
    let r = world.font.lend().mem_size();
    total += r;
    println!("    world::font = {:?}", r);
    let r = world.border_radius.lend().mem_size();
    total += r;
    println!("    world::border_radius = {:?}", r);
    let r = world.image.lend().mem_size();
    total += r;
    println!("    world::image = {:?}", r);
    let r = world.image_clip.lend().mem_size();
    total += r;
    println!("    world::image_clip = {:?}", r);
    let r = world.object_fit.lend().mem_size();
    total += r;
    println!("    world::object_fit = {:?}", r);
    let r = world.filter.lend().mem_size();
    total += r;
    println!("    world::filter = {:?}", r);
    let r = world.rect_layout_style.lend().mem_size();
    total += r;
	println!("    world::rect_layout_style = {:?}", r);
	let r = world.other_layout_style.lend().mem_size();
    total += r;
    println!("    world::other_layout_style = {:?}", r);
    let r = world.class_name.lend().mem_size();
    total += r;
    println!("    world::class_name = {:?}", r);
    let r = world.style_mark.lend().mem_size();
    total += r;
    println!("    world::style_mark = {:?}", r);
    let r = world.z_depth.lend().mem_size();
    total += r;
    println!("world::z_depth = {:?}", r);
    let r = world.enable.lend().mem_size();
    total += r;
    println!("    world::enable = {:?}", r);
    let r = world.visibility.lend().mem_size();
    total += r;
    println!("    world::visibility = {:?}", r);
    let r = world.world_matrix.lend().mem_size();
    total += r;
    println!("    world::world_matrix = {:?}", r);
    let r = world.by_overflow.lend().mem_size();
    total += r;
    println!("    world::by_overflow = {:?}", r);
    let r = world.copacity.lend().mem_size();
    total += r;
    println!("    world::copacity = {:?}", r);
    let r = world.layout.lend().mem_size();
    total += r;
    println!("    world::layout = {:?}", r);
    let r = world.hsv.lend().mem_size();
    total += r;
    println!("    world::hsv = {:?}", r);
    let r = world.culling.lend().mem_size();
    total += r;
    println!("    world::culling = {:?}", r);
    // let r = world.idtree.lend().mem_size();
    // total += r;
    // println!("    world::idtree = {:?}", r);
    let r = world.oct.lend().mem_size();
    total += r;
    println!("    world::oct = {:?}", r);
    let r = world.overflow_clip.lend().mem_size();
    total += r;
    println!("    world::overflow_clip = {:?}", r);
    let r = world.engine.lend().res_mgr.mem_size();
    total += r;
    println!("    world::engine.resMap = {:?}", r);
    let r = world.render_objs.lend().mem_size();
    total += r;
    println!("    world::render_objs = {:?}", r);
    let r = world.font_sheet.lend().mem_size();
    total += r;
    println!("    world::font_sheet = {:?}", r);
    let r = world.default_table.lend().mem_size();
    total += r;
    println!("    world::default_table = {:?}", r);
    let r = world.class_sheet.lend().mem_size();
    total += r;
    println!("    world::class_sheet = {:?}", r);
    let r = world.image_wait_sheet.lend().mem_size();
    total += r;
    println!("    world::image_wait_sheet = {:?}", r);

    let engine = world.engine.lend_mut();
    let stat = engine.gl.render_get_stat();

    total += stat.slab_mem_size;
    println!(
        "    world::engine::slab_mem_size = {:?}",
        stat.slab_mem_size
    );

    let total: f32 = total as f32;
    println!(" slab total bytes = {:?} MB", total / 1024.0 / 1024.0);
    println!("");

    println!("    world::engine::rt_count = {:?}", stat.rt_count);
    println!(
        "    world::engine::texture_count = {:?}",
        stat.texture_count
    );
    println!("    world::engine::buffer_count = {:?}", stat.buffer_count);
    println!(
        "    world::engine::geometry_count = {:?}",
        stat.geometry_count
    );
    println!(
        "    world::engine::program_count = {:?}",
        stat.program_count
    );

    println!("print_memory end");
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ResMgrSize {
    pub count_texture: usize,
    pub count_geometry: usize,
    pub count_buffer: usize,
    pub count_sampler: usize,
    pub count_rs: usize,
    pub count_bs: usize,
    pub count_ss: usize,
    pub count_ds: usize,
    pub count_ucolor: usize,
    pub count_hsv: usize,
    pub count_msdf_stroke: usize,
    pub count_canvas_stroke: usize,

    pub count_catch_texture: usize,
    pub count_catch_geometry: usize,
    pub count_catch_buffer: usize,
    pub count_catch_sampler: usize,
    pub count_catch_rs: usize,
    pub count_catch_bs: usize,
    pub count_catch_ss: usize,
    pub count_catch_ds: usize,
    pub count_catch_ucolor: usize,
    pub count_catch_hsv: usize,
    pub count_catch_msdf_stroke: usize,
    pub count_catch_canvas_stroke: usize,

    pub texture: usize,
    pub geometry: usize,
    pub buffer: usize,
    pub sampler: usize,
    pub rs: usize,
    pub bs: usize,
    pub ss: usize,
    pub ds: usize,
    pub ucolor: usize,
    pub hsv: usize,
    pub msdf_stroke: usize,
    pub canvas_stroke: usize,

    pub catch_texture: usize,
    pub catch_geometry: usize,
    pub catch_buffer: usize,
    pub catch_sampler: usize,
    pub catch_rs: usize,
    pub catch_bs: usize,
    pub catch_ss: usize,
    pub catch_ds: usize,
    pub catch_ucolor: usize,
    pub catch_hsv: usize,
    pub catch_msdf_stroke: usize,
    pub catch_canvas_stroke: usize,

    pub total_capacity: usize,
    pub texture_max_capacity: usize,
}

// #[allow(unused_attributes)]
// #[wasm_bindgen]
// pub fn bound_box(world: u32, node: u32) {
//     let node = node as usize
//     let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
// let world = &mut world.gui;
//     let overflow_clip = world.fetch_single::<OverflowClip>().unwrap();
//     js!{
//         console.log("overflow_clip:", @{format!("{:?}", &overflow_clip.value)});
//     }
// }

#[wasm_bindgen]
pub fn get_world_matrix(world: u32, node: u32) -> JsValue {
    let node = node as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let world_matrixs = world.world_matrix.lend();
    let world_matrix = match world_matrixs.get(node) {
        Some(r) => r,
        None => return JsValue::null(),
	};
	
    JsValue::from_serde(world_matrix).unwrap()
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn get_transform(world: u32, node: u32) -> JsValue {
    let node = node as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let transforms = world.transform.lend();
    let transform = match transforms.get(node) {
        Some(r) => r,
        None => return JsValue::null(),
	};
	JsValue::from_serde(transform).unwrap()
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn get_layout(world: u32, node: u32) -> JsValue {
    let node = node as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let rect_layout_style = world.rect_layout_style.lend();
	let other_layout_style = world.other_layout_style.lend();
	// let layouts = world.layout.lend();

	JsValue::from_serde(&Layout{
		rect: match rect_layout_style.get(node) {
			Some(r) => Some(r.clone()),
			None => None
		},
		other: match other_layout_style.get(node) {
			Some(r) => Some(r.clone()),
			None => None
		},
		// node_state: world.node_state.lend().get(node).clone(),
	}).unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Layout{
	pub rect: Option<RectLayoutStyle>,
	pub other: Option<OtherLayoutStyle>,
	// node_state: Option<NodeState>,
}

// #[derive(Serialize, Deserialize, Debug)]
// struct Point2{
//     x: f32,
//     y: f32,
// }
// js_serializable!( Point2 );

// impl Point2 {
//     fn new(x: f32, y: f32) -> Self {
//         Self {x, y}
//     }
// }

// #[allow(unused_attributes)]
// #[no_mangle]
// #[js_export]
// pub fn test_create_render_obj(world: u32, count: u32) {
// 	let world = unsafe {&mut *(world as usize as *mut GuiWorld)};
// 	let world = &mut world.gui;

// 	let default_state = world.world.fetch_single::<gui::single::DefaultState>().unwrap();
// 	let default_state = default_state.lend();
// 	let render_objs = world.world.fetch_single::<RenderObjs>().unwrap();
// 	let render_objs = render_objs.lend_mut();
// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj(default_state);
// 	}
// 	println!("create_render_obj: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj1(default_state);
// 	}
// 	println!("create_render_obj1: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj3(default_state);
// 	}
// 	println!("create_render_obj3: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj4(default_state);
// 	}
// 	println!("create_render_obj4: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj5(default_state);
// 	}
// 	println!("create_render_obj5: {:?}", std::time::Instant::now() - time);

// 	let mut m = map::vecmap::VecMap::default();
// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj6(&mut m, 2, render_objs, default_state);
// 	}
// 	println!("create_render_obj6: {:?}", std::time::Instant::now() - time);

// 	let mut m = map::vecmap::VecMap::default();
// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj7(&mut m, 2, render_objs, default_state);
// 	}
// 	println!("create_render_obj7: {:?}", std::time::Instant::now() - time);

// 	let p: share::Share<dyn hal_core::ProgramParamter> = share::Share::new(ImageParamter::default());
// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		create_render_obj13(&mut m, 2, render_objs, default_state, &p);
// 	}
// 	println!("create_render_obj13: {:?}", std::time::Instant::now() - time);

// 	let read = (world.copacity.lend(), world.visibility.lend(), world.hsv.lend(), world.z_depth.lend(), world.culling.lend());
// 	let render_objs = world.world.fetch_single::<gui::single::RenderObjs>().unwrap();
// 	let node_render_map = world.world.fetch_single::<gui::single::NodeRenderMap>().unwrap();
// 	let write = (render_objs.lend_mut(), node_render_map.lend_mut());
// 	let v:Option<share::Share<dyn UniformBuffer>> = Some(share::Share::new(gui::component::calc::ViewMatrixUbo::new(hal_core::UniformValue::MatrixV4(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0]))));
// 	let p:Option<share::Share<dyn UniformBuffer>> = Some(share::Share::new(gui::component::calc::ProjectMatrixUbo::new(hal_core::UniformValue::MatrixV4(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0]))));

// 	// let mut m = map::vecmap::VecMap::default();
// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		render_objs_create8((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
// 	}
// 	println!("create_render_obj8: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		render_objs_create9((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
// 	}
// 	println!("render_objs_create9: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		render_objs_create10((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
// 	}
// 	println!("render_objs_create10: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		render_objs_create11((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
// 	}
// 	println!("render_objs_create11: {:?}", std::time::Instant::now() - time);

// 	let time = std::time::Instant::now();
// 	for i in 0..count {
// 		render_objs_create12((read.0, read.1, read.2, read.3, read.4), (write.0, write.1), &v, &p);
// 	}
// 	println!("render_objs_create12: {:?}", std::time::Instant::now() - time);

// }

// //  RenderObj {
// //         depth: 0.0,
// //         program_dirty: true,
// //         visibility: false,
// //         vs_defines: Box::new(VsDefines::default()),
// //         fs_defines: Box::new(FsDefines::default()),
// //         program: None,
// //         geometry: None,
// //         depth_diff,
// //         is_opacity,
// //         vs_name,
// //         fs_name,
// //         paramter,
// //         state,
// //         context,
// //     }

// #[inline]
// pub fn create_render_obj(
//     default_state: &gui::single::DefaultState,
// ){
//     let state = gui::single::State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
//     let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	gui::system::util::new_render_obj(1, 2.0, true, gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(), gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(), share::Share::new(gui::component::calc::ImageParamter::default()), state);
// }

// #[inline]
// pub fn create_render_obj1(
//     default_state: &gui::single::DefaultState,
// ){
//     let state = gui::single::State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
//     let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();

// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// 	let notify = default_state.df_ds.clone();
// }

// #[inline]
// pub fn create_render_obj3(
//     default_state: &gui::single::DefaultState,
// ){
//     let state = gui::single::State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
//     let vs = gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone();
// 	let fs = gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone();
// 	let p = share::Share::new(gui::component::calc::ImageParamter::default());

// }

// #[inline]
// pub fn create_render_obj4(
//     default_state: &gui::single::DefaultState,
// ){
//     let state = gui::single::State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
// 	let p = share::Share::new(gui::component::calc::ImageParamter::default());

// }

// #[inline]
// pub fn create_render_obj5(
//     default_state: &gui::single::DefaultState,
// ){
//     let state = gui::single::State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
// 	share::Share::new(1);
// 	share::Share::new(1);
// 	share::Share::new(1);
// 	share::Share::new(1);
// 	share::Share::new(1);
// 	share::Share::new(1);
// 	share::Share::new(1);
// }

// #[inline]
// fn create_render_obj6(
// 	render_map: &mut map::vecmap::VecMap<usize>,
// 	id: usize,
// 	render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
// 	default_state: &DefaultState,
// ) -> usize{
// 	gui::system::util::create_render_obj(
// 		id,
// 		-0.1,
// 		true,
// 		gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(),
// 		gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(),
// 		share::Share::new(ImageParamter::default()),
// 		default_state, render_objs,
// 		render_map
// 	)
// }

// #[inline]
// fn create_render_obj7(
// 	render_map: &mut map::vecmap::VecMap<usize>,
// 	id: usize,
// 	render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
// 	default_state: &DefaultState,
// ) -> usize{
// 	create_render_obj_(
// 		id,
// 		-0.1,
// 		true,
// 		gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(),
// 		gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(),
// 		share::Share::new(ImageParamter::default()),
// 		default_state, render_objs,
// 		render_map
// 	)
// }

// #[inline]
// pub fn create_render_obj_(
//     context: usize,
//     depth_diff: f32,
//     is_opacity: bool,
//     vs_name: atom::Atom,
//     fs_name: atom::Atom,
//     paramter: share::Share<dyn ProgramParamter>,
//     default_state: &DefaultState,
//     render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
//     render_map: &mut map::vecmap::VecMap<usize>,
// ) -> usize{
//     let state = gui::single::State {
//         bs: default_state.df_bs.clone(),
//         rs: default_state.df_rs.clone(),
//         ss: default_state.df_ss.clone(),
//         ds: default_state.df_ds.clone(),
//     };
//     let notify = render_objs.get_notify();
//     let render_index = render_objs.insert(
//         gui::system::util::new_render_obj(context, depth_diff, is_opacity, vs_name, fs_name, paramter, state),
//         None
//     );
//     render_map.insert(context, render_index);
//     render_index
// }

// fn render_objs_create8<'a>(read: (
//         &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
//         &'a ecs::MultiCaseImpl<Node, Visibility>,
//         &'a ecs::MultiCaseImpl<Node, HSV>,
//         &'a ecs::MultiCaseImpl<Node, ZDepth>,
//         &'a ecs::MultiCaseImpl<Node, Culling>,
//     ),
// 	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
// 	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// 	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// ) {
// 	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
// 	let (render_objs, node_render_map) = write;
// 	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
// 	let notify = node_render_map.get_notify();
// 	unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };

// 	let paramter = &mut render_obj.paramter;

// 	paramter.set_value("viewMatrix", view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
// 	paramter.set_value("projectMatrix", project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

// 	let z_depth = unsafe { z_depths.get_unchecked(render_obj.context) }.0;
// 	let opacity = unsafe { opacitys.get_unchecked(render_obj.context) }.0;
// 	paramter.set_single_uniform("alpha", UniformValue::Float1(opacity)); // alpha
// 	println!("id: {}, alpha: {:?}", render_obj.context, opacity);

// 	let visibility = unsafe { visibilitys.get_unchecked(render_obj.context) }.0;
// 	let culling = unsafe { cullings.get_unchecked(render_obj.context) }.0;
// 	render_obj.visibility = visibility & !culling;

// 	render_obj.depth = z_depth + render_obj.depth_diff;

// 	let hsv = unsafe { hsvs.get_unchecked(render_obj.context) };
// 	if !(hsv.h == 0.0 && hsv.s == 0.0 && hsv.v == 0.0) {
// 		render_obj.fs_defines.add("HSV");
// 		// paramter.set_value("hsvValue", self.create_hsv_ubo(hsv)); // hsv
// 	}
// }

// fn render_objs_create9<'a>(read: (
//         &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
//         &'a ecs::MultiCaseImpl<Node, Visibility>,
//         &'a ecs::MultiCaseImpl<Node, HSV>,
//         &'a ecs::MultiCaseImpl<Node, ZDepth>,
//         &'a ecs::MultiCaseImpl<Node, Culling>,
//     ),
// 	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
// 	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// 	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// ) {
// 	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
// 	let (render_objs, node_render_map) = write;
// 	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
// 	let notify = node_render_map.get_notify();
// 	unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };

// 	let paramter = &mut render_obj.paramter;

// 	paramter.set_value("viewMatrix", view_matrix_ubo.clone().unwrap()); // VIEW_MATRIX
// 	paramter.set_value("projectMatrix", project_matrix_ubo.clone().unwrap()); // PROJECT_MATRIX

// }

// fn render_objs_create10<'a>(read: (
//         &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
//         &'a ecs::MultiCaseImpl<Node, Visibility>,
//         &'a ecs::MultiCaseImpl<Node, HSV>,
//         &'a ecs::MultiCaseImpl<Node, ZDepth>,
//         &'a ecs::MultiCaseImpl<Node, Culling>,
//     ),
// 	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
// 	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// 	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// ) {
// 	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
// 	let (render_objs, node_render_map) = write;
// 	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
// 	let notify = node_render_map.get_notify();
// 	unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };
// }

// fn render_objs_create11<'a>(read: (
//         &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
//         &'a ecs::MultiCaseImpl<Node, Visibility>,
//         &'a ecs::MultiCaseImpl<Node, HSV>,
//         &'a ecs::MultiCaseImpl<Node, ZDepth>,
//         &'a ecs::MultiCaseImpl<Node, Culling>,
//     ),
// 	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
// 	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// 	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// ) {
// 	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
// 	let (render_objs, node_render_map) = write;
// 	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
// 	let notify = node_render_map.get_notify();
// 	// unsafe{ node_render_map.add_unchecked(render_obj.context, 3, &notify) };
// }

// fn render_objs_create12<'a>(read: (
//         &'a ecs::MultiCaseImpl<Node, gui::component::calc::Opacity>,
//         &'a ecs::MultiCaseImpl<Node, Visibility>,
//         &'a ecs::MultiCaseImpl<Node, HSV>,
//         &'a ecs::MultiCaseImpl<Node, ZDepth>,
//         &'a ecs::MultiCaseImpl<Node, Culling>,
//     ),
// 	write: (&'a mut ecs::SingleCaseImpl<RenderObjs>, &'a mut ecs::SingleCaseImpl<NodeRenderMap>),
// 	view_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// 	project_matrix_ubo: &Option<share::Share<dyn UniformBuffer>>,
// ) {
// 	let (opacitys, visibilitys, hsvs, z_depths, cullings) = read;
// 	let (render_objs, node_render_map) = write;
// 	let render_obj = unsafe { render_objs.get_unchecked_mut(3) };
// 	let notify = node_render_map.get_notify();
// 	// unsafe{ node_render_map.add_unchecked(render_obj.context, 3, notify) };
// }

// #[inline]
// fn create_render_obj13(
// 	render_map: &mut map::vecmap::VecMap<usize>,
// 	id: usize,
// 	render_objs: &mut ecs::SingleCaseImpl<RenderObjs>,
// 	default_state: &DefaultState,
// 	p: &share::Share<dyn hal_core::ProgramParamter>
// ) -> usize{
// 	create_render_obj_(
// 		id,
// 		-0.1,
// 		true,
// 		gui::system::render::shaders::image::IMAGE_VS_SHADER_NAME.clone(),
// 		gui::system::render::shaders::image::IMAGE_FS_SHADER_NAME.clone(),
// 		p.clone(),
// 		default_state, render_objs,
// 		render_map
// 	)
// }
