use std::fmt;

/** 
 * Uniform的值，包含各种Uniform枚举
 */
pub enum UniformValue {
    Float(u8, f32, f32, f32, f32), // 第一个是后面有效的个数，值只能为: 1, 2, 3, 4
    Int(u8, i32, i32, i32, i32),   // 第一个是后面有效的个数，值只能为: 1, 2, 3, 4
    FloatV(u8, Vec<f32>),          // 第一个是vec中的item_count，值只能为: 1, 2, 3, 4
    IntV(u8, Vec<i32>),            // 第一个是vec中的item_count，值只能为: 1, 2, 3, 4   
    MatrixV(u8, Vec<f32>),         // 第一个是vec中的item_count，值只能为: 2, 3, 4 
}

impl fmt::Debug for UniformValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniformValue::Float(r0, r1, r2, r3, r4) => write!(f, " UniformValue::Float({}, {}, {}, {}, {})", r0, r1, r2, r3, r4),
            UniformValue::Int(r0, r1, r2, r3, r4) => write!(f, " UniformValue::Int({}, {}, {}, {}, {})", r0, r1, r2, r3, r4),
            UniformValue::FloatV(r0, r1) => write!(f, " UniformValue::FloatV({}, {:?})", r0, r1),
            UniformValue::IntV(r0, r1) => write!(f, " UniformValue::IntV({}, {:?})", r0, r1),
            UniformValue::MatrixV(r0, r1) => write!(f, " UniformValue::MatrixV({}, {:?})", r0, r1),
        }
        
    }
}

impl Clone for UniformValue {
    fn clone(&self) -> Self {
        match self {
            UniformValue::Float(c, v1, v2, v3, v4) => UniformValue::Float(*c, *v1, *v2, *v3, *v4),
            UniformValue::Int(c, v1, v2, v3, v4) => UniformValue::Int(*c, *v1, *v2, *v3, *v4),
            UniformValue::FloatV(c, v) => UniformValue::FloatV(*c, v.clone()),
            UniformValue::IntV(c, v) => UniformValue::IntV(*c, v.clone()),
            UniformValue::MatrixV(c, v) => UniformValue::MatrixV(*c, v.clone()),
        }
    }
}