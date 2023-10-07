pub mod style_macro {
    //! 将设置布局属性的接口导出到js
    use std::mem::transmute;
    use gui::component::user::ClassName;
    use ordered_float::NotNan;
    use pi_flex_layout::prelude::*;
    use hash::XHashMap;
    use map::vecmap::VecMap;
    use pi_style::style::*;
    use pi_style::style_type::*;
    use pi_style::style_parse::{
        parse_comma_separated, parse_text_shadow, parse_as_image, StyleParse,
    };
    use smallvec::SmallVec;
    pub use crate::index::{OffsetDocument, Size, GuiWorld, Atom};
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen::prelude::wasm_bindgen;
    pub enum Edge {
        Left = 0,
        Top = 1,
        Right = 2,
        Bottom = 3,
        Start = 4,
        End = 5,
        Horizontal = 6,
        Vertical = 7,
        All = 8,
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_align_content(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, AlignContentType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_align_content(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetAlignContentType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_align_items(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, AlignItemsType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_align_items(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetAlignItemsType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_justify_content(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, JustifyContentType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_justify_content(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetJustifyContentType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_direction(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexDirectionType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_direction(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexDirectionType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_wrap(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexWrapType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_wrap(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexWrapType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_align_self(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, AlignSelfType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_align_self(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetAlignSelfType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_position_type(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, PositionTypeType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_position_type(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetPositionTypeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_grow(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexGrowType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_grow(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexGrowType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_shrink(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexGrowType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_shrink(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexGrowType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_basis_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexBasisType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_basis_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexBasisType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_basis(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexBasisType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_basis(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexBasisType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_flex_basis_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FlexBasisType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_flex_basis_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFlexBasisType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_width_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, WidthType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_width_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_width(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, WidthType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_width(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_width_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, WidthType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_width_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_height_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, HeightType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_height_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_height(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, HeightType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_height(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_height_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, HeightType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_height_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_min_width_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MinWidthType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_min_width_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMinWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_min_width(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MinWidthType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_min_width(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMinWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_min_width_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MinWidthType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_min_width_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMinWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_min_height_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MinHeightType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_min_height_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMinHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_min_height(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MinHeightType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_min_height(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMinHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_min_height_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MinHeightType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_min_height_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMinHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_max_width_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MaxWidthType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_max_width_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaxWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_max_width(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MaxWidthType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_max_width(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaxWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_max_width_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MaxWidthType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_max_width_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaxWidthType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_max_height_percent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MaxHeightType(Dimension::Percent(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_max_height_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaxHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_max_height(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MaxHeightType(Dimension::Points(v)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_max_height(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaxHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_max_height_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, MaxHeightType(Dimension::Auto));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_max_height_auto(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaxHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_padding_percent(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => {
                gui.gui.set_style(node_id, PaddingTopType(Dimension::Percent(v)))
            }
            Edge::Right => {
                gui.gui.set_style(node_id, PaddingRightType(Dimension::Percent(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, PaddingBottomType(Dimension::Percent(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, PaddingLeftType(Dimension::Percent(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_padding_percent(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetPaddingTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetPaddingRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetPaddingBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetPaddingLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_padding(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, PaddingTopType(Dimension::Points(v))),
            Edge::Right => {
                gui.gui.set_style(node_id, PaddingRightType(Dimension::Points(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, PaddingBottomType(Dimension::Points(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, PaddingLeftType(Dimension::Points(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_padding(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetPaddingTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetPaddingRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetPaddingBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetPaddingLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_padding_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, PaddingTopType(Dimension::Auto)),
            Edge::Right => gui.gui.set_style(node_id, PaddingRightType(Dimension::Auto)),
            Edge::Bottom => {
                gui.gui.set_style(node_id, PaddingBottomType(Dimension::Auto))
            }
            Edge::Left => gui.gui.set_style(node_id, PaddingLeftType(Dimension::Auto)),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_padding_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetPaddingTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetPaddingRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetPaddingBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetPaddingLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_margin_percent(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, MarginTopType(Dimension::Percent(v))),
            Edge::Right => {
                gui.gui.set_style(node_id, MarginRightType(Dimension::Percent(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, MarginBottomType(Dimension::Percent(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, MarginLeftType(Dimension::Percent(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_margin_percent(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetMarginTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetMarginRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetMarginBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetMarginLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_margin(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, MarginTopType(Dimension::Points(v))),
            Edge::Right => {
                gui.gui.set_style(node_id, MarginRightType(Dimension::Points(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, MarginBottomType(Dimension::Points(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, MarginLeftType(Dimension::Points(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_margin(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetMarginTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetMarginRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetMarginBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetMarginLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_margin_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, MarginTopType(Dimension::Auto)),
            Edge::Right => gui.gui.set_style(node_id, MarginRightType(Dimension::Auto)),
            Edge::Bottom => gui.gui.set_style(node_id, MarginBottomType(Dimension::Auto)),
            Edge::Left => gui.gui.set_style(node_id, MarginLeftType(Dimension::Auto)),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_margin_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetMarginTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetMarginRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetMarginBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetMarginLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_percent(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, BorderTopType(Dimension::Percent(v))),
            Edge::Right => {
                gui.gui.set_style(node_id, BorderRightType(Dimension::Percent(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, BorderBottomType(Dimension::Percent(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, BorderLeftType(Dimension::Percent(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_percent(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetBorderTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetBorderRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetBorderBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetBorderLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, BorderTopType(Dimension::Points(v))),
            Edge::Right => {
                gui.gui.set_style(node_id, BorderRightType(Dimension::Points(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, BorderBottomType(Dimension::Points(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, BorderLeftType(Dimension::Points(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetBorderTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetBorderRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetBorderBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetBorderLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, BorderTopType(Dimension::Auto)),
            Edge::Right => gui.gui.set_style(node_id, BorderRightType(Dimension::Auto)),
            Edge::Bottom => gui.gui.set_style(node_id, BorderBottomType(Dimension::Auto)),
            Edge::Left => gui.gui.set_style(node_id, BorderLeftType(Dimension::Auto)),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetBorderTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetBorderRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetBorderBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetBorderLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_position_percent(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => {
                gui.gui.set_style(node_id, PositionTopType(Dimension::Percent(v)))
            }
            Edge::Right => {
                gui.gui.set_style(node_id, PositionRightType(Dimension::Percent(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, PositionBottomType(Dimension::Percent(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, PositionLeftType(Dimension::Percent(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_position_percent(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetPositionTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetPositionRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetPositionBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetPositionLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_position(gui: u32, node_id: f64, edge: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => {
                gui.gui.set_style(node_id, PositionTopType(Dimension::Points(v)))
            }
            Edge::Right => {
                gui.gui.set_style(node_id, PositionRightType(Dimension::Points(v)))
            }
            Edge::Bottom => {
                gui.gui.set_style(node_id, PositionBottomType(Dimension::Points(v)))
            }
            Edge::Left => {
                gui.gui.set_style(node_id, PositionLeftType(Dimension::Points(v)))
            }
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_position(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetPositionTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetPositionRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetPositionBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetPositionLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_position_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, PositionTopType(Dimension::Auto)),
            Edge::Right => gui.gui.set_style(node_id, PositionRightType(Dimension::Auto)),
            Edge::Bottom => {
                gui.gui.set_style(node_id, PositionBottomType(Dimension::Auto))
            }
            Edge::Left => gui.gui.set_style(node_id, PositionLeftType(Dimension::Auto)),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_position_auto(gui: u32, node_id: f64, edge: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        match unsafe { transmute(edge as u8) } {
            Edge::Top => gui.gui.set_style(node_id, ResetPositionTopType),
            Edge::Right => gui.gui.set_style(node_id, ResetPositionRightType),
            Edge::Bottom => gui.gui.set_style(node_id, ResetPositionBottomType),
            Edge::Left => gui.gui.set_style(node_id, ResetPositionLeftType),
            _ => return,
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_background_rgba_color(
        gui: u32,
        node_id: f64,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BackgroundColorType(Color::RGBA(CgColor::new(r, g, b, a))),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_background_rgba_color(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBackgroundColorType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_background_linear_color(
        gui: u32,
        node_id: f64,
        direction: f32,
        color_and_positions: Vec<f32>,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BackgroundColorType(
                    Color::LinearGradient(
                        to_linear_gradient_color(
                            color_and_positions.as_slice(),
                            direction,
                        ),
                    ),
                ),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_background_linear_color(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBackgroundColorType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_color(gui: u32, node_id: f64, r: f32, g: f32, b: f32, a: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, BorderColorType(CgColor::new(r, g, b, a)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_color(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBorderColorType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_radius(gui: u32, node_id: f64, s: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BorderRadiusType({
                    let mut input = cssparser::ParserInput::new(s);
                    let mut parse = cssparser::Parser::new(&mut input);
                    let border_radius = pi_style::style_parse::parse_border_radius(
                        &mut parse,
                    );
                    if let Ok(value) = border_radius {
                        value
                    } else {
                        Default::default()
                    }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_radius(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBorderRadiusType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_box_shadow(
        gui: u32,
        node_id: f64,
        h: f32,
        v: f32,
        blur: f32,
        spread: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BoxShadowType(BoxShadow {
                    h: h,
                    v: v,
                    blur: blur,
                    spread: spread,
                    color: CgColor::new(r, g, b, a),
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_box_shadow(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBoxShadowType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_object_fit(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ObjectFitType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_object_fit(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetObjectFitType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_background_repeat(gui: u32, node_id: f64, x: u8, y: u8) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BackgroundRepeatType(ImageRepeat {
                    x: unsafe { transmute(x as u8) },
                    y: unsafe { transmute(y as u8) },
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_background_repeat(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBackgroundRepeatType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_mask_image_linear(
        gui: u32,
        node_id: f64,
        direction: f32,
        color_and_positions: Vec<f32>,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                MaskImageType(
                    MaskImage::LinearGradient(
                        to_linear_gradient_color(
                            color_and_positions.as_slice(),
                            direction,
                        ),
                    ),
                ),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_mask_image_linear(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaskImageType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_image_clip(gui: u32, node_id: f64, u1: f32, v1: f32, u2: f32, v2: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BackgroundImageClipType(
                    NotNanRect::new(
                        unsafe { NotNan::new_unchecked(v1) },
                        unsafe { NotNan::new_unchecked(u2) },
                        unsafe { NotNan::new_unchecked(v2) },
                        unsafe { NotNan::new_unchecked(u1) },
                    ),
                ),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_image_clip(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBackgroundImageClipType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_mask_image_clip(
        gui: u32,
        node_id: f64,
        u1: f32,
        v1: f32,
        u2: f32,
        v2: f32,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                MaskImageClipType(
                    NotNanRect::new(
                        unsafe { NotNan::new_unchecked(v1) },
                        unsafe { NotNan::new_unchecked(u2) },
                        unsafe { NotNan::new_unchecked(v2) },
                        unsafe { NotNan::new_unchecked(u1) },
                    ),
                ),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_mask_image_clip(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaskImageClipType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_image_clip(
        gui: u32,
        node_id: f64,
        u1: f32,
        v1: f32,
        u2: f32,
        v2: f32,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BorderImageClipType(
                    NotNanRect::new(
                        unsafe { NotNan::new_unchecked(v1) },
                        unsafe { NotNan::new_unchecked(u2) },
                        unsafe { NotNan::new_unchecked(v2) },
                        unsafe { NotNan::new_unchecked(u1) },
                    ),
                ),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_image_clip(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBorderImageClipType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_image_slice(
        gui: u32,
        node_id: f64,
        top: f32,
        right: f32,
        bottom: f32,
        left: f32,
        fill: bool,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BorderImageSliceType(BorderImageSlice {
                    top: unsafe { NotNan::new_unchecked(top) },
                    right: unsafe { NotNan::new_unchecked(right) },
                    bottom: unsafe { NotNan::new_unchecked(bottom) },
                    left: unsafe { NotNan::new_unchecked(left) },
                    fill,
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_image_slice(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBorderImageSliceType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_image_repeat(
        gui: u32,
        node_id: f64,
        vertical: u8,
        horizontal: u8,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                BorderImageRepeatType(ImageRepeat {
                    x: unsafe { transmute(vertical as u8) },
                    y: unsafe { transmute(horizontal as u8) },
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_image_repeat(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBorderImageRepeatType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_overflow(gui: u32, node_id: f64, v: bool) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, OverflowType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_overflow(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetOverflowType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_opacity(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, OpacityType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_opacity(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetOpacityType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_display(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, DisplayType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_display(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetDisplayType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_visibility(gui: u32, node_id: f64, v: bool) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, VisibilityType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_visibility(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetVisibilityType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_enable(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, EnableType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_enable(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetEnableType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_blend_mode(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, BlendModeType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_blend_mode(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBlendModeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_zindex(gui: u32, node_id: f64, v: i32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ZIndexType(v as isize));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_zindex(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetZIndexType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_filter_blur(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, BlurType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_filter_blur(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBlurType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_transform_will_change(gui: u32, node_id: f64, v: bool) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, TransformWillChangeType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_transform_will_change(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTransformWillChangeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_filter_hsi(gui: u32, node_id: f64, h: f32, s: f32, _i: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                HsiType({
                    let (mut h, mut s, mut _i) = (h, s, _i);
                    if h > 180.0 {
                        h = 180.0;
                    } else if h < -180.0 {
                        h = -180.0
                    }
                    if s > 100.0 {
                        s = 100.0;
                    } else if s < -100.0 {
                        s = -100.0
                    }
                    if _i > 100.0 {
                        _i = 100.0;
                    } else if _i < -100.0 {
                        _i = -100.0
                    }
                    Hsi {
                        hue_rotate: h / 360.0,
                        saturate: s / 100.0,
                        bright_ness: _i / 100.0,
                    }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_filter_hsi(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetHsiType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_translate(gui: u32, node_id: f64, s: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TranslateType({
                    let mut input = cssparser::ParserInput::new(s);
                    let mut parse = cssparser::Parser::new(&mut input);
                    let translate = pi_style::style_parse::parse_mult(
                        &mut parse,
                        [LengthUnit::default(), LengthUnit::default()],
                        pi_style::style_parse::parse_len_or_percent,
                    );
                    if let Ok(translate) = translate {
                        translate
                    } else {
                        Default::default()
                    }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_translate(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTranslateType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_scale(gui: u32, node_id: f64, s: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                ScaleType({
                    let mut input = cssparser::ParserInput::new(s);
                    let mut parse = cssparser::Parser::new(&mut input);
                    let scale = pi_style::style_parse::parse_mult(
                        &mut parse,
                        [1.0f32, 1.0f32],
                        pi_style::style_parse::parse_number,
                    );
                    if let Ok(scale) = scale { scale } else { Default::default() }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_scale(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetScaleType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_rotate(gui: u32, node_id: f64, s: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                RotateType({
                    let mut input = cssparser::ParserInput::new(s);
                    let mut parse = cssparser::Parser::new(&mut input);
                    let rotate = pi_style::style_parse::parse_angle(&mut parse);
                    if let Ok(rotate) = rotate { rotate } else { Default::default() }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_rotate(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetRotateType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_transform(gui: u32, node_id: f64, s: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TransformType({
                    let mut input = cssparser::ParserInput::new(s);
                    let mut parse = cssparser::Parser::new(&mut input);
                    let transform = pi_style::style_parse::parse_transform(&mut parse);
                    if let Ok(transform) = transform {
                        transform
                    } else {
                        Default::default()
                    }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_transform(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTransformType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_transform_origin(
        gui: u32,
        node_id: f64,
        x_ty: f64,
        x: f32,
        y_ty: f64,
        y: f32,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TransformOriginType({
                    let x_ty = unsafe { transmute(x_ty as u8) };
                    let y_ty = unsafe { transmute(y_ty as u8) };
                    let x_value = match x_ty {
                        LengthUnitType::Pixel => LengthUnit::Pixel(x),
                        LengthUnitType::Percent => LengthUnit::Percent(x),
                    };
                    let y_value = match y_ty {
                        LengthUnitType::Pixel => LengthUnit::Pixel(y),
                        LengthUnitType::Percent => LengthUnit::Percent(y),
                    };
                    TransformOrigin::XY(x_value, y_value)
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_transform_origin(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTransformOriginType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_letter_spacing(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, LetterSpacingType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_letter_spacing(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetLetterSpacingType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_word_spacing(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, WordSpacingType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_word_spacing(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetWordSpacingType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_rgba_color(gui: u32, node_id: f64, r: f32, g: f32, b: f32, a: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ColorType(Color::RGBA(CgColor::new(r, g, b, a))));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_rgba_color(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetColorType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_linear_gradient_color(
        gui: u32,
        node_id: f64,
        direction: f32,
        color_and_positions: Vec<f32>,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                ColorType(
                    Color::LinearGradient(
                        to_linear_gradient_color(
                            color_and_positions.as_slice(),
                            direction,
                        ),
                    ),
                ),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_linear_gradient_color(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetColorType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_line_height_normal(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, LineHeightType(LineHeight::Normal));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_line_height_normal(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetLineHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_line_height(gui: u32, node_id: f64, value: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, LineHeightType(LineHeight::Length(value)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_line_height(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetLineHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_line_height_percent(gui: u32, node_id: f64, value: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, LineHeightType(LineHeight::Percent(value)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_line_height_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetLineHeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_indent(gui: u32, node_id: f64, v: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, TextIndentType(v));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_indent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTextIndentType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_align(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        {
            let v: TextAlign = unsafe { transmute(v as u8) };
            gui.gui.set_style(node_id, TextAlignType(v));
            gui.gui
                .set_style(
                    node_id,
                    JustifyContentType(
                        match v {
                            TextAlign::Left => JustifyContent::FlexStart,
                            TextAlign::Right => JustifyContent::FlexEnd,
                            TextAlign::Center => JustifyContent::Center,
                            TextAlign::Justify => JustifyContent::SpaceBetween,
                        },
                    ),
                );
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_align(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        {
            gui.gui.set_style(node_id, ResetTextAlignType);
            gui.gui.set_style(node_id, ResetJustifyContentType);
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_vertical_align(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        {
            let v: VerticalAlign = unsafe { transmute(v as u8) };
            gui.gui.set_style(node_id, VerticalAlignType(v));
            gui.gui
                .set_style(
                    node_id,
                    AlignSelfType(
                        match v {
                            VerticalAlign::Top => AlignSelf::FlexStart,
                            VerticalAlign::Bottom => AlignSelf::FlexEnd,
                            VerticalAlign::Middle => AlignSelf::Center,
                        },
                    ),
                );
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_vertical_align(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        {
            gui.gui.set_style(node_id, ResetVerticalAlignType);
            gui.gui.set_style(node_id, ResetAlignSelfType);
        };
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_stroke(
        gui: u32,
        node_id: f64,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TextStrokeType(Stroke {
                    width: NotNan::new(width).expect("stroke width is nan"),
                    color: CgColor::new(r, g, b, a),
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_stroke(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTextStrokeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_white_space(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, WhiteSpaceType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_white_space(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetWhiteSpaceType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_font_style(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FontStyleType(unsafe { transmute(v as u8) }));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_font_style(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFontStyleType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_font_weight(gui: u32, node_id: f64, v: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FontWeightType(v as usize));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_font_weight(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFontWeightType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_font_size_none(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FontSizeType(FontSize::None));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_font_size_none(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFontSizeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_font_size(gui: u32, node_id: f64, value: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FontSizeType(FontSize::Length(value as usize)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_font_size(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFontSizeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_font_size_percent(gui: u32, node_id: f64, value: f32) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FontSizeType(FontSize::Percent(value)));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_font_size_percent(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFontSizeType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_content_utf8(gui: u32, node_id: f64, content: Vec<u8>) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TextContentType({
                    let content = unsafe { String::from_utf8_unchecked(content) };
                    TextContent(content, pi_atom::Atom::from(""))
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_content_utf8(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTextContentType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_clip_path_str(gui: u32, node_id: f64, value: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                ClipPathType({
                    let mut input = cssparser::ParserInput::new(value);
                    let mut parse = cssparser::Parser::new(&mut input);
                    match BaseShape::parse(&mut parse) {
                        Ok(r) => r,
                        Err(e) => {
                            ();
                            return;
                        }
                    }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_clip_path_str(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetClipPathType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_mask_image(gui: u32, node_id: f64, image_hash: &Atom) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(node_id, MaskImageType(MaskImage::Path((**image_hash).clone())));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_mask_image(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetMaskImageType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_background_image(gui: u32, node_id: f64, image_hash: &Atom) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, BackgroundImageType((**image_hash).clone()));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_background_image(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBackgroundImageType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_border_image(gui: u32, node_id: f64, image_hash: &Atom) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, BorderImageType((**image_hash).clone()));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_border_image(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetBorderImageType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_shadow(gui: u32, node_id: f64, s: &str) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TextShadowType({
                    let mut input = cssparser::ParserInput::new(s);
                    let mut parse = cssparser::Parser::new(&mut input);
                    let shadows = parse_text_shadow(&mut parse);
                    if let Ok(value) = shadows { value } else { Default::default() }
                }),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_shadow(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTextShadowType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_font_family(gui: u32, node_id: f64, name: &Atom) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, FontFamilyType((**name).clone()));
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_font_family(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetFontFamilyType);
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn set_text_content(gui: u32, node_id: f64, content: String) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui
            .set_style(
                node_id,
                TextContentType(TextContent(content, pi_atom::Atom::from(""))),
            );
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    #[allow(unused_attributes)]
    pub fn reset_text_content(gui: u32, node_id: f64) {
        let node_id = node_id as usize;
        let gui = unsafe { &mut *(gui as usize as *mut GuiWorld) };
        gui.gui.set_style(node_id, ResetTextContentType);
    }
    pub enum LengthUnitType {
        Pixel,
        Percent,
    }
    pub fn to_linear_gradient_color(
        color_and_positions: &[f32],
        direction: f32,
    ) -> LinearGradientColor {
        let arr = color_and_positions;
        let len = arr.len();
        let count = len / 5;
        let mut list = Vec::with_capacity(count);
        for i in 0..count {
            let start = i * 5;
            let color_pos = ColorAndPosition {
                rgba: CgColor::new(
                    arr[start],
                    arr[start + 1],
                    arr[start + 2],
                    arr[start + 3],
                ),
                position: arr[start + 4],
            };
            list.push(color_pos);
        }
        LinearGradientColor {
            direction: direction,
            list: list,
        }
    }
}

pub use self::style_macro::*;