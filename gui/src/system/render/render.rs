/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 在渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::default::Default;
use std::sync::Arc;
use std::collections::HashMap;

use hal_core::*;
use ecs::{SingleCaseImpl, Share, Runner, ModifyEvent, CreateEvent, DeleteEvent, SingleCaseListener};
use atom::Atom;

use render::engine::Engine;
use single::{ RenderObjs, RenderObj, ViewPort};

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
    type ReadData = (
        &'a SingleCaseImpl<RenderObjs<C>>,
        &'a SingleCaseImpl<ViewPort>,
    );
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let (render_objs, view_port) = read;
        debug_println!("RenderSys run");
        if self.dirty == false {
            return;
        }

        self.dirty = false;

        let mut transparent_list = Vec::new();
        let mut opacity_list = Vec::new();
        for item in render_objs.iter() {
            if item.1.visibility == true {
                if item.1.is_opacity == true {
                    opacity_list.push(OpacityOrd(item.1));
                }else {
                    transparent_list.push(TransparentOrd(item.1));
                }
            }
            
        }

        transparent_list.sort();
        opacity_list.sort();

        let gl = &mut engine.gl;
        gl.begin_render(
            &(gl.get_default_render_target().clone() as Arc<dyn AsRef<C::ContextRenderTarget>>), 
            &(view_port.0.clone() as Arc<dyn AsRef<RenderBeginDesc>>));
        
        for obj in opacity_list.into_iter() {
            debug_println!("draw opacity-------------------------");
            render(gl, obj.0);
        }

        for obj in transparent_list.into_iter() {
            debug_println!("draw transparent-------------------------");
            render(gl, obj.0);
        }

        gl.end_render();
    }
}

impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, CreateEvent> for RenderSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _: &CreateEvent, _: Self::ReadData, _: Self::WriteData){
        self.dirty = true;
    }
}

impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, ModifyEvent> for RenderSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _: &ModifyEvent, _: Self::ReadData, _: Self::WriteData){
        self.dirty = true;
    }
}

impl<'a, C: Context + Share> SingleCaseListener<'a, RenderObjs<C>, DeleteEvent> for RenderSys<C>{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _: &DeleteEvent, _: Self::ReadData, _: Self::WriteData){
        self.dirty = true;
    }
}

fn render<C: Context + Share>(gl: &mut C, obj: &RenderObj<C>){
    if obj.geometry.get_vertex_count() == 0 {
        return;
    }
    gl.set_pipeline(&mut (obj.pipeline.pipeline.clone() as Arc<dyn AsRef<Pipeline>>));
    let mut ubos: HashMap<Atom, Arc<dyn AsRef<Uniforms<C>>>> = HashMap::new();
    for (k, v) in obj.ubos.iter() {
        ubos.insert(k.clone(), v.clone() as Arc<dyn AsRef<Uniforms<C>>>);
    }
    debug_println!("draw-------------------------------------{}", obj.context);
    gl.draw(&(obj.geometry.clone() as Arc<dyn AsRef<<C as Context>::ContextGeometry>>), &ubos);
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
    {
        SingleCaseListener<RenderObjs<C>, CreateEvent>
        SingleCaseListener<RenderObjs<C>, ModifyEvent>
        SingleCaseListener<RenderObjs<C>, DeleteEvent>  
    }
}