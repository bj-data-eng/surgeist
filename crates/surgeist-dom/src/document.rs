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
/// have no unchecked constructor here. The current native creator makes XML
/// documents only; an HTML value in the class enum does not create an HTML host.
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
}

impl DomDocumentMetadata {
    pub(crate) fn native() -> Self {
        Self {
            class: DomDocumentClass::Xml,
            content_type: CONTENT_TYPE.into(),
            encoding: ENCODING.into(),
            url: URL.into(),
            origin: DomOpaqueOrigin(Arc::new(OpaqueOriginIdentity)),
            mode: HtmlDocumentMode::NoQuirks,
            allow_declarative_shadow_roots: false,
            registry: DomRegistryBinding::Null,
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
    pub(crate) fn units(&self) -> Option<usize> {
        self.content_type
            .len()
            .checked_add(self.encoding.len())?
            .checked_add(self.url.len())
    }
}
