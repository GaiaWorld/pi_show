use std::any::TypeId;

use pi_assets::{allocator::Allocator, asset::{Asset, GarbageEmpty, Garbageer, Size}, homogeneous::HomogeneousMgr, mgr::AssetMgr};
use pi_hash::XHashMap;
use pi_share::Share;

/// 资产配置
#[derive(Debug, Clone, Default)]
pub struct AssetConfig (XHashMap<TypeId, AssetDesc>);

impl AssetConfig {
	// 为某类型的资产管理器配置容量和超时时间
	#[inline]
    pub fn insert<T: 'static>(&mut self, cfg: AssetDesc) {
        self.0.insert(std::any::TypeId::of::<T>(), cfg);
    }

	// 取到某类型的资产管理器的容量、超时配置
	#[inline]
    pub fn get<T: 'static>(&self) -> Option<&AssetDesc> {
		self.0.get(&std::any::TypeId::of::<T>())
    }
}

/// 资产容量和超时描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDesc {
	pub min: usize,
    pub weight: usize, // 权重
	pub timeout: usize,
	pub ref_garbage: bool,
}


/// 资源、资产管理器
#[derive( Deref, DerefMut)]
pub struct ShareAssetMgr<A: Asset, G: Garbageer<A> = GarbageEmpty>(pub Share<AssetMgr<A, G>>);

impl<A: Asset, G: Garbageer<A>> ShareAssetMgr<A, G> {
	pub fn new_with_config(garbage: G, default: &AssetDesc, asset_config: &AssetConfig, allocator: &mut Allocator) -> Self {
		let desc = asset_config.get::<A>().unwrap_or(default);
		let r = AssetMgr::new(garbage, desc.ref_garbage, desc.min, desc.timeout);
		allocator.register(r.clone(), desc.min, desc.weight);
		Self(r)
	}

    /// 用指定的参数创建资产管理器， ref_garbage为是否采用引用整理
    pub fn new(garbage: G, ref_garbage: bool, capacity: usize, timeout: usize) -> Self {
		Self(AssetMgr::new(garbage, ref_garbage, capacity, timeout))
	}
}

impl<A: Asset, G: Garbageer<A>> Clone for ShareAssetMgr<A, G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


/// 资源， 同质资产管理器
#[derive( Deref, DerefMut)]
pub struct ShareHomogeneousMgr<A: Size, G: pi_assets::homogeneous::Garbageer<A> = pi_assets::homogeneous::GarbageEmpty>(pub Share<HomogeneousMgr<A, G>>);

impl<A: Asset + Size, G: pi_assets::homogeneous::Garbageer<A>> ShareHomogeneousMgr<A, G> {
    /// 用指定的参数创建资产管理器， ref_garbage为是否采用引用整理
    pub fn new(garbage: G, capacity: usize, timeout: usize) -> Self {
		Self(HomogeneousMgr::new(garbage, capacity, timeout))
	}

	pub fn new_with_config(garbage: G, default: &AssetDesc, asset_config: &AssetConfig, allocator: &mut Allocator) -> Self {
		let desc = asset_config.get::<A>().unwrap_or(default);
		let r = HomogeneousMgr::new(garbage, desc.min, desc.timeout);
		allocator.register(r.clone(), desc.min, desc.weight);
		Self(r)
	}
}

impl<A: Size, G: pi_assets::homogeneous::Garbageer<A>> Clone for ShareHomogeneousMgr<A, G> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}