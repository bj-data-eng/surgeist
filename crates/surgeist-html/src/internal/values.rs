//! Immutable construction values shared by HTML and its tree consumers.

use std::{collections::HashMap, error::Error, fmt, sync::Arc};

pub use document_string::{
    DocumentString, TextBlock, TextRangeError, Utf8ConversionError, WeakTextBlock,
};

pub(crate) const HTML_NAMESPACE_URI: &str = "http://www.w3.org/1999/xhtml";
pub(crate) const SVG_NAMESPACE_URI: &str = "http://www.w3.org/2000/svg";
pub(crate) const MATHML_NAMESPACE_URI: &str = "http://www.w3.org/1998/Math/MathML";

/// Exact UTF-16 comparison with the HTML namespace, without allocating.
///
/// An absent namespace is not the HTML namespace. The URI is ASCII, so a unit
/// count different from its byte length rejects before any unit is compared.
pub(crate) fn is_html_namespace_uri(uri: Option<&DocumentString>) -> bool {
    uri.is_some_and(|uri| {
        uri.len() == HTML_NAMESPACE_URI.len()
            && uri
                .code_units()
                .iter()
                .copied()
                .eq(HTML_NAMESPACE_URI.encode_utf16())
    })
}

/// Namespace identity by exact URI, with an empty URI representing no namespace.
///
/// Named constructors and [`Self::from_uri`] share the same URI identity; prefixes
/// and URI casing do not change or normalize the stored URI.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Namespace {
    uri: Option<DocumentString>,
}

impl Namespace {
    /// Returns the absence of a namespace.
    pub fn none() -> Self {
        Self::default()
    }

    /// Returns the HTML namespace.
    pub fn html() -> Self {
        Self::from_uri(HTML_NAMESPACE_URI)
    }

    /// Returns the SVG namespace.
    pub fn svg() -> Self {
        Self::from_uri(SVG_NAMESPACE_URI)
    }

    /// Returns the MathML namespace.
    pub fn mathml() -> Self {
        Self::from_uri(MATHML_NAMESPACE_URI)
    }

    /// Constructs namespace identity from a URI, mapping an empty URI to none.
    /// No other normalization or URI validation is performed.
    pub fn from_uri(uri: impl Into<DocumentString>) -> Self {
        let uri = uri.into();
        Self {
            uri: (!uri.is_empty()).then_some(uri),
        }
    }

    /// Returns the exact URI, or `None` for no namespace.
    pub fn uri(&self) -> Option<&DocumentString> {
        self.uri.as_ref()
    }

    /// Reports whether this is exactly the HTML namespace, without allocating.
    ///
    /// The comparison is by exact UTF-16 identity: no namespace, other
    /// namespaces, and any case, prefix, suffix or whitespace variation of the
    /// HTML namespace URI are all `false`.
    pub fn is_html(&self) -> bool {
        is_html_namespace_uri(self.uri())
    }
}

/// An exact namespace, optional prefix, and local-name construction value.
///
/// This raw record performs no XML/DOM name validation or HTML case folding.
/// An absent prefix and a present empty prefix remain distinct.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    namespace: Namespace,
    prefix: Option<DocumentString>,
    local_name: DocumentString,
}

impl QualifiedName {
    /// Preserves the supplied name components without language validation.
    pub fn new(
        namespace: Namespace,
        prefix: Option<DocumentString>,
        local_name: impl Into<DocumentString>,
    ) -> Self {
        Self {
            namespace,
            prefix,
            local_name: local_name.into(),
        }
    }

    /// Returns the namespace identity.
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Returns the exact optional prefix.
    pub fn prefix(&self) -> Option<&DocumentString> {
        self.prefix.as_ref()
    }

    /// Returns the exact local name.
    pub fn local_name(&self) -> &DocumentString {
        &self.local_name
    }
}

/// An exact qualified attribute name and raw value.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Attribute {
    name: QualifiedName,
    value: DocumentString,
}

impl Attribute {
    /// Constructs an attribute without interpreting its raw value.
    pub fn new(name: QualifiedName, value: impl Into<DocumentString>) -> Self {
        Self {
            name,
            value: value.into(),
        }
    }

    /// Returns the qualified name.
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// Returns the exact raw value.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// What reserving capacity in a construction backing did: its result, and the capacity after it.
pub(crate) type ReserveOutcome = (Result<(), std::collections::TryReserveError>, usize);

/// Immutable attributes in input order, unique by namespace and local name.
///
/// Prefixes and values do not participate in duplicate identity. Native
/// construction rejects duplicates; parser recovery belongs to the parser.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrderedAttributes {
    attributes: Arc<Vec<Attribute>>,
}

impl OrderedAttributes {
    /// Preserves input order and rejects the first duplicate expanded name.
    pub fn new(
        attributes: impl IntoIterator<Item = Attribute>,
    ) -> Result<Self, DuplicateAttributeError> {
        let attributes: Vec<_> = attributes.into_iter().collect();
        let mut identities = HashMap::new();
        for (duplicate_index, attribute) in attributes.iter().enumerate() {
            let name = attribute.name();
            let identity = (name.namespace(), name.local_name());
            if let Some(first_index) = identities.insert(identity, duplicate_index) {
                return Err(DuplicateAttributeError {
                    namespace: name.namespace().clone(),
                    local_name: name.local_name().clone(),
                    first_index,
                    duplicate_index,
                });
            }
        }
        Ok(Self {
            attributes: Arc::new(attributes),
        })
    }

    /// Borrows the actual immutable backing, including retained spare capacity.
    /// This observation does not clone, validate or allocate.
    pub fn shared_attributes(&self) -> &Arc<Vec<Attribute>> {
        &self.attributes
    }

    /// Only the controlled construction owner may mutate a uniquely owned
    /// backing after proving expanded-name uniqueness and admission.
    ///
    /// This is the one way to change the attributes of an element of a construction storage in
    /// place. Taking the backing advances the storage's attribute `revision`, which a table of
    /// attribute names compares to know that its element's attributes have not changed since it
    /// described them, so no writer can skip it.
    pub(crate) fn construction_backing_mut(
        &mut self,
        revision: &mut u64,
    ) -> &mut Arc<Vec<Attribute>> {
        *revision = revision.wrapping_add(1);
        &mut self.attributes
    }

    /// Replaces the whole list of an element of a construction storage, advancing the storage's
    /// attribute `revision` as [`construction_backing_mut`](Self::construction_backing_mut) does.
    pub(crate) fn replace_in_construction(&mut self, attributes: Self, revision: &mut u64) {
        *revision = revision.wrapping_add(1);
        *self = attributes;
    }

    /// Reserves exactly `additional` slots in a uniquely held backing and reports the capacity it
    /// has afterwards, or `None` when the backing is shared. Capacity is not content, so the
    /// revision does not advance.
    pub(crate) fn reserve_exact_in_construction(
        &mut self,
        additional: usize,
    ) -> Option<ReserveOutcome> {
        let backing = Arc::get_mut(&mut self.attributes)?;
        let result = backing.try_reserve_exact(additional);
        Some((result, backing.capacity()))
    }

    /// Returns the actual backing without copying or revalidating its attributes.
    pub fn into_shared_attributes(self) -> Arc<Vec<Attribute>> {
        self.attributes
    }

    /// Returns all attributes in their original order.
    pub fn as_slice(&self) -> &[Attribute] {
        &self.attributes
    }

    /// Iterates in original attribute order.
    pub fn iter(&self) -> std::slice::Iter<'_, Attribute> {
        self.attributes.iter()
    }

    /// Returns the number of attributes.
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Reports whether there are no attributes.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Finds an attribute by exact namespace and local-name identity.
    pub fn get(&self, namespace: &Namespace, local_name: &DocumentString) -> Option<&Attribute> {
        self.attributes.iter().find(|attribute| {
            attribute.name().namespace() == namespace && attribute.name().local_name() == local_name
        })
    }
}

/// A duplicate expanded attribute name in native ordered construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DuplicateAttributeError {
    namespace: Namespace,
    local_name: DocumentString,
    first_index: usize,
    duplicate_index: usize,
}

impl DuplicateAttributeError {
    /// Returns the duplicate namespace identity.
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Returns the duplicate exact local name.
    pub fn local_name(&self) -> &DocumentString {
        &self.local_name
    }

    /// Returns the first attribute's zero-based index.
    pub fn first_index(&self) -> usize {
        self.first_index
    }

    /// Returns the rejected attribute's zero-based index.
    pub fn duplicate_index(&self) -> usize {
        self.duplicate_index
    }
}

impl fmt::Display for DuplicateAttributeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "attribute at index {} duplicates the namespace and local name at index {}",
            self.duplicate_index, self.first_index
        )
    }
}

impl Error for DuplicateAttributeError {}

pub(crate) mod attribute_name_index;
pub(crate) mod bounded;
pub(crate) mod custom_names;
mod document_string;
pub(crate) mod finish;
pub(crate) mod interpretation;
pub(crate) mod name_index;
pub(crate) mod newlines;
pub(crate) mod semantic_input;
