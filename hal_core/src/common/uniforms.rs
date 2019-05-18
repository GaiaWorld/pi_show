use std::collections::{HashMap};
use atom::{Atom};
use traits::{Sampler};

/** 
 * Uniform集合
 * 渲染一个物体，需要很多个Uniforms。
 */
pub struct Uniforms {
    pub values: HashMap<Atom, UniformValue>,
}

/** 
 * Uniform的值，包含各种Uniform枚举
 */
pub enum UniformValue {
    Float(u8, f32, f32, f32, f32),
    Int(u8, i32, i32, i32, i32),
    FloatV(u8, Vec<i32>),
    IntV(u8, i32),
    MatrixV(u8, Vec<f32>),
}

impl Uniforms {

    pub fn new() -> Self {
        Uniforms {
            values: HashMap::new(),
        }
    }

    pub fn set_int_1(&mut self, _name: &Atom, _v: i32) {

    }

    pub fn set_int_2(&mut self, _name: &Atom, _v1:i32, _v2: i32) {

    }

    pub fn set_int_3(&mut self, _name: &Atom, _v1:i32, _v2: i32, _v3: i32) {

    }

    pub fn set_int_4(&mut self, _name: &Atom, _v1:i32, _v2: i32, _v3: i32, _v4:  i32) {

    }
    
    pub fn set_float_1(&mut self, _name: &Atom, _v: f32) {

    }

    pub fn set_float_2(&mut self, _name: &Atom, _v1:f32, _v2: f32) {

    }

    pub fn set_float_3(&mut self, _name: &Atom, _v1:f32, _v2: f32, _v3: f32) {

    }

    pub fn set_float_4(&mut self, _name: &Atom, _v1:f32, _v2: f32, _v3: f32, _v4:  f32) {

    }

    pub fn set_int_1v(&mut self, _name: &Atom, _v: &[i32]) {

    }

    pub fn set_int_2v(&mut self, _name: &Atom, _v: &[i32]) {

    }

    pub fn set_int_3v(&mut self, _name: &Atom, _v: &[i32]) {

    }

    pub fn set_int_4v(&mut self, _name: &Atom, _v: &[i32]) {

    }

    pub fn set_float_1v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_float_2v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_float_3v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_float_4v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_mat_2v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_mat_3v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_mat_4v(&mut self, _name: &Atom, _v: &[f32]) {

    }

    pub fn set_sampler<T: Sampler>(&mut self, _name: &Atom, _sampler: T) {
        
    }
}