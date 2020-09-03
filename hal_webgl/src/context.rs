use atom::Atom;
use share::Share;
use slab::Slab;
use std::convert::TryFrom;
// use stdweb::unstable::TryInto;
// use stdweb::Object;
use web_sys::{WebGlRenderingContext as WebGlRenderingContext1, WebGlTexture, WebGlFramebuffer};
use wasm_bindgen::prelude::{JsValue};
use js_sys::Object;

use webgl_bind::{WebGlRenderingContext, OESVertexArrayObject};

use hal_core::*;

use buffer::WebGLBufferImpl;
use extension::*;
use extension::OESVertexArrayObject as OESVertexArrayObject1;
use geometry::WebGLGeometryImpl;
use program::WebGLProgramImpl;
use render_target::{WebGLRenderBufferImpl, WebGLRenderTargetImpl};
use shader_cache::ShaderCache;
use state_machine::StateMachine;
use texture::WebGLTextureImpl;
use util::*;

pub struct WebglHalContextImpl {
    // 用于给每个context
    pub stat: RenderStat,

    // 具体实现
    pub gl: WebGlRenderingContext,
    pub caps: Capabilities,
    pub extensions: Extensions,
    pub vao_extension: Option<OESVertexArrayObject>,
    pub shader_cache: ShaderCache,
    pub state_machine: StateMachine,

    // u32代表该槽分配的次数
    pub buffer_slab: Slab<(WebGLBufferImpl, u32)>,
    pub geometry_slab: Slab<(WebGLGeometryImpl, u32)>,
    pub texture_slab: Slab<(WebGLTextureImpl, u32)>,
    pub sampler_slab: Slab<(SamplerDesc, u32)>,
    pub rt_slab: Slab<(WebGLRenderTargetImpl, u32)>,
    pub rb_slab: Slab<(WebGLRenderBufferImpl, u32)>,
    pub bs_slab: Slab<(BlendStateDesc, u32)>,
    pub ds_slab: Slab<(DepthStateDesc, u32)>,
    pub rs_slab: Slab<(RasterStateDesc, u32)>,
    pub ss_slab: Slab<(StencilStateDesc, u32)>,
    pub program_slab: Slab<(WebGLProgramImpl, u32)>,
}

pub struct Extensions {
    // pub s3tc: Option<CompressedTexS3tc>,
    pub pvrtc: Option<CompressedTexPvrtcExtension>,
    pub etc1: Option<CompressedTexEtc1Extension>,
    pub etc2: Option<CompressedTexEtc2Extension>,
    pub astc: Option<CompressedTexAstcExtension>,
}

pub struct WebglHalContext(Share<WebglHalContextImpl>, HalRenderTarget);

impl HalContext for WebglHalContext {
    // ==================== HalBuffer

    fn buffer_create(
        &self,
        btype: BufferType,
        count: usize,
        data: Option<BufferData>,
        is_updatable: bool,
    ) -> Result<HalBuffer, String> {
        WebGLBufferImpl::new(&self.0.gl, btype, count, data, is_updatable).map(|buffer| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.buffer_slab, buffer);
            context.stat.buffer_count += 1;

            let context_impl = self.0.clone();
            HalBuffer {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.buffer_destroy(index, use_count);
                }),
            }
        })
    }

    fn buffer_update(&self, buffer: &HalBuffer, offset: usize, data: BufferData) {
        let context = convert_to_mut(self.0.as_ref());
        if let Some(buffer) = get_mut_ref(
            &mut context.buffer_slab,
            buffer.item.index,
            buffer.item.use_count,
        ) {
            buffer.update(&self.0.gl, offset, data);
        }
    }

    // ==================== HalGeometry

    fn geometry_create(&self) -> Result<HalGeometry, String> {
        WebGLGeometryImpl::new(&self.0.vao_extension).map(|geometry| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.geometry_slab, geometry);
            context.stat.geometry_count += 1;

            let context_impl = self.0.clone();
            HalGeometry {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.geometry_destroy(index, use_count);
                }),
            }
        })
    }

    fn geometry_get_vertex_count(&self, geometry: &HalGeometry) -> u32 {
        get_ref(
            &self.0.geometry_slab,
            geometry.item.index,
            geometry.item.use_count,
        )
        .map_or(0, |geometry| geometry.get_vertex_count())
    }

    fn geometry_set_vertex_count(&self, geometry: &HalGeometry, count: u32) {
        let slab = convert_to_mut(&self.0.geometry_slab);
        if let Some(geometry) = get_mut_ref(slab, geometry.item.index, geometry.item.use_count) {
            geometry.set_vertex_count(count);
        }
    }

    fn geometry_set_attribute(
        &self,
        geometry: &HalGeometry,
        name: &AttributeName,
        buffer: &HalBuffer,
        item_count: usize,
    ) -> Result<(), String> {
        let slab = convert_to_mut(&self.0.geometry_slab);
        let g = get_mut_ref(slab, geometry.item.index, geometry.item.use_count)
            .ok_or("geometry isn't found")?;

        let slab = convert_to_mut(&self.0.buffer_slab);
        let b = get_mut_ref(slab, buffer.item.index, buffer.item.use_count)
            .ok_or("buffer isn't found")?;

        g.set_attribute(
            &self.0.gl,
            &self.0.vao_extension,
            name,
            b,
            buffer,
            item_count,
        )
    }

    fn geometry_set_attribute_with_offset(
        &self,
        geometry: &HalGeometry,
        name: &AttributeName,
        buffer: &HalBuffer,
        item_count: usize,
        offset: usize,
        count: usize,
        stride: usize,
    ) -> Result<(), String> {
        let slab = convert_to_mut(&self.0.geometry_slab);
        let g = get_mut_ref(slab, geometry.item.index, geometry.item.use_count)
            .ok_or("geometry isn't found")?;

        let slab = convert_to_mut(&self.0.buffer_slab);
        let b = get_mut_ref(slab, buffer.item.index, buffer.item.use_count)
            .ok_or("buffer isn't found")?;

        g.set_attribute_with_offset(
            &self.0.gl,
            &self.0.vao_extension,
            name,
            b,
            buffer,
            item_count,
            offset,
            count,
            stride,
        )
    }

    fn geometry_remove_attribute(&self, geometry: &HalGeometry, name: &AttributeName) {
        let slab = convert_to_mut(&self.0.geometry_slab);
        if let Some(g) = get_mut_ref(slab, geometry.item.index, geometry.item.use_count) {
            g.remove_attribute(&self.0.gl, &self.0.vao_extension, name);
        }
    }

    fn geometry_set_indices_short(
        &self,
        geometry: &HalGeometry,
        buffer: &HalBuffer,
    ) -> Result<(), String> {
        let slab = convert_to_mut(&self.0.geometry_slab);
        let g = get_mut_ref(slab, geometry.item.index, geometry.item.use_count)
            .ok_or("geometry isn't found")?;

        let slab = convert_to_mut(&self.0.buffer_slab);
        let b = get_mut_ref(slab, buffer.item.index, buffer.item.use_count)
            .ok_or("buffer isn't found")?;

        g.set_indices_short(&self.0.gl, &self.0.vao_extension, b, buffer)
    }

    fn geometry_set_indices_short_with_offset(
        &self,
        geometry: &HalGeometry,
        buffer: &HalBuffer,
        offset: usize,
        count: usize,
    ) -> Result<(), String> {
        let slab = convert_to_mut(&self.0.geometry_slab);
        let g = get_mut_ref(slab, geometry.item.index, geometry.item.use_count)
            .ok_or("geometry isn't found")?;

        let slab = convert_to_mut(&self.0.buffer_slab);
        let b = get_mut_ref(slab, buffer.item.index, buffer.item.use_count)
            .ok_or("buffer isn't found")?;

        g.set_indices_short_with_offset(&self.0.gl, &self.0.vao_extension, b, buffer, offset, count)
    }

    fn geometry_remove_indices(&self, geometry: &HalGeometry) {
        let slab = convert_to_mut(&self.0.geometry_slab);
        if let Some(g) = get_mut_ref(slab, geometry.item.index, geometry.item.use_count) {
            g.remove_indices(&self.0.gl, &self.0.vao_extension);
        }
    }

    // ==================== HalProgram

    fn program_create_with_vs_fs(
        &self,
        vs_id: u64,
        fs_id: u64,
        vs_name: &str,
        vs_defines: &[Option<&str>],
        fs_name: &str,
        fs_defines: &[Option<&str>],
        uniform_layout: &UniformLayout,
    ) -> Result<HalProgram, String> {
        let vs_name = Atom::from(vs_name);
        let fs_name = Atom::from(fs_name);

        let shader_cache = convert_to_mut(&self.0.shader_cache);
        WebGLProgramImpl::new_with_vs_fs(
            &self.0.gl,
            &self.0.caps,
            shader_cache,
            vs_id,
            fs_id,
            &vs_name,
            vs_defines,
            &fs_name,
            fs_defines,
            uniform_layout,
        )
        .map(|program| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.program_slab, program);
            context.stat.program_count += 1;

            let context_impl = self.0.clone();
            HalProgram {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.program_destroy(index, use_count);
                }),
            }
        })
    }

    // ==================== HalRenderTarget

    fn rt_create(
        &self,
        tex: Option<&HalTexture>,
        w: u32,
        h: u32,
        pformat: PixelFormat,
        dformat: DataFormat,
        has_depth: bool,
    ) -> Result<HalRenderTarget, String> {
        let texture_wrap = match tex {
            None => self.texture_create_2d(0, w, h, pformat, dformat, false, None)?,
            Some(r) => HalTexture {
                item: HalItem {
                    index: r.item.index,
                    use_count: r.item.use_count,
                },
                destroy_func: Share::new(move |_index: u32, _use_count: u32| {}),
            },
        };

        let rb_wrap = if has_depth {
            let rbimpl = self.rb_create(w, h, PixelFormat::DEPTH16);
            if let Err(e) = &rbimpl {
                return Err(e.clone());
            }
            Some(rbimpl.unwrap())
        } else {
            None
        };

        let texture = get_ref(
            &self.0.texture_slab,
            texture_wrap.item.index,
            texture_wrap.item.use_count,
        )
        .unwrap();

        let rb = if has_depth {
            let r = rb_wrap.as_ref().unwrap();
            Some(get_ref(&self.0.rb_slab, r.item.index, r.item.use_count).unwrap())
        } else {
            None
        };

        WebGLRenderTargetImpl::new(
            &self.0.gl,
            w,
            h,
            texture,
            rb,
            texture_wrap,
            rb_wrap,
            tex.is_none(),
        )
        .map(|rt| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.rt_slab, rt);
            context.stat.rt_count += 1;

            let context_impl = self.0.clone();
            HalRenderTarget {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.rt_destroy(index, use_count);
                }),
            }
        })
    }

    fn rt_get_size(&self, rt: &HalRenderTarget) -> (u32, u32) {
        get_ref(&self.0.rt_slab, rt.item.index, rt.item.use_count)
            .map(|rt| rt.get_size())
            .unwrap()
    }

    fn rt_get_color_texture(&self, rt: &HalRenderTarget, _index: u32) -> Option<&HalTexture> {
        get_ref(&self.0.rt_slab, rt.item.index, rt.item.use_count)
            .and_then(|rt| rt.get_color_texture())
    }

    // ==================== HalRenderBuffer

    fn rb_create(&self, w: u32, h: u32, pformat: PixelFormat) -> Result<HalRenderBuffer, String> {
        WebGLRenderBufferImpl::new(&self.0.gl, w, h, pformat).map(|rb| {
            let slab = convert_to_mut(&self.0.rb_slab);
            let (index, use_count) = create_new_slot(slab, rb);

            let context_impl = self.0.clone();
            HalRenderBuffer {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.rb_destroy(index, use_count);
                }),
            }
        })
    }

    fn rb_get_size(&self, rb: &HalRenderBuffer) -> (u32, u32) {
        get_ref(&self.0.rb_slab, rb.item.index, rb.item.use_count)
            .map(|rb| rb.get_size())
            .unwrap()
    }

    // ==================== HalTexture

    fn texture_create_2d(
        &self,
        mipmap_level: u32,
        width: u32,
        height: u32,
        pformat: PixelFormat,
        dformat: DataFormat,
        is_gen_mipmap: bool,
        data: Option<TextureData>,
    ) -> Result<HalTexture, String> {
        WebGLTextureImpl::new_2d(
            &self.0.gl,
            mipmap_level,
            width,
            height,
            pformat,
            dformat,
            is_gen_mipmap,
            data,
            None,
        )
        .map(|texture| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.texture_slab, texture);
            context.stat.texture_count += 1;

            let context_impl = self.0.clone();
            HalTexture {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.texture_destroy(index, use_count);
                }),
            }
        })
    }

    fn compressed_texture_create_2d(
        &self,
        mipmap_level: u32,
        width: u32,
        height: u32,
        internalformat: CompressedTexFormat,
        is_gen_mipmap: bool,
        data: Option<TextureData>,
    ) -> Result<HalTexture, String> {
        let context_impl = self.0.clone();
        // let webgl_internalformat = get_compressed_tex_const(internalformat, &context_impl);
        let webgl_internalformat = internalformat.0;
        if webgl_internalformat == -1 {
            return Err(format!("不支持压缩纹理格式：{:?}", internalformat));
        }
        WebGLTextureImpl::new_compressed_2d(
            &self.0.gl,
            mipmap_level,
            width,
            height,
            webgl_internalformat as u32,
            is_gen_mipmap,
            data,
            None,
        )
        .map(|texture| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.texture_slab, texture);
            context.stat.texture_count += 1;

            HalTexture {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.texture_destroy(index, use_count);
                }),
            }
        })
    }

    fn texture_get_size(&self, texture: &HalTexture) -> (u32, u32) {
        get_ref(
            &self.0.texture_slab,
            texture.item.index,
            texture.item.use_count,
        )
        .map(|tex| tex.get_size())
        .unwrap()
    }

    fn texture_get_render_format(&self, texture: &HalTexture) -> PixelFormat {
        get_ref(
            &self.0.texture_slab,
            texture.item.index,
            texture.item.use_count,
        )
        .map(|tex| tex.get_render_format())
        .unwrap()
    }

    fn texture_is_gen_mipmap(&self, texture: &HalTexture) -> bool {
        get_ref(
            &self.0.texture_slab,
            texture.item.index,
            texture.item.use_count,
        )
        .map_or(false, |tex| tex.is_gen_mipmap())
    }

    fn texture_resize(&self, texture: &HalTexture, mipmap_level: u32, width: u32, height: u32) {
        let slab = convert_to_mut(&self.0.texture_slab);
        if let Some(t) = get_mut_ref(slab, texture.item.index, texture.item.use_count) {
            t.resize(&self.0.gl, mipmap_level, width, height);
        }
    }

    fn texture_update(&self, texture: &HalTexture, mipmap_level: u32, data: &TextureData) {
        let slab = convert_to_mut(&self.0.texture_slab);
        if let Some(t) = get_mut_ref(slab, texture.item.index, texture.item.use_count) {
            t.update(&self.0.gl, mipmap_level, Some(data), None);
        }
    }

    fn texture_copy(
        &self,
        dst: &HalTexture,
        src: &HalTexture,
        src_mipmap_level: u32,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        height: u32,
    ) {
        if let Some(dst) = get_ref(&self.0.texture_slab, dst.item.index, dst.item.use_count) {
            self.texture_copy_impl(
                dst,
                src,
                src_mipmap_level,
                src_x,
                src_y,
                dst_x,
                dst_y,
                width,
                height,
            );
        }
    }

    fn texture_extend(&self, texture: &HalTexture, width: u32, height: u32) -> bool {
        let slab = convert_to_mut(&self.0.texture_slab);
        if let Some(old_tex) = get_mut_ref(slab, texture.item.index, texture.item.use_count) {
            match WebGLTextureImpl::new_2d(
                &self.0.gl,
                0,
                width,
                height,
                old_tex.pixel_format,
                old_tex.data_format,
                old_tex.is_gen_mipmap,
                None,
                None,
            ) {
                Ok(dst) => {
                    self.texture_copy_impl(
                        &dst,
                        texture,
                        0,
                        0,
                        0,
                        0,
                        0,
                        old_tex.width,
                        old_tex.height,
                    );
                    old_tex.delete(&self.0.gl);
                    old_tex.width = width;
                    old_tex.height = height;

                    old_tex.handle = dst.handle;

                    old_tex.cache_index = -1;
                    old_tex.curr_sampler = (0, 0);

                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn texture_pixel_storei(&self, texture: &HalTexture, value: PixelStore) {
        get_ref(
            &self.0.texture_slab,
            texture.item.index,
            texture.item.use_count,
        )
        .map(|tex| tex.pixel_storei(&self.0.gl, value));
    }

    // ==================== HalSampler

    fn sampler_create(&self, desc: SamplerDesc) -> Result<HalSampler, String> {
        let slab = convert_to_mut(&self.0.sampler_slab);
        let (index, use_count) = create_new_slot(slab, desc);

        let context_impl = self.0.clone();
        Ok(HalSampler {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_impl.sampler_destroy(index, use_count);
            }),
        })
    }

    fn sampler_get_desc(&self, sampler: &HalSampler) -> &SamplerDesc {
        get_ref(
            &self.0.sampler_slab,
            sampler.item.index,
            sampler.item.use_count,
        )
        .unwrap()
    }

    // ==================== HalRasterState

    fn rs_create(&self, desc: RasterStateDesc) -> Result<HalRasterState, String> {
        let slab = convert_to_mut(&self.0.rs_slab);
        let (index, use_count) = create_new_slot(slab, desc);

        let context_impl = self.0.clone();
        Ok(HalRasterState {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_impl.rs_destroy(index, use_count);
            }),
        })
    }

    fn rs_get_desc(&self, state: &HalRasterState) -> &RasterStateDesc {
        get_ref(&self.0.rs_slab, state.item.index, state.item.use_count).unwrap()
    }

    // ==================== HalDepthState

    fn ds_create(&self, desc: DepthStateDesc) -> Result<HalDepthState, String> {
        let slab = convert_to_mut(&self.0.ds_slab);
        let (index, use_count) = create_new_slot(slab, desc);

        let context_impl = self.0.clone();
        Ok(HalDepthState {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_impl.ds_destroy(index, use_count);
            }),
        })
    }

    fn ds_get_desc(&self, state: &HalDepthState) -> &DepthStateDesc {
        get_ref(&self.0.ds_slab, state.item.index, state.item.use_count).unwrap()
    }

    // ==================== HalStencilState

    fn ss_create(&self, desc: StencilStateDesc) -> Result<HalStencilState, String> {
        let slab = convert_to_mut(&self.0.ss_slab);
        let (index, use_count) = create_new_slot(slab, desc);

        let context_impl = self.0.clone();
        Ok(HalStencilState {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_impl.ss_destroy(index, use_count);
            }),
        })
    }

    fn ss_get_desc(&self, state: &HalStencilState) -> &StencilStateDesc {
        get_ref(&self.0.ss_slab, state.item.index, state.item.use_count).unwrap()
    }

    // ==================== HalBlendState

    fn bs_create(&self, desc: BlendStateDesc) -> Result<HalBlendState, String> {
        let slab = convert_to_mut(&self.0.bs_slab);
        let (index, use_count) = create_new_slot(slab, desc);

        let context_impl = self.0.clone();
        Ok(HalBlendState {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_impl.bs_destroy(index, use_count);
            }),
        })
    }

    fn bs_get_desc(&self, state: &HalBlendState) -> &BlendStateDesc {
        get_ref(&self.0.bs_slab, state.item.index, state.item.use_count).unwrap()
    }

    // ==================== 上下文相关

    fn render_get_caps(&self) -> &Capabilities {
        &self.0.caps
    }

    fn render_set_shader_code(&self, name: &str, code: &str) {
        let cache = convert_to_mut(&self.0.shader_cache);
        cache.set_shader_code(name, code)
    }

    fn restore_state(&self) {
        let context = convert_to_mut(self.0.as_ref());
        context.state_machine.apply_all_state(
            &context.gl,
            self.0.vao_extension.is_some(),
            &mut context.texture_slab,
            &mut context.rt_slab,
        );
    }

    fn restore_active_uniform(&self, program: &HalProgram, index: usize) {
        let context = convert_to_mut(self.0.as_ref());
        let p = get_mut_ref(
            &mut context.program_slab,
            program.item.index,
            program.item.use_count,
        )
        .expect("param not found");
        p.restore_active_uniform(index);
    }

    fn render_begin(&self, render_target: Option<&HalRenderTarget>, data: &RenderBeginDesc) {
        self.restore_state();

        let context = convert_to_mut(self.0.as_ref());

        #[cfg(feature = "frame_stat")]
        context.stat.reset_frame();

        let render_target = if render_target.is_none() {
            &self.1
        } else {
            render_target.unwrap()
        };

        let rt = get_ref(
            &context.rt_slab,
            render_target.item.index,
            render_target.item.use_count,
        )
        .expect("rt param not found");
        if context
            .state_machine
            .set_render_target(&context.gl, render_target, rt)
        {
            #[cfg(feature = "frame_stat")]
            context.stat.add_rt_change();
        }
        context
            .state_machine
            .set_viewport(&context.gl, &data.viewport);
        context.state_machine.set_clear(
            &context.gl,
            &data.clear_color,
            &data.clear_depth,
            &data.clear_stencil,
        );
    }

    fn render_end(&self) {
		let context = convert_to_mut(self.0.as_ref());
        if let Some(vao_extension) = &self.0.vao_extension {
			// let extension = vao_extension.as_ref();
			vao_extension.bindVertexArrayOES(None);
            // js! {
			// 	@{&extension}.wrap.bindVertexArrayOES(null);
			// }
		}
		context.state_machine.restore_state(&context.gl);
    }

    fn render_get_stat(&self) -> &RenderStat {
        let mut r = 0;
        r += self.0.buffer_slab.mem_size();
        r += self.0.geometry_slab.mem_size();
        r += self.0.texture_slab.mem_size();
        r += self.0.sampler_slab.mem_size();
        r += self.0.rt_slab.mem_size();
        r += self.0.rb_slab.mem_size();
        r += self.0.bs_slab.mem_size();
        r += self.0.ds_slab.mem_size();
        r += self.0.rs_slab.mem_size();
        r += self.0.ss_slab.mem_size();
        r += self.0.program_slab.mem_size();

        let context = convert_to_mut(self.0.as_ref());
        context.stat.slab_mem_size = r;

        &self.0.stat
    }

    fn render_set_program(&self, program: &HalProgram) {
        let context = convert_to_mut(self.0.as_ref());

        let p = get_ref(
            &context.program_slab,
            program.item.index,
            program.item.use_count,
        )
        .expect("param not found");
        if context.state_machine.set_program(&context.gl, program, p) {
            #[cfg(feature = "frame_stat")]
            context.stat.add_program_change();
        }
    }

    fn render_set_state(
        &self,
        bs: &HalBlendState,
        ds: &HalDepthState,
        rs: &HalRasterState,
        ss: &HalStencilState,
    ) {
        let context = convert_to_mut(self.0.as_ref());
        let bsdesc = get_ref(&context.bs_slab, bs.item.index, bs.item.use_count)
            .expect("bs param not found");
        let dsdesc = get_ref(&context.ds_slab, ds.item.index, ds.item.use_count)
            .expect("ds param not found");
        let ssdesc = get_ref(&context.ss_slab, ss.item.index, ss.item.use_count)
            .expect("ss param not found");
        let rsdesc = get_ref(&context.rs_slab, rs.item.index, rs.item.use_count)
            .expect("rs param not found");
        context.state_machine.set_state(
            &context.gl,
            rs,
            bs,
            ss,
            ds,
            rsdesc,
            bsdesc,
            ssdesc,
            dsdesc,
        );
    }

    fn render_draw(&self, geometry: &HalGeometry, pp: &Share<dyn ProgramParamter>) {
        let context = convert_to_mut(self.0.as_ref());

        let program = context.state_machine.get_curr_program();
        let pimpl = get_mut_ref(&mut context.program_slab, program.0, program.1)
            .expect("curr program not found");

        let _count = context.state_machine.set_uniforms(
            &context.gl,
            pimpl,
            pp,
            &mut context.texture_slab,
            &mut context.sampler_slab,
        );

        #[cfg(feature = "frame_stat")]
        context.stat.add_texture_change(_count);

        let gimpl = get_ref(
            &mut context.geometry_slab,
            geometry.item.index,
            geometry.item.use_count,
        )
        .expect("geometry not found");
        if context.state_machine.draw(
            &context.gl,
            &context.vao_extension,
            geometry,
            gimpl,
            &context.buffer_slab,
        ) {
            #[cfg(feature = "frame_stat")]
            context.stat.add_geometry_change();
        }

        #[cfg(feature = "frame_stat")]
        context.stat.add_draw_call();
    }
}

impl WebglHalContextImpl {
    fn buffer_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.buffer_slab, index, use_count).is_some() {
            let context = convert_to_mut(self);
            let rimpl = context.buffer_slab.remove(index as usize);
            context.stat.buffer_count -= 1;
            rimpl.0.delete(&self.gl);
        }
    }

    fn geometry_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.geometry_slab, index, use_count).is_some() {
            let context = convert_to_mut(self);
            let rimpl = context.geometry_slab.remove(index as usize);
            rimpl.0.delete(&self.vao_extension);
            context.stat.geometry_count -= 1;
        }
    }

    fn program_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.program_slab, index, use_count).is_some() {
            let context = convert_to_mut(self);
            let rimpl = context.program_slab.remove(index as usize);
            context.stat.program_count -= 1;
            rimpl.0.delete(&self.gl);
        }
    }

    fn rt_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.rt_slab, index, use_count).is_some() {
            let context = convert_to_mut(self);
            context.stat.rt_count -= 1;
            let rimpl = context.rt_slab.remove(index as usize);
            let rimpl = rimpl.0;
            let is_tex_destroy = rimpl.is_tex_destroy;
            rimpl.delete(&self.gl);

            if let Some(t) = &rimpl.color {
                if is_tex_destroy {
                    self.texture_destroy(t.item.index, t.item.use_count);
                }
            }
            if let Some(rb) = &rimpl.depth {
                self.rb_destroy(rb.item.index, rb.item.use_count);
            }
        }
    }

    fn rb_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.rb_slab, index, use_count).is_some() {
            let slab = convert_to_mut(&self.rb_slab);
            let rimpl = slab.remove(index as usize);
            rimpl.0.delete(&self.gl);
        }
    }

    fn texture_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.texture_slab, index, use_count).is_some() {
            let context = convert_to_mut(self);
            let rimpl = context.texture_slab.remove(index as usize);
            context.stat.texture_count -= 1;
            rimpl.0.delete(&self.gl);
        }
    }

    fn sampler_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.sampler_slab, index, use_count).is_some() {
            let slab = convert_to_mut(&self.sampler_slab);
            slab.remove(index as usize);
        }
    }

    fn rs_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.rs_slab, index, use_count).is_some() {
            let slab = convert_to_mut(&self.rs_slab);
            slab.remove(index as usize);
        }
    }

    fn ds_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.ds_slab, index, use_count).is_some() {
            let slab = convert_to_mut(&self.ds_slab);
            slab.remove(index as usize);
        }
    }

    fn ss_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.ss_slab, index, use_count).is_some() {
            let slab = convert_to_mut(&self.ss_slab);
            slab.remove(index as usize);
        }
    }

    fn bs_destroy(&self, index: u32, use_count: u32) {
        if get_ref(&self.bs_slab, index, use_count).is_some() {
            let slab = convert_to_mut(&self.bs_slab);
            slab.remove(index as usize);
        }
    }
}

impl WebglHalContext {
    pub fn new(gl: WebGlRenderingContext, use_vao: bool) -> WebglHalContext {
        let buffer_slab = Slab::new();
        let geometry_slab = Slab::new();
        let mut texture_slab = Slab::new();
        let sampler_slab = Slab::new();
        let mut rt_slab = Slab::new();
        let rb_slab = Slab::new();
        let bs_slab = Slab::new();
        let ds_slab = Slab::new();
        let rs_slab = Slab::new();
        let ss_slab = Slab::new();
        let program_slab = Slab::new();

        let (caps, extensions) = WebglHalContext::create_caps_and_extensions(&gl);
        let vao_extension = if use_vao && caps.vertex_array_object {
			match gl.get_extension("OES_vertex_array_object") {
				Ok(r) => match r {
					Some(r) => Some(OESVertexArrayObject::from(JsValue::from(r))),
					None => None,
				},
				Err(_) => None,
			}
            // TryInto::<Object>::try_into(js! {

            //     var extension = @{gl.as_ref()}.getExtension("OES_vertex_array_object");
            //     if (!extension) { return; }
            //     var vaoExtensionWrap = {
            //         wrap: extension
            //     };
            //     return vaoExtensionWrap;
            // })
            // .ok()
        } else {
            None
        };

        let shader_cache = ShaderCache::new();
        let state_machine = StateMachine::new(
            &gl,
            vao_extension.is_some(),
            caps.max_vertex_attribs,
            caps.max_textures_image_units,
            &mut texture_slab,
            &rt_slab,
        );

        let rt = WebGLRenderTargetImpl::new_default(None, 0, 0);
        let (index, use_count) = create_new_slot(&mut rt_slab, rt);

        let context_impl = Share::new(WebglHalContextImpl {
            stat: RenderStat::new(),
            gl: gl,
            caps: caps,
            extensions: extensions,
            vao_extension,
            shader_cache,
            state_machine,
            buffer_slab,
            geometry_slab,
            texture_slab,
            sampler_slab,
            rt_slab,
            rb_slab,
            bs_slab,
            ds_slab,
            rs_slab,
            ss_slab,
            program_slab,
        });

        let context_clone = context_impl.clone();
        let default_rt = HalRenderTarget {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_clone.rt_destroy(index, use_count)
            }),
        };

        let rt = get_ref(
            &context_impl.rt_slab,
            default_rt.item.index,
            default_rt.item.use_count,
        )
        .expect("rt param not found");
        let cc = convert_to_mut(context_impl.as_ref());
        cc.state_machine
            .set_render_target(&context_impl.gl, &default_rt, &rt);

        WebglHalContext(context_impl, default_rt)
    }

    pub fn texture_get_object_webgl(&self, tex: &HalTexture) -> Option<&WebGlTexture> {
        let slab = convert_to_mut(&self.0.texture_slab);
        if let Some(t) = get_mut_ref(slab, tex.item.index, tex.item.use_count) {
            Some(&t.handle)
        } else {
            None
        }
    }

    /**
     * 创建webgl纹理
     *    data是个普通的Javascript Object对象，wrap字段是：Canvas，Image，...
     */
    pub fn texture_create_2d_webgl(
        &self,
        mipmap_level: u32,
        width: u32,
        height: u32,
        pformat: PixelFormat,
        dformat: DataFormat,
        is_gen_mipmap: bool,
        data: Option<&Object>,
    ) -> Result<HalTexture, String> {
        WebGLTextureImpl::new_2d(
            &self.0.gl,
            mipmap_level,
            width,
            height,
            pformat,
            dformat,
            is_gen_mipmap,
            None,
            data,
        )
        .map(|texture| {
            let slab = convert_to_mut(&self.0.texture_slab);
            let (index, use_count) = create_new_slot(slab, texture);

            {
                let context = convert_to_mut(self.0.as_ref());
                context.stat.texture_count += 1;
            }

            let context_impl = self.0.clone();
            HalTexture {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.texture_destroy(index, use_count);
                }),
            }
        })
    }

    pub fn compressed_texture_create_2d_webgl(
        &self,
        mipmap_level: u32,
        width: u32,
        height: u32,
        internalformat: CompressedTexFormat,
        is_gen_mipmap: bool,
        data: Option<&Object>,
    ) -> Result<HalTexture, String> {
        let context_impl = self.0.clone();
        // let webgl_internalformat = get_compressed_tex_const(internalformat, &context_impl);
        let webgl_internalformat = internalformat.0;
        if webgl_internalformat == -1 {
            return Err(format!("不支持压缩纹理格式：{:?}", internalformat));
        }
        WebGLTextureImpl::new_compressed_2d(
            &self.0.gl,
            mipmap_level,
            width,
            height,
            webgl_internalformat as u32,
            is_gen_mipmap,
            None,
            data,
        )
        .map(|texture| {
            let context = convert_to_mut(self.0.as_ref());
            let (index, use_count) = create_new_slot(&mut context.texture_slab, texture);
            context.stat.texture_count += 1;

            HalTexture {
                item: HalItem { index, use_count },
                destroy_func: Share::new(move |index: u32, use_count: u32| {
                    context_impl.texture_destroy(index, use_count);
                }),
            }
        })
    }

    /**
     * 更新webgl纹理
     *    data是个普通的Javascript Object对象，wrap字段是：Canvas，Image，...
     */
    pub fn texture_update_webgl(
        &self,
        texture: &HalTexture,
        mipmap_level: u32,
        x: u32,
        y: u32,
        data: &Object,
    ) {
        let slab = convert_to_mut(&self.0.texture_slab);
        if let Some(t) = get_mut_ref(slab, texture.item.index, texture.item.use_count) {
            t.update(&self.0.gl, mipmap_level, None, Some((x, y, data)));
        }
    }

    /**
     * 注：WebGlFramebuffer在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: WebGlFramebuffer}
     */
    pub fn rt_create_webgl(&self, fbo: WebGlFramebuffer) -> HalRenderTarget {
        let rt_slab = convert_to_mut(&self.0.rt_slab);

        let rt = WebGLRenderTargetImpl::new_default(Some(fbo), 0, 0);
        let (index, use_count) = create_new_slot(rt_slab, rt);

        let context_impl = self.0.clone();
        HalRenderTarget {
            item: HalItem { index, use_count },
            destroy_func: Share::new(move |index: u32, use_count: u32| {
                context_impl.rt_destroy(index, use_count);
            }),
        }
    }

    fn texture_copy_impl(
        &self,
        dst: &WebGLTextureImpl,
        src: &HalTexture,
        src_mipmap_level: u32,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        height: u32,
    ) {
        let rt;
        {
            let temp = self.0.state_machine.get_curr_rt();
            rt = get_ref(&self.0.rt_slab, temp.0, temp.1).unwrap();
        }

        if let Some(src) = get_ref(&self.0.texture_slab, src.item.index, src.item.use_count) {
            let fb_type = WebGlRenderingContext1::FRAMEBUFFER;
            let tex_target = WebGlRenderingContext1::TEXTURE_2D;
            let color_attachment = WebGlRenderingContext1::COLOR_ATTACHMENT0;

            self.0
                .gl
                .bind_framebuffer(fb_type, Some(&self.0.state_machine.copy_fbo));
            self.0.gl.framebuffer_texture_2d(
                fb_type,
                color_attachment,
                tex_target,
                Some(&src.handle),
                0,
            );
        }

        dst.copy(
            &self.0.gl,
            src_mipmap_level,
            src_x,
            src_y,
            dst_x,
            dst_y,
            width,
            height,
        );

        let context = convert_to_mut(self.0.as_ref());
        context.state_machine.set_render_target_impl(&self.0.gl, rt);
    }

    fn create_caps_and_extensions(gl: &WebGlRenderingContext) -> (Capabilities, Extensions) {
        let max_textures_image_units = gl
			.get_parameter(WebGlRenderingContext1::MAX_TEXTURE_IMAGE_UNITS)
			.unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_vertex_texture_image_units = gl
            .get_parameter(WebGlRenderingContext1::MAX_VERTEX_TEXTURE_IMAGE_UNITS)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_combined_textures_image_units = gl
            .get_parameter(WebGlRenderingContext1::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_texture_size = gl
            .get_parameter(WebGlRenderingContext1::MAX_TEXTURE_SIZE)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_render_texture_size = gl
            .get_parameter(WebGlRenderingContext1::MAX_RENDERBUFFER_SIZE)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_vertex_attribs = gl
            .get_parameter(WebGlRenderingContext1::MAX_VERTEX_ATTRIBS)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_varying_vectors = gl
            .get_parameter(WebGlRenderingContext1::MAX_VARYING_VECTORS)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_vertex_uniform_vectors = gl
            .get_parameter(WebGlRenderingContext1::MAX_VERTEX_UNIFORM_VECTORS)
            .unwrap()
			.as_f64()
			.unwrap() as u32;
        let max_fragment_uniform_vectors = gl
            .get_parameter(WebGlRenderingContext1::MAX_FRAGMENT_UNIFORM_VECTORS)
            .unwrap()
			.as_f64()
			.unwrap() as u32;

        let standard_derivatives = has_extension::<OESStandardDerivatives>(gl);
        let uint_indices = has_extension::<OESElementIndexUint>(gl);

        let fragment_depth_supported = has_extension::<EXTFragDepth>(gl);

        let texture_float =  has_extension::<OESTextureFloat>(gl);
        let texture_float_linear_filtering = texture_float
            && has_extension::<OESTextureFloatLinear>(gl);

        let texture_lod = has_extension::<EXTShaderTextureLod>(gl);
        let color_buffer_float = has_extension::<WEBGLColorBufferFloat>(gl);

        let depth_texture_extension = has_extension::<WEBGLDepthTexture>(gl);
        // depth_texture_extension.UNSIGNED_INT_24_8_WEBGL;

        let vertex_array_object = has_extension::<OESVertexArrayObject1>(gl);
        let instanced_arrays = has_extension::<ANGLEInstancedArrays>(gl);

        // let (astc, astc_e) = match gl
        // .get_extension::<CompressedTextureAstc>(){
        // 	Some(e) => (true)
        // 	None => (false, None);
        // };
		let (astc, astc_e) = get_extension::<CompressedTextureAstc>(gl).map_or(
			get_extension::<WebkitCompressedTextureAstc>(gl).map_or(
				(false, None), 
				|v| (true, Some(v.0))), 
				|v| (true, Some(v.0)));

        let mut s3tc = has_extension::<CompressedTextureS3tc>(gl);
        if !s3tc {
            s3tc = has_extension::<WebkitCompressedTextureS3tc>(gl);
        }

		let (pvrtc, pvrtc_e) = get_extension::<CompressedTexturePvrtc>(gl).map_or(
			get_extension::<WebkitCompressedTexturePvrtc>(gl).map_or(
				(false, None), 
				|v| (true, Some(v.0))), 
				|v| (true, Some(v.0)));

		let (etc1, etc1_e) = get_extension::<CompressedTextureEtc1>(gl).map_or(
			get_extension::<WebkitCompressedTextureEtc1>(gl).map_or(
				(false, None), 
				|v| (true, Some(v.0))), 
				|v| (true, Some(v.0)));

		let (etc2, etc2_e) = get_extension::<CompressedTextureEtc2>(gl).map_or(
			get_extension::<WebkitCompressedTextureEtc2>(gl).map_or(
				(false, None), 
				|v| (true, Some(v.0))),
				|v| (true, Some(v.0)));

        // if !etc2 {
        //     etc2 = gl
        //         .get_extension::<CompressedTextureEs3>()
        //         .map_or(false, |_v| true);
        // }

        (
            Capabilities {
                astc: astc,
                s3tc: s3tc,
                pvrtc: pvrtc,
                etc1: etc1,
                etc2: etc2,
                max_textures_image_units: max_textures_image_units,
                max_vertex_texture_image_units: max_vertex_texture_image_units,
                max_combined_textures_image_units: max_combined_textures_image_units,
                max_texture_size: max_texture_size,
                max_render_texture_size: max_render_texture_size,
                max_vertex_attribs: max_vertex_attribs,
                max_varying_vectors: max_varying_vectors,
                max_vertex_uniform_vectors: max_vertex_uniform_vectors,
                max_fragment_uniform_vectors: max_fragment_uniform_vectors,
                standard_derivatives: standard_derivatives,
                uint_indices: uint_indices,
                fragment_depth_supported: fragment_depth_supported,
                texture_float: texture_float,
                texture_float_linear_filtering: texture_float_linear_filtering,
                texture_lod: texture_lod,
                color_buffer_float: color_buffer_float,
                depth_texture_extension: depth_texture_extension,
                vertex_array_object: vertex_array_object,
                instanced_arrays: instanced_arrays,
            },
            Extensions {
                pvrtc: pvrtc_e,
                etc1: etc1_e,
                etc2: etc2_e,
                astc: astc_e,
            },
        )
    }
}

// #[allow(unreachable_patterns)]
// fn get_compressed_tex_const(
//     format: CompressedTexFormat,
//     context_impl: &WebglHalContextImpl,
// ) -> i32 {
//     match format {
//         CompressedTexFormat::RGBA_ASTC_4x4 => context_impl
//             .extensions
//             .astc
//             .as_ref()
//             .map_or(-1, |v| v.rgba_astc_4x4 as i32),
//         CompressedTexFormat::RGBA_PVRTC_2BPPV1 => context_impl
//             .extensions
//             .pvrtc
//             .as_ref()
//             .map_or(-1, |v| v.rgba_pvrtc_2bppv1 as i32),
//         CompressedTexFormat::RGB_PVRTC_4BPPV1 => context_impl
//             .extensions
//             .pvrtc
//             .as_ref()
//             .map_or(-1, |v| v.rgba_pvrtc_4bppv1 as i32),
//         CompressedTexFormat::RGB_PVRTC_2BPPV1 => context_impl
//             .extensions
//             .pvrtc
//             .as_ref()
//             .map_or(-1, |v| v.rgb_pvrtc_2bppv1 as i32),
//         CompressedTexFormat::RGB_PVRTC_4BPPV1 => context_impl
//             .extensions
//             .pvrtc
//             .as_ref()
//             .map_or(-1, |v| v.rgb_pvrtc_4bppv1 as i32),
//         CompressedTexFormat::RGB_ETC1 => context_impl
//             .extensions
//             .etc1
//             .as_ref()
//             .map_or(-1, |v| v.rgb_etc1 as i32),
//         CompressedTexFormat::RGBA8_ETC2_EAC => context_impl
//             .extensions
//             .etc2
//             .as_ref()
//             .map_or(-1, |v| v.rgba8_etc2_eac as i32),
//         CompressedTexFormat::RGB8_ETC2 => context_impl
//             .extensions
//             .etc2
//             .as_ref()
//             .map_or(-1, |v| v.rgb8_etc2 as i32),
//         _ => panic!("compressed_tex format invaild:{:?}", format),
//     }
// }

fn get_extension<E: Extension + TryFrom<JsValue>>(gl: &WebGlRenderingContext) -> Option<E> {
	match gl.get_extension(E::NAME) {
		Ok(r) => match r {
			Some(r) => E::try_from(JsValue::from(r)).ok(),
			None => None
		},
		Err(_) => None,
	}
}

fn has_extension<E: Extension>(gl: &WebGlRenderingContext) -> bool {
	match gl.get_extension(E::NAME) {
		Ok(r) => match r {
			Some(_) => true,
			None => false,
		},
		Err(_) => false,
	}
}
