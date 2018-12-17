// use std::rc::Rc;
// use std::cell::RefCell;

// use wcs::world::{System, World, ComponentMgr, WeakWorld};
// use wcs::component::ComponentGroup;

// use component::Node;

// pub struct ZSystem(Rc<RefCell<ZSystem>>);

// impl ZSystem {
//     pub fn init(&self, c: &mut ComponentGroup<Node>){
        
//     }
// }

// pub struct ZSystemImpl {

// }



// impl<E> System<E> for ZSystemImpl{
//     fn run(&mut self, e: &E){
        
//     }
//     fn init<C: ComponentMgr>(self, world: World<C, E>) -> Rc<RefCell<ZSystemImpl>>{
//         let sys = Rc::new(RefCell::new(self));
//         //world.
//         sys
//     }
// }

