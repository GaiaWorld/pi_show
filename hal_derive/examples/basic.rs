#![feature(proc_macro_hygiene)]
#![recursion_limit="512"]

///一个基本的例子， 演示如何使用 uniform_buffer! 和 program_paramter! 定义UniformBuffer 和 ProgramParamter


extern crate hal_core;
#[macro_use]
extern crate hal_derive;
extern crate share;
#[macro_use]
extern crate lazy_static;
extern crate atom;

use hal_core::*;
use share::Share;
use atom::Atom;

uniform_buffer! {
    #[derive(Debug)]
    #[derive(Clone)]
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

defines! {
    struct Define {
        common: String,
        clip: String,
        texture: String,
    }
}

fn main() { 
    
}