
use fnv::FnvHashMap;

use atom::{Atom};

/// 字体管理器 使用SDF(signed distance field 有向距离场)渲染字体， 支持预定义字体纹理及配置， 也支持动态计算字符的SDF
pub struct FontMgr {
    sdf_font: FnvHashMap<Atom, SdfFont>,
    font_family: FnvHashMap<Atom, Vec<Atom>>,
}


// 异步计算字符的sdf数据的函数, 返回0表示不支持该字符，否则返回该字符的宽度。 参数only_measure表示仅计算宽度，不生成SDF数据
// Fn(font_name: &Atom, font: &mut SdfFont, c: char, only_measure: bool) -> usize;
pub type SdfCalc = Box<Fn(&Atom, &mut SdfFont, char, bool) -> usize>;

pub struct SdfFont {
    pub size: f32, //SDF大小, 一般32像素
    pub baseline: bool, //字符是否有基线
    pub fixed_width: f32, //字符的固定宽度, 0为非定宽
    pub char_uv_map: FnvHashMap<char, CharUV>, // 每字符的UV，可以通过UV计算宽度
    pub char_baseline_map: FnvHashMap<char, f32>, // 每字符的基线， 如果没有，则为0
    pub texture_size: cg::Vector2<usize>, // 字体纹理的大小
    pub texture_handle: usize, // 字体纹理的句柄
    pub use_area: CharUV, // 当前动态创建字符的可使用的区域
    pub cur_pos: usize, // 当前可写字符的位置
    pub sdf_calc: SdfCalc,
}

/// 字符UV， 如果字高为0，表示字符正在创建
pub struct CharUV (cg::Point2<f32>, cg::Point2<f32>);


/// 扩展字体
pub struct Font {
    pub size: f32, // 字体大小
    pub weight: f32, // 字体粗细
    pub oblique: f32, // 倾斜度
    // pub align: f32, // 倾斜度造成的间距，可以计算出来
    pub font_face: Atom, // 字体表现。 对应font_family
}
