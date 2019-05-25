use atom::Atom;

// 渲染图片
lazy_static! {
    pub static ref Image_SHADER_NAME: Atom = Atom::from("image");
    pub static ref Image_FS_SHADER_NAME: Atom = Atom::from("image_fs");
    pub static ref Image_VS_SHADER_NAME: Atom = Atom::from("image_vs");
}