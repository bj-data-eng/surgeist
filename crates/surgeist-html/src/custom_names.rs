//! HTML's checked custom-element names, preserving exact native UTF-16.
//!
//! HTML 877d6146487f §4.13.3 delegates the local-name condition to the selected
//! DOM foundation. The required lowercase ASCII first code point selects DOM's
//! ASCII-alpha branch; subsequent code points need not be Unicode scalars.

use crate::{Attribute, DocumentString, KnownHtmlAttribute, Namespace, QualifiedName};
use std::{error::Error, fmt};

/// A name satisfying HTML's valid-custom-element-name predicate.
///
/// This proves local name syntax, not a registered definition, autonomous or
/// customized-built-in classification, an `extends` match, serialized HTML
/// syntax, upgrade, callbacks or any other runtime behavior. Exact code units
/// remain unchanged, including isolated surrogates. Allocation is ordinary
/// native allocation, without a work, physical-memory or cancellation budget.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlCustomElementName {
    value: DocumentString,
}

impl HtmlCustomElementName {
    /// Checks the complete name without trimming, folding or normalization.
    ///
    /// After the first lowercase ASCII letter, the selected DOM rule permits
    /// characters such as `<`, `=`, quotes, noncharacters and isolated
    /// surrogates. ASCII whitespace, NULL, `/`, `>`, uppercase ASCII, missing
    /// hyphens and HTML's eight reserved names are rejected.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlCustomElementNameError> {
        let value = value.into();
        validate_custom_element_name(&value)?;
        Ok(Self { value })
    }

    pub fn value(&self) -> &DocumentString {
        &self.value
    }

    /// Emits an unprefixed HTML qualified name with the exact checked local name.
    /// No custom-element definition or behavior is selected by this conversion.
    pub fn into_qualified_name(self) -> QualifiedName {
        QualifiedName::new(Namespace::html(), None, self.value)
    }

    /// Emits a lowercase no-namespace `is` attribute with the exact checked name.
    ///
    /// This proves only the value's local grammar. Whether the element may
    /// carry `is`, and whether a matching customized-built-in definition exists,
    /// require actual context. Emission does not set a runtime `is` value.
    pub fn into_is_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Is.qualified_name(), self.value)
    }
}

/// A rejected custom-element name, with offsets measured in UTF-16 code units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomElementNameError {
    Empty,
    FirstCodePointNotLowercaseAscii,
    /// A code unit forbidden by HTML or the selected DOM ASCII-first branch.
    ForbiddenCodeUnit {
        code_unit_offset: usize,
        code_unit: u16,
    },
    MissingHyphen,
    /// One of HTML's eight reserved hyphen-containing names.
    ReservedName,
}

impl fmt::Display for HtmlCustomElementNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTML custom-element name: {self:?}")
    }
}
impl Error for HtmlCustomElementNameError {}

/// The sole borrowed predicate used by native checking and capture admission.
///
/// A lowercase ASCII first unit is necessarily the first code point. It selects
/// DOM's ASCII-alpha local-name branch, so scanning forbidden units is equivalent
/// to its code-point exclusions and preserves isolated surrogates. No storage is
/// allocated; each name has one unit scan and fixed reserved-name comparisons.
pub(crate) fn validate_custom_element_name(
    name: &DocumentString,
) -> Result<(), HtmlCustomElementNameError> {
    let units = name.code_units();
    let Some(first) = units.first() else {
        return Err(HtmlCustomElementNameError::Empty);
    };
    if !matches!(first, 0x61..=0x7a) {
        return Err(HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii);
    }
    let mut hyphen = false;
    for (code_unit_offset, &code_unit) in units.iter().enumerate() {
        if matches!(code_unit, 0 | 9 | 10 | 12 | 13 | 32 | 47 | 62 | 0x41..=0x5a) {
            return Err(HtmlCustomElementNameError::ForbiddenCodeUnit {
                code_unit_offset,
                code_unit,
            });
        }
        hyphen |= code_unit == 0x2d;
    }
    if !hyphen {
        return Err(HtmlCustomElementNameError::MissingHyphen);
    }
    if [
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ]
    .iter()
    .any(|reserved| units.iter().copied().eq(reserved.encode_utf16()))
    {
        return Err(HtmlCustomElementNameError::ReservedName);
    }
    Ok(())
}
