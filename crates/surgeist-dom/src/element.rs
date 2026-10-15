use crate::names::{name_units, qualified_equals, valid_element_local_name};
use crate::{
    DocumentString, DomError, DomRegistryBinding, HtmlCustomElementState, Namespace,
    OrderedAttributes, QualifiedName,
};
use std::sync::atomic::AtomicBool;
use surgeist_html::{
    HtmlCanvasNoneState, HtmlMarqueeState, HtmlNativeCommonState,
    HtmlNativeInitializationRequirement as Requirement, KnownHtmlElement,
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
            cancelled,
        )
    }
    pub(crate) fn native_ns(
        namespace: Namespace,
        qualified: DocumentString,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        let name = crate::names::element_name(namespace, qualified, cancelled)?;
        Self::initialize(name, None, cancelled)
    }
    /// Initialize trusted captured name fields; copied attributes and the
    /// source nonce slot are applied separately in source cloning order.
    pub(crate) fn clone_initialized(&self, cancelled: &AtomicBool) -> Result<Self, DomError> {
        Self::initialize(self.name.clone(), self.saved_is.clone(), cancelled)
    }
    fn initialize(
        name: QualifiedName,
        saved_is: Option<DocumentString>,
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
            Requirement::KnownHtml(KnownHtmlElement::Canvas) => {
                NativeFamily::Canvas(std::sync::Arc::new(HtmlCanvasNoneState::fresh()))
            }
            Requirement::HtmlMarquee => NativeFamily::Marquee(HtmlMarqueeState::fresh()),
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
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        self.family = self
            .family
            .after_attribute(changed, attributes.as_slice(), &mut || {
                crate::store::check_cancel(cancelled)
            })?;
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
