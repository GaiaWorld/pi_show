/**
 * WebGL 状态设置
 */

use share::{Share, ShareWeak};
use hal_core::*;
use convert::*;
use extension::*;
use shader::{Program, ProgramManager};
use texture::{WebGLTextureImpl};
use sampler::{WebGLSamplerImpl};
use geometry::{WebGLGeometryImpl};
use render_target::{WebGLRenderTargetImpl};
use webgl_rendering_context::{WebGLRenderingContext};
// use debug_info::*;

pub struct State {
    clear_color: (f32, f32, f32, f32), 
    clear_depth: f32, 
    clear_stencil: u8,

    gl: Share<WebGLRenderingContext>, 
    
    real_depth_mask: bool, // 实际的深度写入的开关

    pub pipeline: Share<dyn AsRef<Pipeline>>,

    geometry: Option<Share<dyn AsRef<WebGLGeometryImpl>>>,
    target: Share<dyn AsRef<WebGLRenderTargetImpl>>,
    viewport_rect: (i32, i32, i32, i32), // x, y, w, h
    enable_attrib_indices: Vec<bool>,

    tex_caches: TextureCache,
}

struct TextureSlot {
    unit: usize,
    count: usize, // 等于0代表没用过
    texture: ShareWeak<dyn AsRef<WebGLTextureImpl>>,
    sampler: ShareWeak<dyn AsRef<WebGLSamplerImpl>>,
}

impl TextureSlot {
    fn new(unit: usize) -> Self {
        TextureSlot {
            unit: unit,
            count: 0,
            texture: ShareWeak::<WebGLTextureImpl>::new(),
            sampler: ShareWeak::<WebGLSamplerImpl>::new(),
        }
    }
}

struct TextureCache {
    gl: Share<WebGLRenderingContext>,
    tex_use_count: usize,
    values: Vec<TextureSlot>,
}

impl TextureCache {
    fn new(gl: &Share<WebGLRenderingContext>, max_tex_unit_num: usize) -> Self {
        // 第0个纹理通道内部使用
        let mut cache = Vec::with_capacity(max_tex_unit_num - 1);
        for i in 1..max_tex_unit_num {
            cache.push(TextureSlot::new(i));
        }
        TextureCache {
            gl: gl.clone(),
            values: cache,
            tex_use_count: 1,
        }
    }

    pub fn reset(&mut self) {
        let cap = self.values.capacity() + 1;
        let mut cache = Vec::with_capacity(cap - 1);
        for i in 1..cap {
            cache.push(TextureSlot::new(i));
        }
        self.values = cache;
    }

    // 缓存策略：当槽不够的时候，移除最远的槽。
    pub fn use_texture(&mut self, texture: &ShareWeak<dyn AsRef<WebGLTextureImpl>>, sampler: &ShareWeak<dyn AsRef<WebGLSamplerImpl>>) -> u32 {
        let mut min_index = 0;
        let mut min_count = usize::max_value();

        for (i, v) in self.values.iter_mut().enumerate() {
            if v.count < min_count {
                min_count = v.count;
                min_index = i;
            }
            if ShareWeak::ptr_eq(texture, &v.texture) {
                v.count = self.tex_use_count;
                self.tex_use_count += 1;

                if !ShareWeak::ptr_eq(sampler, &v.sampler) {
                    match (texture.upgrade(), sampler.upgrade()) {
                        (Some(texture), Some(sampler)) => {
                            texture.as_ref().as_ref().apply_sampler(sampler.as_ref().as_ref());
                        }
                        _ => {
                            panic!("use_texture failed, texture or sampler not exist");
                        }
                    }
                    v.sampler = sampler.clone();
                }
                return v.unit as u32;
            }
        }
    
        let v = self.values.get_mut(min_index).unwrap();
        v.count = self.tex_use_count;
        self.tex_use_count += 1;
        
        let unit = v.unit;
        v.texture = texture.clone();
        v.sampler = sampler.clone();

        match (texture.upgrade(), sampler.upgrade()) {
            (Some(texture), Some(sampler)) => {
                self.gl.active_texture(WebGLRenderingContext::TEXTURE0 + (unit as u32));
                self.gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture.as_ref().as_ref().handle));
                texture.as_ref().as_ref().apply_sampler(sampler.as_ref().as_ref());
                return unit as u32;
            }
            _ => {
                panic!("use_texture failed, texture or sampler not exist");
            }
        }
    }
}

impl State {

    pub fn new(gl: &Share<WebGLRenderingContext>, rt: &Share<dyn AsRef<WebGLRenderTargetImpl>>, max_attributes: u32, max_tex_unit_num: u32) -> State {
        
        let pipeline = Pipeline {
            vs_hash: 0,
            fs_hash: 0,
            raster_state: Share::new(RasterState::new()),
            stencil_state: Share::new(StencilState::new()),
            blend_state: Share::new(BlendState::new()),
            depth_state: Share::new(DepthState::new()),
        };
        
        let tex_caches = TextureCache::new(gl, max_tex_unit_num as usize);
        let mut state = State {
            gl: gl.clone(),
            real_depth_mask: false,
            clear_color: (1.0, 1.0, 1.0, 1.0), 
            clear_depth: 1.0, 
            clear_stencil: 0,
            pipeline: Share::new(pipeline),
            
            geometry: None,
            target: rt.clone(),
            viewport_rect: (0, 0, 0, 0),
            enable_attrib_indices: vec![false; max_attributes as usize],
            
            tex_caches: tex_caches,
        };  

        Self::apply_all_state(gl, &mut state);

        state
    }

    pub fn use_texture(&mut self, texture: &ShareWeak<dyn AsRef<WebGLTextureImpl>>, sampler: &ShareWeak<dyn AsRef<WebGLSamplerImpl>>) -> u32 {
        self.tex_caches.use_texture(texture, sampler)
    }

    pub fn set_render_target(&mut self, rt: &Share<dyn AsRef<WebGLRenderTargetImpl>>) {
        if !Share::ptr_eq(&self.target, rt) {
            self.set_render_target_impl(rt);
            self.target = rt.clone();
        }
    }

    fn set_render_target_impl(&mut self, rt: &Share<dyn AsRef<WebGLRenderTargetImpl>>) {
        let fbo = &rt.as_ref().as_ref().frame_buffer;
        if fbo.is_none() {
            js! {
                @{self.gl.as_ref()}.bindFramebuffer(@{WebGLRenderingContext::FRAMEBUFFER}, null);
            }
        } else {
            let fbo = fbo.as_ref().unwrap();
            js!{
                @{self.gl.as_ref()}.bindFramebuffer(@{WebGLRenderingContext::FRAMEBUFFER}, @{&fbo}.wrap);
            }
        }
        // debug_println!("State::set_render_target, fbo = {:?}", fbo.as_ref());
    }

    /** 
     * rect: (x, y, width, height)
     */
    pub fn set_viewport(&mut self, rect: &(i32, i32, i32, i32)) {
        if self.viewport_rect != *rect {
            
            self.gl.viewport(rect.0, rect.1, rect.2, rect.3);
            self.gl.scissor(rect.0, rect.1, rect.2, rect.3);
            // debug_println!("State::set_viewport, rect = {:?}", rect);
            self.viewport_rect = *rect;
        }
    }

    pub fn set_clear(&mut self, color: &Option<(f32, f32, f32, f32)>, depth: &Option<f32>, stencil: &Option<u8>) {
        let mut flag = 0;
        if let Some(color) = color {
            flag |= WebGLRenderingContext::COLOR_BUFFER_BIT;

            if *color != self.clear_color {
                // debug_println!("State::set_clear, color = {:?}", color);
                self.gl.clear_color(color.0, color.1, color.2, color.3);
                self.clear_color = *color;
            }
        }

        if let Some(depth) = depth {
            flag |= WebGLRenderingContext::DEPTH_BUFFER_BIT;
            
            // 清除深度的时候，必须打开深度写。
            self.real_depth_mask = true;
            self.gl.depth_mask(true);
            
            if *depth != self.clear_depth {
                // debug_println!("State::set_clear, depth = {:?}", depth);
                self.gl.clear_depth(*depth);
                self.clear_depth = *depth;
            }
        }

        if let Some(stencil) = stencil {
            flag |= WebGLRenderingContext::STENCIL_BUFFER_BIT;

            if *stencil != self.clear_stencil {
                // debug_println!("State::set_clear, stencil = {:?}", stencil);
                self.gl.clear_stencil(*stencil as i32);
                self.clear_stencil = *stencil;
            }
        }

        if flag != 0 {
            // debug_println!("State::set_clear, flag = {:?}", flag);
            self.gl.clear(flag);
        }
    }

    /** 
     * 如果program相同，返回true
     */
    pub fn set_pipeline(&mut self, pipeline: &Share<dyn AsRef<Pipeline>>) -> bool {
        if Share::ptr_eq(&self.pipeline, pipeline) {
            return true;
        }
        
        let curr = pipeline.as_ref().as_ref();
        let old = self.pipeline.as_ref().as_ref();
        
        if !Share::ptr_eq(&old.raster_state, &curr.raster_state) {
            Self::set_raster_state(&self.gl, Some(old.raster_state.as_ref().as_ref()), curr.raster_state.as_ref().as_ref());
        }
        if !Share::ptr_eq(&old.depth_state, &curr.depth_state) {
            Self::set_depth_state(&self.gl, Some(old.depth_state.as_ref().as_ref()), curr.depth_state.as_ref().as_ref(), &mut self.real_depth_mask);
        }
        if !Share::ptr_eq(&old.stencil_state, &curr.stencil_state) {
            Self::set_stencil_state(&self.gl, Some(old.stencil_state.as_ref().as_ref()), curr.stencil_state.as_ref().as_ref());
        }
        if !Share::ptr_eq(&old.blend_state, &curr.blend_state) {
            Self::set_blend_state(&self.gl, Some(old.blend_state.as_ref().as_ref()), curr.blend_state.as_ref().as_ref());
        }

        let r = old.vs_hash == curr.vs_hash && old.fs_hash == curr.fs_hash;
        self.pipeline = pipeline.clone();
        return r;
    }

    pub fn get_current_program<'a>(&self, mgr: &'a mut ProgramManager) -> Result<&'a mut Program, String> {
        let p = self.pipeline.as_ref().as_ref();
        mgr.get_program(p.vs_hash, p.fs_hash)
    }

    pub fn draw(&mut self, geometry: &Share<dyn AsRef<WebGLGeometryImpl>>) {

        let need_set_geometry = match &self.geometry {
            None => true,
            Some(g) => !Share::ptr_eq(g, geometry),
        };

        if need_set_geometry {            
            match &geometry.as_ref().as_ref().vao {
                Some(vao) => {
                    let extension = geometry.as_ref().as_ref().vao_extension.as_ref().unwrap().as_ref();
                    js! {
                        @{&extension}.wrap.bindVertexArrayOES(@{&vao}.wrap);
                    }
                }
                None => {
                    for (n, v) in geometry.as_ref().as_ref().attributes.iter() {
                        let index = get_attribute_location(n) as usize;
                        
                        self.gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&v.buffer));
                        
                        if !self.enable_attrib_indices[index] {
                            self.gl.enable_vertex_attrib_array(index as u32);
                            self.enable_attrib_indices[index] = true;
                        }
                        
                        self.gl.vertex_attrib_pointer(index as u32, v.item_count as i32, WebGLRenderingContext::FLOAT, false, 0, 0);
                        // debug_println!("State::draw, bind_buffer index = {:?}, buffer = {:?}, ", index, &v.buffer);
                    }
                    
                    match &geometry.as_ref().as_ref().indices {
                        None => {}
                        Some(indices) => {
                            self.gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&indices.buffer));
                        }
                    }
                }
            }
            
            self.geometry = Some(geometry.clone());
        }
        
        let geometry = geometry.as_ref().as_ref();
        match &geometry.indices {
            None => {
                self.gl.draw_arrays(WebGLRenderingContext::TRIANGLES, 0, geometry.vertex_count as i32);
            }
            Some(indices) => {
                
                let (data_type, count) = if indices.is_short_type {
                    (WebGLRenderingContext::UNSIGNED_SHORT, indices.size / 2)
                } else {
                    (WebGLRenderingContext::UNSIGNED_INT, indices.size / 4)
                };
                // debug_println!("State::draw, draw_elements index = {:?}, count = {:?}, data_type = {:?}, ", &indices.buffer, count, data_type);
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

    fn set_depth_state(gl: &WebGLRenderingContext, old: Option<&DepthState>, curr: &DepthState, real_depth_write: &mut bool) {
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
                
                if old.is_depth_write_enable == curr.is_depth_write_enable {
                    if curr.is_depth_write_enable != *real_depth_write {
                        *real_depth_write = curr.is_depth_write_enable;
                        Self::set_depth_write(gl, curr);
                    }
                } else if old.is_depth_write_enable != curr.is_depth_write_enable {
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
    pub fn apply_all_state(gl: &Share<WebGLRenderingContext>, state: &mut State) {

        gl.enable(WebGLRenderingContext::BLEND);
        gl.enable(WebGLRenderingContext::SCISSOR_TEST);

        // debug_println!("State::apply_all_state");
		
        gl.clear_color(state.clear_color.0, state.clear_color.1, state.clear_color.2, state.clear_color.3);
        
        gl.clear_depth(state.clear_depth);
        gl.clear_stencil(state.clear_stencil as i32);

        let p = state.pipeline.as_ref().as_ref();
        Self::set_raster_state(gl.as_ref(), None, p.raster_state.as_ref().as_ref());
        Self::set_depth_state(gl.as_ref(), None, p.depth_state.as_ref().as_ref(), &mut state.real_depth_mask);
        Self::set_blend_state(gl.as_ref(), None, p.blend_state.as_ref().as_ref());
        Self::set_stencil_state(gl.as_ref(), None, p.stencil_state.as_ref().as_ref());

        state.geometry = None;
        
        state.set_render_target_impl(&state.target.clone());

        let rect = &state.viewport_rect;
        gl.viewport(rect.0, rect.1, rect.2, rect.3);
        gl.scissor(rect.0, rect.1, rect.2, rect.3);

        let is_vao_extension = gl.get_extension::<OESVertexArrayObject>().map_or(false, |_v| true);
        if !is_vao_extension {
            for (i, v) in state.enable_attrib_indices.iter_mut().enumerate() {
                if *v {
                    gl.disable_vertex_attrib_array(i as u32);
                    *v = false;
                }
            }
        }
        state.tex_caches.reset();
    }

    fn set_cull_mode(gl: &WebGLRenderingContext, curr: &RasterState) {
        // debug_println!("State::set_cull_mode, mode = {:?}", &curr.cull_mode);
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
        // debug_println!("State::set_front_face, is_ccw = {:?}", &curr.is_front_face_ccw);
        let face = if curr.is_front_face_ccw { WebGLRenderingContext::CCW } else { WebGLRenderingContext::CW };
        gl.front_face(face);
    }

    fn set_polygon_offset(gl: &WebGLRenderingContext, curr: &RasterState) {
        // debug_println!("State::set_polygon_offset, value = {:?}", &curr.polygon_offset);
        if curr.polygon_offset != (0.0, 0.0) {
            gl.enable(WebGLRenderingContext::POLYGON_OFFSET_FILL);
            gl.polygon_offset(curr.polygon_offset.0, curr.polygon_offset.1);
        } else {
            gl.disable(WebGLRenderingContext::POLYGON_OFFSET_FILL);
        }
    }

    fn set_depth_test(gl: &WebGLRenderingContext, curr: &DepthState) {
        // debug_println!("State::set_depth_write, enable = {:?}", &curr.is_depth_test_enable);
        if curr.is_depth_test_enable {
            gl.enable(WebGLRenderingContext::DEPTH_TEST);
        } else { 
            gl.disable(WebGLRenderingContext::DEPTH_TEST);
        }
    }

    fn set_depth_write(gl: &WebGLRenderingContext, curr: &DepthState) {
        // debug_println!("State::set_depth_write, enable = {:?}", &curr.is_depth_write_enable);
        gl.depth_mask(curr.is_depth_write_enable);
    }

    fn set_depth_test_func(gl: &WebGLRenderingContext, curr: &DepthState) {
        // debug_println!("State::set_depth_test_func, func = {:?}", &curr.depth_test_func);
        gl.depth_func(get_compare_func(&curr.depth_test_func));
    }

    fn set_stencil_test(gl: &WebGLRenderingContext, curr: &StencilState) {
        // debug_println!("State::set_stencil_test, enable = {:?}", &curr.is_stencil_test_enable);
        if curr.is_stencil_test_enable {
            gl.enable(WebGLRenderingContext::STENCIL_TEST);
        } else {
            gl.disable(WebGLRenderingContext::STENCIL_TEST);
        }
    }

    fn set_stencil_test_func(gl: &WebGLRenderingContext, curr: &StencilState) {
        // debug_println!("State::set_stencil_test_func, func = {:?}, ref = {:?}, mask = {:?}", &curr.stencil_test_func, &curr.stencil_ref, &curr.stencil_mask);
        let func = get_compare_func(&curr.stencil_test_func);
        gl.stencil_func(func, curr.stencil_ref, curr.stencil_mask);
    }

    fn set_stencil_op(gl: &WebGLRenderingContext, curr: &StencilState) {
        // debug_println!("State::set_stencil_op, fail = {:?}, zfail = {:?}, zpass = {:?}", &curr.stencil_fail_op, &curr.stencil_zfail_op, &curr.stencil_zpass_op);
        let fail = get_stencil_op(&curr.stencil_fail_op);
        let zfail = get_stencil_op(&curr.stencil_zfail_op);
        let zpass = get_stencil_op(&curr.stencil_zpass_op);
        gl.stencil_op(fail, zfail,zpass);
    }

    fn set_blend_equation(gl: &WebGLRenderingContext, curr: &BlendState) {
        // debug_println!("State::set_blend_equation, rgb = {:?}, alpha = {:?}", &curr.rgb_equation, &curr.alpha_equation);
        let rgb = get_blend_func(&curr.rgb_equation);
        let alpha = get_blend_func(&curr.alpha_equation);
        gl.blend_equation_separate(rgb, alpha);
    }

    fn set_blend_factor(gl: &WebGLRenderingContext, curr: &BlendState) {
        // debug_println!("State::set_blend_factor, src_rgb = {:?}, dst_rgb = {:?}, src_alpha = {:?}, dst_alpha = {:?}", &curr.src_rgb_factor, &curr.dst_rgb_factor, &curr.src_alpha_factor, &curr.dst_alpha_factor);
        let srgb = get_blend_factor(&curr.src_rgb_factor);
        let drgb = get_blend_factor(&curr.dst_rgb_factor);
        let salpha = get_blend_factor(&curr.src_alpha_factor);
        let dalpha = get_blend_factor(&curr.dst_alpha_factor);
        gl.blend_func_separate(srgb, drgb, salpha, dalpha);
    }

    fn set_blend_color(gl: &WebGLRenderingContext, curr: &BlendState) {
        // debug_println!("State::set_blend_color, rgba = {:?}", &curr.const_rgba);
        gl.blend_color(curr.const_rgba.0, curr.const_rgba.1, curr.const_rgba.2, curr.const_rgba.3);
    }
}