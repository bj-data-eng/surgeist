//! Local `is` authored-value checking, independently of custom definitions.

use super::*;
use crate::HtmlCustomElementName;

impl HtmlValueInterpreter {
    /// Checks `is`'s valid-custom-element-name value on an HTML element.
    ///
    /// Original raw UTF-16 admission precedes expanded-name applicability and
    /// local checking. No value is trimmed, case-folded, normalized or repaired;
    /// missing and unrecognized values have no checked intrinsic name or default.
    /// Every applicable present report leaves author context `Unchecked`, even
    /// when local syntax fails. A definition/`extends` match and the prohibition
    /// on autonomous elements need actual context, not an inferred hyphen proof.
    ///
    /// The intrinsic payload is a checked local name, not the saved runtime `is`
    /// value or a registered/customized element. Later content changes need not
    /// change runtime behavior. This operation certifies no whole-element author
    /// conformance, registry lookup, upgrade or callback execution. It uses
    /// ordinary allocation and the per-original-value cap, without a semantic
    /// session work, physical-memory or cancellation guarantee.
    pub fn is_attribute(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlCustomElementName>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Is,
                rows: &[78],
                source: source!("attr-is"),
                context: Context::Global,
                vocabulary_source: None,
            },
            element,
            raw,
        );
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let Some(raw) = raw else {
            result.recognition = HtmlAttributeRecognition::Missing;
            result.author = HtmlAttributeAuthorState::Missing;
            return Ok(result);
        };
        result.context_author = HtmlAttributeContextAuthorState::Unchecked;
        match HtmlCustomElementName::new(raw.value().clone()) {
            Ok(name) => {
                result.recognition = HtmlAttributeRecognition::NonemptyText;
                result.author = HtmlAttributeAuthorState::Conforming;
                result.intrinsic = Some(name);
            }
            Err(error) => {
                result.recognition = if raw.value().is_empty() {
                    HtmlAttributeRecognition::Empty
                } else {
                    HtmlAttributeRecognition::Unrecognized
                };
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::InvalidCustomElementName(error),
                );
            }
        }
        Ok(result)
    }
}
