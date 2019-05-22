/**
 * WebGL 状态设置
 */

use std::sync::{Arc};
use hal_core::*;
use convert::*;
use geometry::{WebGLGeometryImpl};
use render_target::{WebGLRenderTargetImpl};
use webgl_rendering_context::{WebGLRenderingContext};

pub struct State {
    
    gl: Arc<WebGLRenderingContext>, 

    raster: Arc<AsRef<RasterState>>,
    stencil: Arc<AsRef<StencilState>>,
    blend: Arc<AsRef<BlendState>>,
    depth: Arc<AsRef<DepthState>>,
    
    geometry: Option<Arc<AsRef<WebGLGeometryImpl>>>,
    program: (u64, u64),
    target: Arc<AsRef<WebGLRenderTargetImpl>>,
    viewport_rect: (i32, i32, i32, i32), // x, y, w, h
    enable_attrib_indices: Vec<bool>,
}

impl State {

    pub fn new(gl: &Arc<WebGLRenderingContext>, rt: &Arc<AsRef<WebGLRenderTargetImpl>>, max_attributes: u32, max_tex_unit_num: u32) -> State {
        
        gl.enable(WebGLRenderingContext::BLEND);
        gl.enable(WebGLRenderingContext::SCISSOR_TEST);

        let state = State {
            gl: gl.clone(),
            program: (0, 0),
            raster: Arc::new(RasterState::new()),
            stencil: Arc::new(StencilState::new()),
            blend: Arc::new(BlendState::new()),
            depth: Arc::new(DepthState::new()),
            
            geometry: None,
            target: rt.clone(),
            viewport_rect: (0, 0, 0, 0),
            enable_attrib_indices: vec![false; max_attributes as usize],
        };

        Self::apply_all_state(gl, &state);

        state
    }

    /** 
     * 相同，返回true
     * 不相同，更新
     */
    pub fn set_program(&mut self, vs_hash: u64, fs_hash: u64) -> bool {
        if self.program == (vs_hash, fs_hash) {
            return true;
        }

        self.program = (vs_hash, fs_hash);
        return false;
    }

    pub fn set_render_target(&mut self, rt: &Arc<AsRef<WebGLRenderTargetImpl>>) {
        if !Arc::ptr_eq(&self.target, rt) {
            let fbo = &rt.as_ref().as_ref().frame_buffer;
            self.gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, fbo.as_ref());
            
            self.target = rt.clone();
        }
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.viewport_rect != (x, y, width, height) {
            
            self.gl.viewport(x, y, width, height);
            self.gl.scissor(x, y, width, height);

            self.viewport_rect = (x, y, width, height);
        }
    }

    pub fn set_pipeline_state(&mut self, r: &Arc<AsRef<RasterState>>, d: &Arc<AsRef<DepthState>>, s: &Arc<AsRef<StencilState>>, b: &Arc<AsRef<BlendState>>) {
        if !Arc::ptr_eq(&self.raster, r) {
            Self::set_raster_state(&self.gl, Some(self.raster.as_ref().as_ref()), r.as_ref().as_ref());
            self.raster = r.clone();
        }
        if !Arc::ptr_eq(&self.depth, d) {
            Self::set_depth_state(&self.gl, Some(self.depth.as_ref().as_ref()), d.as_ref().as_ref());
            self.depth = d.clone();
        }
        if !Arc::ptr_eq(&self.stencil, s) {
            Self::set_stencil_state(&self.gl, Some(self.stencil.as_ref().as_ref()), s.as_ref().as_ref());
            self.stencil = s.clone();
        }
        if !Arc::ptr_eq(&self.blend, b) {
            Self::set_blend_state(&self.gl, Some(self.blend.as_ref().as_ref()), b.as_ref().as_ref());
            self.blend = b.clone();
        }
    }

    pub fn draw(&mut self, geometry: &Arc<AsRef<WebGLGeometryImpl>>) {
        let need_set_geometry = match &self.geometry {
            None => true,
            Some(g) => !Arc::ptr_eq(g, geometry),
        };

        if !need_set_geometry {

            self.geometry = Some(geometry.clone());
            
            for (n, v) in geometry.as_ref().as_ref().attributes.iter() {
                let index = get_attribute_location(n) as usize;
                
                if !self.enable_attrib_indices[index] {
                    self.gl.enable_vertex_attrib_array(index as u32);
                    self.enable_attrib_indices[index] = true;
                }
                
                self.gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&v.buffer));
                self.gl.vertex_attrib_pointer(index as u32, v.item_count as i32, WebGLRenderingContext::FLOAT, false, 0, 0);
            }
        }
        
        let geometry = geometry.as_ref().as_ref();
        match &geometry.indices {
            None => {
                self.gl.draw_arrays(WebGLRenderingContext::TRIANGLES, 0, geometry.vertex_count as i32);
            }
            Some(indices) => {
                
                self.gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&indices.buffer));
                
                let (data_type, count) = if indices.is_short_type {
                    (WebGLRenderingContext::UNSIGNED_SHORT, indices.size / 2)
                } else {
                    (WebGLRenderingContext::UNSIGNED_INT, indices.size / 4)
                };

                self.gl.draw_elements(WebGLRenderingContext::TRIANGLES, count as i32, data_type, 0);
            }
        }
    }

    fn set_raster_state(gl: &WebGLRenderingContext, old: Option<&RasterState>, curr: &RasterState) {
        match old {
            None => {
                Self::set_cull_mode(gl, curr);    
                Self::set_front_face(gl, curr);
                Self::set_polygon_offset(gl, curr);
            }
            Some(old) => {
                if old.cull_mode != curr.cull_mode {
                    Self::set_cull_mode(gl, curr);
                }
                if old.is_front_face_ccw != curr.is_front_face_ccw {
                    Self::set_front_face(gl, curr);
                }
                if old.polygon_offset != curr.polygon_offset {
                    Self::set_polygon_offset(gl, curr);
                }
            }
        }
    }

    fn set_depth_state(gl: &WebGLRenderingContext, old: Option<&DepthState>, curr: &DepthState) {
        match old {
            None => {
                Self::set_depth_test(gl, curr);
                Self::set_depth_write(gl, curr);
                Self::set_depth_test_func(gl, curr);
            }
            Some(old) => {
                if old.is_depth_test_enable != curr.is_depth_test_enable {
                    Self::set_depth_test(gl, curr);
                }
                if old.is_depth_write_enable != curr.is_depth_write_enable {
                    Self::set_depth_write(gl, curr);
                }
                if old.depth_test_func != curr.depth_test_func {
                    Self::set_depth_test_func(gl, curr);
                }
            }
        }
    }

    fn set_stencil_state(gl: &WebGLRenderingContext, old: Option<&StencilState>, curr: &StencilState) {
        match old {
            None => {
                Self::set_stencil_test(gl, curr);
                Self::set_stencil_test_func(gl, curr);
                Self::set_stencil_op(gl, curr);
            }
            Some(old) => {
                if old.is_stencil_test_enable != curr.is_stencil_test_enable {
                    Self::set_stencil_test(gl, curr);
                }
            
                if old.stencil_test_func != curr.stencil_test_func ||
                    old.stencil_ref != curr.stencil_ref ||
                    old.stencil_mask != curr.stencil_mask {
                    Self::set_stencil_test_func(gl, curr);
                }

                if old.stencil_fail_op != curr.stencil_fail_op ||
                    old.stencil_zfail_op != curr.stencil_zfail_op ||
                    old.stencil_zpass_op != curr.stencil_zpass_op {
                    Self::set_stencil_op(gl, curr);
                }
            }
        }
    }

    fn set_blend_state(gl: &WebGLRenderingContext, old: Option<&BlendState>, curr: &BlendState) {
        match old {
            None => {
                Self::set_blend_equation(gl, curr);
                Self::set_blend_factor(gl, curr);
                Self::set_blend_color(gl, curr);
            }
            Some(old) => {
                if old.rgb_equation != curr.rgb_equation ||
                    old.alpha_equation != curr.alpha_equation {
                    Self::set_blend_equation(gl, curr);
                }

                if old.src_rgb_factor != curr.src_rgb_factor ||
                    old.dst_rgb_factor != curr.dst_rgb_factor ||
                    old.src_alpha_factor != curr.src_alpha_factor ||
                    old.dst_alpha_factor != curr.dst_alpha_factor {
                    Self::set_blend_factor(gl, curr);
                }

                if old.const_rgba != curr.const_rgba {
                    Self::set_blend_color(gl, curr);
                }
            }
        }
    }

    /** 
     * 全状态设置，仅用于创建State时候
     */
    fn apply_all_state(gl: &Arc<WebGLRenderingContext>, state: &State) {
        Self::set_raster_state(gl.as_ref(), None, state.raster.as_ref().as_ref());
        Self::set_depth_state(gl.as_ref(), None, state.depth.as_ref().as_ref());
        Self::set_blend_state(gl.as_ref(), None, state.blend.as_ref().as_ref());
        Self::set_stencil_state(gl.as_ref(), None, state.stencil.as_ref().as_ref());
    }

    fn set_cull_mode(gl: &WebGLRenderingContext, curr: &RasterState) {
        match &curr.cull_mode {
            None => {
                gl.disable(WebGLRenderingContext::CULL_FACE);
            }
            Some(v) => {
                gl.enable(WebGLRenderingContext::CULL_FACE);
                gl.cull_face(get_cull_mode(v));
            }
        }
    }

    fn set_front_face(gl: &WebGLRenderingContext, curr: &RasterState) {
        let face = if curr.is_front_face_ccw { WebGLRenderingContext::CCW } else { WebGLRenderingContext::CW };
        gl.front_face(face);
    }

    fn set_polygon_offset(gl: &WebGLRenderingContext, curr: &RasterState) {
        if curr.polygon_offset != (0.0, 0.0) {
            gl.enable(WebGLRenderingContext::POLYGON_OFFSET_FILL);
            gl.polygon_offset(curr.polygon_offset.0, curr.polygon_offset.1);
        } else {
            gl.disable(WebGLRenderingContext::POLYGON_OFFSET_FILL);
        }
    }

    fn set_depth_test(gl: &WebGLRenderingContext, curr: &DepthState) {
        if curr.is_depth_test_enable {
            gl.enable(WebGLRenderingContext::DEPTH_TEST);
        } else { 
            gl.disable(WebGLRenderingContext::DEPTH_TEST);
        }
    }

    fn set_depth_write(gl: &WebGLRenderingContext, curr: &DepthState) {
        gl.depth_mask(curr.is_depth_write_enable);
    }

    fn set_depth_test_func(gl: &WebGLRenderingContext, curr: &DepthState) {
        gl.depth_func(get_compare_func(&curr.depth_test_func));
    }

    fn set_stencil_test(gl: &WebGLRenderingContext, curr: &StencilState) {
        if curr.is_stencil_test_enable {
            gl.enable(WebGLRenderingContext::STENCIL_TEST);
        } else {
            gl.disable(WebGLRenderingContext::STENCIL_TEST);
        }
    }

    fn set_stencil_test_func(gl: &WebGLRenderingContext, curr: &StencilState) {
        let func = get_compare_func(&curr.stencil_test_func);
        gl.stencil_func(func, curr.stencil_ref, curr.stencil_mask);
    }

    fn set_stencil_op(gl: &WebGLRenderingContext, curr: &StencilState) {
        let fail = get_stencil_op(&curr.stencil_fail_op);
        let zfail = get_stencil_op(&curr.stencil_zfail_op);
        let zpass = get_stencil_op(&curr.stencil_zpass_op);
        gl.stencil_op(fail, zfail,zpass);
    }

    fn set_blend_equation(gl: &WebGLRenderingContext, curr: &BlendState) {
        let rgb = get_blend_func(&curr.rgb_equation);
        let alpha = get_blend_func(&curr.alpha_equation);
        gl.blend_equation_separate(rgb, alpha);
    }

    fn set_blend_factor(gl: &WebGLRenderingContext, curr: &BlendState) {
        let srgb = get_blend_factor(&curr.src_rgb_factor);
        let drgb = get_blend_factor(&curr.dst_rgb_factor);
        let salpha = get_blend_factor(&curr.src_alpha_factor);
        let dalpha = get_blend_factor(&curr.dst_alpha_factor);
        gl.blend_func_separate(srgb, drgb, salpha, dalpha);
    }

    fn set_blend_color(gl: &WebGLRenderingContext, curr: &BlendState) {
        gl.blend_color(curr.const_rgba.0, curr.const_rgba.1, curr.const_rgba.2, curr.const_rgba.3);
    }
}