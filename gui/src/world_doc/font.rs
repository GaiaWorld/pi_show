use std::default::Default;

use fnv::FnvHashMap;

use atom::{Atom};

use component::math::{UV};
use component::color::Color;

pub const FONT_SIZE: f32 = 32.0;

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontSize {
    None,	// 默认尺寸。
    Length(f32),	//把 font-size 设置为一个固定的值。
    Percent(f32), //把 font-size 设置为基于父元素的一个百分比值。
}

/// 字体表 使用SDF(signed distance field 有向距离场)渲染字体， 支持预定义字体纹理及配置， 也支持动态计算字符的SDF
pub struct FontSheet {
    font_size: f32,
    font_color: Color,
    font_src: FnvHashMap<Atom, SdfFont>,
    font_face: FnvHashMap<Atom, FontFace>,
}
impl Default for FontSheet {
    fn default() -> FontSheet {
        FontSheet {
            font_size: FONT_SIZE,
            font_color: Color::default(),
            font_src: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            font_face: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
        }
    }
}

impl FontSheet {
    // 设置默认字号
    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
    // 设置默认字色
    pub fn set_font_color(&mut self, font_color: Color) {
        self.font_color = font_color;
    }
    // 设置SDFFont
    pub fn set_font_src(&mut self, name: Atom, sdf: SdfFont) {
        self.font_src.insert(name, sdf);
    }
    // 设置FontFace
    pub fn set_font_face(&mut self, family: Atom, oblique: f32, font_size: f32, font_weight: f32, src: String) {
        let mut v = Vec::new();
        for s in src.split(',') {
            v.push(Atom::from(s.trim_start().trim_end()))
        }
        let face = FontFace {
            oblique: oblique,
            font_size: font_size,
            font_weight: font_weight,
            src: v,
        };
        self.font_face.insert(family, face);
    }
    //  获得字体大小, 0表示没找到该font_face
    pub fn get_font_size(&self, font_face: &Atom, font_size: FontSize) -> f32 {
        match self.font_face.get(font_face) {
            Some(face) => face.size(font_size),
            _ => 0.0
        }
    }

    // 测量指定字体下，指定字符的宽度。 0表示没有该字符
    pub fn measure(&self, font_face: &Atom, font_size: FontSize, c: char) -> f32 {
        match self.font_face.get(font_face) {
            Some(face) => {
                let size = face.size(font_size);
                for name in &face.src {
                    match self.font_src.get(name) {
                        Some(font) => {
                            let r = font.measure(name, c);
                            if r > 0.0 {
                                return r * font.size / size
                            }
                        },
                        _ => ()
                    }
                }
                0.0
            },
            _ => 0.0
        }
    }
}


pub trait FontHelper {
    // 获得纹理句柄
    fn get_texture(&self) -> usize;
    // 获得纹理大小
    fn get_texture_size(&self) -> cg::Vector2<usize>;
    // 异步计算字符的sdf数据的函数, 返回None表示异步设置该字符，否则返回该字符的的sdf数据
    fn sdf(&self, font_name: &Atom, font: &mut SdfFont, c: char) -> Option<Result<(), usize>>;
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_name: &Atom, font_size: f32, c: char) -> f32;
}

// 静态字体实现 TODO 应该是渲染模块实现
struct StaticFont {
    pub texture: usize, // 字体纹理的句柄
    pub texture_size: cg::Vector2<usize>, // 字体纹理的大小
    pub index: usize, // 纹理当前索引
    pub cur_pos: usize, // 当前可写字符的位置
}


impl FontHelper for StaticFont {
    // 获得纹理句柄
    fn get_texture(&self) -> usize {
        self.texture
    }
    // 获得纹理大小
    fn get_texture_size(&self) -> cg::Vector2<usize> {
        self.texture_size
    }
    // 异步计算字符的sdf数据的函数, 返回None表示异步设置该字符，否则返回该字符的的sdf数据
    fn sdf(&self, _font_name: &Atom, _font: &mut SdfFont, _c: char) -> Option<Result<(), usize>> {
        Some(Err(0))
    }
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, _font_name: &Atom, _font_size: f32, _c: char) -> f32 {
        0.0
    }
}

pub struct SdfFont {
    pub size: f32, //SDF大小, 一般32像素
    pub fixed_width: f32, //字符的固定宽度, 0为非定宽
    pub char_uv_map: FnvHashMap<char, UV>, // 每字符的UV，可以通过UV计算宽度， 如果字高为0，表示字符正在创建
    pub helper: Box<dyn FontHelper>,
}
impl SdfFont {
    pub fn new(mut size: f32, fixed_width: f32, helper: Box<dyn FontHelper>) -> SdfFont {
        if size == 0.0 {
            size = FONT_SIZE;
        }
        SdfFont {
            size: size,
            fixed_width: fixed_width,
            char_uv_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            helper: helper,
        }
    }
    // 测量指定字体下，指定字符的宽度。 0表示没有该字符
    pub fn measure(&self, name: &Atom, c: char) -> f32 {
        if self.fixed_width > 0.0 {
            return self.fixed_width
        }
        match self.char_uv_map.get(&c) {
            Some(uv) => (uv.u2 - uv.u1) * (self.helper.get_texture_size().x as f32),
            _ => self.helper.measure(name, self.size, c)
        }
    }
}
// 字体表现
#[derive(Default, Debug)]
pub struct FontFace {
    oblique: f32,
    font_size: f32,
    font_weight: f32,
    src: Vec<Atom>,
}
impl FontFace {
    pub fn size(&self, s:FontSize) -> f32 {
        match s {
            FontSize::None => self.font_size,
            FontSize::Length(r) => r,
            FontSize::Percent(r) => r * self.font_size
        }
    }
}
// 倾斜度造成的间距
pub fn oblique_spacing(oblique: f32, font_size: f32, char_width: f32) -> f32 {
    oblique * font_size * char_width // TODO FIX!!!
}
