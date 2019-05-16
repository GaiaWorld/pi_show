use component::Point2;

pub mod oct;

#[derive(Debug)]
pub struct OverflowClip{
    pub id_vec: [usize;8],
    pub clip: [[Point2;4];8],
}
