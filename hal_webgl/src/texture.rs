use std::sync::{Arc};
use hal_core::{Texture, TextureData, PixelFormat, DataFormat};
use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};
use stdweb::{Object};

use convert::*;
use sampler::{WebGLSamplerImpl};

#[derive(Debug)]
pub struct WebGLTextureImpl {
    pub gl: Arc<WebGLRenderingContext>,
    pub width: u32,
    pub height: u32,
    pub level: u32,
    pub pixel_format: PixelFormat,
    pub data_format: DataFormat,
    pub is_gen_mipmap: bool,
    pub handle: WebGLTexture,
    pub sampler: WebGLSamplerImpl,
}

impl Texture for WebGLTextureImpl {
    
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
    }
}

impl AsRef<Self> for WebGLTextureImpl {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl WebGLTextureImpl {
    
    pub fn new_2d(gl: &Arc<WebGLRenderingContext>, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &TextureData) -> Result<Self, String> {
        match gl.create_texture()  {
            Some(texture) => {
                let p = get_pixel_format(pformat);
                let d = get_data_format(dformat);
                gl.active_texture(WebGLRenderingContext::TEXTURE0);
                gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));

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
                
                let t = WebGLTextureImpl {
                    gl: gl.clone(),
                    width: w,
                    height: h,
                    level: level,
                    pixel_format: *pformat,
                    data_format: *dformat,
                    is_gen_mipmap: is_gen_mipmap,
                    handle: texture,
                    sampler: WebGLSamplerImpl::new(),
                };

                t.apply_sampler(&WebGLSamplerImpl::new());

                Ok(t)
            }
            None => Err("new_2d_with_data failed".to_string())
        }
    }

    pub fn new_2d_webgl(gl: &Arc<WebGLRenderingContext>, w: u32, h: u32, level: u32, pformat: &PixelFormat, dformat: &DataFormat, is_gen_mipmap: bool, data: &Object) -> Result<WebGLTextureImpl, String> {
        match gl.create_texture()  {
            Some(texture) => {
                let p = get_pixel_format(pformat);
                let d = get_data_format(dformat);
                gl.active_texture(WebGLRenderingContext::TEXTURE0);
                gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));
                
                js! {
                    @{gl.as_ref()}.texImage2D(@{WebGLRenderingContext::TEXTURE_2D}, @{level}, @{p}, @{p}, @{d}, @{data});
                }
                
                if is_gen_mipmap {
                    gl.generate_mipmap(WebGLRenderingContext::TEXTURE_2D);
                }
                
                let t = WebGLTextureImpl {
                    gl: gl.clone(),
                    width: w,
                    height: h,
                    level: level,
                    pixel_format: *pformat,
                    data_format: *dformat,
                    is_gen_mipmap: is_gen_mipmap,
                    handle: texture,
                    sampler: WebGLSamplerImpl::new(),
                };

                t.apply_sampler(&WebGLSamplerImpl::new());

                Ok(t)
            }
            None => Err("new_2d_with_data failed".to_string())
        }
    }

    pub fn update_webgl(&self, x: u32, y: u32, w: u32, h: u32, data: &Object) {
        let p = get_pixel_format(&self.pixel_format);
        let d = get_data_format(&self.data_format);

        js! {
            @{self.gl.as_ref()}.texSubImage2D(@{WebGLRenderingContext::TEXTURE_2D}, @{self.level}, @{x}, @{y}, @{w}, @{h}, @{p}, @{d}, @{data});
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