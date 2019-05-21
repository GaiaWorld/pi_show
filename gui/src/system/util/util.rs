use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl};

use component::Matrix4;
use component::user::Transform;
use component::calc::WorldMatrix;
use layout::Layout;
use Node;

fn cal_matrix(
    id: usize,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    layouts: &&MultiCaseImpl<Node, Layout>,
    mut offset: (f32, f32)
) -> Matrix4 {
    let world_matrix = unsafe { world_matrixs.get_unchecked(id) };
    let transform = unsafe { transforms.get_unchecked(id) };
    let layout = unsafe { layouts.get_unchecked(id) };

    let origin = transform.origin.to_value(layout.width, layout.height);
    offset.0 -= origin.x;
    offset.1 -= origin.y;

    if offset.0 != 0.0 || offset.1 != 0.0 {
        return world_matrix.0 * Matrix4::from_translation(cg::Vector3::new(offset.0, offset.1, 0.0));
    }
    
    world_matrix.0.clone()
}