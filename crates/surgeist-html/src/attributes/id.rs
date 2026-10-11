//! The local HTML id authored-value rule, independently of tree uniqueness.

use super::*;
use crate::HtmlSpaceToken;

impl HtmlSpaceToken {
    /// Emits the exact token as a canonical no-namespace lowercase id attribute.
    ///
    /// The existing token constructor proves only the local nonempty and
    /// no-ASCII-whitespace rule. This adapter does not prove tree uniqueness,
    /// serialized HTML characters, DOM ID state or complete author conformance.
    /// The immutable shared value is retained without copying its code units,
    /// normalization, revalidation or serialized reparsing.
    pub fn into_id_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Id.qualified_name(),
            self.value().clone(),
        )
    }
}

impl HtmlValueInterpreter {
    /// Checks HTML id's local nonempty and no-ASCII-whitespace author rule.
    ///
    /// The full original raw UTF-16 length is admitted before expanded-name
    /// applicability or validation. Applicable absence is Missing; specified
    /// empty/whitespace values remain exact and locally nonconforming. Every
    /// applicable present id leaves tree-uniqueness author context Unchecked.
    /// A wrong role remains Unapplied with its exact raw record.
    ///
    /// The intrinsic payload is a checked local authored-value token, not DOM
    /// ID state: no checked token does not establish absence of a DOM ID. This
    /// HTML author operation does not supply DOM's any-namespace ID behavior,
    /// tree uniqueness, fragment resolution, CSS selector validity, serialized
    /// character conformance or complete element/document author conformance.
    ///
    /// Allocation follows the ordinary shared-string/report contract plus the
    /// per-original-value raw cap, not a physical-memory, work or cancellation
    /// budget. Validation reuses the existing single-token owner without
    /// trimming, splitting, case folding or Unicode normalization.
    pub fn id_attribute(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlSpaceToken>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Id,
                rows: &[72],
                source: source!("the-id-attribute"),
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
                    HtmlAttributeConformanceError::InvalidIdToken(error),
                );
            }
        }
        Ok(result)
    }
}
