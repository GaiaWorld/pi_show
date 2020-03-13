/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 再渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */
use std::cmp::Ordering;
use std::default::Default;
use std::marker::PhantomData;

use ecs::{CreateEvent, DeleteEvent, ModifyEvent, Runner, SingleCaseImpl, SingleCaseListener};
use hal_core::*;

use render::engine::ShareEngine;
use single::{RenderBegin, RenderObj, RenderObjs};
use DIRTY;

pub struct RenderSys<C: HalContext + 'static> {
    program_dirtys: Vec<usize>,
    transparent_dirty: bool,
    opacity_dirty: bool,
    dirty: bool,
    opacity_list: Vec<usize>,
    transparent_list: Vec<usize>,
    marker: PhantomData<C>,
}

impl<C: HalContext + 'static> Default for RenderSys<C> {
    fn default() -> Self {
        Self {
            program_dirtys: Vec::default(),
            transparent_dirty: false,
            opacity_dirty: false,
            dirty: false,
            opacity_list: Vec::new(),
            transparent_list: Vec::new(),
            // transparent_list: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

impl<'a, C: HalContext + 'static> Runner<'a> for RenderSys<C> {
    type ReadData = &'a SingleCaseImpl<RenderBegin>;
    type WriteData = (
        &'a mut SingleCaseImpl<RenderObjs>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
    );
    fn run(&mut self, render_begin: Self::ReadData, write: Self::WriteData) {
        let (render_objs, engine) = write;

        for id in self.program_dirtys.iter() {
            let render_obj = match render_objs.get_mut(*id) {
                Some(render_obj) => render_obj,
                None => continue,
            };
            let program = engine.create_program(
                render_obj.vs_name.get_hash() as u64,
                render_obj.fs_name.get_hash() as u64,
                &render_obj.vs_name,
                &*render_obj.vs_defines,
                &render_obj.fs_name,
                &*render_obj.fs_defines,
                render_obj.paramter.as_ref(),
            );
            render_obj.program = Some(program);
            render_obj.program_dirty = false;
        }

        self.program_dirtys.clear();

        if self.dirty == false {
            return;
        }
        self.dirty = false;

        #[cfg(feature = "performance")]
        js! {
            __time = performance.now();
        }

        let mut visibility = Vec::new();
        if self.transparent_dirty && self.opacity_dirty {
            self.opacity_list.clear();
            self.transparent_list.clear();
            for item in render_objs.iter() {
                if item.1.visibility == true {
                    if item.1.is_opacity == true {
                        self.opacity_list.push(item.0);
                    } else {
                        self.transparent_list.push(item.0);
                    }
                } else {
                    visibility.push(1);
                }
            }
            self.transparent_list.sort_by(|id1, id2| {
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                obj1.depth.partial_cmp(&obj2.depth).unwrap()
            });
            self.opacity_list.sort_by(|id1, id2| {
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                (obj1.program.as_ref().unwrap().item.index)
                    .partial_cmp(&(obj2.program.as_ref().unwrap().item.index))
                    .unwrap()
            });
        } else if self.transparent_dirty {
            self.transparent_list.clear();
            for item in render_objs.iter() {
                if item.1.visibility == true {
                    if item.1.is_opacity != true {
                        self.transparent_list.push(item.0);
                    }
                } else {
                    visibility.push(1);
                }
            }
            self.transparent_list.sort_by(|id1, id2| {
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
                } else {
                    visibility.push(1);
                }
            }
            self.opacity_list.sort_by(|id1, id2| {
                let obj1 = unsafe { render_objs.get_unchecked(*id1) };
                let obj2 = unsafe { render_objs.get_unchecked(*id2) };
                obj1.program
                    .as_ref()
                    .unwrap()
                    .item
                    .index
                    .partial_cmp(&obj2.program.as_ref().unwrap().item.index)
                    .unwrap()
            });
        }

        #[cfg(feature = "performance")]
        js! {
            if (__p) {
                __p.RenderSys<C>_run_sort = performance.now() - __time;
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
        js! {
            __time = performance.now();
        }
        let target = match &render_begin.1 {
            Some(r) => Some(&**r),
            None => None,
        };
        let gl = &engine.gl;
        gl.render_begin(target, &render_begin.0);
        for id in self.opacity_list.iter() {
            let obj = unsafe { render_objs.get_unchecked(*id) };
            render(gl, obj);
        }
        for id in self.transparent_list.iter() {
            let obj = unsafe { render_objs.get_unchecked(*id) };
            render(gl, obj);
        }

        gl.render_end();

        #[cfg(feature = "performance")]
        js! {
            if (__p) {
                __p.RenderSys<C>_run_render = performance.now() - __time;
            }
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for RenderSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &CreateEvent, _: Self::ReadData, render_objs: Self::WriteData) {
        self.dirty = true;
        unsafe { DIRTY = true };
        let obj = unsafe { render_objs.get_unchecked_mut(event.id) };
        if obj.is_opacity == false {
            self.transparent_dirty = true;
        } else {
            self.opacity_dirty = true;
        }
        self.program_dirtys.push(event.id);
        obj.program_dirty = true;
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, ModifyEvent> for RenderSys<C> {
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs>;
    fn listen(&mut self, event: &ModifyEvent, _: Self::ReadData, render_objs: Self::WriteData) {
        self.dirty = true;
        unsafe { DIRTY = true };
        match event.field {
            "depth" => {
                let obj = unsafe { render_objs.get_unchecked(event.id) };
                if obj.is_opacity == false {
                    self.transparent_dirty = true;
                }
            }
            "program_dirty" => {
                let obj = unsafe { render_objs.get_unchecked_mut(event.id) };
                if obj.is_opacity == true {
                    self.opacity_dirty = true;
                }
                if obj.program_dirty == false {
                    self.program_dirtys.push(event.id);
                    obj.program_dirty = true;
                }
            }
            "is_opacity" => {
                self.opacity_dirty = true;
                self.transparent_dirty = true;
            }
            "visibility" => {
                let obj = unsafe { render_objs.get_unchecked(event.id) };
                if obj.is_opacity {
                    self.opacity_dirty = true;
                } else {
                    self.transparent_dirty = true;
                }
            }
            _ => (),
        }
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, DeleteEvent> for RenderSys<C> {
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = ();
    fn listen(&mut self, event: &DeleteEvent, render_objs: Self::ReadData, _: Self::WriteData) {
        self.dirty = true;
        unsafe { DIRTY = true };
        let obj = unsafe { render_objs.get_unchecked(event.id) };
        if obj.is_opacity == false {
            self.transparent_dirty = true;
        } else {
            self.opacity_dirty = true;
        }
    }
}

fn render<C: HalContext + 'static>(gl: &C, obj: &RenderObj) {
    let geometry = match &obj.geometry {
        None => return,
        Some(g) => g,
    };
    gl.render_set_program(obj.program.as_ref().unwrap());
    gl.render_set_state(&obj.state.bs, &obj.state.ds, &obj.state.rs, &obj.state.ss);
    gl.render_draw(&geometry.geo, &obj.paramter);
}

struct OpacityOrd<'a>(&'a RenderObj, usize);

impl<'a> PartialOrd for OpacityOrd<'a> {
    fn partial_cmp(&self, other: &OpacityOrd<'a>) -> Option<Ordering> {
        self.0
            .program
            .as_ref()
            .unwrap()
            .item
            .index
            .partial_cmp(&other.0.program.as_ref().unwrap().item.index)
    }
}

impl<'a> PartialEq for OpacityOrd<'a> {
    fn eq(&self, other: &OpacityOrd<'a>) -> bool {
        self.0.program.as_ref().unwrap().item.index.eq(&other
            .0
            .program
            .as_ref()
            .unwrap()
            .item
            .index)
    }
}

impl<'a> Eq for OpacityOrd<'a> {}

impl<'a> Ord for OpacityOrd<'a> {
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

impl<'a> PartialEq for TransparentOrd<'a> {
    fn eq(&self, other: &TransparentOrd<'a>) -> bool {
        (self.0.depth + other.0.depth_diff).eq(&(other.0.depth + other.0.depth_diff))
    }
}

impl<'a> Eq for TransparentOrd<'a> {}

impl<'a> Ord for TransparentOrd<'a> {
    fn cmp(&self, other: &TransparentOrd<'a>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

impl_system! {
    RenderSys<C> where [C: HalContext + 'static],
    true,
    {
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<RenderObjs, ModifyEvent>
        SingleCaseListener<RenderObjs, DeleteEvent>
    }
}
