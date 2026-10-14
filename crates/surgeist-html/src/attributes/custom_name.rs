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
        Ok(Self::is_attribute_report_from_borrowed(element, raw))
    }
    /// Conservative atomic inspection/share bound, without copying UTF-16.
    /// One value scan plus eight reserved-name comparisons use at most nine
    /// value-unit visits; the tenth pass covers first-unit/completion inspection.
    /// Namespace and attribute-name visits are included in the same envelope.
    /// The fixed 256 credits cover comparison headers and fixed report/Arc-share
    /// fields (no backing payload allocation), including all early exits.
    pub fn is_attribute_report_work_bound(
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Option<usize> {
        let mut units = element.namespace().uri().map_or(0, DocumentString::len);
        units = units.checked_add(element.local_name().len())?;
        if let Some(raw) = raw {
            units = units
                .checked_add(raw.name().namespace().uri().map_or(0, DocumentString::len))?
                .checked_add(raw.name().local_name().len())?
                .checked_add(raw.value().len())?;
        }
        units.checked_add(1)?.checked_mul(10)?.checked_add(256)
    }
    /// Same report owner for ordinary callers and the admitted semantic caller.
    /// Only existing Arc-backed names/attributes are shared; no namespace,
    /// vocabulary or raw backing storage is allocated here.
    pub(crate) fn is_attribute_report_from_borrowed(
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> HtmlAttributeReport<HtmlCustomElementName> {
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
            return result;
        }
        let Some(raw) = raw else {
            result.recognition = HtmlAttributeRecognition::Missing;
            result.author = HtmlAttributeAuthorState::Missing;
            return result;
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
        result
    }
}
