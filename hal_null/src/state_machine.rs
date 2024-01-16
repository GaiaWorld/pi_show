use deque::slab_deque::SlabDeque;
use ordered_float::OrderedFloat;
use share::Share;
use slab::Slab;
// use stdweb::Object;

use buffer::WebGLBufferImpl;
use convert::*;
use geometry::WebGLGeometryImpl;
use hal_core::*;
use program::WebGLProgramImpl;
use render_target::WebGLRenderTargetImpl;
use texture::WebGLTextureImpl;
use util::*;
use web_sys::{WebGlFramebuffer, WebGlRenderingContext as WebGlRenderingContext1};
use webgl_bind::{WebGlRenderingContext, OESVertexArrayObject};

pub struct StateMachine {
    clear_color: (
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        OrderedFloat<f32>,
        OrderedFloat<f32>,
    ),
    clear_depth: OrderedFloat<f32>,
    clear_stencil: u8,

    real_depth_mask: bool, // 实际的深度写入的开关

    rs: (u32, u32), // rs_slab的index, use_count
    ds: (u32, u32), // rs_slab的index, use_count
    bs: (u32, u32), // bs_slab的index, use_count
    ss: (u32, u32), // ss_slab的index, use_count

    rsdesc: RasterStateDesc,
    bsdesc: BlendStateDesc,
    dsdesc: DepthStateDesc,
    ssdesc: StencilStateDesc,

    pub copy_fbo: WebGlFramebuffer, // 用于纹理拷贝的FBO

    program: (u32, u32),                 // program_slab的index, use_count
    geometry: (u32, u32),                // geometry_slab的index, use_count
    target: (u32, u32),                  // target_slab的index, use_count
	viewport_rect: (i32, i32, i32, i32), // x, y, w, h
	scissor_rect: (i32, i32, i32, i32),
    enable_attrib_indices: Vec<bool>,
    tex_caches: TextureCache,
}

struct TextureCache {
    total_units: usize,
    curr_gl_unit: u32,

    // 槽和unit的关系是 unit = 槽索引 + 1
    // unit = 0 用于更新纹理等，不能用于普通纹理。

    // u32是当前纹理的gl_unit, index, use_count
    values: SlabDeque<(u32, u32, u32)>,
}

impl TextureCache {
    fn new(max_tex_unit_num: usize) -> Self {
        Self {
            curr_gl_unit: 1, // 第0个纹理通道内部使用
            total_units: max_tex_unit_num - 1,
            values: SlabDeque::new(),
        }
	}
	pub fn restore(&mut self, gl: &WebGlRenderingContext) {
		for i in 1..(self.total_units + 1){
			gl.active_texture(WebGlRenderingContext1::TEXTURE0 + i as u32);
			gl.bind_texture(WebGlRenderingContext1::TEXTURE_2D, None);
		}
	}

    pub fn reset(&mut self, texture_slab: &mut Slab<(WebGLTextureImpl, u32)>) {
        for (_, index, use_count) in self.values.iter() {
            match get_mut_ref(texture_slab, *index, *use_count) {
                None => {}
                Some(t) => {
                    t.cache_index = -1;
                    t.curr_sampler = (0, 0);
                }
            }
        }

        self.curr_gl_unit = 1;
        self.values.clear();
    }

    pub fn use_texture(
        &mut self,
        gl: &WebGlRenderingContext,
        texture: &HalItem,
        sampler: &HalItem,
        texture_slab: &mut Slab<(WebGLTextureImpl, u32)>,
        sampler_slab: &mut Slab<(SamplerDesc, u32)>,
    ) -> Result<(u32, bool), String>  {
        if let (Some(t), Some(s)) = (
            get_mut_ref(texture_slab, texture.index, texture.use_count),
            get_ref(sampler_slab, sampler.index, sampler.use_count),
        ) {
            // 命中，放回到队列的头部
            if t.cache_index >= 0 {
                let (unit, index, use_count) = self.values.remove(t.cache_index as usize);
                t.cache_index = self.values.push_front((unit, index, use_count)) as i32;

                if t.curr_sampler.0 != sampler.index || t.curr_sampler.1 != sampler.use_count {
                    t.apply_sampler(gl, s);
                    t.curr_sampler = (sampler.index, sampler.use_count);
                }
                return Ok((unit as u32, false));
            }
        }

        // 缓存已经满了，替换老的
        let unit = if self.values.len() == self.total_units {
            let (u, old_index, old_use_count) = self.values.pop_back().unwrap();
            if let Some(old) = get_mut_ref(texture_slab, old_index, old_use_count) {
                old.cache_index = -1;
            }
            u
        } else {
            // 缓存没满，添加新的
            let u = self.curr_gl_unit;
            self.curr_gl_unit += 1;
            u
        };

        if let (Some(t), Some(s)) = (
            get_mut_ref(texture_slab, texture.index, texture.use_count),
            get_ref(sampler_slab, sampler.index, sampler.use_count),
        ) {
            t.cache_index =
                self.values
                    .push_front((unit, texture.index, texture.use_count)) as i32;
            t.curr_sampler = (sampler.index, sampler.use_count);

            gl.active_texture(WebGlRenderingContext1::TEXTURE0 + (unit as u32));
            gl.bind_texture(WebGlRenderingContext1::TEXTURE_2D, Some(&t.handle));
            t.apply_sampler(gl, s);

            return Ok((unit as u32, true));
        }

		Err(format!("not found texture or sampler, texture: {:?}, sampler: {:?}", texture, sampler))
    }
}

impl StateMachine {
    pub fn new(
        gl: &WebGlRenderingContext,
        is_vao_extension: bool,
        max_attributes: u32,
        max_tex_unit_num: u32,
        texture_slab: &mut Slab<(WebGLTextureImpl, u32)>,
        rt_slab: &Slab<(WebGLRenderTargetImpl, u32)>,
    ) -> StateMachine {
        let tex_caches = TextureCache::new(max_tex_unit_num as usize);
        let mut state = StateMachine {
            real_depth_mask: false,
            clear_color: (
                OrderedFloat(1.0),
                OrderedFloat(1.0),
                OrderedFloat(1.0),
                OrderedFloat(1.0),
            ),
            clear_depth: OrderedFloat(1.0),
            clear_stencil: 0,

            program: (0, 0),
            geometry: (0, 0),
            target: (0, 0),
			viewport_rect: (0, 0, 0, 0),
			scissor_rect: (0, 0, 0, 0),

            rs: (0, 0),
            bs: (0, 0),
            ds: (0, 0),
            ss: (0, 0),

            rsdesc: RasterStateDesc::new(),
            bsdesc: BlendStateDesc::new(),
            dsdesc: DepthStateDesc::new(),
            ssdesc: StencilStateDesc::new(),
            enable_attrib_indices: vec![false; max_attributes as usize],
            tex_caches: tex_caches,

            copy_fbo: gl.create_framebuffer().unwrap(),
        };

        state.apply_all_state(gl, is_vao_extension, texture_slab, rt_slab);

        state
    }

    #[inline(always)]
    pub fn get_curr_program(&self) -> (u32, u32) {
        self.program
    }

    #[inline(always)]
    pub fn get_curr_rt(&self) -> (u32, u32) {
        self.target
    }

    #[inline(always)]
    pub fn use_texture(
        &mut self,
        gl: &WebGlRenderingContext,
        texture: &HalItem,
        sampler: &HalItem,
        texture_slab: &mut Slab<(WebGLTextureImpl, u32)>,
        sampler_slab: &mut Slab<(SamplerDesc, u32)>,
    ) -> Result<(u32, bool), String> {
        self.tex_caches
            .use_texture(gl, texture, sampler, texture_slab, sampler_slab)
    }

    /**
     * 返回是否切换渲染目标
     */
    pub fn set_render_target(
        &mut self,
        gl: &WebGlRenderingContext,
        rt: &HalRenderTarget,
        rtimpl: &WebGLRenderTargetImpl,
    ) -> bool {
        let is_change = self.target.0 != rt.item.index || self.target.1 != rt.item.use_count;
        if is_change {
            self.set_render_target_impl(gl, rtimpl);
            self.target = (rt.item.index, rt.item.use_count);
        }
        is_change
    }

    /**
     * rect: (x, y, width, height)
     */
    pub fn set_viewport(&mut self, gl: &WebGlRenderingContext, rect: &(i32, i32, i32, i32)) {
        if self.viewport_rect != *rect {
            gl.viewport(rect.0, rect.1, rect.2, rect.3);
            self.viewport_rect = *rect;
        }
	}

	/**
     * rect: (x, y, width, height)
     */
	 pub fn set_scissor(&mut self, gl: &WebGlRenderingContext, rect: &(i32, i32, i32, i32)) {
        if self.scissor_rect != *rect {
			gl.scissor(rect.0, rect.1, rect.2, rect.3);
            self.scissor_rect = *rect;
        }
    }

    pub fn set_clear(
        &mut self,
        gl: &WebGlRenderingContext,
        color: &Option<(
            OrderedFloat<f32>,
            OrderedFloat<f32>,
            OrderedFloat<f32>,
            OrderedFloat<f32>,
        )>,
        depth: &Option<OrderedFloat<f32>>,
        stencil: &Option<u8>,
    ) {
        let mut flag = 0;
        if let Some(color) = color {
            flag |= WebGlRenderingContext1::COLOR_BUFFER_BIT;

            if *color != self.clear_color {
                gl.clear_color(*color.0, *color.1, *color.2, *color.3);
                self.clear_color = *color;
            }
        }

        if let Some(depth) = depth {
            flag |= WebGlRenderingContext1::DEPTH_BUFFER_BIT;

            // 清除深度的时候，必须打开深度写。
            self.real_depth_mask = true;
            gl.depth_mask(true);

            if *depth != self.clear_depth {
                gl.clear_depth(**depth);
                self.clear_depth = *depth;
            }
        }

        if let Some(stencil) = stencil {
            flag |= WebGlRenderingContext1::STENCIL_BUFFER_BIT;

            if *stencil != self.clear_stencil {
                gl.clear_stencil(*stencil as i32);
                self.clear_stencil = *stencil;
            }
        }

        if flag != 0 {
            gl.clear(flag);
        }
    }

    pub fn set_state(
        &mut self,
        gl: &WebGlRenderingContext,
        rs: &HalRasterState,
        bs: &HalBlendState,
        ss: &HalStencilState,
        ds: &HalDepthState,
        rsdesc: &RasterStateDesc,
        bsdesc: &BlendStateDesc,
        ssdesc: &StencilStateDesc,
        dsdesc: &DepthStateDesc,
    ) {
        if self.rs.0 != rs.item.index || self.rs.1 != rs.item.use_count {
            Self::set_raster_state(&gl, Some(&self.rsdesc), rsdesc);
            self.rs = (rs.item.index, rs.item.use_count);
            self.rsdesc = rsdesc.clone();
        }
        if self.ds.0 != ds.item.index || self.ds.1 != ds.item.use_count {
            Self::set_depth_state(&gl, Some(&self.dsdesc), dsdesc, &mut self.real_depth_mask);

            self.ds = (ds.item.index, ds.item.use_count);
            self.dsdesc = dsdesc.clone();
        }
        if self.ss.0 != ss.item.index || self.ss.1 != ss.item.use_count {
            Self::set_stencil_state(&gl, Some(&self.ssdesc), ssdesc);

            self.ss = (ss.item.index, ss.item.use_count);
            self.ssdesc = ssdesc.clone();
        }
        if self.bs.0 != bs.item.index || self.bs.1 != bs.item.use_count {
            Self::set_blend_state(&gl, Some(&self.bsdesc), bsdesc);

            self.bs = (bs.item.index, bs.item.use_count);
            self.bsdesc = bsdesc.clone();
        }
    }

    /**
     * 返回是否切换program
     */
    pub fn set_program(
        &mut self,
        gl: &WebGlRenderingContext,
        program: &HalProgram,
        pimpl: &WebGLProgramImpl,
    ) -> bool {
        let is_change =
            self.program.0 != program.item.index || self.program.1 != program.item.use_count;
        if is_change {
            gl.use_program(Some(&pimpl.handle));
            self.program = (program.item.index, program.item.use_count);
        }
        is_change
    }

    /**
     * 返回切换纹理的次数
     */
    pub fn set_uniforms(
        &mut self,
        gl: &WebGlRenderingContext,
        program: &mut WebGLProgramImpl,
        pp: &Share<dyn ProgramParamter>,
        texture_slab: &mut Slab<(WebGLTextureImpl, u32)>,
        sampler_slab: &mut Slab<(SamplerDesc, u32)>,
    ) -> Result<i32, String> {
        let mut tex_change_count = 0;

        let texs = pp.get_textures();
        for loc in program.active_textures.iter_mut() {
            let (t, s) = &texs[loc.slot_uniform];
            let (unit, is_change) = match self.use_texture(gl, &t, &s, texture_slab, sampler_slab) {
				Ok(r) => r,
				Err(e) => return Err(e),
			};
            if is_change {
                tex_change_count += 1;
            }
            loc.set_gl_uniform(gl, unit as i32);
        }

        let singles = pp.get_single_uniforms();
        for loc in program.active_single_uniforms.iter_mut() {
            loc.set_gl_uniform(gl, &singles[loc.slot_uniform])?;
        }

        let pp = pp.get_values();
        for ubo_loc in program.active_uniforms.iter_mut() {
            let should_set_ubo = ubo_loc
                .last
                .as_ref()
                .map_or(true, |v| !Share::ptr_eq(v, &pp[ubo_loc.slot_ubo]));
            if should_set_ubo {
                let uniforms = pp[ubo_loc.slot_ubo].get_values();
                for u_loc in ubo_loc.values.iter_mut() {
                    u_loc.set_gl_uniform(gl, &uniforms[u_loc.slot_uniform])?;
                }
                ubo_loc.last = Some(pp[ubo_loc.slot_ubo].clone());
            }
        }

        Ok(tex_change_count)
    }

	/**
	 * 重置 geometry 無效
	 */
	pub fn reset_geometry(&mut self) {
		self.geometry = (0, 0);
	}

    /**
     * 返回是否切换geometry
     */
    pub fn draw(
        &mut self,
        gl: &WebGlRenderingContext,
        vao_extension: &Option<OESVertexArrayObject>,
        geometry: &HalGeometry,
        gimpl: &WebGLGeometryImpl,
        buffer_slab: &Slab<(WebGLBufferImpl, u32)>,
    ) -> bool {
        let need_set_geometry =
            geometry.item.index != self.geometry.0 || geometry.item.use_count != self.geometry.1;
        if need_set_geometry {
            match &gimpl.vao {
                Some(vao) => {
					let extension = vao_extension.as_ref().unwrap();
					extension.bindVertexArrayOES(Some(vao));
                    // js! {
                    //     @{&extension}.wrap.bindVertexArrayOES(@{&vao}.wrap);
                    // }
                }
                None => {
                    for (i, v) in gimpl.attributes.iter().enumerate() {
                        if let Some(v) = v {
                            if let Some(a) = get_ref(buffer_slab, v.handle.0, v.handle.1) {
                                if !self.enable_attrib_indices[i] {
                                    self.enable_attrib_indices[i] = true;
                                    gl.enable_vertex_attrib_array(i as u32);
                                }
                                gl.bind_buffer(
                                    WebGlRenderingContext1::ARRAY_BUFFER,
                                    Some(&a.handle),
                                );
                                gl.vertex_attrib_pointer_with_i32(
                                    i as u32,
                                    v.item_count as i32,
                                    WebGlRenderingContext1::FLOAT,
                                    false,
                                    0,
                                    0,
                                );
                            }
                        } else {
                            if self.enable_attrib_indices[i] {
                                self.enable_attrib_indices[i] = false;
                                gl.disable_vertex_attrib_array(i as u32);
                            }
                        }
                    }

                    if let Some(indices) = &gimpl.indices {
                        if let Some(i) = get_ref(buffer_slab, indices.handle.0, indices.handle.1) {
                            gl.bind_buffer(
                                WebGlRenderingContext1::ELEMENT_ARRAY_BUFFER,
                                Some(&i.handle),
                            );
                        }
                    }
                }
            }

            self.geometry = (geometry.item.index, geometry.item.use_count);
        }

        match &gimpl.indices {
            None => {
                gl.draw_arrays(
                    WebGlRenderingContext1::TRIANGLES,
                    0,
                    gimpl.vertex_count as i32,
                );
            }
            Some(indices) => {
                gl.draw_elements_with_i32(
                    WebGlRenderingContext1::TRIANGLES,
                    indices.count as i32,
                    WebGlRenderingContext1::UNSIGNED_SHORT,
                    indices.offset as i32,
                );
            }
        }

        need_set_geometry
	}

	pub fn restore_state(&mut self, texture_slab: &mut Slab<(WebGLTextureImpl, u32)>, gl: &WebGlRenderingContext) {
		self.tex_caches.restore(gl);
		self.tex_caches.reset(texture_slab);
	}

    /**
     * 全状态设置，仅用于创建State时候
     */
    pub fn apply_all_state(
        &mut self,
        gl: &WebGlRenderingContext,
        is_vao_extension: bool,
        texture_slab: &mut Slab<(WebGLTextureImpl, u32)>,
        rt_slab: &Slab<(WebGLRenderTargetImpl, u32)>,
    ) {
        gl.enable(WebGlRenderingContext1::BLEND);
        gl.enable(WebGlRenderingContext1::SCISSOR_TEST);

        // debug_println!("State::apply_all_state");

        gl.clear_color(
            *self.clear_color.0,
            *self.clear_color.1,
            *self.clear_color.2,
            *self.clear_color.3,
        );

        gl.clear_depth(*self.clear_depth);
        gl.clear_stencil(self.clear_stencil as i32);

        Self::set_raster_state(gl, None, &self.rsdesc);
        Self::set_depth_state(gl, None, &self.dsdesc, &mut self.real_depth_mask);
        Self::set_blend_state(gl, None, &self.bsdesc);
        Self::set_stencil_state(gl, None, &self.ssdesc);

        self.geometry = (0, 0);
        self.program = (0, 0);

        if let Some(rt) = get_ref(rt_slab, self.target.0, self.target.1) {
            self.set_render_target_impl(gl, &rt);
        }

		let rect = &self.viewport_rect;
		let scissor = &self.scissor_rect;
        gl.viewport(rect.0, rect.1, rect.2, rect.3);
		gl.scissor(scissor.0, scissor.1, scissor.2, scissor.3);

        if !is_vao_extension {
            for (i, v) in self.enable_attrib_indices.iter_mut().enumerate() {
                *v = false;
                gl.disable_vertex_attrib_array(i as u32);
            }
        }
        self.tex_caches.reset(texture_slab);
    }

    pub fn set_render_target_impl(
        &mut self,
        gl: &WebGlRenderingContext,
        rt: &WebGLRenderTargetImpl,
    ) {
		gl.bind_framebuffer(WebGlRenderingContext1::FRAMEBUFFER, rt.handle.as_ref());
        // if rt.handle.is_none() {
        //     js! {
        //         @{gl}.bindFramebuffer(@{WebGlRenderingContext1::FRAMEBUFFER}, null);
        //     }
        // } else {
		// 	let fbo = rt.handle.as_ref();
		// 	gl.bind_framebuffer(WebGlRenderingContext1::FRAMEBUFFER, Some(fbo))
        //     // js! {
        //     //     @{gl}.bindFramebuffer(@{WebGlRenderingContext1::FRAMEBUFFER}, @{&fbo}.wrap);
        //     // }
        // }
    }

    fn set_raster_state(
        gl: &WebGlRenderingContext,
        old: Option<&RasterStateDesc>,
        curr: &RasterStateDesc,
    ) {
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

    fn set_depth_state(
        gl: &WebGlRenderingContext,
        old: Option<&DepthStateDesc>,
        curr: &DepthStateDesc,
        real_depth_write: &mut bool,
    ) {
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

    fn set_stencil_state(
        gl: &WebGlRenderingContext,
        old: Option<&StencilStateDesc>,
        curr: &StencilStateDesc,
    ) {
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

                if old.stencil_test_func != curr.stencil_test_func
                    || old.stencil_ref != curr.stencil_ref
                    || old.stencil_mask != curr.stencil_mask
                {
                    Self::set_stencil_test_func(gl, curr);
                }

                if old.stencil_fail_op != curr.stencil_fail_op
                    || old.stencil_zfail_op != curr.stencil_zfail_op
                    || old.stencil_zpass_op != curr.stencil_zpass_op
                {
                    Self::set_stencil_op(gl, curr);
                }
            }
        }
    }

    fn set_blend_state(
        gl: &WebGlRenderingContext,
        old: Option<&BlendStateDesc>,
        curr: &BlendStateDesc,
    ) {
        match old {
            None => {
                Self::set_blend_equation(gl, curr);
                Self::set_blend_factor(gl, curr);
                Self::set_blend_color(gl, curr);
            }
            Some(old) => {
                if old.rgb_equation != curr.rgb_equation
                    || old.alpha_equation != curr.alpha_equation
                {
                    Self::set_blend_equation(gl, curr);
                }

                if old.src_rgb_factor != curr.src_rgb_factor
                    || old.dst_rgb_factor != curr.dst_rgb_factor
                    || old.src_alpha_factor != curr.src_alpha_factor
                    || old.dst_alpha_factor != curr.dst_alpha_factor
                {
                    Self::set_blend_factor(gl, curr);
                }

                if old.const_rgba != curr.const_rgba {
                    Self::set_blend_color(gl, curr);
                }
            }
        }
    }

    fn set_cull_mode(gl: &WebGlRenderingContext, curr: &RasterStateDesc) {
        // debug_println!("State::set_cull_mode, mode = {:?}", &curr.cull_mode);
        match curr.cull_mode {
            None => {
                gl.disable(WebGlRenderingContext1::CULL_FACE);
            }
            Some(v) => {
                gl.enable(WebGlRenderingContext1::CULL_FACE);
                gl.cull_face(get_cull_mode(v));
            }
        }
    }

    fn set_front_face(gl: &WebGlRenderingContext, curr: &RasterStateDesc) {
        // debug_println!("State::set_front_face, is_ccw = {:?}", &curr.is_front_face_ccw);
        let face = if curr.is_front_face_ccw {
            WebGlRenderingContext1::CCW
        } else {
            WebGlRenderingContext1::CW
        };
        gl.front_face(face);
    }

    fn set_polygon_offset(gl: &WebGlRenderingContext, curr: &RasterStateDesc) {
        // debug_println!("State::set_polygon_offset, value = {:?}", &curr.polygon_offset);
        if curr.polygon_offset != (OrderedFloat(0.0), OrderedFloat(0.0)) {
            gl.enable(WebGlRenderingContext1::POLYGON_OFFSET_FILL);
            gl.polygon_offset(*curr.polygon_offset.0, *curr.polygon_offset.1);
        } else {
            gl.disable(WebGlRenderingContext1::POLYGON_OFFSET_FILL);
        }
    }

    fn set_depth_test(gl: &WebGlRenderingContext, curr: &DepthStateDesc) {
        // debug_println!("State::set_depth_write, enable = {:?}", &curr.is_depth_test_enable);
        if curr.is_depth_test_enable {
            gl.enable(WebGlRenderingContext1::DEPTH_TEST);
        } else {
            gl.disable(WebGlRenderingContext1::DEPTH_TEST);
        }
    }

    fn set_depth_write(gl: &WebGlRenderingContext, curr: &DepthStateDesc) {
        // debug_println!("State::set_depth_write, enable = {:?}", &curr.is_depth_write_enable);
        gl.depth_mask(curr.is_depth_write_enable);
    }

    fn set_depth_test_func(gl: &WebGlRenderingContext, curr: &DepthStateDesc) {
        // debug_println!("State::set_depth_test_func, func = {:?}", &curr.depth_test_func);
        gl.depth_func(get_compare_func(curr.depth_test_func));
    }

    fn set_stencil_test(gl: &WebGlRenderingContext, curr: &StencilStateDesc) {
        // debug_println!("State::set_stencil_test, enable = {:?}", &curr.is_stencil_test_enable);
        if curr.is_stencil_test_enable {
            gl.enable(WebGlRenderingContext1::STENCIL_TEST);
        } else {
            gl.disable(WebGlRenderingContext1::STENCIL_TEST);
        }
    }

    fn set_stencil_test_func(gl: &WebGlRenderingContext, curr: &StencilStateDesc) {
        // debug_println!("State::set_stencil_test_func, func = {:?}, ref = {:?}, mask = {:?}", &curr.stencil_test_func, &curr.stencil_ref, &curr.stencil_mask);
        let func = get_compare_func(curr.stencil_test_func);
        gl.stencil_func(func, curr.stencil_ref, curr.stencil_mask);
    }

    fn set_stencil_op(gl: &WebGlRenderingContext, curr: &StencilStateDesc) {
        // debug_println!("State::set_stencil_op, fail = {:?}, zfail = {:?}, zpass = {:?}", &curr.stencil_fail_op, &curr.stencil_zfail_op, &curr.stencil_zpass_op);
        let fail = get_stencil_op(curr.stencil_fail_op);
        let zfail = get_stencil_op(curr.stencil_zfail_op);
        let zpass = get_stencil_op(curr.stencil_zpass_op);
        gl.stencil_op(fail, zfail, zpass);
    }

    fn set_blend_equation(gl: &WebGlRenderingContext, curr: &BlendStateDesc) {
        // debug_println!("State::set_blend_equation, rgb = {:?}, alpha = {:?}", &curr.rgb_equation, &curr.alpha_equation);
        let rgb = get_blend_func(curr.rgb_equation);
        let alpha = get_blend_func(curr.alpha_equation);
        gl.blend_equation_separate(rgb, alpha);
    }

    fn set_blend_factor(gl: &WebGlRenderingContext, curr: &BlendStateDesc) {
        // debug_println!("State::set_blend_factor, src_rgb = {:?}, dst_rgb = {:?}, src_alpha = {:?}, dst_alpha = {:?}", &curr.src_rgb_factor, &curr.dst_rgb_factor, &curr.src_alpha_factor, &curr.dst_alpha_factor);
        let srgb = get_blend_factor(curr.src_rgb_factor);
        let drgb = get_blend_factor(curr.dst_rgb_factor);
        let salpha = get_blend_factor(curr.src_alpha_factor);
        let dalpha = get_blend_factor(curr.dst_alpha_factor);
        gl.blend_func_separate(srgb, drgb, salpha, dalpha);
    }

    fn set_blend_color(gl: &WebGlRenderingContext, curr: &BlendStateDesc) {
        // debug_println!("State::set_blend_color, rgba = {:?}", &curr.const_rgba);
        gl.blend_color(
            *curr.const_rgba.0,
            *curr.const_rgba.1,
            *curr.const_rgba.2,
            *curr.const_rgba.3,
        );
    }
}