/**
 * Gpu硬件特性
 */
#[derive(Debug)]
pub struct Capabilities {
    // fs中最多的纹理单元
    pub max_textures_image_units: u32,
    // vs中最多的纹理单元
    pub max_vertex_texture_image_units: u32,
    // vs + fs 中最多的纹理单元
    pub max_combined_textures_image_units: u32,
    // 最大的纹理尺寸
    pub max_texture_size: u32,
    // 作为渲染目标的纹理的最大尺寸
    pub max_render_texture_size: u32,
    // attributes的最大数量
    pub max_vertex_attribs: u32,
    // varyings的最大数量
    pub max_varying_vectors: u32,
    // vs最大的uniform数量
    pub max_vertex_uniform_vectors: u32,
    // fs最大的uniform数量
    pub max_fragment_uniform_vectors: u32,
    // 是否支持标准的导数(dx/dy)
    pub standard_derivatives: bool,
    // 是否支持32位索引
    pub uint_indices: bool,
    // 是否支持fs中读深度信息
    pub fragment_depth_supported: bool,
    // 是否支持浮点纹理
    pub texture_float: bool,
    // 浮点纹理的线性过滤
    pub texture_float_linear_filtering: bool,
    // shader中能否使用textureLOD
    pub texture_lod: bool,
    // 是否支持浮点color buffer
    pub color_buffer_float: bool,
    // 是否支持深度纹理
    pub depth_texture_extension: bool,
    // 是否支持VAO
    pub vertex_array_object: bool,
    // 是否支持实例化
    pub instanced_arrays: bool,
}

impl Capabilities {
    pub fn new() -> Self {
        Capabilities {
            max_textures_image_units: 0,
            max_vertex_texture_image_units: 0,
            max_combined_textures_image_units: 0,
            max_texture_size: 0,
            max_render_texture_size: 0,
            max_vertex_attribs: 0,
            max_varying_vectors: 0,
            max_vertex_uniform_vectors: 0,
            max_fragment_uniform_vectors: 0,
            standard_derivatives: false,
            uint_indices: false,
            fragment_depth_supported: false,
            texture_float: false,
            texture_float_linear_filtering: false,
            texture_lod: false,
            color_buffer_float: false,
            depth_texture_extension: false,
            vertex_array_object: false,
            instanced_arrays: false,
        }
    }
}