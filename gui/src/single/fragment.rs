
use pi_hash::XHashMap;
use pi_null::Null;
use pi_style::{
    style_parse::{style_list_to_buffer, Attribute},
    style_type::ClassMeta,
};
use smallvec::SmallVec;
use std::{collections::VecDeque, ops::Range, convert::TryFrom};

use crate::component::user::ClassName;

/// 模板map
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FragmentMap {
    pub style_buffer: Vec<u8>,
    pub fragments: Vec<NodeFragmentCmd>,
    pub map: XHashMap<u32, Range<usize>>,
}

impl FragmentMap {
    pub fn extend(&mut self, value: Fragments) {
        if value.fragments.len() + self.fragments.len() < value.fragments.capacity() {
            self.fragments
                .reserve(value.fragments.capacity() - (value.fragments.len() + self.fragments.len()));
        }
        let fragments = &mut self.fragments;
        let index = fragments.len();
        let style_buffer = &mut self.style_buffer;
        for mut node in value.fragments.into_iter() {
            let count = node.style.len();
            let meta = style_list_to_buffer(style_buffer, &mut node.style, count);
            fragments.push(NodeFragmentCmd {
                tag: node.tag,
                parent: if node.parent.is_null() { usize::null() } else { node.parent as usize },
                style_meta: meta,
                class: ClassName(node.class.into_iter().map(|r| r as usize).collect::<SmallVec<[usize; 1]>>()),
            });
        }

        self.map.extend(value.map.into_iter().map(|(k, v)| (k, v.start + index..v.end + index)));
    }
}

/// 每节点的模板指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFragmentCmd {
    pub tag: NodeTag,
    pub parent: usize, // 在Vec<NodeFragmentCmd>中的索引
    pub style_meta: ClassMeta,
    pub class: ClassName,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum NodeTag {
    #[default]
    Div,
    Image,
    Span,
    Canvas,
    VNode,
}

#[derive(Debug)]
pub enum TagErr {
    InvaildName(String),
}

impl TryFrom<&str> for NodeTag {
    type Error = TagErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let r = match value {
            "div" => NodeTag::Div,
            "canvas" => NodeTag::Canvas,
            "span" => NodeTag::Span,
            "image" => NodeTag::Image,
            "template" => NodeTag::VNode,
            _ => return Err(TagErr::InvaildName(value.to_string())),
        };
        Ok(r)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragments {
    pub fragments: Vec<NodeFragment>,
    pub map: XHashMap<u32, Range<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeFragment {
    pub tag: NodeTag,
    pub parent: u32,
    pub style: VecDeque<Attribute>,
    pub class: SmallVec<[u32; 1]>,
}
