//! Native Fieldset state in the canonical owner, independent of current markup.
use crate::internal::{form::DomForm, identity::NodeId, snapshot::DomNode, store::DomError};
use surgeist_html::elements::form::HtmlNativeFormAssociationKind;
use surgeist_html::values::DocumentString;

/// Actual native Fieldset parser flag and custom-error state.
///
/// Owner identity belongs to the checked node view. This is not a complete validation
/// interface. Native clone/import initializes it afresh; adoption preserves it
/// subject to the source removal/reset steps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomFieldsetState {
    pub(crate) control: crate::internal::control::NativeControlState,
}
impl DomFieldsetState {
    pub(crate) fn fresh() -> Self {
        Self {
            control: crate::internal::control::NativeControlState::fresh(),
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
