pub fn get_or_default<'a, T>(value: Option<&'a T>, default_value: &'a T) -> &'a T {
	match value {
		Some(r) => r,
		None => default_value
	}
}