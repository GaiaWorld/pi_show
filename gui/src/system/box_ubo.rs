/**
 *  计算opacity
 *  该系统默认为所有已经创建的Entity创建BoxColor组件， 并监听BoxColor的创建修改， 以及监听idtree上的创建事件， 计算已经在idtree上存在的实体的BoxColor
 */
use std::marker::PhantomData;
use std::sync::Arc;

use ecs::{CreateEvent, ModifyEvent, DeleteEvent, MultiCaseListener, EntityListener, SingleCaseListener, SingleCaseImpl, MultiCaseImpl, Share};
use ecs::idtree::{ IdTree};
use map::vecmap::VecMap;
use hal_core::{Context, Uniforms};
use atom::Atom;

use component::user::{BoxColor, Transform};
use component::calc::{Visibility, WorldMatrix, Opacity, ByOverflow, ZDepth};
use component::{Color, CgColor};
use layout::Layout;
use entity::{Node};
use single::{RenderObjs, RenderObjWrite, RenderObj, ViewMatrix, ProjectionMatrix, ClipUbo, ViewUbo, ProjectionUbo};
use render::engine::Engine;
use system::util::{cal_matrix, color_is_opaque};


lazy_static! {
    static ref BOX_SHADER_NAME: Atom = Atom::from("box");
    static ref BOX_FS_SHADER_NAME: Atom = Atom::from("box_fs");
    static ref BOX_VS_SHADER_NAME: Atom = Atom::from("box_vas");

    static ref STROKE_COLOR: Atom = Atom::from("strokeColor");
    static ref COLOR: Atom = Atom::from("color");
    static ref COLOR_ANGLE: Atom = Atom::from("colorAngle");
    static ref DISTANCE: Atom = Atom::from("distance");
    static ref COLOR1: Atom = Atom::from("color1");
    static ref COLOR2: Atom = Atom::from("color2");
    static ref COLOR3: Atom = Atom::from("color3");
    static ref COLOR4: Atom = Atom::from("color4");
    static ref FONT_CLAMP: Atom = Atom::from("fontClamp");  // 0-1的小数，超过这个值即认为有字体，默认传0.75
    static ref SMOOT_HRANFE: Atom = Atom::from("smoothRange");
    static ref TEXTURE: Atom = Atom::from("texture");
}

#[derive(Default)]
pub struct BoxUboSys<C: Context +Share>{
    box_render_map: VecMap<usize>,
    mark: PhantomData<C>,
}

// 插入渲染对象
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, BoxColor, CreateEvent> for BoxUboSys<C>{
    type ReadData = (
        &'a SingleCaseImpl<ViewUbo>,
        &'a SingleCaseImpl<ProjectionUbo>,
        &'a SingleCaseImpl<ClipUbo>,
        &'a SingleCaseImpl<IdTree>,
        &'a MultiCaseImpl<Node, BoxColor>,
        &'a MultiCaseImpl<Node, ZDepth>,
        &'a MultiCaseImpl<Node, Visibility>,
        &'a MultiCaseImpl<Node, Opacity>,
        &'a MultiCaseImpl<Node, WorldMatrix>,
        &'a MultiCaseImpl<Node, Transform>,
        &'a MultiCaseImpl<Node, Layout>,
    );
    type WriteData = (&'a mut SingleCaseImpl<RenderObjs<C>>, &'a mut SingleCaseImpl<Engine<C>>);
    fn listen(&mut self, event: &CreateEvent, read: Self::ReadData, write: Self::WriteData){
        let (view_ubo, projection_ubo, clip_ubo, id_tree, box_color, z_depth, visibility, opacity, world_matrix, transform, layout) = read;
        let (render_objs, engine) = write;
        let mut ubos = vec![Arc::new(Uniforms::new()), view_ubo.0.clone(), projection_ubo.0.clone(), clip_ubo.0.clone(), Arc::new(Uniforms::new())]; // 世界矩阵，视图矩阵， 投影矩阵， 裁剪属性， 材质属性，

        let opacity = unsafe { opacity.get_unchecked(event.id) }.0;
        let box_color = unsafe { box_color.get_unchecked(event.id) };
        let world_matrix = cal_matrix(event.id, &world_matrix, &transform, &layout, (0.0, 0.0));

        let render_obj: RenderObj<C> = RenderObj{
            depth: unsafe { z_depth.get_unchecked(event.id) }.0,
            visibility: unsafe { visibility.get_unchecked(event.id) }.0,
            is_opacity: box_is_opacity(opacity, &box_color.background, &box_color.border),
            ubos: ubos,
            shader_attr: None,
        };

        // // 设置ubo
        // let index = write.insert(RenderObj::default());
        // self.box_render_map.insert(event.id, index);
    }
}

// 删除渲染对象
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, BoxColor, DeleteEvent> for BoxUboSys<C>{
    type ReadData = ();
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &DeleteEvent, read: Self::ReadData, write: Self::WriteData){
        let index = self.box_render_map.remove(event.id).unwrap();
        let notify = write.get_notify();
        write.remove(index, Some(notify));
    }
}

// BoxColor变化, 修改ubo
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, BoxColor, ModifyEvent> for BoxUboSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, BoxColor>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        let box_color = unsafe { read.get_unchecked(event.id) };
        match event.field {
            "background" => {},
            "border" => {},
            _ => (),
        }
        // 设置ubo TODO
    }
}

//世界矩阵变化， 设置ubo
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for BoxUboSys<C>{
    type ReadData = (&'a MultiCaseImpl<Node, WorldMatrix>, &'a MultiCaseImpl<Node, Transform>,);
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

//不透明度变化， 设置ubo
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, Opacity, ModifyEvent> for BoxUboSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Opacity>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

//by_overfolw变化， 设置ubo
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for BoxUboSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, ByOverflow>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        // 设置ubo TODO
    }
}

// 设置visibility
impl<'a, C: Context +Share> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for BoxUboSys<C>{
    type ReadData = &'a MultiCaseImpl<Node, Visibility>;
    type WriteData = &'a mut SingleCaseImpl<RenderObjs<C>>;
    fn listen(&mut self, event: &ModifyEvent, read: Self::ReadData, write: Self::WriteData){
        match self.box_render_map.get(event.id) {
            Some(index) => {
                let notify = write.get_notify();
                unsafe {write.get_unchecked_write(*index, &notify)}.set_visibility(unsafe {read.get_unchecked(event.id).0});
            },
            None => (),
        }
    }
}

fn box_is_opacity(opacity: f32, backgroud_color: &Color, border_color: &CgColor) -> bool {
    if opacity < 1.0 {
        return false;
    }
    
    if border_color.a < 1.0 {
        return false;
    }

    return color_is_opaque(backgroud_color);
}

// impl_system!{
//     BoxUboSys,
//     false,
//     {
//         EntityListener<Node, CreateEvent>
//         MultiCaseListener<Node, BoxColor, ModifyEvent>
//         SingleCaseListener<IdTree, CreateEvent>
//     }
// }


// #[cfg(test)]
// use ecs::{World, BorrowMut, SeqDispatcher, Dispatcher};
// #[cfg(test)]
// use atom::Atom;
// #[cfg(test)]
// use component::user::BoxColorWrite;

// #[test]
// fn test(){
//     let world = new_world();

//     let idtree = world.fetch_single::<IdTree>().unwrap();
//     let idtree = BorrowMut::borrow_mut(&idtree);
//     let notify = idtree.get_notify();
//     let opacitys = world.fetch_multi::<Node, BoxColor>().unwrap();
//     let opacitys = BorrowMut::borrow_mut(&opacitys);
//     let copacitys = world.fetch_multi::<Node, CBoxColor>().unwrap();
//     let copacitys = BorrowMut::borrow_mut(&copacitys);

//     let e0 = world.create_entity::<Node>();
    
//     idtree.create(e0);
//     idtree.insert_child(e0, 0, 0, Some(&notify)); //根
//     opacitys.insert(e0, BoxColor::default());

//     world.run(&Atom::from("test_opacity_sys"));
    
//     let e00 = world.create_entity::<Node>();
//     let e01 = world.create_entity::<Node>();
//     let e02 = world.create_entity::<Node>();
//     idtree.create(e00);
//     idtree.insert_child(e00, e0, 1, Some(&notify));
//     opacitys.insert(e00, BoxColor::default());
//     idtree.create(e01);
//     idtree.insert_child(e01, e0, 2, Some(&notify));
//     opacitys.insert(e01, BoxColor::default());
//     idtree.create(e02);
//     idtree.insert_child(e02, e0, 3, Some(&notify));
//     opacitys.insert(e02, BoxColor::default());

//     let e000 = world.create_entity::<Node>();
//     let e001 = world.create_entity::<Node>();
//     let e002 = world.create_entity::<Node>();
//     idtree.create(e000);
//     idtree.insert_child(e000, e00, 1, Some(&notify));
//     opacitys.insert(e000, BoxColor::default());
//     idtree.create(e001);
//     idtree.insert_child(e001, e00, 2, Some(&notify));
//     opacitys.insert(e001, BoxColor::default());
//     idtree.create(e002);
//     idtree.insert_child(e002, e00, 3, Some(&notify));
//     opacitys.insert(e002, BoxColor::default());

//     let e010 = world.create_entity::<Node>();
//     let e011 = world.create_entity::<Node>();
//     let e012 = world.create_entity::<Node>();
//     idtree.create(e010);
//     idtree.insert_child(e010, e01, 1, Some(&notify));
//     opacitys.insert(e010, BoxColor::default());
//     idtree.create(e011);
//     idtree.insert_child(e011, e01, 2, Some(&notify));
//     opacitys.insert(e011, BoxColor::default());
//     idtree.create(e012);
//     idtree.insert_child(e012, e01, 3, Some(&notify));
//     opacitys.insert(e012, BoxColor::default());
//     world.run(&Atom::from("test_opacity_sys"));

//     unsafe { opacitys.get_unchecked_write(e0)}.set_0(0.5);
//     unsafe { opacitys.get_unchecked_write(e00)}.set_0(0.5);

//     world.run(&Atom::from("test_opacity_sys"));

//     println!("e0:{:?}, e00:{:?}, e01:{:?}, e02:{:?}, e000:{:?}, e001:{:?}, e002:{:?}, e010:{:?}, e011:{:?}, e012:{:?}",
//         unsafe{copacitys.get_unchecked(e0)},
//         unsafe{copacitys.get_unchecked(e00)},
//         unsafe{copacitys.get_unchecked(e01)},
//         unsafe{copacitys.get_unchecked(e02)},
//         unsafe{copacitys.get_unchecked(e000)},
//         unsafe{copacitys.get_unchecked(e001)},
//         unsafe{copacitys.get_unchecked(e002)},
//         unsafe{copacitys.get_unchecked(e010)},
//         unsafe{copacitys.get_unchecked(e011)},
//         unsafe{copacitys.get_unchecked(e012)},
//     );
// }

// #[cfg(test)]
// fn new_world() -> World {
//     let mut world = World::default();

//     world.register_entity::<Node>();
//     world.register_multi::<Node, BoxColor>();
//     world.register_multi::<Node, CBoxColor>();
//     world.register_single::<IdTree>(IdTree::default());
     
//     let system = CellBoxUboSys::new(BoxUboSys::default());
//     world.register_system(Atom::from("system"), system);

//     let mut dispatch = SeqDispatcher::default();
//     dispatch.build("system".to_string(), &world);

//     world.add_dispatcher( Atom::from("test_opacity_sys"), dispatch);
//     world
// }

