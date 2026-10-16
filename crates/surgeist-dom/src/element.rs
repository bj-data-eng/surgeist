use crate::names::{name_units, qualified_equals, valid_element_local_name};
use crate::{
    DocumentString, DomError, DomRegistryBinding, HtmlCustomElementState, Namespace,
    OrderedAttributes, QualifiedName,
};
use std::sync::atomic::AtomicBool;
use surgeist_html::{
    HtmlCanvasNoneState, HtmlFrameNullContent, HtmlIframeNativeState, HtmlMarqueeState,
    HtmlNativeCommonState, HtmlNativeInitializationRequirement as Requirement, KnownHtmlElement,
};
mod family;
use family::NativeFamily;

/// Definition association established by this native generic creation surface.
/// Registry/definition object admission is a separate, unexposed capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomElementDefinition {
    Null,
}

/// Canonical supported native Element state, captured with its owner and tree.
/// Private fields prevent bypassing name validation and native initialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomElement {
    name: QualifiedName,
    attributes: OrderedAttributes,
    registry: DomRegistryBinding,
    definition: DomElementDefinition,
    custom_state: HtmlCustomElementState,
    saved_is: Option<DocumentString>,
    common: HtmlNativeCommonState,
    nonce: Option<DocumentString>,
    family: NativeFamily,
    captured_in_view_transition: bool,
}

impl DomElement {
    pub(crate) fn native_components(
        name: QualifiedName,
        environment: crate::DomNativeDocumentEnvironment,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        crate::names::validate_element_components(&name, cancelled)?;
        Self::initialize(name, None, environment, cancelled)
    }
    pub(crate) fn native(
        local_name: DocumentString,
        document: &crate::DomDocumentMetadata,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        if !valid_element_local_name(&local_name, cancelled)? {
            return Err(DomError::InvalidCharacter);
        }
        let html = document.class() == crate::DomDocumentClass::Html;
        let local_name = if html {
            crate::names::ascii_lowercase(local_name, cancelled)?
        } else {
            local_name
        };
        let namespace = if html
            || document
                .content_type()
                .code_units()
                .iter()
                .copied()
                .eq("application/xhtml+xml".encode_utf16())
        {
            Namespace::from_uri("http://www.w3.org/1999/xhtml")
        } else {
            Namespace::none()
        };
        Self::initialize(
            QualifiedName::new(namespace, None, local_name),
            None,
            document.native_environment(),
            cancelled,
        )
    }
    pub(crate) fn native_ns(
        namespace: Namespace,
        qualified: DocumentString,
        environment: crate::DomNativeDocumentEnvironment,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let name = crate::names::element_name(namespace, qualified, cancelled)?;
        Self::initialize(name, None, environment, cancelled)
    }
    /// Initialize trusted captured name fields; copied attributes and the
    /// source nonce slot are applied separately in source cloning order.
    pub(crate) fn clone_initialized(
        &self,
        environment: crate::DomNativeDocumentEnvironment,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        Self::initialize(
            self.name.clone(),
            self.saved_is.clone(),
            environment,
            cancelled,
        )
    }
    fn initialize(
        name: QualifiedName,
        saved_is: Option<DocumentString>,
        environment: crate::DomNativeDocumentEnvironment,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let initialization =
            surgeist_html::bounded::native_element_initialization_with(&name, |_| {
                crate::store::check_cancel(cancelled)
            })?;
        let family = match initialization.requirement() {
            Requirement::GenericElement
            | Requirement::CommonHtml
            | Requirement::CommonSvg
            | Requirement::CommonMathml => NativeFamily::Common,
            Requirement::KnownHtml(KnownHtmlElement::Form) => {
                NativeFamily::Form(surgeist_html::HtmlFormNativeState::fresh())
            }
            Requirement::KnownHtml(KnownHtmlElement::Template) => NativeFamily::Template(
                initialization
                    .template_insertion()
                    .expect("actual Template fresh insertion evidence"),
            ),
            Requirement::KnownHtml(KnownHtmlElement::Canvas) => {
                NativeFamily::Canvas(std::sync::Arc::new(HtmlCanvasNoneState::fresh()))
            }
            Requirement::KnownHtml(KnownHtmlElement::Iframe) => {
                // Every admitted target Document has the actual native never-BC
                // environment. This is fresh element creation, not reconstruction
                // from current null content or missing imported metadata.
                NativeFamily::Iframe(HtmlIframeNativeState::fresh())
            }
            Requirement::KnownHtml(KnownHtmlElement::Script) => NativeFamily::Script,
            Requirement::SvgScript => {
                NativeFamily::SvgScript(crate::svg_script::SvgScriptState::fresh())
            }
            Requirement::KnownHtml(KnownHtmlElement::Slot) => {
                let Some(surgeist_html::HtmlNativeFreshSlot::EmptyManualAssignments) =
                    initialization.slot()
                else {
                    unreachable!("actual Slot initialization evidence")
                };
                NativeFamily::Slot
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Button) => {
                NativeFamily::Button(crate::DomButtonState::fresh())
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Fieldset) => {
                NativeFamily::Fieldset(crate::DomFieldsetState::fresh())
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Input) => {
                NativeFamily::Input(std::sync::Arc::new(crate::DomInputState::fresh(
                    environment.input_platform(),
                )))
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Textarea) => {
                NativeFamily::Textarea(std::sync::Arc::new(crate::DomTextareaState::fresh(
                    environment.input_platform(),
                )))
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Output) => {
                NativeFamily::Output(std::sync::Arc::new(crate::DomOutputState::fresh()))
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Select) => {
                NativeFamily::Select(std::sync::Arc::new(crate::DomSelectState::fresh()))
            }
            Requirement::KnownHtml(KnownHtmlElement::Option) => {
                NativeFamily::Option(crate::DomOptionState::default())
            }
            Requirement::KnownHtml(KnownHtmlElement::Details) => NativeFamily::Details,
            Requirement::KnownHtml(KnownHtmlElement::Base) => NativeFamily::Base,
            Requirement::KnownHtml(KnownHtmlElement::Link) => NativeFamily::Link,
            Requirement::KnownHtml(KnownHtmlElement::A | KnownHtmlElement::Area) => {
                NativeFamily::Hyperlink
            }
            Requirement::NativeForm(surgeist_html::HtmlNativeFormAssociationKind::Image) => {
                NativeFamily::Img
            }
            Requirement::KnownHtml(KnownHtmlElement::Picture) => NativeFamily::Picture,
            Requirement::KnownHtml(KnownHtmlElement::Source) => NativeFamily::Source,
            Requirement::KnownHtml(KnownHtmlElement::Optgroup) => NativeFamily::Optgroup,
            Requirement::KnownHtml(KnownHtmlElement::Selectedcontent) => {
                NativeFamily::Selectedcontent(false)
            }
            Requirement::HtmlMarquee => NativeFamily::Marquee(HtmlMarqueeState::fresh()),
            Requirement::HtmlFrame => NativeFamily::Frame(HtmlFrameNullContent::fresh()),
            requirement => return Err(DomError::NativeInitializationRequired(requirement)),
        };
        let common = initialization.common();
        let html = common.html().is_some();
        let custom_name = html
            && (saved_is.is_some()
                || surgeist_html::bounded::validate_custom_element_name_with(
                    name.local_name(),
                    |_| crate::store::check_cancel(cancelled),
                )?
                .is_ok());
        Ok(Self {
            name,
            attributes: OrderedAttributes::default(),
            registry: DomRegistryBinding::Null,
            definition: DomElementDefinition::Null,
            custom_state: surgeist_html::custom_element_state_without_definition(html, custom_name),
            saved_is,
            common,
            nonce: common.nonce().map(|_| DocumentString::from("")),
            family,
            captured_in_view_transition: false,
        })
    }
    /// Actual native Template insertion references; contents identity is in its snapshot view.
    pub fn template_insertion_state(
        &self,
    ) -> Option<surgeist_html::HtmlTemplateFreshInsertionState> {
        match self.family {
            NativeFamily::Template(state) => Some(state),
            _ => None,
        }
    }
    /// Actual native Form scalars, independent of identity-bearing relations.
    pub fn form_state(&self) -> Option<surgeist_html::HtmlFormNativeState> {
        match self.family {
            NativeFamily::Form(state) => Some(state),
            _ => None,
        }
    }
    /// Actual stored Fieldset state, or inapplicability for another family.
    pub fn fieldset_state(&self) -> Option<&crate::DomFieldsetState> {
        match &self.family {
            NativeFamily::Fieldset(state) => Some(state),
            _ => None,
        }
    }
    /// Actual stored Button state; raw weak slots belong to its snapshot view.
    pub fn button_state(&self) -> Option<&crate::DomButtonState> {
        match &self.family {
            NativeFamily::Button(state) => Some(state),
            _ => None,
        }
    }
    pub fn input_state(&self) -> Option<&crate::DomInputState> {
        match &self.family {
            NativeFamily::Input(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn input_state_mut(&mut self) -> Option<&mut crate::DomInputState> {
        match &mut self.family {
            NativeFamily::Input(state) => Some(std::sync::Arc::make_mut(state)),
            _ => None,
        }
    }
    pub fn textarea_state(&self) -> Option<&crate::DomTextareaState> {
        match &self.family {
            NativeFamily::Textarea(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn textarea_state_mut(&mut self) -> Option<&mut crate::DomTextareaState> {
        match &mut self.family {
            NativeFamily::Textarea(state) => Some(std::sync::Arc::make_mut(state)),
            _ => None,
        }
    }
    pub fn output_state(&self) -> Option<&crate::DomOutputState> {
        match &self.family {
            NativeFamily::Output(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn output_state_mut(&mut self) -> Option<&mut crate::DomOutputState> {
        match &mut self.family {
            NativeFamily::Output(state) => Some(std::sync::Arc::make_mut(state)),
            _ => None,
        }
    }
    pub(crate) fn is_base(&self) -> bool {
        matches!(self.family, NativeFamily::Base)
    }
    pub(crate) fn is_hyperlink(&self) -> bool {
        matches!(self.family, NativeFamily::Hyperlink)
    }
    pub(crate) fn is_img(&self) -> bool {
        matches!(self.family, NativeFamily::Img)
    }
    pub(crate) fn image_tree_element(&self) -> Option<KnownHtmlElement> {
        match self.family {
            NativeFamily::Img => Some(KnownHtmlElement::Img),
            NativeFamily::Picture => Some(KnownHtmlElement::Picture),
            NativeFamily::Source => Some(KnownHtmlElement::Source),
            _ => KnownHtmlElement::from_name(self.name())
                .filter(|tag| matches!(tag, KnownHtmlElement::Audio | KnownHtmlElement::Video)),
        }
    }
    pub(crate) fn is_link(&self) -> bool {
        matches!(self.family, NativeFamily::Link)
    }
    pub(crate) fn is_details(&self) -> bool {
        matches!(self.family, NativeFamily::Details)
    }
    /// Native Details state is actual exact no-namespace open presence.
    pub fn details_state(&self) -> Option<surgeist_html::HtmlDetailsState> {
        if !self.is_details() {
            return None;
        }
        Some(
            if self.attributes().iter().any(|a| {
                a.name().namespace().uri().is_none()
                    && a.name().local_name().code_units() == [0x6f, 0x70, 0x65, 0x6e]
            }) {
                surgeist_html::HtmlDetailsState::Open
            } else {
                surgeist_html::HtmlDetailsState::Closed
            },
        )
    }
    pub fn select_state(&self) -> Option<&crate::DomSelectState> {
        match &self.family {
            NativeFamily::Select(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn select_state_mut(&mut self) -> Option<&mut crate::DomSelectState> {
        match &mut self.family {
            NativeFamily::Select(s) => Some(std::sync::Arc::make_mut(s)),
            _ => None,
        }
    }
    pub fn option_state(&self) -> Option<crate::DomOptionState> {
        match self.family {
            NativeFamily::Option(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn option_state_mut(&mut self) -> Option<&mut crate::DomOptionState> {
        match &mut self.family {
            NativeFamily::Option(s) => Some(s),
            _ => None,
        }
    }
    pub fn selectedcontent_disabled(&self) -> Option<bool> {
        match self.family {
            NativeFamily::Selectedcontent(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn selectedcontent_disabled_mut(&mut self) -> Option<&mut bool> {
        match &mut self.family {
            NativeFamily::Selectedcontent(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn native_control(
        &self,
    ) -> Option<(
        &crate::control::NativeControlState,
        surgeist_html::HtmlNativeFormAssociationKind,
    )> {
        match &self.family {
            NativeFamily::Select(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Select,
            )),
            NativeFamily::Fieldset(state) => Some((&state.control, state.kind())),
            NativeFamily::Button(state) => Some((&state.control, state.kind())),
            NativeFamily::Input(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Input,
            )),
            NativeFamily::Output(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Output,
            )),
            NativeFamily::Textarea(state) => Some((
                &state.control,
                surgeist_html::HtmlNativeFormAssociationKind::Textarea,
            )),
            _ => None,
        }
    }
    pub(crate) fn native_control_mut(&mut self) -> Option<&mut crate::control::NativeControlState> {
        match &mut self.family {
            NativeFamily::Select(state) => Some(&mut std::sync::Arc::make_mut(state).control),
            NativeFamily::Fieldset(state) => Some(&mut state.control),
            NativeFamily::Button(state) => Some(&mut state.control),
            NativeFamily::Input(state) => Some(&mut std::sync::Arc::make_mut(state).control),
            NativeFamily::Textarea(state) => Some(&mut std::sync::Arc::make_mut(state).control),
            NativeFamily::Output(state) => Some(&mut std::sync::Arc::make_mut(state).control),
            _ => None,
        }
    }
    pub(crate) fn is_slot(&self) -> bool {
        matches!(self.family, NativeFamily::Slot)
    }
    pub(crate) fn svg_script_state(&self) -> Option<&crate::svg_script::SvgScriptState> {
        match &self.family {
            NativeFamily::SvgScript(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn svg_script_state_mut(
        &mut self,
    ) -> Option<&mut crate::svg_script::SvgScriptState> {
        match &mut self.family {
            NativeFamily::SvgScript(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn is_html_script(&self) -> bool {
        matches!(self.family, NativeFamily::Script)
    }
    /// Actual closed common tuple. Exposed operations preserve its values.
    pub fn common_state(&self) -> HtmlNativeCommonState {
        self.common
    }
    /// Cryptographic nonce slot, independent of content. None means inapplicable.
    pub fn nonce(&self) -> Option<&DocumentString> {
        self.nonce.as_ref()
    }
    /// Actual none-mode Canvas state, or inapplicability for another family.
    pub fn canvas_none_state(&self) -> Option<&HtmlCanvasNoneState> {
        match &self.family {
            NativeFamily::Canvas(state) => Some(state),
            _ => None,
        }
    }
    /// Actual native Marquee state, including its closed zero loop index.
    pub fn marquee_state(&self) -> Option<HtmlMarqueeState> {
        match self.family {
            NativeFamily::Marquee(state) => Some(state),
            _ => None,
        }
    }
    /// The actual Frame null-content relationship, or another family's
    /// inapplicability. This reads stored state; it never reconstructs a child
    /// relationship from attributes or fills unknown imported facts.
    pub fn frame_null_content(&self) -> Option<HtmlFrameNullContent> {
        match self.family {
            NativeFamily::Frame(state) => Some(state),
            _ => None,
        }
    }
    /// Actual stored native unprocessed/null-child Iframe state. Other families
    /// are inapplicable; no numeric sandbox policy or missing state is inferred.
    pub fn iframe_native_state(&self) -> Option<HtmlIframeNativeState> {
        match self.family {
            NativeFamily::Iframe(state) => Some(state),
            _ => None,
        }
    }
    pub(crate) fn set_marquee_running(&mut self, running: bool) -> Result<(), DomError> {
        let NativeFamily::Marquee(state) = &mut self.family else {
            return Err(DomError::MarqueeInapplicable);
        };
        if running {
            state.start();
        } else {
            state.stop();
        }
        Ok(())
    }
    pub(crate) fn set_nonce(&mut self, nonce: DocumentString) {
        self.nonce = Some(nonce);
    }
    pub fn captured_in_view_transition(&self) -> bool {
        self.captured_in_view_transition
    }
    pub(crate) fn set_captured_in_view_transition(&mut self, captured: bool) {
        self.captured_in_view_transition = captured;
    }
    pub(crate) fn same_nonattribute_state(&self, other: &Self) -> bool {
        self.name == other.name
            && self.registry == other.registry
            && self.definition == other.definition
            && self.custom_state == other.custom_state
            && self.saved_is == other.saved_is
            && self.common == other.common
            && self.nonce == other.nonce
            && self.family == other.family
    }
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }
    pub fn attributes(&self) -> &OrderedAttributes {
        &self.attributes
    }
    pub fn custom_element_registry(&self) -> DomRegistryBinding {
        self.registry
    }
    pub fn custom_element_definition(&self) -> DomElementDefinition {
        self.definition
    }
    pub fn custom_element_state(&self) -> HtmlCustomElementState {
        self.custom_state
    }
    /// Saved creation `is` is independent of the current content attribute.
    pub fn saved_is(&self) -> Option<&DocumentString> {
        self.saved_is.as_ref()
    }
    /// First exact qualified spelling, without namespace interpretation.
    pub fn attribute(&self, qualified_name: &DocumentString) -> Option<&DocumentString> {
        self.attributes
            .iter()
            .find(|attribute| qualified_equals(attribute.name(), qualified_name))
            .map(|attribute| attribute.value())
    }
    pub fn attribute_ns(
        &self,
        namespace: &Namespace,
        local_name: &DocumentString,
    ) -> Option<&DocumentString> {
        self.attributes
            .get(namespace, local_name)
            .map(|attribute| attribute.value())
    }
    pub(crate) fn apply_attributes(
        &mut self,
        attributes: OrderedAttributes,
        changed: &QualifiedName,
        removed: bool,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.family = self.family.after_attribute(
            &self.name,
            changed,
            self.attributes.as_slice(),
            attributes.as_slice(),
            removed,
            &mut || crate::store::check_cancel(cancelled),
        )?;
        self.attributes = attributes;
        Ok(())
    }
    pub(crate) fn family_units(&self) -> Option<usize> {
        self.family.units()
    }
    pub(crate) fn units(&self) -> Option<usize> {
        let mut units = name_units(&self.name)?
            .checked_add(self.saved_is.as_ref().map_or(0, DocumentString::len))?
            .checked_add(self.nonce.as_ref().map_or(0, DocumentString::len))?;
        units = units.checked_add(self.family.units()?)?;
        for attribute in self.attributes.iter() {
            units = units
                .checked_add(name_units(attribute.name())?)?
                .checked_add(attribute.value().len())?;
        }
        Some(units)
    }
}
