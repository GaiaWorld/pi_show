/**
 * 监听TransformWillChange的改变， 修改TransformWillChangeMatrix
 */

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Runner};
use ecs::idtree::{ IdTree};
use dirty::LayerDirty;

use component::user::{ Transform };
use component::calc::{ WorldMatrix, StyleMark, StyleType1, TransformWillChangeMatrix, Layout };
use single::DefaultTable;

use component::user::*;
use entity::{Node};

#[derive(Default)]
pub struct TransformWillChangeSys{
    will_change_mark: LayerDirty, // 存放存在TransformWillChang的组件id， 并不是为了记脏
    dirty: bool,
    // TransformWillChang创建时， 如果该节点不在根树上， 该值会+1， 当节点树被添加到根上时， 遍历子孙节点，如果节点上存在TransformWillChang， 该值-1， 直到减为0， 会停止遍历
    // 因为存在TransformWillChang的节点数量应该是少量的， 记录该值， 可以减少每次节点被添加到根时的遍历
    create_will_change_count: usize, 
}

impl TransformWillChangeSys{

    #[inline]
    fn mark_dirty(&mut self, id: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
        self.dirty = true;
        let mark = unsafe { style_marks.get_unchecked_mut(id) };
        mark.dirty1 |= StyleType1::TransformWillChange as usize;
    }
}

impl<'a> Runner<'a> for TransformWillChangeSys{
    type ReadData = (
        &'a SingleCaseImpl<DefaultTable>,
        &'a SingleCaseImpl<IdTree>,
        &'a MultiCaseImpl<Node, Layout>,
        &'a MultiCaseImpl<Node, TransformWillChange>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
    );
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut MultiCaseImpl<Node, TransformWillChangeMatrix>,
    );
    fn run(&mut self, read: Self::ReadData, write: Self::WriteData){
        if !self.dirty {
            return;
        }
        // let time = std::time::Instant::now();
        self.dirty = false;
        let (default_table, idtree, layouts, transform_will_changes, world_matrixs, transforms) = read;
        let (style_marks, transform_will_change_matrixs) = write;
        let mut count = self.will_change_mark.count();
        let default_transform = default_table.get::<Transform>().unwrap();
        for id in self.will_change_mark.iter() {
            if count == 0 {
                break;
            }
            count -= 1;
            let style_mark = match style_marks.get_mut(*id) {
                Some(r) => r,
                None => continue,
            };

            if style_mark.dirty1 | StyleType1::TransformWillChange as usize == 0 {
                continue;
            }
            // 如果节点已经从节点数中移除， 不需要计算矩阵， 跳过
            let parent_id = unsafe { idtree.get_unchecked(*id).parent };
            recursive_cal_matrix(
                *id,
                parent_id, 
                count + 1, 
                None, 
                default_transform, 
                idtree,
                transform_will_changes, 
                layouts, 
                world_matrixs, 
                transforms, 
                style_marks,
                transform_will_change_matrixs);
        }
        // println!("TransformWillChangeSys run : {:?}", std::time::Instant::now() - time);
    }
}

impl<'a> MultiCaseListener<'a, Node, TransformWillChange, CreateEvent> for TransformWillChangeSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
        if let Some(r) = idtree.get(event.id) {
            self.mark_dirty(event.id, style_marks);
            self.will_change_mark.mark(event.id, r.layer);
        } else {
            self.create_will_change_count += 1;
        } 
    }
}

impl<'a> MultiCaseListener<'a, Node, TransformWillChange, ModifyEvent> for TransformWillChangeSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, StyleMark>;
    fn listen(&mut self, event: &ModifyEvent, idtree: Self::ReadData, style_marks: Self::WriteData){
        if let Some(_r) = idtree.get(event.id) {
            self.mark_dirty(event.id, style_marks);
        }
    }
}

// 删除TransformWillChange组件， 标记脏
impl<'a> MultiCaseListener<'a, Node, TransformWillChange, DeleteEvent> for TransformWillChangeSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, TransformWillChangeMatrix>;
    fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, transform_will_change_matrix: Self::WriteData){
        if let Some(r) = idtree.get(event.id) {
            self.will_change_mark.delete(event.id, r.layer);
			if transform_will_change_matrix.get(event.id).is_some() {
				transform_will_change_matrix.delete(event.id);
			}
            // self.mark_dirty(event.id, style_marks);
        }
    }
}

// 从IdTree中移除节点， 标记脏, 在下一次run的调用中， 删除已经不存在的节点的willchange标记
impl<'a> SingleCaseListener<'a, IdTree, DeleteEvent> for TransformWillChangeSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = &'a mut MultiCaseImpl<Node, TransformWillChange>;
    fn listen(&mut self, event: &DeleteEvent, idtree: Self::ReadData, willchanges: Self::WriteData){
        if self.will_change_mark.count() > 0 {
            return;
        }

        // 如果存在TransformWillChange组件， 需要从标记列表中移除
        let node = unsafe { idtree.get_unchecked(event.id) };
        if let Some(_willchange) = willchanges.get_mut(event.id) {
            self.will_change_mark.delete(event.id, node.layer);
        }

        let first = node.children.head;
        for (child_id, child) in idtree.recursive_iter(first) {
            if let Some(_willchange) = willchanges.get_mut(child_id) {
                self.will_change_mark.delete(child_id, child.layer);
            }
        }
    }
}

//  IdTree创建， 递归遍历所有子节点， 如果存在TransformWillChange组件， 在will_change_mark中添加一个标记
impl<'a> SingleCaseListener<'a, IdTree, CreateEvent> for TransformWillChangeSys{
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut MultiCaseImpl<Node, TransformWillChange>,
        &'a mut MultiCaseImpl<Node, StyleMark>,
    );
    fn listen(&mut self, event: &CreateEvent, idtree: Self::ReadData, write: Self::WriteData){
        if self.create_will_change_count == 0 {
            return;
        }
        let (willchanges, style_marks) = write;
        let node = unsafe { idtree.get_unchecked(event.id) };
        if let Some(_willchange) = willchanges.get_mut(event.id) {
            self.mark_dirty(event.id, style_marks);
            self.will_change_mark.mark(event.id, node.layer);
            self.create_will_change_count -= 1;
            if self.create_will_change_count == 0 {
                return;
            }
        }
 
        let first = node.children.head;
        for (child_id, child) in idtree.recursive_iter(first) {
            if let Some(_willchange) = willchanges.get_mut(child_id) {
                self.mark_dirty(event.id, style_marks);
                self.will_change_mark.mark(child_id, child.layer);
                self.create_will_change_count -= 1;
                if self.create_will_change_count == 0 {
                    break;
                }
            }
        } 
    }
}

fn recursive_cal_matrix(
    id: usize,
    parent: usize,
    mut count: usize,
    parent_will_change_matrix: Option<&WorldMatrix>,
    default_transform: &Transform,
    idtree: &IdTree,
    willchange:  &MultiCaseImpl<Node, TransformWillChange>,
    layouts: &MultiCaseImpl<Node, Layout>,
    world_matrixs: &MultiCaseImpl<Node, WorldMatrix>,
    transforms: &MultiCaseImpl<Node, Transform>,
    style_marks: &mut MultiCaseImpl<Node, StyleMark>,
    transform_will_change_matrixs: &mut MultiCaseImpl<Node, TransformWillChangeMatrix>,
){  
    let mut parent_will_change_matrix = parent_will_change_matrix;
    unsafe{style_marks.get_unchecked_mut(id).dirty1 &= !(StyleType1::TransformWillChange as usize)};
    match willchange.get(id) {
        Some(transform_value) => {
            let layout_value = unsafe { layouts.get_unchecked(id) };
            let p_matrix = if parent == 0 {
                WorldMatrix(Matrix4::from_translation(Vector3::new(layout_value.left, layout_value.top, 0.0)), false)
            } else {
                let parent_layout = unsafe { layouts.get_unchecked(parent) };
                let parent_world_matrix = unsafe { world_matrixs.get_unchecked(parent) };
                let parent_transform = match transforms.get(parent) {
                    Some(r) => r,
                    None => default_transform,
                };
                let parent_transform_origin = parent_transform.origin.to_value(parent_layout.width, parent_layout.height);
                let offset = get_lefttop_offset(&layout_value, &parent_transform_origin, &parent_layout);
                parent_world_matrix * default_transform.matrix(layout_value.width, layout_value.height, &offset)
            };

			// println!("p_matrix: {:?}", p_matrix);
            let transform_will_change_matrix = transform_value.0.matrix(layout_value.width, layout_value.height, &Point2::new(-layout_value.width/2.0, -layout_value.height/2.0));
            let invert = p_matrix.invert().unwrap();
			// println!("transform_will_change_matrix: {:?}", transform_will_change_matrix);
			// println!("invert: {:?}", invert);
            let mut will_change_matrix = p_matrix * transform_will_change_matrix * invert;
			// println!("will_change_matrix: {:?}", will_change_matrix);
            if let Some(parent_will_change_matrix) = parent_will_change_matrix {
                will_change_matrix = parent_will_change_matrix * will_change_matrix;
            }

            transform_will_change_matrixs.insert(id, TransformWillChangeMatrix(will_change_matrix));
            parent_will_change_matrix = Some(unsafe { & (&*(transform_will_change_matrixs as *const MultiCaseImpl<Node, TransformWillChangeMatrix>)).get_unchecked(id).0});
            count -= 1;
        },
        None => ()
    };

    let first = unsafe { idtree.get_unchecked(id).children.head };
    for (child_id, _child) in idtree.iter(first) {
        if count == 0 {
            break;
        }
        recursive_cal_matrix(
            child_id, 
            id, 
            count,
            parent_will_change_matrix, 
            default_transform,
            idtree, 
            willchange, 
            layouts, 
            world_matrixs, 
            transforms, 
            style_marks,
            transform_will_change_matrixs);
    }
}


#[inline]
fn get_lefttop_offset(layout: &Layout, parent_origin: &Point2, _parent_layout: &Layout) -> Point2{
    Point2::new(
        // layout.left - parent_origin.x + parent_layout.border_left + parent_layout.padding_left,
        // layout.top - parent_origin.y + parent_layout.border_top + parent_layout.padding_top
        // 当设置宽高为auto时 可能存在bug
        layout.left - parent_origin.x,
        layout.top - parent_origin.y
    )  
}

impl_system!{
    TransformWillChangeSys,
    true,
    {
        MultiCaseListener<Node, TransformWillChange, CreateEvent>
        MultiCaseListener<Node, TransformWillChange, ModifyEvent>
        MultiCaseListener<Node, TransformWillChange, DeleteEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
    }
}