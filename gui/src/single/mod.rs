pub mod class;
/**
 * 定义单例类型
*/
pub mod oct;
pub mod style_parse;

use share::Share;
use std::any::{Any, TypeId};
use std::default::Default;
use std::ops::{Index, IndexMut};

use atom::Atom;
use cgmath::Ortho;
use ecs::monitor::NotifyImpl;
use ecs::Write;
use hal_core::*;
use hash::XHashMap;
use map::vecmap::VecMap;
use slab::Slab;

use component::calc::{ClipBox, WorldMatrix};
use component::user::*;

use render::res::*;

pub use single::class::*;
pub use single::oct::Oct;

pub struct OverflowClip {
    pub id_map: XHashMap<usize, usize>,
    pub clip: Slab<Clip>, //[[Point2;4];16], //Vec<(clip_view, has_rotate, old_has_rotate)> 最多32个
    pub clip_map: XHashMap<usize, (Aabb3, Share<dyn UniformBuffer>)>,
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
                    + std::mem::size_of::<(Aabb3, Share<dyn UniformBuffer>)>())
    }
    pub fn insert_aabb(
        &mut self,
        key: usize,
        value: Aabb3,
        view_matrix: &WorldMatrix,
    ) -> &(Aabb3, Share<dyn UniformBuffer>) {
        let min = view_matrix * Vector4::new(value.min.x, value.min.y, 0.0, 0.0);
        let max = view_matrix * Vector4::new(value.max.x, value.max.y, 0.0, 0.0);
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
            Aabb3::new(
                Point3::new(std::f32::MIN, std::f32::MIN, 0.0),
                Point3::new(std::f32::MAX, std::f32::MAX, 0.0),
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
    pub fn new(width: f32, height: f32, near: f32, far: f32) -> ProjectionMatrix {
        let ortho = Ortho {
            left: 0.0,
            right: width,
            bottom: height,
            top: 0.0,
            near: near,
            far: far,
        };
        ProjectionMatrix(WorldMatrix(Matrix4::from(ortho), false))
    }
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

pub struct DefaultState {
    pub df_rs: Share<RasterStateRes>,
    pub df_bs: Share<BlendStateRes>,
    pub df_ss: Share<StencilStateRes>,
    pub df_ds: Share<DepthStateRes>,

    pub tarns_bs: Share<BlendStateRes>,
    pub tarns_ds: Share<DepthStateRes>,
}

impl DefaultState {
    pub fn new<C: HalContext + 'static>(gl: &C) -> Self {
        let df_rs = RasterStateDesc::default();
        let mut df_bs = BlendStateDesc::default();
        let df_ss = StencilStateDesc::default();
        let mut df_ds = DepthStateDesc::default();

		df_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
		df_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);
        df_ds.set_write_enable(true);

        let mut tarns_bs = BlendStateDesc::default();
		tarns_bs.set_rgb_factor(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha);
		tarns_bs.set_alpha_factor(BlendFactor::One, BlendFactor::OneMinusSrcAlpha);

        // let mut tarns_ds = DepthStateDesc::default();
        let tarns_ds = DepthStateDesc::default();
        // tarns_ds.set_write_enable(false);

        Self {
            df_rs: Share::new(RasterStateRes(gl.rs_create(df_rs).unwrap())),
            df_bs: Share::new(BlendStateRes(gl.bs_create(df_bs).unwrap())),
            df_ss: Share::new(StencilStateRes(gl.ss_create(df_ss).unwrap())),
            df_ds: Share::new(DepthStateRes(gl.ds_create(df_ds).unwrap())),
            tarns_bs: Share::new(BlendStateRes(gl.bs_create(tarns_bs).unwrap())),
            tarns_ds: Share::new(DepthStateRes(gl.ds_create(tarns_ds).unwrap())),
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

#[derive(Clone)]
pub struct State {
    pub rs: Share<RasterStateRes>,
    pub bs: Share<BlendStateRes>,
    pub ss: Share<StencilStateRes>,
    pub ds: Share<DepthStateRes>,
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
}

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
        self.0.remove(id);
        match notify {
            Some(n) => n.delete_event(id),
            _ => (),
        };
    }

    pub fn remove(&mut self, id: usize, notify: Option<&NotifyImpl>) {
        if self.0.contains(id) {
            self.0.remove(id);
            match notify {
                Some(n) => n.delete_event(id),
                _ => (),
            };
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

pub struct RenderBegin(pub RenderBeginDesc, pub Option<Share<HalRenderTarget>>);

/// 脏区域，描述了界面发生修改的区域，用于优化界面局部修改时，进渲染该区域
/// 该区域以根节点的原点最为原点
#[derive(Debug)]
pub struct DirtyViewRect(pub f32, pub f32, pub f32, pub f32, pub bool/*是否与最大视口相等（RenderBeginDesc中的视口）*/);

pub struct DefaultTable(XHashMap<TypeId, Box<dyn Any>>);

#[derive(Default)]
pub struct Statistics {
    pub drawcall_times: usize,
}

#[derive(Default)]
pub struct SystemTime {
	pub cur_time: usize,
}

#[derive(Deref, DerefMut, Default)]
pub struct IdTree(idtree::IdTree<usize>);

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
impl DefaultTable {
    pub fn new() -> Self {
        Self(XHashMap::default())
    }
    pub fn mem_size(&self) -> usize {
        self.0.capacity() * (std::mem::size_of::<TypeId>() + std::mem::size_of::<Box<dyn Any>>())
    }
    pub fn set<T: 'static + Any>(&mut self, value: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static + Any>(&self) -> Option<&T> {
        match self.0.get(&TypeId::of::<T>()) {
            Some(r) => r.downcast_ref::<T>(),
            None => None,
        }
    }

    pub fn get_mut<T: 'static + Any>(&mut self) -> Option<&mut T> {
        match self.0.get_mut(&TypeId::of::<T>()) {
            Some(r) => r.downcast_mut::<T>(),
            None => None,
        }
    }

    pub fn get_unchecked<T: 'static + Any>(&self) -> &T {
        self.0
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref::<T>()
            .unwrap()
    }

    pub fn get_unchecked_mut<T: 'static + Any>(&mut self) -> &mut T {
        self.0
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut::<T>()
            .unwrap()
    }

    pub fn delete<T: 'static + Any>(&mut self) {
        self.0.remove(&TypeId::of::<T>());
    }
}
