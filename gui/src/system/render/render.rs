/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 在渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::default::Default;
use std::sync::Arc;
use std::collections::HashMap;

use hal_core::*;
use ecs::{SingleCaseImpl, Share, Runner};
use atom::Atom;

use render::engine::Engine;
use single::{ RenderObjs, RenderObj};

pub struct RenderSys<C: Context + Share>{
    dirty: bool,
    mark: PhantomData<C>,
}

impl<C: Context + Share> Default for RenderSys<C> {
    fn default() -> Self{
        Self{
            dirty: false,
            mark: PhantomData,
        }
    }
}

impl<'a, C: Context + Share> Runner<'a> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs<C>>;
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, render_objs: Self::ReadData, engine: Self::WriteData){
        if self.dirty == false {
            return;
        }

        self.dirty = false;

        let mut transparent_list = Vec::new();
        let mut opacity_list = Vec::new();
        for item in render_objs.iter() {
            if item.1.is_opacity == true {
                opacity_list.push(OpacityOrd(item.1));
            }else {
                transparent_list.push(TransparentOrd(item.1));
            }
        }

        transparent_list.sort();
        opacity_list.sort();

        for obj in opacity_list.into_iter() {
            render(&mut engine.gl, obj.0);
        }

        for obj in transparent_list.into_iter() {
            render(&mut engine.gl, obj.0);
        }
    }
}

fn render<C: Context + Share>(gl: &mut C, obj: &RenderObj<C>){
    gl.set_pipeline(&mut (obj.pipeline.pipeline.clone() as Arc<AsRef<Pipeline>>));
    let mut ubos: HashMap<Atom, Arc<AsRef<Uniforms<C>>>> = HashMap::new();
    for (k, v) in obj.ubos.iter() {
        ubos.insert(k.clone(), v.clone() as Arc<AsRef<Uniforms<C>>>);
    }
    gl.draw(&(obj.geometry.clone() as Arc<AsRef<<C as Context>::ContextGeometry>>), &ubos);
}

struct OpacityOrd<'a, C: Context + Share>(&'a RenderObj<C>);

impl<'a, C: Context + Share> PartialOrd for OpacityOrd<'a, C> {
	fn partial_cmp(&self, other: &OpacityOrd<'a, C>) -> Option<Ordering> {
        (self.0.pipeline.pipeline.as_ref() as *const Pipeline as usize).partial_cmp(&(other.0.pipeline.pipeline.as_ref() as *const Pipeline as usize))
	}
}

impl<'a, C: Context + Share> PartialEq for OpacityOrd<'a, C>{
	 fn eq(&self, other: &OpacityOrd<'a, C>) -> bool {
        (self.0.pipeline.pipeline.as_ref() as *const Pipeline as usize).eq(&(other.0.pipeline.pipeline.as_ref() as *const Pipeline as usize))
    }
}

impl<'a, C: Context + Share> Eq for OpacityOrd<'a, C>{}

impl<'a, C: Context + Share> Ord for OpacityOrd<'a, C>{
	fn cmp(&self, other: &OpacityOrd<'a, C>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

struct TransparentOrd<'a, C: Context + Share>(&'a RenderObj<C>);

impl<'a, C: Context + Share> PartialOrd for TransparentOrd<'a, C> {
	fn partial_cmp(&self, other: &TransparentOrd<'a, C>) -> Option<Ordering> {
		self.0.depth.partial_cmp(&other.0.depth)
	}
}

impl<'a, C: Context + Share> PartialEq for TransparentOrd<'a, C>{
	 fn eq(&self, other: &TransparentOrd<'a, C>) -> bool {
        self.0.depth.eq(&other.0.depth)
    }
}

impl<'a, C: Context + Share> Eq for TransparentOrd<'a, C>{}

impl<'a, C: Context + Share> Ord for TransparentOrd<'a, C>{
	fn cmp(&self, other: &TransparentOrd<'a, C>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

impl_system!{
    RenderSys<C> where [C: Context + Share],
    true,
    {}
}