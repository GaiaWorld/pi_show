use wcs::component::{Event, ComponentGroup, ComponentGroupTree, notify, Builder};
use std::ops::{Deref};
use std::default::{Default};
use wcs::world::{ComponentMgr};

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Default, Builder)]
pub struct Font{
    style: FontStyle, //	规定字体样式。参阅：font-style 中可能的值。
    // font-variant	规定字体异体。参阅：font-variant 中可能的值。
    weight:FontWeight, //	规定字体粗细。参阅：font-weight 中可能的值。
    size: FontSize, //
    family: String, //	规定字体系列。参阅：font-family 中可能的值。
    // caption	定义被标题控件（比如按钮、下拉列表等）使用的字体。
    // icon	定义被图标标记使用的字体。
    // menu	定义被下拉列表使用的字体。
    // message-box	定义被对话框使用的字体。
    // small-caption	caption 字体的小型版本。
    // status-bar
}

rc!(#[derive(Debug)]RcFont, Font, FONT_SLAB);

impl Default for RcFont {
    fn default() -> RcFont {
        RcFont::new(Font::default())
    }
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontStyle{
    Normal, //	默认值。标准的字体样式。
    Ttalic, //	斜体的字体样式。
    Oblique, //	倾斜的字体样式。
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontWeight{
    Normal, //	默认值。定义标准的字符。
    Bold, // 定义粗体字符。
    Bolder, //	定义更粗的字符。
    Lighter, //	定义更细的字符。
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine, //400 等同于 normal，而 700 等同于 bold。
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontSize {
    Medium,
    XXSmall,    //把字体的尺寸设置为不同的尺寸，从 xx-small 到 xx-large。
    XSmall,
    Small,
    Large,
    XLarge,
    XXLarge,
    Smaller,	//把 font-size 设置为比父元素更小的尺寸。
    Larger,	//把 font-size 设置为比父元素更大的尺寸。
    Length(f32),	//把 font-size 设置为一个固定的值。
    Percent(f32), //把 font-size 设置为基于父元素的一个百分比值。
}