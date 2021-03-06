use convert::*;
use hal_core::{DataFormat, PixelFormat, PixelStore, SamplerDesc, TextureData};
use stdweb::Object;
use webgl_rendering_context::{WebGLRenderingContext, WebGLTexture};

pub struct WebGLTextureImpl {
    pub width: u32,
    pub height: u32,
    pub mipmap_level: u32,
    pub pixel_format: PixelFormat,
    pub data_format: DataFormat,
    pub is_gen_mipmap: bool,
    pub handle: WebGLTexture,

    // 纹理缓存
    pub cache_index: i32,         // 仅当 >= 0 时有意义
    pub curr_sampler: (u32, u32), // slab的index, use_count
}

impl WebGLTextureImpl {
    pub fn new_2d(
        gl: &WebGLRenderingContext,
        mipmap_level: u32,
        width: u32,
        height: u32,
        pformat: PixelFormat,
        dformat: DataFormat,
        is_gen_mipmap: bool,
        data: Option<TextureData>,
        webgl_object: Option<&Object>,
    ) -> Result<Self, String> {
        let texture = gl.create_texture();
        if texture.is_none() {
            return Err("new_2d failed, not found".to_string());
        }
        let texture = texture.unwrap();

        let p = get_pixel_format(pformat);
        let d = get_data_format(dformat);
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));

        gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        match webgl_object {
            Some(object) => {
                js! {
                    @{&gl}.texImage2D(@{WebGLRenderingContext::TEXTURE_2D}, 0, @{p}, @{p}, @{d}, @{object}.wrap);
                }
            }
            None => match data {
                None => {
                    gl.tex_image2_d(
                        WebGLRenderingContext::TEXTURE_2D,
                        0,
                        p as i32,
                        width as i32,
                        height as i32,
                        0,
                        p,
                        d,
                        Option::<&[u8]>::None,
                    );
                }
                Some(TextureData::U8(_, _, _, _, v)) => {
                    gl.tex_image2_d(
                        WebGLRenderingContext::TEXTURE_2D,
                        0,
                        p as i32,
                        width as i32,
                        height as i32,
                        0,
                        p,
                        d,
                        Some(v),
                    );
                }
                Some(TextureData::F32(_, _, _, _, v)) => {
                    gl.tex_image2_d(
                        WebGLRenderingContext::TEXTURE_2D,
                        0,
                        p as i32,
                        width as i32,
                        height as i32,
                        0,
                        p,
                        d,
                        Some(v),
                    );
                }
            },
        }

        if is_gen_mipmap {
            gl.generate_mipmap(WebGLRenderingContext::TEXTURE_2D);
        }

        let t = WebGLTextureImpl {
            width: width,
            height: height,
            mipmap_level: mipmap_level,
            pixel_format: pformat,
            data_format: dformat,
            is_gen_mipmap: is_gen_mipmap,
            handle: texture,

            cache_index: -1,
            curr_sampler: (0, 0),
        };

		t.apply_sampler(gl, &SamplerDesc::new());
		gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, None);

        Ok(t)
    }

    pub fn new_compressed_2d(
        gl: &WebGLRenderingContext,
        mipmap_level: u32,
        width: u32,
        height: u32,
        inf: u32, //internalformat对应的webgl常数值
        is_gen_mipmap: bool,
        data: Option<TextureData>,
        webgl_object: Option<&Object>, // Object 可能是uint8Array、float32Array
    ) -> Result<Self, String> {
        let texture = gl.create_texture();
        if texture.is_none() {
            return Err("new_2d failed, not found".to_string());
        }
        let texture = texture.unwrap();

        // var extension = @{gl.as_ref()}.getExtension("OES_vertex_array_object");
        // let p = get_pixel_format(pformat);
        // let d = get_data_format(dformat);
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&texture));

        gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        // println!("inf==================={:?}", inf);
        match webgl_object {
            Some(object) => {
                js! {
                    @{&gl}.compressedTexImage2D(@{WebGLRenderingContext::TEXTURE_2D}, 0, @{inf}, @{width}, @{height}, 0, @{object}.wrap);
                }
            }
            None => match data {
                None => {
                    // gl.compressed_tex_image2_d(
                    //     WebGLRenderingContext::TEXTURE_2D,
                    //     0,
                    //     internalformat,
                    //     width as i32,
                    //     height as i32,
                    //     0,
                    //     Option::<&[u8]>::None,
                    // );
                    panic!("调用compressed_tex_image2_d方法， 数据不能为none");
                }
                Some(TextureData::U8(_, _, _, _, v)) => {
                    gl.compressed_tex_image2_d(
                        WebGLRenderingContext::TEXTURE_2D,
                        0,
                        inf,
                        width as i32,
                        height as i32,
                        0,
                        v,
                    );
                }
                Some(TextureData::F32(_, _, _, _, v)) => {
                    gl.compressed_tex_image2_d(
                        WebGLRenderingContext::TEXTURE_2D,
                        0,
                        inf,
                        width as i32,
                        height as i32,
                        0,
                        v,
                    );
                }
            },
        };

        if is_gen_mipmap {
            gl.generate_mipmap(WebGLRenderingContext::TEXTURE_2D);
        }

        let t = WebGLTextureImpl {
            width: width,
            height: height,
            mipmap_level: mipmap_level,
            // 这两个属性对于压缩纹理来说无效，随意的值
            pixel_format: PixelFormat::RGBA,
            data_format: DataFormat::Byte,

            is_gen_mipmap: is_gen_mipmap,
            handle: texture,

            cache_index: -1,
            curr_sampler: (0, 0),
        };

		t.apply_sampler(gl, &SamplerDesc::new());
		gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, None);

        Ok(t)
    }

    pub fn pixel_storei(&self, gl: &WebGLRenderingContext, value: PixelStore) {
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.handle));
        match value {
            PixelStore::PackAlignment(r) => {
                gl.pixel_storei(WebGLRenderingContext::PACK_ALIGNMENT, r as i32)
            }
            PixelStore::UnpackAlignment(r) => {
                gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, r as i32)
            }
            PixelStore::UnpackFlipYWebgl(r) => gl.pixel_storei(
                WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL,
                match r {
                    true => 1,
                    false => 0,
                },
            ),
            PixelStore::UnpackPremultiplyAlphaWebgl(r) => gl.pixel_storei(
                WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
                match r {
                    true => 1,
                    false => 0,
                },
            ),
        }

        // 	PackAlignment(PixelChanel),
        // UnpackAlignment(PixelChanel),
        // UnpackFlipYWebgl(bool),
        // UnpackPremultiplyAlphaWebgl(bool),
		gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
		gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, None);
    }

    pub fn delete(&self, gl: &WebGLRenderingContext) {
        gl.delete_texture(Some(&self.handle));
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

    /**
     * 将fbo的纹理拷贝到self来
     */
    pub fn copy(
        &self,
        gl: &WebGLRenderingContext,
        src_mipmap_level: u32,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        height: u32,
    ) {
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.handle));

        gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        gl.copy_tex_sub_image2_d(
            WebGLRenderingContext::TEXTURE_2D,
            src_mipmap_level as i32,
            src_x as i32,
            src_y as i32,
            dst_x as i32,
            dst_y as i32,
            width as i32,
            height as i32,
		);
		gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, None);
    }

    pub fn resize(
        &mut self,
        gl: &WebGLRenderingContext,
        _mipmap_level: u32,
        width: u32,
        height: u32,
    ) {
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.handle));

        gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        let p = get_pixel_format(self.pixel_format);
        let d = get_data_format(self.data_format);

        gl.tex_image2_d(
            WebGLRenderingContext::TEXTURE_2D,
            0,
            p as i32,
            width as i32,
            height as i32,
            0,
            p,
            d,
            Option::<&[u8]>::None,
		);
		gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, None);

        self.width = width;
        self.height = height;
    }

    pub fn update(
        &self,
        gl: &WebGLRenderingContext,
        mipmap_level: u32,
        data: Option<&TextureData>,
        webgl_object: Option<(u32, u32, &Object)>,
    ) {
        let p = get_pixel_format(self.pixel_format);
        let d = get_data_format(self.data_format);

        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&self.handle));

        gl.pixel_storei(WebGLRenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
        gl.pixel_storei(WebGLRenderingContext::UNPACK_ALIGNMENT, 4);

        match webgl_object {
            Some((x, y, object)) => {
                js! {
                    @{&gl}.texSubImage2D(@{WebGLRenderingContext::TEXTURE_2D}, @{mipmap_level}, @{x}, @{y}, @{p}, @{d}, @{object}.wrap);
                }
            }
            None => {
                let data = data.unwrap();
                match data {
                    TextureData::U8(x, y, w, h, v) => {
                        gl.tex_sub_image2_d(
                            WebGLRenderingContext::TEXTURE_2D,
                            mipmap_level as i32,
                            *x as i32,
                            *y as i32,
                            *w as i32,
                            *h as i32,
                            p,
                            d,
                            Some(*v),
                        );
                    }
                    TextureData::F32(x, y, w, h, v) => {
                        gl.tex_sub_image2_d(
                            WebGLRenderingContext::TEXTURE_2D,
                            mipmap_level as i32,
                            *x as i32,
                            *y as i32,
                            *w as i32,
                            *h as i32,
                            p,
                            d,
                            Some(*v),
                        );
                    }
                }
            }
		}
		gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, None);
    }

    pub fn apply_sampler(&self, gl: &WebGLRenderingContext, sampler: &SamplerDesc) {
        let u_wrap = get_texture_wrap_mode(sampler.u_wrap);
        let v_wrap = get_texture_wrap_mode(sampler.v_wrap);

        gl.tex_parameteri(
            WebGLRenderingContext::TEXTURE_2D,
            WebGLRenderingContext::TEXTURE_WRAP_S,
            u_wrap as i32,
        );
        gl.tex_parameteri(
            WebGLRenderingContext::TEXTURE_2D,
            WebGLRenderingContext::TEXTURE_WRAP_T,
            v_wrap as i32,
        );

        let (mag, min) =
            get_texture_filter_mode(sampler.mag_filter, sampler.min_filter, sampler.mip_filter);
        gl.tex_parameteri(
            WebGLRenderingContext::TEXTURE_2D,
            WebGLRenderingContext::TEXTURE_MIN_FILTER,
            min as i32,
        );
        gl.tex_parameteri(
            WebGLRenderingContext::TEXTURE_2D,
            WebGLRenderingContext::TEXTURE_MAG_FILTER,
            mag as i32,
        );
    }
}
