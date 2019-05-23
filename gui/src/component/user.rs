/// 用户操作的组件


use std::{
  sync::Arc,
  mem::transmute,
};

use map::{vecmap::VecMap};
use hal_core::Context;

use render::res::TextureRes;
use ecs::component::Component;
use atom::Atom;

//================================== 组件
#[derive(Clone, Debug, Default, Component)]
pub struct Layout{
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub border: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
}
#[derive(Deref, DerefMut, Component, Default)]
pub struct ZIndex(pub isize);

//超出部分的裁剪方式
#[derive(Deref, DerefMut, Component, Default)]
pub struct Overflow(pub bool);
//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(pub f32);

#[derive(Deref, DerefMut, Component, Clone, Debug, PartialEq)]
pub struct Show(pub usize);

#[derive(Debug, Clone, Component, Default)]
pub struct Transform {
    pub funcs: Vec<TransformFunc>,
    pub origin: TransformOrigin,
}

#[derive(Debug, Clone, Component, Default)]
pub struct BoxColor{
	pub background: Color,
	pub border: CgColor,
}

//ObjectFit
#[derive(Deref, DerefMut, Component, Default)]
pub struct ObjectFit(pub FitType);

#[derive(Component, Default)]
pub struct SrcClip{
  pub uv: (Point2, Point2),
  pub size: Vector2,
}

#[derive(Clone, Component)]
pub struct BackgroundImage<C: Context + 'static + Send + Sync>(pub Arc<TextureRes<C>>);
#[derive(Clone, Component)]
pub struct Image<C: Context + 'static + Send + Sync>{
  pub src: Arc<TextureRes<C>>,
}

#[derive(Clone, Component)]
pub struct BorderImage<C: Context + 'static + Send + Sync>(pub Arc<TextureRes<C>>);

#[derive(Debug, Clone, Component)]
pub struct BorderRadius(pub LengthUnit);

#[derive(Debug, Clone, Default, Component)]
pub struct BoxShadow{
    pub h: f32,
    pub v: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: CgColor,
}

#[derive(Debug, Clone, Component, Default)]
pub struct TextStyle{
    pub letter_spacing: f32, //字符间距， 单位：像素
    pub word_spacing: f32, //字符间距， 单位：像素
    pub line_height: LineHeight, //设置行高
    pub indent: f32, // 缩进， 单位： 像素
    pub white_space: WhiteSpace, //空白处理
    pub color: Color, //颜色
    pub stroke: Stroke,
    pub vertical_align: VerticalAlign,
}

#[derive(Debug, Clone, Component, Default)]
pub struct Text(pub Arc<String>);

#[derive(Debug, Clone, Component, Default)]
pub struct TextShadow{
    pub h: f32, //	必需。水平阴影的位置。允许负值。	测试
    pub v: f32, //	必需。垂直阴影的位置。允许负值。	测试
    pub blur: f32, //	可选。模糊的距离。	测试
    pub color: CgColor, //	可选。阴影的颜色。参阅 CSS 颜色值。
}
#[derive(Component, Debug, Clone, Default)]
pub struct Font{
    pub style: FontStyle, //	规定字体样式。参阅：font-style 中可能的值。
    pub weight: f32, //	规定字体粗细。参阅：font-weight 中可能的值。
    pub size: FontSize, //
    pub family: Atom, //	规定字体系列。参阅：font-family 中可能的值。
}


//================================== 枚举
pub type TypedArray = Vec<f32>;

pub type Matrix4 = cgmath::Matrix4<f32>;
pub type Point2 = cgmath::Point2<f32>;
pub type Point3 = cgmath::Point3<f32>;
pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type CgColor = color::Color<f32>;
pub type Aabb3 = collision::Aabb3<f32>;
pub struct Rect(Point2, Point2, Point2, Point2);

pub enum LengthUnitType{
    Pixel,
    Percent
}
#[derive(Clone, Copy, Debug)]
pub enum LengthUnit {
	Pixel(f32),
	Percent(f32),
}

#[derive(Clone, Copy, Debug)]
pub enum Display{
  Flex,
  None,
}

#[derive(Debug, Clone, EnumDefault)]
pub enum Color{
    RGB(CgColor),
    RGBA(CgColor),
    LinearGradient(LinearGradientColor),
    RadialGradient(RadialGradientColor),
}

impl Color {
    pub fn is_opaque(&self) -> bool {
        match self {
            Color::RGB(c) | Color::RGBA(c) => {
                return c.a >= 1.0
            },
            Color::LinearGradient(l) => {
                for c in l.list.iter() {
                    if c.rgba.a < 1.0 {
                    return false;
                    }
                }
                return true;
            },
            Color::RadialGradient(g) => {
                for c in g.list.iter() {
                    if c.rgba.a < 1.0 {
                        return false
                    }
                }
                return true;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearGradientColor{
    pub direction: f32,
    pub list: Vec<ColorAndPosition>,
}

#[derive(Debug, Clone)]
pub struct RadialGradientColor{
    pub center: (f32, f32),
    pub shape: RadialGradientShape,
    pub size: RadialGradientSize,
    pub list: Vec<ColorAndPosition>,
}

#[derive(Debug, Clone)]
pub struct ColorAndPosition{
    pub rgba: CgColor,
    pub position: f32,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum RadialGradientSize{
    ClosestSide,
    FarthesSide,
    ClosestCorner,
    Farthescorner,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum RadialGradientShape{
    Ellipse,
    Circle,
}
pub type Polygon = TypedArray;

// color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], direction: 0-360度
pub fn to_linear_gradient_color(color_and_positions: TypedArray, direction: f32) -> LinearGradientColor {
    let arr = color_and_positions;
    let len = arr.len();
    let count = len / 5;
    let mut list = Vec::with_capacity(count);
    for i in 0..count{
        let start = i * 5;
        let color_pos = ColorAndPosition {
            rgba: CgColor::new(arr[start], arr[start + 1], arr[start + 2], arr[start + 3]),
            position: arr[start + 4]
        };
        list.push(color_pos);
    }
    LinearGradientColor {
        direction: direction,
        list : list,
    }
}

// color_and_positions: [r, g, b, a, pos,   r, g, b, a, pos], center_x: 0~1, center_y: 0~1, shape: RadialGradientShape, size: RadialGradientSize
pub fn to_radial_gradient_color(color_and_positions: TypedArray, center_x: f32, center_y: f32, shape: u8, size: u8) -> RadialGradientColor {
    let arr = color_and_positions;
    let len = arr.len();
    let count = len / 5;
    let mut list = Vec::with_capacity(count);
    for i in 0..count{
        let start = i * 5;
        let color_pos = ColorAndPosition {
            rgba:CgColor::new(arr[start], arr[start + 1], arr[start + 2], arr[start + 3]),
            position: arr[start + 4]
        };
        list.push(color_pos);
    }
    RadialGradientColor {
        center: (center_x, center_y),
        shape : unsafe{ transmute(shape) },
        size: unsafe{ transmute(size) },
        list: list,
    }
}

#[derive(Default, Debug, Clone)]
pub struct Stroke{
    pub width: f32, //	描边宽度
    pub color: CgColor, //	描边颜色
}

// 图像填充的方式
#[derive(Debug, Clone, EnumDefault)]
pub enum FitType {
  None,
  Fill,
  Contain,
  Cover,
  ScaleDown,
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontSize {
    None,	// 默认尺寸。
    Length(f32),	//把 font-size 设置为一个固定的值。
    Percent(f32), //把 font-size 设置为基于父元素的一个百分比值。
}

//设置行高
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum LineHeight{
    Normal, //设置合理的行间距（等于font-size）
    Length(f32), //固定像素
    Number(f32), //设置数字，此数字会与当前的字体尺寸相乘来设置行间距。
    Percent(f32),   //	基于当前字体尺寸的百分比行间距.
}



#[derive(Debug, Clone)]
pub enum TransformFunc {
    TranslateX(f32),
    TranslateY(f32),
    Translate(f32, f32),

    //平移， 单位： %
    TranslateXPercent(f32),
    TranslateYPercent(f32),
    TranslatePercent(f32, f32),

    ScaleX(f32),
    ScaleY(f32),
    Scale(f32, f32),

    RotateZ(f32),
}

#[derive(Debug, Clone, EnumDefault)]
pub enum TransformOrigin{
    Center,
    XY(LengthUnit, LengthUnit),
}

impl TransformOrigin {
    pub fn to_value(&self, width: f32, height: f32) -> Point2 {
        match self {
            TransformOrigin::Center => Point2::new(0.5 * width, 0.5 * height),
            TransformOrigin::XY(x, y) => {
                Point2::new(
                    match x {
                        LengthUnit::Pixel(v) => v.clone(),
                        LengthUnit::Percent(v) => v * width,
                    },
                    match y {
                        LengthUnit::Pixel(v) => v.clone(),
                        LengthUnit::Percent(v) => v * height,
                    }
                )
            },
        }
    }
}

enum ShowType{
  Display = 1, // 0表示 Flex
  Visibility = 2, // 0表示no Visible
  Enable = 4, // 0表示no Enable
}
impl Transform {
    pub fn matrix(&self, width: f32, height: f32, origin: &Point2) -> Matrix4 {
        // M = T * R * S
        // let mut m = cg::Matrix4::new(
        //     1.0, 0.0, 0.0, 0.0,
        //     0.0, 1.0, 0.0, 0.0,
        //     0.0, 0.0, 1.0, 0.0,
        //     0.0, 0.0, 0.0, 1.0,
        // );

        let value = self.origin.to_value(width, height);
        let mut m = Matrix4::from_translation(Vector3::new(origin.x + value.x, origin.y + value.y, 0.0));

        for func in self.funcs.iter() {
            match func {
                TransformFunc::TranslateX(x) => {
                    m = m * Matrix4::from_translation(Vector3::new(*x, 0.0, 0.0))
                },
                TransformFunc::TranslateY(y) => m = m * Matrix4::from_translation(Vector3::new(0.0, *y, 0.0)),
                TransformFunc::Translate(x, y) => m = m * Matrix4::from_translation(Vector3::new(*x, *y, 0.0)),

                TransformFunc::TranslateXPercent(x) => m = m * Matrix4::from_translation(Vector3::new(*x * width, 0.0, 0.0)),
                TransformFunc::TranslateYPercent(y) => m = m * Matrix4::from_translation(Vector3::new(0.0, *y * height, 0.0)),
                TransformFunc::TranslatePercent(x, y) => m = m * Matrix4::from_translation(Vector3::new(*x * width, *y * height, 0.0)),

                TransformFunc::ScaleX(x) => m = m * Matrix4::from_nonuniform_scale(*x, 1.0, 1.0),
                TransformFunc::ScaleY(y) => m = m * Matrix4::from_nonuniform_scale(1.0, *y, 1.0),
                TransformFunc::Scale(x, y) => m = m * Matrix4::from_nonuniform_scale(*x, *y, 1.0),
                
                TransformFunc::RotateZ(z) => m = m * Matrix4::from_angle_z(cgmath::Deg(*z)),
            }
        }
        m
    }
}

//对齐元素中的文本
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextAlign{
    Left,	//把文本排列到左边。默认值：由浏览器决定。
    Right,	//把文本排列到右边。
    Center,	//把文本排列到中间。
    Justify,	//实现两端对齐文本效果。
}

//设置元素中空白的处理方式
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum WhiteSpace{
    Normal, //	默认。空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围会换行。
    Nowrap, //	空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围文本也不会换行，文本会在在同一行上继续，直到遇到 <br> 标签为止。
    PreWrap, //	保留所有空白符序列，超出范围会换行。
    Pre, //	保留空白符，超出范围不会换行(利用yoga无法支持， 暂不支持)
    PreLine, //	合并空白符序列，如果存在换行符，优先保留换行符， 超出范围会换行。
}
impl WhiteSpace {
    pub fn allow_wrap(&self) -> bool {
        match *self {
            WhiteSpace::Nowrap |
            WhiteSpace::Pre => false,
            WhiteSpace::Normal |
            WhiteSpace::PreWrap |
            WhiteSpace::PreLine => true,
        }
    }

    pub fn preserve_newlines(&self) -> bool {
        match *self {
            WhiteSpace::Normal |
            WhiteSpace::Nowrap => false,
            WhiteSpace::Pre |
            WhiteSpace::PreWrap |
            WhiteSpace::PreLine => true,
        }
    }

    pub fn preserve_spaces(&self) -> bool {
        match *self {
            WhiteSpace::Normal |
            WhiteSpace::Nowrap |
            WhiteSpace::PreLine => false,
            WhiteSpace::Pre |
            WhiteSpace::PreWrap => true,
        }
    }
}

#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum FontStyle{
    Normal, //	默认值。标准的字体样式。
    Ttalic, //	斜体的字体样式。
    Oblique, //	倾斜的字体样式。
}
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum VerticalAlign{
    Top,
    Middle,
    Bottom
}

impl Default for Opacity {
  fn default() -> Opacity{
    Opacity(1.0)
  }
}

impl Show {
  #[inline]
  pub fn get_display(&self) -> Display {
    unsafe { transmute((self.0 & (ShowType::Display as usize)) as u8) }
  }

  #[inline]
  pub fn set_display(&mut self, display: Display){
    match display {
      Display::Flex => self.0 &= !(ShowType::Display as usize),
      Display::None => self.0 |= ShowType::Display as usize,
    }
  }

  #[inline]
  pub fn get_visibility(&self) -> bool{
    (self.0 & (ShowType::Visibility as usize)) != 0
  }

  #[inline]
  pub fn set_visibility(&mut self, visibility: bool){
    if visibility {
      self.0 |= ShowType::Visibility as usize;
    }else{
      self.0 &= !(ShowType::Visibility as usize);
    }
  }

  #[inline]
  pub fn get_enable(&self) -> bool{
    (self.0 & (ShowType::Enable as usize)) != 0
  }

  #[inline]
  pub fn set_enable(&mut self, enable: bool){
    if enable {
      self.0 |= ShowType::Enable as usize;
    }else{
      self.0 &= !(ShowType::Enable as usize);
    }
  }
}

impl Default for Show {
  fn default() -> Show {
    Show((ShowType::Enable as usize) | (ShowType::Visibility as usize))
  }
}

pub fn get_pos_uv<'a, C: Context + 'static + Send + Sync> (img: &Image<C>, clip: Option<&SrcClip>, fit: Option<&ObjectFit>, layout: &Layout) -> (Rect, Rect){
    let (size, mut uv1, mut uv2) = match clip {
        Some(c) => if c.size.x == 0.0 {
            let size = Vector2::new(img.src.width as f32 * (c.uv.1.x - c.uv.0.x).abs(), img.src.height as f32 * (c.uv.1.y - c.uv.0.y).abs());
            (size, c.uv.0, c.uv.1)
        }else{
            (c.size, c.uv.0, c.uv.1)
        },
        _ => (Vector2::new(img.src.width as f32, img.src.height as f32), Point2::new(0.0,0.0), Point2::new(1.0,1.0))
    };
    let mut p1 = Point2::new(layout.left + layout.border + layout.padding_left, layout.top + layout.border + layout.padding_top);
    let mut p2 = Point2::new(layout.left + layout.width - layout.border - layout.padding_right, layout.top + layout.width - layout.border - layout.padding_bottom);
    // 如果不是填充，总是居中显示。 如果在范围内，则修改点坐标。如果超出的部分，会进行剪切，剪切会修改uv坐标。
    match fit {
      Some(f) => match f.0 {
        FitType::None => {
          // 保持原有尺寸比例。同时保持内容原始尺寸大小。 超出部分会被剪切
          let w = p2.x - p1.x;
          let h = p2.y - p1.y;
          if size.x <= w {
            let x = (w - size.x) / 2.0;
            p1.x += x;
            p2.x -= x;
          }else{
            let x = (size.x - w) * (uv2.x - uv1.x) * 0.5 / size.x;
            uv1.x += x; 
            uv2.x -= x; 
          }
          if size.y <= h {
            let y = (h - size.y) / 2.0;
            p1.y += y;
            p2.y -= y;
          }else{
            let y = (size.y - h) * (uv2.y - uv1.y) * 0.5 / size.y;
            uv1.y += y;
            uv2.y -= y;
          }
        },
        FitType::Contain => {
          // 保持原有尺寸比例。保证内容尺寸一定可以在容器里面放得下。因此，此参数可能会在容器内留下空白。
          let w = p2.x - p1.x;
          let h = p2.y - p1.y;
          fill(&size, &mut p1, &mut p2, w, h);
        },
        FitType::Cover => {
          // 保持原有尺寸比例。保证内容尺寸一定大于容器尺寸，宽度和高度至少有一个和容器一致。超出部分会被剪切
          let w = p2.x - p1.x;
          let h = p2.y - p1.y;
          let rw = size.x/w;
          let rh = size.y/h;
          if rw > rh {
            let x = (size.x - w*rh) * (uv2.x - uv1.x) * 0.5 / size.x;
            uv1.x += x; 
            uv2.x -= x; 
          }else{
            let y = (size.y - h*rw) * (uv2.y - uv1.y) * 0.5 / size.y;
            uv1.y += y;
            uv2.y -= y;
          }
        },
        FitType::ScaleDown => {
          // 如果内容尺寸小于容器尺寸，则直接显示None。否则就是Contain
          let w = p2.x - p1.x;
          let h = p2.y - p1.y;
          if size.x <= w && size.y <= h {
            let x = (w - size.x) / 2.0;
            let y = (h - size.y) / 2.0;
            p1.x += x;
            p1.y += y;
            p2.x -= x;
            p2.y -= y;
          }else{
            fill(&size, &mut p1, &mut p2, w, h);
          }
        },
        _ => () // 填充。 内容拉伸填满整个容器，不保证保持原有的比例
      },
      // 默认情况是填充
      _ => ()
    };
    let pos = Rect(p1, Point2::new(p2.x, p1.y), p2, Point2::new(p1.x, p2.y));
    let uv = Rect(uv1, Point2::new(uv2.x, uv1.y), uv2, Point2::new(uv1.x, uv2.y));
    (pos, uv)
}
// 按比例缩放到容器大小，居中显示
fn fill(size: &Vector2, p1: &mut Point2, p2: &mut Point2, w: f32, h: f32){ 
    let rw = size.x/w;
    let rh = size.y/h;
    if rw > rh {
      let y = (h - size.y/rw)/2.0;
      p1.y += y;
      p2.y -= y;
    }else{
      let x = (w - size.x/rh)/2.0;
      p1.x += x;
      p2.x -= x;
    }
}