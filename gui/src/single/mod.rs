use component::{Point2, Matrix4};
use cgmath::Ortho;
use slab::Slab;

pub mod oct;

#[derive(Debug)]
pub struct OverflowClip{
    pub id_vec: [usize;8],
    pub clip: [[Point2;4];8],
}

#[derive(Debug)]
pub struct ViewMatrix(pub Matrix4);

#[derive(Clone)]
pub struct ProjectionMatrix(pub Matrix4);

impl ProjectionMatrix {
    pub fn new(width: f32, height: f32, near: f32, far: f32) -> ProjectionMatrix{
        let ortho = Ortho {
            left: 0.0,
            right: width,
            bottom: height, 
            top: 0.0,
            near: near,
            far: far,
        };
        ProjectionMatrix(Matrix4::from(ortho))
        // let (left, right, top, bottom, near, far) = (0.0, width, 0.0, height, -8388607.0, 8388608.0);
        // ProjectionMatrix(Matrix4::new(
        //         2.0 / (right - left),                  0.0,                               0.0,                        0.0,
        //             0.0,                     2.0 / (top - bottom),                       0.0,                        0.0,
        //             0.0,                              0.0,                       -2.0 / (far - near),   -(far + near) / (far - near),
        //     -(right + left) / (right - left), -(top + bottom) / (top - bottom),           0.0,                        1.0
        // ))
    }
}
