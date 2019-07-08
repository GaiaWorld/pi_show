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

extern crate fxhash;
use fxhash::FxHasher32;
use std::hash::{BuildHasherDefault};

type FxBuildHasher = BuildHasherDefault<FxHasher32>;

type HashMap32<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;
fn hash_map() {
    let time = std::time::Instant::now();
    let mut map: HashMap32<u32, u32> = HashMap32::default();
    for i in 0..1000 {
        map.insert(i, i);
    }
    println!("insert{:?}",  std::time::Instant::now() - time);
    let time = std::time::Instant::now();
    let mut k = 0;
    for i in 0..1000 {
        k += map.get(&i).unwrap();
    }
    println!("get {:?}, {}",  std::time::Instant::now() - time, k);
}

// fn slab() {
//     let time = std::time::Instant::now();
//     let mut map: slab::Slab<u32, u32> = fnv::FnvHashMap::default();
//     for i in 0..1000 {
//         map.insert(i, i);
//     }
//     println!("insert{:?}",  std::time::Instant::now() - time);
//     let time = std::time::Instant::now();
//     let mut k = 0;
//     for i in 0..1000 {
//         k += map.get(&i).unwrap();
//     }
//     println!("get {:?}, {}",  std::time::Instant::now() - time, k);
// }


fn main() { 
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    hash_map();
    // zz();
    // yy();
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
