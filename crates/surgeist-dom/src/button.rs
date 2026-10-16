//! Actual native Button payload and its canonical weak-target view.
use crate::reflected::ReflectedTarget;
use crate::{DocumentString, DomNode, NodeId};
use surgeist_html::HtmlNativeFormAssociationKind;

/// Stored native Button parser flag and custom error, independently of markup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomButtonState {
    pub(crate) control: crate::control::NativeControlState,
}
impl DomButtonState {
    pub(crate) fn fresh() -> Self {
        Self {
            control: crate::control::NativeControlState::fresh(),
        }
    }
    pub fn kind(&self) -> HtmlNativeFormAssociationKind {
        HtmlNativeFormAssociationKind::Button
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
    pub fn custom_validity_message(&self) -> &DocumentString {
        &self.control.message
    }
}
/// Checked Button view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomButton<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomButton<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    /// Actual established relation at this captured revision.
    pub fn form_owner(self) -> Result<Option<NodeId>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner)
    }
    pub fn state(self) -> &'a DomButtonState {
        self.node
            .as_element()
            .expect("checked Button")
            .button_state()
            .expect("checked Button")
    }
    /// Raw weak slot. HTML's scoped associated-element getter is a semantic query.
    pub fn explicit_command_for(self) -> Option<NodeId> {
        self.node
            .node
            .reflected_targets
            .expect("checked Button")
            .get(ReflectedTarget::CommandFor)
            .expect("Button command slot")
    }
    /// Independent raw weak slot, not the common popover trigger state.
    pub fn explicit_popover_target(self) -> Option<NodeId> {
        self.node
            .node
            .reflected_targets
            .expect("checked Button")
            .get(ReflectedTarget::PopoverTarget)
            .expect("Button popover slot")
    }
}
