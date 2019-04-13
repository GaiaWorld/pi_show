use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Builder};
use std::ops::{Deref};
use std::default::{Default};
use wcs::world::{ComponentMgr};
use atom::{Atom};

use world_doc::font::{FontSize};

#[allow(unused_attributes)]
#[derive(Component, Debug, Clone, Default, Builder)]
pub struct Font{
    pub style: FontStyle, //	规定字体样式。参阅：font-style 中可能的值。
    // font-variant	规定字体异体。参阅：font-variant 中可能的值。
    pub weight: f32, //	规定字体粗细。参阅：font-weight 中可能的值。
    pub size: FontSize, //
    pub family: Atom, //	规定字体系列。参阅：font-family 中可能的值。
    // caption	定义被标题控件（比如按钮、下拉列表等）使用的字体。
    // icon	定义被图标标记使用的字体。
    // menu	定义被下拉列表使用的字体。
    // message-box	定义被对话框使用的字体。
    // small-caption	caption 字体的小型版本。
    // status-bar
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontStyle{
    Normal, //	默认值。标准的字体样式。
    Ttalic, //	斜体的字体样式。
    Oblique, //	倾斜的字体样式。
}

