use atom::Atom;

// 渲染单色
lazy_static! {
    pub static ref COLOR_SHADER_NAME: Atom = Atom::from("color");
    pub static ref COLOR_FS_SHADER_NAME: Atom = Atom::from("color_fs");
    pub static ref COLOR_VS_SHADER_NAME: Atom = Atom::from("color_vs");
}