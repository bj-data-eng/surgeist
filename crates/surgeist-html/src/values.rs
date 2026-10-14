//! Immutable construction values shared by HTML and its tree consumers.

use std::{collections::HashMap, error::Error, fmt, sync::Arc};

/// An immutable sequence of exact UTF-16 code units.
///
/// Isolated surrogates, whitespace, and null code units are preserved. Lengths
/// and conversion-error offsets use code units, not Unicode scalar values.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DocumentString {
    code_units: Arc<Vec<u16>>,
}

impl DocumentString {
    /// Constructs a string without validating, replacing, or normalizing units.
    pub fn from_utf16(code_units: impl Into<Vec<u16>>) -> Self {
        Self {
            code_units: Arc::new(code_units.into()),
        }
    }

    /// Adopts immutable UTF-16 backing without copying or normalizing its units.
    pub fn from_shared_code_units(code_units: Arc<Vec<u16>>) -> Self {
        Self { code_units }
    }

    /// Borrows the actual immutable allocation, including its retained capacity.
    pub fn shared_code_units(&self) -> &Arc<Vec<u16>> {
        &self.code_units
    }

    /// Returns the exact stored UTF-16 sequence.
    pub fn code_units(&self) -> &[u16] {
        &self.code_units
    }

    /// Returns the number of UTF-16 code units.
    pub fn len(&self) -> usize {
        self.code_units.len()
    }

    /// Reports whether there are no code units.
    pub fn is_empty(&self) -> bool {
        self.code_units.is_empty()
    }

    /// Iterates code points, combining surrogate pairs and retaining isolated
    /// surrogates as their numeric code points.
    ///
    /// Unlike an iterator of Rust `char`, this can represent every input unit.
    pub fn code_points(&self) -> impl Iterator<Item = u32> + Clone + '_ {
        char::decode_utf16(self.code_units.iter().copied()).map(|decoded| match decoded {
            Ok(scalar) => u32::from(scalar),
            Err(error) => u32::from(error.unpaired_surrogate()),
        })
    }

    /// Converts to UTF-8, rejecting the first isolated surrogate without loss.
    pub fn to_utf8(&self) -> Result<String, Utf8ConversionError> {
        let mut result = String::new();
        let mut code_unit_offset = 0;
        for decoded in char::decode_utf16(self.code_units.iter().copied()) {
            match decoded {
                Ok(scalar) => {
                    result.push(scalar);
                    code_unit_offset += scalar.len_utf16();
                }
                Err(error) => {
                    return Err(Utf8ConversionError {
                        code_unit_offset,
                        surrogate: error.unpaired_surrogate(),
                    });
                }
            }
        }
        Ok(result)
    }
}

impl From<&str> for DocumentString {
    fn from(value: &str) -> Self {
        Self::from_utf16(value.encode_utf16().collect::<Vec<_>>())
    }
}

impl From<String> for DocumentString {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

/// The first isolated surrogate preventing an exact UTF-8 conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Utf8ConversionError {
    code_unit_offset: usize,
    surrogate: u16,
}

impl Utf8ConversionError {
    /// Returns the zero-based UTF-16 code-unit offset.
    pub fn code_unit_offset(&self) -> usize {
        self.code_unit_offset
    }

    /// Returns the isolated surrogate code unit.
    pub fn surrogate(&self) -> u16 {
        self.surrogate
    }
}

impl fmt::Display for Utf8ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "isolated surrogate U+{:04X} at UTF-16 code-unit offset {}",
            self.surrogate, self.code_unit_offset
        )
    }
}

impl Error for Utf8ConversionError {}

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
        Self::from_uri("http://www.w3.org/1999/xhtml")
    }

    /// Returns the SVG namespace.
    pub fn svg() -> Self {
        Self::from_uri("http://www.w3.org/2000/svg")
    }

    /// Returns the MathML namespace.
    pub fn mathml() -> Self {
        Self::from_uri("http://www.w3.org/1998/Math/MathML")
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

    pub(crate) fn backing(&self) -> &Arc<Vec<Attribute>> {
        &self.attributes
    }

    /// Only the controlled construction owner may mutate a uniquely owned
    /// parser backing after proving expanded-name uniqueness and admission.
    pub(crate) fn parser_backing_mut(&mut self) -> &mut Arc<Vec<Attribute>> {
        &mut self.attributes
    }

    pub(crate) fn into_parser_storage(self) -> Arc<Vec<Attribute>> {
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

pub(crate) mod bounded;
