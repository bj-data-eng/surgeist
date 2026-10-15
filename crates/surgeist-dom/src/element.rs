use crate::names::{name_units, qualified_equals, valid_element_local_name};
use crate::{
    DocumentString, DomError, DomRegistryBinding, HtmlCustomElementState, Namespace,
    OrderedAttributes, QualifiedName,
};
use std::sync::atomic::AtomicBool;

/// Definition association established by this native generic creation surface.
/// Registry/definition object admission is a separate, unexposed capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomElementDefinition {
    Null,
}

/// Canonical generic XML Element state, captured with its owner and tree.
///
/// The native creator currently produces no-namespace elements only. A local
/// name such as `input` or `template` does not create an HTML element. This
/// private-field value cannot be used to bypass DOM validation or native hooks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomElement {
    name: QualifiedName,
    attributes: OrderedAttributes,
    registry: DomRegistryBinding,
    definition: DomElementDefinition,
    custom_state: HtmlCustomElementState,
    saved_is: Option<DocumentString>,
    captured_in_view_transition: bool,
}

impl DomElement {
    pub(crate) fn native(
        local_name: DocumentString,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        if !valid_element_local_name(&local_name, cancelled)? {
            return Err(DomError::InvalidCharacter);
        }
        Ok(Self::generic(
            QualifiedName::new(Namespace::none(), None, local_name),
            None,
        ))
    }
    /// DOM single-node cloning uses trusted canonical name fields, not the
    /// public native name constructor. Other namespaces need their real family
    /// initialization before they can enter this import capability.
    pub(crate) fn clone_generic(&self) -> Result<Self, DomError> {
        if self.name.namespace().uri().is_some() || self.name.prefix().is_some() {
            return Err(DomError::NotSupported);
        }
        Ok(Self::generic(self.name.clone(), self.saved_is.clone()))
    }
    fn generic(name: QualifiedName, saved_is: Option<DocumentString>) -> Self {
        Self {
            name,
            attributes: OrderedAttributes::default(),
            registry: DomRegistryBinding::Null,
            definition: DomElementDefinition::Null,
            custom_state: surgeist_html::custom_element_state_without_definition(
                false,
                saved_is.is_some(),
            ),
            saved_is,
            captured_in_view_transition: false,
        }
    }
    pub fn captured_in_view_transition(&self) -> bool {
        self.captured_in_view_transition
    }
    pub(crate) fn set_captured_in_view_transition(&mut self, captured: bool) {
        self.captured_in_view_transition = captured;
    }
    pub(crate) fn same_intrinsic_state(&self, other: &Self) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.registry == other.registry
            && self.definition == other.definition
            && self.custom_state == other.custom_state
            && self.saved_is == other.saved_is
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
    pub(crate) fn set_attributes(&mut self, attributes: OrderedAttributes) {
        self.attributes = attributes;
    }
    pub(crate) fn units(&self) -> Option<usize> {
        let mut units = name_units(&self.name)?
            .checked_add(self.saved_is.as_ref().map_or(0, DocumentString::len))?;
        for attribute in self.attributes.iter() {
            units = units
                .checked_add(name_units(attribute.name())?)?
                .checked_add(attribute.value().len())?;
        }
        Some(units)
    }
}
