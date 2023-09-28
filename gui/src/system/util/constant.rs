use pi_atom::Atom;

lazy_static! {

    pub static ref POSITIONUNIT: Atom = Atom::from("position_unit");
    pub static ref INDEXUNIT: Atom = Atom::from("index_unit");

    // 四边形顶点流
    pub static ref QUAD_POSITION_INDEX: Atom = Atom::from("quad_position_index");

    pub static ref RADIUS_QUAD_POSITION_INDEX: Atom = Atom::from("radius_quad_position_index");
}