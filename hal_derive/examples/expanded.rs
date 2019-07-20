#![feature(prelude_import)]
#![no_std]
#![feature(proc_macro_hygiene)]
#![recursion_limit = "512"]
#[prelude_import]

extern crate std;



///一个基本的例子， 演示如何使用 uniform_buffer! 和 program_paramter! 定义UniformBuffer 和 ProgramParamter
extern crate hal_core;

extern crate hal_derive;
extern crate share;

use hal_core::*;
use share::Share;

pub struct BgColor {
    values: [UniformValue; 2],
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for BgColor {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            BgColor { values: ref __self_0_0 } => {
                let mut debug_trait_builder = f.debug_struct("BgColor");
                let _ = debug_trait_builder.field("values", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::clone::Clone for BgColor {
    #[inline]
    fn clone(&self) -> BgColor {
        match *self {
            BgColor { values: ref __self_0_0 } =>
            BgColor{values: ::std::clone::Clone::clone(&(*__self_0_0)),},
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::default::Default for BgColor {
    #[inline]
    fn default() -> BgColor {
        BgColor{values: ::std::default::Default::default(),}
    }
}
impl BgColor {
    pub const
    FIELDS:
    [&'static str; 2]
    =
    ["color", "depth"];
    #[inline]
    pub fn new(color: UniformValue, depth: UniformValue) -> Self {
        Self{values: [color, depth],}
    }
}
impl UniformBuffer for BgColor {
    #[inline]
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
    #[inline]
    fn get_values(&self) -> &[UniformValue] { &self.values[..] }
    fn get_value(&self, name: &str) -> Option<&UniformValue> {
        match name {
            "color" => Some(&self.values[0]),
            "depth" => Some(&self.values[1]),
            _ => None,
        }
    }
    fn set_value(&mut self, name: &str, value: UniformValue) -> bool {
        match name {
            "color" => self.values[0] = value,
            "depth" => self.values[1] = value,
            _ => return false,
        };
        true
    }
}

pub struct Clip {
    values: [UniformValue; 1],
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::default::Default for Clip {
    #[inline]
    fn default() -> Clip { Clip{values: ::std::default::Default::default(),} }
}
impl Clip {
    pub const
    FIELDS:
    [&'static str; 1]
    =
    ["index"];
    #[inline]
    pub fn new(index: UniformValue) -> Self { Self{values: [index],} }
}
impl UniformBuffer for Clip {
    #[inline]
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
    #[inline]
    fn get_values(&self) -> &[UniformValue] { &self.values[..] }
    fn get_value(&self, name: &str) -> Option<&UniformValue> {
        match name { "index" => Some(&self.values[0]), _ => None, }
    }
    fn set_value(&mut self, name: &str, value: UniformValue) -> bool {
        match name { "index" => self.values[0] = value, _ => return false, };
        true
    }
}

pub struct Color {
    uniforms: [Share<dyn UniformBuffer>; 2],
    single_uniforms: [UniformValue; 1],
    textures: [(Share<HalTexture>, Share<HalSampler>); 1],
}
impl std::default::Default for Color {
    fn default() -> Self {
        Self{uniforms:
                 [Share::new(BgColor::default()),
                  Share::new(Clip::default())],
             single_uniforms: [UniformValue::default()],
             textures:
                 [(Share::new(HalTexture(0, 0)),
                   Share::new(HalSampler(0, 0)))],}
    }
}
impl Color {
    pub const
    FIELDS:
    [&'static str; 2]
    =
    ["common", "clip"];
    pub const
    SINGLE_FIELDS:
    [&'static str; 1]
    =
    ["alpha"];
    pub const
    TEXTURE_FIELDS:
    [&'static str; 1]
    =
    ["texture"];
}
impl ProgramParamter for Color {
    #[inline]
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
    #[inline]
    fn get_single_uniform_layout(&self) -> &[&str] {
        &Self::SINGLE_FIELDS[..]
    }
    #[inline]
    fn get_texture_layout(&self) -> &[&str] { &Self::TEXTURE_FIELDS[..] }
    #[inline]
    fn get_values(&self) -> &[Share<dyn UniformBuffer>] { &self.uniforms[..] }
    #[inline]
    fn get_single_uniforms(&self) -> &[UniformValue] {
        &self.single_uniforms[..]
    }
    #[inline]
    fn get_textures(&self) -> &[(Share<HalTexture>, Share<HalSampler>)] {
        &self.textures[..]
    }
    fn get_value(&self, name: &str) -> Option<&Share<dyn UniformBuffer>> {
        match name {
            "common" => Some(&self.uniforms[0]),
            "clip" => Some(&self.uniforms[1]),
            _ => None,
        }
    }
    fn get_single_uniform(&self, name: &str) -> Option<&UniformValue> {
        match name { "alpha" => Some(&self.single_uniforms[0]), _ => None, }
    }
    fn get_texture(&self, name: &str)
     -> Option<&(Share<HalTexture>, Share<HalSampler>)> {
        match name { "texture" => Some(&self.textures[0]), _ => None, }
    }
    fn set_value(&self, name: &str, value: Share<dyn UniformBuffer>) -> bool {
        let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
        match name {
            "common" => s.uniforms[0] = value,
            "clip" => s.uniforms[1] = value,
            _ => {
                return false
            }
        };
        true
    }
    fn set_single_uniform(&self, name: &str, value: UniformValue) -> bool {
        let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
        match name {
            "alpha" => s.single_uniforms[0] = value,
            _ => {
                return false
            }
        };
        true
    }
    fn set_texture(&self, name: &str,
                   value: (Share<HalTexture>, Share<HalSampler>)) -> bool {
        let s = unsafe { &mut *(self as *const Self as usize as *mut Self) };
        match name {
            "texture" => s.textures[0] = value,
            _ => {
                return false
            }
        };
        true
    }
}
pub struct Define {
    values: [Option<&'static str>; 3],
    id: u32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::default::Default for Define {
    #[inline]
    fn default() -> Define {
        Define{values: ::std::default::Default::default(),
               id: ::std::default::Default::default(),}
    }
}
impl Defines for Define {
    fn add(&mut self, value: &'static str) -> Option<&'static str> {
        match value {
            "common" => {
                self.id |= 1 << 0;
                std::mem::replace(&mut self.values[0], Some("common"))
            }
            "clip" => {
                self.id |= 1 << 1;
                std::mem::replace(&mut self.values[1], Some("clip"))
            }
            "texture" => {
                self.id |= 1 << 2;
                std::mem::replace(&mut self.values[2], Some("texture"))
            }
            _ => None,
        }
    }
    fn remove(&mut self, value: &'static str) -> Option<&'static str> {
        match value {
            "common" => {
                self.id &= !(1 << 0);
                std::mem::replace(&mut self.values[0], None)
            }
            "clip" => {
                self.id &= !(1 << 1);
                std::mem::replace(&mut self.values[1], None)
            }
            "texture" => {
                self.id &= !(1 << 2);
                std::mem::replace(&mut self.values[2], None)
            }
            _ => None,
        }
    }
    fn list(&self) -> &[Option<&str>] { &self.values[..] }
    fn id(&self) -> u32 { self.id }
}
fn main() { }