#![feature(prelude_import)]
#![no_std]
#[prelude_import]
extern crate std;
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
impl BgColor {
    pub const
    FIELDS:
    [&'static str; 2]
    =
    ["color", "depth"];
}
impl UniformBuffer for BgColor {
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
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
impl Clip {
    pub const
    FIELDS:
    [&'static str; 1]
    =
    ["index"];
}
impl UniformBuffer for Clip {
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
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
    textures: [(Share<HalTexture>, Share<HalSampler>); 1],
}
impl Color {
    pub const
    FIELDS:
    [&'static str; 2]
    =
    ["common", "clip"];
    pub const
    TEXTURE_FIELDS:
    [&'static str; 1]
    =
    ["texture"];
}
impl ProgramParamter for Color {
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
    fn get_texture_layout(&self) -> &[&str] { &Self::TEXTURE_FIELDS[..] }
    fn get_values(&self) -> &[Share<dyn UniformBuffer>] { &self.uniforms[..] }
    fn get_textures(&self) -> &[(Share<HalTexture>, Share<HalSampler>)] {
        &self.textures[..]
    }
    fn get_value(&mut self, name: &str) -> Option<&Share<dyn UniformBuffer>> {
        match name {
            "common" => Some(&self.uniforms[0]),
            "clip" => Some(&self.uniforms[1]),
            _ => None,
        }
    }
    fn get_texture(&mut self, name: &str)
     -> Option<&(Share<HalTexture>, Share<HalSampler>)> {
        match name { "texture" => Some(&self.textures[0]), _ => None, }
    }
    fn set_value(&mut self, name: &str, value: Share<dyn UniformBuffer>)
     -> bool {
        match name {
            "common" => self.uniforms[0] = value,
            "clip" => self.uniforms[1] = value,
            _ => return false,
        };
        true
    }
    fn set_texture(&mut self, name: &str,
                   value: (Share<HalTexture>, Share<HalSampler>)) -> bool {
        match name {
            "texture" => self.textures[0] = value,
            _ => return false,
        };
        true
    }
}

fn main() {

}