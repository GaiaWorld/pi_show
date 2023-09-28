use pi_atom::Atom;

// 渲染单色
lazy_static! {
    pub static ref CLIP_SHADER_NAME: Atom = Atom::from("clip");
    pub static ref CLIP_FS_SHADER_NAME: Atom = Atom::from("clip_fs");
    pub static ref CLIP_VS_SHADER_NAME: Atom = Atom::from("clip_vs");
}