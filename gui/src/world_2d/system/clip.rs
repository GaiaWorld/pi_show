// 监听Sdf, Image, 和Word的创建和销毁事件， 创建或销毁对应的Effect

use std::rc::{Rc};
use std::cell::RefCell;
use std::f32::consts::PI;

use webgl_rendering_context::{WebGLRenderingContext, WebGLBuffer};
use stdweb::UnsafeTypedArray;

use wcs::component::{ComponentHandler, SingleModifyEvent};
use wcs::world::System;
use atom::Atom;

use world_2d::World2dMgr;
use world_2d::Overflow;
use render::engine::{get_uniform_location};


pub struct ClipSys(Rc<RefCell<ClipSysImpl>>);

impl ClipSys {
    pub fn init(component_mgr: &mut World2dMgr) -> Rc<ClipSys>{
        let r = Rc::new(ClipSys(Rc::new(RefCell::new(ClipSysImpl::new(component_mgr)))));
        component_mgr.overflow.handlers.register_handler(Rc::downgrade(&(r.clone() as Rc<ComponentHandler<Overflow, SingleModifyEvent, World2dMgr>>)));
        r
    }
}

impl System<(), World2dMgr> for ClipSys{
    fn run(&self, _e: &(), component_mgr: &mut World2dMgr){
        let mut borrow_mut = self.0.borrow_mut();
        if borrow_mut.dirty == false {
            return;
        }
        borrow_mut.dirty = false;

        for i in 0..8 {
            let p = &component_mgr.overflow.1[i];
            let extend = ((p[1].x - p[0].x) /2.0, (p[3].y - p[0].y) /2.0);
            borrow_mut.angles.push(0.0 * PI / 180.0); //旋转暂时为0， TODO
            borrow_mut.translate_scale.extend_from_slice(&[p[0].x + extend.0, p[0].y + extend.1, extend.0, extend.1]);
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

        // gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, Some(&component_mgr.overflow_texture.frambuffer));

        //view
        let arr: &[f32; 16] = component_mgr.view.as_ref();
        println!("clip view: {:?}", &arr[0..16]);
        gl.uniform_matrix4fv( uniform_locations.get(&VIEW), false, &arr[0..16] );

        //projection
        let arr: &[f32; 16] = component_mgr.projection.0.as_ref();
        println!("clip projection: {:?}", &arr[0..16]);
        gl.uniform_matrix4fv( uniform_locations.get(&PROJECTION), false, &arr[0..16]);

        println!("clip mesh_num: {:?}", 8.0);
        gl.uniform1f(uniform_locations.get(&MESH_NUM), 8.0);

        println!("clip angles: {:?}", borrow_mut.angles.as_slice());
        gl.uniform1fv(uniform_locations.get(&ANGLES), borrow_mut.angles.as_slice());
        println!("clip translate_scale: {:?}", borrow_mut.translate_scale.as_slice());
        gl.uniform4fv(uniform_locations.get(&TRANSLATE_SCALE), borrow_mut.translate_scale.as_slice());

        //position
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER,Some(&borrow_mut.positions_buffer));
        let position_location = *(attr_locations.get(&POSITION).unwrap()) ;
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

        gl.blend_func(WebGLRenderingContext::SRC_ALPHA, WebGLRenderingContext::ONE_MINUS_SRC_ALPHA);
        // gl.bind_framebuffer(WebGLRenderingContext::FRAMEBUFFER, None);
    }
}
 
impl ComponentHandler<Overflow, SingleModifyEvent, World2dMgr> for ClipSys{
    fn handle(&self, event: &SingleModifyEvent, _component_mgr: &mut World2dMgr){
        let SingleModifyEvent{field: _} = event;
        self.0.borrow_mut().dirty = true;
    }
}

pub struct ClipSysImpl {
    angles: Vec<f32>,
    translate_scale: Vec<f32>,
    positions_buffer: WebGLBuffer,
    index_buffer: WebGLBuffer,
    indeices_buffer: WebGLBuffer,
    program: u64,
    dirty: bool,
}

impl ClipSysImpl {
    pub fn new(mgr: &mut World2dMgr) -> ClipSysImpl{
        let gl = mgr.engine.gl.clone();
        let positions_buffer = gl.create_buffer().unwrap();
        let index_buffer = gl.create_buffer().unwrap();
        let indeices_buffer = gl.create_buffer().unwrap();
        
        let mut indexs: Vec<f32> = Vec::new();
        let mut indeices: Vec<u16> = Vec::new();
        let mut ps: Vec<f32> = Vec::new();
        for i in 0..8 {
            indexs.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);

            indeices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);

            ps.extend_from_slice(&[
                -1.0, -1.0, 0.0,
                -1.0,  1.0, 0.0,
                1.0,  1.0, 0.0,
                1.0, -1.0, 0.0
            ]);
        }

        let buffer = unsafe { UnsafeTypedArray::new(&ps) };
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER,Some(&positions_buffer));
        js! {
            console.log("position", @{&buffer});
            @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }

        let buffer = unsafe { UnsafeTypedArray::new(&indexs) };
        gl.bind_buffer(WebGLRenderingContext::ARRAY_BUFFER,Some(&index_buffer));
        js! {
            console.log("indexs", @{&buffer});
            @{&gl}.bufferData(@{WebGLRenderingContext::ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }

        let buffer = unsafe { UnsafeTypedArray::new(&indeices) };
        gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER,Some(&indeices_buffer));
        js! {
            console.log("indeices", @{&buffer});
            @{&gl}.bufferData(@{WebGLRenderingContext::ELEMENT_ARRAY_BUFFER}, @{buffer}, @{WebGLRenderingContext::STATIC_DRAW});
        }

        gl.bind_buffer(WebGLRenderingContext::ELEMENT_ARRAY_BUFFER, None);

        let program = create_program(mgr);

        ClipSysImpl {
            dirty: true,
            angles: Vec::new(),
            translate_scale: Vec::new(),
            program: program,
            positions_buffer: positions_buffer,
            index_buffer: index_buffer,
            indeices_buffer: indeices_buffer,
        }
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

    uniform_locations.insert( ANGLES.clone(), get_uniform_location(&gl,program, &ANGLES));

    uniform_locations.insert( TRANSLATE_SCALE.clone(), get_uniform_location(&gl,program, &TRANSLATE_SCALE));

    uniform_locations.insert( PROJECTION.clone(), get_uniform_location(&gl,program, &PROJECTION));

    uniform_locations.insert( VIEW.clone(), get_uniform_location(&gl,program, &VIEW));

    program_id
}

lazy_static! {
    static ref MESH_NUM: Atom = Atom::from("meshNum");
    static ref ANGLES: Atom = Atom::from("angles");
    static ref TRANSLATE_SCALE: Atom = Atom::from("translateScale");
    static ref POSITION: Atom = Atom::from("position");
    static ref MESH_INDEX: Atom = Atom::from("meshIndex");
    static ref VIEW: Atom = Atom::from("view");
    static ref PROJECTION: Atom = Atom::from("projection");
}