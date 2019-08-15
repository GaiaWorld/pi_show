
use std::{
    default::Default,
    str::Chars,
    mem::replace,
    collections::hash_map::Entry,
};
use atom::{Atom};
use slab::Slab;
use data_view::GetView;
use ucd::{Codepoint};
use share::Share;

use render::res::{TextureRes};

use component::user::*;
use font::font_tex::*;
use fx_hashmap::FxHashMap32;

pub const FONT_SIZE: f32 = 32.0;

/// 默认纹理宽度为2048，永远向下扩展
pub const TEX_WIDTH: f32 = 2048.0;
pub const INIT_TEX_HEIGHT: u32 = 256;

// 小字体的大小， 小于该字体，默认勾1个px的边
const SMALL_FONT: usize = 20;

// 默认sdf字体的大小， 用于作为基准
const SDF_FONT_SIZE: f32 = 32.0;

// TODO 将字体样式和字符做为键，查hashmap，获得slab id，slab中放字符和字形信息， 其他地方仅使用id
// TODO pixel font 和 xysdf font, 都使用一个文字纹理。 也是预处理的纹理。 
// 相同的font_farmly在不同的字符上也可能使用不同的font， text_layout需要根据pixel和sdf分成char_blocks 的2个arr
// 测量字符宽度时， 计算出Glyth，并创建font_char_id. 将未绘制的放入全局wait_draw_list, 统一绘制

/// 字体表 使用SDF(signed distance field 有向距离场)渲染字体， 支持预定义字体纹理及配置， 也支持动态计算字符的SDF
pub struct FontSheet {
    size: f32,
    color: CgColor,
    pub src_map: FxHashMap32<Atom, TexFont>,
    face_map: FxHashMap32<Atom, FontFace>,
    char_w_map: FxHashMap32<(Atom, char), (f32,/* char width */ Atom, /* font */ f32,/* factor */ bool), >,
    pub char_map: FxHashMap32<(Atom, usize, /* font_size */ usize, /* stroke_width */ usize, /* weight */ char, ), usize, /* slab id */>, // key (font, stroke_width, char) // 永不回收
    pub char_slab: Slab<(char, Glyph)>, // 永不回收 (char, Glyph, font_size, stroke_width) // 永不回收
    pub wait_draw: TextInfo,
    pub wait_draw_list: Vec<TextInfo>,
    measure_char: Box<dyn Fn(&Atom, usize, char)-> f32>,
    font_tex: FontTex,
}

impl  FontSheet {
    pub fn new(texture: Share<TextureRes>, measure: Box<dyn Fn(&Atom, usize, char)-> f32>) -> Self {
        FontSheet {
            size: FONT_SIZE,
            color: CgColor::default(),
            src_map: FxHashMap32::default(),
            face_map: FxHashMap32::default(),
            char_w_map: FxHashMap32::default(),
            char_map: FxHashMap32::default(),
            char_slab: Slab::default(),
            wait_draw: TextInfo {
                font: Atom::from(""),
                font_size: 0.0,
                stroke_width: 0,
                weight: 500,
                size: Vector2::default(),
                chars: Vec::new(),
            },
            wait_draw_list: Vec::new(),
            measure_char: measure,
            font_tex: FontTex::new(texture),
        }
    }
    // 设置默认字号
    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }
    // 设置默认字色
    pub fn set_color(&mut self, color: CgColor) {
        self.color = color;
    }
    // 设置Font
    pub fn set_src(&mut self, name: Atom, is_pixel: bool, factor: f32) {
        self.src_map.insert(name.clone(), TexFont { name, is_pixel, factor});
    }

    pub fn get_src(&mut self, name: &Atom) -> Option<&TexFont> {
        self.src_map.get(name)
    }

    // 取字体详情
    pub fn get_font_info(&self, font_face: &Atom) -> Option<(&TexFont, usize/* font_size */)> {
        match self.face_map.get(font_face) {
            Some(face) => {;
                for name in &face.src {
                    match self.src_map.get(name) {
                        Some(font) => return Some((font, face.size)),
                        _ => (),
                    }
                }
            },
            _ => ()
        };
        None
    }
    
    // 设置FontFace
    pub fn set_face(&mut self, family: Atom, oblique: f32, size: usize, weight: usize, src: String) {
        let mut v = Vec::new();
        for s in src.split(',') {
            v.push(Atom::from(s.trim_start().trim_end()))
        }
        let face = FontFace {
            oblique: oblique,
            size: size,
            weight: weight,
            src: v.clone(),
        };
        self.face_map.insert(family.clone(), face);
    }

    // 取字体信息
    pub fn get_font(&self, font_face: &Atom) -> Option<&TexFont> {
        match self.face_map.get(font_face){
            Some(face) => {
                for name in &face.src {
                    if let Some(font) = self.src_map.get(name) {
                        return Some(font);
                    }
                }
                None
            },
            None => None
        }
    }
    // TODO 改成返回一个查询器， 这样在多个字符查询时，少很多hash查找
    // 测量指定字符的宽高，返回字符宽高(不考虑scale因素)，字符的slab_id，是否为pixel字体。 不同字符可能使用不同字体。 测量时，计算出Glyth, 并创建font_char_id, 将未绘制的放入wait_draw_list。
    pub fn measure(&mut self, font: &TexFont, font_size: usize, sw: usize, c: char) -> (Vector2/*width,height*/, f32/*base_width*/) {
        match self.char_w_map.entry((font.name.clone(), c)) {
            Entry::Occupied(e) => {
                let r = e.get();
                return (Vector2::new(r.0 * font_size as f32 / FONT_SIZE + sw as f32, r.2 * font_size as f32 + sw as f32), r.0)
            },
            Entry::Vacant(r) => {
                let w = self.measure_char.as_ref()(&font.name, FONT_SIZE as usize, c);
                if w > 0.0 {
                    r.insert((w, font.name.clone(), font.factor, font.is_pixel));
                    return (Vector2::new(w * font_size as f32 / FONT_SIZE + sw as f32, font.factor * font_size as f32 + sw as f32), w)
                }
            }
        }
        (Vector2::new(0.0, 0.0), 0.0)
    }

    // 添加一个字形信息, 
    pub fn calc_gylph(&mut self, font: &TexFont, font_size: usize, stroke_width: usize, weight: usize, scale: f32, base_width: usize, c: char) -> usize {
        if font.is_pixel {// 像素纹理
            //let fs = font_size as f32 * font.factor;
            let fs_scale = (font_size as f32 * scale).round() as usize;
            // 为了泛用，渲染的字符总是会有边框， 要么是默认的，要么是参数指定的
            let sw = if stroke_width != 0 {
                let r = (stroke_width as f32 * scale).round() as usize; // 勾边也要用缩放后
                if r == 0 {1}else{r} // 保证最少1个像素
            // }else if fs_scale < SMALL_FONT {
            //     1
            // }else{
            //     2
            } else {
                0
            };
            // 根据缩放后的字体及勾边大小来查找Glyth, 返回的w需要除以scale
            let id = match self.char_map.entry((font.name.clone(), fs_scale, sw, weight, c)) {
                Entry::Occupied(e) => *e.get(),
                Entry::Vacant(r) => {

                    // 在指定字体及字号下，查找该字符的宽度
                    let mut w = (base_width as f32 * (fs_scale as f32)/FONT_SIZE).ceil(); 
                    w += sw as f32;
                    // 将缩放后的实际字号乘字体的修正系数，得到实际能容纳下的行高
                    let height = (fs_scale as f32 * font.factor).round() as usize + sw;
                    let mut line = self.font_tex.alloc_line(height);
                    let p = line.alloc(w);
                    let id = self.char_slab.insert((c, Glyph{
                        x: p.x,
                        y: p.y,
                        ox: 0.0, 
                        oy: 0.0,
                        width: w, 
                        height: height as f32,
                        advance: w,
                    }));
                    // 将需要渲染的字符放入等待队列
                    if self.wait_draw.font != font.name || self.wait_draw.font_size != fs_scale as f32 || self.wait_draw.stroke_width != sw {
                        if self.wait_draw.chars.len() > 0 {
                            let info = replace(&mut self.wait_draw, TextInfo{
                                font: font.name.clone(),
                                font_size: fs_scale as f32 ,
                                stroke_width: sw,
                                weight: weight,
                                size: Vector2::new(w, height as f32),
                                chars: vec![WaitChar {ch: c, width: w, x: p.x as u32, y: p.y as u32}],
                            });
                            self.wait_draw_list.push(info);
                        }else{
                            self.wait_draw.font = font.name.clone();
                            self.wait_draw.font_size = fs_scale as f32 ;
                            self.wait_draw.stroke_width = sw;
                            self.wait_draw.size = Vector2::new(w, height as f32);
                            self.wait_draw.chars = vec![WaitChar {ch: c, width: w, x: p.x as u32, y: p.y as u32}];
                        }
                    }else{
                        self.wait_draw.size.x += w;
                        self.wait_draw.chars.push(WaitChar {ch: c, width: w, x: p.x as u32, y: p.y as u32});
                    }
                    r.insert(id);
                    id
                }
            };
            return id;
        }else{// SDF 字体， 根据字形Glyph计算宽度
            match self.char_map.get(&(font.name.clone(), 0, 0, 0, c)) {
                Some(id) => return *id,
                _ => ()
            }
        }
        0
    }

    pub fn get_font_tex(&self) -> &Share<TextureRes> {
        &self.font_tex.texture
    }

    pub fn get_glyph(&self, id: usize) -> Option<&(char, Glyph)>{
        self.char_slab.get(id)
    }

    // msdf 需要修正字形信息
    pub fn fix_gylph(gylph: &Glyph, font_size: f32) -> (f32, f32){
        let radio = font_size / FONT_SIZE;
        (gylph.width * radio, gylph.width * radio)
    }
}

// 字体表现
#[derive(Default, Debug)]
pub struct FontFace {
    oblique: f32,
    size: usize,
    weight: usize,
    src: Vec<Atom>,
}

pub fn get_size(size:usize, s:&FontSize) -> usize {
    match s {
        &FontSize::None => size,
        &FontSize::Length(r) => r.round() as usize,
        &FontSize::Percent(r) => (r * size as f32).round() as usize
    }
}
// 行高
pub fn get_line_height(size: usize, line_height: &LineHeight) -> f32 {
    match line_height {
        LineHeight::Length(r) => *r, //固定像素
        LineHeight::Number(r) => *r + size as f32, //设置数字，此数字会与当前的字体尺寸相加来设置行间距。
        LineHeight::Percent(r) => *r * size as f32,   //	基于当前字体尺寸的百分比行间距.
        LineHeight::Normal => size as f32,
    }
}
// // 倾斜度造成的间距
// pub fn oblique_spacing(oblique: f32, font_size: f32, char_width: f32) -> f32 {
//     oblique * font_size * char_width // TODO FIX!!!
// }

#[derive(Clone)]
pub struct TexFont {
    pub name: Atom,
    pub is_pixel: bool, // 是否为像素纹理， 否则为sdf纹理
    pub factor: f32, // 像素纹理字体大小有时超出，需要一个字体的修正系数
}

impl TexFont {
    #[inline]
    pub fn get_line_height(&self, font_size: usize, line_height: &LineHeight) -> f32 {
        (get_line_height(font_size, line_height) * 100.0 ).round() / 100.0
    }
    
    #[inline]
    //  获得字体大小, 0表示没找到该font_face
    pub fn get_font_height(&self, size: usize, stroke_width: f32) -> f32 {
        size as f32 * self.factor + stroke_width
    }
}

#[derive(Debug, Default, Clone)]
pub struct Glyph {
    pub x: f32,
    pub y: f32,
    pub ox: f32, //文字可见区域左上角相对于文字外边框的左上角在水平轴上的距离
    pub oy: f32, //文字可见区域左上角相对于文字外边框的左上角在垂直轴上的距离
    pub width: f32,
    pub height: f32,
    pub advance: f32,
}

impl Glyph {

    pub fn get_uv(&self, tex_size: &Vector2) -> Aabb2 {
        Aabb2::new(Point2::new(self.x / tex_size.x, self.y/ tex_size.y), Point2::new((self.x + self.width) / tex_size.x, (self.y + self.height) / tex_size.y))
    }

    pub fn parse(value: &[u8], offset: &mut usize) -> Self {
        let x = value.get_lu16(*offset);
        *offset += 2;
        let y = value.get_lu16(*offset);
        *offset += 2;
        let ox = value.get_li8(*offset);
        *offset += 1;
        let oy = value.get_u8(*offset);
        *offset += 1;
        let width = value.get_u8(*offset);
        *offset += 1;
        let height = value.get_u8(*offset);
        *offset += 1;
        let advance = value.get_u8(*offset);
        *offset += 2; // 加2， 对齐

        Glyph {
            x: x as f32,
            y: y as f32,
            ox: ox as f32,
            oy: oy as f32,
            width: width as f32,
            height: height as f32,
            advance: advance as f32,
        }
    }
}

pub struct TextInfo {
    pub font: Atom,
    pub font_size: f32,
    pub stroke_width: usize,
    pub weight: usize,
    pub size: Vector2,
    pub chars: Vec<WaitChar>,
}

pub struct WaitChar {
    ch: char,
    width: f32,
    x: u32,
    y: u32,
}

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