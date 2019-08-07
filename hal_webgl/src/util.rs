pub fn convert_to_mut<T>(obj: &T) -> &mut T {
    let mut_obj = obj as *const T as usize as *mut T;
	unsafe { &mut *mut_obj }
}