/**
 * 样式标记
* StyleMarkSys系统会在Node实体创建时， 自动为Node创建一个StyleMark组件， 该组件用于标记了各种样式脏、是否为本地样式
* StyleMarkSys系统会监听本地样式的修改，以标记样式的脏， 并覆盖class设置的样式属性（覆盖的方式为：修改该属性的本地样式标记为1）
* StyleMarkSys系统会监听ClassName的修改， 遍历class中的属性， 如果该属性没有设置本地样式，将覆盖该属性对应的组件，并标记样式脏
* class中的图片， 是一个url， 在设置class时， 该图片资源可能还未加载， StyleMarkSys会将不存在的图片url放入ImageWaitSheet中， 由外部处理ImageWaitSheet中的等待列表，图片加载完成， 应该将图片放入完成列表中， 并通知ImageWaitSheet修改， 由StyleMarkSys来处理ImageWaitSheet中的完成列表
* StyleMarkSys系统监听ImageWaitSheet单例的修改， 将完成加载的图片设置在对应的Node组件上， 并标记样式脏
*/
use std::marker::PhantomData;

use ecs::{
    CreateEvent, DeleteEvent, EntityImpl, EntityListener, Event, ModifyEvent, MultiCaseImpl, MultiCaseListener, Runner, SingleCaseImpl,
    SingleCaseListener, StdCell, World,
};
use flex_layout::*;
use hal_core::*;
use hash::XHashSet;
use pi_style::style_type::ClassSheet;
use share::Share;

use crate::component::calc::*;
use crate::component::calc::{LayoutR, Opacity as COpacity};
use crate::component::user::*;
use crate::component::user::serialize::{StyleTypeReader, StyleAttr};
use crate::component::user::{Opacity, Overflow};
use crate::entity::Node;
use crate::render::engine::{Engine, ShareEngine};
use crate::render::res::TextureRes;
use crate::single::IdTree;
use crate::single::*;
use crate::world::GuiWorldExt;

lazy_static! {
	//文字样式脏
	static ref TEXT_DIRTY: StyleBit = style_bit().set_bit(StyleType::LetterSpacing as usize)
		.set_bit(StyleType::WordSpacing as usize)
		.set_bit(StyleType::LineHeight as usize)
		.set_bit(StyleType::TextIndent as usize)
		.set_bit(StyleType::WhiteSpace as usize)
		.set_bit(StyleType::TextAlign as usize)
		.set_bit(StyleType::VerticalAlign as usize)
		.set_bit(StyleType::TextShadow as usize)
		.set_bit(StyleType::Color as usize)
		.set_bit(StyleType::TextStroke as usize);

	//字体脏
	static ref FONT_DIRTY: StyleBit =
		style_bit().set_bit(StyleType::FontStyle as usize) .set_bit(StyleType::FontFamily as usize) .set_bit(StyleType::FontSize as usize) .set_bit(StyleType::FontWeight as usize);

	// 节点属性脏（不包含text， image， background等渲染属性）
	static ref NODE_DIRTY: StyleBit = style_bit().set_bit(StyleType::Hsi as usize) .set_bit(StyleType::Opacity as usize) .set_bit(StyleType::BorderRadius as usize);
	// 节点属性脏（不包含text， image， background等渲染属性）
	static ref NODE_DIRTY1: StyleBit = style_bit().set_bit(StyleType::Visibility as usize)
		.set_bit(StyleType::Enable as usize)
		.set_bit(StyleType::ZIndex as usize)
		.set_bit(StyleType::Transform as usize)
		.set_bit(StyleType::Display as usize);

	pub static ref TEXT_STYLE_DIRTY: StyleBit = TEXT_DIRTY.clone() | &*FONT_DIRTY | style_bit().set_bit(StyleType::TextShadow as usize);

	// 节点属性脏（不包含text， image， background等渲染属性）
	static ref IMAGE_DIRTY: StyleBit = style_bit().set_bit(StyleType::BackgroundImage as usize) .set_bit(StyleType::BackgroundImageClip as usize) .set_bit(StyleType::ObjectFit as usize);

	static ref MASK_IMAGE_DIRTY: StyleBit = style_bit().set_bit(StyleType::MaskImage as usize) .set_bit(StyleType::MaskImageClip as usize);
	// 节点属性脏（不包含text， image， background等渲染属性）
	static ref BORDER_IMAGE_DIRTY: StyleBit = style_bit().set_bit(StyleType::BorderImage as usize)
	.set_bit(StyleType::BorderImageClip as usize)
	.set_bit(StyleType::BorderImageSlice as usize)
	.set_bit(StyleType::BorderImageRepeat as usize);

	// 布局脏
	static ref LAYOUT_OTHER_DIRTY: StyleBit = style_bit().set_bit(StyleType::Width as usize)
	.set_bit(StyleType::Height as usize)
	.set_bit(StyleType::MinWidth as usize)
	.set_bit(StyleType::MinHeight as usize)
	.set_bit(StyleType::MaxHeight as usize)
	.set_bit(StyleType::MaxWidth as usize)
	.set_bit(StyleType::FlexShrink as usize)
	.set_bit(StyleType::FlexGrow as usize)
	.set_bit(StyleType::PositionType as usize)
	.set_bit(StyleType::FlexWrap as usize)
	.set_bit(StyleType::FlexDirection as usize)
	.set_bit(StyleType::AlignContent as usize)
	.set_bit(StyleType::AlignItems as usize)
	.set_bit(StyleType::AlignSelf as usize)
	.set_bit(StyleType::JustifyContent as usize)
	| &*LAYOUT_MARGIN_MARK
	| &*LAYOUT_PADDING_MARK
	| &*LAYOUT_BORDER_MARK
	| &*LAYOUT_POSITION_MARK;
}


type ImageTextureWrite<'a, C> = (
    &'a mut MultiCaseImpl<Node, BackgroundImage>,
    &'a mut MultiCaseImpl<Node, BorderImage>,
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, BackgroundImageClip>,
    &'a mut MultiCaseImpl<Node, BorderImageClip>,
    &'a mut MultiCaseImpl<Node, MaskImage>,
    &'a mut MultiCaseImpl<Node, MaskImageClip>,
    &'a mut MultiCaseImpl<Node, MaskTexture>,
    &'a mut MultiCaseImpl<Node, ImageTexture>,
    &'a mut MultiCaseImpl<Node, BorderImageTexture>,
);

type ReadData<'a> = (&'a MultiCaseImpl<Node, ClassName>, &'a SingleCaseImpl<Share<StdCell<ClassSheet>>>);
type WriteData<'a, C> = (
    &'a mut MultiCaseImpl<Node, TextStyle>,
    &'a mut MultiCaseImpl<Node, BackgroundImage>,
    &'a mut MultiCaseImpl<Node, BackgroundImageClip>,
    &'a mut MultiCaseImpl<Node, BackgroundImageMod>,
    &'a mut MultiCaseImpl<Node, BorderImage>,
    &'a mut MultiCaseImpl<Node, BorderImageClip>,
    &'a mut MultiCaseImpl<Node, BorderImageSlice>,
    &'a mut MultiCaseImpl<Node, BorderImageRepeat>,
    &'a mut MultiCaseImpl<Node, BorderColor>,
    &'a mut MultiCaseImpl<Node, BackgroundColor>,
    &'a mut MultiCaseImpl<Node, BoxShadow>,
    &'a mut MultiCaseImpl<Node, WorldMatrix>,
    &'a mut MultiCaseImpl<Node, Opacity>,
    &'a mut MultiCaseImpl<Node, Transform>,
    &'a mut MultiCaseImpl<Node, BorderRadius>,
    &'a mut MultiCaseImpl<Node, Hsi>,
    &'a mut MultiCaseImpl<Node, ZIndex>,
    &'a mut MultiCaseImpl<Node, Show>,
    &'a mut MultiCaseImpl<Node, Overflow>,
    &'a mut MultiCaseImpl<Node, StyleMark>,
    &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
    &'a mut MultiCaseImpl<Node, OtherLayoutStyle>,
    &'a mut SingleCaseImpl<IdTree>,
    &'a mut SingleCaseImpl<ShareEngine<C>>,
    &'a mut SingleCaseImpl<ImageWaitSheet>,
    &'a mut SingleCaseImpl<DirtyList>,
    &'a mut MultiCaseImpl<Node, MaskImage>,
    &'a mut MultiCaseImpl<Node, MaskImageClip>,
    &'a mut MultiCaseImpl<Node, MaskTexture>,
    &'a mut MultiCaseImpl<Node, BlendMode>,
    &'a mut MultiCaseImpl<Node, ImageTexture>,
    &'a mut MultiCaseImpl<Node, BorderImageTexture>,
    &'a mut MultiCaseImpl<Node, Blur>,
	&'a mut MultiCaseImpl<Node, ClipPath>,
);
pub struct ClassSetting<C> {
    dirtys: XHashSet<usize>,
	ext: GuiWorldExt,
    mark: PhantomData<C>,
}
impl<C> ClassSetting<C> {
    pub fn new(world: &mut World) -> Self {
        Self {
            dirtys: XHashSet::default(),
			ext: GuiWorldExt::new(world),
            mark: PhantomData,
        }
    }
}

// 将class的设置延迟
impl<'a, C: HalContext + 'static> Runner<'a> for ClassSetting<C> {
    type ReadData = (&'a MultiCaseImpl<Node, ClassName>, &'a SingleCaseImpl<Share<StdCell<ClassSheet>>>);
    type WriteData = WriteData<'a, C>;

    fn run(&mut self, (class_names, class_sheet): Self::ReadData, write: Self::WriteData) {
		let dirty_list = &mut **write.25;
        if self.dirtys.len() > 0 {
            for id in self.dirtys.iter() {
                let id = *id;
                if let Some(class) = class_names.get(id) {
                    let style_mark = &mut write.19[id];
					let (old_class_style_mark, local_style_mark) = (style_mark.class_style.clone(), style_mark.local_style.clone());
					let mut new_class_style_mark: StyleBit = StyleBit::default();
                    style_mark.class_style = StyleBit::default();

					let class_sheet = class_sheet.borrow();
                     // 设置class样式
					for i in class.iter() {
						if let Some(class) = class_sheet.class_map.get(i) {
							// log::warn!("set class1==========={:?}, {:?}", node, i);
							let mut style_reader = StyleTypeReader::new(&class_sheet.style_buffer, class.start, class.end);
							let is_write = |ty: StyleType| {
								// if !local_style_mark[ty as usize] {
								// 	count.fetch_add(1, Ordering::Relaxed);
								// }
								// if local_style_mark[ty as usize] {
								// 	log::warn!("!==========={:?}", ty);
								// }
								// 本地样式不存在，才会设置class样式
								!local_style_mark[ty as usize]
							};
							while let Some(ty) = style_reader
								.or_write_to_component(&mut new_class_style_mark, id, &mut self.ext, is_write) 
							{
								if !local_style_mark[ty as usize] {
									set_dirty(dirty_list, id, ty as usize, style_mark);
								}
							}
							// new_class_style_mark |= class.class_style_mark;
						}
					}

					// 旧的class_style中存在，但新的class_style和local_style中都不存在的样式，需要重置为默认值
					let mut cur_style_mark = new_class_style_mark | local_style_mark;
					let invalid_style = old_class_style_mark ^ cur_style_mark & old_class_style_mark;
					let buffer = Vec::new();
					for i in invalid_style.iter_ones() {
						// count.fetch_add(1, Ordering::Relaxed);
						StyleAttr::reset(&mut cur_style_mark, i as u8, &buffer, 0, &mut self.ext, id);
					}
					if invalid_style.any() {
						set_dirty_many(dirty_list, id, invalid_style, style_mark);
					}

					style_mark.class_style = new_class_style_mark;
                }
            }
            self.dirtys.clear();
        }
    }
}


pub struct StyleMarkSys<C> {
    show: Show,
    mark: PhantomData<C>,
}

impl<'a, C: HalContext + 'static> StyleMarkSys<C> {
    pub fn new() -> Self {
        Self {
            show: Show::default(),
            mark: PhantomData,
        }
    }
}

// #[inline]
// fn set_local_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
//     let style_mark = &mut style_marks[id];
//     set_dirty(dirty_list, id, ty, style_mark);
//     style_mark.local_style |= ty;
// }

#[inline]
fn set_local_dirty1(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
    let style_mark = match style_marks.get_mut(id) {
        Some(r) => r,
        None => return,
    };
    set_dirty1(dirty_list, id, ty, style_mark);
    style_mark.style |= ty;
}

// #[inline]
// fn set_local_dirty2(dirty_list: &mut DirtyList, id: usize, ty: usize, style_marks: &mut MultiCaseImpl<Node, StyleMark>) {
//     let style_mark = &mut style_marks[id];
//     set_dirty2(dirty_list, id, ty, style_mark);
//     style_mark.local_style2 |= ty;
// }

#[inline]
pub fn set_dirty(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty.not_any(){
        dirty_list.0.push(id);
    }

    style_mark.dirty.set(ty, true);
}
#[inline]
pub fn set_dirty_many(dirty_list: &mut DirtyList, id: usize, ty: StyleBit, style_mark: &mut StyleMark) {
    if style_mark.dirty.not_any(){
        dirty_list.0.push(id);
    }

    style_mark.dirty |= ty;
}

#[inline]
pub fn set_dirty1(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
    if style_mark.dirty.not_any() && style_mark.dirty1 == 0 {
        dirty_list.0.push(id);
    }
    style_mark.dirty1 |= ty;
}

// #[inline]
// pub fn set_dirty2(dirty_list: &mut DirtyList, id: usize, ty: usize, style_mark: &mut StyleMark) {
//     if style_mark.dirty == 0 && style_mark.dirty1 == 0 && style_mark.dirty2 == 0 {
//         dirty_list.0.push(id);
//     }
//     style_mark.dirty2 |= ty;
// }

impl<'a, C: HalContext + 'static> Runner<'a> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn run(&mut self, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        for id in dirty_list.0.iter() {
            match style_marks.get_mut(*id) {
                Some(style_mark) => {
                    style_mark.dirty = StyleBit::default();
                    style_mark.dirty1 = 0;
                    style_mark.dirty_other = 0;
                }
                None => (),
            }
        }
        dirty_list.0.clear();
    }
}

// 监听节点的创建， 插入StyleMark， ClassName组件
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, CreateEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut MultiCaseImpl<Node, ClassName>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, (style_marks, class_names): Self::WriteData) {
        style_marks.insert(event.id, StyleMark::default());
        class_names.insert_no_notify(event.id, ClassName::default());
    }
}

// 监听节点修改事件，添加到脏列表
impl<'a, C: HalContext + 'static> EntityListener<'a, Node, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);

    fn listen(&mut self, event: &Event, _read: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, CalcType::Delete as usize, r);
        }
    }
}

// 文字内容创建， 将文字样式全部设脏
impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, TextContent, CreateEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        // &'a mut MultiCaseImpl<Node, TextStyle>,
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
    );

    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (/*text_styles, */ style_marks, dirty_list) = write;
        // text_styles.insert_no_notify(event.id, self.default_text.clone());
        let style_mark = &mut style_marks[event.id];
        set_dirty_many(dirty_list, event.id, TEXT_STYLE_DIRTY.set_bit(StyleType::TextContent as usize), style_mark);
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskTexture, (CreateEvent, ModifyEvent, DeleteEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, CalcType::MaskImageTexture as usize, r);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageTexture, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, BackgroundImageClip>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
    );
    fn listen(&mut self, event: &Event, _: Self::ReadData, write: Self::WriteData) {
		log::warn!("ImageTexture change====={:?}", event.id, );
        let (style_marks, dirty_list, layout_styles, image_clips, image_textures) = write;
        let id = event.id;
        set_dirty1(dirty_list, id, CalcType::BackgroundImageTexture as usize, &mut style_marks[id]);

        if let Some(texture) = image_textures.get(id) {
            if let ImageTexture::All(texture, _url) = texture {
                set_image_size(texture, &mut layout_styles[id], image_clips.get(id), &mut style_marks[id]);
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ImageTexture, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = event.id;
        // 当节点不再树上时，设置脏没有意义
        if let Some(r) = idtree.get(id) {
            if r.layer() > 0 {
                if let Some(style_mark) = style_marks.get_mut(id) {
                    set_dirty1(dirty_list, id, CalcType::BackgroundImageTexture as usize, style_mark);
                }
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImageTexture, (CreateEvent, ModifyEvent, DeleteEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, CalcType::BorderImageTexture as usize, r);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ContentBox, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _: Self::ReadData, (style_marks, dirty_list): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, CalcType::ContentBox as usize, r);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, MaskImage, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (&'a mut SingleCaseImpl<DirtyList>, &'a mut MultiCaseImpl<Node, StyleMark>);
    fn listen(&mut self, event: &Event, _idtree: Self::ReadData, (dirty_list, style_marks): Self::WriteData) {
        if let Some(r) = style_marks.get_mut(event.id) {
            set_dirty1(dirty_list, event.id, CalcType::ContentBox as usize, r);
        }
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundImage, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut MultiCaseImpl<Node, BackgroundImage>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, BackgroundImageClip>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
    );
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        // set_image_local(event.id, idtree, write);
        let id = event.id;
        if let Some(n) = idtree.get(id) {
            if n.layer() > 0 {
                let image = &mut write.4[id];
                set_image(id, write.2, write.3, image, write.7, ImageType::BorderImage);
            }
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundImage, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, ImageTexture>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, image_textutes: Self::WriteData) {
        let id = event.id;
        image_textutes.delete(id); // border_image删除时，删除对应的纹理set_local_dirty
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BackgroundImageClip, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, BackgroundImageClip>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
    );
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, _dirty_list, layout_styles, image_clips, image_textures) = write;
        let id = event.id;

        if let Some(texture) = image_textures.get(id) {
            if let ImageTexture::All(texture, _) = texture {
                set_image_size(texture, &mut layout_styles[id], image_clips.get(id), &mut style_marks[id]);
            }
        }
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, (CreateEvent, ModifyEvent)> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<DirtyList>,
        &'a mut SingleCaseImpl<ShareEngine<C>>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        &'a mut MultiCaseImpl<Node, BorderImage>,
        &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, BorderImageClip>,
        &'a mut MultiCaseImpl<Node, BorderImageTexture>,
    );
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        let id = event.id;

        if let Some(_) = idtree.get(id) {
            let image = &mut write.4[id];
            set_border_image(id, write.2, write.3, image, write.7, ImageType::BorderImage);
        }
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, BorderImage, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = &'a mut MultiCaseImpl<Node, BorderImageTexture>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, border_image_textutes: Self::WriteData) {
        let id = event.id;
        border_image_textutes.delete(id); // border_image删除时，删除对应的纹理
    }
}



// // type BorderImageWrite<'a, C> = (
// //     &'a mut MultiCaseImpl<Node, StyleMark>,
// //     &'a mut SingleCaseImpl<DirtyList>,
// //     &'a mut SingleCaseImpl<ShareEngine<C>>,
// //     &'a mut SingleCaseImpl<ImageWaitSheet>,
// //     &'a mut MultiCaseImpl<Node, BorderImage>,
// //     &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
// //     &'a mut MultiCaseImpl<Node, BorderImageClip>,
// // );

// // type ImageWrite<'a, C> = (
// //     &'a mut MultiCaseImpl<Node, StyleMark>,
// //     &'a mut SingleCaseImpl<DirtyList>,
// //     &'a mut SingleCaseImpl<ShareEngine<C>>,
// //     &'a mut SingleCaseImpl<ImageWaitSheet>,
// //     &'a mut MultiCaseImpl<Node, Image>,
// //     &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
// //     &'a mut MultiCaseImpl<Node, ImageClip>,
// // 	&'a mut MultiCaseImpl<Node, ImageTexture>,
// // );

// type MaskImageWrite<'a, C> = (
//     &'a mut MultiCaseImpl<Node, StyleMark>,
//     &'a mut SingleCaseImpl<DirtyList>,
//     &'a mut SingleCaseImpl<ShareEngine<C>>,
//     &'a mut SingleCaseImpl<ImageWaitSheet>,
//     &'a mut MultiCaseImpl<Node, MaskImage>,
//     &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
//     &'a mut MultiCaseImpl<Node, MaskImageClip>,
//     &'a mut MultiCaseImpl<Node, MaskTexture>,
// );


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ByOverflow, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        set_local_dirty1(dirty_list, event.id, CalcType::ByOverflow as usize, style_marks);
    }
}


// // // visibility修改， 设置ByOverflow脏（clipsys 使用， dirty上没有位置容纳Visibility脏了， 因此设置在ByOverflow上）
// // impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, Visibility, ModifyEvent> for StyleMarkSys<C>{
// // 	type ReadData = ();
// // 	type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
// // 	fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData){
// // 		let (style_marks, dirty_list) = write;
// // 		set_local_dirty(dirty_list, event.id, StyleType::ByOverflow as usize, style_marks);
// // 	}
// // }

// RenderObjs 创建， 设置ByOverflow脏
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, RenderObjs, CreateEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<RenderObjs>;
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, render_objs: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let id = render_objs[event.id].context;
        let style_mark = &mut style_marks[id];
        set_dirty1(dirty_list, id, CalcType::ByOverflow as usize, style_mark);
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, COpacity, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, LayoutR, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        // 虚拟节点不存在StyleMark组件
        let style_mark = match style_marks.get_mut(event.id) {
            Some(r) => r,
            None => return,
        };
        set_dirty1(dirty_list, event.id, CalcType::Layout as usize, style_mark);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    // type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
    // type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty1(dirty_list, event.id, CalcType::Oct as usize, style_mark);
    }
}

// 包围盒修改，
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, Oct, CreateEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    // type ReadData = (&'a SingleCaseImpl<Oct>, &'a SingleCaseImpl<RenderBegin>);
    // type WriteData = &'a mut SingleCaseImpl<DirtyViewRect>;
    fn listen(&mut self, event: &Event, _: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty1(dirty_list, event.id, CalcType::Oct as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, WorldMatrix, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty1(dirty_list, event.id, CalcType::Matrix as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ZDepth, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = ();
    type WriteData = (&'a mut MultiCaseImpl<Node, StyleMark>, &'a mut SingleCaseImpl<DirtyList>);
    fn listen(&mut self, event: &Event, _read: Self::ReadData, write: Self::WriteData) {
        let (style_marks, dirty_list) = write;
        let style_mark = &mut style_marks[event.id];
        set_dirty1(dirty_list, event.id, CalcType::Matrix as usize, style_mark);
    }
}

impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, CreateEvent> for StyleMarkSys<C> {
    type ReadData = (&'a SingleCaseImpl<IdTree>, &'a MultiCaseImpl<Node, NodeState>);
    type WriteData = ImageTextureWrite<'a, C>;
    fn listen(&mut self, event: &Event, (idtree, node_states): Self::ReadData, mut write: Self::WriteData) {
        idtree_create(event.id, &idtree, &node_states, &mut write);
    }
}

fn idtree_create<C: HalContext + 'static>(
    id: usize,
    idtree: &IdTree,
    node_states: &MultiCaseImpl<Node, NodeState>,
    write: &mut ImageTextureWrite<'_, C>,
) {
    let node_state = match node_states.get(id) {
        Some(r) => r,
        None => return,
    };

    if !node_state.0.is_rnode() {
        return;
    }

    load_image(id, write);
    set_local_dirty1(&mut write.3, id, CalcType::Create as usize, &mut write.2);
    let mark = &mut write.2[id];
    let (dirty, dirty1) = (
        mark.local_style | mark.class_style,
        mark.style,
    );
    mark.dirty |= dirty;
    mark.dirty1 |= dirty1;

    let head = idtree[id].children().head;
    for (id, _n) in idtree.iter(head) {
        idtree_create(id, idtree, node_states, write);
    }
}


impl<'a, C: HalContext + 'static> SingleCaseListener<'a, IdTree, DeleteEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = ImageTextureWrite<'a, C>;
    fn listen(&mut self, event: &Event, idtree: Self::ReadData, mut write: Self::WriteData) {
        release_image(event.id, &mut write);

        let node = &idtree[event.id];
        for (id, _n) in idtree.recursive_iter(node.children().head) {
            release_image(id, &mut write);
        }
    }
}


impl<'a, C: HalContext + 'static> MultiCaseListener<'a, Node, ClassName, ModifyEvent> for ClassSetting<C> {
    type ReadData = ();
    type WriteData = ();
    fn listen(&mut self, event: &Event, _: Self::ReadData, _: Self::WriteData) { self.dirtys.insert(event.id); }
}


// 监听图片等待列表的改变， 将已加载完成的图片设置到对应的组件上
impl<'a, C: HalContext + 'static> SingleCaseListener<'a, ImageWaitSheet, ModifyEvent> for StyleMarkSys<C> {
    type ReadData = &'a SingleCaseImpl<IdTree>;
    type WriteData = (
        &'a mut EntityImpl<Node>,
        // &'a mut MultiCaseImpl<Node, RectLayoutStyle>,
        &'a mut MultiCaseImpl<Node, BackgroundImage>,
        &'a mut MultiCaseImpl<Node, ImageTexture>,
        // &'a mut MultiCaseImpl<Node, ImageClip>,
        &'a mut MultiCaseImpl<Node, BorderImage>,
        &'a mut MultiCaseImpl<Node, BorderImageTexture>,
        &'a mut MultiCaseImpl<Node, MaskImage>,
        &'a mut MultiCaseImpl<Node, MaskTexture>,
        // &'a mut MultiCaseImpl<Node, BorderImageClip>,
        // &'a mut MultiCaseImpl<Node, StyleMark>,
        &'a mut SingleCaseImpl<ImageWaitSheet>,
        // &'a mut SingleCaseImpl<DirtyList>,
    );
    fn listen(&mut self, _event: &Event, idtree: Self::ReadData, write: Self::WriteData) {
        let (
            entitys,
            // layout_nodes,
            images,
            image_textures,
            // image_clips,
            border_images,
            border_image_textures,
            // border_image_clips,
            mask_images,
            mask_textures,
            // style_marks,
            image_wait_sheet,
            // dirty_list,
        ) = write;

        for wait in image_wait_sheet.finish.iter() {
            for image_wait in wait.2.iter() {
                // 图片加载完成后， 节点可能已经删除， 因此跳过
                if !entitys.is_exist(image_wait.id) {
                    continue;
                }
                // 节点不可见， 跳过
                if idtree.get(image_wait.id).is_none() {
                    continue;
                }
				log::warn!("image_wait success: {:?}, {:?}, {:?}", image_wait.id, &image_wait.ty, &wait.0);
                // 判断等待类型， 设置对应的组件
                match image_wait.ty {
                    ImageType::Image => {
                        if let Some(image) = images.get_mut(image_wait.id) {
                            if image.0 == wait.0 {
                                image_textures.insert(image_wait.id, ImageTexture::All(wait.1.clone(), image.0.clone()));
								

                                // set_image_size(
                                //     &wait.1,
                                //     &mut layout_nodes[image_wait.id],
                                //     image_clips.get(image_wait.id),
                                //     &mut style_marks[image_wait.id],
                                // );

                                // // if Some image_clips.get(image_wait.id)
                                // set_local_dirty(
                                //     dirty_list,
                                //     image_wait.id,
                                //     StyleType::Image as usize,
                                //     style_marks,
                                // );
                            }
                        }
                    }
                    ImageType::BorderImage => {
                        if let Some(image) = border_images.get_mut(image_wait.id) {
                            if image.0 == wait.0 {
                                border_image_textures.insert(image_wait.id, BorderImageTexture(wait.1.clone()));
                            }
                        }
                    }
                    ImageType::MaskImage => {
                        if let Some(image) = mask_images.get_mut(image_wait.id) {
                            if let pi_style::style::MaskImage::Path(url) = &image.0 {
                                if *url == wait.0 {
                                    mask_textures.insert(image_wait.id, MaskTexture::All(wait.1.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        image_wait_sheet.finish.clear(); // 清空
    }
}

fn set_image_size(src: &Share<TextureRes>, layout_style: &mut RectLayoutStyle, image_clip: Option<&BackgroundImageClip>, style_mark: &mut StyleMark) {
    let img_clip;
    let image_clip = match image_clip {
        Some(r) => {
			img_clip = Aabb2::new(Point2::new(*r.left, *r.top), Point2::new(*r.right, *r.bottom));
			&img_clip
		},
        None => {
            img_clip = Aabb2::new(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0));
            &img_clip
        }
    };

    if !style_mark.local_style[StyleType::Width as usize] && !style_mark.class_style[StyleType::Width as usize] {
        layout_style.size.width = Dimension::Points(src.width as f32 * (image_clip.maxs.x - image_clip.mins.x));
        // set_image_size必然是在样式脏的情况下调用，只需要标记脏类型，无需再次添加到脏列表
        style_mark.dirty.set(StyleType::Width as usize, true);
    }

    if !style_mark.local_style[StyleType::Height as usize] && !style_mark.class_style[StyleType::Height as usize] {
        layout_style.size.height = Dimension::Points(src.height as f32 * (image_clip.maxs.y - image_clip.mins.y));
        // set_image_size必然是在样式脏的情况下调用，只需要标记脏类型，无需再次添加到脏列表
		style_mark.dirty.set(StyleType::Height as usize, true);
    }
}


// #[inline]
// fn reset_attr<C: HalContext>(id: usize, read: ReadData, write: &mut WriteData<C>, old_style: usize, old_style1: usize, old_style2: usize) {
//     let (_class_names, _class_sheet) = read;
//     let (
//         text_styles,
//         images,
//         image_clips,
//         obj_fits,
//         border_images,
//         border_image_clips,
//         border_image_slices,
//         border_image_repeats,
//         border_colors,
//         background_colors,
//         box_shadows,
//         _world_matrixs,
//         opacitys,
//         transforms,
//         border_radiuss,
//         filters,
//         zindexs,
//         shows,
//         _overflows,
//         style_marks,
//         rect_layout_styles,
//         other_layout_styles,
//         _idtree,
//         _engine,
//         _image_wait_sheet,
//         dirty_list,
//         mask_images,
//         mask_clips,
//         _mask_textures,
//         blend_modes,
//         _image_textures,
//         _border_image_textures,
//         blur,
// 		clip_paths,
//     ) = write;

//     let rect_layout_style = &mut rect_layout_styles[id];
//     let other_layout_style = &mut other_layout_styles[id];
//     let style_mark = &mut style_marks[id];
//     // old_style中为1， class_style和local_style不为1的属性, 应该删除
//     let old_style = !(!old_style | (old_style & (style_mark.class_style | style_mark.local_style)));
//     let old_style1 = !(!old_style1 | (old_style1 & (style_mark.class_style1 | style_mark.local_style1)));
//     let old_style2 = !(!old_style2 | (old_style2 & (style_mark.class_style2 | style_mark.local_style2)));
//     if old_style != 0 {
//         if old_style & TEXT_STYLE_DIRTY != 0 {
//             let defualt_text = unsafe { &*(&text_styles[0] as *const TextStyle as usize as *const TextStyle) };
//             if let Some(text_style) = text_styles.get_mut(id) {
//                 if old_style & TEXT_DIRTY != 0 {
//                     if old_style & StyleType::LetterSpacing as usize != 0 {
//                         text_style.text.letter_spacing = defualt_text.text.letter_spacing;
//                         set_dirty(dirty_list, id, StyleType::LetterSpacing as usize, style_mark);
//                     }
//                     if old_style & StyleType::WordSpacing as usize != 0 {
//                         text_style.text.word_spacing = defualt_text.text.word_spacing;
//                         set_dirty(dirty_list, id, StyleType::WordSpacing as usize, style_mark);
//                     }
//                     if old_style & StyleType::LineHeight as usize != 0 {
//                         text_style.text.line_height = defualt_text.text.line_height;
//                         set_dirty(dirty_list, id, StyleType::LineHeight as usize, style_mark);
//                     }
//                     if old_style & StyleType::Indent as usize != 0 {
//                         text_style.text.indent = defualt_text.text.indent;
//                         set_dirty(dirty_list, id, StyleType::Indent as usize, style_mark);
//                     }
//                     if old_style & StyleType::WhiteSpace as usize != 0 {
//                         text_style.text.white_space = defualt_text.text.white_space;
//                         set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
//                     }

//                     if old_style & StyleType::Color as usize != 0 {
//                         text_style.text.color = defualt_text.text.color.clone();
//                         set_dirty(dirty_list, id, StyleType::Color as usize, style_mark);
//                     }

//                     if old_style & StyleType::Stroke as usize != 0 {
//                         text_style.text.stroke = defualt_text.text.stroke.clone();
//                         set_dirty(dirty_list, id, StyleType::Stroke as usize, style_mark);
//                     }

//                     if old_style & StyleType::TextAlign as usize != 0 {
//                         text_style.text.text_align = defualt_text.text.text_align;
//                         set_dirty(dirty_list, id, StyleType::TextAlign as usize, style_mark);
//                     }

//                     if old_style & StyleType::VerticalAlign as usize != 0 {
//                         text_style.text.vertical_align = defualt_text.text.vertical_align;
//                         set_dirty(dirty_list, id, StyleType::VerticalAlign as usize, style_mark);
//                     }
//                 }

//                 if old_style & StyleType::TextShadow as usize != 0 {
//                     text_style.shadow = defualt_text.shadow.clone();
//                     set_dirty(dirty_list, id, StyleType::TextShadow as usize, style_mark);
//                 }

//                 if old_style & FONT_DIRTY == 0 {
//                     if old_style & StyleType::FontStyle as usize != 0 {
//                         text_style.font.style = defualt_text.font.style;
//                         set_dirty(dirty_list, id, StyleType::FontStyle as usize, style_mark);
//                     }
//                     if old_style & StyleType::FontWeight as usize != 0 {
//                         text_style.font.weight = defualt_text.font.weight;
//                         set_dirty(dirty_list, id, StyleType::FontWeight as usize, style_mark);
//                     }
//                     if old_style & StyleType::FontSize as usize != 0 {
//                         text_style.font.size = defualt_text.font.size;
//                         set_dirty(dirty_list, id, StyleType::FontSize as usize, style_mark);
//                     }
//                     if old_style & StyleType::FontFamily as usize != 0 {
//                         text_style.font.family = defualt_text.font.family.clone();
//                         set_dirty(dirty_list, id, StyleType::FontFamily as usize, style_mark);
//                     }
//                 }
//             }
//         }

//         if old_style & IMAGE_DIRTY != 0 {
//             if old_style & StyleType::Image as usize != 0 {
//                 images.delete(id);
//             }
//             if old_style & StyleType::ImageClip as usize != 0 {
//                 image_clips.delete(id);
//                 set_dirty(dirty_list, id, StyleType::ImageClip as usize, style_mark);
//             }
//             if old_style & StyleType::ObjectFit as usize != 0 {
//                 obj_fits.delete(id);
//                 set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
//             }
//         }

//         if old_style & BORDER_IMAGE_DIRTY != 0 {
//             if old_style & StyleType::BorderImage as usize != 0 {
//                 border_images.delete(id);
//             }
//             if old_style & StyleType::BorderImageClip as usize != 0 {
//                 border_image_clips.delete(id);
//                 set_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_mark);
//             }
//             if old_style & StyleType::BorderImageSlice as usize != 0 {
//                 border_image_slices.delete(id);
//                 set_dirty(dirty_list, id, StyleType::BorderImageSlice as usize, style_mark);
//             }
//             if old_style & StyleType::BorderImageRepeat as usize != 0 {
//                 border_image_repeats.delete(id);
//                 set_dirty(dirty_list, id, StyleType::BorderImageRepeat as usize, style_mark);
//             }
//         }

//         if old_style & StyleType::BorderColor as usize != 0 {
//             border_colors.delete(id);
//             set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
//         }

//         if old_style & StyleType::BackgroundColor as usize != 0 {
//             background_colors.delete(id);
//             set_dirty(dirty_list, id, StyleType::BackgroundColor as usize, style_mark);
//         }

//         if old_style & StyleType::BoxShadow as usize != 0 {
//             box_shadows.delete(id);
//             set_dirty(dirty_list, id, StyleType::BoxShadow as usize, style_mark);
//         }

//         if old_style & NODE_DIRTY != 0 {
//             if old_style & StyleType::Opacity as usize != 0 {
//                 opacitys.delete(id);
//                 set_dirty(dirty_list, id, StyleType::Opacity as usize, style_mark);
//             }

//             if old_style & StyleType::BorderRadius as usize != 0 {
//                 border_radiuss.delete(id);
//                 set_dirty(dirty_list, id, StyleType::BorderRadius as usize, style_mark);
//             }

//             if old_style & StyleType::Filter as usize != 0 {
//                 filters.delete(id);
//                 set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
//             }
//         }

//         if old_style1 & StyleType::MaskImage as usize != 0 {
//             mask_images.delete(id);
//             set_dirty1(dirty_list, id, StyleType::MaskImage as usize, style_mark);
//         }
//         if old_style1 & StyleType::MaskImageClip as usize != 0 {
//             mask_clips.delete(id);
//             set_dirty1(dirty_list, id, StyleType::MaskImageClip as usize, style_mark);
//         }

// 		if old_style1 & StyleType::ClipPath as usize != 0 {
//             clip_paths.delete(id);
//             set_dirty1(dirty_list, id, StyleType::ClipPath as usize, style_mark);
//         }

//         if old_style2 & StyleType::BlendMode as usize != 0 {
//             blend_modes.delete(id);
//             set_dirty2(dirty_list, id, StyleType::BlendMode as usize, style_mark);
//         }
//     }

//     if old_style1 != 0 {
//         if old_style1 & NODE_DIRTY1 != 0 {
//             if old_style1 & StyleType::Enable as usize != 0
//                 || old_style1 & StyleType::Display as usize != 0
//                 || old_style1 & StyleType::Visibility as usize != 0
//             {
//                 if let Some(show) = shows.get_mut(id) {
//                     if old_style1 & StyleType::Enable as usize != 0 {
//                         show.set_enable(EnableType::Auto);
//                     }
//                     if old_style1 & StyleType::Display as usize != 0 {
//                         other_layout_style.display = Display::Flex;
//                         show.set_display(Display::Flex);
//                     }
//                     if old_style1 & StyleType::Visibility as usize != 0 {
//                         show.set_visibility(true);
//                     }
//                 }
//                 shows.get_notify_ref().modify_event(id, "", 0);

//                 if old_style1 & StyleType::ZIndex as usize != 0 {
//                     zindexs.insert_no_notify(id, ZIndex(0));
//                     // 字段为class， zindex的监听器不会设置zinde为本地样式
//                     zindexs.get_notify_ref().modify_event(id, "class", 0);
//                 }

//                 if old_style1 & StyleType::Transform as usize != 0 {
//                     transforms.delete(id);
//                 }
//             }
//         }

//         if old_style1 & StyleType::FlexBasis as usize != 0 {
//             other_layout_style.flex_basis = Dimension::Undefined;
//         }
//     }

//     if old_style2 != 0 {
//         if old_style2 & LAYOUT_RECT_MARK != 0 {
//             if old_style2 & StyleType::Width as usize != 0 {
//                 rect_layout_style.size.width = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::Height as usize != 0 {
//                 rect_layout_style.size.height = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MarginTop as usize != 0 {
//                 rect_layout_style.margin.top = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MarginRight as usize != 0 {
//                 rect_layout_style.margin.end = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MarginBottom as usize != 0 {
//                 rect_layout_style.margin.bottom = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MarginLeft as usize != 0 {
//                 rect_layout_style.margin.start = Dimension::Undefined;
//             }
//             // *rect_layout_style = RectLayoutStyle::default();
//             // reset_layout_attr(layout_style, old_style2);
//         }

//         if old_style2 & LAYOUT_OTHER_DIRTY != 0 {
//             if old_style2 & StyleType::PaddingTop as usize != 0 {
//                 other_layout_style.padding.top = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::PaddingRight as usize != 0 {
//                 other_layout_style.padding.end = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::PaddingBottom as usize != 0 {
//                 other_layout_style.padding.bottom = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::PaddingLeft as usize != 0 {
//                 other_layout_style.padding.start = Dimension::Undefined;
//             }

//             if old_style2 & StyleType::BorderTop as usize != 0 {
//                 other_layout_style.border.top = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::BorderRight as usize != 0 {
//                 other_layout_style.border.end = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::BorderBottom as usize != 0 {
//                 other_layout_style.border.bottom = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::BorderLeft as usize != 0 {
//                 other_layout_style.border.start = Dimension::Undefined;
//             }

//             if old_style2 & StyleType::PositionTop as usize != 0 {
//                 other_layout_style.position.top = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::PositionRight as usize != 0 {
//                 other_layout_style.position.end = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::PositionBottom as usize != 0 {
//                 other_layout_style.position.bottom = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::PositionLeft as usize != 0 {
//                 other_layout_style.position.start = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MinWidth as usize != 0 {
//                 other_layout_style.min_size.width = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MinHeight as usize != 0 {
//                 other_layout_style.min_size.height = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MaxWidth as usize != 0 {
//                 other_layout_style.max_size.width = Dimension::Undefined;
//             }
//             if old_style2 & StyleType::MaxHeight as usize != 0 {
//                 other_layout_style.max_size.height = Dimension::Undefined;
//             }

//             if old_style2 & StyleType::FlexShrink as usize != 0 {
//                 other_layout_style.flex_shrink = 0.0;
//             }
//             if old_style2 & StyleType::FlexGrow as usize != 0 {
//                 other_layout_style.flex_grow = 0.0;
//             }
//             if old_style2 & StyleType::PositionType as usize != 0 {
//                 other_layout_style.position_type = PositionType::Absolute;
//             }
//             if old_style2 & StyleType::FlexWrap as usize != 0 {
//                 other_layout_style.flex_wrap = FlexWrap::NoWrap;
//             }
//             if old_style2 & StyleType::FlexDirection as usize != 0 {
//                 other_layout_style.flex_direction = FlexDirection::Row;
//             }
//             if old_style2 & StyleType::AlignContent as usize != 0 {
//                 other_layout_style.align_content = AlignContent::FlexStart;
//             }
//             if old_style2 & StyleType::AlignItems as usize != 0 {
//                 other_layout_style.align_items = AlignItems::FlexStart;
//             }
//             if old_style2 & StyleType::AlignSelf as usize != 0 {
//                 other_layout_style.align_self = AlignSelf::FlexStart;
//             }
//             if old_style2 & StyleType::JustifyContent as usize != 0 {
//                 other_layout_style.justify_content = JustifyContent::FlexStart;
//             }

//             // *other_layout_style = OtherLayoutStyle::default();
//             // reset_layout_attr(layout_style, old_style2);
//         }
//     }
// }

// // fn reset_layout_attr(layout_style: &LayoutStyle, old_style1: usize) {
// //     if old_style1 & StyleType::Width as usize != 0 {
// // 		layout_style.size.width = Dimension::undefined;
// //     }
// //     if old_style1 & StyleType::Height as usize != 0 {
// // 		layout_style.size.height = Dimension::undefined;
// //     }
// //     if old_style1 & StyleType::Margin as usize != 0 {
// // 		layout_style.margin.start = Dimension::undefined;
// // 		layout_style.margin.end = Dimension::undefined;
// // 		layout_style.margin.top = Dimension::undefined;
// // 		layout_style.margin.bottom = Dimension::undefined;
// //     }
// //     if old_style1 & StyleType::Padding as usize != 0 {
// //         layout_style.padding.start = Dimension::undefined;
// // 		layout_style.size.padding.end = Dimension::undefined;
// // 		layout_style.size.padding.top = Dimension::undefined;
// // 		layout_style.size.padding.bottom = Dimension::undefined;
// //     }
// //     if old_style1 & StyleType::Border as usize != 0 {
// //         layout_style.size.border.start = Dimension::undefined;
// // 		layout_style.size.border.end = Dimension::undefined;
// // 		layout_style.size.border.top = Dimension::undefined;
// // 		layout_style.size.border.bottom = Dimension::undefined;
// //     }
// //     if old_style1 & StyleType::Position as usize != 0 {
// // 		layout_style.position = Rect{start:Dimension::undefined, end: Dimension::undefined, top: Dimension::undefined, top: Dimension::bottom};
// //     }
// //     if old_style1 & StyleType::MinWidth as usize != 0 || old_style1 & StyleType::MinHeight as usize != 0 {
// // 		layout_style.min_size = Size{width: Dimension::undefined, height: Dimension::undefined};
// //     }

// //     if old_style1 & StyleType::MaxWidth as usize != 0 {
// //         layout_style.set_max_width(std::f32::NAN);
// //     }
// //     if old_style1 & StyleType::MaxHeight as usize != 0 {
// //         layout_style.set_max_height(std::f32::NAN);
// //     }
// //     if old_style1 & StyleType::FlexBasis as usize != 0 {
// //         layout_style.set_flex_basis(std::f32::NAN);
// //     }
// //     if old_style1 & StyleType::FlexShrink as usize != 0 {
// //         layout_style.set_flex_shrink(std::f32::NAN);
// //     }
// //     if old_style1 & StyleType::FlexGrow as usize != 0 {
// //         layout_style.set_flex_grow(std::f32::NAN);
// //     }
// //     if old_style1 & StyleType::PositionType as usize != 0 {
// //         layout_style.set_position_type(YGPositionType::YGPositionTypeAbsolute);
// //     }
// //     if old_style1 & StyleType::FlexWrap as usize != 0 {
// //         layout_style.set_flex_wrap(YGWrap::YGWrapWrap);
// //     }
// //     if old_style1 & StyleType::FlexDirection as usize != 0 {
// //         layout_style.set_flex_direction(YGFlexDirection::YGFlexDirectionRow);
// //     }
// //     if old_style1 & StyleType::AlignContent as usize != 0 {
// //         layout_style.set_align_content(YGAlign::YGAlignFlexStart);
// //     }
// //     if old_style1 & StyleType::AlignItems as usize != 0 {
// //         layout_style.set_align_items(YGAlign::YGAlignFlexStart);
// //     }
// //     if old_style1 & StyleType::AlignSelf as usize != 0 {
// //         layout_style.set_align_self(YGAlign::YGAlignFlexStart);
// //     }
// //     if old_style1 & StyleType::JustifyContent as usize != 0 {
// //         layout_style.set_justify_content(YGJustify::YGJustifyFlexStart);
// //     }
// // }

// fn set_attr<C: HalContext>(id: usize, class_name: usize, read: ReadData, write: &mut WriteData<C>) {
//     if class_name == 0 {
//         return;
//     }
//     let (_class_names, class_sheet) = read;
//     let (
//         text_styles,
//         images,
//         image_clips,
//         obj_fits,
//         border_images,
//         border_image_clips,
//         border_image_slices,
//         border_image_repeats,
//         border_colors,
//         background_colors,
//         box_shadows,
//         _world_matrixs,
//         opacitys,
//         transforms,
//         border_radiuss,
//         filters,
//         zindexs,
//         shows,
//         overflows,
//         style_marks,
//         rect_layout_styles,
//         other_layout_styles,
//         idtree,
//         engine,
//         image_wait_sheet,
//         dirty_list,
//         mask_images,
//         mask_image_clips,
//         mask_textures,
//         blend_modes,
//         image_textures,
//         border_image_textures,
//         blurs,
// 		clip_paths,
//     ) = write;
//     let class_sheet = &class_sheet.borrow();
//     let style_mark = &mut style_marks[id];
//     // 设置布局属性， 没有记录每个个属性是否在本地样式表中存在， TODO
//     let rect_layout_style = &mut rect_layout_styles[id];
//     let other_layout_style = &mut other_layout_styles[id];

//     let class = match class_sheet.class_map.get(&class_name) {
//         Some(class) => class,
//         None => return,
//     };

//     let text_style = &mut text_styles[id];

//     style_mark.class_style |= class.class_style_mark;
//     style_mark.class_style1 |= class.class_style_mark1;
//     style_mark.class_style2 |= class.class_style_mark2;

//     set_attr1(
//         id,
//         dirty_list,
//         &class.attrs1,
//         style_mark,
//         text_style,
//         shows,
//         overflows,
//         other_layout_style,
//         obj_fits,
//     );
//     set_attr2(
//         id,
//         dirty_list,
//         &class.attrs2,
//         style_mark,
//         text_style,
//         rect_layout_style,
//         other_layout_style,
//         zindexs,
//         opacitys,
//         blurs,
//         border_image_repeats,
//         images,
//         image_textures,
//         border_images,
//         image_wait_sheet,
//         engine,
//         idtree,
//         // image_clips.get(id),
//         mask_textures,
//         mask_images,
//         blend_modes,
//         border_image_textures,
//         // border_image_clips.get(id),
//     );
//     set_attr3(
//         id,
//         dirty_list,
//         &class.attrs3,
//         style_mark,
//         text_style,
//         border_image_slices,
//         border_image_clips,
//         // border_images,
//         image_clips,
//         // images,
//         image_textures,
//         box_shadows,
//         background_colors,
//         border_colors,
//         border_radiuss,
//         filters,
//         transforms,
//         rect_layout_styles,
//         mask_image_clips,
// 		clip_paths,
//     );
// }

// #[inline]
// fn set_mark(class_sheet: &ClassSheet, name: usize, mark: &mut StyleMark) {
//     match class_sheet.class_map.get(&name) {
//         Some(class) => {
//             mark.class_style |= class.class_style_mark;
//             mark.class_style1 |= class.class_style_mark1;
//             mark.class_style2 |= class.class_style_mark2;
//         }
//         None => (),
//     };
// }

// pub fn set_attr1(
//     id: usize,
//     dirty_list: &mut DirtyList,
//     layout_attrs: &Vec<Attribute1>,
//     style_mark: &mut StyleMark,
//     text_style: &mut TextStyle,
//     shows: &mut MultiCaseImpl<Node, Show>,
//     overflows: &mut MultiCaseImpl<Node, Overflow>,
//     other_style: &mut OtherLayoutStyle,
//     obj_fits: &mut MultiCaseImpl<Node, BackgroundImageMod>,
// ) {
//     for layout_attr in layout_attrs.iter() {
//         match layout_attr {
//             Attribute1::AlignContent(r) => {
//                 if StyleType::AlignContent as usize & style_mark.local_style2 == 0 {
//                     other_style.align_content = *r;
//                 }
//             }
//             Attribute1::AlignItems(r) => {
//                 if StyleType::AlignItems as usize & style_mark.local_style2 == 0 {
//                     other_style.align_items = *r;
//                 }
//             }
//             Attribute1::AlignSelf(r) => {
//                 if StyleType::AlignSelf as usize & style_mark.local_style2 == 0 {
//                     other_style.align_self = *r;
//                 }
//             }
//             Attribute1::JustifyContent(r) => {
//                 if StyleType::JustifyContent as usize & style_mark.local_style2 == 0 {
//                     other_style.justify_content = *r;
//                 }
//             }
//             Attribute1::FlexDirection(r) => {
//                 if StyleType::FlexDirection as usize & style_mark.local_style2 == 0 {
//                     other_style.flex_direction = *r;
//                 }
//             }
//             Attribute1::FlexWrap(r) => {
//                 if StyleType::FlexWrap as usize & style_mark.local_style2 == 0 {
//                     other_style.flex_wrap = *r;
//                 }
//             }
//             Attribute1::PositionType(r) => {
//                 if StyleType::PositionType as usize & style_mark.local_style2 == 0 {
//                     other_style.position_type = *r;
//                 }
//             }

//             Attribute1::ObjectFit(r) => {
//                 if style_mark.local_style == 0 & StyleType::ObjectFit as usize {
//                     if let Some(image_option) = obj_fits.get_mut(id) {
//                         image_option.object_fit = r.clone();
//                     }
//                     set_dirty(dirty_list, id, StyleType::ObjectFit as usize, style_mark);
//                 }
//             }

//             Attribute1::BackgroundRepeat(r) => {
//                 if style_mark.local_style == 0 & StyleType::ObjectFit as usize {
//                     if let Some(image_option) = obj_fits.get_mut(id) {
//                         image_option.repeat = r.clone();
//                     }
//                     set_dirty1(dirty_list, id, StyleType::BackgroundRepeat as usize, style_mark);
//                 }
//             }

//             Attribute1::TextAlign(r) => {
//                 if style_mark.local_style & StyleType::TextAlign as usize == 0 {
//                     text_style.text.text_align = *r;
//                     set_dirty(dirty_list, id, StyleType::TextAlign as usize, style_mark);
//                 }
//             }
//             Attribute1::VerticalAlign(r) => {
//                 if style_mark.local_style & StyleType::VerticalAlign as usize == 0 {
//                     text_style.text.vertical_align = *r;
//                     set_dirty(dirty_list, id, StyleType::VerticalAlign as usize, style_mark);
//                 }
//             }
//             Attribute1::WhiteSpace(r) => {
//                 if style_mark.local_style & StyleType::WhiteSpace as usize == 0 {
//                     text_style.text.white_space = *r;
//                     set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
//                 }
//             }
//             Attribute1::FontStyle(r) => {
//                 if style_mark.local_style & StyleType::FontStyle as usize == 0 {
//                     text_style.font.style = *r;
//                     set_dirty(dirty_list, id, StyleType::WhiteSpace as usize, style_mark);
//                 }
//             }
//             Attribute1::Enable(r) => {
//                 if style_mark.local_style1 & StyleType::Enable as usize == 0 {
//                     unsafe { shows.get_unchecked_write(id) }.modify(|show: &mut Show| {
//                         show.set_enable(*r);
//                         true
//                     });
//                 }
//             }
//             Attribute1::Display(r) => {
//                 if style_mark.local_style1 & StyleType::Display as usize == 0 {
//                     other_style.display = *r;
//                     // layout_style.set_display(unsafe { transmute(*r) });
//                     unsafe { shows.get_unchecked_write(id) }.modify(|show: &mut Show| {
//                         show.set_display(*r);
//                         true
//                     });
//                 }
//             }
//             Attribute1::Visibility(r) => {
//                 if style_mark.local_style1 & StyleType::Visibility as usize == 0 {
//                     unsafe { shows.get_unchecked_write(id) }.modify(|show: &mut Show| {
//                         show.set_visibility(*r);
//                         true
//                     });
//                 }
//             }
//             Attribute1::Overflow(r) => {
//                 if style_mark.local_style1 & StyleType::Overflow as usize == 0 {
//                     unsafe { overflows.get_unchecked_write(id) }.modify(|overflow: &mut Overflow| {
//                         overflow.0 = *r;
//                         true
//                     });
//                 }
//             },
//         }
//     }
// }

// pub fn set_attr2<C: HalContext>(
//     id: usize,
//     dirty_list: &mut DirtyList,
//     layout_attrs: &Vec<Attribute2>,
//     style_mark: &mut StyleMark,
//     text_style: &mut TextStyle,
//     rect_layout_style: &mut RectLayoutStyle,
//     other_layout_style: &mut OtherLayoutStyle,
//     zindexs: &mut MultiCaseImpl<Node, ZIndex>,
//     opacitys: &mut MultiCaseImpl<Node, Opacity>,
//     blurs: &mut MultiCaseImpl<Node, Blur>,
//     border_image_repeats: &mut MultiCaseImpl<Node, BorderImageRepeat>,
//     images: &mut MultiCaseImpl<Node, BackgroundImage>,
//     image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
//     border_images: &mut MultiCaseImpl<Node, BorderImage>,
//     image_wait_sheet: &mut SingleCaseImpl<ImageWaitSheet>,
//     engine: &mut Engine<C>,
//     idtree: &SingleCaseImpl<IdTree>,
//     // image_clip: Option<&ImageClip>,
//     mask_textures: &mut MultiCaseImpl<Node, MaskTexture>,
//     mask_images: &mut MultiCaseImpl<Node, MaskImage>,
//     blend_modes: &mut MultiCaseImpl<Node, BlendMode>,
//     // border_image_clip: Option<&BorderImageClip>,
//     border_image_textures: &mut MultiCaseImpl<Node, BorderImageTexture>,
// ) {
//     for layout_attr in layout_attrs.iter() {
//         match layout_attr {
//             Attribute2::LetterSpacing(r) => {
//                 if style_mark.local_style & StyleType::LetterSpacing as usize == 0 && text_style.text.letter_spacing != *r {
//                     text_style.text.letter_spacing = *r;
//                     set_dirty(dirty_list, id, StyleType::LetterSpacing as usize, style_mark);
//                 }
//             }
//             Attribute2::LineHeight(r) => {
//                 if style_mark.local_style & StyleType::LineHeight as usize == 0 {
//                     text_style.text.line_height = *r;
//                     set_dirty(dirty_list, id, StyleType::LineHeight as usize, style_mark);
//                 }
//             }
//             Attribute2::TextIndent(r) => {
//                 if style_mark.local_style & StyleType::Indent as usize == 0 && text_style.text.indent != *r {
//                     text_style.text.indent = *r;
//                     set_dirty(dirty_list, id, StyleType::Indent as usize, style_mark);
//                 }
//             }
//             Attribute2::WordSpacing(r) => {
//                 if style_mark.local_style & StyleType::WordSpacing as usize == 0 {
//                     text_style.text.word_spacing = *r;
//                     set_dirty(dirty_list, id, StyleType::WordSpacing as usize, style_mark);
//                 }
//             }
//             Attribute2::FontWeight(r) => {
//                 if style_mark.local_style & StyleType::FontWeight as usize == 0 {
//                     text_style.font.weight = *r as usize;
//                     set_dirty(dirty_list, id, StyleType::FontWeight as usize, style_mark);
//                 }
//             }
//             Attribute2::FontSize(r) => {
//                 if style_mark.local_style & StyleType::FontSize as usize == 0 {
//                     text_style.font.size = *r;
//                     set_dirty(dirty_list, id, StyleType::FontSize as usize, style_mark);
//                 }
//             }
//             Attribute2::FontFamily(r) => {
//                 if style_mark.local_style & StyleType::FontFamily as usize == 0 {
//                     text_style.font.family = *r;
//                     set_dirty(dirty_list, id, StyleType::FontFamily as usize, style_mark);
//                 }
//             }
//             Attribute2::ZIndex(r) => {
//                 if style_mark.local_style1 & StyleType::ZIndex as usize == 0 {
//                     zindexs.insert_no_notify(id, ZIndex(*r));
//                     zindexs.get_notify_ref().modify_event(id, "class", 0);
//                 }
//             }
//             Attribute2::Opacity(r) => {
//                 if style_mark.local_style & StyleType::Opacity as usize == 0 {
//                     opacitys.insert_no_notify(id, r.clone());
//                     opacitys.get_notify_ref().modify_event(id, "class", 0);
//                     // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
//                 }
//             }
//             Attribute2::Blur(r) => {
//                 if style_mark.local_style1 & StyleType::Blur as usize == 0 {
//                     blurs.insert_no_notify(id, r.clone());
//                     blurs.get_notify_ref().modify_event(id, "class", 0);
//                     // set_dirty(dirty_list, event.id, StyleType::Opacity as usize, style_mark); 不需要设脏，opacity还需要通过级联计算得到最终值， 监听到该值的变化才会设脏
//                 }
//             }
//             Attribute2::BlendMode(r) => {
//                 if style_mark.local_style2 & StyleType::BlendMode as usize == 0 {
//                     blend_modes.insert_no_notify(id, r.clone());
//                 }
//             }
//             Attribute2::BorderImageRepeat(r) => {
//                 if style_mark.local_style & StyleType::BorderImageRepeat as usize == 0 {
//                     border_image_repeats.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BorderImageRepeat as usize, style_mark);
//                 }
//             }

//             Attribute2::ImageUrl(r) => {
//                 if style_mark.local_style & StyleType::Image as usize == 0 {
//                     let mut image = BackgroundImage { url: r.clone() };
//                     if let Some(n) = idtree.get(id) {
//                         if n.layer() > 0 {
//                             set_image(id, engine, image_wait_sheet, &mut image, image_textures, ImageType::ImageClass);
//                         }
//                     }
//                     images.insert_no_notify(id, image);
//                 }
//             }
//             Attribute2::MaskImage(r) => {
//                 if style_mark.local_style1 & StyleType::MaskImage as usize == 0 {
//                     let mut mask_image = r.clone();

//                     if let Some(n) = idtree.get(id) {
//                         if n.layer() > 0 {
//                             set_mask_image(id, engine, image_wait_sheet, &mut mask_image, mask_textures, ImageType::MaskImageClass);
//                         }
//                     }
//                     mask_images.insert_no_notify(id, mask_image);
//                 }
//             }
//             Attribute2::BorderImageUrl(r) => {
//                 if style_mark.local_style & StyleType::BorderImage as usize == 0 {
//                     let mut image = BorderImage { url: r.clone() };
//                     if let Some(n) = idtree.get(id) {
//                         if n.layer() > 0 {
//                             set_border_image(
//                                 id,
//                                 engine,
//                                 image_wait_sheet,
//                                 &mut image,
//                                 border_image_textures,
//                                 ImageType::BorderImageClass,
//                             );
//                         }

//                         // if
//                         // set_image(
//                         //     id,
//                         //     StyleType::BorderImage,
//                         //     engine,
//                         //     image_wait_sheet,
//                         //     dirty_list,
//                         //     &mut image.0,
//                         //     style_mark,
//                         //     ImageType::BorderImageClass,
//                         // );
//                         //  {
//                         //     set_border_image_size(
//                         //         image.0.src.as_ref().unwrap(),
//                         //         layout_style,
//                         //         border_image_clip,
//                         //         style_mark,
//                         //     );
//                         // }
//                     }
//                     border_images.insert_no_notify(id, image);
//                 }
//             }

//             Attribute2::Width(r) => {
//                 if StyleType::Width as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.size.width = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::Width as usize, style_mark);
//                 }
//             }
//             Attribute2::Height(r) => {
//                 if StyleType::Height as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.size.height = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::Height as usize, style_mark);
//                 }
//             }
//             Attribute2::MarginLeft(r) => {
//                 if StyleType::MarginLeft as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginLeft as usize, style_mark);
//                 }
//             }
//             Attribute2::MarginTop(r) => {
//                 if StyleType::MarginTop as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginTop as usize, style_mark);
//                 }
//             }
//             Attribute2::MarginBottom(r) => {
//                 if StyleType::MarginBottom as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginBottom as usize, style_mark);
//                 }
//             }
//             Attribute2::MarginRight(r) => {
//                 if StyleType::MarginRight as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.end = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginRight as usize, style_mark);
//                 }
//             }
//             Attribute2::Margin(r) => {
//                 if StyleType::MarginLeft as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginLeft as usize, style_mark);
//                 }
//                 if StyleType::MarginTop as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginTop as usize, style_mark);
//                 }
//                 if StyleType::MarginBottom as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginBottom as usize, style_mark);
//                 }
//                 if StyleType::MarginRight as usize & style_mark.local_style2 == 0 {
//                     rect_layout_style.margin.end = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MarginRight as usize, style_mark);
//                 }
//             }
//             Attribute2::PaddingLeft(r) => {
//                 if StyleType::PaddingLeft as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingLeft as usize, style_mark);
//                 }
//             }
//             Attribute2::PaddingTop(r) => {
//                 if StyleType::PaddingTop as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingTop as usize, style_mark);
//                 }
//             }
//             Attribute2::PaddingBottom(r) => {
//                 if StyleType::PaddingBottom as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingBottom as usize, style_mark);
//                 }
//             }
//             Attribute2::PaddingRight(r) => {
//                 if StyleType::PaddingRight as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.end = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingRight as usize, style_mark);
//                 }
//             }
//             Attribute2::Padding(r) => {
//                 if StyleType::PaddingLeft as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingLeft as usize, style_mark);
//                 }
//                 if StyleType::PaddingTop as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingTop as usize, style_mark);
//                 }
//                 if StyleType::PaddingBottom as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingBottom as usize, style_mark);
//                 }
//                 if StyleType::PaddingRight as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.padding.end = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PaddingRight as usize, style_mark);
//                 }
//             }
//             Attribute2::BorderLeft(r) => {
//                 if StyleType::BorderLeft as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderLeft as usize, style_mark);
//                 }
//             }
//             Attribute2::BorderTop(r) => {
//                 if StyleType::BorderTop as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderTop as usize, style_mark);
//                 }
//             }
//             Attribute2::BorderBottom(r) => {
//                 if StyleType::BorderBottom as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderBottom as usize, style_mark);
//                 }
//             }
//             Attribute2::BorderRight(r) => {
//                 if StyleType::BorderRight as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.end = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderRight as usize, style_mark);
//                 }
//             }
//             Attribute2::Border(r) => {
//                 if StyleType::BorderLeft as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderLeft as usize, style_mark);
//                 }
//                 if StyleType::BorderTop as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderTop as usize, style_mark);
//                 }
//                 if StyleType::BorderBottom as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderBottom as usize, style_mark);
//                 }
//                 if StyleType::BorderBottom as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.border.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::BorderBottom as usize, style_mark);
//                 }
//             }
//             Attribute2::PositionLeft(r) => {
//                 if StyleType::PositionLeft as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.position.start = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PositionLeft as usize, style_mark);
//                 }
//             }
//             Attribute2::PositionTop(r) => {
//                 if StyleType::PositionTop as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.position.top = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PositionTop as usize, style_mark);
//                 }
//             }
//             Attribute2::PositionRight(r) => {
//                 if StyleType::PositionRight as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.position.end = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PositionRight as usize, style_mark);
//                 }
//             }
//             Attribute2::PositionBottom(r) => {
//                 if StyleType::PositionBottom as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.position.bottom = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::PositionBottom as usize, style_mark);
//                 }
//             }
//             Attribute2::MinWidth(r) => {
//                 if StyleType::MinWidth as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.min_size.width = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MinWidth as usize, style_mark);
//                 }
//             }
//             Attribute2::MinHeight(r) => {
//                 if StyleType::MinHeight as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.min_size.height = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MinHeight as usize, style_mark);
//                 }
//             }
//             Attribute2::MaxHeight(r) => {
//                 if StyleType::MaxHeight as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.max_size.height = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MaxHeight as usize, style_mark);
//                 }
//             }
//             Attribute2::MaxWidth(r) => {
//                 if StyleType::MaxWidth as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.max_size.width = r.clone();
//                     set_dirty2(dirty_list, id, StyleType::MaxWidth as usize, style_mark);
//                 }
//             }
//             Attribute2::FlexBasis(r) => {
//                 if StyleType::FlexBasis as usize & style_mark.local_style1 == 0 {
//                     other_layout_style.flex_basis = r.clone();
//                     set_dirty1(dirty_list, id, StyleType::FlexBasis as usize, style_mark);
//                 }
//             }
//             Attribute2::FlexShrink(r) => {
//                 if StyleType::FlexShrink as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.flex_shrink = *r;
//                     set_dirty2(dirty_list, id, StyleType::FlexShrink as usize, style_mark);
//                 }
//             }
//             Attribute2::FlexGrow(r) => {
//                 if StyleType::FlexGrow as usize & style_mark.local_style2 == 0 {
//                     other_layout_style.flex_grow = *r;
//                     set_dirty2(dirty_list, id, StyleType::FlexGrow as usize, style_mark);
//                 }
//             }
//         }
//     }
// }

// pub fn set_attr3(
//     id: usize,
//     dirty_list: &mut DirtyList,
//     attrs: &Vec<Attribute3>,
//     style_mark: &mut StyleMark,
//     text_style: &mut TextStyle,
//     border_image_slices: &mut MultiCaseImpl<Node, BorderImageSlice>,
//     border_image_clips: &mut MultiCaseImpl<Node, BorderImageClip>,
//     // border_images: &mut MultiCaseImpl<Node, BorderImage>,
//     image_clips: &mut MultiCaseImpl<Node, BackgroundImageClip>,
//     // images: &mut MultiCaseImpl<Node, Image>,
//     image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
//     box_shadows: &mut MultiCaseImpl<Node, BoxShadow>,
//     background_colors: &mut MultiCaseImpl<Node, BackgroundColor>,
//     border_colors: &mut MultiCaseImpl<Node, BorderColor>,
//     border_radiuss: &mut MultiCaseImpl<Node, BorderRadius>,
//     filters: &mut MultiCaseImpl<Node, Hsi>,
//     transforms: &mut MultiCaseImpl<Node, Transform>,
//     rect_layout_styles: &mut MultiCaseImpl<Node, RectLayoutStyle>,
//     mask_image_clips: &mut MultiCaseImpl<Node, MaskImageClip>,
// 	clip_path: &mut MultiCaseImpl<Node, ClipPath>,
// ) {
//     for attr in attrs.iter() {
//         match attr {
// 			Attribute3::ClipPath(r) => {
//                 if style_mark.local_style1 & StyleType::ClipPath as usize == 0 {
//                     clip_path.insert_no_notify(id, r.clone());
//                     set_dirty1(dirty_list, id, StyleType::ClipPath as usize, style_mark);
//                 }
//             }
//             Attribute3::BGColor(r) => {
//                 if style_mark.local_style & StyleType::BackgroundColor as usize == 0 {
//                     background_colors.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BackgroundColor as usize, style_mark);
//                 }
//             }
//             Attribute3::BorderColor(r) => {
//                 if style_mark.local_style & StyleType::BorderColor as usize == 0 {
//                     border_colors.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BorderColor as usize, style_mark);
//                 }
//             }
//             Attribute3::BoxShadow(r) => {
//                 if style_mark.local_style & StyleType::BoxShadow as usize == 0 {
//                     box_shadows.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BoxShadow as usize, style_mark);
//                 }
//             }

//             Attribute3::ImageClip(r) => {
//                 if style_mark.local_style & StyleType::ImageClip as usize == 0 {
//                     set_dirty(dirty_list, id, StyleType::ImageClip as usize, style_mark);
//                     if let Some(teture) = image_textures.get(id) {
//                         if let ImageTexture::All(src, _) = teture {
//                             set_image_size(src, &mut rect_layout_styles[id], Some(r), style_mark);
//                         }
//                     }
//                     image_clips.insert_no_notify(id, r.clone());
//                 }
//             }
//             Attribute3::MaskImageClip(r) => {
//                 if style_mark.local_style1 & StyleType::MaskImageClip as usize == 0 {
//                     set_dirty(dirty_list, id, StyleType::MaskImageClip as usize, style_mark);
//                     mask_image_clips.insert_no_notify(id, r.clone());
//                 }
//             }

//             Attribute3::BorderImageClip(r) => {
//                 if style_mark.local_style & StyleType::BorderImageClip as usize == 0 {
//                     border_image_clips.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BorderImageClip as usize, style_mark);
//                 }
//             }
//             Attribute3::BorderImageSlice(r) => {
//                 if style_mark.local_style & StyleType::BorderImageSlice as usize == 0 {
//                     border_image_slices.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BorderImageSlice as usize, style_mark);
//                 }
//             }

//             Attribute3::Color(r) => {
//                 if style_mark.local_style & StyleType::Color as usize == 0 {
//                     text_style.text.color = r.clone();
//                     set_dirty(dirty_list, id, StyleType::Color as usize, style_mark);
//                 }
//             }
//             Attribute3::TextShadow(r) => {
//                 if style_mark.local_style & StyleType::TextShadow as usize == 0 {
//                     text_style.shadow = r.clone();
//                     set_dirty(dirty_list, id, StyleType::TextShadow as usize, style_mark);
//                 }
//             }
//             Attribute3::TextStroke(r) => {
//                 if style_mark.local_style & StyleType::Stroke as usize == 0 {
//                     text_style.text.stroke = r.clone();
//                     set_dirty(dirty_list, id, StyleType::Stroke as usize, style_mark);
//                 }
//             }

//             Attribute3::BorderRadius(r) => {
//                 if style_mark.local_style & StyleType::BorderRadius as usize == 0 {
//                     border_radiuss.insert_no_notify(id, r.clone());
//                     set_dirty(dirty_list, id, StyleType::BorderRadius as usize, style_mark);
//                 }
//             }
//             Attribute3::TransformFunc(r) => {
//                 if style_mark.local_style1 & StyleType::Transform as usize == 0 {
//                     match transforms.get_mut(id) {
//                         Some(t) => t.funcs = r.clone(),
//                         None => {
//                             transforms.insert_no_notify(
//                                 id,
//                                 Transform {
//                                     funcs: r.clone(),
//                                     origin: TransformOrigin::Center,
//                                 },
//                             );
//                         }
//                     };
//                     transforms.get_notify_ref().modify_event(id, "class", 0);
//                 }
//             }
//             Attribute3::TransformOrigin(r) => {
//                 if style_mark.local_style1 & StyleType::TransformOrigin as usize == 0 {
//                     match transforms.get_mut(id) {
//                         Some(t) => t.origin = r.clone(),
//                         None => {
//                             transforms.insert_no_notify(
//                                 id,
//                                 Transform {
//                                     funcs: Vec::default(),
//                                     origin: r.clone(),
//                                 },
//                             );
//                         }
//                     };
//                     transforms.get_notify_ref().modify_event(id, "class", 0);
//                 }
//             }
//             Attribute3::Filter(r) => {
//                 if style_mark.local_style & StyleType::Filter as usize == 0 {
//                     filters.insert_no_notify(id, r.clone());
//                     filters.get_notify_ref().modify_event(id, "class", 0);
//                     set_dirty(dirty_list, id, StyleType::Filter as usize, style_mark);
//                 }
//             }
//         }
//     }
// }

// // fn set_image<C: HalContext>(
// //     id: usize,
// //     ty: StyleType,
// //     engine: &mut Engine<C>,
// //     image_wait_sheet: &mut ImageWaitSheet,
// //     dirty_list: &mut DirtyList,
// //     image: &mut Image,
// // 	image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
// //     style_mark: &mut StyleMark,
// //     wait_ty: ImageType,
// // ) -> bool {
// //     if image.src.is_none() {
// //         match engine.texture_res_map.get(&image.url) {
// //             Some(r) => {
// //                 image.src = Some(r);
// //                 set_dirty(dirty_list, id, ty as usize, style_mark);
// //                 return true;
// //             }
// //             None => {
// //                 image_wait_sheet.add(
// //                     image.url,
// //                     ImageWait {
// //                         id: id,
// //                         ty: wait_ty,
// //                     },
// //                 );
// //                 return false;
// //             }
// //         }
// //     } else {
// //         set_dirty(dirty_list, id, ty as usize, style_mark);
// //         return true;
// //     }ImageTextureWrite
// // }


// 节点被添加到树上， 加载图片
fn load_image<'a, C: HalContext>(id: usize, write: &mut ImageTextureWrite<'a, C>) {
    if let Some(image) = write.0.get_mut(id) {
        // let style_mark = &mut write.2[id];
        set_image(id, &mut *write.4, &mut *write.5, image, &mut write.12, ImageType::Image);
        // if style_mark.local_style & StyleType::Image as usize != 0 {
        // 	set_image(
        //         id,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         image,
        // 		&mut write.12,
        //         style_mark,
        //         ImageType::ImageLocal,
        //     );
        //     // if  {
        //     //     set_image_size(
        //     //         image.src.as_ref().unwrap(),
        //     //         &mut write.6[id],
        //     //         write.7.get(id),
        //     //         style_mark,
        //     //     );
        //     // }
        // } else {
        //     if set_image(
        //         id,
        //         StyleType::Image,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         image,
        //         style_mark,
        //         ImageType::ImageClass,
        //     ) {
        //         set_image_size(
        //             image.src.as_ref().unwrap(),
        //             &mut write.6[id],
        //             write.7.get(id),
        //             style_mark,
        //         );
        //     }
        // }
    }
    if let Some(image) = write.1.get_mut(id) {
        // let style_mark = &mut write.2[id];
        set_border_image(id, &mut *write.4, &mut *write.5, image, &mut *write.13, ImageType::BorderImage);
        // if style_mark.local_style & StyleType::BorderImage as usize != 0 {
        //     // if
        //     set_border_image(
        //         id,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         &mut image.0,
        // 		&mut *write.3,
        //         style_mark,
        //         ImageType::BorderImageLocal,
        //     );
        // } else {
        //     // if
        //     set_image(
        //         id,
        //         StyleType::BorderImage,
        //         &mut *write.4,
        //         &mut *write.5,
        //         &mut *write.3,
        //         &mut image.0,
        //         style_mark,
        //         ImageType::BorderImageClass,
        //     );
        // }
    }

    if let Some(image) = write.9.get_mut(id) {
        // let style_mark = &mut write.2[id];
        set_mask_image(id, &mut *write.4, &mut *write.5, image, &mut *write.11, ImageType::MaskImage);
    }
}

// 从树上删除节点， 删除节点对图片资源的引用
fn release_image<'a, C: HalContext>(id: usize, write: &mut ImageTextureWrite<'a, C>) {
    if let Some(_r) = write.11.get(id) {
        write.11.delete(id);
    }
    if let Some(_r) = write.12.get(id) {
        write.12.delete(id);
    }
    if let Some(_r) = write.13.get(id) {
        write.13.delete(id);
    }

    // if let Some(image) = write.12.get_mut(id) {
    //     let style_mark = &mut write.2[id];
    //     if image.src.is_some() {
    //         image.src = None;
    //         set_dirty(&mut *write.3, id, StyleType::Image as usize, style_mark);
    //     }
    // }
    // if let Some(image) = write.1.get_mut(id) {
    //     let style_mark = &mut write.2[id];
    //     if image.0.src.is_some() {
    //         image.0.src = None;
    //         set_dirty(
    //             &mut *write.3,
    //             id,
    //             StyleType::BorderImage as usize,
    //             style_mark,
    //         );
    //     }
    // }
}


fn set_image<C: HalContext + 'static>(
    id: usize,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    image: &mut BackgroundImage,
    image_textures: &mut MultiCaseImpl<Node, ImageTexture>,
    wait_ty: ImageType,
) {
    match engine.texture_res_map.get(&image.0.get_hash()) {
        Some(texture) => {
            image_textures.insert(id, ImageTexture::All(texture, image.0.clone()));
        }
        None => {
            image_wait_sheet.add(image.0.clone(), ImageWait { id, ty: wait_ty });
        }
    }
}

fn set_border_image<C: HalContext + 'static>(
    id: usize,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    image: &mut BorderImage,
    image_textures: &mut MultiCaseImpl<Node, BorderImageTexture>,
    wait_ty: ImageType,
) {
    match engine.texture_res_map.get(&image.0.get_hash()) {
        Some(texture) => {
            image_textures.insert(id, BorderImageTexture(texture));
        }
        None => {
            image_wait_sheet.add(image.0.clone(), ImageWait { id, ty: wait_ty });
        }
    }
}

// // fn set_image_local<'a, C: HalContext>(
// //     id: usize,
// //     idtree: &SingleCaseImpl<IdTree>,
// //     write: ImageWrite<'a, C>,
// // ) {
// //     let style_mark = &mut write.0[id];
// //     style_mark.local_style |= StyleType::Image as usize;
// // 	set_dirty(&mut *write.1, id, StyleType::Image as usize, style_mark);

// //     if let Some(_) = idtree.get(id) {
// //         let image = &mut write.4[id];
// //         if set_image(
// //             id,
// //             StyleType::Image,
// //             &mut *write.2,
// //             &mut *write.3,
// //             &mut *write.1,
// //             image,
// //             style_mark,
// //             ImageType::ImageLocal,
// //         ) {
// //             set_image_size(
// //                 image.src.as_ref().unwrap(),
// //                 &mut write.5[id],
// //                 write.6.get(id),
// //                 style_mark,
// //             );
// //         }
// //     }
// // }


// // fn set_border_image_local<'a, C: HalContext>(
// //     id: usize,
// //     idtree: &SingleCaseImpl<IdTree>,
// //     write: BorderImageWrite<'a, C>,
// // ) {
// //     let style_mark = &mut write.0[id];
// //     style_mark.local_style |= StyleType::BorderImage as usize;

// //     if let Some(_) = idtree.get(id) {
// //         let image = &mut write.4[id].0;
// //         // if
// //         set_image(
// //             id,
// //             StyleType::BorderImage,
// //             &mut *write.2,
// //             &mut *write.3,
// //             &mut *write.1,
// //             image,
// //             style_mark,
// //             ImageType::BorderImageLocal,
// //         );
// //     }
// // }

// fn set_mask_image_local<'a, C: HalContext>(
//     id: usize,
//     idtree: &SingleCaseImpl<IdTree>,
//     (style_marks, _dirty_list, engine, wait_sheet, mask_images, _layout, _mask_image_clips, mask_texture): MaskImageWrite<'a, C>,
// ) {
//     let style_mark = &mut style_marks[id];
//     style_mark.local_style1 |= StyleType::MaskImage as usize;

//     if let Some(_) = idtree.get(id) {
//         let image = &mut mask_images[id];
//         set_mask_image(id, engine, wait_sheet, image, mask_texture, ImageType::MaskImageLocal);
//     }
// }

fn set_mask_image<C: HalContext>(
    id: usize,
    engine: &mut Engine<C>,
    image_wait_sheet: &mut ImageWaitSheet,
    image: &mut MaskImage,
    texure: &mut MultiCaseImpl<Node, MaskTexture>,
    wait_ty: ImageType,
) {
    if let pi_style::style::MaskImage::Path(url) = &image.0 {
        if texure.get(id).is_none() {
            match engine.texture_res_map.get(&url.get_hash()) {
                Some(r) => {
                    texure.insert(id, MaskTexture::All(r.clone()));
                }
                None => {
                    image_wait_sheet.add(url.clone(), ImageWait { id: id, ty: wait_ty });
                }
            }
        }
    }
}

impl_system! {
    StyleMarkSys<C> where [C: HalContext + 'static],
    true,
    {
        EntityListener<Node, CreateEvent>
        EntityListener<Node, ModifyEvent>
        // MultiCaseListener<Node, TextStyle, ModifyEvent>
        // MultiCaseListener<Node, RectLayoutStyle, ModifyEvent>
        // MultiCaseListener<Node, OtherLayoutStyle, ModifyEvent>

        MultiCaseListener<Node, TextContent, CreateEvent>
        // MultiCaseListener<Node, TextContent, ModifyEvent>

        // MultiCaseListener<Node, BlendMode, (CreateEvent, ModifyEvent)>

        // MultiCaseListener<Node, MaskImage, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, MaskImage, DeleteEvent>
        // MultiCaseListener<Node, MaskImageClip, (CreateEvent, ModifyEvent)>

        MultiCaseListener<Node, MaskTexture, (CreateEvent, ModifyEvent, DeleteEvent)>
        MultiCaseListener<Node, ImageTexture, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, ImageTexture, DeleteEvent>
        MultiCaseListener<Node, BorderImageTexture, (CreateEvent, ModifyEvent, DeleteEvent)>

        MultiCaseListener<Node, BackgroundImage, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BackgroundImage, DeleteEvent>
        MultiCaseListener<Node, BackgroundImageClip, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, BackgroundImageMod, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderImage, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, BorderImage, DeleteEvent>
        // MultiCaseListener<Node, BorderImageClip, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, BorderImageSlice, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, BorderImageRepeat, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, BorderColor, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, BackgroundColor, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, BoxShadow, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, ZIndex, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, Transform, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, TransformWillChange, (CreateEvent, ModifyEvent)>
        // MultiCaseListener<Node, Overflow, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, ContentBox, (CreateEvent, ModifyEvent)>
		// MultiCaseListener<Node, ClipPath, (CreateEvent, ModifyEvent)>

        MultiCaseListener<Node, WorldMatrix, ModifyEvent>
        MultiCaseListener<Node, ZDepth, ModifyEvent>
        // MultiCaseListener<Node, Opacity, ModifyEvent>
        // MultiCaseListener<Node, Blur, (CreateEvent, ModifyEvent)>
        MultiCaseListener<Node, LayoutR, ModifyEvent>
        // MultiCaseListener<Node, BorderRadius, ModifyEvent>
        // MultiCaseListener<Node, Hsi, ModifyEvent>
        MultiCaseListener<Node, ByOverflow, ModifyEvent>
        // MultiCaseListener<Node, Visibility, ModifyEvent>
        SingleCaseListener<Oct, ModifyEvent>
        SingleCaseListener<Oct, CreateEvent>
        // SingleCaseListener<Oct, DeleteEvent>

        MultiCaseListener<Node, COpacity, ModifyEvent>

        // MultiCaseListener<Node, Show, ModifyEvent>

        SingleCaseListener<ImageWaitSheet, ModifyEvent>
        SingleCaseListener<RenderObjs, CreateEvent>
        SingleCaseListener<IdTree, CreateEvent>
        SingleCaseListener<IdTree, DeleteEvent>
    }
}

// // reset_value_del!(display, other_layout_style, display, "1", Display);
// // reset_value_del!(visibility, , visibility, "1", Visibility);
// // reset_value_del!(enable, "1", Enable);
// reset_value_del!(z_index, "1", ZIndex);
// reset_value_del!(transform, "1", Transform);
// reset_value_del!(transform_will_change, "1", TransformWillChange);
// reset_value_del!(overflow, "1", Overflow);

impl_system! {
    ClassSetting<C> where [C: HalContext + 'static],
    true,
    {
        MultiCaseListener<Node, ClassName, ModifyEvent>
    }
}
