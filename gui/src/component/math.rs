pub trait OwnDefault {
    fn default() -> Self;
}

pub type Aabb3 = cg::Aabb3<f32>;

impl OwnDefault for Aabb3 {
    fn default() -> Aabb3 {
        Aabb3{
            min: cg::Point3::new(0.0, 0.0, 0.0),
            max: cg::Point3::new(0.0, 0.0, 0.0),
        }
    }
}

pub type Point2 = cg::Point2<f32>;

impl OwnDefault for Point2 {
    fn default() -> Point2 {
        Point2::new(0.0, 0.0)
    }
}

pub type Point3 = cg::Point3<f32>;

impl OwnDefault for Point3 {
    fn default() -> Point3 {
        Point3::new(0.0, 0.0, 0.0)
    }
}

pub type Vector3 = cg::Vector3<f32>;

impl OwnDefault for Vector3 {
    fn default() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }
}

pub type Vector2 = cg::Vector2<f32>;

impl OwnDefault for Vector2 {
    fn default() -> Vector2 {
        Vector2::new(0.0, 0.0)
    }
}

pub type Quaternion = cg::Quaternion<f32>;

impl OwnDefault for Quaternion {
    fn default() -> Quaternion {
        Quaternion::new(0.0, 0.0, 0.0, 0.0)
    }
}

pub type Vector4 = cg::Vector4<f32>;

impl OwnDefault for Vector4 {
    fn default() -> Vector4 {
        Vector4::new(0.0, 0.0, 0.0, 0.0)
    }
}

pub type Matrix4 = cg::Matrix4<f32>;

pub type Matrix3 = cg::Matrix3<f32>;
