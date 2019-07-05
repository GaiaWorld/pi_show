#![feature(custom_attribute)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// #[use_macro]
// extern crate stdweb;
// #[use_macro]
// extern crate stdweb_derive;
extern crate rustc_hash;
extern crate fnv;

use rustc_hash::FxHasher;
use fnv::FnvHasher;

// extern crate cgmath;

// pub mod test_matrix;
#[allow(unused_attributes)]
#[no_mangle]
pub fn xx()  {
    let time = std::time::Instant::now();
    let mut a = 0;
    for i in 0..300 {
        let mut hasher = DefaultHasher::new();
        i.hash(&mut hasher);
        i.hash(&mut hasher);
        i.hash(&mut hasher);
        a = hasher.finish();
    }
    println!("{:?}, {}",  std::time::Instant::now() - time, a);
    
    // js! {
    //     console.log(format!("{:?}, {}",  std::time::Instant::now() - time, a));
    // }
}

fn yy() {
    let time = std::time::Instant::now();
    let mut a = 0;
    for i in 0..300 {
        let mut hasher = FxHasher::default();
        i.hash(&mut hasher);
        i.hash(&mut hasher);
        i.hash(&mut hasher);
        a = hasher.finish();
    }
    println!("{:?}, {}",  std::time::Instant::now() - time, a);
}

fn zz() {
    let time = std::time::Instant::now();
    let mut a = 0;
    for i in 0..300 {
        let mut hasher = FnvHasher::default();
        i.hash(&mut hasher);
        i.hash(&mut hasher);
        i.hash(&mut hasher);
        a = hasher.finish();
    }
    println!("{:?}, {}",  std::time::Instant::now() - time, a);
}


fn main() { 
    // zz();
    yy();
    // xx();
    // let time = std::time::Instant::now();
    // let mut a = 0;
    // for i in 0..300 {
    //     let mut hasher = DefaultHasher::new();
    //     i.hash(&mut hasher);
    //     i.hash(&mut hasher);
    //     i.hash(&mut hasher);
    //     a = hasher.finish();
    // }
    // println!("{:?}, {}",  std::time::Instant::now() - time, a);
}

// fn main() {
//     test_matrix::test_cal_martix4();
// }
