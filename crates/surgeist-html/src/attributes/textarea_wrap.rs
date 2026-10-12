//! Textarea wrap content state and the separate Hard-state cols-presence rule.
//! This does not perform the textarea wrapping transformation or form submission.

use super::*;
use crate::OrderedAttributes;

/// Conforming local textarea wrap keywords and their intrinsic content states.
/// Soft is also the source's missing/invalid fallback. A Hard native attribute
/// proves keyword validity; its element still requires specified cols. No
/// wrapped text, complete element conformance or submission value is produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTextareaWrap {
    Soft,
    Hard,
}

impl HtmlTextareaWrap {
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::Soft => "soft",
            Self::Hard => "hard",
        }
    }

    /// Emits a canonical no-namespace wrap attribute with ordinary allocation.
    /// Hard's independent element-context requirement is not certified here.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Wrap.qualified_name(),
            self.canonical_keyword(),
        )
    }
}

enum_rule!(TEXTAREA_WRAP, HtmlTextareaWrap, Wrap,
    Context::Elements(&[KnownHtmlElement::Textarea]), [179], "attr-textarea-wrap",
    Some(HtmlTextareaWrap::Soft), Some(HtmlTextareaWrap::Soft), None,
    ["soft" => ("soft", HtmlTextareaWrap::Soft),
     "hard" => ("hard", HtmlTextareaWrap::Hard)]);

impl<T> HtmlAttributeReport<T> {
    /// Returns cols evidence supplied by the textarea-wrap companion operation.
    /// None means unrequested; Some(None) is observed absence; Some(Some(raw))
    /// retains the exact qualified present cols, regardless of numeric validity.
    /// An unapplied companion still retains this evidence; applicability and
    /// context_author distinguish it from an applicable conformance result.
    pub fn textarea_wrap_cols_context(&self) -> Option<Option<&Attribute>> {
        match &self.supplied_context {
            SuppliedContext::TextareaCols(cols) => Some(cols.as_ref()),
            _ => None,
        }
    }
}

fn with_cols_context(
    mut report: HtmlAttributeReport<HtmlTextareaWrap>,
    cols: Option<&Attribute>,
) -> HtmlAttributeReport<HtmlTextareaWrap> {
    report.supplied_context = SuppliedContext::TextareaCols(cols.cloned());
    report.context_author = if report.applicability != HtmlAttributeApplicability::Applicable {
        HtmlAttributeContextAuthorState::Unapplied
    } else if report.intrinsic == Some(HtmlTextareaWrap::Hard) {
        if cols.is_some() {
            HtmlAttributeContextAuthorState::Conforming
        } else {
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::TextareaHardWrapRequiresCols,
            )
        }
    } else {
        HtmlAttributeContextAuthorState::NotRequired
    };
    report
}

impl HtmlValueInterpreter {
    /// Interprets textarea wrap through the shared enumerated-attribute owner.
    /// Missing/invalid select Soft, while recognition and author spelling remain
    /// distinct. Empty has no conforming empty default. Applicable Hard leaves
    /// cols-presence author context Unchecked; Soft selects no such requirement.
    ///
    /// Original raw UTF-16 admission precedes expanded-name applicability. The
    /// result retains exact raw evidence and creates no wrapping, rendering or
    /// submission value. Allocation uses ordinary strings/reports plus the
    /// per-original-value cap, without physical/work/cancellation accounting.
    pub fn textarea_wrap(
        &self,
        element: &QualifiedName,
        wrap: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlTextareaWrap>, HtmlValueAdmissionError> {
        let mut report = self.enumerated(&TEXTAREA_WRAP, element, wrap)?;
        if report.applicability == HtmlAttributeApplicability::Applicable
            && report.intrinsic == Some(HtmlTextareaWrap::Hard)
        {
            report.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(report)
    }

    /// Observes wrap and cols from one complete supplied attribute collection.
    /// Both exact no-namespace lowercase slots are selected, with stored prefixes
    /// retained, then their original values are admitted before applicability or
    /// content recognition. The collection and supplied element name are ordinary
    /// inputs, not a capture/revision/element-identity certificate. Collection
    /// size and unrelated values are outside this per-selected-value cap.
    ///
    /// Hard remains Hard: absent cols yields a separate context-author violation;
    /// any present cols satisfies only this presence requirement, including empty
    /// or malformed text. Numeric author/processing outcomes remain with
    /// textarea_cols. Soft selects no cols requirement. Wrong roles retain
    /// selected raw evidence but are Unapplied. No whole collection is cloned.
    /// Allocation has the same ordinary contract as textarea_wrap, without a
    /// physical-memory ledger, controlled work or cancellation guarantee.
    pub fn textarea_wrap_with_cols(
        &self,
        element: &QualifiedName,
        attributes: &OrderedAttributes,
    ) -> Result<HtmlAttributeReport<HtmlTextareaWrap>, HtmlValueAdmissionError> {
        let namespace = Namespace::none();
        let wrap = attributes.get(&namespace, &DocumentString::from("wrap"));
        let cols = attributes.get(&namespace, &DocumentString::from("cols"));
        self.admit_attribute_raw(wrap.map(Attribute::value))?;
        self.admit_attribute_raw(cols.map(Attribute::value))?;
        Ok(with_cols_context(self.textarea_wrap(element, wrap)?, cols))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cols_context_is_not_an_author_context_for_unrelated_enum_or_boolean_rules() {
        let interpreter = HtmlValueInterpreter::new(32);
        let cols = Attribute::new(KnownHtmlAttribute::Cols.qualified_name(), "garbage");
        let context = SuppliedContext::TextareaCols(Some(cols));
        let method = Attribute::new(KnownHtmlAttribute::Method.qualified_name(), "get");
        let result = interpreter
            .enumerated_with_context(
                &super::super::FORM_METHOD,
                &KnownHtmlElement::Form.qualified_name(),
                Some(&method),
                context.clone(),
            )
            .unwrap();
        assert_eq!(
            result.applicability(),
            HtmlAttributeApplicability::DifferentContext
        );
        assert_eq!(result.intrinsic(), None);
        assert_eq!(
            result.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
        let inert = Attribute::new(KnownHtmlAttribute::Inert.qualified_name(), "");
        let result = interpreter
            .boolean_with_context(
                HtmlBooleanAttribute::Inert,
                &KnownHtmlElement::Div.qualified_name(),
                Some(&inert),
                context,
            )
            .unwrap();
        assert_eq!(
            result.applicability(),
            HtmlAttributeApplicability::DifferentContext
        );
        assert_eq!(result.intrinsic(), None);
        assert_eq!(
            result.context_author(),
            HtmlAttributeContextAuthorState::Unapplied
        );
    }
}
