use atom::Atom;

// 渲染文字
lazy_static! {
    pub static ref TEXT_SHADER_NAME: Atom = Atom::from("text");
    pub static ref TEXT_FS_SHADER_NAME: Atom = Atom::from("text_fs");
    pub static ref TEXT_VS_SHADER_NAME: Atom = Atom::from("text_vs");
}