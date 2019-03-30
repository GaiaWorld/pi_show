use std::hash::{Hash};
use std::cmp::Eq;

use fnv::FnvHashMap;

pub struct Shader<T: Hash + Eq> {
    pub vs: T,
    pub fs: T,
    pub name: T,
}

impl<T: Hash + Eq> Shader<T> {
    pub fn new (name: T, vs: T, fs: T) -> Shader<T>{
        Shader {
            vs,
            fs,
            name,
        }
    }
}

pub struct ShaderStore<T: Hash + Eq> {
    shaders: FnvHashMap<T, String>,
}

impl<T: Hash + Eq> ShaderStore<T> {
    pub fn new() -> ShaderStore<T> {
        ShaderStore {
            shaders: FnvHashMap::default(),
        }
    }

    pub fn store(&mut self, name: T, code: String){
        self.shaders.insert(name, code);
    }

    pub fn remove(&mut self, name: &T) {
        self.shaders.remove(name);
    }

    pub fn get(&mut self, name: &T) -> Option<&String> {
        self.shaders.get(name)
    }
}