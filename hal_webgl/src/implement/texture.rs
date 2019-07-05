use hal_core::{PixelFormat, DataFormat, TextureData, SamplerDesc};
use share::{Share};
use implement::context::{WebGLContextImpl}; 
use wrap::{WebGLContextWrap};
use webgl_rendering_context::{WebGLTexture, WebGLRenderingContext};

use implement::convert::*;

pub struct WebGLTextureImpl {
    context: Share<WebGLContextImpl>,
 
    pub width: u32,
    pub height: u32,
    pub level: u32,
    pub pixel_format: PixelFormat,
    pub data_format: DataFormat,
    pub is_gen_mipmap: bool,
    pub handle: WebGLTexture,
 
    pub sampler: SamplerDesc,
}

impl WebGLTextureImpl {
    pub fn new_2d(context: &Share<WebGLContextImpl>, width: u32, height: u32, pformat: PixelFormat, dformat: DataFormat, is_gen_mipmap: bool, data: Option<TextureData<WebGLContextWrap>>) -> Result<Self, String> {
        let gl = &context.context;
        let texture = gl.create_texture();
        if texture.is_none() {
            return Err("new_2d failed, not found".to_string());
        }
        let texture = texture.unwrap();

        let p = get_pixel_format(pformat);
        let d = get_data_format(dformat);
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));

        match data {
            None => {
                gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, 0, p as i32, width as i32, height as i32, 0, p, d, Option::<&[u8]>::None);
            }
            Some(TextureData::U8(_, _, _, _, v)) => {
                gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, 0, p as i32, width as i32, height as i32, 0, p, d, Some(v));
            }
            Some(TextureData::F32(_, _, _, _, v)) => {
                gl.tex_image2_d(WebGLRenderingContext::TEXTURE_2D, 0, p as i32, width as i32, height as i32, 0, p, d, Some(v));
            }
            _ => return Err("new_2d failed, invalid data".to_string())
        }
        
        if is_gen_mipmap {
            gl.generate_mipmap(WebGLRenderingContext::TEXTURE_2D);
        }
        
        let t = WebGLTextureImpl {
            context: context.clone(),
            width: width,
            height: height,
            level: 0,
            pixel_format: pformat,
            data_format: dformat,
            is_gen_mipmap: is_gen_mipmap,
            handle: texture,
            sampler: SamplerDesc::new(),
        };

        t.apply_sampler(&t.sampler);
    }

    pub fn delete(&self) {
        self.context.context.delete_texture(Some(&self.handle));
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_render_format(&self) -> PixelFormat {
        self.pixel_format
    }

    pub fn is_gen_mipmap(&self) -> bool {
        self.is_gen_mipmap
    }

    pub fn update(&self, mipmap_level: u32, data: &TextureData<WebGLContextWrap>) {

    }

    pub fn apply_sampler(&self, sampler: &SamplerDesc) {

        let gl = &self.context.context;
        let sampler = &sampler.desc;

        let u_wrap = get_texture_wrap_mode(&sampler.u_wrap);
        let v_wrap = get_texture_wrap_mode(&sampler.v_wrap);

        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_WRAP_S, u_wrap as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_WRAP_T, v_wrap as i32);
    
        let (mag, min) = get_texture_filter_mode(&sampler.mag_filter, &sampler.min_filter, sampler.mip_filter.as_ref());
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MIN_FILTER, min as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MAG_FILTER, mag as i32);
    }
}