
/**
 * 将不可变引用变为可变引用
 */
pub fn convert_to_mut<T>(obj: &T) -> &mut T {
    let mut_obj = obj as *const T as usize as *mut T;
	let mut_obj = unsafe { &mut *mut_obj };
    mut_obj
}