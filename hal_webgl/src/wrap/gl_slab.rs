use share::Share;
use slab::Slab;

/**
 * 将不可变引用变为可变引用
 */
pub fn convert_to_mut<T>(obj: &T) -> &mut T {
    let mut_obj = obj as *const T as usize as *mut T;
	let mut_obj = unsafe { &mut *mut_obj };
    mut_obj
}
	
/**
 * Slab槽
 */
pub struct GLSlot<T> {
    pub slab: Share<Slab<T>>,
    pub index: usize,    // 槽的索引
}

impl<T> Clone for GLSlot<T> {
    fn clone(&self) -> Self {
        Self {
            slab: self.slab.clone(),
            index: self.index,
        }
    }
}

impl<T> GLSlot<T> {
    pub fn new(slab: &Share<Slab<T>>, obj: T) -> GLSlot<T> {
        let s = convert_to_mut(slab.as_ref());
        let index = s.insert(obj);
        
        GLSlot::<T> {
            slab: slab.clone(),
            index: index,
        }
    }
}