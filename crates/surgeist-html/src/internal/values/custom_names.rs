//! HTML's checked custom-element names, preserving exact native UTF-16.
//!
//! HTML 877d6146487f §4.13.3 delegates the local-name condition to the selected
//! DOM foundation. The required lowercase ASCII first code point selects DOM's
//! ASCII-alpha branch; subsequent code points need not be Unicode scalars.

use crate::internal::microsyntax::tokens::is_ascii_whitespace;
use crate::{
    internal::values::Attribute, internal::values::DocumentString, internal::values::Namespace,
    internal::values::QualifiedName, vocabulary::KnownHtmlAttribute,
};
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

    /// The exact checked local name, with every UTF-16 code unit unchanged.
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
#[non_exhaustive]
pub enum HtmlCustomElementNameError {
    /// The name has no code units.
    Empty,
    /// The first code unit is not an ASCII lowercase letter (`a` to `z`), so the name does not
    /// start a valid custom element name.
    FirstCodePointNotLowercaseAscii,
    /// A code unit forbidden by HTML or the selected DOM ASCII-first branch.
    ForbiddenCodeUnit {
        /// Offset of the first forbidden code unit, in UTF-16 code units.
        code_unit_offset: usize,
        /// The forbidden UTF-16 code unit found at `code_unit_offset`.
        code_unit: u16,
    },
    /// The name contains no hyphen (U+002D), which every valid custom element name requires.
    MissingHyphen,
    /// One of HTML's eight reserved hyphen-containing names.
    ReservedName,
}

impl fmt::Display for HtmlCustomElementNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("custom-element name is empty"),
            Self::FirstCodePointNotLowercaseAscii => {
                f.write_str("custom-element name does not start with a lowercase ASCII letter")
            }
            Self::ForbiddenCodeUnit {
                code_unit_offset,
                code_unit,
            } => write!(
                f,
                "custom-element name contains forbidden code unit U+{code_unit:04X} at UTF-16 offset {code_unit_offset}"
            ),
            Self::MissingHyphen => f.write_str("custom-element name contains no hyphen"),
            Self::ReservedName => f.write_str("custom-element name is reserved by HTML"),
        }
    }
}
impl Error for HtmlCustomElementNameError {}

/// The sole borrowed predicate used by native checking and capture admission.
///
/// A lowercase ASCII first unit is necessarily the first code point. It selects
/// DOM's ASCII-alpha local-name branch, so scanning forbidden units is equivalent
/// to its code-point exclusions and preserves isolated surrogates. No storage is
/// allocated; each name has one unit scan and fixed reserved-name comparisons.
pub fn validate_custom_element_name(
    name: &DocumentString,
) -> Result<(), HtmlCustomElementNameError> {
    match validate_custom_element_name_with(name, |_| Ok::<(), std::convert::Infallible>(())) {
        Ok(result) => result,
        Err(never) => match never {},
    }
}

/// Actual borrowed scanner actions. A consuming controlled operation admits
/// each action before the indicated dereference; native validation remains
/// explicitly unbudgeted and uses the same grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomNameScanAction {
    /// The fixed per-name step, admitted once before any unit of the name is inspected.
    Header,
    /// One unit read of the name, charged before that unit is dereferenced.
    Unit,
    /// Starting the comparison against one of the eight reserved names.
    ReservedHeader,
    /// One unit pair compared against a reserved name, charged before the name's unit is read.
    ReservedPair,
}
/// Validates `name` like [`validate_custom_element_name`] while admitting each scan step first.
///
/// `admit` is called with the [`HtmlCustomNameScanAction`] about to be performed, before the
/// corresponding read of `name`. The outer `Err(E)` is the first refusal from `admit`; the scan
/// stops there and nothing further is read. The inner result is the validation outcome, with
/// the first reached rejection as in [`HtmlCustomElementName::new`]. The grammar is identical to
/// the unbudgeted scan; this adds no allocation.
pub fn validate_custom_element_name_with<E>(
    name: &DocumentString,
    mut admit: impl FnMut(HtmlCustomNameScanAction) -> Result<(), E>,
) -> Result<Result<(), HtmlCustomElementNameError>, E> {
    let units = name.code_units();
    admit(HtmlCustomNameScanAction::Header)?;
    if units.is_empty() {
        return Ok(Err(HtmlCustomElementNameError::Empty));
    }
    admit(HtmlCustomNameScanAction::Unit)?;
    if !matches!(units[0], 0x61..=0x7a) {
        return Ok(Err(
            HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii,
        ));
    }
    let mut hyphen = false;
    let mut code_unit_offset = 0;
    while code_unit_offset < units.len() {
        admit(HtmlCustomNameScanAction::Unit)?;
        let code_unit = units[code_unit_offset];
        if is_ascii_whitespace(code_unit) || matches!(code_unit, 0 | 47 | 62 | 0x41..=0x5a) {
            return Ok(Err(HtmlCustomElementNameError::ForbiddenCodeUnit {
                code_unit_offset,
                code_unit,
            }));
        }
        hyphen |= code_unit == 0x2d;
        code_unit_offset += 1;
    }
    if !hyphen {
        return Ok(Err(HtmlCustomElementNameError::MissingHyphen));
    }
    for reserved in [
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ] {
        admit(HtmlCustomNameScanAction::ReservedHeader)?;
        if units.len() != reserved.len() {
            continue;
        }
        let mut equal = true;
        for (offset, right) in reserved.bytes().enumerate() {
            admit(HtmlCustomNameScanAction::ReservedPair)?;
            if units[offset] != u16::from(right) {
                equal = false;
                break;
            }
        }
        if equal {
            return Ok(Err(HtmlCustomElementNameError::ReservedName));
        }
    }
    Ok(Ok(()))
}

#[cfg(test)]
mod scanner_tests {
    use super::*;
    #[test]
    fn shared_scanner_preserves_literal_native_errors_and_stops_before_rejected_read() {
        for (units, expected) in [
            (vec![], Err(HtmlCustomElementNameError::Empty)),
            (
                vec![65, 45, 98],
                Err(HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii),
            ),
            (
                vec![97, 45, 47],
                Err(HtmlCustomElementNameError::ForbiddenCodeUnit {
                    code_unit_offset: 2,
                    code_unit: 47,
                }),
            ),
            (
                "font-face".encode_utf16().collect(),
                Err(HtmlCustomElementNameError::ReservedName),
            ),
            (vec![97, 98], Err(HtmlCustomElementNameError::MissingHyphen)),
            (vec![97, 45, 0xd800], Ok(())),
        ] {
            let name = DocumentString::from_utf16(units);
            assert_eq!(validate_custom_element_name(&name), expected);
            let scanned = validate_custom_element_name_with(&name, |_| Ok::<(), ()>(())).unwrap();
            assert_eq!(scanned, expected);
        }
        let name = DocumentString::from("a-b");
        let mut accepted = 0;
        let result = validate_custom_element_name_with(&name, |action| {
            if matches!(action, HtmlCustomNameScanAction::Unit) {
                if accepted == 2 {
                    return Err("cancelled before next actual read");
                }
                accepted += 1;
            }
            Ok(())
        });
        assert_eq!(result, Err("cancelled before next actual read"));
        assert_eq!(accepted, 2);
    }
}
