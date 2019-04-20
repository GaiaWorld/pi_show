use std::mem::transmute;
use std::rc::Rc;

use fnv::FnvHashMap;
use cg::{Vector4};
use render::res::TextureRes; 
use data_view::GetView;

use atom::Atom;

#[derive(Debug)]
pub struct SdfFont {
    pub name: Atom,
    pub line_height: f32,
    pub index: usize, // 纹理当前索引
    pub cur_pos: usize, // 当前可写字符的位置
    pub atlas_width: usize,
    pub atlas_height: usize,
    pub glyph_table: FnvHashMap<char, Glyph>,
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
        self.glyph_table.get(&c)
    }

    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    pub fn measure(&self, font_size: f32, c: char) -> Option<f32> {
        match self.glyph_table.get(&c) {
            Some(glyph) => Some(font_size/self.line_height*glyph.advance),
            None => None,
        }
    }

    // 获得纹理uv
    pub fn get_texture_uv(&self, c: char) -> Option<Vector4<f32>>{
        match self.glyph_table.get(&c ) {
            Some(glyph) => Some(Vector4::new(glyph.x, glyph.y, glyph.width, glyph.height)),
            None => None,
        }
    }
}

impl SdfFont {
    pub fn parse(&mut self, value: &[u8]) -> Result<(), String>{
        println!("parse----------------------------{:?}", value);
        let mut offset = 12;
        match String::from_utf8(Vec::from(&value[0..11])) {
            Ok(s) => if s != "GLYPL_TABLE".to_string() {
                return Err("parse error, it's not GLYPL_TABLE".to_string());
            },
            Err(s) => return Err(s.to_string()),
        };
        
        println!("parse----------------------------{}", offset);
        let name_len = value.get_u8(offset);
        println!("name_len----------------------------{}", name_len);
        offset += 1;
        let name_str = match String::from_utf8(Vec::from(&value[offset..offset + name_len as usize])) {
            Ok(s) => s,
            Err(s) => return Err(s.to_string()),
        };
        offset += name_len as usize;
        if offset%2 == 1 {
            offset += 1;
        }
        println!("name_str--------------------{}", name_str);
        self.name = Atom::from(name_str);
        println!("parse2----------------------------{}", offset);

        self.line_height = value.get_lf32(offset);
        offset += 4;

        self.atlas_width = value.get_lu16(offset) as usize;
        offset += 2;
        self.atlas_height = value.get_lu16(offset) as usize;
        offset += 2;

        println!("parse3----------------------------{:?}, {:?}", offset, self);
        //字符uv表
        loop {
            if offset >= value.len() {
                println!("offset----------------------------{:?}", offset);
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
                unsafe{transmute(id as u32)},
                Glyph {
                    id: unsafe{transmute(id as u32)},
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
        println!("len----------------------------{:?}", self.glyph_table.len());
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


