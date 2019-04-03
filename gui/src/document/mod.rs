pub mod component;
pub mod system;

#[cfg(feature = "web")]
mod have_web;

#[cfg(not(feature = "web"))]
mod no_web;

#[cfg(feature = "web")]
pub use document::have_web::*;

#[cfg(not(feature = "web"))]
pub use document::no_web::*;