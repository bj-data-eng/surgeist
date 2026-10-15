//! Actual native Button payload and its canonical weak-target view.
use crate::{DocumentString, DomNode, NodeId};
use surgeist_html::{HtmlNativeFormAssociationKind, KnownHtmlAttribute};

/// Stored native Button association and custom error, independently of markup.
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
    pub fn form_owner(&self) -> Option<NodeId> {
        self.control.owner
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
            .button_targets
            .expect("checked Button")
            .command_for
    }
    /// Independent raw weak slot, not the common popover trigger state.
    pub fn explicit_popover_target(self) -> Option<NodeId> {
        self.node
            .node
            .button_targets
            .expect("checked Button")
            .popover_target
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ButtonTargets<K> {
    pub(crate) command_for: Option<K>,
    pub(crate) popover_target: Option<K>,
}
impl<K> ButtonTargets<K> {
    pub(crate) fn fresh() -> Self {
        Self {
            command_for: None,
            popover_target: None,
        }
    }
    pub(crate) fn map<T>(self, mut f: impl FnMut(K) -> T) -> ButtonTargets<T> {
        ButtonTargets {
            command_for: self.command_for.map(&mut f),
            popover_target: self.popover_target.map(f),
        }
    }
    pub(crate) fn slot_mut(&mut self, key: ButtonTarget) -> &mut Option<K> {
        match key {
            ButtonTarget::CommandFor => &mut self.command_for,
            ButtonTarget::PopoverTarget => &mut self.popover_target,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum ButtonTarget {
    CommandFor,
    PopoverTarget,
}
impl ButtonTarget {
    pub(crate) fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::CommandFor => KnownHtmlAttribute::Commandfor,
            Self::PopoverTarget => KnownHtmlAttribute::Popovertarget,
        }
    }
}
