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
    match validate_custom_element_name_with(name, |_| Ok::<(), std::convert::Infallible>(())) {
        Ok(result) => result,
        Err(never) => match never {},
    }
}

/// Actual borrowed scanner actions. A consuming controlled operation admits
/// each action before the indicated dereference; native validation remains
/// explicitly unbudgeted and uses the same grammar.
pub(crate) enum CustomNameScanAction {
    Header,
    Unit,
    ReservedHeader,
    ReservedPair,
}
pub(crate) fn validate_custom_element_name_with<E>(
    name: &DocumentString,
    mut admit: impl FnMut(CustomNameScanAction) -> Result<(), E>,
) -> Result<Result<(), HtmlCustomElementNameError>, E> {
    let units = name.code_units();
    admit(CustomNameScanAction::Header)?;
    if units.is_empty() {
        return Ok(Err(HtmlCustomElementNameError::Empty));
    }
    admit(CustomNameScanAction::Unit)?;
    if !matches!(units[0], 0x61..=0x7a) {
        return Ok(Err(
            HtmlCustomElementNameError::FirstCodePointNotLowercaseAscii,
        ));
    }
    let mut hyphen = false;
    let mut code_unit_offset = 0;
    while code_unit_offset < units.len() {
        admit(CustomNameScanAction::Unit)?;
        let code_unit = units[code_unit_offset];
        if matches!(code_unit, 0 | 9 | 10 | 12 | 13 | 32 | 47 | 62 | 0x41..=0x5a) {
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
        admit(CustomNameScanAction::ReservedHeader)?;
        if units.len() != reserved.len() {
            continue;
        }
        let mut equal = true;
        for (offset, right) in reserved.bytes().enumerate() {
            admit(CustomNameScanAction::ReservedPair)?;
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
    fn controlled_scanner_propagates_actual_cancellation_before_next_unit() {
        use crate::parser::control::{ParseControl, ParseStop, WorkCategory};
        let cancellation = crate::HtmlParseCancellation::new();
        let mut control = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            cancellation.clone(),
        );
        let name = DocumentString::from("a-b");
        let mut units = 0;
        let result = validate_custom_element_name_with(&name, |action| {
            match action {
                CustomNameScanAction::Header | CustomNameScanAction::ReservedHeader => {
                    control.charge(WorkCategory::RecordVisit, 1)?
                }
                CustomNameScanAction::Unit => {
                    if units == 2 {
                        cancellation.cancel();
                    }
                    control.charge(WorkCategory::InspectByte, 2)?;
                    units += 1;
                }
                CustomNameScanAction::ReservedPair => {
                    control.charge(WorkCategory::InspectByte, 4)?
                }
            }
            Ok::<(), ParseStop>(())
        });
        assert_eq!(result, Err(ParseStop::Cancelled));
        assert_eq!(units, 2);
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            control.charge(WorkCategory::RecordVisit, 1),
            Err(ParseStop::Cancelled)
        );
    }

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
            if matches!(action, CustomNameScanAction::Unit) {
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
