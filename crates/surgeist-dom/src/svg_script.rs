//! Native SVG Tiny processing state, separate from HTML preparation.
use crate::{DomNode, NodeId};
use surgeist_html::SvgScriptFlags;

/// Actual native creation cause; neither variant claims parser provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSvgScriptOrigin {
    Native,
    Clone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SvgScriptState {
    pub(crate) flags: SvgScriptFlags,
    pub(crate) origin: DomSvgScriptOrigin,
}
impl SvgScriptState {
    pub(crate) const fn fresh() -> Self {
        Self {
            flags: SvgScriptFlags::fresh(),
            origin: DomSvgScriptOrigin::Native,
        }
    }
    pub(crate) const fn clone_for_element(self) -> Self {
        Self {
            flags: self.flags.clone_for_element(),
            origin: DomSvgScriptOrigin::Clone,
        }
    }
}

/// Checked SVG Script state in one canonical immutable revision.
/// This readback grants no acquisition, execution or load/error delivery.
#[derive(Clone, Copy, Debug)]
pub struct DomSvgScript<'a> {
    node: DomNode<'a>,
}
impl DomSvgScript<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn flags(self) -> SvgScriptFlags {
        self.state().flags
    }
    pub fn origin(self) -> DomSvgScriptOrigin {
        self.state().origin
    }
    fn state(self) -> SvgScriptState {
        *self
            .node
            .as_element()
            .expect("checked SVG Script")
            .svg_script_state()
            .expect("checked SVG Script")
    }
}
impl<'a> DomNode<'a> {
    pub fn as_svg_script(self) -> Option<DomSvgScript<'a>> {
        self.as_element()?.svg_script_state()?;
        Some(DomSvgScript { node: self })
    }
}
