// 监听Sdf, Image, 和Word的创建和销毁事件， 创建或销毁对应的Effect

// use std::sync::{Arc};
// use std::cell::RefCell;

use webgl_rendering_context::{WebGLRenderingContext, WebGLBuffer};
use stdweb::UnsafeTypedArray;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};

// use wcs::component::{ComponentHandler, SingleModifyEvent};
// use wcs::world::System;
// use atom::Atom;

// use world_2d::World2dMgr;
// use world_2d::Overflow;
// use render::engine::{get_uniform_location};

use single::{OverflowClip, ProjectionMatrix, ViewMatrix};
use render::RenderTarget;

#[derive(Default)]
pub struct ClipSys{
    positions_buffer: WebGLBuffer,
    index_buffer: WebGLBuffer,
    indeices_buffer: WebGLBuffer,
    program: u64,
    dirty: bool,
}

impl ClipSys {
    pub fn new(gl: &WebGLRenderingContext) -> ClipSys{
        let positions_buffer = gl.create_buffer().unwrap();
        let index_buffer = gl.create_buffer().unwrap();
        let indeices_buffer = gl.create_buffer().unwrap();
        
        let mut indexs: Vec<f32> = Vec::new();
        let mut indeices: Vec<u16> = Vec::new();
        // let mut ps: Vec<f32> = Vec::new();
        for i in 0..8 {
            indexs.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);

            indeices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);

            // ps.extend_from_slice(&[
            //     -1.0, -1.0, 0.0,
            //     -1.0,  1.0, 0.0,
            //     1.0,  1.0, 0.0,
            //     1.0, -1.0, 0.0
            // ]);
        }

        debug_println!("clip indexs---------------------------{:?}", indexs);
        let buffer = unsafe { UnsafeTypedArray::new(&indexs) };
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER,Some(&index_buffer));
        js! {
            @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }

        debug_println!("clip indeices---------------------------{:?}", indeices);
        let buffer = unsafe { UnsafeTypedArray::new(&indeices) };
        gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&indeices_buffer));
        js! {
            @{&gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }

        gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, None);

        let program = create_program(mgr);

        ClipSys {
            dirty: true,
            program: program,
            positions_buffer: positions_buffer,
            index_buffer: index_buffer,
            indeices_buffer: indeices_buffer,
        }
    }
}

impl<'a> Runner<'a> for ClipSys{
    type ReadData = (
        &'a SingleCaseImpl<RenderTarget>,
        &'a SingleCaseImpl<OverflowClip>,
        &'a SingleCaseImpl<ProjectionMatrix>,
        &'a SingleCaseImpl<ViewMatrix>
    );
    type WriteData = ();
    fn run(&mut self, read: Self::ReadData, _write: Self::WriteData){
        if self.dirty == false {
            return;
        }
        self.dirty = false;

        let (target, overflow, projection, view) = read;
        let mut positions = [0.0; 96];
        for i in 0..8 {
            let p = &overflow.clip[i];

            positions[i * 12 + 0] = p[0].x;
            positions[i * 12 + 1] = p[0].y;
            positions[i * 12 + 2] = 0.0;

            positions[i * 12 + 3] = p[2].x;
            positions[i * 12 + 4] = p[2].y;
            positions[i * 12 + 5] = 0.0;

            positions[i * 12 + 6] = p[3].x;
            positions[i * 12 + 7] = p[3].y;
            positions[i * 12 + 8] = 0.0;

            positions[i * 12 + 9] = p[1].x;
            positions[i * 12 + 10] = p[1].y;
            positions[i * 12 + 11] = 0.0;
        }

        let program = component_mgr.engine.lookup_program(borrow_mut.program).unwrap();
        let uniform_locations = &program.uniform_locations;
        let attr_locations = &program.attr_locations;
        let program = &program.program;
        let gl = component_mgr.engine.gl.clone();

        // use_program
        gl.use_program(Some(program));

        gl.blend_func(WebGLRenderingContext::ONE, WebGLRenderingContext::ONE);
        gl.enable(WebGLRenderingContext::BLEND);

        gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, Some(&component_mgr.overflow_texture.frambuffer));

        gl.clear(WebGLRenderingContext::COLOR_BUFFER_BIT | WebGLRenderingContext::DEPTH_BUFFER_BIT);

        //view
        let arr: &[f32; 16] = component_mgr.view.as_ref();
        debug_println!("clip view: {:?}", &arr[0..16]);
        gl.uniform_matrix4fv( uniform_locations.get(&VIEW), false, &arr[0..16] );

        //projection
        let arr: &[f32; 16] = component_mgr.projection.0.as_ref();
        debug_println!("clip projection: {:?}", &arr[0..16]);
        gl.uniform_matrix4fv( uniform_locations.get(&PROJECTION), false, &arr[0..16]);

        debug_println!("clip mesh_num: {:?}", 8.0);
        gl.uniform1f(uniform_locations.get(&MESH_NUM), 8.0);

        //position
        debug_println!("clip positions---------------------------{:?}", &positions[0..96]);
        let buffer = unsafe { UnsafeTypedArray::new(&positions) };
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER,Some(&borrow_mut.positions_buffer));
        let position_location = *(attr_locations.get(&POSITION).unwrap()) ;
        js! {
            @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{&buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }
        gl.vertex_attrib_pointer(position_location,3,WebGLRenderingContext::FLOAT,false,0,0,);
        gl.enable_vertex_attrib_array(position_location);

        //meshIndex
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER, Some(&borrow_mut.index_buffer));
        let mesh_index_location = *(attr_locations.get(&MESH_INDEX).unwrap()) ;
        gl.vertex_attrib_pointer(mesh_index_location,1,WebGLRenderingContext::FLOAT,false,0,0);
        gl.enable_vertex_attrib_array(mesh_index_location);

        //index
        gl.bind_buffer( WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&borrow_mut.indeices_buffer));

        //draw
        gl.draw_elements(WebGLRenderingContext::TRIANGLES, 48, WebGLRenderingContext::UNSIGNED_SHORT, 0);
        debug_println!("clip draw_elements end---------------------------");

        gl.blend_func(WebGLRenderingContext::SRC_ALPHA, WebGLRenderingContext::ONE_MINUS_SRC_ALPHA);
        gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, None);
    }
}

impl<'a> SingleCaseListener<'a, OverflowClip, CreateEvent> for ClipSys{
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, _event: &CreateEvent, _read: Self::ReadData, _write: Self::WriteData){
        self.dirty = true;
    }
}

// 初始化location
pub fn create_program(component_mgr: &mut World2dMgr) -> u64 {
    let gl = component_mgr.engine.gl.clone();
    let program = component_mgr.engine.create_program(
        component_mgr.shader_store.get(&component_mgr.clip_shader.vs).unwrap(),
        component_mgr.shader_store.get(&component_mgr.clip_shader.fs).unwrap(),
        &Vec::<Atom>::new()
    );

    let program_id = match program {
        Ok(v) => v,
        Err(s) => {
            println!("create_program error: {:?}", s);
            return 0;
        },
    };

    let program = component_mgr.engine.lookup_program_mut(program_id).unwrap();
    let uniform_locations = &mut program.uniform_locations;
    let attr_locations = &mut program.attr_locations;
    let program = &program.program;
    
    let position_location = gl.get_attrib_location(program, &POSITION) as u32;
    attr_locations.insert(
        POSITION.clone(),
        position_location,
    );
    gl.vertex_attrib_pointer(position_location, 3, WebGLRenderingContext::FLOAT, false, 0, 0);

    let mesh_index_location = gl.get_attrib_location(program, &MESH_INDEX) as u32;
    attr_locations.insert(
        MESH_INDEX.clone(),
        mesh_index_location,
    );
    gl.vertex_attrib_pointer(mesh_index_location, 1, WebGLRenderingContext::FLOAT, false, 0, 0);

    uniform_locations.insert( MESH_NUM.clone(), get_uniform_location(&gl,program, &MESH_NUM));

    uniform_locations.insert( PROJECTION.clone(), get_uniform_location(&gl,program, &PROJECTION));

    uniform_locations.insert( VIEW.clone(), get_uniform_location(&gl,program, &VIEW));

    program_id
}

lazy_static! {
    static ref MESH_NUM: Atom = Atom::from("meshNum");
    static ref POSITION: Atom = Atom::from("position");
    static ref MESH_INDEX: Atom = Atom::from("meshIndex");
    static ref VIEW: Atom = Atom::from("view");
    static ref PROJECTION: Atom = Atom::from("projection");
}