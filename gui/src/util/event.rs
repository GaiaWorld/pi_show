use std::{marker::PhantomData};
use bevy_ecs::{archetype::{Archetype, ArchetypeGeneration, ArchetypeId}, component::Component, prelude::{Entity, ResMut, System, World}, system::{ ResMutState, SystemMeta, SystemParam, SystemParamFetch, SystemParamState}};
use crate::component::calc::StyleIndex;

#[derive(Clone, Debug)]
pub enum EventType {
	Create,
	Modify,
	Delete,
}

#[derive(Clone, Debug)]
pub struct EntityEventData {
	pub ty: EventType,
	pub style_index: StyleIndex,
	pub id: Entity,
}

impl EntityEventData {
	fn new(id: Entity, ty: EventType, style_index: StyleIndex) -> Self {
		EntityEventData {
			id,
			ty,
			style_index
		}
	}
}

#[derive(Debug)]
pub struct EntityEvent<T>{
	value: EntityEventData,
	marker: PhantomData<T>,
}

impl<T> Clone for EntityEvent<T> {
	fn clone(&self) -> Self {
		Self{
			value: self.value.clone(),
			marker: PhantomData,
		}
	}
}

impl<T> EntityEvent<T> {
	pub fn new_create(id: Entity) -> Self {
		EntityEvent {
			value: EntityEventData::new(id, EventType::Create, StyleIndex::Create),
			marker: PhantomData,
		}
	}

	pub fn new_modify(id: Entity, style_index: StyleIndex) -> Self {
		EntityEvent {
			value: EntityEventData::new(id, EventType::Modify, style_index),
			marker: PhantomData,
		}
	}

	pub fn new_delete(id: Entity) -> Self {
		EntityEvent {
			value: EntityEventData::new(id, EventType::Delete, StyleIndex::Delete),
			marker: PhantomData,
		}
	}
}

impl<T> std::ops::Deref for EntityEvent<T> {
	type Target = EntityEventData;

    fn deref(&self) -> &Self::Target {
		&self.value
	}
}

pub struct ResModifyEvent<T>(PhantomData<T>);
impl<T> ResModifyEvent<T> {
	pub fn new() -> Self {
		ResModifyEvent(PhantomData)
	}
}
impl<T> Clone for ResModifyEvent<T> {
	fn clone(&self) -> Self {
		Self(PhantomData)
	}
}

pub struct Children;

pub struct IdTreeEvent{
	pub ty: EventType,
	pub id: Entity,
	pub parent: Entity,
	pub field: &'static str,
}

#[derive(Clone)]
pub struct RenderObjEvent{
	pub ty: EventType,
	pub id: usize,
	pub field: &'static str,
	pub index: usize,
}

impl RenderObjEvent {
	pub fn new_create(id: usize) -> Self {
		RenderObjEvent {
			ty: EventType::Create,
			id,
			field: "",
			index: 0,
		}
	}

	pub fn new_modify(id: usize, field: &'static str, index: usize) -> Self {
		RenderObjEvent {
			ty: EventType::Modify,
			id,
			field,
			index,
		}
	}

	pub fn new_delete(id: usize) -> Self {
		RenderObjEvent {
			ty: EventType::Delete,
			id,
			field: "",
			index: 0,
		}
	}
}

/// 立即事件, 事件发生时，立即通知监听器
pub struct ImEvent<E: Clone + 'static> {
	listeners: Vec<Box<dyn System<In = E, Out = ()>>>, // 监听器，按类型分类
	// global_listeners: Vec<Box<dyn System<In = E, Out = ()>>>,
	archetype_generation: ArchetypeGeneration,
	uninitialized_start: usize,
}

impl<E: Clone + 'static> Default for ImEvent<E> {
	fn default() -> Self {
		ImEvent {
			listeners: Vec::new(),
			archetype_generation: ArchetypeGeneration::initial(),
			uninitialized_start: 0,
		}
	}
}

impl<E: Clone + 'static> ImEvent<E> {
	pub fn send(&mut self, e: E, world: &mut World) {
		if self.listeners.len() > 0 {
			// 初始化
			if self.uninitialized_start < self.listeners.len() {
				for i in self.uninitialized_start..self.listeners.len() {
					self.listeners[i].initialize(world);
					self.uninitialized_start += 1;
				}
			}
			
			// 更新原型
			self.update_archetypes(world);
			// 运行监听
			for l in self.listeners.iter_mut() {
				l.run(e.clone(), world);
			}
		}
	}

	pub fn add_listener<Sys: System<In = E, Out = ()>>(&mut self, sys: Sys) {
		self.listeners.push(Box::new(sys));
	}

	fn update_archetypes(&mut self, world: &mut World) {
		let archetypes = world.archetypes();
        let new_generation = archetypes.generation();
		if self.archetype_generation == new_generation {
			return;
		}
        let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
        let archetype_index_range = old_generation.value()..new_generation.value();

		for i in archetype_index_range {
			let archetype = archetypes.get(ArchetypeId::new(i)).unwrap();
			for l in self.listeners.iter_mut() {
				l.new_archetype(archetype);
			}
		}
	}
}

// 发送一个事件（用于外部，非system内， system可以通过fetch <ImMessenger>来发送事件，w）
pub fn send_im_event<E: Clone + 'static>(world: &mut World, e: E) {
	let w = unsafe{&mut *(world as *mut World)};
	if let Some(mut res) = world.get_resource_mut::<ImEvent<E>>() {
		res.send(e, w);
	}
}

// 添加监听器
pub fn add_listener<E: Clone + 'static, Sys: System<In = E, Out = ()>>(world: &mut World, sys: Sys) {
	if let None = world.get_resource::<ImEvent<E>>() {
		world.insert_resource(ImEvent::<E>::default());
	}
	world.get_resource_mut::<ImEvent<E>>().unwrap().add_listener(sys);
}

// 立即事件消息传递者， 实现了SystemParam参数
pub struct ImMessenger<'w, E: 'static + Clone> {
    value: ResMut<'w, ImEvent<E>>,
	world: &'w World,
}

impl<'w, E: 'static + Clone> ImMessenger<'w, E>  {
	/// 发送事件
	pub fn send(&mut self, e: E) {
		let w = unsafe{&mut *(self.world as *const World as usize as *mut World)};
		self.value.send(e, w);
	}
}

impl<'w, E: Component + Clone> std::ops::Deref for ImMessenger<'w, E> {
    type Target = ImEvent<E>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'w, E: Component + Clone> std::ops::DerefMut for ImMessenger<'w, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// The [`SystemParamState`] of [`Res`].
pub struct ImMessengerState<E: Component + Clone> {
	state: ResMutState<ImEvent<E>>,
    // component_id: ComponentId,
    marker: PhantomData<E>,
}

impl<'a, E: Component + Clone> SystemParam for ImMessenger<'a, E> {
    type Fetch = ImMessengerState<E>;
}

// SAFE: Res ComponentId and ArchetypeComponentId access is applied to SystemMeta. If this Res
// conflicts with any prior access, a panic will occur.
unsafe impl<E: Component + Clone> SystemParamState for ImMessengerState<E> {
    type Config = ();

    fn init(world: &mut World, system_meta: &mut SystemMeta, config: Self::Config) -> Self {
		if let None = world.get_resource::<ImEvent<E>>() {
			world.insert_resource(ImEvent::<E>::default());
		}
        Self{
			state: ResMutState::<ImEvent<E>>::init(world, system_meta, config),
			marker: std::marker::PhantomData,
		}
    }

    fn new_archetype(&mut self, archetype: &Archetype, system_meta: &mut SystemMeta) {
		self.state.new_archetype(archetype, system_meta)
	}

	fn default_config() {}

	fn apply(&mut self, world: &mut World) {
		self.state.apply(world)
	}
}

impl<'a, E: Component + Clone> SystemParamFetch<'a> for ImMessengerState<E> {
    type Item = ImMessenger<'a, E>;

    #[inline]
    unsafe fn get_param(
        state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
        change_tick: u32,
    ) -> Self::Item {
		ImMessenger {
            value: ResMutState::<ImEvent<E>>::get_param(&mut state.state, system_meta, world, change_tick),
            world: world,
        }
    }
}
