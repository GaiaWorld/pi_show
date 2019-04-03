pub mod component;
#[cfg(feature = "web")]
pub mod system;
#[cfg(feature = "web")]
pub mod shaders;

#[cfg(feature = "web")]
mod have_web;

#[cfg(not(feature = "web"))]
mod no_web;

#[cfg(feature = "web")]
pub use object2d::have_web::*;

#[cfg(not(feature = "web"))]
pub use object2d::no_web::*;