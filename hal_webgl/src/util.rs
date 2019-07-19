use slab::{Slab};

/**
 * 将不可变引用变为可变引用
 */
#[inline(always)]
pub fn convert_to_mut<T>(obj: &T) -> &mut T {
    let mut_obj = obj as *const T as usize as *mut T;
	unsafe { &mut *mut_obj }
}

#[inline(always)]
pub fn create_new_slot<T>(slab: &mut Slab<(T, u32)>, obj: T) -> (u32, u32) {
    let (key, v, is_first) = slab.alloc_with_is_first();
    if is_first {
        v.1 = 0;
    }
    
    unsafe { std::ptr::write(&mut v.0 as *mut T, obj) };
    v.1 += 1;

    (key as u32, v.1 as u32)
}

#[inline(always)]
pub fn get_mut_ref<T>(slab: &mut Slab<(T, u32)>, key: u32, count: u32) -> Option<&mut T> {
    slab.get_mut(key as usize).filter(|v| v.1 == count).map(|v| &mut v.0)
}

#[inline(always)]
pub fn get_ref<T>(slab: &Slab<(T, u32)>, key: u32, count: u32) -> Option<&T> {
    slab.get(key as usize).filter(|v| v.1 == count).map(|v| &v.0)
}