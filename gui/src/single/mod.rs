pub mod class;
/**
 * 定义单例类型
*/
pub mod oct;
pub mod style_parse;
pub mod dyn_texture;

use dirty::LayerDirty;
use flex_layout::Size;
use share::Share;
use std::any::{Any, TypeId};
use std::default::Default;
use std::ops::{Index, IndexMut};

use atom::Atom;
// use cgmath::Ortho;
use nalgebra::Orthographic3;
use ecs::monitor::NotifyImpl;
use ecs::Write;
use hal_core::*;
use hash::XHashMap;
use map::vecmap::VecMap;
use slab::Slab;

use crate::component::calc::{ClipBox, WorldMatrix, ImageTexture};
use crate::component::user::*;
use crate::render::res::*;
pub use crate::single::class::*;
pub use crate::single::oct::Oct;

pub struct OverflowClip {
    pub id_map: XHashMap<usize, usize>,
    pub clip: Slab<Clip>, //[[Point2;4];16], //Vec<(clip_view, has_rotate, old_has_rotate)> 最多32个
    pub clip_map: XHashMap<usize, (Aabb2, Share<dyn UniformBuffer>)>,
    // pub id_vec: [usize;16],
    // pub clip: [[Point2;4];16],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Clip {
    pub view: [Point2; 4],
    pub has_rotate: bool,
    pub old_has_rotate: bool,
    pub node_id: usize,
}

impl OverflowClip {
    pub fn mem_size(&self) -> usize {
        2 * self.id_map.capacity() * std::mem::size_of::<usize>()
            + self.clip.mem_size()
            + self.clip_map.capacity()
                * (std::mem::size_of::<usize>()
                    + std::mem::size_of::<(Aabb2, Share<dyn UniformBuffer>)>())
    }
    pub fn insert_aabb(
        &mut self,
        key: usize,
        value: Aabb2,
        view_matrix: &WorldMatrix,
    ) -> &(Aabb2, Share<dyn UniformBuffer>) {
        let min = view_matrix * Vector4::new(value.mins.x, value.mins.y, 0.0, 0.0);
        let max = view_matrix * Vector4::new(value.maxs.x, value.maxs.y, 0.0, 0.0);
        let (w, h) = ((max.x - min.x) / 2.0, (max.y - min.y) / 2.0);
        let ubo = ClipBox::new(UniformValue::Float4(min.x + w, min.y + h, w, h));

        // self.clip_map.entry(key).and_modify(|e|{
        //     *e = (value, Share::new(ubo))
        // }).or {
        //     Entry::Occupied(mut r) => {
        //         r.insert((value, Share::new(ubo)));
        //         r.get()
        //     },
        //     Entry::Vacant(r) => r.insert((value, Share::new(ubo))),
        // }
        self.clip_map.insert(key, (value, Share::new(ubo)));
        self.clip_map.get(&key).unwrap()
    }
}


impl Default for OverflowClip {
    fn default() -> Self {
        let mut r = Self {
            id_map: XHashMap::default(),
            clip: Slab::default(),
            clip_map: XHashMap::default(),
        };
        r.insert_aabb(
            0,
            Aabb2::new(
                Point2::new(std::f32::MIN, std::f32::MIN),
                Point2::new(std::f32::MAX, std::f32::MAX),
            ),
            &WorldMatrix::default(),
        );
        r
    }
}

#[derive(Debug)]
pub struct ViewMatrix(pub WorldMatrix);

#[derive(Clone)]
pub struct ProjectionMatrix(pub WorldMatrix);

impl ProjectionMatrix {
    pub fn new( width: f32, height: f32, near: f32, far: f32) -> ProjectionMatrix {
        let ortho = Orthographic3::new(0.0, width, height, 0.0, near, far);
		ProjectionMatrix(WorldMatrix(Matrix4::from(ortho), false))
    }
}

pub struct RenderRect {
	pub width: usize,
	pub height: usize,
	pub view_port: Aabb2,
	pub flex: (f32, f32),
}

// 图片等待表
#[derive(Default)]
pub struct ImageWaitSheet {
    pub wait: XHashMap<usize, Vec<ImageWait>>,
    pub finish: Vec<(usize, Share<TextureRes>, Vec<ImageWait>)>,
    pub loads: Vec<usize>,
}


impl ImageWaitSheet {
    pub fn mem_size(&self) -> usize {
        let mut r = 0;
        for (_, v) in self.wait.iter() {
            r += v.capacity() * std::mem::size_of::<ImageWait>();
        }
        for v in self.finish.iter() {
            r += v.2.capacity() * std::mem::size_of::<ImageWait>();
        }

        r += self.loads.capacity() * std::mem::size_of::<Atom>();

        r
    }
    pub fn add(&mut self, name: usize, wait: ImageWait) {
        let loads = &mut self.loads;
        self.wait
            .entry(name)
            .or_insert_with(|| {
                loads.push(name);
                Vec::with_capacity(1)
            })
            .push(wait);
    }
}

#[derive(Debug)]
pub enum ImageType {
    ImageClass,
    ImageLocal,
    BorderImageClass,
    BorderImageLocal,
	MaskImageClass,
    MaskImageLocal,
}

// 渲染根节点
pub struct RenderRoot {
	target: Option<usize>, // None表示使用默认渲染目标，否则，数字使用dyn_texture中的的索引
}

pub struct RenderRootSet {
	roots: XHashMap<usize/*实体id */, RenderRoot>,
}

#[derive(Debug)]
pub struct ImageWait {
    pub ty: ImageType,
    pub id: usize,
}

pub struct UnitQuad(pub Share<GeometryRes>);

#[derive(Default)]
pub struct DirtyList(pub Vec<usize>);

impl DirtyList {
	pub fn with_capacity(capacity: usize) -> DirtyList{
        Self(Vec::with_capacity(capacity))
    }
}

/// 预乘模式
#[derive(Deref, DerefMut)]
pub struct PremultiState(pub CommonState);

impl PremultiState {
	pub fn from_common<C: HalContext + 'static>(common: &CommonState, gl: &C) -> Self {
        let mut df_bs = BlendStateDesc::default();
        let mut df_ds = DepthStateDesc::default();

		df_bs.set_rgb_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
		df_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
        df_ds.set_write_enable(true);

		let tarns_ds = DepthStateDesc::default();

		Self(CommonState{
			df_rs: common.df_rs.clone(),
			df_ss: common.df_ss.clone(),
			df_bs: Share::new(BlendStateRes(gl.bs_create(df_bs).unwrap())),
            df_ds: Share::new(DepthStateRes(gl.ds_create(df_ds).unwrap())),
            alpha_add_bs: common.alpha_add_bs.clone(),
			multiply_bs: common.multiply_bs.clone(),
			subtract_bs: common.subtract_bs.clone(),
			one_one_bs: common.one_one_bs.clone(),
            tarns_ds: Share::new(DepthStateRes(gl.ds_create(tarns_ds).unwrap())),
		})
	}
}

// 根索引
#[derive(Debug, Default)]
pub struct RootIndexs(pub LayerDirty<usize>, pub bool);

impl std::ops::Deref for RootIndexs {
    type Target = LayerDirty<usize>;

    fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for RootIndexs {
    fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}


#[derive(Deref, DerefMut)]
pub struct DefaultState(pub CommonState);
pub struct CommonState {
    pub df_rs: Share<RasterStateRes>,
    pub df_bs: Share<BlendStateRes>,
    pub df_ss: Share<StencilStateRes>,
    pub df_ds: Share<DepthStateRes>,

    pub tarns_ds: Share<DepthStateRes>,

	pub alpha_add_bs: Share<BlendStateRes>,
	pub multiply_bs: Share<BlendStateRes>,
	pub subtract_bs: Share<BlendStateRes>,
	pub one_one_bs: Share<BlendStateRes>,
}

impl CommonState {
    pub fn new<C: HalContext + 'static>(gl: &C) -> Self {
        let df_rs = RasterStateDesc::default();
        let mut df_bs = BlendStateDesc::default();
        let df_ss = StencilStateDesc::default();
        let mut df_ds = DepthStateDesc::default();

		df_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
		df_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
        df_ds.set_write_enable(true);

		let mut alpha_add_bs = BlendStateDesc::default();
		alpha_add_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::One);
		alpha_add_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);

		let mut subtract_bs = BlendStateDesc::default();
		subtract_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::One);
		subtract_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);

		let mut one_one_bs = BlendStateDesc::default();
		one_one_bs.set_rgb_factor(BlendFactor::One, BlendFactor::One);
		one_one_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);

		let mut multiply_bs = BlendStateDesc::default();
		multiply_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::One);
		multiply_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);

        // let mut tarns_ds = DepthStateDesc::default();
        let tarns_ds = DepthStateDesc::default();
        // tarns_ds.set_write_enable(false);

        Self {
            df_rs: Share::new(RasterStateRes(gl.rs_create(df_rs).unwrap())),
            df_ss: Share::new(StencilStateRes(gl.ss_create(df_ss).unwrap())),
			df_bs: Share::new(BlendStateRes(gl.bs_create(df_bs).unwrap())),
            df_ds: Share::new(DepthStateRes(gl.ds_create(df_ds).unwrap())),
            tarns_ds: Share::new(DepthStateRes(gl.ds_create(tarns_ds).unwrap())),
			alpha_add_bs: Share::new(BlendStateRes(gl.bs_create(alpha_add_bs).unwrap())),
			subtract_bs: Share::new(BlendStateRes(gl.bs_create(subtract_bs).unwrap())),
			one_one_bs: Share::new(BlendStateRes(gl.bs_create(one_one_bs).unwrap())),
			multiply_bs: Share::new(BlendStateRes(gl.bs_create(multiply_bs).unwrap())),
        }
    }
}

pub struct Data<C> {
    map: Slab<C>,
    notify: NotifyImpl,
}

impl<C> Default for Data<C> {
    fn default() -> Self {
        Self {
            map: Slab::default(),
            notify: NotifyImpl::default(),
        }
    }
}

impl<T> Index<usize> for Data<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self.map[index]
    }
}

impl<T> IndexMut<usize> for Data<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.map[index]
    }
}

impl<C> Data<C> {
    pub fn get(&self, id: usize) -> Option<&C> {
        self.map.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut C> {
        self.map.get_mut(id)
    }

    pub fn get_write(&mut self, id: usize) -> Option<Write<C>> {
        match self.map.get_mut(id) {
            Some(r) => Some(Write::new(id, r, &self.notify)),
            None => None,
        }
    }

    pub unsafe fn get_unchecked_write(&mut self, id: usize) -> Write<C> {
        Write::new(id, self.map.get_unchecked_mut(id), &self.notify)
    }

    pub fn create(&mut self, c: C) -> usize {
        let r = self.map.insert(c);
        self.notify.create_event(r);
        r
    }

    pub fn delete(&mut self, id: usize) {
        self.notify.delete_event(id);
        self.map.remove(id);
    }

    pub fn get_notify(&self) -> NotifyImpl {
        self.notify.clone()
    }
}

#[derive(Clone, Deref, DerefMut, Default)]
pub struct RenderContextAttrCount(usize);

#[derive(Clone)]
pub struct State {
    pub rs: Share<RasterStateRes>,
    pub bs: Share<BlendStateRes>,
    pub ss: Share<StencilStateRes>,
    pub ds: Share<DepthStateRes>,
}

// 预渲染内容
#[derive(Deref, DerefMut)]
pub struct PreRenderList(pub Vec<PreRenderItem>);

pub struct PreRenderItem {
	pub index: usize, // 渲染目标 在DynAtlasSet中的索引
	pub obj: RenderObj,
}

#[derive(Write)]
pub struct RenderObj {
    pub depth: f32,
    pub depth_diff: f32,
    pub visibility: bool,
    pub is_opacity: bool,
    pub vs_name: Atom,
    pub fs_name: Atom,
    pub vs_defines: Box<dyn Defines>,
    pub fs_defines: Box<dyn Defines>,
    pub paramter: Share<dyn ProgramParamter>,
    pub program_dirty: bool,

    pub program: Option<Share<HalProgram>>,
    pub geometry: Option<Share<GeometryRes>>,
    pub state: State,

    pub context: usize,

	pub post_process: Option<Box<PostProcessContext>>,

	pub vert_type: VertType,
}

/// 是否使用单位四边形渲染
#[derive(EnumDefault, PartialEq, Eq, Clone, Copy)]
pub enum VertType {
	Border, // 渲染为边框部分
	ContentNone, // 渲染为content区，世界矩阵不变换
	BorderNone, // 渲染为border区，世界矩阵不变换
	ContentRect, // 渲染为content区，世界矩阵需要变换
	BorderRect,	// 渲染为border区，世界矩阵需要变换
}

impl PostProcessObj for RenderObj {
	fn get_post(&self) -> &Option<Box<PostProcessContext>> {
		&self.post_process
	}
	fn get_post_mut(&mut self) -> &mut Option<Box<PostProcessContext>> {
		&mut self.post_process
	}
	fn set_post(&mut self, v: Option<Box<PostProcessContext>>) {
		self.post_process = v;
	}
}

pub trait PostProcessObj {
	fn get_post(&self) -> &Option<Box<PostProcessContext>>;
	fn get_post_mut(&mut self) -> &mut Option<Box<PostProcessContext>>;
	fn set_post(&mut self, v: Option<Box<PostProcessContext>>);
}

/// 渲染上下文
/// 渲染上下文包含一个渲染目标和一个或多个可以渲染到该目标的渲染对象
/// 渲染上下文也可以包含一些后处理
pub struct PostProcessContext {
	pub content_box: Aabb2, // 内容的最大包围盒（纹理在gui最终）
	pub render_target: Option<usize>, // 后处理对象, 在dyn_texture中的索引，如果为None，不会进行后处理
	pub post_processes: Vec<PostProcess>, // 后处理
	pub result: Option<usize>, // 后处理的处理结果, 在dyn_texture中的索引
	pub copy: usize, // 在renderobjs中的索引，可以将后处理结果拷贝在最终的渲染目标上
}

/// 后处理。
/// 每个渲染对象都可能存在一个后处理
pub struct PostProcess {
	// pub render_target: Option<usize>, 
	// pub render_rect: Aabb2, // 大小、尺寸（将纹理渲染到目标的哪个区域）
	pub render_size: Size<f32>,
	pub render_obj: RenderObj, // 在RenderObjs中的索引
}

/// 像素比
pub struct PixelRatio(pub f32);

#[derive(Deref, DerefMut)]
pub struct RenderObjs(pub Slab<RenderObj>);

impl Default for RenderObjs {
    fn default() -> Self {
        Self(Slab::default())
    }
}

impl RenderObjs {
	pub fn with_capacity(capacity: usize) -> Self {
        Self(Slab::with_capacity(capacity))
    }
    pub fn mem_size(&self) -> usize {
        self.0.mem_size()
    }
    pub fn insert(&mut self, value: RenderObj, notify: Option<&NotifyImpl>) -> usize {
        let id = self.0.insert(value);
        match notify {
            Some(n) => n.create_event(id),
            _ => (),
        };
        id
    }

    pub unsafe fn remove_unchecked(&mut self, id: usize, notify: Option<&NotifyImpl>) {
		match notify {
            Some(n) => n.delete_event(id),
            _ => (),
        };
        self.0.remove(id);
    }

    pub fn remove(&mut self, id: usize, notify: Option<&NotifyImpl>) {
        if self.0.contains(id) {
			match notify {
                Some(n) => n.delete_event(id),
                _ => (),
            };
            self.0.remove(id);
        }
    }
	pub fn get_write<'a>(
        &'a mut self,
        id: usize,
        notify: &'a NotifyImpl,
    ) -> Write<RenderObj> {
        unsafe { Write::new(id, self.0.get_unchecked_mut(id), &notify) }
    }
    pub unsafe fn get_unchecked_write<'a>(
        &'a mut self,
        id: usize,
        notify: &'a NotifyImpl,
    ) -> Write<RenderObj> {
        Write::new(id, self.0.get_unchecked_mut(id), &notify)
    }

    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut RenderObj {
        self.0.get_unchecked_mut(id)
    }
}

pub struct NodeRenderMap(VecMap<Vec<usize>>);

impl Index<usize> for NodeRenderMap {
    type Output = Vec<usize>;

    fn index(&self, index: usize) -> &Vec<usize> {
        &self.0[index]
    }
}

impl NodeRenderMap {
    pub fn new() -> Self {
        Self(VecMap::default())
	}
	
	pub fn with_capacity(capacity: usize) -> Self {
        Self(VecMap::with_capacity(capacity))
    }

	pub fn add(&mut self, node_id: usize, render_id: usize, notify: &NotifyImpl) {
        self.0[node_id].push(render_id);
        notify.modify_event(node_id, "add", render_id);
	}

    pub fn remove(
        &mut self,
        node_id: usize,
        render_id: usize,
        notify: &NotifyImpl,
    ) {
        notify.modify_event(node_id, "remove", render_id);
		let mut i = None;
		let arr = &mut self.0[node_id];
		for index in 0..arr.len() {
			if arr[index] == render_id {
				i = Some(index);
				break;
			}
		}
		if let Some(i) = i {
			self.0[node_id].swap_remove(i);
		}
        
    }

    pub fn create(&mut self, node_id: usize) {
        self.0.insert(node_id, Vec::new());
    }

    pub unsafe fn destroy_unchecked(&mut self, node_id: usize) {
        self.0.remove_unchecked(node_id);
    }

    pub fn get(&self, node_id: usize) -> Option<&Vec<usize>> {
        self.0.get(node_id)
    }
}

pub struct RenderBegin(pub RenderBeginDesc, pub Option<Share<HalRenderTarget>> );

/// 脏区域，描述了界面发生修改的区域，用于优化界面局部修改时，进渲染该区域
/// 该区域以根节点的原点最为原点
#[derive(Debug)]
pub struct DirtyViewRect(pub f32, pub f32, pub f32, pub f32, pub bool/*是否与最大视口相等（RenderBeginDesc中的视口）*/);

#[derive(Default)]
pub struct Statistics {
    pub drawcall_times: usize,
}

#[derive(Default)]
pub struct SystemTime {
	pub cur_time: usize,
}

#[derive(Deref, DerefMut, Default)]
pub struct IdTree(idtree::IdTree<u32>);

impl IdTree {
	pub fn with_capacity(capacity: usize) -> Self {
        Self(idtree::IdTree::with_capacity(capacity))
	}
	
	pub fn insert_child_with_notify(&mut self, child: usize, parent:usize, index: usize, notify: &NotifyImpl) {
		self.insert_child(child, parent, index);
		let node = &self[child];
		if node.layer() > 0 {
			notify.create_event(child);
		} else {
			notify.modify_event(child, "add", 0);
		}
	}

	pub fn insert_brother_with_notify(&mut self, child: usize, brother:usize, index: idtree::InsertType, notify: &NotifyImpl) {
		self.insert_brother(child, brother, index);
		let node = &self[child];
		if node.layer() > 0 {
			notify.create_event(child);
			notify.modify_event(child, "totree", 0);
			let head = node.children().head;
			if head > 0 {
				for (child, _) in self.recursive_iter(head) {
					notify.modify_event(child, "totree", 0);
				}
			}
		} else {
			notify.modify_event(child, "add", 0);
		}
	}

	pub fn remove_with_notify(&mut self, id: usize, notify: &NotifyImpl) -> Option<usize> {
		let r = match self.get(id) {
			Some(n) => {
				if n.parent() == 0 && n.layer() == 0 {
					return None;
				}
				(n.parent(), n.layer(), n.count(), n.prev(), n.next(), n.children().head)
			}
			_ => return None,
		};
		IdTree::notify_move(id, r.1, notify);
		self.remove(id, r);
		// log::warn!("remove is some========:{:?}", self.get(id).is_some());
		Some(id)
	}

	pub fn remove_no_notify(&mut self, id: usize) -> Option<usize> {
		let r = match self.get(id) {
			Some(n) => {
				if n.parent() == 0 && n.layer() == 0 {
					return None;
				}
				(n.parent(), n.layer(), n.count(), n.prev(), n.next(), n.children().head)
			}
			_ => return None,
		};
		self.remove(id, r);
		Some(id)
	}
	pub fn destroy(&mut self, id: usize) {
		let r = match self.get(id) {
			Some(n) => (n.parent(), n.layer(), n.count(), n.prev(), n.next(), n.children().head),
			_ => return,
		};
		self.0.destroy(id, r, true);
	}

	fn notify_move(id: usize, layer: usize, notify: &NotifyImpl){
		if layer > 0 {
			notify.delete_event(id)
		} else {
			notify.modify_event(id, "remove", layer)
		}
	}
}