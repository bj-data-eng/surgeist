//! Input accept's intrinsic token constraints and literal File-state hints.
//! MIME conformance and picker/file matching are separate external operations.
//! WHATWG source attribution is in the crate notice.

use super::*;
use crate::{HtmlCommaSeparatedTokens, HtmlCommaToken, HtmlTokenError};
use std::{collections::HashMap, error::Error, fmt};

const RULE: RuleMetadata = RuleMetadata {
    attribute: KnownHtmlAttribute::Accept,
    rows: &[2],
    source: source!("attr-input-accept"),
    context: Context::Elements(&[KnownHtmlElement::Input]),
    vocabulary_source: None,
};
const MIME_SOURCE: &str = "https://mimesniff.spec.whatwg.org/#valid-mime-type-with-no-parameters";

/// HTML's recognized branches; a symbolic MIME role certifies no MIME syntax.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAcceptSpecifierKind {
    Audio,
    Video,
    Image,
    Extension,
    SymbolicMime,
}

/// One exact comma token with its HTML accept role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAcceptSpecifier {
    token: HtmlCommaToken,
    kind: HtmlAcceptSpecifierKind,
}
impl HtmlAcceptSpecifier {
    /// Classifies without trimming, rewriting, or validating external MIME syntax.
    pub fn new(token: HtmlCommaToken) -> Self {
        let value = token.value();
        let kind = if ascii_keyword(value, "audio/*") {
            HtmlAcceptSpecifierKind::Audio
        } else if ascii_keyword(value, "video/*") {
            HtmlAcceptSpecifierKind::Video
        } else if ascii_keyword(value, "image/*") {
            HtmlAcceptSpecifierKind::Image
        } else if value.code_units().first() == Some(&u16::from(b'.')) {
            HtmlAcceptSpecifierKind::Extension
        } else {
            HtmlAcceptSpecifierKind::SymbolicMime
        };
        Self { token, kind }
    }
    pub fn audio() -> Self {
        Self::literal("audio/*")
    }
    pub fn video() -> Self {
        Self::literal("video/*")
    }
    pub fn image() -> Self {
        Self::literal("image/*")
    }
    fn literal(value: &str) -> Self {
        Self::new(HtmlCommaToken::new(value).expect("defining literal is one comma token"))
    }
    /// HTML requires only a leading dot, in addition to comma-token grammar.
    /// A suffix, registered type, filename grammar or scalar string is not required.
    pub fn extension(value: impl Into<DocumentString>) -> Result<Self, HtmlAcceptSpecifierError> {
        let token = HtmlCommaToken::new(value).map_err(HtmlAcceptSpecifierError::Token)?;
        if token.value().code_units().first() != Some(&u16::from(b'.')) {
            return Err(HtmlAcceptSpecifierError::MissingLeadingDot);
        }
        Ok(Self::new(token))
    }
    pub fn token(&self) -> &HtmlCommaToken {
        &self.token
    }
    pub fn value(&self) -> &DocumentString {
        self.token.value()
    }
    pub fn kind(&self) -> HtmlAcceptSpecifierKind {
        self.kind
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAcceptSpecifierError {
    Token(HtmlTokenError),
    MissingLeadingDot,
}
impl fmt::Display for HtmlAcceptSpecifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTML accept extension: {self:?}")
    }
}
impl Error for HtmlAcceptSpecifierError {}

/// An established HTML uniqueness violation, independent of MIME availability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAcceptDuplicate {
    first_index: usize,
    duplicate_index: usize,
    token: DocumentString,
}
impl HtmlAcceptDuplicate {
    pub fn first_index(&self) -> usize {
        self.first_index
    }
    pub fn duplicate_index(&self) -> usize {
        self.duplicate_index
    }
    pub fn token(&self) -> &DocumentString {
        &self.token
    }
}
impl fmt::Display for HtmlAcceptDuplicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTML accept token {} duplicates token {} under ASCII-insensitive comparison",
            self.duplicate_index, self.first_index
        )
    }
}
impl Error for HtmlAcceptDuplicate {}

/// External conformance availability, never inferred from symbolic storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAcceptMimeConformance {
    NotRequired,
    Unavailable,
    Unapplied,
}

/// Native accept construction with exact spelling order and HTML uniqueness.
/// Symbolic MIME entries remain unchecked even when construction succeeds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAcceptValue {
    specifiers: Vec<HtmlAcceptSpecifier>,
}
impl HtmlAcceptValue {
    pub fn new(
        specifiers: impl IntoIterator<Item = HtmlAcceptSpecifier>,
    ) -> Result<Self, HtmlAcceptDuplicate> {
        let specifiers: Vec<_> = specifiers.into_iter().collect();
        if let Some(error) = duplicate(&specifiers) {
            return Err(error);
        }
        Ok(Self { specifiers })
    }
    pub fn specifiers(&self) -> &[HtmlAcceptSpecifier] {
        &self.specifiers
    }
    pub fn mime_conformance(&self) -> HtmlAcceptMimeConformance {
        mime_conformance(&self.specifiers)
    }
    /// Serializes through the shared comma sequence, preserving token spelling.
    pub fn as_document_string(&self) -> DocumentString {
        HtmlCommaSeparatedTokens::new(self.specifiers.iter().map(|value| value.token.clone()))
            .as_document_string()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAcceptAuthorState {
    Missing,
    Conforming,
    Nonconforming(HtmlAcceptDuplicate),
    RequiresMimeConformance,
    Unapplied,
}

/// Literal intrinsic hint availability, not actual file acceptance or a picker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlAcceptProcessingState {
    MissingAttribute,
    FileHintTokens,
    InputStateRequired,
    IgnoredForInputState(HtmlInputType),
    Unapplied,
}

/// Exact raw, authored fields and processing fields are independent observations.
/// Raw-unit admission is not parser work or physical-allocation accounting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAcceptReport {
    input: HtmlAttributeReport<()>,
    author_fields: Option<Vec<HtmlAcceptSpecifier>>,
    processing_tokens: Option<Vec<HtmlAcceptSpecifier>>,
    author: HtmlAcceptAuthorState,
    mime: HtmlAcceptMimeConformance,
    processing_duplicate: Option<HtmlAcceptDuplicate>,
    context_author: HtmlAttributeContextAuthorState,
    processing: HtmlAcceptProcessingState,
}
impl HtmlAcceptReport {
    pub fn element(&self) -> &QualifiedName {
        self.input.element()
    }
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.input.raw_attribute()
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.input.raw()
    }
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.input.observation()
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.input.applicability()
    }
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.input.attribute()
    }
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.input.inventory_rows()
    }
    pub fn source_clause(&self) -> &'static str {
        self.input.source_clause()
    }
    pub fn mime_source_clause(&self) -> &'static str {
        MIME_SOURCE
    }
    pub fn context_source_clause(&self) -> Option<&'static str> {
        self.input.input_state().map(input_state_source)
    }
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    pub fn input_state(&self) -> Option<HtmlInputType> {
        self.input.input_state()
    }
    /// Fields from authored comma grammar, including a required terminal empty.
    /// Empty/all-ASCII-whitespace strings admit the grammar's zero-token set.
    pub fn author_fields(&self) -> Option<&[HtmlAcceptSpecifier]> {
        self.author_fields.as_deref()
    }
    /// Actual Infra split-on-commas tokens; the final comma has no following field.
    /// These remain visible even when a non-File state ignores the hint effect.
    pub fn processing_tokens(&self) -> Option<&[HtmlAcceptSpecifier]> {
        self.processing_tokens.as_deref()
    }
    pub fn author(&self) -> &HtmlAcceptAuthorState {
        &self.author
    }
    /// MIME availability for authored fields, independent of known duplicates.
    pub fn mime_conformance(&self) -> HtmlAcceptMimeConformance {
        self.mime
    }
    /// Availability for literal processing fields can differ from author fields:
    /// all-whitespace author grammar permits zero tokens, but Infra emits an empty.
    pub fn processing_mime_conformance(&self) -> HtmlAcceptMimeConformance {
        self.processing_tokens.as_ref().map_or(
            if self.input.applicability() == HtmlAttributeApplicability::Applicable {
                HtmlAcceptMimeConformance::NotRequired
            } else {
                HtmlAcceptMimeConformance::Unapplied
            },
            |tokens| mime_conformance(tokens),
        )
    }
    pub fn processing_duplicate(&self) -> Option<&HtmlAcceptDuplicate> {
        self.processing_duplicate.as_ref()
    }
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        self.context_author
    }
    pub fn processing(&self) -> HtmlAcceptProcessingState {
        self.processing
    }
}

fn duplicate(specifiers: &[HtmlAcceptSpecifier]) -> Option<HtmlAcceptDuplicate> {
    let mut seen = HashMap::new();
    for (index, specifier) in specifiers.iter().enumerate() {
        let key: Vec<_> = specifier
            .value()
            .code_units()
            .iter()
            .map(|&unit| {
                if (0x41..=0x5a).contains(&unit) {
                    unit + 0x20
                } else {
                    unit
                }
            })
            .collect();
        if let Some(&first_index) = seen.get(&key) {
            return Some(HtmlAcceptDuplicate {
                first_index,
                duplicate_index: index,
                token: specifier.value().clone(),
            });
        }
        seen.insert(key, index);
    }
    None
}
fn mime_conformance(specifiers: &[HtmlAcceptSpecifier]) -> HtmlAcceptMimeConformance {
    if specifiers
        .iter()
        .any(|value| value.kind == HtmlAcceptSpecifierKind::SymbolicMime)
    {
        HtmlAcceptMimeConformance::Unavailable
    } else {
        HtmlAcceptMimeConformance::NotRequired
    }
}

impl HtmlValueInterpreter {
    /// Retains token observations; effective hints require actual input state.
    pub fn input_accept(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAcceptReport, HtmlValueAdmissionError> {
        self.input_accept_with_state(element, raw, None)
    }
    /// Actual File state supplies literal hints; other states ignore the effect.
    /// This operation does not execute a picker, sniff files or validate MIME.
    pub fn input_accept_for_input(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlAcceptReport, HtmlValueAdmissionError> {
        self.input_accept_with_state(element, raw, Some(state))
    }
    fn input_accept_with_state(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlAcceptReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut input = report(RULE, element, raw);
        if let Some(state) = state {
            input.supplied_context = SuppliedContext::Input(state);
        }
        if input.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(HtmlAcceptReport {
                input,
                author_fields: None,
                processing_tokens: None,
                author: HtmlAcceptAuthorState::Unapplied,
                mime: HtmlAcceptMimeConformance::Unapplied,
                processing_duplicate: None,
                context_author: HtmlAttributeContextAuthorState::Unapplied,
                processing: HtmlAcceptProcessingState::Unapplied,
            });
        }
        let processing_tokens: Option<Vec<_>> = raw.map(|attribute| {
            crate::tokens::split_comma_tokens(attribute.value())
                .tokens()
                .iter()
                .cloned()
                .map(HtmlAcceptSpecifier::new)
                .collect()
        });
        // Reuse the common splitter. Only author grammar's zero-set and literal
        // terminal comma need additional observations; no second delimiter scan.
        let author_fields = raw.map(|attribute| {
            let units = attribute.value().code_units();
            if units.iter().all(|&u| crate::tokens::ascii_whitespace(u)) {
                return Vec::new();
            }
            let mut fields = processing_tokens
                .as_ref()
                .expect("present raw has processing tokens")
                .clone();
            if units.last() == Some(&u16::from(b',')) {
                fields.push(HtmlAcceptSpecifier::new(
                    HtmlCommaToken::new("").expect("empty comma token is permitted"),
                ));
            }
            fields
        });
        let mime = author_fields
            .as_ref()
            .map_or(HtmlAcceptMimeConformance::NotRequired, |fields| {
                mime_conformance(fields)
            });
        let author = author_fields
            .as_ref()
            .map_or(HtmlAcceptAuthorState::Missing, |fields| {
                if let Some(error) = duplicate(fields) {
                    HtmlAcceptAuthorState::Nonconforming(error)
                } else if mime == HtmlAcceptMimeConformance::Unavailable {
                    HtmlAcceptAuthorState::RequiresMimeConformance
                } else {
                    HtmlAcceptAuthorState::Conforming
                }
            });
        let processing_duplicate = processing_tokens
            .as_ref()
            .and_then(|tokens| duplicate(tokens));
        let context_author = if raw.is_none() {
            HtmlAttributeContextAuthorState::NotRequired
        } else {
            match state {
                None => HtmlAttributeContextAuthorState::Unchecked,
                Some(HtmlInputType::File) => HtmlAttributeContextAuthorState::Conforming,
                Some(_) => HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                ),
            }
        };
        let processing = match state {
            None => HtmlAcceptProcessingState::InputStateRequired,
            Some(HtmlInputType::File) if raw.is_none() => {
                HtmlAcceptProcessingState::MissingAttribute
            }
            Some(HtmlInputType::File) => HtmlAcceptProcessingState::FileHintTokens,
            Some(state) => HtmlAcceptProcessingState::IgnoredForInputState(state),
        };
        Ok(HtmlAcceptReport {
            input,
            author_fields,
            processing_tokens,
            author,
            mime,
            processing_duplicate,
            context_author,
            processing,
        })
    }
}
