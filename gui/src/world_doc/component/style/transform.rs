use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Builder};
use wcs::world::{ComponentMgr};

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
    XY(OriginValue, OriginValue),
}

impl TransformOrigin {
    pub fn to_value(&self, width: f32, height: f32) -> cg::Point2<f32> {
        match self {
            TransformOrigin::Center => cg::Point2::new(0.5 * width, 0.5 * height),
            TransformOrigin::XY(x, y) => {
                cg::Point2::new(
                    match x {
                        OriginValue::ValueLength(v) => v.clone(),
                        OriginValue::ValuePercent(v) => v * width,
                    },
                    match y {
                        OriginValue::ValueLength(v) => v.clone(),
                        OriginValue::ValuePercent(v) => v * height,
                    }
                )
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OriginValue {
    ValueLength(f32),
    ValuePercent(f32),
}

#[allow(unused_attributes)]
#[derive(Debug, Clone, Component, Builder, Default)]
pub struct Transform {
    pub funcs: Vec<TransformFunc>,
    pub origin: TransformOrigin,
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
                    println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx:{}", x);
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