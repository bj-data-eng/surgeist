//! Native Output state; derived text readbacks use the HTML semantic queries.
use crate::internal::{form::DomForm, identity::NodeId, snapshot::DomNode, store::DomError};
use surgeist_html::values::DocumentString;

/// Actual nullable default-value override, independent of descendant Text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomOutputState {
    pub(crate) control: crate::internal::control::NativeControlState,
    pub(crate) default_override: Option<DocumentString>,
}
impl DomOutputState {
    pub(crate) fn fresh() -> Self {
        Self {
            control: crate::internal::control::NativeControlState::fresh(),
            default_override: None,
        }
    }
    /// None is the source null override; Some(empty) is an established empty default.
    pub fn default_value_override(&self) -> Option<&DocumentString> {
        self.default_override.as_ref()
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
    pub fn custom_validity_message(&self) -> &DocumentString {
        &self.control.message
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.control.message.len().checked_add(
            self.default_override
                .as_ref()
                .map_or(0, DocumentString::len),
        )
    }
}
/// Checked Output view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomOutput<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomOutput<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a DomOutputState {
        self.node
            .as_element()
            .expect("checked Output")
            .output_state()
            .expect("checked Output")
    }
    pub fn form_owner(self) -> Result<Option<NodeId>, DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner())
    }
    /// Read the stored Form owner in this published snapshot segment.
    ///
    /// A live caller qualifies its access to the current segment. `step` admits
    /// the association read and each reached owner lookup.
    ///
    /// # Errors
    ///
    /// Returns the first admission or lookup failure, or
    /// [`DomError::WrongNodeKind`] if the stored owner is not a native Form.
    pub fn form_owner_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomForm<'a>>, E> {
        self.node.stored_form_owner_with(step)
    }
}
#[derive(Clone, Debug)]
pub(crate) enum Edit {
    Value(DocumentString),
    DefaultValue(DocumentString),
    Reset,
}
