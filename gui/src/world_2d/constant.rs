use atom::Atom;

lazy_static! {
    //defines
    pub static ref POSITION: Atom = Atom::from("position");
    pub static ref WORLD_VIEW_PROJECTION: Atom = Atom::from("worldViewProjection");
    pub static ref PROJECTION: Atom = Atom::from("Projection");
    pub static ref ALPHA: Atom = Atom::from("alpha");
    pub static ref CLIP_INDEICES: Atom = Atom::from("clipIndices");
    pub static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    pub static ref CLIP_INDEICES_SIZE: Atom = Atom::from("clipTextureSize");
    pub static ref SCREEN_SIZE: Atom = Atom::from("screenSize");
}