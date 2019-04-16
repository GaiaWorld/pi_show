// 设置image的uniform，atrribute, index

use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::UnsafeTypedArray;

use atom::Atom;

use world_2d::World2dMgr;
use world_2d::component::image::{ImageDefines};
use render::engine::{Engine, get_uniform_location};

lazy_static! {
    static ref POSITION: Atom = Atom::from("position");
    static ref WORLD_VIEW_PROJECTION: Atom = Atom::from("worldViewProjection");
    static ref UV_OFFSET_SCALE: Atom = Atom::from("uvOffsetScale");
    static ref EXTEND: Atom = Atom::from("extend");
    static ref TEXTURE: Atom = Atom::from("texture");
    static ref ALPHA: Atom = Atom::from("alpha");
    static ref COLOR: Atom = Atom::from("color");
    static ref UV: Atom = Atom::from("uv");
    static ref CLIP_INDEICES: Atom = Atom::from("clipIndices");
    static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
}

// 初始化location
pub fn init_location(defines: &ImageDefines, engine: &mut Engine, program_id: u64) {
    let gl = engine.gl.clone();
    
    let program = engine.lookup_program_mut(program_id).unwrap();
    let uniform_locations = &mut program.uniform_locations;
    let attr_locations = &mut program.attr_locations;
    let program = &program.program;

    if uniform_locations.len() > 0 {
        return;
    }
    
    let positions_location = gl.get_attrib_location(program, &POSITION) as u32;
    attr_locations.insert(POSITION.clone(), positions_location);
    gl.vertex_attrib_pointer(positions_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0);

    let uvs_location = gl.get_attrib_location(program, &UV) as u32;
    attr_locations.insert(UV.clone(), uvs_location);
    gl.vertex_attrib_pointer(uvs_location, 2, WebGLRenderingContext::FLOAT, false, 0, 0);

    uniform_locations.insert(WORLD_VIEW_PROJECTION.clone(), get_uniform_location(&gl,program, &WORLD_VIEW_PROJECTION),);
    uniform_locations.insert(TEXTURE.clone(), get_uniform_location(&gl,program, &TEXTURE),);
    uniform_locations.insert(UV_OFFSET_SCALE.clone(), get_uniform_location(&gl,program, &UV_OFFSET_SCALE),);
    uniform_locations.insert(COLOR.clone(), get_uniform_location(&gl,program, &COLOR),);
    uniform_locations.insert(ALPHA.clone(),get_uniform_location(&gl,program, &ALPHA),);

    if defines.clip_plane {
        uniform_locations.insert(CLIP_INDEICES.clone(), get_uniform_location(&gl,program, &CLIP_INDEICES),);
        uniform_locations.insert(CLIP_TEXTURE.clone(), get_uniform_location(&gl,program, &CLIP_TEXTURE),);
        uniform_locations.insert(CLIP_INDEICES_SIZE.clone(),get_uniform_location(&gl,program, &CLIP_INDEICES_SIZE));
    }
}

//更新uniform和buffer， 并渲染
pub fn render(mgr: &mut World2dMgr, effect_id: usize) {
    let image_effect = mgr.image_effect._group.get(effect_id);
    let image = mgr.image._group.get(image_effect.image_id);

    let defines = mgr.image_effect.defines._group.get(image_effect.defines);
    #[cfg(feature = "log")]
    println!("defines---------------------------{:?}", defines);


    let gl = &mgr.engine.gl;

    let program = mgr.engine.lookup_program(image_effect.program).unwrap();
    let uniform_locations = &program.uniform_locations;
    let attr_locations = &program.attr_locations;
    let program = &program.program;

    // use_program
    gl.use_program(Some(program));

    //设置worldViewProjection
    let world_view = mgr.projection.0 * image.world_matrix.0;
    // let world_view = image.world_matrix.0 * mgr.projection.0;
    #[cfg(feature = "log")]
    println!("p_matrix----------------{:?}", mgr.projection.0);
    #[cfg(feature = "log")]
    println!("world_matrix----------------{:?}", image.world_matrix.0);
    #[cfg(feature = "log")]
    println!("world_view----------------{:?}", world_view);

    let arr: &[f32; 16] = world_view.as_ref();
    gl.uniform_matrix4fv(uniform_locations.get(&WORLD_VIEW_PROJECTION), false, &arr[0..16]);

    // color
    #[cfg(feature = "log")]
    println!("color:{:?}", image.color);
    gl.uniform4f(uniform_locations.get(&COLOR), image.color.r, image.color.g, image.color.b, image.color.a);


     // uvOffsetScale
    let uv_offset_scale = [0.0, 0.0, 1.0, 1.0];
    #[cfg(feature = "log")]
    println!("uv_offset_scale:{:?}", &uv_offset_scale[0..4]);
    gl.uniform4f(uniform_locations.get(&UV_OFFSET_SCALE),uv_offset_scale[0], uv_offset_scale[1], uv_offset_scale[2], uv_offset_scale[3]);

    gl.active_texture(WebGLRenderingContext::TEXTURE1);
    gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&image.src.bind));
    gl.uniform1i(uniform_locations.get(&TEXTURE), 1);

    // alpha
    #[cfg(feature = "log")]
    println!("alpha:{}", image.alpha);
    gl.uniform1f(uniform_locations.get(&ALPHA), image.alpha);

    if defines.clip_plane {
        #[cfg(feature = "log")]
        println!("by_overflow:{:?}", image.by_overflow);
        gl.uniform1f(uniform_locations.get(&CLIP_INDEICES), image.by_overflow as f32);
        gl.uniform1f(uniform_locations.get(&CLIP_INDEICES_SIZE), 1024.0);
        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&mgr.overflow_texture.texture));
        gl.uniform1i(uniform_locations.get(&CLIP_TEXTURE), 0);
    }

    //position
    gl.bind_buffer(
        WebGLRenderingContext::ARRAY_BUFFER,
        Some(&image_effect.positions_buffer),
    );

    let extend = &image.extend;
    let pad = 5.0;
    //如果shape_dirty， 更新定点顶点数据
    if image_effect.positions_dirty {
        let buffer = [
            -extend.x  - pad,
            -extend.y  - pad,
            image.z_depth, // left_top
            -extend.x  - pad,
            extend.y  + pad,
            image.z_depth, // left_bootom
            extend.x  + pad,
            extend.y  + pad,
            image.z_depth, // right_bootom
            extend.x  + pad,
            -extend.y  - pad,
            image.z_depth, // right_top
        ];

        #[cfg(feature = "log")]
        println!("position: {:?}", buffer);
        let buffer = unsafe { UnsafeTypedArray::new(&buffer) };
        js! {
            @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }
    }
    
    let positions_location = *(attr_locations.get(&POSITION).unwrap()) ;
    gl.vertex_attrib_pointer(positions_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(positions_location);

    //uvs
    gl.bind_buffer(
        WebGLRenderingContext::ARRAY_BUFFER,
        Some(&image_effect.uvs_buffer),
    );
    let uvs_location = *(attr_locations.get(&UV).unwrap()) ;
    gl.vertex_attrib_pointer(uvs_location, 2, WebGLRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(uvs_location);

    //index
    gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER ,Some(&image_effect.indeices_buffer),);

    #[cfg(feature = "log")]
    println!("is_opaque: {}", image.is_opaque);

    //draw
    gl.draw_elements(
        WebGLRenderingContext::TRIANGLES,
        6,
        WebGLRenderingContext::UNSIGNED_SHORT,
        0,
    );

    gl.disable_vertex_attrib_array(positions_location);
    gl.disable_vertex_attrib_array(uvs_location);
}