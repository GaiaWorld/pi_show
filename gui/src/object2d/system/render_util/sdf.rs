use webgl_rendering_context::{WebGLRenderingContext};
use stdweb::UnsafeTypedArray;

use atom::Atom;

use object2d::Object2dMgr;
use object2d::component::sdf::{SdfDefines};
use generic_component::color::Color;
use generic_component::math::Color as MathColor;
use render::engine::{Engine, get_uniform_location};

lazy_static! {
    static ref POSITION: Atom = Atom::from("position");
    static ref WORLD_VIEW_PROJECTION: Atom = Atom::from("worldViewProjection");
    static ref CENTER: Atom = Atom::from("center");
    static ref BLUR: Atom = Atom::from("blur");
    static ref EXTEND: Atom = Atom::from("extend");
    static ref ALPHA: Atom = Atom::from("alpha");
    static ref SCREEN_SIZE: Atom = Atom::from("screenSize");
    static ref ANGLE: Atom = Atom::from("angle");
    static ref RADIUS: Atom = Atom::from("radius");
    static ref STROKE_SIZE: Atom = Atom::from("strokeSize");
    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref COLOR: Atom = Atom::from("color");
    static ref COLOR_ANGLE: Atom = Atom::from("colorAngle");
    static ref DISTANCE: Atom = Atom::from("distance");
    static ref COLOR1: Atom = Atom::from("color1");
    static ref COLOR2: Atom = Atom::from("color2");
    static ref COLOR3: Atom = Atom::from("color3");
    static ref COLOR4: Atom = Atom::from("color4");
    static ref SIZE_TYPE: Atom = Atom::from("sizeType");
    static ref CLIP_INDEICES: Atom = Atom::from("clipIndices");
    static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
}

// 初始化location
pub fn init_location(defines: &SdfDefines, engine: &mut Engine, program_id: u64) {
    let gl = engine.gl.clone();

    let program = engine.lookup_program_mut(program_id).unwrap();
    let uniform_locations = &mut program.uniform_locations;
    let attr_locations = &mut program.attr_locations;
    let program = &program.program;

    if uniform_locations.len() > 0 {
        return;
    }
    
    let position_location = gl.get_attrib_location(program, &POSITION) as u32;
    attr_locations.insert(
        POSITION.clone(),
        position_location,
    );
    gl.vertex_attrib_pointer(position_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0);

    uniform_locations.insert(
        WORLD_VIEW_PROJECTION.clone(),
        get_uniform_location(&gl,program, &WORLD_VIEW_PROJECTION),
    );
    uniform_locations.insert(
        CENTER.clone(),
        get_uniform_location(&gl,program, &CENTER),
    );
    uniform_locations.insert(
        BLUR.clone(),
        get_uniform_location(&gl,program, &BLUR),
    );
    uniform_locations.insert(
        EXTEND.clone(),
        get_uniform_location(&gl,program, &EXTEND),
    );
    uniform_locations.insert(
        ALPHA.clone(),
        get_uniform_location(&gl,program, &ALPHA),
    );
    uniform_locations.insert(
        SCREEN_SIZE.clone(),
        get_uniform_location(&gl,program, &SCREEN_SIZE),
    );
    uniform_locations.insert(
        ANGLE.clone(),
        get_uniform_location(&gl,program, &ANGLE),
    );

    if defines.sdf_rect {
        uniform_locations.insert(
            RADIUS.clone(),
            get_uniform_location(&gl,program, &RADIUS),
        );
    }
    if defines.stroke {
        uniform_locations.insert(
            STROKE_SIZE.clone(),
            get_uniform_location(&gl,program, &STROKE_SIZE),
        );
        uniform_locations.insert(
            STROKE_COLOR.clone(),
            get_uniform_location(&gl,program, &STROKE_COLOR),
        );
    }
    if defines.clip_plane {
        uniform_locations.insert(
            CLIP_INDEICES.clone(),
            get_uniform_location(&gl,program, &CLIP_INDEICES),
        );

        uniform_locations.insert(
            CLIP_TEXTURE.clone(),
            get_uniform_location(&gl,program, &CLIP_TEXTURE),
        );
        uniform_locations.insert(
            CLIP_INDEICES_SIZE.clone(),
            get_uniform_location(&gl,program, &CLIP_INDEICES_SIZE)
                ,
        );
    }

    if defines.color {
        uniform_locations.insert(
            COLOR.clone(),
            get_uniform_location(&gl,program, &COLOR),
        );
    } else if defines.linear_color_gradient_2 {
        uniform_locations.insert(
            COLOR_ANGLE.clone(),
            get_uniform_location(&gl,program, &COLOR_ANGLE),
        );
        uniform_locations.insert(
            DISTANCE.clone(),
            get_uniform_location(&gl,program, &DISTANCE),
        );
        uniform_locations.insert(
            COLOR1.clone(),
            get_uniform_location(&gl,program, &COLOR1),
        );
        uniform_locations.insert(
            COLOR2.clone(),
            get_uniform_location(&gl,program, &COLOR2),
        );
    } else if defines.linear_color_gradient_4 {
        uniform_locations.insert(
            COLOR_ANGLE.clone(),
            get_uniform_location(&gl,program, &COLOR_ANGLE),
        );
        uniform_locations.insert(
            DISTANCE.clone(),
            get_uniform_location(&gl,program, &DISTANCE),
        );
        uniform_locations.insert(
            COLOR1.clone(),
            get_uniform_location(&gl,program, &COLOR1),
        );
        uniform_locations.insert(
            COLOR2.clone(),
            get_uniform_location(&gl,program, &COLOR2),
        );
        uniform_locations.insert(
            COLOR3.clone(),
            get_uniform_location(&gl,program, &COLOR3),
        );
        uniform_locations.insert(
            COLOR4.clone(),
            get_uniform_location(&gl,program, &COLOR4),
        );
    } else if defines.ellipse_color_gradient {
        uniform_locations.insert(
            SIZE_TYPE.clone(),
            get_uniform_location(&gl,program, &SIZE_TYPE),
        );
        uniform_locations.insert(
            DISTANCE.clone(),
            get_uniform_location(&gl,program, &DISTANCE),
        );
        uniform_locations.insert(
            COLOR1.clone(),
            get_uniform_location(&gl,program, &COLOR1),
        );
        uniform_locations.insert(
            COLOR2.clone(),
            get_uniform_location(&gl,program, &COLOR2),
        );
        uniform_locations.insert(
            COLOR3.clone(),
            get_uniform_location(&gl,program, &COLOR3),
        );
        uniform_locations.insert(
            COLOR4.clone(),
            get_uniform_location(&gl,program, &COLOR4),
        );
    }
}

//更新uniform和buffer， 并渲染
pub fn render(mgr: &mut Object2dMgr, effect_id: usize) {
    let sdf_effect = mgr.sdf_effect._group.get(effect_id);
    let sdf = mgr.sdf._group.get(sdf_effect.sdf_id);

    let defines = mgr.sdf_effect.defines._group.get(effect_id);

    let gl = &mgr.engine.gl;

    let program = mgr.engine.lookup_program(sdf_effect.program).unwrap();
    let uniform_locations = &program.uniform_locations;
    let attr_locations = &program.attr_locations;
    let program = &program.program;

    // use_program
    gl.use_program(Some(program));
    gl.viewport(0,0,1000,1000);
    

    //设置worldViewProjection
    js! {
        console.log("world_view", @{&(*mgr.world_view)});
    }
    gl.uniform_matrix4fv(
        uniform_locations.get(&WORLD_VIEW_PROJECTION),
        false,
        &(*mgr.world_view),
    );

    //blur
    js! {
        console.log("blur", 1.0);
    }
    // gl.uniform1f(uniform_locations.get(&BLUR), 1.0);

    //extend
    js! {
        console.log("extent", @{sdf.extend.x}, @{sdf.extend.y});
    }
    gl.uniform2f(uniform_locations.get(&EXTEND), sdf.extend.x, sdf.extend.y);

    // alpha
    gl.uniform1f(uniform_locations.get(&ALPHA), sdf.alpha);

    // screenSize
    gl.uniform2f(
        uniform_locations.get(&SCREEN_SIZE),
        mgr.width,
        mgr.height,
    );
    js!{console.log("SCREEN_SIZE", @{mgr.width}, @{mgr.height})}

    //angle
    js!{console.log("ANGLE", 0)}
    gl.uniform1f(uniform_locations.get(&ANGLE), 0.0); //TODO

    //set_uniforms
    if defines.sdf_rect {
        //设置radius
        gl.uniform1f(uniform_locations.get(&RADIUS), sdf.radius);
    }
    if defines.stroke {
        //设置strokeSize
        gl.uniform1f(uniform_locations.get(&STROKE_SIZE), sdf.border_size);

        //设置strokeColor
        gl.uniform4f(uniform_locations.get(&STROKE_COLOR), sdf.border_color.r, sdf.border_color.g, sdf.border_color.b, sdf.border_color.a);
    }
    if defines.clip_plane {
        //TODO
        panic!("ccccccccccccccccccccccccc");
        // uniform float clipIndices;
        // uniform sampler2D clipTexture;
        // uniform float clipTextureSize;

        // arr.push(SDF_CLIP_PLANE.clone());
    }

    match &sdf.color {
        Color::RGB(color) | Color::RGBA(color) => {
            js!{console.log("color", @{color.r}, @{color.g}, @{color.b}, @{color.a})}
            // color
            gl.uniform4f(uniform_locations.get(&COLOR), color.r, color.g, color.b, color.a,
            );
        }
        Color::LinearGradient(color) => {
            //colorAngle
            gl.uniform1f(uniform_locations.get(&COLOR_ANGLE), color.direction);

            if defines.linear_color_gradient_2 {
                //distance
                gl.uniform2f(
                    uniform_locations.get(&DISTANCE),
                    color.list[0].position,
                    color.list[1].position,
                );

                //color1
                let color1 = &color.list[0].rgba;
                gl.uniform4f(
                    uniform_locations.get(&COLOR1),
                    color1.r,
                    color1.g,
                    color1.b,
                    color1.a,
                );

                //color2
                let color2 = &color.list[1].rgba;
                gl.uniform4f(
                    uniform_locations.get(&COLOR2),
                    color2.r,
                    color2.g,
                    color2.b,
                    color2.a,
                );
            } else {
                let mut distances = [0.0, 100.0, 100.0, 100.0];
                let default_color = MathColor(cg::color::Color::new(1.0, 1.0, 1.0, 1.0));
                let mut colors = [
                    &default_color,
                    &default_color,
                    &default_color,
                    &default_color,
                ];
                let mut i = 0;
                for k in color.list.iter() {
                    if i > 3 {
                        break;
                    }
                    distances[i] = k.position;
                    colors[i] = &k.rgba;
                    i += 1;
                }
                gl.uniform4f(
                    uniform_locations.get(&DISTANCE),
                    distances[0],
                    distances[1],
                    distances[2],
                    distances[3],
                );

                //color1
                gl.uniform4f(
                    uniform_locations.get(&COLOR1),
                    colors[0].r,
                    colors[0].g,
                    colors[0].b,
                    colors[0].a,
                );

                //color2
                gl.uniform4f(
                    uniform_locations.get(&COLOR2),
                    colors[1].r,
                    colors[1].g,
                    colors[1].b,
                    colors[1].a,
                );

                //color3
                gl.uniform4f(
                    uniform_locations.get(&COLOR3),
                    colors[2].r,
                    colors[2].g,
                    colors[2].b,
                    colors[2].a,
                );

                //color4
                gl.uniform4f(
                    uniform_locations.get(&COLOR4),
                    colors[3].r,
                    colors[3].g,
                    colors[3].b,
                    colors[3].a,
                );
            }
        }
        Color::RadialGradient(_color) => {
            //TODO
            panic!("color type error");
            // uniform float sizeType;
            // uniform vec4 distance;
            // uniform vec4 color1;
            // uniform vec4 color2;
            // uniform vec4 color3;
            // uniform vec4 color4;
        }
    }

    gl.uniform2f(
        uniform_locations.get(&CENTER),
        sdf.center.x,
        sdf.center.y,
    );

    //position
    gl.bind_buffer(
        WebGLRenderingContext::ARRAY_BUFFER,
        Some(&sdf_effect.positions_buffer),
    );
    //如果shape_dirty， 更新定点顶点数据
    if sdf_effect.positions_dirty {
        let bound_box = &sdf.bound_box;
        let buffer = [
            bound_box.min.x,
            bound_box.min.y,
            sdf.z_depth, // left_top
            bound_box.min.x,
            bound_box.max.y,
            sdf.z_depth, // left_bootom
            bound_box.max.x,
            bound_box.max.y,
            sdf.z_depth, // right_bootom
            bound_box.max.x,
            bound_box.min.y,
            sdf.z_depth, // right_top
        ];

        let buffer = unsafe { UnsafeTypedArray::new(&buffer) };
        js! {
            console.log("position", @{&buffer});
            @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }
    }
    
    let position_location = *(attr_locations.get(&POSITION).unwrap()) ;

    gl.vertex_attrib_pointer(
        position_location,
        3,
        WebGLRenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(position_location);

    //index
    gl.bind_buffer(
        WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(&sdf_effect.indeices_buffer),
    );

    js! {
        console.log("draw_elements-------------------");
    }
    //draw
    gl.draw_elements(
        WebGLRenderingContext::TRIANGLES,
        6,
        WebGLRenderingContext::UNSIGNED_SHORT,
        0,
    );

    js! {
        console.log("draw_elements-------------------end");
    }

    gl.disable_vertex_attrib_array(position_location);
}