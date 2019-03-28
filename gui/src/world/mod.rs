#[cfg(feature = "web")]
mod shader;
#[cfg(feature = "web")]
mod have_web;

#[cfg(not(feature = "web"))]
mod no_web;

#[cfg(feature = "web")]
pub use self::have_web::*;
#[cfg(feature = "web")]
pub use self::shader::*;

#[cfg(not(feature = "web"))]
pub use no_web::*;