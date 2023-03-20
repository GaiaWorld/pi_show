use hash::XHashMap;



// 文字自身的字形信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlyphInfo {
    pub ox: i16, //文字可见区域左上角相对于文字外边框的左上角在水平轴上的距离 百分比(实际百分比应该除以256，之所以这样，是为了压缩数据尺寸)
    pub oy: i16, //文字可见区域左上角相对于文字外边框的左上角在垂直轴上的距离 百分比(实际百分比应该除以256，之所以这样，是为了压缩数据尺寸)
    pub width: u8,
    pub height: u8, 
    pub advance: u8, // advancePx
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MetricsInfo {
	pub font_size: f32,             // 文字尺寸
	pub line_height: f32,           // 默认行高
	pub max_height: f32,	        // 所有字形，最大高度（用于在纹理中分配行高）
	pub ascender: f32,              // 升线 （单位： font_size的百分比）
	pub descender: f32,             // 降线 （单位： font_size的百分比）
	pub underline_y: f32,           // 下划线的位置 （暂未使用）
	pub underline_thickness: f32,   // 
	pub distance_range: f32,        // msdf才会用到（0~1范围内的sdf所跨过的像素数量）
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontCfg {
    pub name: String,
    pub metrics: MetricsInfo,
    pub glyphs: XHashMap<char, GlyphInfo>,
}

// 字符的sdf值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharSdf {
	pub unicode: u32,        // 字符的unicode编码
    pub buffer: Vec<u8>,  // 字符的sdf buffer
}