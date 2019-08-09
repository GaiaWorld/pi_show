use atom::Atom;

lazy_static! {
    //common attribute
    pub static ref POSITION: Atom = Atom::from("position");
    pub static ref INDEX: Atom = Atom::from("index");
    pub static ref COLOR: Atom = Atom::from("color");

    pub static ref POSITIONUNIT: Atom = Atom::from("position_unit");
    pub static ref INDEXUNIT: Atom = Atom::from("index_unit");

    //common uniform
    pub static ref VIEW_MATRIX: Atom = Atom::from("viewMatrix");
    pub static ref PROJECT_MATRIX: Atom = Atom::from("projectMatrix");
    pub static ref WORLD_MATRIX: Atom = Atom::from("worldMatrix");
    pub static ref CLIP_INDICES: Atom = Atom::from("clipindices");
    pub static ref UV: Atom = Atom::from("uv");
    pub static ref ALPHA: Atom = Atom::from("alpha");
    pub static ref TEXTURE: Atom = Atom::from("texture");
    pub static ref UV_OFFSET_SCALE: Atom = Atom::from("uvOffsetScale");
    
    
    // ubo name
    pub static ref VIEW: Atom = Atom::from("VIEW");
    pub static ref PROJECT: Atom = Atom::from("PROJECT");
    pub static ref WORLD: Atom = Atom::from("WORLD");
    pub static ref CLIP: Atom = Atom::from("CLIP");
    pub static ref COMMON: Atom = Atom::from("COMMON");

    // clip uniform
    pub static ref CLIP_TEXTURE: Atom = Atom::from("clipTexture");
    pub static ref CLIP_INDICES_SIZE: Atom = Atom::from("clipTextureSize");

    // 四边形顶点流
    pub static ref QUAD_POSITION_INDEX: Atom = Atom::from("quad_position_index");

    pub static ref RADIUS_QUAD_POSITION_INDEX: Atom = Atom::from("radius_quad_position_index");
    pub static ref RADIUS: Atom = Atom::from("radius");
}