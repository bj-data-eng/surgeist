//! Local map-name author syntax, independent of own-id equality and tree uniqueness.
//! [HTML 877d map name](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-map-name).

use super::*;
use crate::HtmlSpaceToken;

impl HtmlSpaceToken {
    /// Emits the exact token as a canonical no-namespace lowercase name attribute.
    /// Local syntax does not certify map-id equality or same-tree uniqueness.
    /// Construction has the ordinary native shared-string allocation contract.
    pub fn into_map_name_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Name.qualified_name(),
            self.value().clone(),
        )
    }
}

impl HtmlValueInterpreter {
    /// Checks the required map name's local nonempty/no-ASCII-whitespace syntax.
    /// Absence remains Missing; specified invalid values remain exact. Every
    /// applicable present value leaves independent author context Unchecked.
    /// Raw admission precedes expanded-name applicability. This standalone
    /// operation has a raw-unit cap, not a work or aggregate allocation budget.
    pub fn map_name(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlSpaceToken>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        Ok(Self::map_name_report_from_borrowed(element, raw))
    }

    /// Checked atomic envelope for the actual borrowed report assembly below.
    /// Each vocabulary candidate inspects at most supplied length + 1 positions;
    /// four actions per position cover input/static reads, comparison and advance.
    /// The two namespace guards fit eight actions per URI position. Token syntax
    /// uses at most eight actions per unit (read, five whitespace comparisons,
    /// position and advance). Fixed 128 covers guards, shared-handle retentions,
    /// ten report/five metadata fields and fixed state/branch writes. All loops
    /// and arithmetic are input-bounded; this is not a parser polling quantum.
    pub fn map_name_report_work_bound(
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
            work = work
                .checked_add(
                    raw.name()
                        .local_name()
                        .len()
                        .checked_add(1)?
                        .checked_mul(KnownHtmlAttribute::ALL.len())?
                        .checked_mul(4)?,
                )?
                .checked_add(raw.value().len().checked_add(1)?.checked_mul(8)?)?;
        }
        Some(work)
    }

    /// Actual fixed report construction: no temporary Namespace/QualifiedName,
    /// token buffer or generic report applicability allocation. The caller owns
    /// raw admission and, for captured use, precharge and surrounding polls.
    pub(crate) fn map_name_report_from_borrowed(
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> HtmlAttributeReport<HtmlSpaceToken> {
        let mut result = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Name,
                rows: &[106],
                source: source!("attr-map-name"),
                context: Context::Elements(&[KnownHtmlElement::Map]),
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
        match HtmlSpaceToken::new(raw.value().clone()) {
            Ok(token) => {
                result.recognition = HtmlAttributeRecognition::NonemptyText;
                result.author = HtmlAttributeAuthorState::Conforming;
                result.intrinsic = Some(token);
            }
            Err(error) => {
                result.recognition = if raw.value().is_empty() {
                    HtmlAttributeRecognition::Empty
                } else {
                    HtmlAttributeRecognition::Unrecognized
                };
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::InvalidMapNameToken(error),
                );
            }
        }
        result
    }
}
