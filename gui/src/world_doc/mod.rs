pub mod component;
pub mod system;

#[cfg(feature = "web")]
mod have_web;

#[cfg(not(feature = "web"))]
mod no_web;

#[cfg(feature = "web")]
pub use world_doc::have_web::*;

#[cfg(not(feature = "web"))]
pub use world_doc::no_web::*;