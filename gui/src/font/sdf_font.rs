use std::mem::transmute;

use fnv::FnvHashMap;

use data_view::GetView;
use atom::Atom;
use share::Share;

use render::res::TextureRes;
// use font::FontMeasure;
// pub const FONT_FACTOR: f32 = 1.3198;

pub trait SdfFont {
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_size: f32, c: char) -> f32;

    fn name(&self) -> Atom;

    fn glyph_info(&self, c: char, font_size: f32) -> Option<GlyphInfo>;

    fn atlas_width(&self) -> usize;

    fn atlas_height(&self) -> usize;

    fn line_height(&self) -> f32;

    fn font_size(&self) -> f32;
    
    fn texture(&self) -> &Share<TextureRes>;

    fn distance_for_pixel(&self, font_size: f32) -> f32;

    fn get_glyph(&self, c: &char) -> Option<&Glyph>;

    fn add_glyph(&self, c: char, glyph: Glyph);

    fn get_dyn_type(&self) -> usize;

    fn curr_uv(&self) -> (f32, f32);

    fn set_curr_uv(&self, value: (f32, f32));

    fn stroke_width(&self) -> f32;

    fn weight(&self) -> f32;
}

// // 字体生成器
// pub trait MSdfGenerator{
//     fn gen(&self, font_name: &str, c: char) -> Glyph;

//     // fn gen_mult(&self, chars: &[char]) -> Vec<Glyph>;
// }

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

pub struct DefaultSdfFont {
    pub name: Atom,
    line_height: f32,
    atlas_width: usize,
    atlas_height: usize,
    padding: f32,
    pub glyph_table: FnvHashMap<char, Glyph>,
    texture: Share<TextureRes>,
    dyn_type: usize,
    curr_uv: (f32, f32),
    stroke_width: f32,
    font_size: f32,
    weight: f32,
}

impl SdfFont for DefaultSdfFont { 
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_size: f32, c: char) -> f32 {
        match self.glyph_table.get(&c) {
            Some(glyph) => {
                font_size/self.font_size*glyph.advance
            },
            None => {
                font_size
                // let glyph = self.generator.gen(self.name.as_ref(), c);
                // let advance = glyph.advance;
                // unsafe { &mut *(&self.glyph_table as *const FnvHashMap<char, Glyph> as usize as *mut FnvHashMap<char, Glyph>) }.insert(c, glyph);
                // font_size/self.line_height*advance
            },
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
        let ratio = font_size/self.font_size;
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

    fn get_glyph(&self, c: &char) -> Option<&Glyph>{
        self.glyph_table.get(c) 
    }

    fn add_glyph(&self, c: char, glyph: Glyph){
        let s = unsafe { &mut  *(self as *const Self as usize as *mut Self) };
        s.glyph_table.insert(c, glyph);
    }

    fn distance_for_pixel(&self, font_size: f32) -> f32{
        let ratio = font_size/self.line_height;
        0.5/(ratio * self.padding)
    }

    fn get_dyn_type(&self) -> usize {
        self.dyn_type
    }

    fn line_height(&self) -> f32 {
        self.line_height
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn curr_uv(&self) -> (f32, f32) {
        (self.curr_uv.0, self.curr_uv.1)
    }

    fn set_curr_uv(&self, value: (f32, f32)) {
        unsafe {&mut *(self as *const Self as usize as *mut Self)}.curr_uv = value
    }

    fn stroke_width(&self) -> f32 {
        self.stroke_width
    }

    fn weight(&self) -> f32 {
        self.weight
    }


    #[inline]
    fn texture(&self) -> &Share<TextureRes> {
        &self.texture
    }
}

impl DefaultSdfFont {
    pub fn new(texture: Share<TextureRes>, dyn_type: usize) -> Self{
        DefaultSdfFont {
            name: Atom::from(""),
            line_height: 0.0,
            atlas_width: 0,
            atlas_height: 0,
            padding: 0.0,
            glyph_table: FnvHashMap::default(),
            texture: texture,
            dyn_type: dyn_type,
            curr_uv: (0.0, 0.0),
            stroke_width: 0.0,
            font_size: 0.0,
            weight: 0.0,
        }
    }

    pub fn new_width_data(
        name: Atom,
        line_height: f32,
        atlas_width: usize,
        atlas_height: usize,
        padding: f32,
        glyph_table: FnvHashMap<char, Glyph>,
        texture: Share<TextureRes>,
    ) -> Self{
        DefaultSdfFont {
            name,
            line_height,
            atlas_width,
            atlas_height,
            padding,
            glyph_table,
            texture,
            dyn_type: 0,
            curr_uv: (0.0, 0.0),
            stroke_width: 0.0,
            font_size: 0.0,
            weight: 0.0
        }
    }
}

impl DefaultSdfFont {
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
        self.font_size = value.get_u8(offset) as f32;
        self.stroke_width = value.get_u8(offset + 2) as f32; // stroke_width使用padding位置
        // self.weight = value.get_u8(offset + 4) as f32; // stroke_width使用padding位置
        // value.get_lu16(offset) as f32;
        // value.get_lu16(offset) as f32;
        // value.get_lu16(offset) as f32;
        // value.get_lu16(offset) as f32;
        offset += 8;

        if self.font_size == 0.0 {
            self.font_size = self.line_height;
        }

        //字符uv表
        loop {

            if offset >= value.len() {
                break;
            }
            // Glyph::parse(value, &mut offset)
            // let id = value.get_lu16(offset);
            // offset += 2;
            // let x = value.get_lu16(offset);
            // offset += 2;
            // let y = value.get_lu16(offset);
            // offset += 2;
            // let ox = value.get_li8(offset);
            // offset += 1;
            // let oy = value.get_u8(offset);
            // offset += 1;
            // let width = value.get_u8(offset);
            // offset += 1;
            // let height = value.get_u8(offset);
            // offset += 1;
            // let advance = value.get_u8(offset);
            // // offset += 1;
            // offset += 2; // 加2， 对齐
            let glyph = Glyph::parse(value, &mut offset);
            self.glyph_table.insert(
                glyph.id,
                glyph
            );
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
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

impl Glyph {
    pub fn parse(value: &[u8], offset: &mut usize) -> Self {
        let id = value.get_lu16(*offset);
        *offset += 2;
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
            id: unsafe{ transmute(id as u32) },
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