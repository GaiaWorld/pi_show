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


	pub static ref BLUR_DOWN_SHADER_NAME: Atom = Atom::from("blur_down");
    pub static ref BLUR_DOWN_FS_SHADER_NAME: Atom = Atom::from("blur_down_fs");
    pub static ref BLUR_DOWN_VS_SHADER_NAME: Atom = Atom::from("blur_down_vs");

	pub static ref BLUR_UP_SHADER_NAME: Atom = Atom::from("blur_up");
    pub static ref BLUR_UP_FS_SHADER_NAME: Atom = Atom::from("blur_up_fs");
    pub static ref BLUR_UP_VS_SHADER_NAME: Atom = Atom::from("blur_up_vs");

	pub static ref GAUSS_BLUR_SHADER_NAME: Atom = Atom::from("gauss_blur");
    pub static ref GAUSS_BLUR_FS_SHADER_NAME: Atom = Atom::from("gauss_blur_fs");
    pub static ref GAUSS_BLUR_VS_SHADER_NAME: Atom = Atom::from("gauss_blur_vs");
}