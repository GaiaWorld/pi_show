// 监听Overflow， 绘制裁剪纹理

use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;
use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share, Runner};
use ecs::idtree::{ IdTree};
use map::vecmap::VecMap;
use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, BlendFunc, CullMode, ShaderType, Pipeline};
use atom::Atom;

use component::user::*;
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::Engine;
use system::util::{cal_matrix, color_is_opaque, create_geometry};
use system::constant::{POSITION, ViewMatrix, ProjectionMatrix};

lazy_static! {
    static ref MESH_NUM: Atom = Atom::from("meshNum");
    static ref MESH_INDEX: Atom = Atom::from("meshIndex");
    static ref CLIP_RENDER: Atom = Atom::from("clip_render");
}

#[derive(Default)]
pub struct ClipSys<C: Context + Share>{
    shader_attr: Option<ShaderAttr>,
    mark: PhantomData<C>,
    dirty: bool,
}

struct ShaderAttr {
    geometry: Arc<(C as Context)::ContextGeometry>,
    pipeline: Arc<Pipeline>,
    ubos: HashMap<Atom, Arc<Uniforms>>,
}


impl<'a, C: Context + Share> Runner<'a> for ClipSys<C>{
    type ReadData = (
        &'a mut SingleCaseImpl<<C as Context>::ContextRenderTarget>,
        &'a mut SingleCaseImpl<Overflow>,
        &'a mut SingleCaseImpl<ProjectionMatrix>,
        &'a mut SingleCaseImpl<ViewMatrix>,
        &'a mut SingleCaseImpl<ClipUbo>
    );
    type WriteData = &'a mut SingleCaseImpl<Engine<C>>;
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        let gl = &mut write.gl;
        if self.dirty == false {
            return;
        }
        self.dirty = false;

        let shader_attr = match self.shader_attr {
            Some(r) => r,
            None => return,
        };

        // set_pipeline
        gl.set_pipeline(shader_attr.pipeline.clone());

        let geometry_ref = shader_attr.geometry.mark_mut();

        let (target, overflow, projection, view, clip_ubo) = read;

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
        geometry_ref.set_attribute(&POSITION.clone(), 3, positions.as_slice(), true);

        //draw
        gl.draw(geometry.clone(), &shader_attr.ubos);

        //设置clip_ubo TODO
        // clip_ubo
    }
    
    fn setup(&mut self, read: Self::ReadData, write: Self::WriteData){
        let engine = write;
        let pipeline = engine.create_pipeline(0, &BOX_VS_SHADER_NAME.clone(), &BOX_FS_SHADER_NAME.clone(), item.defines.list().as_slice(), self.rs.clone(), self.bs.clone(), self.ss.clone(), self.ds.clone());
        let mut geometry = create_geometry(&mut engine.gl, 96);

        let mut indexs: Vec<f32> = Vec::new();
        let mut indeices: Vec<u16> = Vec::new();

        for i in 0..8 {
            indexs.extend_from_slice(&[i as f32, i as f32, i as f32, i as f32]);
            indeices.extend_from_slice(&[4 * i + 0, 4 * i + 1, 4 * i + 2, 4 * i + 0, 4 * i + 2, 4 * i + 3]);
        }

        let geometry_ref = Arc::mark_mut(&mut geometry);
        geometry_ref.set_indices_short(indeices.as_slice());
        geometry_ref.set_attribute(&POSITION.clone(), 1, indexs.as_slice(), true);

        let (_, _, projection, view, _) = read;
        let mut ubo = Uniforms::new();
        let view: &[f32; 16] = view.0.as_ref();
        ubo.set_mat_4v(ViewMatrix, &view[0..16]);
        let projection: &[f32; 16] = projection.0.as_ref();
        ubo.set_mat_4v(ProjectionMatrix, &projection[0..16]);
        ubo.set_float_1(MESH_NUM, 8.0);

        let mut ubos = HashMap::default();
        ubos.insert(CLIP_RENDER.clone(), Arc::new(ubo));

        self.dirty = true;
        write.shader_attr = Some(ShaderAttr{
            geometry: geometry,
            pipeline: pipeline,
            ubos: ubos,
        });
    }
}

impl<'a, C: Context + Share> SingleCaseImpl<'a, Overflow, ModifyEvent> for ClipSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        self.dirty = true;
    }
}