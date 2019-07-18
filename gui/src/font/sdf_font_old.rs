use std::mem::transmute;
use share::Share;



use hal_core::{Context};
use data_view::GetView;
use atom::Atom;

use render::res::TextureRes;
// use font::FontMeasure;

pub trait SdfFont {
    type Ctx: Context;
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_size: f32, c: char) -> f32;

    fn name(&self) -> Atom;

    fn glyph_info(&self, c: char, font_size: f32) -> Option<GlyphInfo>;

    fn atlas_width(&self) -> usize;

    fn atlas_height(&self) -> usize;
    
    fn texture(&self) -> &Share<TextureRes<Self::Ctx>>;

    fn distance_for_pixel(&self, font_size: f32) -> f32;
}

pub struct GlyphInfo{
    pub width: f32,
    pub height: f32,
    pub ox: f32, //文字可见区域左上角相对于文字外边框的左上角在水平轴上的距离
    pub oy: f32, //文字可见区域左上角相对于文字外边框的左上角在垂直轴上的距离
    pub u_min: f32,
    pub u_max: f32,
    pub v_min: f32,
    pub v_max: f32,
    pub adv: f32,
}

pub struct StaticSdfFont<C: Context + 'static> {
    pub name: Atom,
    line_height: f32,
    atlas_width: usize,
    atlas_height: usize,
    // base: f32,
    padding: f32,
    pub glyph_table: FxHashMap32<char, Glyph>,
    texture: Share<TextureRes<C>>,
}

impl<C: Context + 'static> SdfFont for StaticSdfFont<C> { 
    type Ctx = C;
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_size: f32, c: char) -> f32 {
        match self.glyph_table.get(&c) {
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
        match self.glyph_table.get(&c) {
            Some(glyph) => {
                let (min_u, min_v) = (glyph.x, glyph.y); //左上角
                Some(
                    GlyphInfo {
                        height: ratio * glyph.height,
                        width: ratio * glyph.width,
                        ox: ratio * glyph.ox,
                        oy: ratio * glyph.oy,
                        u_min: min_u/(self.atlas_width as f32),
                        u_max: (min_u + glyph.width)/(self.atlas_width as f32),
                        v_min: min_v/(self.atlas_height as f32),
                        v_max: (min_v + glyph.height)/(self.atlas_height as f32),
                        adv: ratio * glyph.advance,
                    }
                )
            },
            None => None,
        }     
    }

    fn distance_for_pixel(&self, font_size: f32) -> f32{
        let ratio = font_size/self.line_height;
        0.5/(ratio * self.padding)
    }

    #[inline]
    fn texture(&self) -> &Share<TextureRes<C>> {
        &self.texture
    }
}

impl<C: Context + 'static> StaticSdfFont<C> {
    pub fn new(texture: Share<TextureRes<C>>) -> Self{
        StaticSdfFont {
            name: Atom::from(""),
            line_height: 0.0,
            atlas_width: 0,
            atlas_height: 0,
            padding: 0.0,
            glyph_table: FxHashMap32::default(),
            texture: texture,
        }
    }

    pub fn new_width_data(
        name: Atom,
        line_height: f32,
        atlas_width: usize,
        atlas_height: usize,
        padding: f32,
        glyph_table: FxHashMap32<char, Glyph>,
        texture: Share<TextureRes<C>>,
    ) -> Self{
        StaticSdfFont {
            name,
            line_height,
            atlas_width,
            atlas_height,
            padding,
            glyph_table,
            texture,
        }
    }
}

impl<C: Context + 'static> StaticSdfFont<C> {
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
        // if offset%2 == 1 {
        //     offset += 1;
        // }
        self.name = Atom::from(name_str);

        self.line_height = value.get_u8(offset) as f32;
        offset += 1;

        self.atlas_width = value.get_lu16(offset) as usize;
        offset += 2;
        self.atlas_height = value.get_lu16(offset) as usize;
        offset += 2;
        // padding
        value.get_lu16(offset) as f32;
        value.get_lu16(offset) as f32;
        value.get_lu16(offset) as f32;
        value.get_lu16(offset) as f32;
        offset += 8;

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
            let ox = value.get_li8(offset);
            offset += 1;
            let oy = value.get_u8(offset);
            offset += 1;
            let width = value.get_u8(offset);
            offset += 1;
            let height = value.get_u8(offset);
            offset += 1;
            let advance = value.get_u8(offset);
            // offset += 1;
            offset += 2; // 加2， 对齐

            self.glyph_table.insert(
                unsafe{ transmute(id as u32) },
                Glyph {
                    id: unsafe{ transmute(id as u32) },
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
    pub id: char,
    pub x: f32,
    pub y: f32,
    pub ox: f32, 
    pub oy: f32,
    pub width: f32, 
    pub height: f32,
    pub advance: f32,
}