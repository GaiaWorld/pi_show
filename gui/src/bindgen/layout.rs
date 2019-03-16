use bindgen::data::*;
// use layout;

#[wasm_bindgen]
pub struct Layout{
    id: usize
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl Layout {
    pub fn container(&self) -> {
        let mut world = self.world.borrow_mut();
        let mut style_ref = Layout::get_ref(&mut world.component_mgr);
        style_ref.set_display(Some(transmute(value)));
    }

    pub fn item(&self){
        let mut world = self.world.borrow_mut();
        let mut style_ref = Layout::get_ref(&mut world.component_mgr);
        style_ref.set_display(None);
    }

    pub fn margin(&self) ->  {
        let mut world = self.world.borrow_mut();
        let mut style = Layout::get_ref(&mut world.component_mgr);
        style.set
    }

    pub fn border(&self) ->  {
        let mut world = self.world.borrow_mut();
        let mut style = Layout::get_ref(&mut world.component_mgr);
        style.set
    }

    pub fn wh(&self) ->  {
        let mut world = self.world.borrow_mut();
        let mut style = Layout::get_ref(&mut world.component_mgr);
        style.set
    }

    pub fn position(&self) ->  {
        let mut world = self.world.borrow_mut();
        let mut style = Layout::get_ref(&mut world.component_mgr);
        style.set
    }

    fn get_ref(mgr: &mut GuiComponentMgr) -> StyleWriteRef<GuiComponentMgr>{
        StyleWriteRef::new(id, mgr.node.style.layout.to_usize(), mgr)
    }
}

#[wasm_bindgen]
pub struct FlexContainer {
    id: usize
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl FlexContainer {
    pub align_content(&self) -> Option<AlignContent>{
        unimplemented!()
    }

    pub align_items(&self) -> Option<AlignItems>{
        unimplemented!()
    }

    pub justify_ontent(&self) -> Option<JustifyContent>{
        unimplemented!()
    }

    pub flex_direction(&self) -> Option<FlexDirection>{
        unimplemented!()
    }

    pub flex_wrap(&self) -> Option<FlexWrap>{
        unimplemented!()
    }

    pub set_align_content(&self, value: Option<AlignContent>){
        unimplemented!()
    }

    pub set_align_items(&self, value: Option<AlignItems>){
        unimplemented!()
    }

    pub set_justify_ontent(&self, value: Option<JustifyContent>{
        unimplemented!()
    }

    pub set_flex_direction(&self, value: Option<FlexDirection>){
        unimplemented!()
    }

    pub set_flex_wrap(&self, value: Option<FlexWrap>){
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct FlexItem {
    id: usize
    world: Rc<RefCell<World<GuiComponentMgr, ()>>>,
}

#[wasm_bindgen]
impl FlexItem {
    pub flex_grow(&self) -> Option<f32> {
        unimplemented!()
    }

    pub flex_shrink(&self) -> Option<f32> {
        unimplemented!()
    }

    pub flex_basis(&self) -> StyleUnit {
        unimplemented!()
    }

    pub align_self(&self) -> Option<AlignSelf>{
        unimplemented!()
    }

    pub set_flex_grow(&self, value: Option<f32>){
        unimplemented!()
    }

    pub set_flex_shrink(&self, value: Option<f32>){
        unimplemented!()
    }

    pub set_flex_basis(&self, value: StyleUnit){
        unimplemented!()
    }

    pub set_align_self(&self, value: Option<AlignSelf>{
        unimplemented!()
    }
}


#[derive(Debug, Component, Default)]
pub struct Boundary{
    bottom: Option<StyleUnit>,
    left: Option<StyleUnit>,
    right: Option<StyleUnit>,
    top: Option<StyleUnit>,
}

#[derive(Debug, Component, Default)]
pub struct Rect {
    width: Option<StyleUnit>,
    height: Option<StyleUnit>,
}


#[derive(Debug, Component, Default)]
pub struct MinMax {
    max_height: Option<StyleUnit>,
    max_width: Option<StyleUnit>,
    min_height: Option<StyleUnit>,
    min_hidth: Option<StyleUnit>,
}

#[derive(Debug, Component, Default)]
pub struct Position {
    ty: Option<PositionType>,
    bottom: Option<StyleUnit>,
    left: Option<StyleUnit>,
    right: Option<StyleUnit>,
    top: Option<StyleUnit>,
}

#[allow(unused_attributes)]
#[derive(Debug, Component, Builder)]
pub struct Layout{
    #[component(FlexContainer)]
    pub container: usize,

    #[component(FlexItem)]
    pub item: usize,

    #[component(Boundary)]
    pub paddind: usize,

    #[component(Boundary)]
    pub margin: usize,

    #[component(Boundary)]
    pub border: usize,

    #[component(Boundary)]
    pub padding: usize,

    #[component(Rect)]
    pub wh: usize,

    #[component(Position)]
    pub position: usize,
}