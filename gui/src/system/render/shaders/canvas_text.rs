use atom::Atom;

// 渲染文字
lazy_static! {
    pub static ref CANVAS_TEXT_SHADER_NAME: Atom = Atom::from("canvas_text");
    pub static ref CANVAS_TEXT_FS_SHADER_NAME: Atom = Atom::from("canvas_text_fs");
    pub static ref CANVAS_TEXT_VS_SHADER_NAME: Atom = Atom::from("canvas_text_vs");
}