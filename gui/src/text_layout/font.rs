use std::mem::transmute;
use std::rc::Rc;

use fnv::FnvHashMap;
use cg::{Vector4, Vector2};
use render::res::TextureRes; 

use atom::Atom;

pub struct SdfFont {
    pub name: Atom,
    pub line_height: f32,
    pub index: usize, // 纹理当前索引
    pub cur_pos: usize, // 当前可写字符的位置
    pub atlas_width: usize,
    pub atlas_height: usize,
    pub glyph_table: FnvHashMap<u32, Glyph>,
    pub texture: Rc<TextureRes>,
}

impl SdfFont {
    pub fn new(texture: Rc<TextureRes>) -> SdfFont{
        SdfFont {
            name: Atom::from(""),
            line_height: 0.0,
            index: 0,
            cur_pos: 0,
            atlas_width: 0,
            atlas_height: 0,
            glyph_table: FnvHashMap::default(),
            texture: texture,
        }
    }

    pub fn get_glyph(&self, c: char) -> Option<&Glyph>{
        self.glyph_table.get(&unsafe{ transmute(c) })
    }

    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    pub fn measure(&self, font_size: f32, c: char) -> Option<f32> {
        match self.glyph_table.get(&unsafe{ transmute(c) }) {
            Some(glyph) => Some(font_size/self.line_height*glyph.advance),
            None => None,
        }
    }

    // 获得纹理uv
    pub fn get_texture_uv(&self, c: char) -> Option<Vector4<f32>>{
        match self.glyph_table.get(&unsafe{ transmute(c) }) {
            Some(glyph) => Some(Vector4::new(glyph.x, glyph.y, glyph.width, glyph.height)),
            None => None,
        }
    }
}

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


