use std::{fmt, sync::Arc};

use crate::{DocumentString, HtmlDocumentMode};

const CONTENT_TYPE: &str = "application/xml";
const ENCODING: &str = "UTF-8";
const URL: &str = "about:blank";
// All three source-defined spellings are ASCII, so byte and UTF-16 lengths agree.
pub(crate) const NATIVE_DOCUMENT_STRING_UNITS: usize =
    CONTENT_TYPE.len() + ENCODING.len() + URL.len();

/// Source-defined document type, independent of namespaces and compatibility mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDocumentClass {
    Xml,
    Html,
}

/// The registry binding supported by native document/element creation.
/// No registry object admission or registration operation is exposed yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomRegistryBinding {
    Null,
}

#[derive(Debug)]
struct OpaqueOriginIdentity;

/// Real opaque-origin identity, unrelated to URL spelling or document labels.
///
/// Cloning preserves identity. Fresh documents get distinct tokens, including
/// across stores. No serialized/raw identity or origin-construction escape is
/// exposed. Private aborted drafts cannot publish or consume an observable ID.
#[derive(Clone)]
pub struct DomOpaqueOrigin(Arc<OpaqueOriginIdentity>);

impl PartialEq for DomOpaqueOrigin {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DomOpaqueOrigin {}
impl fmt::Debug for DomOpaqueOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DomOpaqueOrigin(..)")
    }
}

/// Canonical document metadata, captured together with its tree and revision.
///
/// Native defaults follow the selected DOM document clause. Host/imported facts
/// have no unchecked constructor here. Native HTML creation publishes its full
/// source skeleton and copies an actual associated Document origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentMetadata {
    class: DomDocumentClass,
    content_type: DocumentString,
    encoding: DocumentString,
    url: DocumentString,
    origin: DomOpaqueOrigin,
    mode: HtmlDocumentMode,
    allow_declarative_shadow_roots: bool,
    registry: DomRegistryBinding,
    environment: DomNativeDocumentEnvironment,
}

/// Native documents have never been associated with a navigable and have no
/// browsing context. This proof is independent of the document's XML/HTML class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNativeDocumentEnvironment {
    input: DomNativeInputPlatform,
}

/// Established native selection capabilities, independent of browsing context.
/// Construction fixes both facts for this native environment's lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNativeInputPlatform {
    empty_selection: surgeist_html::HtmlInputEmptySelectionSupport,
    none_direction: surgeist_html::HtmlInputNoneDirectionSupport,
}
impl DomNativeInputPlatform {
    pub const fn new(
        empty_selection: surgeist_html::HtmlInputEmptySelectionSupport,
        none_direction: surgeist_html::HtmlInputNoneDirectionSupport,
    ) -> Self {
        Self {
            empty_selection,
            none_direction,
        }
    }
    /// Selected headless profile; neither capability is inferred from no BC.
    pub const fn headless() -> Self {
        Self::new(
            surgeist_html::HtmlInputEmptySelectionSupport::Supported,
            surgeist_html::HtmlInputNoneDirectionSupport::Supported,
        )
    }
    pub const fn empty_selection(self) -> surgeist_html::HtmlInputEmptySelectionSupport {
        self.empty_selection
    }
    pub const fn none_direction(self) -> surgeist_html::HtmlInputNoneDirectionSupport {
        self.none_direction
    }
}
impl DomNativeDocumentEnvironment {
    pub(crate) const fn new(input: DomNativeInputPlatform) -> Self {
        Self { input }
    }
    pub const fn input_platform(self) -> DomNativeInputPlatform {
        self.input
    }
    /// Selected native creation policy, not an inference from missing resources.
    pub const fn input_images(self) -> surgeist_html::HtmlInputImagesDisabled {
        surgeist_html::HtmlInputImagesDisabled::ExplicitlyDisabled
    }
    pub const fn browsing_context(&self) -> surgeist_html::HtmlBrowsingContextPresence {
        surgeist_html::HtmlBrowsingContextPresence::Absent
    }
    pub const fn has_ever_had_navigable(&self) -> bool {
        false
    }
    pub const fn is_fully_active(&self) -> bool {
        false
    }
}

impl DomDocumentMetadata {
    pub(crate) fn native(environment: DomNativeDocumentEnvironment) -> Self {
        Self::native_defaults(
            DomDocumentClass::Xml,
            CONTENT_TYPE,
            DomOpaqueOrigin(Arc::new(OpaqueOriginIdentity)),
            environment,
        )
    }
    pub(crate) fn template_owner(
        class: DomDocumentClass,
        environment: DomNativeDocumentEnvironment,
    ) -> Self {
        let mut metadata = Self::native(environment);
        metadata.class = class;
        metadata
    }
    /// Used only by the complete createHTMLDocument transaction, whose source
    /// Document supplies this identity. No caller can set arbitrary metadata.
    pub(crate) fn native_html(
        origin: DomOpaqueOrigin,
        environment: DomNativeDocumentEnvironment,
    ) -> Self {
        Self::native_defaults(DomDocumentClass::Html, "text/html", origin, environment)
    }
    fn native_defaults(
        class: DomDocumentClass,
        content_type: &str,
        origin: DomOpaqueOrigin,
        environment: DomNativeDocumentEnvironment,
    ) -> Self {
        Self {
            class,
            content_type: content_type.into(),
            encoding: ENCODING.into(),
            url: URL.into(),
            origin,
            mode: HtmlDocumentMode::NoQuirks,
            allow_declarative_shadow_roots: false,
            registry: DomRegistryBinding::Null,
            environment,
        }
    }
    pub fn class(&self) -> DomDocumentClass {
        self.class
    }
    pub fn content_type(&self) -> &DocumentString {
        &self.content_type
    }
    pub fn encoding(&self) -> &DocumentString {
        &self.encoding
    }
    /// Exact URL serialization; this bounded API does not accept arbitrary URLs.
    pub fn url(&self) -> &DocumentString {
        &self.url
    }
    pub fn origin(&self) -> &DomOpaqueOrigin {
        &self.origin
    }
    pub fn mode(&self) -> HtmlDocumentMode {
        self.mode
    }
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        self.allow_declarative_shadow_roots
    }
    pub fn custom_element_registry(&self) -> DomRegistryBinding {
        self.registry
    }
    pub fn native_environment(&self) -> DomNativeDocumentEnvironment {
        self.environment
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.content_type
            .len()
            .checked_add(self.encoding.len())?
            .checked_add(self.url.len())
    }
}
