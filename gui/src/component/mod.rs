pub mod calc;
pub mod user;

#[derive(Clone, Copy, Debug)]
pub enum LengthUnit {
	Pixel(f32),
	Percent(f32),
}

#[derive(Clone, Copy, Debug)]
pub enum Display{
  Flex,
  None,
}