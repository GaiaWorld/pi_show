/// 将设置节点属性的接口导出到js
use std::{f32::INFINITY as FMAX, usize::MAX as UMAX};

use wasm_bindgen::prelude::*;
use web_sys::WebGlTexture;

use atom::Atom;
use cg2d::{include_quad2, InnOuter};
use gui::single::{IdTree};
use idtree::InsertType;
use ecs::{Lend, LendMut, MultiCaseImpl, SingleCaseImpl};
use ecs::monitor::NotifyImpl;
use spatialtree::quad_helper::intersects;

// use share::Share;
use gui::component::calc::*;
use gui::component::user::*;
use gui::entity::Node;
use gui::render::res::Opacity as ROpacity;
use gui::single::*;
use hal_core::*;
// use gui::
// use gui::system::set_layout_style;
use gui::render::res::TextureRes;
use gui::Z_MAX;

use crate::world::GuiWorld;

fn create(world: &GuiWorld) -> usize {
    let gui = &world.gui;
    let idtree = gui.idtree.lend_mut();
	let node = gui.node.lend_mut().create();
    let border_radius = gui.border_radius.lend_mut();
    border_radius.insert(
        node,
        BorderRadius {
            x: LengthUnit::Pixel(0.0),
            y: LengthUnit::Pixel(0.0),
        },
    );
	idtree.create(node);
    // set_layout_style(&world.default_attr, unsafe {gui.yoga.lend_mut().get_unchecked(node)}, &mut StyleMark::default());
    node
}

/// 创建容器节点， 容器节点可设置背景颜色
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_node(world: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let node = create(world);


    node as u32
}

/// 创建虚拟节点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_vnode(world: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let node = create(world);

	let gui = &world.gui;
	let node_states = gui.node_state.lend_mut();
	node_states[node].0.set_vnode(true);

    node as u32
}

/// 节点是否存在
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn node_is_exist(world: u32, node: u32) -> bool {
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let node_states = &world.gui.node_state.lend_mut();
	return match node_states.get(node as usize) {
		Some(r) => if r.0.is_rnode(){ 
			true
		} else {
			false
		},
		None => false,
	};
}

/// 创建文本节点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_text_node(world: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let node = create(world);
    world
        .gui
        .text_content
        .lend_mut()
        .insert(node, TextContent("".to_string(), Atom::from("")));
    node as u32
}

/// 创建图片节点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_image_node(world: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let node = create(world);
    node as u32
}

/// 创建图片节点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn create_canvas_node(world: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let node = create(world);
    node as u32
}

/**
 * canvas宽高改变时调用
 * @return __jsObj 纹理
*/
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_canvas_size(
    world: u32,
    node: u32,
    w: u32,
    h: u32,
    width: u32,
    height: u32,
    avail_width: u32,
	avail_height: u32,
) -> Option<WebGlTexture> {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let engine = world.gui.engine.lend_mut();
    let images = world.gui.image.lend_mut();
    let image_clips = world.gui.image_clip.lend_mut();
    // let image_notify = images.get_notify_ref();

    if width != 0 && height != 0 {
        image_clips.insert(
            node as usize,
            ImageClip(Aabb2::new(
                Point2::new(0.0, 0.0),
                Point2::new(
                    avail_width as f32 / width as f32,
                    avail_height as f32 / height as f32,
                ),
            )),
        );
    } else {
        image_clips.insert(
            node as usize,
            ImageClip(Aabb2::new(Point2::new(0.0, 0.0), Point2::new(0.0, 0.0))),
        );
    }
    // 如果已经存在纹理， 则更新纹理大小
    match images.get_mut(node as usize) {
        Some(r) => {
            if let Some(texture) = &r.src {
                // 宽高相等，不需要重设纹理尺寸
                if texture.width != width as usize || texture.height != height as usize {
                    engine.gl.texture_resize(&texture.bind, 0, width, height);
                    texture.update_size(width as usize, height as usize);
                }
                r.width = Some(w as f32);
                r.height = Some(h as f32);
                images.get_notify_ref().modify_event(node as usize, "", 0);
                // // 设置渲染脏r
                // let render_objs = world.gui.render_objs.lend();
                // render_objs.get_notify().modify_event(1, "", 0);
                return None;
            }
        }
        None => (),
    };
    //否则创建新的纹理
    let texture = engine
        .gl
        .texture_create_2d_webgl(
            0,
            width,
            height,
            PixelFormat::RGBA,
            DataFormat::UnsignedByte,
            false,
            None,
        )
        .unwrap();
    // engine.gl.texture_pixel_storei(&texture, PixelStore::UnpackFlipYWebgl(true));

    let js_texture = engine.gl.texture_get_object_webgl(&texture).unwrap().clone();
    // let fbo = engine.gl.rt_create(Some(&texture), width, height, PixelFormat::RGBA, DataFormat::UnsignedByte, false);
    let name = Atom::from(format!("canvas{}", node)).get_hash();
    let texture = engine.create_texture_res(
        name.clone(),
        TextureRes::new(
            width as usize,
            height as usize,
            PixelFormat::RGBA,
            DataFormat::UnsignedByte,
            ROpacity::Transparent,
            None,
            texture,
			None,
        ),
        0,
    );
    images.insert(
        node as usize,
        Image {
            src: Some(texture),
            url: name,
            width: Some(w as f32),
            height: Some(h as f32),
        },
    );
    // let width: u32 = js!{return __jsObj.width}.try_into().unwrap();
    // let height: u32 = js!{return __jsObj.height}.try_into().unwrap();
    // let texture = engine.gl.texture_create_2d(width, height, 0, PixelFormat::RGBA, DataFormat::UnsignedByte, false, None).unwrap();

    // let js_texture = engine.gl.texture_get_object_webgl(&texture).unwrap();
    // js! {
    //     window.__jsObj = @{js_texture};
    // }
    // // let fbo = engine.gl.rt_create(Some(&texture), width, height, PixelFormat::RGBA, DataFormat::UnsignedByte, false);
    // let name = Atom::from(format!("canvas{}", node));
    // let texture = engine.create_texture_res(
    //     name.clone(),
    //     TextureRes::new(
    //         width as usize,
    //         height as usize,
    //         PixelFormat::RGBA,
    //         DataFormat::UnsignedByte,
    //         ROpacity::Transparent,
    //         Compress::None,
    //         texture,
    //     ),
    //     0,
    // );
    // images.insert(
    //     node as usize,
    //     Image {
    //         src: Some(texture),
    //         url: name,
    //         width: Some(w as f32),
    //         height: Some(h as f32),
    //     },
	// );
	Some(js_texture)
}

/**
 * canvas内容发生改变时，应该调用此方法更新gui渲染
*/
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn update_canvas(world: u32, _node: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    // let engine = world.gui.engine.lend_mut();
    // let images = world.gui.image.lend_mut();
    // let texture = match images.get(node as usize) {
    //     Some(r) => match &r.src {
    //         Some(r) => r,
    //         None => return,
    //     },
    //     None => return,
    // };
    // let width: u32 = js! {return __jsObj.width}.try_into().unwrap();
    // let height: u32 = js! {return __jsObj.height}.try_into().unwrap();
    // match TryInto::<Object>::try_into(js! {return {wrap: __jsObj};}) {
    //     Ok(canvas) => engine.gl.texture_update_webgl(
    //         &texture.bind,
    //         0,
    //         0,
    //         0,
    //         &TryInto::<Object>::try_into(js! {return {wrap: __jsObj};}).unwrap(),
    //     ),
    //     Err(s) => panic!("set_src error, {:?}", s),
    // };

	// 设置视口脏区域为全屏（优化TODO: 设置为改canvas所在的区域为脏区域）
	let dirty_view_rect = world.gui.dirty_view_rect.lend_mut();
	dirty_view_rect.4 = true;
    // 设置渲染脏
    let render_objs = world.gui.render_objs.lend();
    render_objs.get_notify_ref().modify_event(1, "", 0);
}
/// 在尾部插入子节点，如果该节点已经存在父节点， 会移除原有父节点对本节点的引用
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn append_child(world: u32, child: u32, parent: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
	let notify = unsafe { &*(idtree.get_notify_ref() as * const NotifyImpl)};
    // 如果child在树上， 则会从树上移除节点， 但不会发出事件
	// idtree.remove(child as usize, None);
	idtree.insert_child_with_notify(child as usize, parent as usize, UMAX, notify);
}

/// 在某个节点之前插入节点，如果该节点已经存在父节点， 会移除原有父节点对本节点的引用
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn insert_before(world: u32, child: u32, brother: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
	let notify = unsafe { &*(idtree.get_notify_ref() as * const NotifyImpl)};
    // 如果child在树上， 则会从树上移除节点， 但不会发出事件
    idtree.insert_brother_with_notify(
        child as usize,
        brother as usize,
        InsertType::Front,
        notify,
	);
}

/// 移除节点， 但不会销毁， 如果节点树中存在图片资源， 将会释放其对纹理的引用， 如果节点树中overflow=hidden， 将会删除对应的裁剪平面，
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn remove_node(world: u32, node_id: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let idtree = world.idtree.lend_mut();
	let notify = idtree.get_notify();
	idtree.remove_with_notify(node_id as usize, &notify);
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn first_child(world: u32, parent: u32) -> usize{
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let idtree = world.idtree.lend_mut();
	idtree[parent as usize].children().head
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn last_child(world: u32, parent: u32) -> usize{
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let idtree = world.idtree.lend_mut();
	idtree[parent as usize].children().tail
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn next_sibling(world: u32, node: u32) -> usize{
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let idtree = world.idtree.lend_mut();
	idtree[node as usize].next()
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn previous_sibling(world: u32, node: u32) -> usize{
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
	let idtree = world.idtree.lend_mut();
	idtree[node as usize].prev()
}

// /// 在某个节点之前插入节点，如果该节点已经存在父节点， 会移除原有父节点对本节点的引用
// #[allow(unused_attributes)]
// #[no_mangle]
// #[js_export]
// // pub fn insert_after(world: u32, child: u32, brother: u32) {
//     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//     let world = &mut world.gui;
//     let idtree = world.idtree.lend_mut();
//     let notify = idtree.get_notify_ref();
//     // 如果child在树上， 则会从树上移除节点， 但不会发出事件
//     idtree.remove(child as usize, None);
//     idtree.insert_brother(
//         child as usize,
//         brother as usize,
//         InsertType::Back,
//         Some(&notify),
//     );
// }

/// 返回节点的层级（如果节点未添加在根上，返回0, 不存在节点，返回0）
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn get_layer(world: u32, node: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    match idtree.get(node as usize) {
        Some(n) => n.layer() as u32,
        None => 0,
    }
}

/// 销毁节点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn destroy_node(world: u32, node_id: u32) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let idtree = world.idtree.lend_mut();
    let notify = idtree.get_notify();
    let nodes = world.node.lend_mut();
	
	idtree.remove_with_notify(node_id as usize, &notify);
	// if let None = idtree.remove_with_notify(node_id as usize, &notify) {
	// 	// return;
	// }
	let head = idtree[node_id as usize].children().head;
	nodes.delete(node_id as usize);
	for (id, _n) in idtree.recursive_iter(head) {
		nodes.delete(id);
	}
	//销毁节点（remove已经发出了移除事件，这里不需要再次通知）
	idtree.destroy(node_id as usize);
}

/// __jsObj: image_name(String)
/// 设置图片的src
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn set_src(world: u32, node: u32, url: usize) {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };

    let images = world.gui.image.lend_mut();
    images.insert(
        node as usize,
        Image {
            src: None,
            url: url,
            width: None,
            height: None,
        },
    );
}

/// 返回值原类型为f32,这里之所以返回u32，是因为在iphonex以上的机型的浏览器上多次连续调用返回值为浮点数时，浏览器会自动刷新或白屏，原因未知
/// 节点到gui的上边界的距离
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn offset_top(world: u32, node: u32) -> f32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;
	world.layout.lend()[node as usize].rect.top
}

/// 返回值原类型为f32,这里之所以返回u32，是因为在iphonex以上的机型的浏览器上多次连续调用返回值为浮点数时，浏览器会自动刷新或白屏，原因未知
/// 节点到gui的左边界的距离
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn offset_left(world: u32, node: u32) -> f32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;
	world.layout.lend()[node as usize].rect.start
}

/// 返回值原类型为f32,这里之所以返回u32，是因为在iphonex以上的机型的浏览器上多次连续调用返回值为浮点数时，浏览器会自动刷新或白屏，原因未知
/// 节点的布局宽度
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn offset_width(world: u32, node: u32) -> f32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;
	let r = &world.layout.lend()[node as usize];
	(r.rect.end - r.rect.start)
}

/// 返回值原类型为f32,这里之所以返回u32，是因为在iphonex以上的机型的浏览器上多次连续调用返回值为浮点数时，浏览器会自动刷新或白屏，原因未知
/// 节点布局高度
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn offset_height(world: u32, node: u32) -> f32 {
	// 10.0
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
	let world = &mut world.gui;
	let r = &world.layout.lend()[node as usize];
    (r.rect.bottom - r.rect.top)
}

// /// left top width height
// #[no_mangle]
// #[js_export]
// // pub fn offset_document(world: u32, node_id: u32) {
//     let node_id = node_id as usize;
//     let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
//     let world = &mut world.gui;
//     let layouts = world.layout.lend();
//     let world_matrixs = world.world_matrix.lend();
//     let transforms = world.transform.lend();

//     let transform;
//     let transform1;
//     match transforms.get(node_id) {
//         Some(r) => transform = r,
//         None => {
//             transform1 = Transform::default();
//             transform = &transform1;
//         }
//     };

//     let layout = unsafe { layouts.get_unchecked(node_id) };
//     let origin = transform.origin.to_value(layout.width, layout.height);

//     let world_matrix = unsafe { world_matrixs.get_unchecked(node_id) };
//     let point = Vector4::new(
//         -origin.x + layout.border_left + layout.padding_left,
//         -origin.y + layout.border_top + layout.padding_top,
//         1.0,
//         1.0,
//     );
//     let left_top = world_matrix.0 * point;

//     js! {
//         __jsObj.left = @{left_top.x};
//         __jsObj.top = @{left_top.y};
//         __jsObj.width = @{layout.width - layout.border_left - layout.padding_left};
//         __jsObj.height = @{layout.height - layout.border_top- layout.padding_top};
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct Rect {
	pub left: f32,
	pub top: f32,
	pub width: f32,
	pub height: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Size {
	pub width: f32,
	pub height: f32,
}

/// 等同于html的getBoundingClientRect
/// left top width height
#[wasm_bindgen]
pub fn offset_document(world: u32, node_id: u32) -> JsValue {
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    // let layouts = world.layout.lend();
    // let world_matrixs = world.world_matrix.lend();
    // let transforms = world.transform.lend();
    let octs = world.oct.lend();
    // debug_println!("oct====================={:?}, {:?}", node_id, oct);
    match octs.get(node_id) {
        Some((oct, _)) => JsValue::from_serde(&Rect{left: oct.mins.x, top: oct.mins.y, width: oct.maxs.x - oct.mins.x, height: oct.maxs.y - oct.mins.y}).unwrap() ,
        None => JsValue::from_serde(&Rect{left: 0.0, top: 0.0, width: 0.0, height: 0.0}).unwrap(),
    }

    // let transform;
    // let transform1;
    // match transforms.get(node_id) {
    //     Some(r) => transform = r,
    //     None => {
    //         transform1 = Transform::default();
    //         transform = &transform1;
    //     }
    // };

    // let layout = unsafe { layouts.get_unchecked(node_id) };
    // let origin = transform.origin.to_value(layout.width, layout.height);

    // let world_matrix = unsafe { world_matrixs.get_unchecked(node_id) };
    // let point = Vector4::new(
    //     -origin.x + layout.border_left + layout.padding_left,
    //     -origin.y + layout.border_top + layout.padding_top,
    //     1.0,
    //     1.0,
    // );
    // let left_top = world_matrix.0 * point;

    // js! {
    //     __jsObj.left = @{layout.left};
    //     __jsObj.top = @{layout.top};
    //     __jsObj.width = @{layout.width};
    //     __jsObj.height = @{layout.height};
    // }
}

/// content宽高的累加值
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn content_box(world: u32, node: u32) -> JsValue {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;
    let layout = world.layout.lend();
    let idtree = world.idtree.borrow();
    let (mut left, mut right, mut top, mut bottom) = (FMAX, 0.0, FMAX, 0.0);
    for (id, _) in idtree.iter(idtree[node as usize].children().head) {
        let l = &layout[id];
        let r = l.rect.end;
        let b = l.rect.bottom;
        if l.rect.start < left {
            left = l.rect.start;
        }
        if r > right {
            right = r;
        }
        if b > bottom {
            bottom = b;
        }
        if l.rect.top < top {
            top = l.rect.top;
        }
	}
	JsValue::from_serde(&Size{
		width: right - left,
		height: bottom - top,
	}).unwrap()
}

/// 用点命中一个节点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn query(world: u32, x: f32, y: f32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let octree = world.oct.lend();
    let enables = world.enable.lend();
    let overflow_clip = world.overflow_clip.lend();
    let by_overflows = world.by_overflow.lend();
    let z_depths = world.z_depth.lend();
    let idtree = world.idtree.lend();

    let aabb = Aabb2::new(Point2::new(x, y), Point2::new(x, y));
    let mut args = AbQueryArgs::new(
        enables,
        by_overflows,
        z_depths,
        overflow_clip,
        idtree,
        aabb.clone(),
        0,
    );
    octree.query(&aabb, intersects, &mut args, ab_query_func);
    args.result as u32
}

/// 判断点是否能命中点
#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn is_intersect(world: u32, x: f32, y: f32, node: u32) -> bool {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let octree = world.oct.lend();
    match octree.get(node as usize) {
        Some((oct, _bind)) => !(x < oct.mins.x || x > oct.maxs.x || y < oct.mins.y || y > oct.maxs.y),
        None => false,
    }
    // let octree = world.oct.lend();
    // let enables = world.enable.lend();
    // let overflow_clip = world.overflow_clip.lend();
    // let by_overflows = world.by_overflow.lend();
    // let z_depths = world.z_depth.lend();
    // let idtree = world.idtree.lend();

    // let aabb = Aabb3::new(Point2::new(x, y, -Z_MAX), Point2::new(x, y, Z_MAX));
    // let mut args = AbQueryArgs::new(
    //     enables,
    //     by_overflows,
    //     z_depths,
    //     overflow_clip,
    //     idtree,
    //     aabb.clone(),
    //     0,
    // );
    // octree.query(&aabb, intersects, &mut args, ab_query_func);
    // args.result as u32
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn iter_query(world: u32, x: f32, y: f32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    let world = &mut world.gui;

    let entitys = world.node.lend();
    let octree = world.oct.lend();
    let enables = world.enable.lend();
    let overflow_clip = world.overflow_clip.lend();
    let by_overflows = world.by_overflow.lend();
    let z_depths = world.z_depth.lend();
    let idtree = world.idtree.lend();

    let aabb = Aabb2::new(Point2::new(x, y), Point2::new(x, y));
    let mut args = AbQueryArgs::new(
        enables,
        by_overflows,
        z_depths,
        overflow_clip,
        idtree,
        aabb.clone(),
        0,
    );

    for e in entitys.iter() {
        let oct = match octree.get(e) {
            Some(r) => r,
            None => {
                debug_println!("query fail, id: {}", e);
                return 0;
            }
        };
        ab_query_func(&mut args, e, oct.0, &e);
    }
    args.result as u32
}

#[allow(unused_attributes)]
#[wasm_bindgen]
pub fn add_class(world_id: u32, node_id: u32, key: u32, index: u32) {
    let node_id = node_id as usize;
    let world = unsafe { &mut *(world_id as usize as *mut GuiWorld) };
    if index == 0 {
        unsafe {
            world
                .gui
                .class_name
                .lend_mut()
                .get_unchecked_mut(node_id as usize)
        }
        .one = key as usize;
    } else if index == 1 {
        unsafe {
            world
                .gui
                .class_name
                .lend_mut()
                .get_unchecked_mut(node_id as usize)
        }
        .two = key as usize;
    }
    if index > 1 {
        unsafe {
            world
                .gui
                .class_name
                .lend_mut()
                .get_unchecked_mut(node_id as usize)
        }
        .other
        .push(key as usize);
    }
}

#[wasm_bindgen]
pub fn add_class_end(world: u32, node_id: u32, old: u32) {
	let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    world
        .gui
        .class_name
        .lend_mut()
        .get_notify_ref()
        .modify_event(node_id as usize, "", old as usize);
}

#[wasm_bindgen]
pub fn add_class_start(world: u32, node_id: u32) -> u32 {
    let world = unsafe { &mut *(world as usize as *mut GuiWorld) };
    Box::into_raw(Box::new(
        world
            .gui
            .class_name
            .lend_mut()
            .insert_no_notify(node_id as usize, ClassName::default()),
    )) as u32
}

#[wasm_bindgen]
pub fn set_class(world_id: u32, node_id: u32, class_names: &[u32]) {
	let old = add_class_start(world_id, node_id);
	for i in 0..class_names.len() {
		add_class(world_id, node_id, class_names[i], i as u32);
	}
	add_class_end(world_id, node_id, old)
}

/// aabb的查询函数的参数
struct AbQueryArgs<'a> {
    enables: &'a MultiCaseImpl<Node, Enable>,
    by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
    z_depths: &'a MultiCaseImpl<Node, ZDepth>,
    overflow_clip: &'a SingleCaseImpl<OverflowClip>,
    id_tree: &'a SingleCaseImpl<IdTree>,
    aabb: Aabb2,
    ev_type: u32,
    max_z: f32,
    result: usize,
}
impl<'a> AbQueryArgs<'a> {
    pub fn new(
        enables: &'a MultiCaseImpl<Node, Enable>,
        by_overflows: &'a MultiCaseImpl<Node, ByOverflow>,
        z_depths: &'a MultiCaseImpl<Node, ZDepth>,
        overflow_clip: &'a SingleCaseImpl<OverflowClip>,
        id_tree: &'a SingleCaseImpl<IdTree>,
        aabb: Aabb2,
        ev_type: u32,
    ) -> AbQueryArgs<'a> {
        AbQueryArgs {
            enables,
            by_overflows,
            z_depths,
            overflow_clip,
            id_tree,
            aabb: aabb,
            ev_type: ev_type,
            max_z: -Z_MAX,
            result: 0,
        }
    }
}
/// aabb的ab查询函数, aabb的oct查询函数应该使用intersects
fn ab_query_func(arg: &mut AbQueryArgs, _id: usize, aabb: &Aabb2, bind: &usize) {
    match arg.id_tree.get(*bind) {
        Some(node) => {
            if node.layer() == 0 {
                return;
            }
        }
        None => return,
    };
    // log::info!("ab_query_func----------------------------bind: {}, aabb: {:?}, arg: {:?}", *bind, aabb, arg.aabb);
    if intersects(&arg.aabb, aabb) {
		let enable = arg.enables[*bind].0;
		let z_depth = arg.z_depths[*bind].0;
        // log::info!("enable----------id: {}, enable: {}, z_depth: {}, max_z: {}", bind, enable, z_depth,  arg.max_z);
        //如果enable true 表示不接收事件
        match enable {
            true => (),
            false => return,
        };

        
        // log::info!("z_depth----------id: {}, z_depth: {}, arg.max_z:{}", bind, z_depth, arg.max_z);
        // 取最大z的node
        if z_depth > arg.max_z {
            let by_overflow = arg.by_overflows[*bind].0;
			//   log::info!("by_overflow1---------------------------bind: {},  by: {}, clip: {:?}, x: {}, y: {}", bind, by_overflow, &arg.overflow_clip.clip, arg.aabb.mins.x, arg.aabb.mins.y);
            // 检查是否有裁剪，及是否在裁剪范围内
            if by_overflow == 0
                || in_overflow(
                    &arg.overflow_clip,
                    by_overflow,
                    arg.aabb.mins.x,
                    arg.aabb.mins.y,
                )
            {
                // log::info!("in_overflow------------------by: {}, bind: {}, ", by_overflow, bind);
                arg.result = *bind;
                arg.max_z = z_depth;
            }
        }
        // // 判断类型是否有相交
        // if (node.event_type as u32) & arg.ev_type != 0 {
        //     // 取最大z的node
        //     if node.z_depth > arg.max_z {
        //       // 检查是否有裁剪，及是否在裁剪范围内
        //       if node.by_overflow == 0 || in_overflow(&arg.mgr, node.by_overflow, aabb.min.x, aabb.min.y) {
        //         arg.result = *bind;
        //         arg.max_z = node.z_depth;
        //        }
        //     }
        // }
    }
}

/// 检查坐标是否在裁剪范围内， 直接在裁剪面上检查
fn in_overflow(
    overflow_clip: &SingleCaseImpl<OverflowClip>,
    by_overflow: usize,
    x: f32,
    y: f32,
) -> bool {
    let xy = Point2::new(x, y);
    for (i, c) in overflow_clip.clip.iter() {
        // log::info!("i + 1---------------------------{}",i + 1);
        if (by_overflow & (1 << (i - 1))) != 0 {
            let p = &c.view;
            match include_quad2(&xy, &p[0], &p[1], &p[2], &p[3]) {
                InnOuter::Inner => (),
                _ => {
                    // log::info!("overflow----------clip: {:?}, {:?}, {:?}, {:?},x: {}, y: {}", p[0],  p[1],  p[2], p[3], x, y);
                    return false;
                }
            }
        }
    }
    return true;
}

// /// 检查坐标是否在裁剪范围内， 直接在裁剪面上检查
// fn in_overflow(overflow_clip: &SingleCaseImpl<OverflowClip>, by_overflow: usize, x: f32, y: f32) -> bool{
//   let xy = Point2::new(x, y);
//   for i in 0..overflow_clip.id_vec.len() {
//     // debug_println!("i + 1---------------------------{}",i + 1);
//     // debug_println!("overflow_clip.id_vec[i]---------------------------{}",overflow_clip.id_vec[i]);
//     if (by_overflow & (1<<i)) != 0 && overflow_clip.id_vec[i] > 0 {
//       let p = &overflow_clip.clip[i];
//       match include_quad2(&xy, &p[0], &p[1], &p[2], &p[3]) {
//         InnOuter::Inner => (),
//         _ => {
//             // println!("overflow----------clip: {:?},x: {}, y: {}", p[0], x, y);
//             return false
//         }
//       }
//     }
//   }
//   return true
// }
