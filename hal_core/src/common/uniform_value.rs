use std::hash::{Hash, Hasher};

use ordered_float::NotNan;

/**
 * Uniform的值，包含各种Uniform枚举
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum UniformValue {
    Float1(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    Int1(i32),
    Int2(i32, i32),
    Int3(i32, i32, i32),
    Int4(i32, i32, i32, i32),
    FloatV1(Vec<f32>),
    FloatV2(Vec<f32>),
    FloatV3(Vec<f32>),
    FloatV4(Vec<f32>),
    IntV1(Vec<i32>),
    IntV2(Vec<i32>),
    IntV3(Vec<i32>),
    IntV4(Vec<i32>),
    MatrixV2(Vec<f32>),
    MatrixV3(Vec<f32>),
    MatrixV4(Vec<f32>),
}

impl Hash for UniformValue {
    fn hash<T: Hasher>(&self, hasher: &mut T) {
        match self {
            UniformValue::Float1(f1) => unsafe { NotNan::unchecked_new(*f1).hash(hasher) },
            UniformValue::Float2(f1, f2) => {
                unsafe { NotNan::unchecked_new(*f1).hash(hasher) };
                unsafe { NotNan::unchecked_new(*f2).hash(hasher) }
            }
            UniformValue::Float3(f1, f2, f3) => {
                unsafe { NotNan::unchecked_new(*f1).hash(hasher) };
                unsafe { NotNan::unchecked_new(*f2).hash(hasher) };
                unsafe { NotNan::unchecked_new(*f3).hash(hasher) };
            }
            UniformValue::Float4(f1, f2, f3, f4) => {
                unsafe { NotNan::unchecked_new(*f1).hash(hasher) };
                unsafe { NotNan::unchecked_new(*f2).hash(hasher) };
                unsafe { NotNan::unchecked_new(*f3).hash(hasher) };
                unsafe { NotNan::unchecked_new(*f4).hash(hasher) };
            }
            UniformValue::Int1(i1) => i1.hash(hasher),
            UniformValue::Int2(i1, i2) => {
                i1.hash(hasher);
                i2.hash(hasher);
            }
            UniformValue::Int3(i1, i2, i3) => {
                i1.hash(hasher);
                i2.hash(hasher);
                i3.hash(hasher);
            }
            UniformValue::Int4(i1, i2, i3, i4) => {
                i1.hash(hasher);
                i2.hash(hasher);
                i3.hash(hasher);
                i4.hash(hasher);
            }
            UniformValue::FloatV1(fv)
            | UniformValue::FloatV2(fv)
            | UniformValue::FloatV3(fv)
            | UniformValue::FloatV4(fv)
            | UniformValue::MatrixV2(fv)
            | UniformValue::MatrixV3(fv)
            | UniformValue::MatrixV4(fv) => {
                for f in fv.iter() {
                    unsafe { NotNan::unchecked_new(*f).hash(hasher) };
                }
            }
            UniformValue::IntV1(fv)
            | UniformValue::IntV2(fv)
            | UniformValue::IntV3(fv)
            | UniformValue::IntV4(fv) => {
                for i in fv.iter() {
                    i.hash(hasher);
                }
            }
        }
    }
}

impl Default for UniformValue {
    fn default() -> Self {
        UniformValue::Int1(0)
    }
}
