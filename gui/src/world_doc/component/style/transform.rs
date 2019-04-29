use std::ops::{Deref};

use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent, Builder};
use wcs::world::{ComponentMgr};

use component::math::Point2;

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

// pub enum TransformOrigin{

// }

#[allow(unused_attributes)]
#[derive(Debug, Clone, Component, Builder, Default)]
pub struct Transform {
    pub funcs: Vec<TransformFunc>,
    pub origin: Point2,
}
 
impl Transform {

    pub fn matrix(&self, percent_base: cg::Vector4<f32>) -> cg::Matrix4<f32> {
        // M = T * R * S
        let mut m = cg::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        println!("percent_base: {:?}", percent_base);

        for func in self.funcs.iter() {
            match func {
                TransformFunc::TranslateX(x) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x, 0.0, 0.0)),
                TransformFunc::TranslateY(y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(0.0, *y, 0.0)),
                TransformFunc::Translate(x, y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x, *y, 0.0)),

                TransformFunc::TranslateXPercent(x) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x * percent_base.x / 100.0, 0.0, 0.0)),
                TransformFunc::TranslateYPercent(y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(0.0, *y * percent_base.y / 100.0, 0.0)),
                TransformFunc::TranslatePercent(x, y) => m = m * cg::Matrix4::from_translation(cg::Vector3::new(*x * percent_base.x / 100.0, *y * percent_base.y / 100.0, 0.0)),

                TransformFunc::ScaleX(x) => m = m * cg::Matrix4::from_nonuniform_scale(*x, 1.0, 1.0),
                TransformFunc::ScaleY(y) => m = m * cg::Matrix4::from_nonuniform_scale(1.0, *y, 1.0),
                TransformFunc::Scale(x, y) => m = m * cg::Matrix4::from_nonuniform_scale(*x, *y, 1.0),

                TransformFunc::RotateZ(z) => m = m * cg::Matrix4::from_angle_z(cg::Deg(*z)),
            }
        }
        m
    }
}