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
    Float(u8, f32, f32, f32, f32), // 第一个是后面有效的个数，值只能为: 1, 2, 3, 4
    Int(u8, i32, i32, i32, i32),   // 第一个是后面有效的个数，值只能为: 1, 2, 3, 4
    FloatV(u8, Vec<f32>),          // 第一个是vec中的item_count，值只能为: 1, 2, 3, 4
    IntV(u8, Vec<i32>),                 // 第一个是vec中的item_count，值只能为: 1, 2, 3, 4   
    MatrixV(u8, Vec<f32>),         // 第一个是vec中的item_count，值只能为: 2, 3, 4 
}

impl Uniforms {

    pub fn new() -> Self {
        Uniforms {
            values: HashMap::new(),
        }
    }

    pub fn set_int_1(&mut self, name: &Atom, v: i32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, _, _, _) => { 
                        if *rc == 1 {
                            *rv1 = v;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Int(1, v, 0, 0, 0));
        res
    }

    pub fn set_int_2(&mut self, name: &Atom, v1:i32, v2: i32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, rv2, _, _) => { 
                        if *rc == 2 {
                            *rv1 = v1;
                            *rv2 = v2;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Int(2, v1, v2, 0, 0));
        res
    }

    pub fn set_int_3(&mut self, name: &Atom, v1:i32, v2: i32, v3: i32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, rv2, rv3, _) => { 
                        if *rc == 3 {
                            *rv1 = v1;
                            *rv2 = v2;
                            *rv3 = v3;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Int(3, v1, v2, v3, 0));
        res
    }

    pub fn set_int_4(&mut self, name: &Atom, v1:i32, v2: i32, v3: i32, v4:  i32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Int(rc, rv1, rv2, rv3, rv4) => { 
                        if *rc == 4 {
                            *rv1 = v1;
                            *rv2 = v2;
                            *rv3 = v3;
                            *rv4 = v4;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Int(4, v1, v2, v3, v4));
        res
    }
    
    pub fn set_float_1(&mut self, name: &Atom, v: f32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, _, _, _) => { 
                        if *rc == 1 {
                            *rv1 = v;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Float(1, v, 0.0, 0.0, 0.0));
        res
    }

    pub fn set_float_2(&mut self, name: &Atom, v1:f32, v2: f32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, rv2, _, _) => { 
                        if *rc == 2 {
                            *rv1 = v1;
                            *rv2 = v2;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Float(2, v1, v2, 0.0, 0.0));
        res
    }

    pub fn set_float_3(&mut self, name: &Atom, v1:f32, v2: f32, v3: f32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, rv2, rv3, _) => { 
                        if *rc == 3 {
                            *rv1 = v1;
                            *rv2 = v2;
                            *rv3 = v3;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Float(3, v1, v2, v3, 0.0));
        res
    }

    pub fn set_float_4(&mut self, name: &Atom, v1:f32, v2: f32, v3: f32, v4:  f32) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::Float(rc, rv1, rv2, rv3, rv4) => { 
                        if *rc == 4 {
                            *rv1 = v1;
                            *rv2 = v2;
                            *rv3 = v3;
                            *rv4 = v4;
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::Float(4, v1, v2, v3, v4));
        res
    }

    pub fn set_int_1v(&mut self, name: &Atom, v: &[i32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) => { 
                        if *rc == 1 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::IntV(1, v.to_vec()));
        res
    }

    pub fn set_int_2v(&mut self, name: &Atom, v: &[i32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) => { 
                        if *rc == 2 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::IntV(2, v.to_vec()));
        res
    }

    pub fn set_int_3v(&mut self, name: &Atom, v: &[i32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) => { 
                        if *rc == 3 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::IntV(3, v.to_vec()));
        res
    }

    pub fn set_int_4v(&mut self, name: &Atom, v: &[i32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::IntV(rc, rvec) => { 
                        if *rc == 4 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::IntV(4, v.to_vec()));
        res
    }

    pub fn set_float_1v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) => { 
                        if *rc == 1 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::FloatV(1, v.to_vec()));
        res
    }

    pub fn set_float_2v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) => { 
                        if *rc == 2 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::FloatV(2, v.to_vec()));
        res
    }

    pub fn set_float_3v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) => { 
                        if *rc == 3 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::FloatV(3, v.to_vec()));
        res
    }

    pub fn set_float_4v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::FloatV(rc, rvec) => { 
                        if *rc == 4 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::FloatV(4, v.to_vec()));
        res
    }

    pub fn set_mat_2v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::MatrixV(rc, rvec) => { 
                        if *rc == 2 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::MatrixV(2, v.to_vec()));
        res
    }

    pub fn set_mat_3v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::MatrixV(rc, rvec) => { 
                        if *rc == 3 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::MatrixV(3, v.to_vec()));
        res
    }

    pub fn set_mat_4v(&mut self, name: &Atom, v: &[f32]) -> Result<(), String> {
        let mut res = Ok(());
        self.values.entry(name.clone())
            .and_modify(|rv| {
                match rv {
                    UniformValue::MatrixV(rc, rvec) => { 
                        if *rc == 4 && rvec.len() == v.len() {
                            rvec.copy_from_slice(v);
                        } else { 
                            res = Err("value type not match".to_string()); 
                        } 
                    }
                    _ => { 
                        res = Err("value type not match".to_string()); 
                    }
                }
            })
            .or_insert(UniformValue::MatrixV(4, v.to_vec()));
        res
    }

    pub fn set_sampler<T: Sampler>(&mut self, name: &Atom, sampler: T) -> Result<(), String> {
        Err("".to_string())
    }
}