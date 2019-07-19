/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 在渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::default::Default;

use hal_core::*;
use ecs::{SingleCaseImpl, Runner, ModifyEvent, CreateEvent, DeleteEvent, SingleCaseListener};

use render::engine::Engine;
use single::{ RenderObjs, RenderObj, RenderBegin};

pub struct RenderSys<C: HalContext + 'static>{
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

impl<C: HalContext + 'static> Default for RenderSys<C> {
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

impl<'a, C: HalContext + 'static> Runner<'a> for RenderSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<RenderObjs>,
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
                (obj1.program.as_ref().unwrap() ).partial_cmp(&( obj2.program.as_ref().unwrap() )).unwrap()
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
                (obj1.program.as_ref().unwrap() ).partial_cmp(&( obj2.program.as_ref().unwrap() )).unwrap()
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
        gl.render_begin(
            &gl.render_get_default_target(), 
            &render_begin.0);
        for id in self.opacity_list.iter() {
            let obj = unsafe { render_objs.get_unchecked(*id) };
            // println!("draw opacity_list-------------------------depth: {}, id: {}", obj.depth,  obj.HalContext);
            render(gl, obj); 
        }

        for id in self.transparent_list.iter() {
            let obj = unsafe { render_objs.get_unchecked(*id) };
            // println!("draw transparent-------------------------depth: {}, id: {}", obj.depth,  obj.HalContext);
            render(gl, obj);
        }

        gl.render_end();

        #[cfg(feature = "performance")]
        js!{
            if (__p) {
                __p.rendersys_run_render = performance.now() - __time;
            }
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
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

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, ModifyEvent> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
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

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, DeleteEvent> for RenderSys<C>{
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
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

fn render<C: HalContext + 'static>(gl: &mut C, obj: &RenderObj){
    let geometry = match &obj.geometry {
        None => return,
        Some(g) => g,
    };
    gl.render_set_program(obj.program.as_ref().unwrap());
    gl.render_set_state(&obj.state.bs, &obj.state.ds, &obj.state.rs, &obj.state.ss);
    println!("geo: {:?}", &geometry.geo);
    gl.render_draw(&geometry.geo, &obj.paramter);
}

struct OpacityOrd<'a>(&'a RenderObj, usize);

impl<'a> PartialOrd for OpacityOrd<'a> {
	fn partial_cmp(&self, other: &OpacityOrd<'a>) -> Option<Ordering> {
        (self.0.program.as_ref().unwrap() ).partial_cmp(&( other.0.program.as_ref().unwrap() ))
	}
}

impl<'a> PartialEq for OpacityOrd<'a>{
	 fn eq(&self, other: &OpacityOrd<'a>) -> bool {
        (self.0.program.as_ref().unwrap() ).eq(&( other.0.program.as_ref().unwrap() ))
    }
}

impl<'a> Eq for OpacityOrd<'a>{}

impl<'a> Ord for OpacityOrd<'a>{
	fn cmp(&self, other: &OpacityOrd<'a>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

struct TransparentOrd<'a>(&'a RenderObj, usize);

impl<'a> PartialOrd for TransparentOrd<'a> {
	fn partial_cmp(&self, other: &TransparentOrd<'a>) -> Option<Ordering> {
		(self.0.depth + other.0.depth_diff).partial_cmp(&(other.0.depth + other.0.depth_diff))
	}
}

impl<'a> PartialEq for TransparentOrd<'a>{
	 fn eq(&self, other: &TransparentOrd<'a>) -> bool {
        (self.0.depth + other.0.depth_diff).eq(&(other.0.depth + other.0.depth_diff))
    }
}

impl<'a> Eq for TransparentOrd<'a>{}

impl<'a> Ord for TransparentOrd<'a>{
	fn cmp(&self, other: &TransparentOrd<'a>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

impl_system!{
    RenderSys<C> where [C: HalContext + 'static],
    true,
    {
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<RenderObjs, ModifyEvent>
        SingleCaseListener<RenderObjs, DeleteEvent>  
    }
}