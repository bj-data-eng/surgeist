//! Exact local form-name author syntax; collection uniqueness is independent.
//! [HTML 877d](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-form-name).

use super::*;

/// An exact nonempty form name, without a collection-uniqueness certificate.
/// Whitespace, case, NULL and isolated surrogate code units remain unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlFormName {
    value: DocumentString,
}
impl HtmlFormName {
    /// Rejects only empty; allocation follows the ordinary DocumentString contract.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlAttributeConformanceError> {
        let value = value.into();
        if value.is_empty() {
            return Err(HtmlAttributeConformanceError::FormNameMustNotBeEmpty);
        }
        Ok(Self { value })
    }
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
    /// Moves the exact value into canonical no-namespace lowercase `name`.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Name.qualified_name(), self.value)
    }
}
impl HtmlValueInterpreter {
    /// Checks the optional form name's specified-nonempty local author rule.
    /// Missing recognition is distinct from an empty local failure; optional
    /// absence is locally conforming. Every applicable present value leaves
    /// collection conformance Unchecked. Whole raw admission precedes expanded
    /// name checks. This standalone report has ordinary allocation and a per-raw
    /// UTF-16 cap, not a work/cancellation or physical-allocation guarantee.
    pub fn form_name(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlFormName>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        Ok(Self::form_name_report_from_borrowed(element, raw))
    }

    /// Atomic bounded borrowed assembly envelope, with caller-owned polls.
    /// Four actions per vocabulary candidate position cover reads/comparison/
    /// advance. Eight per namespace position cover both guards; fixed 128 covers
    /// state/metadata fields, length tests and shared-handle retentions. Local
    /// form-name grammar tests only empty, without scanning/copying the value.
    pub fn form_name_report_work_bound(
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Option<usize> {
        let namespace = element.namespace().uri().map_or(0, DocumentString::len);
        let element_work = element
            .local_name()
            .len()
            .checked_add(1)?
            .checked_mul(KnownHtmlElement::ALL.len())?
            .checked_mul(4)?;
        let mut work = 128usize
            .checked_add(namespace.checked_add(1)?.checked_mul(8)?)?
            .checked_add(element_work)?;
        if let Some(raw) = raw {
            work = work.checked_add(
                raw.name()
                    .local_name()
                    .len()
                    .checked_add(1)?
                    .checked_mul(KnownHtmlAttribute::ALL.len())?
                    .checked_mul(4)?,
            )?;
        }
        Some(work)
    }

    /// No temporary names or value buffers. Raw admission/precharge belong to caller.
    pub(crate) fn form_name_report_from_borrowed(
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> HtmlAttributeReport<HtmlFormName> {
        let mut result = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Name,
                rows: &[104],
                source: source!("attr-form-name"),
                context: Context::Elements(&[KnownHtmlElement::Form]),
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
            result.author = HtmlAttributeAuthorState::Conforming;
            return result;
        };
        result.context_author = HtmlAttributeContextAuthorState::Unchecked;
        match HtmlFormName::new(raw.value().clone()) {
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
        result
    }
}
