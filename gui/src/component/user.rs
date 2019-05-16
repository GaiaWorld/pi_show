/// 用户操作的组件


use std::{
  f32,
  default::Default,
  mem::transmute,
};

use map::{vecmap::VecMap};
use cg::color::Color as CgColor;

use ecs::component::Component;
use component::{LengthUnit, Display, Color};

#[derive(Component, Default)]
pub struct ZIndex(pub isize);

//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(pub f32);

#[derive(Deref, DerefMut, Component, Debug)]
pub struct Show(pub usize);

#[derive(Debug, Clone, Component, Default)]
pub struct Transform {
    pub funcs: Vec<TransformFunc>,
    pub origin: TransformOrigin,
}

#[derive(Debug, Clone, Component)]
pub struct BackgroundColor(pub Color);

#[derive(Debug, Clone, Component)]
pub struct BackgroundImage(pub usize);

#[derive(Debug, Clone, Component)]
pub struct BorderColor(CgColor<f32>);

#[derive(Debug, Clone, Component)]
pub struct BorderImage(pub usize);

#[derive(Debug, Clone, Component)]
pub struct BorderRadius(pub f32);

#[derive(Debug, Clone, Default, Component)]
pub struct BoxShadow{
    pub h: f32,
    pub v: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: CgColor<f32>,
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
    Show(6)
  }
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
    pub fn to_value(&self, width: f32, height: f32) -> cg::Point2<f32> {
        match self {
            TransformOrigin::Center => cg::Point2::new(0.5 * width, 0.5 * height),
            TransformOrigin::XY(x, y) => {
                cg::Point2::new(
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
    pub fn matrix(&self, width: f32, height: f32, origin: &cg::Point2<f32>) -> cg::Matrix4<f32> {
        // M = T * R * S
        // let mut m = cg::Matrix4::new(
        //     1.0, 0.0, 0.0, 0.0,
        //     0.0, 1.0, 0.0, 0.0,
        //     0.0, 0.0, 1.0, 0.0,
        //     0.0, 0.0, 0.0, 1.0,
        // );

        let value = self.origin.to_value(width, height);
        let mut m = cg::Matrix4::from_translation(cg::Vector3::new(origin.x + value.x, origin.y + value.y, 0.0));

        for func in self.funcs.iter() {
            match func {
                TransformFunc::TranslateX(x) => {
                    m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x, 0.0, 0.0))
                },
                TransformFunc::TranslateY(y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(0.0, *y, 0.0)),
                TransformFunc::Translate(x, y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x, *y, 0.0)),

                TransformFunc::TranslateXPercent(x) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x * width, 0.0, 0.0)),
                TransformFunc::TranslateYPercent(y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(0.0, *y * height, 0.0)),
                TransformFunc::TranslatePercent(x, y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x * width, *y * height, 0.0)),

                TransformFunc::ScaleX(x) => m = m * cg::Matrix4::from_nonuniform_scale(*x, 1.0, 1.0),
                TransformFunc::ScaleY(y) => m = m * cg::Matrix4::from_nonuniform_scale(1.0, *y, 1.0),
                TransformFunc::Scale(x, y) => m = m * cg::Matrix4::from_nonuniform_scale(*x, *y, 1.0),

                TransformFunc::RotateZ(z) => m = m * cg::Matrix4::from_angle_z(cg::Deg(*z)),
            }
        }
        m
    }
}