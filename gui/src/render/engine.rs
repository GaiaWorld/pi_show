/**
 *  对HalContext的封装， 并管理gl资源
*/
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::cell::RefCell;

use pi_assets::allocator::Allocator;
use pi_assets::asset::{Asset, GarbageEmpty, Handle};
use pi_atom::Atom;
use hash::{DefaultHasher, XHashMap};
use pi_share::Share;
use hal_core::*;

use crate::component::calc::*;
use crate::component::user::CgColor;
use crate::render::res::*;
use crate::single::dyn_texture::UnuseTexture;
use crate::util::f32_4_hash;

use super::asset::{AssetConfig, AssetDesc, ShareAssetMgr, ShareHomogeneousMgr};

pub type ShareEngine<C> = UnsafeMut<Engine<C>>;


#[derive(Debug)]
pub enum ResWrapper<T: Asset> {
    None,
    Handle(Handle<T>),
    Share(Share<T>),
}

#[derive(Debug)]
pub enum ResWrapper1<T: Asset> {
    Handle(Handle<T>),
    Share(Share<T>),
}

impl<T: Asset> Clone for ResWrapper1<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Handle(arg0) => Self::Handle(arg0.clone()),
            Self::Share(arg0) => Self::Share(arg0.clone()),
        }
    }
}

impl<T: Asset> Deref for ResWrapper1<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self{
            ResWrapper1::Handle(r) => &**r,
            ResWrapper1::Share(r) => &**r,
        }
    }
}

impl<T: Asset> Clone for ResWrapper<T> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Handle(arg0) => Self::Handle(arg0.clone()),
            Self::Share(arg0) => Self::Share(arg0.clone()),
        }
    }
}

impl<T: Asset> ResWrapper<T> {
    pub fn is_none(&self) -> bool {
        if let ResWrapper::None = self {
            return true;
        } else {
            false
        }
    }

    pub fn unwrap_ref(&self) -> &T {
        match self{
            ResWrapper::None => panic!("unwrap_ref: ResWrapper::None"),
            ResWrapper::Handle(r) => &**r,
            ResWrapper::Share(r) => &**r,
        }
    }
}

pub struct Engine<C: HalContext + 'static> {
    pub gl: C,
    pub share_allocator: Share<RefCell<Allocator>>,
    pub programs: XHashMap<u64, Share<HalProgram>>,
    pub texture_res_map: ShareAssetMgr<TextureRes>,
	pub texture_part_res_map: ShareAssetMgr<TexturePartRes>,
	pub renderbuffer_res_map: ShareAssetMgr<RenderBufferRes>,
    pub unuse_texture_map: ShareHomogeneousMgr<UnuseTexture>,
    pub geometry_res_map: ShareAssetMgr<GeometryRes>,
    pub buffer_res_map: ShareAssetMgr<BufferRes>,

    pub rs_res_map: ShareAssetMgr<RasterStateRes>,
    pub bs_res_map: ShareAssetMgr<BlendStateRes>,
    pub ss_res_map: ShareAssetMgr<StencilStateRes>,
    pub ds_res_map: ShareAssetMgr<DepthStateRes>,
    pub sampler_res_map: ShareAssetMgr<SamplerRes>,

    pub u_color_ubo_map: ShareAssetMgr<ShareUbo<UColorUbo>>,
    pub msdf_stroke_ubo_map: ShareAssetMgr<ShareUbo<MsdfStrokeUbo>>,
    pub canvas_stroke_ubo_map: ShareAssetMgr<ShareUbo<CanvasTextStrokeColorUbo>>,
    pub hsv_ubo_map: ShareAssetMgr<ShareUbo<HsvUbo>>,
    
}

impl<C: HalContext + 'static> Engine<C> {
    pub fn new(gl: C, share_allocator: Share<RefCell<Allocator>>, asset_config: &AssetConfig ) -> Self { 
		let texture_res_map;
		let texture_part_res_map;
		let renderbuffer_res_map;
		let geometry_res_map;
		let buffer_res_map;
		let rs_res_map;
		let bs_res_map;
		let ss_res_map;
		let ds_res_map;
		let sampler_res_map;
		let u_color_ubo_map;
        let msdf_stroke_ubo_map;
        let canvas_text_ubo_map;
        let hsv_ubo_map;
        let unuse_texture_map;

		{
            let mut allocator = share_allocator.borrow_mut();
			texture_res_map = ShareAssetMgr::<TextureRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 30 * 1024 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 50,
                },
                asset_config,
                &mut allocator,
            );

			texture_part_res_map = ShareAssetMgr::<TexturePartRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 1 * 1024 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 5,
                },
                asset_config,
                &mut allocator,
            );

			renderbuffer_res_map = ShareAssetMgr::<RenderBufferRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 2 * 1024 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 5,
                },
                asset_config,
                &mut allocator,
            );
			geometry_res_map = ShareAssetMgr::<GeometryRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min:  (0.1 * 1024.0 * 1024.0) as usize,
                    timeout: 10 * 60 * 1000,
                    weight: 5,
                },
                asset_config,
                &mut allocator,
            );
			buffer_res_map = ShareAssetMgr::<BufferRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 10 * 1024 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 0,
                },
                asset_config,
                &mut allocator,
            );
			rs_res_map = ShareAssetMgr::<RasterStateRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 200 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
			bs_res_map = ShareAssetMgr::<BlendStateRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 100 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
			ss_res_map = ShareAssetMgr::<StencilStateRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 100 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
			ds_res_map = ShareAssetMgr::<DepthStateRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 100 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
			sampler_res_map = ShareAssetMgr::<SamplerRes>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 100 * 1024,

                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
			u_color_ubo_map = ShareAssetMgr::<ShareUbo<UColorUbo>>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 100 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
            msdf_stroke_ubo_map = ShareAssetMgr::<ShareUbo<MsdfStrokeUbo>>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 1000 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
            canvas_text_ubo_map = ShareAssetMgr::<ShareUbo<CanvasTextStrokeColorUbo>>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 1000 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
            hsv_ubo_map = ShareAssetMgr::<ShareUbo<HsvUbo>>::new_with_config(
                GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    min: 1000 * 1024,
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                asset_config,
                &mut allocator,
            );
            unuse_texture_map = ShareHomogeneousMgr::<UnuseTexture>::new_with_config(
                pi_assets::homogeneous::GarbageEmpty(),
                &AssetDesc {
                    ref_garbage: false,
                    // 至少缓存五张， 最多缓存10张
                    min: 5 * std::mem::size_of::<UnuseTexture>(),
                    timeout: 10 * 60 * 1000,
                    weight: 1,
                },
                &asset_config,
                &mut allocator,
            );
		}

        Engine {
            gl: gl,
            texture_res_map,
			texture_part_res_map,
			renderbuffer_res_map,
            unuse_texture_map,
            geometry_res_map,
            buffer_res_map,
            rs_res_map,
            bs_res_map,
            ss_res_map,
            ds_res_map,
            sampler_res_map,
            u_color_ubo_map,
            msdf_stroke_ubo_map,
            canvas_stroke_ubo_map: canvas_text_ubo_map,
            hsv_ubo_map,
            programs: XHashMap::default(),
            share_allocator,
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
                    Err(e) => {
						log::warn!("create_program error: {:?}, vs_name: {:?}, fs_name: {:?}",
						e, vs_name, fs_name);
						panic!(
							"create_program error: {:?}, vs_name: {:?}, fs_name: {:?}",
							e, vs_name, fs_name
						);
					},
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
    ) -> Handle<BufferRes> {
		match self.buffer_res_map.get(&key) {
			Some(r) => r,
			None => {
				let size = buffer_size(count, btype);
				let buffer = BufferRes{ value: self.create_buffer(btype, count, data, is_updatable), size: size};
                match self.buffer_res_map.insert(key, buffer) {
                    Ok(r) => r,
                    Err(r) => self.buffer_res_map.get(&key).unwrap(),
                }
			}
		}
    }

    #[inline]
    pub fn create_texture_res(
        &mut self,
        key: Atom,
        texture_res: TextureRes,
    ) -> Handle<TextureRes> {
        match self.texture_res_map.insert(key.clone(), texture_res) {
            Ok(r) => r,
            Err(r) => self.texture_res_map.get(&key).unwrap(),
        }
    }

    //创建一个geo, 该geo的buffer不可更新, 不共享
    pub fn create_geo_res(
        &mut self,
        key: u64,
        indices: &[u16],
        attributes: &[AttributeDecs],
    ) -> ResWrapper<GeometryRes> {
        let i_len = indices.len();
        let mut size = buffer_size(i_len, BufferType::Indices);
        let indices = BufferRes{value: self.create_buffer(
            BufferType::Indices,
            i_len,
            Some(BufferData::Short(indices)),
            false,
        ), size: size};
        let geo = self.create_geometry();
        self.gl.geometry_set_indices_short(&geo, &indices).unwrap(); 

        let mut buffers = Vec::with_capacity(attributes.len() + 1);
        buffers.push(ResWrapper1::Share(Share::new(indices)));

        for desc in attributes.iter() {
            let len = desc.buffer.len();
            let s = buffer_size(len, BufferType::Attribute);
            let atrribute = BufferRes{ value: self.create_buffer(
                BufferType::Attribute,
                len,
                Some(BufferData::Float(desc.buffer)),
                false,
            ), size: s};
            self.gl
                .geometry_set_attribute(&geo, &desc.name, &atrribute, desc.item_count)
                .unwrap();
            size += s;
            buffers.push(ResWrapper1::Share(Share::new(atrribute)));
        }

        // 创建缓存
        let geo_res = GeometryRes { geo, buffers, size };
        if key == 0 {
            ResWrapper::Share(Share::new(geo_res))
        } else {
            ResWrapper::Handle(match self.geometry_res_map.insert(key, geo_res) {
                Ok(r) => r,
                _ => self.geometry_res_map.get(&key).unwrap(),
            })
        }
    }

    pub fn create_rs_res(&mut self, desc: RasterStateDesc) -> Handle<RasterStateRes> {
        let h = get_hash(&desc);
        match self.rs_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_rs(desc);
                match self.rs_res_map.insert(h, RasterStateRes(r)) {
                    Ok(r) => r,
                    _ => self.rs_res_map.get(&h).unwrap(),
                }
            }
        }
    }

    pub fn create_bs_res(&mut self, desc: BlendStateDesc) -> Handle<BlendStateRes> {
        let h = get_hash(&desc);
        match self.bs_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_bs(desc);
                match self.bs_res_map.insert(h, BlendStateRes(r)) {
                    Ok(r) => r,
                    _ => self.bs_res_map.get(&h).unwrap(),
                }
            }
        }
    }

    pub fn create_ss_res(&mut self, desc: StencilStateDesc) -> Handle<StencilStateRes> {
        let h = get_hash(&desc);
        match self.ss_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_ss(desc);
                match self.ss_res_map.insert(h, StencilStateRes(r)) {
                    Ok(r) => r,
                    _ => self.ss_res_map.get(&h).unwrap(),
                }
            }
        }
    }

    pub fn create_ds_res(&mut self, desc: DepthStateDesc) -> Handle<DepthStateRes> {
        let h = get_hash(&desc);
        match self.ds_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_ds(desc);
                match self.ds_res_map.insert(h, DepthStateRes(r)) {
                    Ok(r) => r,
                    _ => self.ds_res_map.get(&h).unwrap(),
                }
            }
        }
    }

    pub fn create_sampler_res(&mut self, desc: SamplerDesc) -> Handle<SamplerRes> {
        let h = get_hash(&desc);
        match self.sampler_res_map.get(&h) {
            Some(r) => r,
            None => {
                let r = self.create_sampler(desc);
                match self.sampler_res_map.insert(h, SamplerRes(r)) {
                    Ok(r) => r,
                    _ => self.sampler_res_map.get(&h).unwrap(),
                }
            }
        }
    }

    #[inline]
    pub fn create_u_color_ubo(&mut self, c: &CgColor) -> Share<UColorUbo> {
        let h = f32_4_hash(c.x, c.y, c.z, c.w);
        match self.u_color_ubo_map.get(&h) {
            Some(r) => (**r).0.clone(),
            None => match self.u_color_ubo_map.insert(h, ShareUbo(Share::new(UColorUbo::new(UniformValue::Float4(c.x, c.y, c.z, c.w))))) {
                Ok(r) => (**r).0.clone(),
                _ => (**self.u_color_ubo_map.get(&h).unwrap()).0.clone(),
            }
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

pub fn create_hash_res<T: Asset<Key = u64> + Hash>(res: T, res_map: &ShareAssetMgr<T>) -> Handle<T> {
    let h = get_hash(&res);
    match res_map.get(&h) {
        Some(r) => r,
        None => match res_map.insert(h, res) {
            Ok(r) => r,
            Err(_) => res_map.get(&h).unwrap(),
        },
    }
}
