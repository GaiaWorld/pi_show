use atom::Atom;

// 渲染图片
lazy_static! {
    pub static ref IMAGE_SHADER_NAME: Atom = Atom::from("image");
    pub static ref IMAGE_FS_SHADER_NAME: Atom = Atom::from("image_fs");
    pub static ref IMAGE_VS_SHADER_NAME: Atom = Atom::from("image_vs");

	pub static ref FBO_FS_SHADER_NAME: Atom = Atom::from("fbo_fs");
    pub static ref FBO_VS_SHADER_NAME: Atom = Atom::from("fbo_vs");

	pub static ref CANVAS_SHADER_NAME: Atom = Atom::from("canvas");
    pub static ref CANVAS_FS_SHADER_NAME: Atom = Atom::from("canvas_fs");
    pub static ref CANVAS_VS_SHADER_NAME: Atom = Atom::from("canvas_vs");
}