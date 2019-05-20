/// 中间计算的组件

use std::{
	f32,
	default::Default,
};

use map::{vecmap::VecMap};
use ecs::component::Component;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ZDepth(pub f32);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ByOverflow(pub usize);

#[derive(Debug, Clone, Component, Default, Deref, DerefMut)]
pub struct WorldMatrix(pub super::Matrix4);

//是否可见， 不可见时， 也会占据布局位置
#[derive(Deref, DerefMut, Component, Debug, Default)]
pub struct Visibility(pub bool);

//不透明度
#[derive(Deref, DerefMut, Component, Debug)]
pub struct Opacity(pub f32);

impl Default for Opacity {
	fn default() -> Opacity{
	Opacity(1.0)
	}
}

//是否响应事件
#[derive(Deref, DerefMut, Component, Debug, Default)]
pub struct Enable(pub bool);