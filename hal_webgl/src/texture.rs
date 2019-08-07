use share::{Share};
use hal_core::{Texture, TextureData, PixelFormat, DataFormat};
use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};
use stdweb::{Object};

use convert::*;
use sampler::{WebGLSamplerImpl};

use std::rc::{Rc};
use std::cell::{RefCell};
use context::{RenderStats};
use util::*;
use state::{State};

pub struct WebGLTextureImpl {
    pub gl: Share<WebGLRenderingContext>,
    pub state: Share<State>,
    pub width: u32,
    pub height: u32,
    pub level: u32,
    pub pixel_format: PixelFormat,
    pub data_format: DataFormat,
    pub is_gen_mipmap: bool,
    pub handle: WebGLTexture,
    pub sampler: WebGLSamplerImpl,
    pub stats: Rc<RefCell<RenderStats>>,
}

impl Texture for WebGLTextureImpl {
    
    fn extend(&self, width: u32, height: u32) -> bool {
        match Self::new_2d_impl(&self.gl, width, height, 0, &self.pixel_format, &self.data_format, self.is_gen_mipmap, &TextureData::None) {
            None => false,
            Some(new_tex) => {
                self.copy_impl(&new_tex);
                self.gl.delete_texture(Some(&self.handle));
                
                let old_tex = convert_to_mut(self);
                old_tex.handle = new_tex;
                old_tex.width = width;
                old_tex.height = height;
                
                true
            }
        }
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn get_render_format(&self) -> PixelFormat {
        self.pixel_format
    }

    fn is_gen_mipmap(&self) -> bool {
        self.is_gen_mipmap
    }

    fn update(&self, x: u32, y: u32, width: u32, height: u32, data: &TextureData) {
        let p = get_pixel_format(&self.pixel_format);
        let d = get_data_format(&self.data_format);

        self.gl.active_texture(WebGLRenderingContext::TEXTURE0);
        self.gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.handle));

        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        match data {
            TextureData::None => {
                self.gl.tex_sub_image2_d(WebGLRenderingContext::TEXTURE_2D, self.level as i32, x as i32, y as i32, width as i32, height as i32, p, d, Option::<&[u8]>::None);
            }
            TextureData::U8(v) => {
                self.gl.tex_sub_image2_d(WebGLRenderingContext::TEXTURE_2D, self.level as i32, x as i32, y as i32, width as i32, height as i32, p, d, Some(*v));
            }
            TextureData::F32(v) => {
                self.gl.tex_sub_image2_d(WebGLRenderingContext::TEXTURE_2D, self.level as i32, x as i32, y as i32, width as i32, height as i32, p, d, Some(*v));
            }
        }
    }
}

impl Drop for WebGLTextureImpl {
    fn drop(&mut self) {
        self.stats.borrow_mut().texture_count -= 1;
        //println!("================= WebGLTextureImpl Drop, stats = {:?}", self.stats.borrow());
        self.gl.delete_texture(Some(&self.handle));
    }
}

impl AsRef<Self> for WebGLTextureImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl WebGLTextureImpl {
    
    // 将self的纹理内存拷贝到dst去
    pub fn copy_impl(&self, dst: &WebGLTexture) {
        let state = convert_to_mut(self.state.as_ref());
        
        let rt = state.get_curr_rt();
        
        let fb_type = WebGLRenderingContext::FRAMEBUFFER;
        let tex_target = WebGLRenderingContext::TEXTURE_2D;
        let color_attachment = WebGLRenderingContext::COLOR_ATTACHMENT0;
        
        self.gl.bind_framebuffer(fb_type, Some(&self.state.copy_fbo));
        self.gl.framebuffer_texture2_d(fb_type, color_attachment, tex_target, Some(&self.handle), 0);
        
        // 将fbo的内容拷到目标纹理
        self.gl.active_texture(WebGLRenderingContext::TEXTURE0);
        self.gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(dst));

        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        self.gl.copy_tex_sub_image2_d(WebGLRenderingContext::TEXTURE_2D, 0, 0, 0, 0, 0, self.width as i32, self.height as i32);
        
        state.set_render_target_impl(&rt);
    }

    pub fn new_2d(stats: &Rc<RefCell<RenderStats>>, state: Share<State>, gl: &Share<WebGLRenderingContext>, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &TextureData) -> Result<Self, String> {
        match Self::new_2d_impl(gl, w, h, level, pformat, dformat, is_gen_mipmap, data) {
            Some(handle) => {
                let t = WebGLTextureImpl {
                    gl: gl.clone(),
                    state: state,
                    width: w,
                    height: h,
                    level: level,
                    pixel_format: *pformat,
                    data_format: *dformat,
                    is_gen_mipmap: is_gen_mipmap,
                    handle: handle,
                    sampler: WebGLSamplerImpl::new(stats),
                    stats: stats.clone(),
                };

                t.apply_sampler(&t.sampler);

                Ok(t)
            }
            None => Err("new_2d_with_data failed".to_string())
        }
    }

    fn new_2d_impl(gl: &Share<WebGLRenderingContext>, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &TextureData) -> Option<WebGLTexture> {
        match gl.create_texture()  {
            Some(texture) => {
                let p = get_pixel_format(pformat);
                let d = get_data_format(dformat);
                gl.active_texture(WebGLRenderingContext::TEXTURE0);
                gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));

                gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
                gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
                gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

                match data {
                    TextureData::None => {
                        gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, level as i32, p as i32, w as i32, h as i32, 0, p, d, Option::<&[u8]>::None);
                    }
                    TextureData::U8(v) => {
                        gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, level as i32, p as i32, w as i32, h as i32, 0, p, d, Some(*v));
                    }
                    TextureData::F32(v) => {
                        gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, level as i32, p as i32, w as i32, h as i32, 0, p, d, Some(*v));
                    }
                }
                
                if is_gen_mipmap {
                    gl.generate_mipmap(WebGLRenderingContext::TEXTURE_2D);
                }
                Some(texture)
            }
            None => None
        }
    }

    /** 
     * 注：data是Image或者是Canvas对象，但是那两个在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: Image | Canvas}
     */
    pub fn new_2d_webgl(stats: &Rc<RefCell<RenderStats>>, state: Share<State>, gl: &Share<WebGLRenderingContext>, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &Object) -> Result<WebGLTextureImpl, String> {
        match gl.create_texture()  {
            Some(texture) => {
                let p = get_pixel_format(pformat);
                let d = get_data_format(dformat);
                gl.active_texture(WebGLRenderingContext::TEXTURE0);
                gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));

                gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
                gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
                gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

                js! {
                    @{gl.as_ref()}.texImage2D(@{WebGLRenderingContext::TEXTURE_2D}, @{level}, @{p}, @{p}, @{d}, @{data}.wrap);
                }
                
                if is_gen_mipmap {
                    gl.generate_mipmap(WebGLRenderingContext::TEXTURE_2D);
                }
                
                let t = WebGLTextureImpl {
                    gl: gl.clone(),
                    state: state,
                    width: w,
                    height: h,
                    level: level,
                    pixel_format: *pformat,
                    data_format: *dformat,
                    is_gen_mipmap: is_gen_mipmap,
                    handle: texture,
                    sampler: WebGLSamplerImpl::new(&stats),
                    stats: stats.clone(),
                };

                t.apply_sampler(&t.sampler);

                Ok(t)
            }
            None => Err("new_2d_with_data failed".to_string())
        }
    }

    /** 
     * 注：data是Image或者是Canvas对象，但是那两个在小游戏真机上不是真正的Object对象，所以要封装成：{wrap: Image | Canvas}
     */
    pub fn update_webgl(&self, x: u32, y: u32, _w: u32, _h: u32, data: &Object) {
        let p = get_pixel_format(&self.pixel_format);
        let d = get_data_format(&self.data_format);
        
        self.gl.active_texture(WebGLRenderingContext::TEXTURE0);
        self.gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.handle));

        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        self.gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        js! {
            @{self.gl.as_ref()}.texSubImage2D(@{WebGLRenderingContext::TEXTURE_2D}, @{self.level}, @{x}, @{y}, @{p}, @{d}, @{data}.wrap);
        }
    }

    pub fn apply_sampler(&self, sampler: &WebGLSamplerImpl) {

        let u_wrap = get_texture_wrap_mode(&sampler.u_wrap);
        let v_wrap = get_texture_wrap_mode(&sampler.v_wrap);

        self.gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_WRAP_S, u_wrap as i32);
        self.gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_WRAP_T, v_wrap as i32);
    
        let (mag, min) = get_texture_filter_mode(&sampler.mag_filter, &sampler.min_filter, sampler.mip_filter.as_ref());
        self.gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MIN_FILTER, min as i32);
        self.gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MAG_FILTER, mag as i32);
    }
}