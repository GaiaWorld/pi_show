use ecs::LendMut;

use bc::*;
use gui::component::calc::{StyleType, StyleType1};
use gui::component::user::*;
use gui::layout::*;
use GuiWorld;

#[macro_use()]
macro_rules! delete_value {
    (
		$world:ident, 
		$node_id:ident, 
		$key:ident, 
		$csy:ident,/*class_style | class_style1*/ 
		$lsy:ident,/*local_style | local_style1*/ 
		$lsyty:expr/*StyleType::XX |*/) => {
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        let node_id = $node_id as usize;

        let ty = $lsyty as usize;
        let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
        if (style_mark.$lsy & ty) == ty {
            style_mark.$lsy &= std::usize::MAX - ty;
            world.$key.lend_mut().delete(node_id);
        }
    };
}

#[macro_use()]
macro_rules! reset_value {
    (
		$world:ident,
		$node_id:ident,
		$key:ident,
		$value:expr,
		$csy:ident,/*class_style | class_style1*/
		$lsy:ident,/*local_style | local_style1*/
		$lsyty:expr/*StyleType::XX |*/) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;

        let ty = $lsyty as usize;
        let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
        if (style_mark.$lsy & ty) == ty {
            style_mark.$lsy &= std::usize::MAX - ty;
            world.$key.lend_mut().insert(node_id, $value);
        }
    };
}

#[macro_use()]
macro_rules! reset_text_attr {
    (
		$world:ident,
		$node_id:ident,
		$key:ident,
		$csy:ident,/*class_style | class_style1*/ 
		$lsy:ident, /*local_style | local_style1*/
		$lsyty:expr, /*StyleType::XX |*/
		$name:ident,
		$name1:ident,
		$name2:expr,
		$value:expr,
	) => {
        let node_id = $node_id as usize;
        let world = unsafe { &mut *($world as usize as *mut GuiWorld) };
        let world = &mut world.gui;
        let attr = world.$key.lend_mut();
        let v = $value;
        let ty = $lsyty as usize;

        let style_mark = unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) };
        if style_mark.$lsy & ty == ty {
            $crate::paste::item! {
                style_mark.$lsy &= std::usize::MAX - ty;
                let r = unsafe { attr.get_unchecked_mut(node_id) };
                r.$name.$name1 = v;
                attr.get_notify_ref().modify_event(node_id, $name2, 0);
            }
        }
    };
}

/// 设置transform_will_change
#[allow(unused_attributes)]
#[no_mangle]
#[js_export]
pub fn reset_style(world: u32, node_id: u32, ty: u32) {
    let defult_text = unsafe { &mut *(world as usize as *mut GuiWorld) }
        .default_text_style
        .clone();
    // let node_id = node_id as usize;

    let ty: StyleAttr = unsafe { std::mem::transmute_copy(&ty) };
    let undefined = std::f32::NAN;
    match ty {
        // StyleAttr::Text => (),//reset_text_attr!(world, node_id, text_style, class_style,local_style, StyleType::Text, text,),
        StyleAttr::FontStyle => {
            let v = defult_text.font.style;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::FontStyle,
                font,
                style,
                "font_style",
                v,
            );
        }
        StyleAttr::FontWeight => {
            let v = defult_text.font.weight;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::FontWeight,
                font,
                weight,
                "font_weight",
                v,
            );
        }
        StyleAttr::FontSize => {
            let v = defult_text.font.size;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::FontSize,
                font,
                size,
                "font_size",
                v,
            );
        }
        StyleAttr::FontFamily => {
            let v = defult_text.font.family;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::FontFamily,
                font,
                family,
                "font_family",
                v,
            );
        }
        StyleAttr::LetterSpacing => {
            let v = defult_text.text.letter_spacing;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::LetterSpacing,
                text,
                letter_spacing,
                "letter_spacing",
                v,
            );
        }
        StyleAttr::WordSpacing => {
            let v = defult_text.text.word_spacing;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::WordSpacing,
                text,
                word_spacing,
                "word_spacing",
                v,
            );
        }
        StyleAttr::LineHeight => {
            let v = defult_text.text.line_height;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::LineHeight,
                text,
                line_height,
                "line_height",
                v,
            );
        }
        StyleAttr::Indent => {
            let v = defult_text.text.indent;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::Indent,
                text,
                indent,
                "indent",
                v,
            );
        }
        StyleAttr::WhiteSpace => {
            let v = defult_text.text.white_space;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::WhiteSpace,
                text,
                white_space,
                "white_space",
                v,
            );
        }
        StyleAttr::TextAlign => {
            let v = defult_text.text.text_align;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::TextAlign,
                text,
                text_align,
                "text_align",
                v,
            );
        }
        StyleAttr::VerticalAlign => {
            let v = defult_text.text.vertical_align;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::VerticalAlign,
                text,
                vertical_align,
                "vertical_align",
                v,
            );
        }
        StyleAttr::Color => {
            let v = defult_text.text.color;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::Color,
                text,
                color,
                "color",
                v,
            );
        }
        StyleAttr::Stroke => {
            let v = defult_text.text.stroke;
            reset_text_attr!(
                world,
                node_id,
                text_style,
                class_style,
                local_style,
                StyleType::Stroke,
                text,
                stroke,
                "stroke",
                v,
            );
        }
        // StyleAttr::TextShadow => (),//reset_text_attr!(world, node_id, text_style, class_style,local_style, StyleType::Text, font, font_style, "font_style",defult_text.shadow.stroke),
        StyleAttr::Image => {
            let v = StyleType::Image;
            delete_value!(world, node_id, image, class_style, local_style, v);
        }
        StyleAttr::ImageClip => {
            let v = StyleType::ImageClip;
            delete_value!(world, node_id, image_clip, class_style, local_style, v);
        }
        StyleAttr::ObjectFit => {
            let v = StyleType::ObjectFit;
            delete_value!(world, node_id, object_fit, class_style, local_style, v);
        }

        StyleAttr::BorderImage => {
            let v = StyleType::BorderImage;
            delete_value!(world, node_id, border_image, class_style, local_style, v);
        }
        StyleAttr::BorderImageClip => {
            let v = StyleType::BorderImageClip;
            delete_value!(
                world,
                node_id,
                border_image_clip,
                class_style,
                local_style,
                v
            );
        }
        StyleAttr::BorderImageSlice => {
            let v = StyleType::BorderImageSlice;
            delete_value!(
                world,
                node_id,
                border_image_slice,
                class_style,
                local_style,
                v
            );
        }
        StyleAttr::BorderImageRepeat => {
            let v = StyleType::BorderImageRepeat;
            delete_value!(
                world,
                node_id,
                border_image_repeat,
                class_style,
                local_style,
                v
            );
        }

        StyleAttr::BorderColor => {
            let v = StyleType::BorderColor;
            delete_value!(world, node_id, border_color, class_style, local_style, v);
        }

        StyleAttr::BackgroundColor => {
            let v = StyleType::BackgroundColor;
            delete_value!(
                world,
                node_id,
                background_color,
                class_style,
                local_style,
                v
            );
        }

        StyleAttr::BoxShadow => {
            let v = StyleType::BoxShadow;
            delete_value!(world, node_id, box_shadow, class_style, local_style, v);
        }
        // StyleType::Matrix => 0x2000000,
        StyleAttr::Opacity => {
            let v = StyleType::Opacity;
            reset_value!(
                world,
                node_id,
                opacity,
                Opacity::default(),
                class_style1,
                local_style1,
                v
            );
        }
        // StyleType::Layout => 0x8000000,
        StyleAttr::BorderRadius => {
            let v = StyleType::BorderRadius;
            reset_value!(
                world,
                node_id,
                border_radius,
                BorderRadius {
                    x: LengthUnit::Pixel(0.0),
                    y: LengthUnit::Pixel(0.0),
                },
                class_style,
                local_style,
                v
            );
        }
        // StyleType::ByOverflow => 0x20000000,
        StyleAttr::Filter => {
            let v = StyleType::Filter;
            reset_value!(
                world,
                node_id,
                filter,
                Filter::default(),
                class_style,
                local_style,
                v
            );
        }

        StyleAttr::Width => reset_layout(world, node_id, StyleType1::Width, |n| {
            n.set_width(undefined);
        }),
        StyleAttr::Height => reset_layout(world, node_id, StyleType1::Height, |n| {
            n.set_height(undefined);
        }),

        StyleAttr::Margin => reset_layout(world, node_id, StyleType1::Margin, |n| {
            n.set_margin(YGEdge::YGEdgeAll, undefined);
        }),
        // StyleAttr::MarginTop => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
        // StyleAttr::MarginRight => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
        // StyleAttr::MarginBottom => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
        // StyleAttr::MarginLeft => reset_layout(world, node_id, StyleType1::Margin, |n|{n.set_margin(YGEdge::YGEdgeTop, undefined)}),
        StyleAttr::Padding => reset_layout(world, node_id, StyleType1::Padding, |n| {
            n.set_padding(YGEdge::YGEdgeAll, undefined);
        }),
        // StyleAttr::PaddingTop => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeTop, undefined)}),
        // StyleAttr::PaddingRight => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeRight, undefined)}),
        // StyleAttr::PaddingBottom => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeBootom, undefined)}),
        // StyleAttr::PaddingLeft => reset_layout(world, node_id, StyleType1::Padding, |n|{n.set_padding(YGEdge::YGEdgeLeft, undefined)}),
        StyleAttr::Border => reset_layout(world, node_id, StyleType1::Border, |n| {
            n.set_border(YGEdge::YGEdgeAll, undefined);
        }),
        // StyleAttr::BorderTop => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeTop, undefined)}),
        // StyleAttr::BorderRight => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeRight, undefined)}),
        // StyleAttr::BorderBottom => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeBootom, undefined)}),
        // StyleAttr::BorderLeft => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_border(YGEdge::YGEdgeLeft, undefined)}),
        StyleAttr::Top => reset_layout(world, node_id, StyleType1::Position, |n| {
            n.set_position(YGEdge::YGEdgeTop, undefined);
        }),
        StyleAttr::Right => reset_layout(world, node_id, StyleType1::Position, |n| {
            n.set_position(YGEdge::YGEdgeRight, undefined);
        }),
        StyleAttr::Bottom => reset_layout(world, node_id, StyleType1::Position, |n| {
            n.set_position(YGEdge::YGEdgeBottom, undefined);
        }),
        StyleAttr::Left => reset_layout(world, node_id, StyleType1::Position, |n| {
            n.set_position(YGEdge::YGEdgeLeft, undefined);
        }),

        StyleAttr::MinWidth => reset_layout(world, node_id, StyleType1::MinWidth, |n| {
            n.set_min_width(undefined);
        }),
        StyleAttr::MinHeight => reset_layout(world, node_id, StyleType1::MinHeight, |n| {
            n.set_min_height(undefined);
        }),
        StyleAttr::MaxHeight => reset_layout(world, node_id, StyleType1::MaxHeight, |n| {
            n.set_max_height(undefined)
        }),
        StyleAttr::MaxWidth => reset_layout(world, node_id, StyleType1::MaxWidth, |n| {
            n.set_max_width(undefined);
        }),

        StyleAttr::FlexBasis => reset_layout(world, node_id, StyleType1::FlexBasis, |n| {
            n.set_flex_basis_auto();
        }),
        StyleAttr::FlexShrink => reset_layout(world, node_id, StyleType1::FlexShrink, |n| {
            n.set_flex_shrink(0.0);
        }),
        StyleAttr::FlexGrow => reset_layout(world, node_id, StyleType1::FlexGrow, |n| {
            n.set_flex_grow(0.0)
        }),
        StyleAttr::PositionType => reset_layout(world, node_id, StyleType1::PositionType, |n| {
            n.set_position_type(YGPositionType::YGPositionTypeRelative);
        }),
        StyleAttr::FlexWrap => reset_layout(world, node_id, StyleType1::FlexWrap, |n| {
            n.set_flex_wrap(YGWrap::YGWrapWrap);
        }),
        StyleAttr::FlexDirection => reset_layout(world, node_id, StyleType1::FlexDirection, |n| {
            n.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
        }),
        StyleAttr::AlignContent => reset_layout(world, node_id, StyleType1::AlignContent, |n| {
            n.set_align_content(YGAlign::YGAlignFlexStart);
        }),
        StyleAttr::AlignItems => reset_layout(world, node_id, StyleType1::AlignItems, |n| {
            n.set_align_items(YGAlign::YGAlignFlexStart);
        }),
        StyleAttr::AlignSelf => reset_layout(world, node_id, StyleType1::AlignSelf, |n| {
            n.set_align_self(YGAlign::YGAlignFlexStart);
        }),
        StyleAttr::JustifyContent => {
            reset_layout(world, node_id, StyleType1::JustifyContent, |n| {
                n.set_justify_content(YGJustify::YGJustifyFlexStart);
            })
        }

        // StyleAttr::Display => reset_layout(world, node_id, StyleType1::Position, |n|{n.set_width()}),
        // StyleAttr::Visibility => reset_value!(world, text_style, Visibility::default(), class_style1,local_style1, StyleType1::Visibility, visibility),
        // StyleAttr::Enable => reset_value!(world, text_style, Enable::default(), class_style1,local_style1, StyleType1::Enable, enable),
        StyleAttr::ZIndex => {
            let v = StyleType1::ZIndex;
            reset_value!(
                world,
                node_id,
                z_index,
                ZIndex::default(),
                class_style1,
                local_style1,
                v
            );
        }
        StyleAttr::Transform => {
            let v = StyleType1::Transform;
            reset_value!(
                world,
                node_id,
                transform,
                Transform::default(),
                class_style1,
                local_style1,
                v
            );
        }
        // StyleAttr::TransformWillChange => reset_value!(world, text_style, Filter::default(), class_style,local_style, StyleType::Filter, filter),
        StyleAttr::Overflow => {
            let v = StyleType1::Overflow;
            reset_value!(
                world,
                node_id,
                overflow,
                Overflow::default(),
                class_style1,
                local_style1,
                v
            );
        }
        _ => (),
    }
}

fn reset_layout<T: FnOnce(&mut YgNode)>(world: u32, node_id: u32, ty: StyleType1, call_back: T) {
    // let edge = unsafe { transmute(edge) };
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let ty = ty as usize;
    if (unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 & ty) == ty {
        unsafe { world.style_mark.lend_mut().get_unchecked_mut(node_id) }.local_style1 &=
            std::usize::MAX - ty;
        unsafe { world.yoga.lend_mut().get_unchecked_write(node_id) }.modify(|s| {
            call_back(s);
            true
        })
    }
}

enum StyleAttr {
    FontStyle,
    FontWeight,
    FontSize,
    FontFamily,
    LetterSpacing,
    WordSpacing,
    LineHeight,
    Indent,
    WhiteSpace,
    TextAlign,
    VerticalAlign,
    Color,
    Stroke,
    TextShadow,

    BackgroundColor,
    Image,
    ImageClip,
    ObjectFit,
    BorderImage,
    BorderImageClip,
    BorderImageSlice,
    BorderImageRepeat,
    BorderColor,
    BoxShadow,

    Opacity,
    BorderRadius,
    Filter,

    Width,
    Height,

    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,

    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,

    Border,
    BorderTop,
    BorderRight,
    BorderBottom,
    BorderLeft,

    Top,
    Right,
    Bottom,
    Left,

    MinWidth,
    MinHeight,

    MaxWidth,
    MaxHeight,

    FlexBasis,
    FlexShrink,
    FlexGrow,
    PositionType,
    FlexWrap,
    FlexDirection,
    AlignContent,
    AlignItems,
    AlignSelf,
    JustifyContent,

    Display,
    Visibility,
    Enable,
    Overflow,
    ZIndex,
    Transform,
    TransformWillChange,
}
