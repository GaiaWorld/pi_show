// use std::ops::{Deref, DerefMut};
// use std::default::Default;

// pub trait OwnDefault {
//     fn default() -> Self;
// }

// pub type Aabb3 = cg::Aabb3<f32>;

// impl OwnDefault for Aabb3 {
//     fn default() -> Aabb3 {
//         Aabb3{
//             min: cg::Point3::new(0.0, 0.0, 0.0),
//             max: cg::Point3::new(0.0, 0.0, 0.0),
//         }
//     }
// }

// pub type Point2 = cg::Point2<f32>;

// impl OwnDefault for Point2 {
//     fn default() -> Point2 {
//         Point2::new(0.0, 0.0)
//     }
// }

// pub type Point3 = cg::Point3<f32>;

// impl OwnDefault for Point3 {
//     fn default() -> Point3 {
//         Point3::new(0.0, 0.0, 0.0)
//     }
// }

// pub type Vector3 = cg::Vector3<f32>;

// impl OwnDefault for Vector3 {
//     fn default() -> Vector3 {
//         Vector3::new(0.0, 0.0, 0.0)
//     }
// }

// pub type Vector2 = cg::Vector2<f32>;

// impl OwnDefault for Vector2 {
//     fn default() -> Vector2 {
//         Vector2::new(0.0, 0.0)
//     }
// }

// pub type Quaternion = cg::Quaternion<f32>;

// impl OwnDefault for Quaternion {
//     fn default() -> Quaternion {
//         Quaternion::new(0.0, 0.0, 0.0, 0.0)
//     }
// }

// pub type Vector4 = cg::Vector4<f32>;

// impl OwnDefault for Vector4 {
//     fn default() -> Vector4 {
//         Vector4::new(0.0, 0.0, 0.0, 0.0)
//     }
// }

// pub type Matrix4 = cg::Matrix4<f32>;

// pub type Matrix3 = cg::Matrix3<f32>;

// #[derive(Debug, Clone)]
// pub struct ColorRGBAFloat(cg::color::Color<f32>);

// impl Deref for ColorRGBAFloat{
//     type Target = cg::color::Color<f32>;
//     fn deref(&self) -> &cg::color::Color<f32> {
//         &self.0
//     }
// }

// impl DerefMut for ColorRGBAFloat{
//     fn deref_mut(&mut self) -> &mut cg::color::Color<f32> {
//         &mut self.0
//     }
// }

// impl Default for ColorRGBAFloat{
//     fn default() -> ColorRGBAFloat {
//         ColorRGBAFloat(cg::color::Color::new(0.0, 0.0,0.0,1.0))
//     }
// }

// // pub struct ColorRGBAUint(cg::color::Color<u8>);

// // impl Deref for ColorRGBAUint{
// //     type Target = cg::color::Color<u8>;
// //     fn deref(&self) -> &cg::color::Color<u8> {
// //         &self.0
// //     }
// // }

// // impl DerefMut for ColorRGBAUint{
// //     fn deref_mut(&mut self) -> &mut cg::color::Color<u8> {
// //         &mut self.0
// //     }
// // }

// // impl Default for ColorRGBAUint{
// //     fn default() -> ColorRGBAUint {
// //         ColorRGBAUint(cg::color::Color::new(0, 0, 0, 255))
// //     }
// // }

use std::ops::{Deref, DerefMut};
use std::default::Default;

use cg::Zero;
use wcs::component::{ComponentGroup, ComponentGroupTree, ModifyFieldEvent, CreateEvent, DeleteEvent};
use wcs::world::{ComponentMgr};

#[derive(Debug, Clone, Copy)]
pub struct Aabb3(pub cg::Aabb3<f32>);

impl Deref for Aabb3{
    type Target = cg::Aabb3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Aabb3{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Aabb3{
    fn default() -> Aabb3{
        Aabb3(cg::Aabb3::new(cg::Point3::new(0.0, 0.0, 0.0), cg::Point3::new(1.0, 1.0, 1.0)))
    }
}

impl Aabb3 {
    pub fn new(min: cg::Point3<f32>, max: cg::Point3<f32>) -> Aabb3{
        Aabb3(cg::Aabb3::new(min, max))
    }
}

getter_setter!(
    struct Aabb3{
        min: cg::Point3<f32>,
        max: cg::Point3<f32>,
    }
);

component!(
    struct Aabb3{
        min: cg::Point3<f32>,
        max: cg::Point3<f32>,
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Vector2(pub cg::Vector2<f32>);

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2{
        Vector2(cg::Vector2::new(x, y))
    }
}

impl Deref for Vector2{
    type Target = cg::Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector2{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Vector2{
    fn default() -> Vector2{
        Vector2(cg::Vector2::new(0.0, 0.0))
    }
}

getter_setter!(
    struct Vector2{
        x: f32,
        y: f32,
    }
);

component!(
    struct Vector2{
        x: f32,
        y: f32,
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Matrix4(pub cg::Matrix4<f32>);

impl Deref for Matrix4{
    type Target = cg::Matrix4<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Matrix4{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Matrix4{
    fn default() -> Matrix4{
        let t: cg::Matrix4<f32> =  cg::Matrix4::from_translation(cg::Vector3::new(0.0, 0.0, 0.0));
        let r: cg::Matrix4<f32> = cg::Euler::new(cg::Rad::zero(), cg::Rad::zero(), cg::Rad::zero()).into();
        let s: cg::Matrix4<f32> = cg::Matrix4::from_scale(1.0);
        Matrix4(t * r * s)
    }
}

getter_setter!(
    struct Matrix4{
        x: cg::Vector4<f32>,
        y: cg::Vector4<f32>,
        z: cg::Vector4<f32>,
        w: cg::Vector4<f32>,
    }
);

component!(
    struct Matrix4{
        x: cg::Vector4<f32>,
        y: cg::Vector4<f32>,
        z: cg::Vector4<f32>,
        w: cg::Vector4<f32>,
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Quaternion(pub cg::Quaternion<f32>);

impl Deref for Quaternion{
    type Target = cg::Quaternion<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Quaternion{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Quaternion{
    fn default() -> Quaternion{
        let r: cg::Quaternion<f32> = cg::Euler::new(cg::Rad::zero(), cg::Rad::zero(), cg::Rad::zero()).into();
        Quaternion(r)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector3(pub cg::Vector3<f32>);

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3{
        Vector3(cg::Vector3::new(x, y, z))
    }
}

impl Deref for Vector3{
    type Target = cg::Vector3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector3{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Vector3{
    fn default() -> Vector3{
        Vector3(cg::Vector3::new(0.0, 0.0, 0.0))
    }
}

getter_setter!(
    struct Vector3{
        x: f32,
        y: f32,
        z: f32,
    }
);

component!(
    struct Vector3{
        x: f32,
        y: f32,
        z: f32,
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Color(pub cg::color::Color<f32>);

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color{
        Color(cg::color::Color::new(r, g, b, a))
    }
}

impl Deref for Color{
    type Target = cg::color::Color<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Color{
    fn default() -> Color{
        Color(cg::color::Color::new(0.0, 0.0, 0.0, 0.0))
    }
}

getter_setter!(
    struct Color{
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }
);

component!(
    struct Color{
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Scale(pub cg::Vector3<f32>);

impl Deref for Scale{
    type Target = cg::Vector3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scale{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Scale{
    fn default() -> Scale{
        Scale(cg::Vector3::new(1.0, 1.0, 1.0))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point2(pub cg::Point2<f32>);

impl Deref for Point2{
    type Target = cg::Point2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Point2{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Point2{
    fn default() -> Point2{
        Point2(cg::Point2::new(0.0, 0.0))
    }
}

/// UV 
#[derive(Debug, Clone, Copy, Default)]
pub struct UV {
    pub min: Point2,
    pub max: Point2,
    pub tex_index: usize,
}
