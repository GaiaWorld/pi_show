
///  basic 中代码展开的结果
extern crate hal_core;
extern crate hal_derive;
extern crate share;

use hal_core::*;
use share::Share;

pub struct BgColor {
    values: [UniformValue; 2],
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

pub struct Color<C: Context> {
    uniforms: [Share<dyn UniformBuffer>; 2],
    textures: [Share<UniformTexture<C>>; 1],
}
impl <C: Context> Color<C> {
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
impl <C: Context> ProgramParamter<C> for Color<C> {
    fn get_layout(&self) -> &[&str] { &Self::FIELDS[..] }
    fn get_texture_layout(&self) -> &[&str] { &Self::TEXTURE_FIELDS[..] }
    fn get_values(&self) -> &[Share<dyn UniformBuffer>] { &self.uniforms[..] }
    fn get_textures(&self) -> &[Share<UniformTexture<C>>] {
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
     -> Option<&Share<UniformTexture<C>>> {
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
    fn set_texture(&mut self, name: &str, value: Share<UniformTexture<C>>)
     -> bool {
        match name {
            "texture" => self.textures[0] = value,
            _ => return false,
        };
        true
    }
}
fn main() {

}