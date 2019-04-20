use std::rc::Rc;

use stdweb::unstable::TryInto;
use stdweb::web::TypedArray;

use wcs::world::World;
use atom::Atom;

use component::color::{Color};
use component::math::{Matrix4, Color as MathColor, Point2};
use world_2d::{create_world, World2dMgr };
use world_2d::component::char_block::{CharBlock, Char};
use render::res::{TextureRes};
use render::engine::Engine;
use text_layout::font::{ SdfFont};

#[no_mangle]
pub fn create_world_2d(engine: u32, width: f32, height: f32) -> u32{
    let engine: Engine = *unsafe { Box::from_raw(engine as usize as *mut Engine)};
    let world = create_world(engine, -1000.0, 1000.0, width, height);
    Box::into_raw(Box::new(world)) as u32
}

#[no_mangle]
pub fn create_sdf_font(texture: u32) -> u32{
    let bind: TypedArray<u8> = js!(return __jsObj;).try_into().unwrap();
    let bind = bind.to_vec();
    let mut sdf_font = SdfFont::new(unsafe { &*(texture as usize as *const Rc<TextureRes>)}.clone());
    match sdf_font.parse(bind.as_slice()) {
        Ok(_) => (),
        Err(s) => panic!("{}", s),
    };
    println!("sdf_font----------------------{:?}", sdf_font);
    Box::into_raw(Box::new(Rc::new(sdf_font))) as u32
}

#[no_mangle]
pub fn test_char_block(world: u32, sdf_font: u32){
    let world = unsafe {&mut *(world as usize as *mut World<World2dMgr, ()>)};
    let char_block = CharBlock {
        world_matrix: Matrix4::default(),
        alpha: 1.0,
        visibility: true,
        is_opaque: true,
        z_depth: 1.0,
        by_overflow: 0,
        stroke_size: 0.0,
        stroke_color: MathColor::default(),
        font_size: 32.0,
        sdf_font: unsafe{ &*(sdf_font as usize as *mut Rc<SdfFont>)}.clone() ,
        color: Color::RGBA(MathColor::default()),
        chars: vec![
            Char{
                value: '测',
                pos: Point2::default(),
            }, Char{
                value: '试',
                pos: Point2(cg::Point2::new(60.0, 0.0)),
            }
        ],
    };

    world.component_mgr.add_char_block(char_block);
    world.run(&Atom::from("All"), ());
}