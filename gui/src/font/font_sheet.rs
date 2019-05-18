use std::default::Default;
use std::str::Chars;
use std::sync::Arc;

use fnv::FnvHashMap;

use atom::{Atom};
use ucd::{Codepoint};

use component::CgColor as Color;
use font::sdf_font::SdfFont;

pub const FONT_SIZE: f32 = 32.0;

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontSize {
    None,	// 默认尺寸。
    Length(f32),	//把 font-size 设置为一个固定的值。
    Percent(f32), //把 font-size 设置为基于父元素的一个百分比值。
}

//设置行高
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum LineHeight{
    Normal, //设置合理的行间距（等于font-size）
    Length(f32), //固定像素
    Number(f32), //设置数字，此数字会与当前的字体尺寸相乘来设置行间距。
    Percent(f32),   //	基于当前字体尺寸的百分比行间距.
}

/// 字体表 使用SDF(signed distance field 有向距离场)渲染字体， 支持预定义字体纹理及配置， 也支持动态计算字符的SDF
pub struct FontSheet {
    size: f32,
    color: Color,
    src_map: FnvHashMap<Atom, Arc<SdfFont>>,
    face_map: FnvHashMap<Atom, FontFace>,
}

impl Default for FontSheet {
    fn default() -> FontSheet {
        FontSheet {
            size: FONT_SIZE,
            color: Color::default(),
            src_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            face_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
        }
    }
}

impl FontSheet {
    // 设置默认字号
    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }
    // 设置默认字色
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    // 设置SDFFont
    pub fn set_src(&mut self, name: Atom, src: Arc<SdfFont>) {
        self.src_map.insert(name, src);
    }
    
    // 设置FontFace
    pub fn set_face(&mut self, family: Atom, oblique: f32, size: f32, weight: f32, src: String) {
        let mut v = Vec::new();
        for s in src.split(',') {
            v.push(Atom::from(s.trim_start().trim_end()))
        }
        let face = FontFace {
            oblique: oblique,
            size: size,
            weight: weight,
            src: v,
        };
        self.face_map.insert(family, face);
    }
    //  获得字体大小, 0表示没找到该font_face
    pub fn get_size(&self, font_face: &Atom, size: &FontSize) -> f32 {
        match self.face_map.get(font_face) {
            Some(face) => get_size(face.size, size),
            _ => 0.0
        }
    }
    // 行高
    pub fn get_line_height(&self, font_face: &Atom, line_height: &LineHeight) -> f32 {
        match self.face_map.get(font_face) {
            Some(face) => get_line_height(face.size, line_height),
            _ => 0.0
        }
    }

    // 测量指定字体下，指定字符的宽度。 0表示没有该字符
    pub fn measure(&self, font_face: &Atom, size: f32, c: char) -> f32 {
        match self.face_map.get(font_face) {
            Some(face) => {
                for name in &face.src {
                    match self.src_map.get(name) {
                        Some(font) => return font.measure(size, c),
                        _ => ()
                    }
                }
                0.0
            },
            _ => 0.0
        }
    }

    pub fn get_first_font(&self, font_face: &Atom) -> Option<Arc<SdfFont>>{
        match self.face_map.get(font_face) {
            Some(face) => {
                for name in &face.src {
                    match self.src_map.get(name) {
                        Some(font) => return Some(font.clone()),
                        _ => ()
                    }
                }
                None
            },
            _ => None
        }
    }
}

// 字体表现
#[derive(Default, Debug)]
pub struct FontFace {
    oblique: f32,
    size: f32,
    weight: f32,
    src: Vec<Atom>,
}

pub fn get_size(size:f32, s:&FontSize) -> f32 {
    match s {
        &FontSize::None => size,
        &FontSize::Length(r) => r,
        &FontSize::Percent(r) => r * size
    }
}
// 行高
pub fn get_line_height(size:f32, line_height: &LineHeight) -> f32 {
    match line_height {
        LineHeight::Length(r) => *r, //固定像素
        LineHeight::Number(r) => *r + size, //设置数字，此数字会与当前的字体尺寸相加来设置行间距。
        LineHeight::Percent(r) => *r * size,   //	基于当前字体尺寸的百分比行间距.
        LineHeight::Normal => size,
    }
}
// // 倾斜度造成的间距
// pub fn oblique_spacing(oblique: f32, font_size: f32, char_width: f32) -> f32 {
//     oblique * font_size * char_width // TODO FIX!!!
// }

// 劈分结果
pub enum SplitResult {
    Newline,
    Whitespace,
    Word(char), // 单字词
    WordStart(char), // 单词开始, 连续的字母或数字(必须字符的type_id相同)组成的单词
    WordNext(char), // 单词字符继续
    WordEnd, // 单词字符结束
}
// 劈分字符迭代器
pub struct SplitChar<'a> {
    iter: Chars<'a>,
    word_split: bool,
    merge_whitespace: bool,
    last: Option<char>,
    type_id: usize, // 0表示单字词, 1表示ascii字母 2及以上代表字符的type_id, MAX表示数字
}

impl<'a> Iterator for SplitChar<'a> {
    type Item = SplitResult;
    fn next(&mut self) -> Option<Self::Item> {
        match self.last {
            Some(c) if self.type_id == 0 => {
                if c == '\n' {
                    self.last = self.iter.next();
                    Some(SplitResult::Newline)
                }else if c.is_whitespace() {
                    if self.merge_whitespace {
                        loop {
                            match self.iter.next() {
                                Some(cc) if cc.is_whitespace() => continue,
                                r => {
                                    self.last = r;
                                    break;
                                }
                            }
                        }
                    }else {
                        self.last = self.iter.next();
                    }
                    Some(SplitResult::Whitespace)
                }else if !self.word_split {
                    self.last = self.iter.next();
                    Some(SplitResult::Word(c))
                }else {
                    self.type_id = get_type_id(c, char::from(0));
                    if self.type_id == 0 {
                        self.last = self.iter.next();
                        Some(SplitResult::Word(c))
                    }else{
                        // 如果是单词开始，不读取下个字符，因为需要保留当前字符做是否为单词的判断
                        Some(SplitResult::WordStart(c))
                    }
                }
            },
            Some(old_c) => {
                self.last = self.iter.next();
                match self.last {
                    Some(c) => {
                        let id = get_type_id(c, old_c);
                        if id == self.type_id {
                            Some(SplitResult::WordNext(c))
                        }else{
                            self.type_id = 0;
                            Some(SplitResult::WordEnd)
                        }
                    },
                    _ => Some(SplitResult::WordEnd)
                }
               
            },
            _ => None
        }
    }
}
/// 数字或字母, 返回对应的类型
fn get_type_id(c: char, prev: char) -> usize {
    if c.is_ascii() {
        if c.is_ascii_alphabetic() {
            return 1
        }else if c.is_ascii_digit() {
            return usize::max_value()
        }else if c == '/' || c == '.' || c == '%' {
            if prev.is_ascii_digit() {
                return usize::max_value()
            }
        }else if c == '\'' {
            if prev.is_ascii_alphabetic() {
                return 1
            }
        }
    }else if c.is_alphabetic() && !c.is_cased() {
        return c.get_type_id()
    }
    0
}
/// 劈分字符串, 返回字符迭代器
pub fn split<'a>(s: &'a str, word_split: bool, merge_whitespace: bool) -> SplitChar<'a> {
    let mut i = s.chars();
    let last = i.next();
    SplitChar {
        iter: i,
        word_split: word_split,
        merge_whitespace: merge_whitespace,
        last: last,
        type_id: 0,
    }
}