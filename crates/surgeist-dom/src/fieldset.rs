//! Native Fieldset state in the canonical owner, independent of current markup.
use crate::{DocumentString, NodeId};
use surgeist_html::HtmlNativeFormAssociationKind;

/// Actual native Fieldset association and custom-error state.
///
/// This is stored state, not a fresh form-owner query or a complete validation
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
    pub fn form_owner(&self) -> Option<NodeId> {
        self.control.owner
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
    /// The stored normalized custom error, independently of candidacy or UI.
    pub fn custom_validity_message(&self) -> &DocumentString {
        &self.control.message
    }
}
