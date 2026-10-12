//! Local authored UTF-8 spelling for form accept-charset and meta charset.
//! Encoding selection, decoding and serialized/document conformance are separate.

use super::*;
use std::{error::Error, fmt};

/// An exact ASCII-case-insensitive match for HTML's required UTF-8 label.
///
/// This declared-spelling proof preserves accepted casing. It does not resolve
/// general Encoding labels, select a document/submission encoding, or certify
/// meta element, document or serialized-byte conformance. Construction uses
/// ordinary DocumentString allocation without an aggregate resource budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlUtf8EncodingLabel {
    value: DocumentString,
}

/// The value is not an exact ASCII-case-insensitive match for UTF-8.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlUtf8EncodingLabelError;

impl fmt::Display for HtmlUtf8EncodingLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HTML requires an exact ASCII-case-insensitive UTF-8 label")
    }
}
impl Error for HtmlUtf8EncodingLabelError {}

impl HtmlUtf8EncodingLabel {
    /// Accepts exactly five code units matching UTF-8, without trimming or aliases.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlUtf8EncodingLabelError> {
        let value = value.into();
        if !ascii_keyword(&value, "utf-8") {
            return Err(HtmlUtf8EncodingLabelError);
        }
        Ok(Self { value })
    }

    /// Returns the exact accepted spelling, including its supplied ASCII casing.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }

    /// Moves the accepted spelling into the canonical no-namespace attribute name.
    /// This does not select a form-submission encoding.
    pub fn into_accept_charset_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::AcceptCharset.qualified_name(),
            self.value,
        )
    }

    /// Moves the accepted spelling into the canonical no-namespace attribute name.
    /// This does not establish a document's encoding or declaration conformance.
    pub fn into_charset_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Charset.qualified_name(), self.value)
    }
}

impl HtmlValueInterpreter {
    /// Checks form accept-charset's local required UTF-8 author spelling.
    ///
    /// Entire raw UTF-16 admission precedes expanded-name applicability. Keyword
    /// recognition denotes the one matched literal, not an enumerated state.
    /// Missing and invalid content select no label and no default. Actual form
    /// submission encoding selection and its Encoding/context inputs are outside
    /// this operation. Reports and checked values use ordinary shared allocation;
    /// no work, cancellation or physical-memory admission is promised.
    pub fn form_accept_charset(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlUtf8EncodingLabel>, HtmlValueAdmissionError> {
        self.encoding_label_report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::AcceptCharset,
                rows: &[3],
                source: source!("attr-form-accept-charset"),
                context: Context::Elements(&[KnownHtmlElement::Form]),
                vocabulary_source: None,
            },
            element,
            raw,
        )
    }

    /// Checks meta charset's local required UTF-8 author spelling.
    ///
    /// An applicable present attribute leaves context author conformance Unchecked
    /// even when its spelling fails: exclusivity/content, document uniqueness and
    /// serialized first-1024-byte/no-character-reference constraints need separate
    /// facts. Missing charset establishes no whole-meta conformance. This does not
    /// select an encoding, decode bytes or restart parsing. Whole raw UTF-16
    /// admission precedes applicability, with ordinary allocation as described by
    /// form_accept_charset; literal recognition supplies no enumerated fallback.
    pub fn meta_charset(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlUtf8EncodingLabel>, HtmlValueAdmissionError> {
        self.encoding_label_report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Charset,
                rows: &[19],
                source: source!("attr-meta-charset"),
                context: Context::Elements(&[KnownHtmlElement::Meta]),
                vocabulary_source: None,
            },
            element,
            raw,
        )
    }

    fn encoding_label_report(
        &self,
        rule: RuleMetadata,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlUtf8EncodingLabel>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(rule, element, raw);
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let Some(raw) = raw else {
            result.recognition = HtmlAttributeRecognition::Missing;
            result.author = HtmlAttributeAuthorState::Missing;
            return Ok(result);
        };
        if rule.attribute == KnownHtmlAttribute::Charset {
            result.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        match HtmlUtf8EncodingLabel::new(raw.value().clone()) {
            Ok(label) => {
                result.recognition = HtmlAttributeRecognition::Keyword {
                    matched: "utf-8",
                    canonical: "utf-8",
                };
                result.author = HtmlAttributeAuthorState::Conforming;
                result.intrinsic = Some(label);
            }
            Err(error) => {
                result.recognition = if raw.value().is_empty() {
                    HtmlAttributeRecognition::Empty
                } else {
                    HtmlAttributeRecognition::Unrecognized
                };
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::InvalidUtf8EncodingLabel(error),
                );
            }
        }
        Ok(result)
    }
}
