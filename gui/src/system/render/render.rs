/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 在渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::default::Default;
use share::Share;

use fnv::FnvHashMap;

use hal_core::*;
use ecs::{SingleCaseImpl, Share as ShareTrait, Runner, ModifyEvent, CreateEvent, DeleteEvent, SingleCaseListener};
use atom::Atom;

use render::engine::Engine;
use single::{ RenderObjs, RenderObj, RenderBegin};

pub struct RenderSys<C: Context + ShareTrait>{
    transparent_dirty: bool,
    opacity_dirty: bool,
    dirty: bool,
    opacity_list: Vec<usize>,
    transparent_list: Vec<usize>,
    // opacity_list: BTreeMap<f32, usize>, // 不透明列表
    // transparent_list: BTreeMap<u32, usize>, // 透明列表
    // transpare
    // nt_map: 
    mark: PhantomData<C>,
}

impl<C: Context + ShareTrait> Default for RenderSys<C> {
    fn default() -> Self{
        Self{
            transparent_dirty: false,
            opacity_dirty: false,
            dirty: false,
            opacity_list: Vec::new(),
            transparent_list: Vec::new(),
            // transparent_list: BTreeMap::new(),
            mark: PhantomData,
        }
    }
}

impl<'a, C: Context + ShareTrait> Runner<'a> for RenderSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<RenderObjs<C>>,
        &'a SingleCaseImpl<RenderBegin>,
    );
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, read: Self::ReadData, engine: Self::WriteData){
        let (render_objs, render_begin) = read;

        if self.dirty == false {
            return;
        }
        self.dirty = false;
        
        #[cfg(feature = "performance")]
        js!{
            __time = performance.now();
        }
        if self.transparent_dirty && self.opacity_dirty {
            self.opacity_list.clear();
            self.transparent_list.clear();
            for item in render_objs.iter() {
                if item.1.visibility == true {
                    if item.1.is_opacity == true {
                        self.opacity_list.push(item.0);
                    }else {
                        self.transparent_list.push(item.0);
                    }
                }
            }
            self.transparent_list.sort_by(|id1, id2|{
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                obj1.depth.partial_cmp(&obj2.depth).unwrap()
            });
            self.opacity_list.sort_by(|id1, id2|{
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                (obj1.pipeline.pipeline.as_ref() as *const Pipeline as usize).partial_cmp(&(obj2.pipeline.pipeline.as_ref() as *const Pipeline as usize)).unwrap()
            });
        } else if self.transparent_dirty {
            self.transparent_list.clear();
            for item in render_objs.iter() {
                if item.1.visibility == true {
                    if item.1.is_opacity != true {
                        self.transparent_list.push(item.0);
                    }
                }
            }
            self.transparent_list.sort_by(|id1, id2|{
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                obj1.depth.partial_cmp(&obj2.depth).unwrap()
            });
        } else if self.opacity_dirty {
            self.opacity_list.clear();
            for item in render_objs.iter() {
                if item.1.visibility == true {
                    if item.1.is_opacity == true {
                        self.opacity_list.push(item.0);
                    }
                }
            }
            self.opacity_list.sort_by(|id1, id2|{
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                (obj1.pipeline.pipeline.as_ref() as *const Pipeline as usize).partial_cmp(&(obj2.pipeline.pipeline.as_ref() as *const Pipeline as usize)).unwrap()
            });
        }

        #[cfg(feature = "performance")]
        js!{
            if (__p) {
                __p.rendersys_run_sort = performance.now() - __time;
            }
        }
        // let mut transparent_list = Vec::new();
        // let mut opacity_list = Vec::new();
        // for item in render_objs.iter() {
        //     if item.1.visibility == true {
        //         if item.1.is_opacity == true {
        //             opacity_list.push(OpacityOrd(item.1, item.0));
        //         }else {
        //             transparent_list.push(TransparentOrd(item.1, item.0));
        //         }
        //     }
            
        // }

        // transparent_list.sort();
        // opacity_list.sort();

        #[cfg(feature = "performance")]
        js!{
            __time = performance.now();
        }
        let gl = &mut engine.gl;
        gl.begin_render(
            &(gl.get_default_render_target().clone() as Share<dyn AsRef<C::ContextRenderTarget>>), 
            &(render_begin.0.clone() as Share<dyn AsRef<RenderBeginDesc>>));
        
        for id in self.opacity_list.iter() {
            let obj = unsafe { render_objs.get_unchecked(*id) };
            // debug_println!("draw opacity-------------------------depth: {}, id: {}", obj.depth,  obj.context);
            render(gl, obj); 
        }

        for id in self.transparent_list.iter() {
            let obj = unsafe { render_objs.get_unchecked(*id) };
            // debug_println!("draw transparent-------------------------depth: {}, id: {}", obj.depth,  obj.context);
            render(gl, obj);
        }

        gl.end_render();

        #[cfg(feature = "performance")]
        js!{
            if (__p) {
                __p.rendersys_run_render = performance.now() - __time;
            }
        }
    }
}

impl<'a, C: Context + ShareTrait> SingleCaseListener<'a, RenderObjs<C>, CreateEvent> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs<C>>;
    type WriteData = ();
    fn listen(&mut self, event: &CreateEvent, render_objs: Self::ReadData, _: Self::WriteData){
        self.dirty = true;
        let obj = unsafe { render_objs.get_unchecked(event.id) };
        if obj.is_opacity == false {
            self.transparent_dirty = true;
        } else {
            self.opacity_dirty = true;
        }
    }
}

impl<'a, C: Context + ShareTrait> SingleCaseListener<'a, RenderObjs<C>, ModifyEvent> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs<C>>;
    type WriteData = ();
    fn listen(&mut self, event: &ModifyEvent, render_objs: Self::ReadData, _: Self::WriteData){
        self.dirty = true;
        match event.field {
            "depth" => {
                let obj = unsafe { render_objs.get_unchecked(event.id) };
                if obj.is_opacity == false {
                    self.transparent_dirty = true;
                }
            },
            "pipiline" => {
                let obj = unsafe { render_objs.get_unchecked(event.id) };
                if obj.is_opacity == true {
                    self.opacity_dirty = true;
                }
            },
            "is_opacity" => {
                self.opacity_dirty = true;
                self.transparent_dirty = true;
            }
            _ => ()
        }
    }
}

impl<'a, C: Context + ShareTrait> SingleCaseListener<'a, RenderObjs<C>, DeleteEvent> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs<C>>;
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, render_objs: Self::ReadData, _: Self::WriteData){
        self.dirty = true;
        let obj = unsafe { render_objs.get_unchecked(event.id) };
        if obj.is_opacity == false {
            self.transparent_dirty = true;
        } else {
            self.opacity_dirty = true;
        }
    }
}

fn render<C: Context + ShareTrait>(gl: &mut C, obj: &RenderObj<C>){
    let geometry = match &obj.geometry {
        None => return,
        Some(g) => g,
    };
    gl.set_pipeline(&mut (obj.pipeline.pipeline.clone() as Share<dyn AsRef<Pipeline>>));
    // println! ("draw-------------------------------------id: {}, key: {}, ds: {:?}", obj.context, obj.pipeline.key, &obj.pipeline.ds);
    let mut ubos: FnvHashMap<Atom, Share<dyn AsRef<Uniforms<C>>>> = FnvHashMap::default();
    for (k, v) in obj.ubos.iter() {
        ubos.insert(k.clone(), v.clone() as Share<dyn AsRef<Uniforms<C>>>);
    }
    // debug_println!("draw-------------------------------------{}", obj.context);
    gl.draw(&(geometry.value.clone() as Share<dyn AsRef<<C as Context>::ContextGeometry>>), &ubos);
}

struct OpacityOrd<'a, C: Context + ShareTrait>(&'a RenderObj<C>, usize);

impl<'a, C: Context + ShareTrait> PartialOrd for OpacityOrd<'a, C> {
	fn partial_cmp(&self, other: &OpacityOrd<'a, C>) -> Option<Ordering> {
        (self.0.pipeline.pipeline.as_ref() as *const Pipeline as usize).partial_cmp(&(other.0.pipeline.pipeline.as_ref() as *const Pipeline as usize))
	}
}

impl<'a, C: Context + ShareTrait> PartialEq for OpacityOrd<'a, C>{
	 fn eq(&self, other: &OpacityOrd<'a, C>) -> bool {
        (self.0.pipeline.pipeline.as_ref() as *const Pipeline as usize).eq(&(other.0.pipeline.pipeline.as_ref() as *const Pipeline as usize))
    }
}

impl<'a, C: Context + ShareTrait> Eq for OpacityOrd<'a, C>{}

impl<'a, C: Context + ShareTrait> Ord for OpacityOrd<'a, C>{
	fn cmp(&self, other: &OpacityOrd<'a, C>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

struct TransparentOrd<'a, C: Context + ShareTrait>(&'a RenderObj<C>, usize);

impl<'a, C: Context + ShareTrait> PartialOrd for TransparentOrd<'a, C> {
	fn partial_cmp(&self, other: &TransparentOrd<'a, C>) -> Option<Ordering> {
		(self.0.depth + other.0.depth_diff).partial_cmp(&(other.0.depth + other.0.depth_diff))
	}
}

impl<'a, C: Context + ShareTrait> PartialEq for TransparentOrd<'a, C>{
	 fn eq(&self, other: &TransparentOrd<'a, C>) -> bool {
        (self.0.depth + other.0.depth_diff).eq(&(other.0.depth + other.0.depth_diff))
    }
}

impl<'a, C: Context + ShareTrait> Eq for TransparentOrd<'a, C>{}

impl<'a, C: Context + ShareTrait> Ord for TransparentOrd<'a, C>{
	fn cmp(&self, other: &TransparentOrd<'a, C>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

impl_system!{
    RenderSys<C> where [C: Context + ShareTrait],
    true,
    {
        SingleCaseListener<RenderObjs<C>, CreateEvent>
        SingleCaseListener<RenderObjs<C>, ModifyEvent>
        SingleCaseListener<RenderObjs<C>, DeleteEvent>  
    }
}