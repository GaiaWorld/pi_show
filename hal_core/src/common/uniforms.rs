use std::sync::{Arc, Weak};
use std::convert::AsRef;
use std::collections::{HashMap};

use atom::{Atom};
use traits::{Context};
use ShareRef;

/** 
 * Uniform集合
 * 渲染一个物体，需要一个或多个Uniforms。
 * shader中Uniform的类型和rust类型的对应关系如下：
 * 
 *    shader类型     Uniforms类型
 *      int            set_int_1
 *      ivec2          set_int_2
 *      ivec3          set_int_3
 *      ivec4          set_int_4
 * 
 *      float          set_float_1
 *      vec2           set_float_2
 *      vec3           set_float_3
 *      vec4           set_float_4
 * 
 *      int[N]         set_int_1v
 *      ivec2[N]       set_int_2v
 *      ivec3[N]       set_int_3v
 *      ivec4[N]       set_int_4v
 * 
 *      float[N]       set_float_1v
 *      vec2[N]        set_float_2v
 *      vec3[N]        set_float_3v
 *      vec4[N]        set_float_4v
 * 
 *      mat2 or mat2[N]   set_mat_2v
 *      mat3 or mat3[N]   set_mat_3v
 *      mat4 or mat4[N]   set_mat_4v
 * 
 *      sampler           set_sampler
 */
pub struct Uniforms<C: Context> {
    pub values: HashMap<Atom, UniformValue<C>>,
}

impl<C: Context> Clone for Uniforms<C> {
    fn clone(&self) -> Self {
        Uniforms{
            values: self.values.clone()
        }
    }
}

/** 
 * Uniform的值，包含各种Uniform枚举
 */
pub enum UniformValue<C: Context> {
    Float(u8, f32, f32, f32, f32), // 第一个是后面有效的个数，值只能为: 1, 2, 3, 4
    Int(u8, i32, i32, i32, i32),   // 第一个是后面有效的个数，值只能为: 1, 2, 3, 4
    FloatV(u8, Vec<f32>),          // 第一个是vec中的item_count，值只能为: 1, 2, 3, 4
    IntV(u8, Vec<i32>),            // 第一个是vec中的item_count，值只能为: 1, 2, 3, 4   
    MatrixV(u8, Vec<f32>),         // 第一个是vec中的item_count，值只能为: 2, 3, 4 
    Sampler(Weak<AsRef<C::ContextSampler>>, Weak<AsRef<C::ContextTexture>>),
}

impl<C: Context> Clone for UniformValue<C> {
    fn clone(&self) -> Self {
        match self {
            UniformValue::<C>::Float(c, v1, v2, v3, v4) => UniformValue::<C>::Float(*c, *v1, *v2, *v3, *v4),
            UniformValue::<C>::Int(c, v1, v2, v3, v4) => UniformValue::<C>::Int(*c, *v1, *v2, *v3, *v4),
            UniformValue::<C>::FloatV(c, v) => UniformValue::<C>::FloatV(*c, v.clone()),
            UniformValue::<C>::IntV(c, v) => UniformValue::<C>::IntV(*c, v.clone()),
            UniformValue::<C>::MatrixV(c, v) => UniformValue::<C>::MatrixV(*c, v.clone()),
            UniformValue::<C>::Sampler(s, t) => UniformValue::<C>::Sampler(s.clone(), t.clone()),
        }
    }
}

impl<C: Context> Uniforms<C> {

    pub fn set_int_1(&mut self, name: &Atom, v: i32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, _, _, _) if *rc == 1 => { 
                        *rv1 = v;
                    }
                    _ => {
                        assert!(false, "Uniforms::set_int_1 failed, type not match or count != 1");
                    }
                }
            })
            .or_insert(UniformValue::Int(1, v, 0, 0, 0));
    }

    pub fn set_int_2(&mut self, name: &Atom, v1:i32, v2: i32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, rv2, _, _) if *rc == 2 => { 
                        *rv1 = v1;
                        *rv2 = v2;
                    }
                    _ => {
                        assert!(false, "Uniforms::set_int_2 failed, type not match or count != 2");
                    }
                }
            })
            .or_insert(UniformValue::Int(2, v1, v2, 0, 0));
    }

    pub fn set_int_3(&mut self, name: &Atom, v1:i32, v2: i32, v3: i32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, rv2, rv3, _) if *rc == 3 => { 
                        *rv1 = v1;
                        *rv2 = v2;
                        *rv3 = v3;
                    }
                    _ => {
                        assert!(false, "Uniforms::set_int_3 failed, type not match or count != 3");
                    }
                }
            })
            .or_insert(UniformValue::Int(3, v1, v2, v3, 0));
    }

    pub fn set_int_4(&mut self, name: &Atom, v1:i32, v2: i32, v3: i32, v4:  i32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, rv2, rv3, rv4) if *rc == 4 => { 
                        *rv1 = v1;
                        *rv2 = v2;
                        *rv3 = v3;
                        *rv4 = v4;
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_int_4 failed, type not match or count != 4");
                    }
                }
            })
            .or_insert(UniformValue::Int(4, v1, v2, v3, v4));
    }
    
    pub fn set_float_1(&mut self, name: &Atom, v: f32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, _, _, _) if *rc == 1 => { 
                        *rv1 = v;
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_1 failed, type not match or count != 1");
                    }
                }
            })
            .or_insert(UniformValue::Float(1, v, 0.0, 0.0, 0.0));
    }

    pub fn set_float_2(&mut self, name: &Atom, v1:f32, v2: f32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, rv2, _, _) if *rc == 2 => { 
                        *rv1 = v1;
                        *rv2 = v2;
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_2 failed, type not match or count != 2");
                    }
                }
            })
            .or_insert(UniformValue::Float(2, v1, v2, 0.0, 0.0));
    }

    pub fn set_float_3(&mut self, name: &Atom, v1:f32, v2: f32, v3: f32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, rv2, rv3, _) if *rc == 3 => { 
                        *rv1 = v1;
                        *rv2 = v2;
                        *rv3 = v3;
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_3 failed, type not match or count != 3");
                    }
                }
            })
            .or_insert(UniformValue::Float(3, v1, v2, v3, 0.0));
    }

    pub fn set_float_4(&mut self, name: &Atom, v1:f32, v2: f32, v3: f32, v4:  f32) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, rv2, rv3, rv4) if *rc == 4 => { 
                        *rv1 = v1;
                        *rv2 = v2;
                        *rv3 = v3;
                        *rv4 = v4;
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_4 failed, type not match or count != 4");
                    }
                }
            })
            .or_insert(UniformValue::Float(4, v1, v2, v3, v4));
    }

    pub fn set_int_1v(&mut self, name: &Atom, v: &[i32]) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) if *rc == 1 && rvec.len() == v.len() => { 
                         rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_int_1v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::IntV(1, v.to_vec()));
    }

    pub fn set_int_2v(&mut self, name: &Atom, v: &[i32]) {
        assert!(v.len() % 2 == 0, "set_int_2v failed, v.len() % 2 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) if *rc == 2 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_int_2v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::IntV(2, v.to_vec()));
    }

    pub fn set_int_3v(&mut self, name: &Atom, v: &[i32]) {
        assert!(v.len() % 3 == 0, "set_int_3v failed, v.len() % 3 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) if *rc == 3 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_int_3v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::IntV(3, v.to_vec()));
    }

    pub fn set_int_4v(&mut self, name: &Atom, v: &[i32]) {
        assert!(v.len() % 3 == 0, "set_int_4v failed, v.len() % 4 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) if *rc == 4 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_int_4v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::IntV(4, v.to_vec()));
    }

    pub fn set_float_1v(&mut self, name: &Atom, v: &[f32]) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) if *rc == 1 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_1v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::FloatV(1, v.to_vec()));
    }

    pub fn set_float_2v(&mut self, name: &Atom, v: &[f32]) {
        assert!(v.len() % 2 == 0, "set_float_2v failed, v.len() % 2 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) if *rc == 2 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_2v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::FloatV(2, v.to_vec()));
    }

    pub fn set_float_3v(&mut self, name: &Atom, v: &[f32]) {
        assert!(v.len() % 3 == 0, "set_float_3v failed, v.len() % 3 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) if *rc == 3 && rvec.len() == v.len() => {
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_3v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::FloatV(3, v.to_vec()));
    }

    pub fn set_float_4v(&mut self, name: &Atom, v: &[f32]) {
        assert!(v.len() % 4 == 0, "set_float_4v failed, v.len() % 4 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) if *rc == 4 && rvec.len() == v.len() => {
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_float_4v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::FloatV(4, v.to_vec()));
    }

    /** 
     * 设置2*2的矩阵，注意：顺序列优先
     */
    pub fn set_mat_2v(&mut self, name: &Atom, v: &[f32]) {
        assert!(v.len() % 4 == 0, "set_mat_2v failed, v.len() % 4 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::MatrixV(rc, rvec) if *rc == 2 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_mat_2v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::MatrixV(2, v.to_vec()));
    }

    /** 
     * 设置3*3的矩阵，注意：顺序列优先
     */
    pub fn set_mat_3v(&mut self, name: &Atom, v: &[f32]) {
        assert!(v.len() % 9 == 0, "set_mat_3v failed, v.len() % 9 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::MatrixV(rc, rvec) if *rc == 3 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_mat_3v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::MatrixV(3, v.to_vec()));
    }

    /** 
     * 设置4*4的矩阵，注意：顺序列优先
     */
    pub fn set_mat_4v(&mut self, name: &Atom, v: &[f32]) {
        assert!(v.len() % 16 == 0, "set_mat_4v failed, v.len() % 16 != 0");
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::MatrixV(rc, rvec) if *rc == 4 && rvec.len() == v.len() => { 
                        rvec.copy_from_slice(v);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_mat_4v failed, type or count not match");
                    }
                }
            })
            .or_insert(UniformValue::MatrixV(4, v.to_vec()));
    }

    /** 
     * 设置纹理对应的Sampler，Uniform设置纹理只能用Sampler的方式设置。
     */
    pub fn set_sampler(&mut self, name: &Atom, sampler: &ShareRef<C::ContextSampler>, texture: &ShareRef<C::ContextTexture>) {
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Sampler(s, t) => { 
                        *s = Arc::downgrade(sampler);
                        *t = Arc::downgrade(texture);
                    }
                    _ => { 
                        assert!(false, "Uniforms::set_sampler failed, type not match");
                    }
                }
            })
            .or_insert(UniformValue::Sampler(Arc::downgrade(sampler), Arc::downgrade(texture)));
    }
}

impl<C: Context> AsRef<Self> for Uniforms<C> {
    fn as_ref(&self) -> &Self {
        &self
    }
}