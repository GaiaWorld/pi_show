use wcs::component::{ComponentGroupTree};
use wcs::world::{ComponentMgr};

use component::math::{Color as CgColor, ColorReadRef as CgColorReadRef, ColorGroup as CgColorGroup, ColorWriteRef as CgColorWriteRef};

#[derive(Debug, Clone, EnumDefault, EnumComponent)]
pub enum Color{
    RGB(CgColor),
    RGBA(CgColor),
}