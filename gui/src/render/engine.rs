/**
 *  对HalContext的封装， 并管理gl资源
*/
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::cell::RefCell;

use atom::Atom;
use ecs::StdCell;
use hash::{DefaultHasher, XHashMap};
use res::{Res, ResMap, ResMgr};
use share::Share;

use component::calc::*;
use component::user::CgColor;
use hal_core::*;
use render::res::*;
use system::util::f32_4_hash;

pub type ShareEngine<C> = UnsafeMut<Engine<C>>;

pub struct Engine<C: HalContext + 'static> {
    pub gl: C,
    pub res_mgr: Share<RefCell<ResMgr>>,
    pub programs: XHashMap<u64, Share<HalProgram>>,
    pub texture_res_map: UnsafeMut<ResMap<TextureRes>>,
    pub geometry_res_map: UnsafeMut<ResMap<GeometryRes>>,
    pub buffer_res_map: UnsafeMut<ResMap<BufferRes>>,

    pub rs_res_map: UnsafeMut<ResMap<RasterStateRes>>,
    pub bs_res_map: UnsafeMut<ResMap<BlendStateRes>>,
    pub ss_res_map: UnsafeMut<ResMap<StencilStateRes>>,
    pub ds_res_map: UnsafeMut<ResMap<DepthStateRes>>,
    pub sampler_res_map: UnsafeMut<ResMap<SamplerRes>>,

    pub u_color_ubo_map: UnsafeMut<ResMap<UColorUbo>>,
}

impl<C: HalContext + 'static> Engine<C> {
    pub fn new(gl: C, res_mgr: Share<RefCell<ResMgr>>) -> Self {
		let texture_res_map;
		let geometry_res_map;
		let buffer_res_map;
		let rs_res_map;
		let bs_res_map;
		let ss_res_map;
		let ds_res_map;
		let sampler_res_map;
		let u_color_ubo_map;

		{
			let res_mgr_ref = res_mgr.borrow();
			texture_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<TextureRes>(0).unwrap());
			geometry_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<GeometryRes>(0).unwrap());
			buffer_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<BufferRes>(0).unwrap());
			rs_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<RasterStateRes>(0).unwrap());
			bs_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<BlendStateRes>(0).unwrap());
			ss_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<StencilStateRes>(0).unwrap());
			ds_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<DepthStateRes>(0).unwrap());
			sampler_res_map = UnsafeMut::new(res_mgr_ref.fetch_map::<SamplerRes>(0).unwrap());
			u_color_ubo_map = UnsafeMut::new(res_mgr_ref.fetch_map::<UColorUbo>(0).unwrap());
		}

        Engine {
            gl: gl,
            texture_res_map,
            geometry_res_map,
            buffer_res_map,
            rs_res_map,
            bs_res_map,
            ss_res_map,
            ds_res_map,
            sampler_res_map,
            u_color_ubo_map,
            programs: XHashMap::default(),
            res_mgr,
        }
    }

    pub fn create_program(
        &mut self,
        vs_id: u64,
        fs_id: u64,
        vs_name: &str,
        vs_defines: &dyn Defines,
        fs_name: &str,
        fs_defines: &dyn Defines,
        paramter: &dyn ProgramParamter,
    ) -> Share<HalProgram> {
        let mut hasher = DefaultHasher::default();
        vs_id.hash(&mut hasher);
        vs_defines.id().hash(&mut hasher);
        let vs_id = hasher.finish();

        let mut hasher = DefaultHasher::default();
        fs_id.hash(&mut hasher);
        fs_defines.id().hash(&mut hasher);
        let fs_id = hasher.finish();

        let mut hasher = DefaultHasher::default();
        vs_id.hash(&mut hasher);
        fs_id.hash(&mut hasher);
        let hash = hasher.finish();

        let gl = &self.gl;
        self.programs
            .entry(hash)
            .or_insert_with(|| {
                let ubos = paramter.get_layout();
                let mut uniforms = Vec::with_capacity(ubos.len());
                for ubo in ubos.iter() {
                    uniforms.push(paramter.get_value(ubo).unwrap().get_layout());
                }

                let uniform_layout = UniformLayout {
                    ubos: ubos,
                    uniforms: uniforms.as_slice(),
                    single_uniforms: paramter.get_single_uniform_layout(),
                    textures: paramter.get_texture_layout(),
                };

                match gl.program_create_with_vs_fs(
                    vs_id,
                    fs_id,
                    vs_name,
                    vs_defines.list(),
                    fs_name,
                    fs_defines.list(),
                    &uniform_layout,
                ) {
                    Ok(r) => Share::new(r),
                    Err(e) => panic!(
                        "create_program error: {:?}, vs_name: {:?}, fs_name: {:?}",
                        e, vs_name, fs_name
                    ),
                }
            })
            .clone()
    }

    pub fn create_buffer_res(
        &mut self,
        key: u64,
        btype: BufferType,
        count: usize,
        data: Option<BufferData>,
        is_updatable: bool,
    ) -> Share<BufferRes> {
        let size = buffer_size(count, btype);
        let buffer = BufferRes(self.create_buffer(btype, count, data, is_updatable));
        self.buffer_res_map.create(key, buffer, size, 0)
    }

    pub fn create_texture_res(
        &mut self,
        key: usize,
        texture_res: TextureRes,
        rtype: usize,
    ) -> Share<TextureRes> {
		let cost = match texture_res.cost {
			Some(r) => r,
			None => texture_res.width
            * texture_res.height
            * pixe_size(texture_res.pformat, texture_res.dformat),
		};
        self.texture_res_map.create(key, texture_res, cost, rtype)
    }

    //创建一个geo, 该geo的buffer不可更新, 不共享
    pub fn create_geo_res(
        &mut self,
        key: u64,
        indices: &[u16],
        attributes: &[AttributeDecs],
    ) -> Share<GeometryRes> {
        let i_len = indices.len();
        let indices = BufferRes(self.create_buffer(
            BufferType::Indices,
            i_len,
            Some(BufferData::Short(indices)),
            false,
        ));
        let geo = self.create_geometry();
        self.gl.geometry_set_indices_short(&geo, &indices).unwrap();

        let mut size = buffer_size(i_len, BufferType::Indices);

        let mut buffers = Vec::with_capacity(attributes.len() + 1);
        buffers.push(Share::new(indices));

        for desc in attributes.iter() {
            let len = desc.buffer.len();
            let atrribute = BufferRes(self.create_buffer(
                BufferType::Attribute,
                len,
                Some(BufferData::Float(desc.buffer)),
                false,
            ));
            self.gl
                .geometry_set_attribute(&geo, &desc.name, &atrribute, desc.item_count)
                .unwrap();
            size += buffer_size(len, BufferType::Attribute);
            buffers.push(Share::new(atrribute));
        }

        // 创建缓存
        let geo_res = GeometryRes { geo, buffers };
        if key == 0 {
            Share::new(geo_res)
        } else {
            self.geometry_res_map.create(key, geo_res, size, 0)
        }
    }

    pub fn create_rs_res(&mut self, desc: RasterStateDesc) -> Share<RasterStateRes> {
        let h = get_hash(&desc);
        match self.rs_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_rs(desc);
                self.rs_res_map.create(h, RasterStateRes(r), 0, 0)
            }
        }
    }

    pub fn create_bs_res(&mut self, desc: BlendStateDesc) -> Share<BlendStateRes> {
        let h = get_hash(&desc);
        match self.bs_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_bs(desc);
                self.bs_res_map.create(h, BlendStateRes(r), 0, 0)
            }
        }
    }

    pub fn create_ss_res(&mut self, desc: StencilStateDesc) -> Share<StencilStateRes> {
        let h = get_hash(&desc);
        match self.ss_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_ss(desc);
                self.ss_res_map.create(h, StencilStateRes(r), 0, 0)
            }
        }
    }

    pub fn create_ds_res(&mut self, desc: DepthStateDesc) -> Share<DepthStateRes> {
        let h = get_hash(&desc);
        match self.ds_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_ds(desc);
                self.ds_res_map.create(h, DepthStateRes(r), 0, 0)
            }
        }
    }

    pub fn create_sampler_res(&mut self, desc: SamplerDesc) -> Share<SamplerRes> {
        let h = get_hash(&desc);
        match self.sampler_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_sampler(desc);
                self.sampler_res_map.create(h, SamplerRes(r), 0, 0)
            }
        }
    }

    #[inline]
    pub fn create_u_color_ubo(&mut self, c: &CgColor) -> Share<UColorUbo> {
        let h = f32_4_hash(c.r, c.g, c.b, c.a);
        match self.u_color_ubo_map.get(&h) {
            Some(r) => r,
            None => self.u_color_ubo_map.create(
                h,
                UColorUbo::new(UniformValue::Float4(c.r, c.g, c.b, c.a)),
                0,
                0,
            ),
        }
    }

    #[inline]
    pub fn create_buffer(
        &self,
        btype: BufferType,
        count: usize,
        data: Option<BufferData>,
        is_updatable: bool,
    ) -> HalBuffer {
        match self.gl.buffer_create(btype, count, data, is_updatable) {
            Ok(r) => r,
            Err(e) => panic!("create_buffer error: {:?}", e),
        }
    }

    #[inline]
    pub fn create_geometry(&self) -> HalGeometry {
        match self.gl.geometry_create() {
            Ok(r) => r,
            Err(e) => panic!("create_geometry error: {:?}", e),
        }
    }

    #[inline]
    pub fn create_rs(&self, desc: RasterStateDesc) -> HalRasterState {
        match self.gl.rs_create(desc) {
            Ok(r) => r,
            Err(e) => panic!("create_rs error: {:?}", e),
        }
    }

    #[inline]
    pub fn create_bs(&self, desc: BlendStateDesc) -> HalBlendState {
        match self.gl.bs_create(desc) {
            Ok(r) => r,
            Err(e) => panic!("create_bs error: {:?}", e),
        }
    }

    #[inline]
    pub fn create_ss(&self, desc: StencilStateDesc) -> HalStencilState {
        match self.gl.ss_create(desc) {
            Ok(r) => r,
            Err(e) => panic!("create_geometry error: {:?}", e),
        }
    }

    #[inline]
    pub fn create_ds(&self, desc: DepthStateDesc) -> HalDepthState {
        match self.gl.ds_create(desc) {
            Ok(r) => r,
            Err(e) => panic!("create_geometry error: {:?}", e),
        }
    }

    #[inline]
    pub fn create_sampler(&self, desc: SamplerDesc) -> HalSampler {
        match self.gl.sampler_create(desc) {
            Ok(r) => r,
            Err(e) => panic!("create_sampler error: {:?}", e),
        }
    }
}

pub struct AttributeDecs<'a> {
    name: AttributeName,
    buffer: &'a [f32],
    item_count: usize,
}

impl<'a> AttributeDecs<'a> {
    pub fn new(name: AttributeName, buffer: &'a [f32], item_count: usize) -> Self {
        Self {
            name,
            buffer,
            item_count,
        }
    }
}

pub struct UnsafeMut<T>(Share<T>);

impl<T> UnsafeMut<T> {
    pub fn new(v: Share<T>) -> Self {
        Self(v)
    }
}

impl<T> Clone for UnsafeMut<T> {
    fn clone(&self) -> Self {
        UnsafeMut(self.0.clone())
    }
}

impl<T> Deref for UnsafeMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for UnsafeMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&*self.0 as *const T as *mut T) }
    }
}

pub fn buffer_size(count: usize, btype: BufferType) -> usize {
    match btype {
        BufferType::Attribute => 4 * count,
        BufferType::Indices => 2 * count,
    }
}

#[inline]
pub fn get_hash<T: Hash>(v: &T) -> u64 {
    let mut hasher = DefaultHasher::default();
    v.hash(&mut hasher);
    hasher.finish()
}

pub fn create_hash_res<T: Res<Key = u64> + Hash>(res: T, res_map: &mut ResMap<T>) -> Share<T> {
    let h = get_hash(&res);
    match res_map.get(&h) {
        Some(r) => r,
        None => res_map.create(h, res, 0, 0),
    }
}
