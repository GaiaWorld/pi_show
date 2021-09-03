pub mod class;
/**
 * 定义单例类型
*/
pub mod oct;
pub mod style_parse;

use std::ops::Index;

// use bevy_ecs::event::EventWriter;
use bevy_ecs::prelude::{EventWriter, Entity};

use idtree::Node;
use map::vecmap::VecMap;
use share::Share;
// use std::any::{Any, TypeId};
use std::default::Default;
use atom::Atom;
// use cgmath::Ortho;
use nalgebra::{Orthographic3};
// use ecs::monitor::EventWriter<Events>;
// use ecs::Write;
use hal_core::*;
use hash::XHashMap;
use slab::Slab;

use crate::component::calc::{ClipBox, WorldMatrix};
use crate::component::user::*;
use crate::render::res::*;
pub use crate::single::class::*;
pub use crate::single::oct::Oct;
use crate::util::event::{EntityEvent};

pub struct OverflowClip {
    pub id_map: XHashMap<usize, usize>,
    pub clip: Slab<Clip>, //[[Point2;4];16], //Vec<(clip_view, has_rotate, old_has_rotate)> 最多32个
    pub clip_map: XHashMap<usize, (Aabb2, Share<dyn UniformBuffer>)>,
    // pub id_vec: [usize;16],
    // pub clip: [[Point2;4];16],
}

unsafe impl Sync for OverflowClip {}
unsafe impl Send for OverflowClip {}

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
    pub fn new(width: f32, height: f32, near: f32, far: f32) -> ProjectionMatrix {
        let ortho = Orthographic3::new(0.0, width, height, 0.0, near, far);
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
pub struct DirtyList(pub Vec<Entity>);

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

// pub struct Data<C> {
//     map: Slab<C>,
//     notify: EventWriter,
// }

// impl<C> Default for Data<C> {
//     fn default() -> Self {
//         Self {
//             map: Slab::default(),
//             notify: EventWriter<Events>::default(),
//         }
//     }
// }

// impl<T> Index<usize> for Data<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &T {
//         &self.map[index]
//     }
// }

// impl<T> IndexMut<usize> for Data<T> {
//     fn index_mut(&mut self, index: usize) -> &mut T {
//         &mut self.map[index]
//     }
// }

// impl<C> Data<C> {
//     pub fn get(&self, id: usize) -> Option<&C> {
//         self.map.get(id)
//     }

//     pub fn get_mut(&mut self, id: usize) -> Option<&mut C> {
//         self.map.get_mut(id)
//     }

//     pub fn get_write(&mut self, id: usize) -> Option<Write<C>> {
//         match self.map.get_mut(id) {
//             Some(r) => Some(Write::new(id, r, &self.notify)),
//             None => None,
//         }
//     }

//     pub unsafe fn get_unchecked_write(&mut self, id: usize) -> Write<C> {
//         Write::new(id, self.map.get_unchecked_mut(id), &self.notify)
//     }

//     pub fn create(&mut self, c: C) -> usize {
//         let r = self.map.insert(c);
//         self.notify.create_event(r);
//         r
//     }

//     pub fn delete(&mut self, id: usize) {
//         self.notify.delete_event(id);
//         self.map.remove(id);
//     }

//     pub fn get_notify(&self) -> EventWriter<Events> {
//         self.notify.clone()
//     }
// }

#[derive(Clone)]
pub struct State {
    pub rs: Share<RasterStateRes>,
    pub bs: Share<BlendStateRes>,
    pub ss: Share<StencilStateRes>,
    pub ds: Share<DepthStateRes>,
}

// #[derive(Write)]
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

    pub context: Entity,
}
/// 强制实现Send和Sync，TODO
unsafe impl Send for RenderObj {}
unsafe impl Sync for RenderObj {}

pub struct RenderObjs {
	pub list: Slab<RenderObj>,
	// pub opacity_dirty: bool,
	// pub transparent_dirty: bool,
	// pub dirty: bool,
	// pub program_dirtys: Vec<usize>,
}

impl std::ops::Deref for RenderObjs {
	type Target = Slab<RenderObj>;

    fn deref(&self) -> &Self::Target {
		&self.list
	}
}

impl std::ops::DerefMut for RenderObjs {
    fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.list
	}
}

pub struct RenderObjType;

impl Default for RenderObjs {
    fn default() -> Self {
		Self{
			list: Slab::default(),
			// opacity_dirty: false,
			// transparent_dirty: false,
			// dirty: false,
			// program_dirtys: Vec::default(),
		}
    }
}

impl RenderObjs {
	pub fn with_capacity(capacity: usize) -> Self {
		Self{
			list: Slab::with_capacity(capacity),
			// opacity_dirty: false,
			// transparent_dirty: false,
			// dirty: false,
			// program_dirtys: Vec::default(),
		}
    }
    pub fn mem_size(&self) -> usize {
        self.list.mem_size()
    }
    pub fn insert(&mut self, value: RenderObj) -> usize {
        let id = self.list.insert(value);
        // match notify {
        //     Some(n) => n.send(RenderObjEvent::new_create(id)),
        //     _ => (),
        // };
        id
    }

    pub unsafe fn remove_unchecked(&mut self, id: usize) {
        self.list.remove(id);
        // match notify {
        //     Some(n) => n.send(RenderObjEvent::new_delete(id)),
        //     _ => (),
        // };
    }

    pub fn remove(&mut self, id: usize,) {
        if self.list.contains(id) {
            self.list.remove(id);
            // match notify {
            //     Some(n) => n.send(RenderObjEvent::new_delete(id)),
            //     _ => (),
            // };
        }
    }

	// pub fn modify(&mut self, id: usize, field: &'static str) {
	// 	self.dirty = true;
	// 	let obj = match self.list.get_mut(id) {
	// 		Some(r) => r,
	// 		None => return, // obj可能不存在
	// 	};
	// 	match field {
	// 		"depth" => {
	// 			if obj.is_opacity == false {
	// 				self.transparent_dirty = true;
	// 			}
	// 		}
	// 		"program_dirty" => {
	// 			if obj.is_opacity == true {
	// 				self.opacity_dirty = true;
	// 			}
	// 			if obj.program_dirty == false {
	// 				self.program_dirtys.push(id);
	// 				obj.program_dirty = true;
	// 			}
	// 		}
	// 		"is_opacity" => {
	// 			self.opacity_dirty = true;
	// 			self.transparent_dirty = true;
	// 		}
	// 		"visibility" => {
	// 			if obj.is_opacity {
	// 				self.opacity_dirty = true;
	// 			} else {
	// 				self.transparent_dirty = true;
	// 			}
	// 		}
	// 		_ => (),
	// 	}
	// }

	// pub fn create(&mut self, id: usize) {
	// 	self.dirty = true;
	// 	let obj = &mut self.list[id];
	// 	if obj.is_opacity == false {
	// 		self.transparent_dirty = true;
	// 	} else {
	// 		self.opacity_dirty = true;
	// 	}
	// 	self.program_dirtys.push(id);
	// 	obj.program_dirty = true;
	// }

	// pub fn delete(&mut self, id: usize) {
	// 	self.dirty = true;
	// 	let obj = &self.list[id];
	// 	if obj.is_opacity == false {
	// 		self.transparent_dirty = true;
	// 	} else {
	// 		self.opacity_dirty = true;
	// 	}
	// }

	// 	match e.ty {
	// 		EventType::Create => {
	// 			local.dirty = true;
	// 			let obj = &mut render_objs[e.id];
	// 			if obj.is_opacity == false {
	// 				local.transparent_dirty = true;
	// 			} else {
	// 				local.opacity_dirty = true;
	// 			}
	// 			local.program_dirtys.push(e.id);
	// 			obj.program_dirty = true;
	// 		},
	// 		EventType::Modify => {
	// 			local.dirty = true;
	// 			let obj = match render_objs.get_mut(e.id) {
	// 				Some(r) => r,
	// 				None => return, // obj可能不存在
	// 			};
	// 			match e.field {
	// 				"depth" => {
	// 					if obj.is_opacity == false {
	// 						local.transparent_dirty = true;
	// 					}
	// 				}
	// 				"program_dirty" => {
	// 					if obj.is_opacity == true {
	// 						local.opacity_dirty = true;
	// 					}
	// 					if obj.program_dirty == false {
	// 						local.program_dirtys.push(e.id);
	// 						obj.program_dirty = true;
	// 					}
	// 				}
	// 				"is_opacity" => {
	// 					local.opacity_dirty = true;
	// 					local.transparent_dirty = true;
	// 				}
	// 				"visibility" => {
	// 					if obj.is_opacity {
	// 						local.opacity_dirty = true;
	// 					} else {
	// 						local.transparent_dirty = true;
	// 					}
	// 				}
	// 				_ => (),
	// 			}
	// 		}
	// 		EventType::Delete => {
	// 			local.dirty = true;
	// 			let obj = &render_objs[e.id];
	// 			if obj.is_opacity == false {
	// 				local.transparent_dirty = true;
	// 			} else {
	// 				local.opacity_dirty = true;
	// 			}
	// 		}
	// 	}
	// }

	// pub fn get_write<'a>(
    //     &'a mut self,
    //     id: usize,
    //     notify: &'a EventWriter<Events>,
    // ) -> Write<RenderObj> {
    //     unsafe { Write::new(id, self.0.get_unchecked_mut(id), &notify) }
    // }
    // pub unsafe fn get_unchecked_write<'a>(
    //     &'a mut self,
    //     id: usize,
    //     notify: &'a EventWriter<Events>,
    // ) -> Write<RenderObj> {
    //     Write::new(id, self.0.get_unchecked_mut(id), &notify)
    // }

    pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut RenderObj {
        self.list.get_unchecked_mut(id)
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

	pub fn add(&mut self, node_id: usize, render_id: usize, /*notify: &EventWriter<EntityEvent<Self>>*/) {
        self.0[node_id].push(render_id);
        // notify.send(EntityEvent::Modify(node_id, "add", render_id));
	}

    pub fn remove(
        &mut self,
        node_id: usize,
        render_id: usize,
        /*notify: &EventWriter<EntityEvent<Self>>,*/
    ) {
        // notify.send(EntityEvent::Modify(node_id, "remove", render_id));
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

// 强行实现Send、Sync 修改?TODO
unsafe impl Send for RenderBegin {}
unsafe impl Sync for RenderBegin {}

/// 脏区域，描述了界面发生修改的区域，用于优化界面局部修改时，进渲染该区域
/// 该区域以根节点的原点最为原点
#[derive(Debug)]
pub struct DirtyViewRect(pub f32, pub f32, pub f32, pub f32, pub bool/*是否与最大视口相等（RenderBeginDesc中的视口）*/);

#[derive(Default)]
pub struct Statistics {
    pub drawcall_times: usize,
}

pub enum SingleChangeLabel {
	FontTexture,
	ImageWaitSheet,
	RenderBegin,
	ProjectionMatrix
}

#[derive(Default)]
pub struct SingleChangeType(pub usize);

#[derive(Default)]
pub struct SystemTime {
	pub cur_time: usize,
}

#[derive(Deref, DerefMut, Default)]
pub struct IdTree(idtree::IdTree<u32>);

pub fn to_entity(id: usize, generation: u32) -> Entity {
	Entity::from_bits(u64::from(generation as u32) << 32 | u64::from(id as u32))
}

impl IdTree {
	pub fn with_capacity(capacity: usize) -> Self {
        Self(idtree::IdTree::with_capacity(capacity))
	}
	
	pub fn insert_child_with_notify(&mut self, child: usize, parent:usize, index: usize, notify: &mut EventWriter<EntityEvent<Self>>) -> &Node<u32> {
		self.insert_child(child, parent, index);
		let node = &self[child];
		// if node.layer() > 0 {
		// 	notify.send(EntityEvent::new_create(to_entity(child, node.data)));
		// } else {
		// 	notify.send(EntityEvent::new_modify(to_entity(child, node.data), "add", 0));
		// }
		node
	}

	pub fn insert_brother_with_notify(&mut self, child: usize, brother:usize, index: idtree::InsertType, notify: &mut EventWriter<EntityEvent<Self>>) -> &Node<u32> {
		self.insert_brother(child, brother, index);
		let node = &self[child];
		// if node.layer() > 0 {
		// 	notify.send(EntityEvent::new_create(to_entity(child, node.data)));
		// } else {
		// 	notify.send(EntityEvent::new_modify(to_entity(child, node.data), "add", 0));
		// }
		node
	}

	pub fn remove_with_notify(&mut self, id: usize, notify: &mut EventWriter<EntityEvent<Self>>) -> Option<usize> {
		let r = match self.get(id) {
			Some(n) => {
				if n.parent() == 0 && n.layer() == 0 {
					return None;
				}
				((n.parent(), n.layer(), n.count(), n.prev(), n.next(), n.children().head), n.data)
			}
			_ => return None,
		};
		IdTree::notify_move(id, r.1, (r.0).1, notify);
		self.remove(id, r.0);
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

	fn notify_move(id: usize, version: u32, layer: usize, notify: &mut EventWriter<EntityEvent<Self>>){
		// if layer > 0 {
		// 	notify.send(EntityEvent::new_delete(to_entity(id, version)))
		// } else {
		// 	notify.send(EntityEvent::new_modify(to_entity(id, version), "remove", layer))
		// }
	}
}