use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::UnsafeTypedArray;

use atom::Atom;

use world_2d::World2dMgr;
use world_2d::component::char_block::{CharBlockDefines};
use world_2d::constant::*;
use component::color::Color;
use component::math::{Color as MathColor};
use render::engine::{Engine, get_uniform_location};

lazy_static! {
    static ref EXTEND: Atom = Atom::from("extend");
    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref COLOR: Atom = Atom::from("color");
    static ref COLOR_ANGLE: Atom = Atom::from("colorAngle");
    static ref DISTANCE: Atom = Atom::from("distance");
    static ref COLOR1: Atom = Atom::from("color1");
    static ref COLOR2: Atom = Atom::from("color2");
    static ref COLOR3: Atom = Atom::from("color3");
    static ref COLOR4: Atom = Atom::from("color4");
    static ref FONT_CLAMP: Atom = Atom::from("fontClamp");  // 0-1的小数，超过这个值即认为有字体，默认传0.75
    static ref SMOOT_HRANFE: Atom = Atom::from("smoothRange");
    static ref TEXTURE: Atom = Atom::from("texture");
}

// 初始化location
pub fn init_location(defines: &CharBlockDefines, engine: &mut Engine, program_id: u64) {
    let gl = engine.gl.clone();
    
    let program = engine.lookup_program_mut(program_id).unwrap();
    let uniform_locations = &mut program.uniform_locations;
    let attr_locations = &mut program.attr_locations;
    let program = &program.program;

    if uniform_locations.len() > 0 {
        return;
    }
    
    // position
    let position_location = gl.get_attrib_location(program, &POSITION) as u32;
    attr_locations.insert(POSITION.clone(), position_location);
    gl.vertex_attrib_pointer(position_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0);

    //uv
    let uv_location = gl.get_attrib_location(program, &UV) as u32;
    attr_locations.insert(UV.clone(), uv_location);
    gl.vertex_attrib_pointer(uv_location, 2, WebGLRenderingContext::FLOAT, false, 0, 0);
    
    //矩阵
    uniform_locations.insert(VIEW.clone(), get_uniform_location(&gl,program, &VIEW));
    uniform_locations.insert(PROJECTION.clone(), get_uniform_location(&gl,program, &PROJECTION));
    uniform_locations.insert(WORLD.clone(), get_uniform_location(&gl,program, &WORLD));

    // 与宏无关的uniform
    uniform_locations.insert(ALPHA.clone(), get_uniform_location(&gl,program, &ALPHA));
    uniform_locations.insert(EXTEND.clone(), get_uniform_location(&gl,program, &EXTEND));
    uniform_locations.insert(FONT_CLAMP.clone(), get_uniform_location(&gl,program, &FONT_CLAMP));
    uniform_locations.insert(SMOOT_HRANFE.clone(), get_uniform_location(&gl,program, &SMOOT_HRANFE));
    uniform_locations.insert(TEXTURE.clone(), get_uniform_location(&gl,program, &TEXTURE));
    
    if defines.color {
        uniform_locations.insert(COLOR.clone(), get_uniform_location(&gl,program, &COLOR));
    } else if defines.linear_color_gradient_2 {
        uniform_locations.insert(COLOR_ANGLE.clone(), get_uniform_location(&gl,program, &COLOR_ANGLE));
        uniform_locations.insert(DISTANCE.clone(), get_uniform_location(&gl,program, &DISTANCE));
        uniform_locations.insert(COLOR1.clone(), get_uniform_location(&gl,program, &COLOR1));
        uniform_locations.insert(COLOR2.clone(), get_uniform_location(&gl,program, &COLOR2));
    } else if defines.linear_color_gradient_4 {
        uniform_locations.insert(COLOR_ANGLE.clone(), get_uniform_location(&gl,program, &COLOR_ANGLE));
        uniform_locations.insert(DISTANCE.clone(),get_uniform_location(&gl,program, &DISTANCE));
        uniform_locations.insert(COLOR1.clone(), get_uniform_location(&gl,program,  &COLOR1));
        uniform_locations.insert(COLOR2.clone(), get_uniform_location(&gl,program, &COLOR2));
        uniform_locations.insert(COLOR3.clone(), get_uniform_location(&gl,program, &COLOR3));
        uniform_locations.insert(COLOR4.clone(), get_uniform_location(&gl,program, &COLOR4));
    }

    if defines.stroke {
        uniform_locations.insert( STROKE_SIZE.clone(), get_uniform_location(&gl,program, &STROKE_SIZE));
        uniform_locations.insert(STROKE_COLOR.clone(), get_uniform_location(&gl,program, &STROKE_COLOR));
    }
    if defines.clip_plane {
        uniform_locations.insert(CLIP_INDEICES.clone(), get_uniform_location(&gl,program, &CLIP_INDEICES));
        uniform_locations.insert(CLIP_TEXTURE.clone(), get_uniform_location(&gl,program, &CLIP_TEXTURE));
        uniform_locations.insert(CLIP_INDEICES_SIZE.clone(), get_uniform_location(&gl,program, &CLIP_INDEICES_SIZE));
    }
}

pub fn update(mgr: &mut World2dMgr, effect_id: usize){
    let attribute = fill_attribute_index(effect_id, mgr);
    let char_block_effect = mgr.char_block_effect._group.get(effect_id);
    let gl = &mgr.engine.gl;

    let program = mgr.engine.lookup_program(char_block_effect.program).unwrap();
    let attr_locations = &program.attr_locations;
    let program = &program.program;

    // use_program
    gl.use_program(Some(program));

    //position
    gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&char_block_effect.positions_buffer));
    let buffer = unsafe { UnsafeTypedArray::new(attribute.positions.as_ref()) };
    js! {
        @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
    }
    let position_location = *(attr_locations.get(&POSITION).unwrap()) ;
    gl.vertex_attrib_pointer(position_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0,);
    gl.enable_vertex_attrib_array(position_location);
    
    debug_println!("update position location: {:?}, buffer = {:?}, data: {:?}", position_location, &char_block_effect.positions_buffer, attribute.positions);

    //uv
    gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&char_block_effect.uvs_buffer));
    //如果shape_dirty， 更新uv数据
    let buffer = unsafe { UnsafeTypedArray::new(attribute.uvs.as_ref()) };
    js! {
        @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
    }
    let uv_location = *(attr_locations.get(&UV).unwrap());
    gl.vertex_attrib_pointer(uv_location, 2, WebGLRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(uv_location);
    
    debug_println!("update uv location: {:?}, buffer = {:?}, data: {:?}", uv_location, &char_block_effect.uvs_buffer, attribute.uvs);
    
    //index
    gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&char_block_effect.indeices_buffer));
    let buffer = unsafe { UnsafeTypedArray::new(attribute.indeices.as_ref()) };
    js! {
        @{&gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
    }

    debug_println!("update indeices buffer = {:?}, data: {:?}", &char_block_effect.indeices_buffer, attribute.indeices);
}

//更新uniform和buffer， 并渲染
pub fn render(mgr: &mut World2dMgr, effect_id: usize) {
    let char_block_effect = mgr.char_block_effect._group.get(effect_id);
    let char_block = mgr.char_block._group.get(char_block_effect.parent);

    let defines = mgr.char_block_effect.defines._group.get(char_block_effect.defines);
    debug_println!("text defines---------------------------{:?}", defines);

    let gl = &mgr.engine.gl;

    debug_println!("program-------------------{}", char_block_effect.program);
    let program = mgr.engine.lookup_program(char_block_effect.program).unwrap();
    let uniform_locations = &program.uniform_locations;
    let attr_locations = &program.attr_locations;
    let program = &program.program;

    // use_program
    gl.use_program(Some(program));

    // view
    debug_println!("charblock view----------------{:?}", &mgr.view);
    let arr: &[f32; 16] = mgr.view.as_ref();
    gl.uniform_matrix4fv(uniform_locations.get(&VIEW), false, &arr[0..16]);

    // projection
    debug_println!("charblock projection----------------{:?}", &mgr.projection.0);
    let arr: &[f32; 16] = mgr.projection.0.as_ref();
    gl.uniform_matrix4fv(uniform_locations.get(&PROJECTION), false, &arr[0..16]);

    // world_matrix
    debug_println!("charblock world_matrix----------------{:?}", &char_block.world_matrix.0);
    let arr: &[f32; 16] = char_block.world_matrix.0.as_ref();
    gl.uniform_matrix4fv(uniform_locations.get(&WORLD), false, &arr[0..16]);

    //extend
    debug_println!("charblock extend: {:?}", char_block.extend);
    gl.uniform2f(uniform_locations.get(&EXTEND), char_block.extend.0, char_block.extend.1);

    // alpha
    debug_println!("charblock alpha: {:?}", char_block.alpha);
    gl.uniform1f(uniform_locations.get(&ALPHA), char_block.alpha);

    // smooth_range
    debug_println!("charblock smoothRange: {:?}", char_block_effect.smooth_range);
    gl.uniform1f(uniform_locations.get(&SMOOT_HRANFE), char_block_effect.smooth_range);

    // smooth_range
    debug_println!("charblock fontClamp: {:?}", char_block_effect.font_clamp);
    gl.uniform1f(uniform_locations.get(&FONT_CLAMP), char_block_effect.font_clamp);

    if defines.stroke {
        //设置strokeSize
        debug_println!("stroke_size:{:?}", char_block.stroke_size);
        gl.uniform1f(uniform_locations.get(&STROKE_SIZE), char_block.stroke_size);

        //设置strokeColor
        debug_println!("stroke_color:{:?}", char_block.stroke_color);
        gl.uniform4f(uniform_locations.get(&STROKE_COLOR), char_block.stroke_color.r, char_block.stroke_color.g, char_block.stroke_color.b, char_block.stroke_color.a);
    }
    if defines.clip_plane {
        debug_println!("by_overflow:{:?}", char_block.by_overflow);
        gl.uniform1f(uniform_locations.get(&CLIP_INDEICES), char_block.by_overflow as f32);
        gl.uniform1f(uniform_locations.get(&CLIP_INDEICES_SIZE), 1024.0);

        gl.active_texture(WebGLRenderingContext::TEXTURE0);
        gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&mgr.overflow_texture.texture));
        gl.uniform1i(uniform_locations.get(&CLIP_TEXTURE), 0);
    }

    gl.active_texture(WebGLRenderingContext::TEXTURE2);
    gl.bind_texture(WebGLRenderingContext::TEXTURE_2D, Some(&char_block.sdf_font.texture().bind));
    gl.uniform1i(uniform_locations.get(&TEXTURE), 2);

    if char_block_effect.is_shadow {
        match &char_block.shadow {
            Some(shadow) => {
                debug_println!("shadow_color: {:?}", shadow.color);
                gl.uniform4f(uniform_locations.get(&COLOR), shadow.color.r, shadow.color.g, shadow.color.b, shadow.color.a);
            },
            None => debug_println!("it not have shadow"),
        }
    }else {
        match &char_block.color {
            Color::RGB(color) | Color::RGBA(color) => {
                // color
                debug_println!("color: {:?}", color);
                gl.uniform4f(uniform_locations.get(&COLOR), color.r, color.g, color.b, color.a);
            },
            Color::LinearGradient(color) => {
                //colorAngle
                gl.uniform1f(uniform_locations.get(&COLOR_ANGLE), color.direction);

                if defines.linear_color_gradient_2 {
                    //distance
                    gl.uniform2f( uniform_locations.get(&DISTANCE), color.list[0].position, color.list[1].position);

                    //color1
                    let color1 = &color.list[0].rgba;
                    gl.uniform4f( uniform_locations.get(&COLOR1), color1.r, color1.g, color1.b, color1.a);

                    //color2
                    let color2 = &color.list[1].rgba;
                    gl.uniform4f(uniform_locations.get(&COLOR2), color2.r, color2.g, color2.b, color2.a);
                } else {
                    let mut distances = [0.0, 100.0, 100.0, 100.0];
                    let default_color = MathColor(cg::color::Color::new(1.0, 1.0, 1.0, 1.0));
                    let mut colors = [&default_color, &default_color, &default_color, &default_color];
                    let mut i = 0;
                    for k in color.list.iter() {
                        if i > 3 {
                            break;
                        }
                        distances[i] = k.position;
                        colors[i] = &k.rgba;
                        i += 1;
                    }
                    gl.uniform4f( uniform_locations.get(&DISTANCE), distances[0],distances[1],distances[2],distances[3]);

                    //color1
                    gl.uniform4f(uniform_locations.get(&COLOR1), colors[0].r, colors[0].g, colors[0].b, colors[0].a);

                    //color2
                    gl.uniform4f(uniform_locations.get(&COLOR2), colors[1].r, colors[1].g, colors[1].b, colors[1].a);

                    //color3
                    gl.uniform4f( uniform_locations.get(&COLOR3), colors[2].r, colors[2].g, colors[2].b, colors[2].a);

                    //color4
                    gl.uniform4f(uniform_locations.get(&COLOR4), colors[3].r, colors[3].g, colors[3].b, colors[3].a);
                }
            },
            _ => panic!("color type error"),
        };
    }

    //position
    gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&char_block_effect.positions_buffer));
    let position_location = *(attr_locations.get(&POSITION).unwrap()) ;
    gl.vertex_attrib_pointer(position_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0,);
    gl.enable_vertex_attrib_array(position_location);
    
    debug_println!("position: location = {:?}, buffer = {:?}", position_location, &char_block_effect.positions_buffer);
    
    //uv
    gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&char_block_effect.uvs_buffer));
    let uv_location = *(attr_locations.get(&UV).unwrap());
    gl.vertex_attrib_pointer(uv_location, 2, WebGLRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(uv_location);
    
    debug_println!("uv: location = {:?}, buffer = {:?}", uv_location, &char_block_effect.uvs_buffer);

    //index
    gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&char_block_effect.indeices_buffer));
  
    //draw
    debug_println!("draw, indeices, buffer = {:?}, len = {:?}", &char_block_effect.indeices_buffer, char_block_effect.indeices_len);
    gl.draw_elements(WebGLRenderingContext::TRIANGLES, char_block_effect.indeices_len as i32, WebGLRenderingContext::UNSIGNED_SHORT, 0);

    gl.disable_vertex_attrib_array(position_location);
}



// 填充顶点 uv 索引
fn fill_attribute_index(effect_id: usize, mgr: &mut World2dMgr) -> Attribute {
    let char_block_effect = mgr.char_block_effect._group.get_mut(effect_id);
    let char_block= mgr.char_block._group.get(char_block_effect.parent);
    let sdf_font = &char_block.sdf_font; //TODO

    println!("charblock---------------------{:?}", char_block);

    let mut positions: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indeices: Vec<u16> = Vec::new();
    let mut i = 0;
    // let line_height = sdf_font.line_height;
    let mut offset = char_block.offset.clone();
    offset.1 -= (char_block.line_height - char_block.font_size)/2.0;

    // 如果是阴影， 会偏移
    if char_block_effect.is_shadow {
        match &char_block.shadow {
            Some(shadow) => {
                offset.1 -= shadow.v; 
                offset.0 -= shadow.h;
            },
            None => debug_println!("it not have shadow"),
        }
    }

    for c in char_block.chars.iter() {
        let glyph = match sdf_font.glyph_info(c.value, char_block.font_size) {
            Some(r) => r,
            None => continue,
        };
        // let pos = &c.pos;
        let pos = (c.pos.x - offset.0, c.pos.y - offset.1);
        // let pos = (c.pos.x - 450.0, c.pos.y - 0.0);

        // extend.x += glyph.width/2.0;
        // extend.y += glyph.height/2.0;

        let max_y = pos.1 + char_block.font_size - glyph.oy;
        println!("char_block.font_size: {}, glyph.oy: {}", char_block.font_size, glyph.oy);
        positions.extend_from_slice(&[
            pos.0 + glyph.ox,                max_y,                 char_block.z_depth,
            pos.0 + glyph.ox,                max_y + glyph.height,  char_block.z_depth,
            glyph.width + pos.0 + glyph.ox,  max_y + glyph.height,  char_block.z_depth,
            glyph.width + pos.0 + glyph.ox,  max_y,                 char_block.z_depth,
        ]);

        let (v_min, v_max) = (1.0 - glyph.v_max, 1.0 - glyph.v_min);
        uvs.extend_from_slice(&[
            glyph.u_min, v_min,
            glyph.u_min, v_max,
            glyph.u_max, v_max,
            glyph.u_max, v_min,
        ]);

        indeices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
        i += 1;
    }

    char_block_effect.indeices_len = indeices.len() as u16;

    Attribute {
        positions: positions,
        uvs: uvs,
        indeices: indeices
    }
}

struct Attribute {
    positions: Vec<f32>,
    uvs: Vec<f32>,
    indeices: Vec<u16>,
}