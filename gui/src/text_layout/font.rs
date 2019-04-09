
use fnv::FnvHashMap;

use atom::{Atom};

use component::math::{UV};

/// 字体管理器 使用SDF(signed distance field 有向距离场)渲染字体， 支持预定义字体纹理及配置， 也支持动态计算字符的SDF
pub struct FontMgr {
    sdf_font: FnvHashMap<Atom, SdfFont>,
    font_family: FnvHashMap<Atom, Vec<Atom>>,
}
impl FontMgr {
    pub fn new() -> FontMgr {
        FontMgr {
            sdf_font: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            font_family: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
        }
    }
    // 测量指定字体下，指定字符的宽度。 0表示没有该字符
    pub fn measure(&self, font_family: &Atom, font_size: f32, c: char) -> f32 {
        match self.font_family.get(font_family) {
            Some(vec) => {
                for name in vec {
                    match self.sdf_font.get(name) {
                        Some(font) => {
                            let r = font.measure(name, c);
                            if r > 0.0 {
                                return r * font.size / font_size
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
    // 异步计算字符的sdf数据的函数, 返回None表示异步设置该字符，否则返回该字符的的sdf数据
    fn sdf(&self, font_name: &Atom, font: &mut SdfFont, c: char) -> Option<Result<Vec<u8>, usize>>;
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, font_name: &Atom, font_size: f32, c: char) -> f32;
}

// 空实现
struct Empty;
impl FontHelper for Empty {
    // 异步计算字符的sdf数据的函数, 返回None表示异步设置该字符，否则返回该字符的的sdf数据
    fn sdf(&self, _font_name: &Atom, _font: &mut SdfFont, _c: char) -> Option<Result<Vec<u8>, usize>> {
        Some(Err(0))
    }
    // 同步计算字符宽度的函数, 返回0表示不支持该字符，否则返回该字符的宽度
    fn measure(&self, _font_name: &Atom, _font_size: f32, _c: char) -> f32 {
        0.0
    }
}

pub struct SdfFont {
    pub size: f32, //SDF大小, 一般32像素
    pub baseline: bool, //字符是否有基线
    pub fixed_width: f32, //字符的固定宽度, 0为非定宽
    pub char_uv_map: FnvHashMap<char, UV>, // 每字符的UV，可以通过UV计算宽度， 如果字高为0，表示字符正在创建
    pub char_baseline_map: FnvHashMap<char, f32>, // 每字符的基线， 如果没有，则为0
    pub texture_size: cg::Vector2<usize>, // 字体纹理的大小
    pub texture_handle: usize, // 字体纹理的句柄
    pub use_area: UV, // 当前动态创建字符的可使用的区域
    pub cur_pos: usize, // 当前可写字符的位置
    pub helper: Box<dyn FontHelper>,
}
impl SdfFont {
    pub fn new() -> SdfFont {
        SdfFont {
            size: 32.0,
            baseline: false,
            fixed_width: 32.0,
            char_uv_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            char_baseline_map: FnvHashMap::with_capacity_and_hasher(0, Default::default()),
            texture_size: cg::Vector2::new(0, 0),
            texture_handle: 0,
            use_area: UV::default(),
            cur_pos: 0,
            helper: Box::new(Empty),
        }
    }
    // 测量指定字体下，指定字符的宽度。 0表示没有该字符
    pub fn measure(&self, name: &Atom, c: char) -> f32 {
        if self.fixed_width > 0.0 {
            return self.fixed_width
        }
        match self.char_uv_map.get(&c) {
            Some(uv) => (uv.1.x - uv.0.x) * (self.texture_size.x as f32),
            _ => self.helper.measure(name, self.size, c)
        }
    }
}

// 倾斜度造成的间距
pub fn oblique_spacing(oblique: f32, font_size: f32, char_width: f32) -> f32 {
    oblique * font_size * char_width // TODO FIX!!!
}
