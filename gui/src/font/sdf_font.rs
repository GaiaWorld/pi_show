use std::mem::transmute;
use std::rc::Rc;

use fnv::FnvHashMap;

use data_view::GetView;

use render::res::TextureRes; 

use atom::Atom;
// use font::FontMeasure;

pub trait SdfFont {
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_size: f32, c: char) -> f32;

    fn name(&self) -> Atom;

    fn glyph_info(&self, c: char, font_size: f32) -> Option<GlyphInfo>;

    fn atlas_width(&self) -> usize;

    fn atlas_height(&self) -> usize;
    
    fn texture(&self) -> &Rc<TextureRes>;
}

pub struct GlyphInfo{
    pub width: f32,
    pub height: f32,
    pub ox: f32, //文字可见区域左上角相对于文字外边框的左上角在水平轴上的距离
    pub oy: f32, //文字可见区域左上角相对于文字外边框的左上角在垂直轴上的距离
    pub min_u: f32,
    pub min_v: f32,
    pub max_u: f32,
    pub max_v: f32,
}

#[derive(Debug)]
pub struct StaticSdfFont {
    name: Atom,
    line_height: f32,
    atlas_width: usize,
    atlas_height: usize,
    glyph_table: FnvHashMap<u32, Glyph>,
    texture: Rc<TextureRes>,
}

impl SdfFont for StaticSdfFont { 
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_size: f32, c: char) -> f32 {
        match self.glyph_table.get(&unsafe{ transmute(c) }) {
            Some(glyph) => font_size/self.line_height*glyph.advance,
            None => 0.0,
        }
    }

    #[inline]
    fn atlas_width(&self) -> usize {
        self.atlas_width
    }

    #[inline]
    fn atlas_height(&self) -> usize {
        self.atlas_height
    }

    #[inline]
    fn name(&self) -> Atom {
        self.name.clone()
    }

    fn glyph_info(&self, c: char, font_size: f32) -> Option<GlyphInfo> {
        let ratio = font_size/self.line_height;
        match self.glyph_table.get(&unsafe{ transmute(c) }) {
            Some(glyph) => {
                let (min_u, max_v) = (glyph.x + glyph.ox, glyph.y - (self.line_height - glyph.oy)); //左上角
                Some(
                    GlyphInfo {
                        height: ratio * glyph.height,
                        width: ratio * glyph.width,
                        ox: ratio * glyph.ox,
                        oy: ratio * (self.line_height - glyph.oy),
                        min_u: min_u/(self.atlas_width as f32),
                        max_v: max_v/(self.atlas_height as f32),
                        max_u: (min_u + glyph.width)/(self.atlas_width as f32),
                        min_v: (max_v - glyph.height)/(self.atlas_height as f32),
                    }
                )
            },
            None => None,
        }     
    }

    #[inline]
    fn texture(&self) -> &Rc<TextureRes> {
        &self.texture
    }
}

impl StaticSdfFont {
    pub fn new(texture: Rc<TextureRes>) -> StaticSdfFont{
        StaticSdfFont {
            name: Atom::from(""),
            line_height: 0.0,
            atlas_width: 0,
            atlas_height: 0,
            glyph_table: FnvHashMap::default(),
            texture: texture,
        }
    }
}

impl StaticSdfFont {
    pub fn parse(&mut self, value: &[u8]) -> Result<(), String>{
        let mut offset = 12;
        match String::from_utf8(Vec::from(&value[0..11])) {
            Ok(s) => if s != "GLYPL_TABLE".to_string() {
                return Err("parse error, it's not GLYPL_TABLE".to_string());
            },
            Err(s) => return Err(s.to_string()),
        };
        
        let name_len = value.get_u8(offset);
        offset += 1;
        let name_str = match String::from_utf8(Vec::from(&value[offset..offset + name_len as usize])) {
            Ok(s) => s,
            Err(s) => return Err(s.to_string()),
        };
        offset += name_len as usize;
        if offset%2 == 1 {
            offset += 1;
        }
        self.name = Atom::from(name_str);

        self.line_height = value.get_lf32(offset);
        offset += 4;

        self.atlas_width = value.get_lu16(offset) as usize;
        offset += 2;
        self.atlas_height = value.get_lu16(offset) as usize;
        offset += 2;

        //字符uv表
        loop {
            if offset >= value.len() {
                break;
            }
            let id = value.get_lu16(offset);
            offset += 2;
            let x = value.get_lu16(offset);
            offset += 2;
            let y = value.get_lu16(offset);
            offset += 2;
            let ox = value.get_u8(offset);
            offset += 1;
            let oy = value.get_u8(offset);
            offset += 1;
            let width = value.get_u8(offset);
            offset += 1;
            let height = value.get_u8(offset);
            offset += 1;
            let advance = value.get_u8(offset);
            offset += 2; // 加2， 对齐

            self.glyph_table.insert(
                id as u32,
                Glyph {
                    id: id as u32,
                    x: x as f32,
                    y: y as f32,
                    ox: ox as f32,
                    oy: oy as f32,
                    width: width as f32,
                    height: height as f32,
                    advance: advance as f32,
                }
            );
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Glyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub ox: f32, 
    pub oy: f32,
    pub width: f32, 
    pub height: f32,
    pub advance: f32,
}

impl Glyph {
    pub fn new(id: u32, x: f32, y: f32, ox: f32, oy: f32, width: f32, height: f32, advance: f32) -> Glyph {
        Glyph { id, x, y, ox, oy, width, height, advance }
    }
}