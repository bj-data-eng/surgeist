//! The local specified-nonempty author rule for `details[name]`.
//! Same-tree grouping, exclusivity and open/close transitions are separate.

use super::*;

/// An exact nonempty authored details name, not document-qualified group proof.
///
/// Whitespace, casing, NULL and isolated surrogate code units remain unchanged.
/// This checks only the defining name clause, not serialized HTML characters,
/// group exclusivity/nesting or complete element/document conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDetailsName {
    value: DocumentString,
}

impl HtmlDetailsName {
    /// Rejects only an empty value without trimming, splitting or case folding.
    /// Construction uses the ordinary allocation contract of DocumentString.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlAttributeConformanceError> {
        let value = value.into();
        if value.is_empty() {
            return Err(HtmlAttributeConformanceError::DetailsNameMustNotBeEmpty);
        }
        Ok(Self { value })
    }

    /// Returns the exact accepted code units.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }

    /// Moves the accepted value into a no-namespace lowercase name attribute.
    /// No revalidation, normalization or serialized reparse occurs.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Name.qualified_name(), self.value)
    }
}

impl HtmlValueInterpreter {
    /// Checks only `details[name]`'s specified-nonempty authored-value requirement.
    ///
    /// Original raw UTF-16 length is admitted before expanded-name applicability.
    /// Applicable absence is Missing; empty is retained and nonconforming; every
    /// nonempty value is exact. A wrong role remains Unapplied with its raw record.
    /// Context-author NotRequired selects no additional check in this operation;
    /// it does not establish same-tree groups, exclusivity or nesting conformance.
    /// Allocation follows the ordinary shared-string/report contract, not a
    /// physical-memory, work or cancellation budget.
    pub fn details_name(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlDetailsName>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut result = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Name,
                rows: &[103],
                source: source!("attr-details-name"),
                context: Context::Elements(&[KnownHtmlElement::Details]),
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
        match HtmlDetailsName::new(raw.value().clone()) {
            Ok(name) => {
                result.recognition = HtmlAttributeRecognition::NonemptyText;
                result.author = HtmlAttributeAuthorState::Conforming;
                result.intrinsic = Some(name);
            }
            Err(error) => {
                result.recognition = HtmlAttributeRecognition::Empty;
                result.author = HtmlAttributeAuthorState::Nonconforming(error);
            }
        }
        Ok(result)
    }
}
