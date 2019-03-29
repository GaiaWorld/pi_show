use std::hash::{Hash, Hasher};
use std::convert::AsRef;

use fnv::FnvHashMap;
use atom::Atom;

pub struct Shader {
    pub vs: Atom,
    pub fs: Atom,
    pub name: Atom,
}

impl Shader {
    pub fn new (name: Atom, vs: Atom, fs: Atom) -> Shader {
        Shader {
            vs,
            fs,
            name,
        }
    }
}

pub struct ShaderStore {
    shaders: FnvHashMap<Atom, ShaderCode>,
}

impl ShaderStore {
    pub fn new() -> ShaderStore {
        ShaderStore {
            shaders: FnvHashMap::default(),
        }
    }

    pub fn store(&mut self, name: &Atom, code: String){
        self.shaders.insert(name.clone(), ShaderCode::new(name.clone(), code));
    }

    pub fn remove(&mut self, name: &Atom) {
        self.shaders.remove(name);
    }

    pub fn get(&self, name: &Atom) -> Option<&ShaderCode> {
        self.shaders.get(name)
    }
}

pub struct ShaderCode {
    name: Atom,
    code: String,
}

impl ShaderCode {
    pub fn new (name: Atom, code: String) -> ShaderCode {
        ShaderCode {
            name,
            code,
        }
    }
}

impl Hash for ShaderCode{
    fn hash<H>(&self, state: &mut H) where H: Hasher{
        self.name.hash(state);
    }
}

impl AsRef<str> for ShaderCode {
    fn as_ref(&self) -> &str {
        &self.code
    }
}