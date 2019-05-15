pub mod calc;
pub mod user;

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

pub type Matrix4 = cgmath::Matrix4<f32>;
pub type Point2 = cgmath::Point2<f32>;
pub type Point3 = cgmath::Point3<f32>;
pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Aabb3 = collision::Aabb3<f32>;
