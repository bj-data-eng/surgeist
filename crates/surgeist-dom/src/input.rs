//! Native Input intrinsic storage; canonical identities stay with Node/Draft.
use crate::{DocumentString, DomNode, NodeId};
use surgeist_html::{
    HtmlInputFileSelection, HtmlInputFlags, HtmlInputImageHistory, HtmlInputSelection,
    HtmlInputType,
};

/// Actual native Input state, independent of its default content attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomInputState {
    pub(crate) control: crate::control::NativeControlState,
    pub(crate) flags: HtmlInputFlags,
    pub(crate) current: DocumentString,
    pub(crate) selection: HtmlInputSelection,
    pub(crate) files: HtmlInputFileSelection,
    pub(crate) image: HtmlInputImageHistory<DocumentString>,
    pub(crate) email_count: Option<usize>,
}
impl DomInputState {
    pub(crate) fn fresh(platform: crate::DomNativeInputPlatform) -> Self {
        Self {
            control: crate::control::NativeControlState::fresh(),
            flags: HtmlInputFlags::fresh(),
            current: "".into(),
            selection: HtmlInputSelection::initial(platform.none_direction()),
            files: HtmlInputFileSelection::Empty,
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
    pub fn selected_files(&self) -> Option<HtmlInputFileSelection> {
        (self.input_type() == HtmlInputType::File).then_some(self.files)
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
    pub fn form_owner(self) -> Result<Option<NodeId>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner())
    }
    /// HTML's value-mode getter: exact current/default/on/empty-file readback.
    pub fn value(self) -> &'a [u16] {
        let element = self.node.as_element().expect("checked Input");
        let content = element
            .attributes()
            .iter()
            .find(|a| {
                a.name().namespace_uri().is_none()
                    && a.name()
                        .local_name()
                        .code_units()
                        .iter()
                        .copied()
                        .eq("value".encode_utf16())
            })
            .map(|a| a.value().code_units());
        self.state()
            .input_type()
            .value_mode()
            .readback(
                self.state().current.code_units(),
                content,
                Some(self.state().files),
            )
            .expect("native selected-file state is established")
    }
    /// Raw permitted-weak slot; the scoped getter remains an HTML semantic query.
    pub fn explicit_popover_target(self) -> Option<NodeId> {
        self.node
            .node
            .reflected_targets()
            .expect("checked Input")
            .get(crate::reflected::ReflectedTarget::PopoverTarget)
            .expect("Input has popover slot")
    }
}
