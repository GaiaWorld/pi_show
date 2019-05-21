use atom::Atom;

lazy_static! {
    //common attribute
    pub static ref POSITION: Atom = Atom::from("position");
    pub static ref COLOR: Atom = Atom::from("color");

    //common uniform
    pub static ref VIEW_MATRIX: Atom = Atom::from("view_matrix");
    pub static ref PROJECT_MATRIX: Atom = Atom::from("project_matrix");
    pub static ref WORLD_MATRIX: Atom = Atom::from("world_matrix");
    pub static ref CLIP_INDEICES: Atom = Atom::from("clipIndices");
    pub static ref UV: Atom = Atom::from("uv");
    pub static ref ALPHA: Atom = Atom::from("alpha");
    
    // ubo name
    pub static ref VIEW: Atom = Atom::from("VIEW");
    pub static ref PROJECT: Atom = Atom::from("PROJECT");
    pub static ref WORLD: Atom = Atom::from("WORLD");
    pub static ref CLIP: Atom = Atom::from("CLIP");
    pub static ref COMMON: Atom = Atom::from("COMMON");

    // clip uniform
    pub static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    pub static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
}