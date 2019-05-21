use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture, WebGLFramebuffer};

pub struct RenderTarget {
    pub frambuffer: WebGLFramebuffer,
    pub texture: WebGLTexture,
}

impl RenderTarget {
    pub fn create(gl: &WebGLRenderingContext, width: f32, height: f32) -> RenderTarget{
        let width = next_power_of_two(width as u32);
        let height = next_power_of_two(height as u32);
        let frambuffer = gl.create_framebuffer().unwrap();
        let texture = gl.create_texture().unwrap();
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_image2_d::<&[u8]>(WebGLRenderingContext::TEXTURE_2D, 0, WebGLRenderingContext::RGB as i32, width as i32, height as i32, 0, WebGLRenderingContext::RGB, WebGLRenderingContext::UNSIGNED_BYTE, None);

        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MIN_FILTER, WebGLRenderingContext::NEAREST as i32);
        gl.tex_parameteri(WebGLRenderingContext::TEXTURE_2D, WebGLRenderingContext::TEXTURE_MAG_FILTER, WebGLRenderingContext::NEAREST as i32);

        gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, Some(&frambuffer));
        gl.framebuffer_texture2_d(WebGLRenderingContext::FRAMEBUFFER,WebGLRenderingContext::COLOR_ATTACHMENT0, WebGLRenderingContext::TEXTURE_2D, Some(&texture), 0);
        gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, None);

        RenderTarget {
            frambuffer, 
            texture
        }
    }
}


fn next_power_of_two(value: u32) -> u32 {
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value += 1;
    value
}