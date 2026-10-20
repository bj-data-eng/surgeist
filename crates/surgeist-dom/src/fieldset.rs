//! Native Fieldset state in the canonical owner, independent of current markup.
use crate::{DocumentString, DomNode, NodeId};
use surgeist_html::HtmlNativeFormAssociationKind;

/// Actual native Fieldset parser flag and custom-error state.
///
/// Owner identity belongs to the checked node view. This is not a complete validation
/// interface. Native clone/import initializes it afresh; adoption preserves it
/// subject to the source removal/reset steps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomFieldsetState {
    pub(crate) control: crate::control::NativeControlState,
}
impl DomFieldsetState {
    pub(crate) fn fresh() -> Self {
        Self {
            control: crate::control::NativeControlState::fresh(),
        }
    }
    pub fn kind(&self) -> HtmlNativeFormAssociationKind {
        HtmlNativeFormAssociationKind::Fieldset
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
    /// The stored normalized custom error, independently of candidacy or UI.
    pub fn custom_validity_message(&self) -> &DocumentString {
        &self.control.message
    }
}

/// Checked Fieldset view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomFieldset<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomFieldset<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a DomFieldsetState {
        self.node
            .as_element()
            .expect("checked Fieldset")
            .fieldset_state()
            .expect("checked Fieldset")
    }
    /// Actual established relation, independently of current content attributes.
    pub fn form_owner(self) -> Result<Option<NodeId>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner())
    }
}
