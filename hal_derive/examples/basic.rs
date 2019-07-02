#![feature(proc_macro_hygiene)]
#![recursion_limit="512"]

///一个基本的例子， 定义组件， 实体， 系统， 已经如何实例化World并运行（TODO）


extern crate hal_core;
#[macro_use]
extern crate hal_derive;
extern crate share;

use hal_core::*;
use share::Share;

uniform_buffer! {
    struct BgColor {
        color: UniformValue,
        depth: UniformValue,
    }
}

uniform_buffer! {
    struct Clip {
        index: UniformValue,
    }
}

program_paramter! {
    struct Color {
        common: BgColor,
        clip: Clip,
        texture: UniformTexture<C>,
    }
}
fn main() { 
    
}