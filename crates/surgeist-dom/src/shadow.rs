//! Native ShadowRoot state from the selected DOM §4.8/attach-shadow clauses.
use crate::{DomRegistryBinding, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};

/// Native attach options. Mode is required; the other source defaults are
/// named slot assignment and false delegates-focus, clonable and serializable.
/// Registry objects and declarative installation are not admitted capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomShadowRootInit {
    pub(crate) properties: HtmlShadowRootProperties,
}
impl DomShadowRootInit {
    pub fn new(mode: HtmlShadowRootMode) -> Self {
        Self {
            properties: HtmlShadowRootProperties::new(
                mode,
                HtmlSlotAssignment::Named,
                false,
                false,
                false,
            ),
        }
    }
    pub fn properties(self) -> HtmlShadowRootProperties {
        self.properties
    }
}
impl From<HtmlShadowRootProperties> for DomShadowRootInit {
    fn from(properties: HtmlShadowRootProperties) -> Self {
        Self { properties }
    }
}

/// Actual canonical native root state. Host identity is a separately staged
/// relation available through the checked `DomShadowRoot` snapshot view.
/// No public constructor can fabricate a declarative or registry-backed root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomShadowRootState {
    properties: HtmlShadowRootProperties,
    available_to_element_internals: bool,
    declarative: bool,
    registry: DomRegistryBinding,
    keep_custom_element_registry_null: bool,
}
impl DomShadowRootState {
    /// Retains the actual source effect's already-qualified null-registry state;
    /// this does not run native attach checks or create a different root.
    pub(crate) fn from_parsed(
        properties: HtmlShadowRootProperties,
        keep_registry_null: bool,
        declarative: bool,
        available_to_internals: bool,
    ) -> Self {
        Self {
            properties,
            keep_custom_element_registry_null: keep_registry_null,
            declarative,
            available_to_element_internals: available_to_internals,
            registry: DomRegistryBinding::Null,
        }
    }
    pub(crate) fn native(init: DomShadowRootInit) -> Self {
        Self {
            properties: init.properties,
            available_to_element_internals: false,
            declarative: false,
            registry: DomRegistryBinding::Null,
            keep_custom_element_registry_null: false,
        }
    }
    pub fn properties(self) -> HtmlShadowRootProperties {
        self.properties
    }
    pub fn mode(self) -> HtmlShadowRootMode {
        self.properties.mode()
    }
    pub fn delegates_focus(self) -> bool {
        self.properties.delegates_focus()
    }
    pub fn serializable(self) -> bool {
        self.properties.serializable()
    }
    pub fn clonable(self) -> bool {
        self.properties.clonable()
    }
    pub fn slot_assignment(self) -> HtmlSlotAssignment {
        self.properties.slot_assignment()
    }
    pub fn available_to_element_internals(self) -> bool {
        self.available_to_element_internals
    }
    pub fn declarative(self) -> bool {
        self.declarative
    }
    pub fn custom_element_registry(self) -> DomRegistryBinding {
        self.registry
    }
    pub fn keep_custom_element_registry_null(self) -> bool {
        self.keep_custom_element_registry_null
    }
}
