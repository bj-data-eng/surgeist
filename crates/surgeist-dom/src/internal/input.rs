//! Native Input intrinsic storage; canonical identities stay with Node/Draft.
use crate::{
    file::DomFileList,
    internal::{form::DomForm, identity::NodeId, snapshot::DomNode, store::DomError},
};
use surgeist_html::values::DocumentString;
use surgeist_html::{
    attributes::HtmlInputType,
    elements::input::{
        HtmlInputFlags, HtmlInputImageHistory, HtmlInputSelection, HtmlInputValueMode,
    },
};

/// Actual native Input state, independent of its default content attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomInputState {
    pub(crate) control: crate::internal::control::NativeControlState,
    pub(crate) flags: HtmlInputFlags,
    pub(crate) current: DocumentString,
    pub(crate) selection: HtmlInputSelection,
    pub(crate) files: Option<DomFileList>,
    pub(crate) image: HtmlInputImageHistory<DocumentString>,
    pub(crate) email_count: Option<usize>,
}
impl DomInputState {
    pub(crate) fn fresh(platform: crate::internal::document::DomNativeInputPlatform) -> Self {
        Self {
            control: crate::internal::control::NativeControlState::fresh(),
            flags: HtmlInputFlags::fresh(),
            current: "".into(),
            selection: HtmlInputSelection::initial(platform.none_direction()),
            files: None,
            image: HtmlInputImageHistory::NeverEntered,
            email_count: None,
        }
    }
    pub fn flags(&self) -> HtmlInputFlags {
        self.flags
    }
    pub fn input_type(&self) -> HtmlInputType {
        self.flags.input_type()
    }
    /// Stored value-mode string, not the mode-dependent value getter.
    pub fn current_value(&self) -> &DocumentString {
        &self.current
    }
    pub fn selection(&self) -> Option<HtmlInputSelection> {
        self.input_type().is_selectable().then_some(self.selection)
    }
    /// The exact native FileList in File Upload state, including an empty list.
    pub fn selected_files(&self) -> Option<&DomFileList> {
        self.files.as_ref()
    }
    pub fn image_history(&self) -> &HtmlInputImageHistory<DocumentString> {
        &self.image
    }
    pub fn email_value_count(&self) -> Option<usize> {
        self.email_count
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
    pub fn custom_validity_message(&self) -> &DocumentString {
        &self.control.message
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.current
            .len()
            .checked_add(self.control.message.len())?
            .checked_add(match &self.image {
                HtmlInputImageHistory::Last {
                    source: Some(value),
                    ..
                } => value.len(),
                _ => 0,
            })
    }
}

/// Checked native Input view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomInput<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomInput<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a DomInputState {
        self.node
            .as_element()
            .expect("checked Input")
            .input_state()
            .expect("checked Input")
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

    /// Read the canonical Input type from this published snapshot segment.
    ///
    /// `step` admits the state read and its refusal is returned unchanged.
    /// A live caller qualifies its access to the current segment.
    pub fn input_type_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlInputType, E> {
        Ok(self.state_with(step)?.input_type())
    }

    /// HTML's value-mode getter over the exact current, default, on or file value.
    pub fn value(self) -> &'a [u16] {
        self.value_with(|| Ok::<_, DomError>(()))
            .expect("native Input value state is established")
    }

    /// Borrow the exact value selected by the current native storage mode.
    ///
    /// `step` admits the state read and the reached attribute lookup in
    /// Default and Default/On modes. Value and Filename modes never look up
    /// the content attribute. The caller admits any output copy and qualifies
    /// access to this published snapshot segment.
    ///
    /// # Errors
    ///
    /// Returns the first admission or canonical lookup failure unchanged.
    pub fn value_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<&'a [u16], E> {
        const VALUE: &[u16] = &[
            b'v' as u16,
            b'a' as u16,
            b'l' as u16,
            b'u' as u16,
            b'e' as u16,
        ];
        let state = self.state_with(&mut step)?;
        let mode = state.input_type().value_mode();
        let (current, content, files) = match mode {
            HtmlInputValueMode::Value => (state.current_value().code_units(), None, None),
            HtmlInputValueMode::Default | HtmlInputValueMode::DefaultOn => {
                let content = self
                    .node
                    .attributes()
                    .ok_or(DomError::WrongNodeKind)?
                    .get_null_namespace_attribute_with(VALUE, step)?
                    .map(|attribute| attribute.value().code_units());
                (&[][..], content, None)
            }
            HtmlInputValueMode::Filename => (
                &[][..],
                None,
                state.selected_files().map(DomFileList::filename_value),
            ),
        };
        mode.readback(current, content, files)
            .ok_or_else(|| DomError::WrongNodeKind.into())
    }

    fn state_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<&'a DomInputState, E> {
        step()?;
        self.node
            .as_element()
            .and_then(|element| element.input_state())
            .ok_or_else(|| DomError::WrongNodeKind.into())
    }

    /// Stored weak reference. Use `DomNode::html_popover_target_with` for
    /// the scoped associated Element.
    pub fn explicit_popover_target(self) -> Option<NodeId> {
        self.node
            .node
            .reflected_targets()
            .expect("checked Input")
            .get(crate::internal::reflected::ReflectedTarget::PopoverTarget)
            .expect("Input has popover slot")
    }
}
