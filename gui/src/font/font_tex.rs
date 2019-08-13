use share::{Share};

use render::res::{TextureRes};
use fx_hashmap::FxHashMap32;

use component::user::{Point2};

pub struct FontTex{
    pub texture: Share<TextureRes>,
    line_map: FxHashMap32<usize, (Point2, usize)>,
    last_v: f32,
}

impl FontTex {
    pub fn new(texture: Share<TextureRes>) -> Self {
        // let tex = ctx.create_texture_2d(TEX_WIDTH as u32, INIT_TEX_HEIGHT, 0, &PixelFormat::RGBA, &DataFormat::UnsignedByte, false, &TextureData::None).unwrap();
        // texture: Share::new(TextureRes::new(Atom::from("FontTex"), TEX_WIDTH as usize, INIT_TEX_HEIGHT as usize, Opacity::Translucent, Compress::None, tex)),
        FontTex {
            texture: texture,
            line_map: FxHashMap32::default(),
            last_v: 0.0,
        }
    }
    // 分配行
    pub fn alloc_line(&mut self, mut line_height: usize) -> TexLine {
        // 将奇数的行高向上变成偶数，这样单行容纳2种字号，提高利用率
        if line_height %2 != 0 {
            line_height += 1;
        }
        let v = self.last_v;
        self.last_v += line_height as f32;
        TexLine {
            line: self.line_map.entry(line_height).or_insert((Point2::new(0.0, v), 0)),
            last_v: &mut self.last_v,
            tex_width: self.texture.width as f32,
            line_height: line_height as f32
        }
    }

    // fn update(&self, tex: Res<TextureRes>, u: f32, v: f32, w: f32, h: f32, data: &Object) {
    //     if v + h > self.last_v {
    //         // 纹理高度扩展1倍
    //     }
    //     self.tex.bind.update_webgl(tex, u, v, w, h, data);
    // }
}

#[derive(Debug)]
pub struct TexLine<'a> {
    line: &'a mut (Point2, usize),
    last_v: &'a mut f32,
    tex_width: f32,
    line_height: f32,
}
impl<'a> TexLine<'a> {
    // 获得起始v
    pub fn get_v(&self) -> f32 {
        self.line.0.y
    }
    // 分配字符的起始uv
    pub fn alloc(&mut self, char_width: f32) -> Point2 {
        if self.tex_width >= self.line.0.x + char_width {
            let r = self.line.0.clone();
            self.line.0.x += char_width;
            self.line.1 += 1;
            r
        }else{
            self.line.0.x = char_width;
            self.line.0.y = *self.last_v;
            self.line.1 = 1;
            *self.last_v += self.line_height;
            Point2::new(0.0, self.line.0.y)
        }
    }
}