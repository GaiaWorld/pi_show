/**
 *  渲染， 将渲染对象按照透明与不透明分类， 先渲染不透明物体， 在渲染透明物体， 不透明物体按照渲染管线的顺序渲染， 透明物体按照物体的深度顺序渲染
 */

use hal_core::{Context, Uniforms, RasterState, BlendState, StencilState, DepthState, BlendFunc, CullMode, ShaderType, Pipeline, Geometry};
use ecs::{SingleCaseImpl, Share};

use render::engine::Engine;

pub struct RenderSys<C: >{
    dirty: bool,
}

impl<'a, C: Context + Share> Runner<'a> for CharBlockSys<C>{
    type ReadData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    type WriteData = ();
    fn run(&mut self, read: Self::ReadData, _: Self::WriteData){
        let (render_objs, engine) = read;
        if self.dirty == false {
            return;
        }

        self.dirty = false;

        let mut transparent_list = Vec::new();
        let mut opacity_list = Vec::new();
        for item in render_objs.iter() {
            if item.1.is_opacity == true {
                opacity_list.push(OpacityOrd(item.1));
            }else {
                transparent_list.push(TransparentOrd(item.1));
            }
        }

        transparent_list.sort();
        opacity_list.sort();

        for obj in opacity_list.into_iter() {
            render(&engine.gl, obj);
        }

        for obj in transparent_list.into_iter() {
            render(&engine.gl, obj);
        }
    }
}

fn render<C: Context + Share>(gl: C, obj: &RenderObjs<C>){
    gl.set_pipeline(&obj.pipeline);
    gl.draw(&obj.geometry, &obj.ubos);
}

struct OpacityOrd<'a, C: Context + Share>(&RenderObjs<C>);

impl<'a, C: Context + Share> PartialOrd for OpacityOrd<'a, C> {
	fn partial_cmp(&self, other: &OpacityOrd<C>) -> Option<Ordering> {
        (self.pipeline.as_ref() as *const Pipeline as usize).partial_cmp(other.pipeline.as_ref() as *const Pipeline as usize);
	}
}

impl<'a, C: Context + Share> PartialEq for OpacityOrd{
	 fn eq(&self, other: &OpacityOrd) -> bool {
        (self.pipeline.as_ref() as *const Pipeline as usize).eq(other.pipeline.as_ref() as *const Pipeline as usize);
        self.z.eq(&other.z)
    }
}

impl<'a, C: Context + Share> Eq for OpacityOrd{}

impl<'a, C: Context + Share> Ord for OpacityOrd{
	fn cmp(&self, other: &OpacityOrd) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}

struct TransparentOrd<'a, C: Context + Share>(&RenderObjs<C>);

impl<'a, C: Context + Share> PartialOrd for TransparentOrd<'a, C> {
	fn partial_cmp(&self, other: &TransparentOrd<C>) -> Option<Ordering> {
		self.z_depth.partial_cmp(&other.z_depth)
	}
}

impl<'a, C: Context + Share> PartialEq for TransparentOrd<'a, C>{
	 fn eq(&self, other: &TransparentOrd<C>) -> bool {
        self.z_depth.eq(&other.z_depth)
    }
}

impl<'a, C: Context + Share> Eq for TransparentOrd<'a, C>{}

impl<'a, C: Context + Share> Ord for TransparentOrd<'a, C>{
	fn cmp(&self, other: &TransparentOrd<C>) -> Ordering {
        let r = self.partial_cmp(&other).unwrap();
        r
    }
}