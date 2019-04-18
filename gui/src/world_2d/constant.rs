use atom::Atom;

lazy_static! {
    //defines
    pub static ref POSITION: Atom = Atom::from("position");
    pub static ref UV: Atom = Atom::from("uv");
    pub static ref WORLD: Atom = Atom::from("world");
    pub static ref PROJECTION: Atom = Atom::from("projection");
    pub static ref VIEW: Atom = Atom::from("view");
    pub static ref ALPHA: Atom = Atom::from("alpha");
    pub static ref CLIP_INDEICES: Atom = Atom::from("clipIndices");
    pub static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    pub static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
    pub static ref SCREEN_SIZE: Atom = Atom::from("screenSize");
}