// use stdweb::web::html_element::CanvasElement;
#[cfg(feature = "web")]
mod have_web;

#[cfg(feature = "web")]
mod shader;

#[cfg(not(feature = "web"))]
mod no_web;

#[cfg(not(feature = "web"))]
pub use world::no_web::*;

#[cfg(feature = "web")]
pub use world::have_web::*;
#[cfg(feature = "web")]
pub use world::shader::*;