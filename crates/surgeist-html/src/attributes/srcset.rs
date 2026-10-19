//! HTML srcset candidate constraints and literal recovery, with symbolic URLs.
//! Sizes CSS, URL resolution, source-set normalization and image selection are
//! separate operations. WHATWG and selected WebKit attribution is in the notice.

use super::*;
use crate::numeric::{
    self, HtmlScientificDecimalKey, NumericParseError, ScientificDecimalKeyError,
};
use crate::{HtmlFloatingPointNumber, HtmlNonNegativeInteger, OrderedAttributes};
use std::{collections::HashMap, error::Error, fmt};

const GRAMMAR: &str = source!("srcset-attributes");
const PROCESSING: &str = source!("parsing-a-srcset-attribute");
const OVERFLOW_SOURCE: &str = "https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/html/parser/HTMLSrcsetParser.cpp#L134";

/// An HTML URL token role, without URL validity or resolution certification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetUrlToken {
    value: DocumentString,
}
impl HtmlSrcsetUrlToken {
    /// Preserves exact units, including relative references and isolated surrogates.
    /// Checks only nonempty, no ASCII whitespace, and no leading/trailing comma.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlSrcsetUrlTokenError> {
        let value = value.into();
        let units = value.code_units();
        if units.is_empty() {
            return Err(HtmlSrcsetUrlTokenError::Empty);
        }
        if units.iter().any(|&u| crate::tokens::ascii_whitespace(u)) {
            return Err(HtmlSrcsetUrlTokenError::AsciiWhitespace);
        }
        if units.first() == Some(&44) || units.last() == Some(&44) {
            return Err(HtmlSrcsetUrlTokenError::EdgeComma);
        }
        Ok(Self { value })
    }
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetUrlTokenError {
    Empty,
    AsciiWhitespace,
    EdgeComma,
}
impl fmt::Display for HtmlSrcsetUrlTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTML srcset URL token: {self:?}")
    }
}
impl Error for HtmlSrcsetUrlTokenError {}

/// Positive exact width; no machine-integer magnitude cap is imposed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetWidth {
    value: HtmlNonNegativeInteger,
}
impl HtmlSrcsetWidth {
    pub fn new(value: HtmlNonNegativeInteger) -> Result<Self, HtmlSrcsetWidthError> {
        if value.is_zero() {
            Err(HtmlSrcsetWidthError)
        } else {
            Ok(Self { value })
        }
    }
    pub fn value(&self) -> &HtmlNonNegativeInteger {
        &self.value
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetWidthError;
impl fmt::Display for HtmlSrcsetWidthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HTML srcset width must be positive")
    }
}
impl Error for HtmlSrcsetWidthError {}

/// A positive mathematical density and its exact valid HTML decimal spelling.
/// Equality preserves spelling; list uniqueness uses exact mathematical values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetDensity {
    syntax: DocumentString,
    key: HtmlScientificDecimalKey,
}
impl HtmlSrcsetDensity {
    /// Checks complete author grammar and mathematical positivity before rounding.
    /// Uses ordinary native allocations, not a resource/cancellation guarantee.
    pub fn from_decimal_syntax(
        syntax: impl Into<DocumentString>,
    ) -> Result<Self, HtmlSrcsetDensityError> {
        let syntax = syntax.into();
        let key = numeric::strict_scientific_decimal_key(&syntax).map_err(|error| match error {
            ScientificDecimalKeyError::Syntax(_) => HtmlSrcsetDensityError::InvalidSyntax,
            ScientificDecimalKeyError::RepresentationExhausted => {
                HtmlSrcsetDensityError::RepresentationExhausted
            }
        })?;
        if !key.is_positive() {
            return Err(HtmlSrcsetDensityError::Nonpositive);
        }
        Ok(Self { syntax, key })
    }
    pub fn syntax(&self) -> &DocumentString {
        &self.syntax
    }
    /// Source finite binary64 rounding; underflow returns canonical zero.
    pub fn to_floating_point_number(
        &self,
    ) -> Result<HtmlFloatingPointNumber, HtmlSrcsetDensityProjectionError> {
        match numeric::validate_floating_point_number(&self.syntax) {
            Ok(value) => Ok(value),
            Err(NumericParseError::FloatingPointOverflow) => Err(HtmlSrcsetDensityProjectionError),
            Err(_) => unreachable!("checked density owns complete strict numeric grammar"),
        }
    }
}
impl TryFrom<HtmlFloatingPointNumber> for HtmlSrcsetDensity {
    type Error = HtmlSrcsetDensityError;
    fn try_from(value: HtmlFloatingPointNumber) -> Result<Self, Self::Error> {
        Self::from_decimal_syntax(value.as_document_string())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetDensityError {
    InvalidSyntax,
    Nonpositive,
    RepresentationExhausted,
}
impl fmt::Display for HtmlSrcsetDensityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTML srcset density: {self:?}")
    }
}
impl Error for HtmlSrcsetDensityError {}
/// A valid mathematical density overflows the source's finite binary64 domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetDensityProjectionError;
impl fmt::Display for HtmlSrcsetDensityProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HTML srcset density binary64 projection overflow")
    }
}
impl Error for HtmlSrcsetDensityProjectionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetDescriptor {
    Unspecified,
    Width(HtmlSrcsetWidth),
    Density(HtmlSrcsetDensity),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetCandidate {
    url: HtmlSrcsetUrlToken,
    descriptor: HtmlSrcsetDescriptor,
}
impl HtmlSrcsetCandidate {
    pub fn new(url: HtmlSrcsetUrlToken, descriptor: HtmlSrcsetDescriptor) -> Self {
        Self { url, descriptor }
    }
    pub fn url(&self) -> &HtmlSrcsetUrlToken {
        &self.url
    }
    pub fn descriptor(&self) -> &HtmlSrcsetDescriptor {
        &self.descriptor
    }
}

/// Established HTML list constraints; external URL/resource conformance is separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetAuthorViolation {
    EmptyList,
    MissingCandidate,
    InvalidUrlToken,
    InvalidDescriptor,
    NonpositiveDescriptor,
    MultipleDescriptors,
    FutureHeightDescriptor,
    DuplicateWidth { first: usize, duplicate: usize },
    DuplicateDensity { first: usize, duplicate: usize },
    MixedWidthDescriptors,
}
impl fmt::Display for HtmlSrcsetAuthorViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML srcset author constraint: {self:?}")
    }
}
impl Error for HtmlSrcsetAuthorViolation {}

/// Native nonempty candidate list with exact mathematical descriptor uniqueness.
/// Construction does not certify URL, image properties, natural width or context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetValue {
    candidates: Vec<HtmlSrcsetCandidate>,
}
impl HtmlSrcsetValue {
    pub fn new(
        candidates: impl Into<Vec<HtmlSrcsetCandidate>>,
    ) -> Result<Self, HtmlSrcsetAuthorViolation> {
        let candidates = candidates.into();
        if candidates.is_empty() {
            return Err(HtmlSrcsetAuthorViolation::EmptyList);
        }
        if let Some((_, violation)) = list_violations(&candidates).into_iter().next() {
            return Err(violation);
        }
        Ok(Self { candidates })
    }
    pub fn candidates(&self) -> &[HtmlSrcsetCandidate] {
        &self.candidates
    }
    /// Unresolved symbolic URL/resource conformance, not HTML grammar failure.
    pub fn external_conformance(&self) -> HtmlSrcsetExternalConformance {
        HtmlSrcsetExternalConformance::Unchecked
    }
    pub fn as_document_string(&self) -> DocumentString {
        let mut units = Vec::new();
        for (index, candidate) in self.candidates.iter().enumerate() {
            if index != 0 {
                units.extend_from_slice(&[44, 32]);
            }
            units.extend_from_slice(candidate.url.value.code_units());
            match &candidate.descriptor {
                HtmlSrcsetDescriptor::Unspecified => {}
                HtmlSrcsetDescriptor::Width(width) => {
                    units.push(32);
                    units.extend_from_slice(width.value.as_document_string().code_units());
                    units.push(119);
                }
                HtmlSrcsetDescriptor::Density(density) => {
                    units.push(32);
                    units.extend_from_slice(density.syntax.code_units());
                    units.push(120);
                }
            }
        }
        DocumentString::from_utf16(units)
    }
    pub fn into_srcset_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Srcset.qualified_name(),
            self.as_document_string(),
        )
    }
    pub fn into_imagesrcset_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Imagesrcset.qualified_name(),
            self.as_document_string(),
        )
    }
}
fn list_violations(candidates: &[HtmlSrcsetCandidate]) -> Vec<(usize, HtmlSrcsetAuthorViolation)> {
    let mut findings = Vec::new();
    let mut widths = HashMap::new();
    let mut densities = HashMap::new();
    let one = numeric::strict_scientific_decimal_key(&DocumentString::from("1"))
        .expect("literal positive integer");
    let any_width = candidates
        .iter()
        .any(|c| matches!(c.descriptor, HtmlSrcsetDescriptor::Width(_)));
    for (index, candidate) in candidates.iter().enumerate() {
        match &candidate.descriptor {
            HtmlSrcsetDescriptor::Width(width) => {
                if let Some(&first) = widths.get(&width.value) {
                    findings.push((
                        index,
                        HtmlSrcsetAuthorViolation::DuplicateWidth {
                            first,
                            duplicate: index,
                        },
                    ));
                } else {
                    widths.insert(width.value.clone(), index);
                }
            }
            descriptor => {
                if any_width {
                    findings.push((index, HtmlSrcsetAuthorViolation::MixedWidthDescriptors));
                }
                let key = match descriptor {
                    HtmlSrcsetDescriptor::Density(density) => &density.key,
                    _ => &one,
                };
                if let Some(&first) = densities.get(key) {
                    findings.push((
                        index,
                        HtmlSrcsetAuthorViolation::DuplicateDensity {
                            first,
                            duplicate: index,
                        },
                    ));
                } else {
                    densities.insert(key.clone(), index);
                }
            }
        }
    }
    findings
}

/// Half-open UTF-16 units relative to the report's raw attribute value.
/// This is not document source identity, original quote/entity span or URL origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetRawRange {
    start: usize,
    end: usize,
}
impl HtmlSrcsetRawRange {
    pub fn start(self) -> usize {
        self.start
    }
    pub fn end(self) -> usize {
        self.end
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlParsedSrcsetDescriptor {
    Unspecified,
    Width(HtmlSrcsetWidth),
    Density(HtmlFloatingPointNumber),
}
/// Literal recovered candidate; its URL remains unresolved symbolic text.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlParsedImageCandidate {
    url: DocumentString,
    descriptor: HtmlParsedSrcsetDescriptor,
    range: HtmlSrcsetRawRange,
    url_range: HtmlSrcsetRawRange,
}
impl HtmlParsedImageCandidate {
    pub fn url(&self) -> &DocumentString {
        &self.url
    }
    pub fn descriptor(&self) -> &HtmlParsedSrcsetDescriptor {
        &self.descriptor
    }
    /// Consumed candidate spelling, including a terminating comma when present.
    pub fn range(&self) -> HtmlSrcsetRawRange {
        self.range
    }
    /// Exact recovered URL slice; removed trailing commas are outside this range.
    pub fn url_range(&self) -> HtmlSrcsetRawRange {
        self.url_range
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetParseError {
    SkippedCommas,
    MultipleTrailingCommas,
    FutureHeightDescriptor,
    RejectedCandidate,
    DensityConversionOverflow,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetParseDiagnostic {
    error: HtmlSrcsetParseError,
    range: HtmlSrcsetRawRange,
    candidate: Option<usize>,
}
impl HtmlSrcsetParseDiagnostic {
    pub fn error(&self) -> HtmlSrcsetParseError {
        self.error
    }
    pub fn range(&self) -> HtmlSrcsetRawRange {
        self.range
    }
    /// Index among all lexical candidates, including rejected candidates.
    pub fn candidate_index(&self) -> Option<usize> {
        self.candidate
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetAuthorDiagnostic {
    violation: HtmlSrcsetAuthorViolation,
    range: HtmlSrcsetRawRange,
    candidate: Option<usize>,
}
impl HtmlSrcsetAuthorDiagnostic {
    pub fn violation(&self) -> HtmlSrcsetAuthorViolation {
        self.violation
    }
    pub fn range(&self) -> HtmlSrcsetRawRange {
        self.range
    }
    pub fn candidate_index(&self) -> Option<usize> {
        self.candidate
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetAuthorState {
    Missing,
    HtmlConstraintsConforming,
    Nonconforming,
    Unapplied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetProcessingState {
    MissingAttribute,
    Candidates,
    Unapplied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetExternalConformance {
    NotRequired,
    Unchecked,
    Unapplied,
}

/// Actual raw observations supplied by an adapter; no topology is inferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetParentObservation {
    Unavailable,
    Absent,
    Element(QualifiedName),
}
/// Actual following sibling image observation, distinct from unavailable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetFollowingImageObservation {
    Unavailable,
    Absent,
    Element {
        name: QualifiedName,
        attributes: OrderedAttributes,
    },
}
/// Surrounding raw facts; these are observations, not a qualified semantic proof.
/// Target raw is derived from the same attributes to avoid contradictory inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetContext {
    Img {
        attributes: OrderedAttributes,
    },
    Source {
        attributes: OrderedAttributes,
        parent: HtmlSrcsetParentObservation,
        following_image: HtmlSrcsetFollowingImageObservation,
    },
    Link {
        attributes: OrderedAttributes,
    },
}
impl HtmlSrcsetContext {
    fn attributes(&self) -> &OrderedAttributes {
        match self {
            Self::Img { attributes }
            | Self::Source { attributes, .. }
            | Self::Link { attributes } => attributes,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetContextViolation {
    SrcOrSrcsetRequired,
    SourceParentNotPicture,
    SourceSrcsetRequired,
    SizesRequired,
    WidthDescriptorsRequiredForSizes,
    ImagesizesRequired,
    PreloadRelRequired,
    ImageAsStateRequired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetContextAuthorState {
    NotRequired,
    Unchecked,
    Conforming,
    Nonconforming,
    Unapplied,
}

/// Native storage and standalone raw admission are ordinary allocation contracts,
/// not parser/semantic work, physical-memory limits or cancellation guarantees.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlSrcsetReport {
    input: HtmlAttributeReport<()>,
    context: Option<HtmlSrcsetContext>,
    author_fields: Option<HtmlSrcsetValue>,
    author: HtmlSrcsetAuthorState,
    author_diagnostics: Vec<HtmlSrcsetAuthorDiagnostic>,
    external: HtmlSrcsetExternalConformance,
    processing: HtmlSrcsetProcessingState,
    candidates: Option<Vec<HtmlParsedImageCandidate>>,
    diagnostics: Vec<HtmlSrcsetParseDiagnostic>,
    context_author: HtmlSrcsetContextAuthorState,
    context_violations: Vec<HtmlSrcsetContextViolation>,
}
impl HtmlSrcsetReport {
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
    pub fn grammar_source_clause(&self) -> &'static str {
        GRAMMAR
    }
    pub fn processing_source_clause(&self) -> &'static str {
        PROCESSING
    }
    pub fn overflow_recovery_source_clause(&self) -> &'static str {
        OVERFLOW_SOURCE
    }
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    pub fn context(&self) -> Option<&HtmlSrcsetContext> {
        self.context.as_ref()
    }
    pub fn author_fields(&self) -> Option<&HtmlSrcsetValue> {
        self.author_fields.as_ref()
    }
    pub fn author(&self) -> HtmlSrcsetAuthorState {
        self.author
    }
    pub fn author_diagnostics(&self) -> &[HtmlSrcsetAuthorDiagnostic] {
        &self.author_diagnostics
    }
    pub fn external_conformance(&self) -> HtmlSrcsetExternalConformance {
        self.external
    }
    pub fn processing(&self) -> HtmlSrcsetProcessingState {
        self.processing
    }
    pub fn candidates(&self) -> Option<&[HtmlParsedImageCandidate]> {
        self.candidates.as_deref()
    }
    pub fn diagnostics(&self) -> &[HtmlSrcsetParseDiagnostic] {
        &self.diagnostics
    }
    pub fn context_author(&self) -> HtmlSrcsetContextAuthorState {
        self.context_author
    }
    pub fn context_violations(&self) -> &[HtmlSrcsetContextViolation] {
        &self.context_violations
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetInterpretationError {
    RawAdmission(HtmlValueAdmissionError),
    RepresentationExhausted,
}
impl fmt::Display for HtmlSrcsetInterpretationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML srcset interpretation failed: {self:?}")
    }
}
impl Error for HtmlSrcsetInterpretationError {}

#[derive(Clone, Copy)]
enum Role {
    Img,
    Source,
    Link,
}
impl Role {
    fn rule(self) -> RuleMetadata {
        let (attribute, rows, source, element) = match self {
            Self::Img => (
                KnownHtmlAttribute::Srcset,
                &[152][..],
                source!("attr-img-srcset"),
                KnownHtmlElement::Img,
            ),
            Self::Source => (
                KnownHtmlAttribute::Srcset,
                &[152][..],
                source!("attr-source-srcset"),
                KnownHtmlElement::Source,
            ),
            Self::Link => (
                KnownHtmlAttribute::Imagesrcset,
                &[74][..],
                source!("attr-link-imagesrcset"),
                KnownHtmlElement::Link,
            ),
        };
        let context = match element {
            KnownHtmlElement::Img => Context::Elements(&[KnownHtmlElement::Img]),
            KnownHtmlElement::Source => Context::Elements(&[KnownHtmlElement::Source]),
            _ => Context::Elements(&[KnownHtmlElement::Link]),
        };
        RuleMetadata {
            attribute,
            rows,
            source,
            context,
            vocabulary_source: None,
        }
    }
    fn matches(self, context: &HtmlSrcsetContext) -> bool {
        matches!(
            (self, context),
            (Self::Img, HtmlSrcsetContext::Img { .. })
                | (Self::Source, HtmlSrcsetContext::Source { .. })
                | (Self::Link, HtmlSrcsetContext::Link { .. })
        )
    }
}
fn attribute<'a, A: crate::HtmlAttributesInput + ?Sized>(
    attributes: &'a A,
    name: &str,
) -> Option<&'a A::Attribute> {
    find_attribute(attributes, &Namespace::none(), &DocumentString::from(name))
}

impl HtmlValueInterpreter {
    /// Interprets literal img candidates; surrounding author facts stay unchecked.
    pub fn img_srcset(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.srcset(Role::Img, element, raw, None)
    }
    /// Interprets literal source candidates without guessing picture ancestry.
    pub fn source_srcset(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.srcset(Role::Source, element, raw, None)
    }
    /// Interprets literal link candidates without executing preload or selection.
    pub fn link_imagesrcset(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.srcset(Role::Link, element, raw, None)
    }
    /// Context target content and surrounding facts come from one raw attribute set.
    pub fn img_srcset_with_context(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.context_srcset(Role::Img, element, context)
    }
    pub fn source_srcset_with_context(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.context_srcset(Role::Source, element, context)
    }
    pub fn link_imagesrcset_with_context(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.context_srcset(Role::Link, element, context)
    }
    fn context_srcset(
        &self,
        role: Role,
        element: &(impl crate::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        let name = match role {
            Role::Link => "imagesrcset",
            _ => "srcset",
        };
        self.srcset(
            role,
            element,
            attribute(context.attributes(), name),
            Some(context),
        )
    }
    fn srcset(
        &self,
        role: Role,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        context: Option<&HtmlSrcsetContext>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))
            .map_err(HtmlSrcsetInterpretationError::RawAdmission)?;
        // Surrounding raw values use the same per-value standalone admission.
        if let Some(context) = context {
            for attribute in context.attributes().iter() {
                self.admit_attribute_raw(Some(attribute.value()))
                    .map_err(HtmlSrcsetInterpretationError::RawAdmission)?;
            }
            if let HtmlSrcsetContext::Source {
                following_image: HtmlSrcsetFollowingImageObservation::Element { attributes, .. },
                ..
            } = context
            {
                for attribute in attributes_iter(attributes) {
                    self.admit_attribute_raw(Some(attribute.value()))
                        .map_err(HtmlSrcsetInterpretationError::RawAdmission)?;
                }
            }
        }
        let mut input = report(role.rule(), element, raw);
        if input.applicability == HtmlAttributeApplicability::Applicable
            && context.is_some_and(|context| !role.matches(context))
        {
            input.applicability = HtmlAttributeApplicability::DifferentContext;
        }
        let applied = input.applicability == HtmlAttributeApplicability::Applicable;
        let mut result = HtmlSrcsetReport {
            input,
            context: context.cloned(),
            author_fields: None,
            author: if applied {
                HtmlSrcsetAuthorState::Missing
            } else {
                HtmlSrcsetAuthorState::Unapplied
            },
            author_diagnostics: Vec::new(),
            external: if applied {
                HtmlSrcsetExternalConformance::NotRequired
            } else {
                HtmlSrcsetExternalConformance::Unapplied
            },
            processing: if applied {
                HtmlSrcsetProcessingState::MissingAttribute
            } else {
                HtmlSrcsetProcessingState::Unapplied
            },
            candidates: None,
            diagnostics: Vec::new(),
            context_author: if applied {
                HtmlSrcsetContextAuthorState::Unchecked
            } else {
                HtmlSrcsetContextAuthorState::Unapplied
            },
            context_violations: Vec::new(),
        };
        if !applied {
            return Ok(result);
        }
        let mut width_seen = false;
        let mut all_width = None;
        if let Some(raw) = raw {
            let parsed = parse_candidates(raw.value())?;
            width_seen = parsed.width_seen;
            all_width = parsed.all_width;
            result.author = if parsed.author_diagnostics.is_empty() {
                HtmlSrcsetAuthorState::HtmlConstraintsConforming
            } else {
                HtmlSrcsetAuthorState::Nonconforming
            };
            result.author_fields = parsed.author_fields;
            result.author_diagnostics = parsed.author_diagnostics;
            result.external = if parsed.lexical_candidates == 0 {
                HtmlSrcsetExternalConformance::NotRequired
            } else {
                HtmlSrcsetExternalConformance::Unchecked
            };
            result.processing = HtmlSrcsetProcessingState::Candidates;
            result.candidates = Some(parsed.candidates);
            result.diagnostics = parsed.diagnostics;
        }
        check_context(
            self,
            role,
            raw.is_some(),
            context,
            width_seen,
            all_width,
            &mut result,
        );
        Ok(result)
    }
}

fn check_context(
    interpreter: &HtmlValueInterpreter,
    role: Role,
    present: bool,
    context: Option<&HtmlSrcsetContext>,
    width_seen: bool,
    all_width: Option<bool>,
    result: &mut HtmlSrcsetReport,
) {
    let Some(context) = context else {
        result.context_author = if !present && matches!(role, Role::Link) {
            HtmlSrcsetContextAuthorState::NotRequired
        } else {
            HtmlSrcsetContextAuthorState::Unchecked
        };
        return;
    };
    let attributes = context.attributes();
    let mut unchecked = false;
    let mut required = false;
    let sizes_name = if matches!(role, Role::Link) {
        "imagesizes"
    } else {
        "sizes"
    };
    if present && attribute(attributes, sizes_name).is_some() {
        required = true;
        match all_width {
            Some(true) => {}
            Some(false) => result
                .context_violations
                .push(HtmlSrcsetContextViolation::WidthDescriptorsRequiredForSizes),
            None => unchecked = true,
        }
    }
    match context {
        HtmlSrcsetContext::Img { .. } => {
            if !present {
                required = true;
                if attribute(attributes, "src").is_none() {
                    result
                        .context_violations
                        .push(HtmlSrcsetContextViolation::SrcOrSrcsetRequired);
                }
            }
            if width_seen {
                required = true;
                if attribute(attributes, "sizes").is_none() {
                    result
                        .context_violations
                        .push(HtmlSrcsetContextViolation::SizesRequired);
                }
            }
        }
        HtmlSrcsetContext::Link { .. } => {
            if present {
                required = true;
                let preload = attribute(attributes, "rel").is_some_and(|rel| {
                    crate::tokens::split_space_tokens(rel.value())
                        .tokens()
                        .iter()
                        .any(|t| ascii_keyword(t.value(), "preload"))
                });
                if !preload {
                    result
                        .context_violations
                        .push(HtmlSrcsetContextViolation::PreloadRelRequired);
                }
                if !attribute(attributes, "as")
                    .is_some_and(|value| ascii_keyword(value.value(), "image"))
                {
                    result
                        .context_violations
                        .push(HtmlSrcsetContextViolation::ImageAsStateRequired);
                }
                if width_seen && attribute(attributes, "imagesizes").is_none() {
                    result
                        .context_violations
                        .push(HtmlSrcsetContextViolation::ImagesizesRequired);
                }
            }
        }
        HtmlSrcsetContext::Source {
            parent,
            following_image,
            ..
        } => match parent {
            HtmlSrcsetParentObservation::Unavailable => unchecked = true,
            HtmlSrcsetParentObservation::Element(name)
                if KnownHtmlElement::from_name(name) == Some(KnownHtmlElement::Picture) =>
            {
                required = true;
                if !present {
                    result
                        .context_violations
                        .push(HtmlSrcsetContextViolation::SourceSrcsetRequired);
                }
                if width_seen && attribute(attributes, "sizes").is_none() {
                    match following_image {
                        HtmlSrcsetFollowingImageObservation::Element { name, attributes }
                            if KnownHtmlElement::from_name(name) == Some(KnownHtmlElement::Img) =>
                        {
                            if !allows_auto_sizes(interpreter, name, attributes) {
                                result
                                    .context_violations
                                    .push(HtmlSrcsetContextViolation::SizesRequired);
                            }
                        }
                        _ => unchecked = true,
                    }
                }
            }
            _ if present => {
                required = true;
                result
                    .context_violations
                    .push(HtmlSrcsetContextViolation::SourceParentNotPicture);
            }
            _ => {}
        },
    }
    result.context_author = if !result.context_violations.is_empty() {
        HtmlSrcsetContextAuthorState::Nonconforming
    } else if unchecked {
        HtmlSrcsetContextAuthorState::Unchecked
    } else if required {
        HtmlSrcsetContextAuthorState::Conforming
    } else {
        HtmlSrcsetContextAuthorState::NotRequired
    };
}
fn allows_auto_sizes(
    interpreter: &HtmlValueInterpreter,
    name: &(impl crate::HtmlNameInput + ?Sized),
    attributes: &(impl crate::HtmlAttributesInput + ?Sized),
) -> bool {
    // Reuse the existing enum owner; no CSS sizes evaluation is required here.
    let loading = interpreter
        .loading(name, attribute(attributes, "loading"))
        .expect("surrounding raw values were admitted before context evaluation");
    if loading.intrinsic() != Some(&HtmlLoading::Lazy) {
        return false;
    }
    attribute(attributes, "sizes").is_some_and(|sizes| {
        ascii_keyword(sizes.value(), "auto")
            || sizes.value().code_units().get(..5).is_some_and(|units| {
                ascii_keyword(&DocumentString::from_utf16(units.to_vec()), "auto,")
            })
    })
}

#[derive(Clone, Debug)]
struct LexicalCandidate {
    range: HtmlSrcsetRawRange,
    url: HtmlSrcsetRawRange,
    descriptors: Vec<HtmlSrcsetRawRange>,
}
struct LexicalResult {
    candidates: Vec<LexicalCandidate>,
    diagnostics: Vec<HtmlSrcsetParseDiagnostic>,
    author_diagnostics: Vec<HtmlSrcsetAuthorDiagnostic>,
}
#[derive(Clone, Copy)]
enum DescriptorState {
    InDescriptor,
    InParens,
    AfterDescriptor,
}
fn range(start: usize, end: usize) -> HtmlSrcsetRawRange {
    HtmlSrcsetRawRange { start, end }
}
fn copied(raw: &DocumentString, range: HtmlSrcsetRawRange) -> DocumentString {
    DocumentString::from_utf16(raw.code_units()[range.start..range.end].to_vec())
}
fn author_diagnostic(
    violation: HtmlSrcsetAuthorViolation,
    range: HtmlSrcsetRawRange,
    candidate: Option<usize>,
) -> HtmlSrcsetAuthorDiagnostic {
    HtmlSrcsetAuthorDiagnostic {
        violation,
        range,
        candidate,
    }
}
fn parse_diagnostic(
    error: HtmlSrcsetParseError,
    range: HtmlSrcsetRawRange,
    candidate: Option<usize>,
) -> HtmlSrcsetParseDiagnostic {
    HtmlSrcsetParseDiagnostic {
        error,
        range,
        candidate,
    }
}

/// The literal source tokenizer's range-backed descriptors preserve all units.
fn tokenize(raw: &DocumentString) -> LexicalResult {
    let units = raw.code_units();
    let mut result = LexicalResult {
        candidates: Vec::new(),
        diagnostics: Vec::new(),
        author_diagnostics: Vec::new(),
    };
    let mut position = 0;
    let mut last_separator = None;
    loop {
        let splitting_start = position;
        let mut skipped_comma = false;
        while units
            .get(position)
            .is_some_and(|&u| crate::tokens::ascii_whitespace(u) || u == 44)
        {
            skipped_comma |= units[position] == 44;
            position += 1;
        }
        if skipped_comma {
            result.diagnostics.push(parse_diagnostic(
                HtmlSrcsetParseError::SkippedCommas,
                range(splitting_start, position),
                None,
            ));
            result.author_diagnostics.push(author_diagnostic(
                HtmlSrcsetAuthorViolation::MissingCandidate,
                range(splitting_start, position),
                None,
            ));
        }
        if position == units.len() {
            if let Some(separator) = last_separator {
                result.author_diagnostics.push(author_diagnostic(
                    HtmlSrcsetAuthorViolation::MissingCandidate,
                    range(separator, position),
                    None,
                ));
            }
            break;
        }
        let start = position;
        while units
            .get(position)
            .is_some_and(|&u| !crate::tokens::ascii_whitespace(u))
        {
            position += 1;
        }
        let original_url_end = position;
        let mut url_end = position;
        let mut descriptors = Vec::new();
        last_separator = None;
        if units[url_end - 1] == 44 {
            while url_end > start && units[url_end - 1] == 44 {
                url_end -= 1;
            }
            last_separator = Some(url_end);
            if original_url_end - url_end > 1 {
                let commas = range(url_end, original_url_end);
                result.diagnostics.push(parse_diagnostic(
                    HtmlSrcsetParseError::MultipleTrailingCommas,
                    commas,
                    Some(result.candidates.len()),
                ));
                result.author_diagnostics.push(author_diagnostic(
                    HtmlSrcsetAuthorViolation::MissingCandidate,
                    commas,
                    Some(result.candidates.len()),
                ));
            }
        } else {
            while units
                .get(position)
                .is_some_and(|&u| crate::tokens::ascii_whitespace(u))
            {
                position += 1;
            }
            let mut state = DescriptorState::InDescriptor;
            let mut descriptor_start = None;
            loop {
                let current = units.get(position).copied();
                match state {
                    DescriptorState::InDescriptor => match current {
                        None => {
                            if let Some(start) = descriptor_start {
                                descriptors.push(range(start, position));
                            }
                            break;
                        }
                        Some(44) => {
                            if let Some(start) = descriptor_start {
                                descriptors.push(range(start, position));
                            }
                            last_separator = Some(position);
                            position += 1;
                            break;
                        }
                        Some(u) if crate::tokens::ascii_whitespace(u) => {
                            if let Some(start) = descriptor_start.take() {
                                descriptors.push(range(start, position));
                            }
                            state = DescriptorState::AfterDescriptor;
                        }
                        Some(40) => {
                            descriptor_start.get_or_insert(position);
                            state = DescriptorState::InParens;
                        }
                        _ => {
                            descriptor_start.get_or_insert(position);
                        }
                    },
                    DescriptorState::InParens => match current {
                        None => {
                            descriptors.push(range(
                                descriptor_start.expect("opening paren starts a descriptor"),
                                position,
                            ));
                            break;
                        }
                        Some(41) => state = DescriptorState::InDescriptor,
                        _ => {}
                    },
                    DescriptorState::AfterDescriptor => match current {
                        None => break,
                        Some(u) if crate::tokens::ascii_whitespace(u) => {}
                        _ => {
                            state = DescriptorState::InDescriptor;
                            continue;
                        }
                    },
                }
                position += 1;
            }
        }
        result.candidates.push(LexicalCandidate {
            range: range(start, position),
            url: range(start, url_end),
            descriptors,
        });
    }
    if result.candidates.is_empty() {
        result.author_diagnostics.push(author_diagnostic(
            HtmlSrcsetAuthorViolation::EmptyList,
            range(0, units.len()),
            None,
        ));
    }
    result
}

struct ParsedResult {
    candidates: Vec<HtmlParsedImageCandidate>,
    diagnostics: Vec<HtmlSrcsetParseDiagnostic>,
    author_fields: Option<HtmlSrcsetValue>,
    author_diagnostics: Vec<HtmlSrcsetAuthorDiagnostic>,
    width_seen: bool,
    all_width: Option<bool>,
    lexical_candidates: usize,
}
fn parse_candidates(raw: &DocumentString) -> Result<ParsedResult, HtmlSrcsetInterpretationError> {
    let lexed = tokenize(raw);
    let lexical_candidates = lexed.candidates.len();
    let mut diagnostics = lexed.diagnostics;
    let mut author_diagnostics = lexed.author_diagnostics;
    let mut recovered = Vec::new();
    let mut authored = Vec::new();
    let mut author_indices = Vec::new();
    let mut width_seen = false;
    for (index, lexical) in lexed.candidates.iter().enumerate() {
        let url = copied(raw, lexical.url);
        let checked_url = match HtmlSrcsetUrlToken::new(url.clone()) {
            Ok(url) => Some(url),
            Err(_) => {
                author_diagnostics.push(author_diagnostic(
                    HtmlSrcsetAuthorViolation::InvalidUrlToken,
                    lexical.url,
                    Some(index),
                ));
                None
            }
        };
        let mut width = None;
        let mut density = None;
        let mut height = None;
        let mut failed = false;
        let mut overflow = false;
        let mut author_descriptor = if lexical.descriptors.is_empty() {
            Some(HtmlSrcsetDescriptor::Unspecified)
        } else {
            None
        };
        if lexical.descriptors.len() > 1 {
            author_diagnostics.push(author_diagnostic(
                HtmlSrcsetAuthorViolation::MultipleDescriptors,
                lexical.range,
                Some(index),
            ));
        }
        for &descriptor_range in &lexical.descriptors {
            let units = &raw.code_units()[descriptor_range.start..descriptor_range.end];
            let (suffix, prefix) = units
                .split_last()
                .expect("tokenizer emits nonempty descriptors");
            let prefix = DocumentString::from_utf16(prefix.to_vec());
            match *suffix {
                119 | 104
                    if !prefix.is_empty()
                        && prefix.code_units().iter().all(|u| (48..=57).contains(u)) =>
                {
                    let value = HtmlNonNegativeInteger::from_decimal_digits(&prefix)
                        .expect("whole digit grammar proved");
                    let is_width = *suffix == 119;
                    if is_width {
                        if width.is_some() || density.is_some() {
                            failed = true;
                        }
                    } else {
                        diagnostics.push(parse_diagnostic(
                            HtmlSrcsetParseError::FutureHeightDescriptor,
                            descriptor_range,
                            Some(index),
                        ));
                        author_diagnostics.push(author_diagnostic(
                            HtmlSrcsetAuthorViolation::FutureHeightDescriptor,
                            descriptor_range,
                            Some(index),
                        ));
                        if height.is_some() || density.is_some() {
                            failed = true;
                        }
                    }
                    if value.is_zero() {
                        failed = true;
                        author_diagnostics.push(author_diagnostic(
                            HtmlSrcsetAuthorViolation::NonpositiveDescriptor,
                            descriptor_range,
                            Some(index),
                        ));
                    } else if is_width {
                        width_seen = true;
                        let value = HtmlSrcsetWidth { value };
                        if lexical.descriptors.len() == 1 {
                            author_descriptor = Some(HtmlSrcsetDescriptor::Width(value.clone()));
                        }
                        width = Some(value);
                    } else {
                        height = Some(value);
                    }
                }
                120 => {
                    let key = match numeric::strict_scientific_decimal_key(&prefix) {
                        Ok(key) => key,
                        Err(ScientificDecimalKeyError::Syntax(_)) => {
                            failed = true;
                            author_diagnostics.push(author_diagnostic(
                                HtmlSrcsetAuthorViolation::InvalidDescriptor,
                                descriptor_range,
                                Some(index),
                            ));
                            continue;
                        }
                        Err(ScientificDecimalKeyError::RepresentationExhausted) => {
                            return Err(HtmlSrcsetInterpretationError::RepresentationExhausted);
                        }
                    };
                    if key.is_positive() {
                        if lexical.descriptors.len() == 1 {
                            author_descriptor =
                                Some(HtmlSrcsetDescriptor::Density(HtmlSrcsetDensity {
                                    syntax: prefix,
                                    key,
                                }));
                        }
                    } else {
                        author_diagnostics.push(author_diagnostic(
                            HtmlSrcsetAuthorViolation::NonpositiveDescriptor,
                            descriptor_range,
                            Some(index),
                        ));
                    }
                    if width.is_some() || density.is_some() || height.is_some() {
                        failed = true;
                    }
                    // Source recovery consumes the descriptor's valid numeric prefix.
                    match numeric::parse_floating_point_number(&copied(raw, descriptor_range)) {
                        Ok(value) if value.value() >= 0.0 => density = Some(value),
                        Ok(_) => failed = true,
                        Err(NumericParseError::FloatingPointOverflow) => {
                            diagnostics.push(parse_diagnostic(
                                HtmlSrcsetParseError::DensityConversionOverflow,
                                descriptor_range,
                                Some(index),
                            ));
                            failed = true;
                            overflow = true;
                        }
                        Err(_) => unreachable!("strict numeric guard proved recoverable prefix"),
                    }
                }
                _ => {
                    failed = true;
                    author_diagnostics.push(author_diagnostic(
                        HtmlSrcsetAuthorViolation::InvalidDescriptor,
                        descriptor_range,
                        Some(index),
                    ));
                }
            }
        }
        if height.is_some() && width.is_none() {
            failed = true;
        }
        if failed {
            if !overflow {
                diagnostics.push(parse_diagnostic(
                    HtmlSrcsetParseError::RejectedCandidate,
                    lexical.range,
                    Some(index),
                ));
            }
        } else {
            let descriptor = if let Some(width) = width {
                HtmlParsedSrcsetDescriptor::Width(width)
            } else if let Some(density) = density {
                HtmlParsedSrcsetDescriptor::Density(density)
            } else {
                HtmlParsedSrcsetDescriptor::Unspecified
            };
            recovered.push(HtmlParsedImageCandidate {
                url,
                descriptor,
                range: lexical.range,
                url_range: lexical.url,
            });
        }
        if let (Some(url), Some(descriptor)) = (checked_url, author_descriptor) {
            authored.push(HtmlSrcsetCandidate::new(url, descriptor));
            author_indices.push(index);
        }
    }
    for (index, mut violation) in list_violations(&authored) {
        match &mut violation {
            HtmlSrcsetAuthorViolation::DuplicateWidth { first, duplicate }
            | HtmlSrcsetAuthorViolation::DuplicateDensity { first, duplicate } => {
                *first = author_indices[*first];
                *duplicate = author_indices[*duplicate];
            }
            _ => {}
        }
        let candidate = author_indices[index];
        author_diagnostics.push(author_diagnostic(
            violation,
            lexed.candidates[candidate].range,
            Some(candidate),
        ));
    }
    // A known nonwidth candidate disproves the universal constraint even
    // when another candidate is malformed. Only the positive conclusion
    // requires complete candidate evidence.
    let all_width = if authored
        .iter()
        .any(|c| !matches!(c.descriptor, HtmlSrcsetDescriptor::Width(_)))
    {
        Some(false)
    } else {
        (authored.len() == lexical_candidates && lexical_candidates > 0).then_some(true)
    };
    let author_fields = if author_diagnostics.is_empty() {
        Some(HtmlSrcsetValue {
            candidates: authored,
        })
    } else {
        None
    };
    Ok(ParsedResult {
        candidates: recovered,
        diagnostics,
        author_fields,
        author_diagnostics,
        width_seen,
        all_width,
        lexical_candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_rewind_keeps_ranges_and_parenthesized_whitespace_without_copying_tokens() {
        let raw = DocumentString::from("a 2w \t(foo, bar) ,b 3x");
        let lexed = tokenize(&raw);
        assert_eq!(lexed.candidates.len(), 2);
        assert_eq!(
            copied(&raw, lexed.candidates[0].url),
            DocumentString::from("a")
        );
        let descriptors: Vec<_> = lexed.candidates[0]
            .descriptors
            .iter()
            .map(|&r| copied(&raw, r))
            .collect();
        assert_eq!(
            descriptors,
            [
                DocumentString::from("2w"),
                DocumentString::from("(foo, bar)")
            ]
        );
        assert_eq!(lexed.candidates[0].descriptors[0], range(2, 4));
        assert_eq!(lexed.candidates[0].descriptors[1], range(6, 16));
        assert_eq!(
            copied(&raw, lexed.candidates[1].url),
            DocumentString::from("b")
        );
        assert_eq!(
            copied(&raw, lexed.candidates[1].descriptors[0]),
            DocumentString::from("3x")
        );
        assert!(lexed.diagnostics.is_empty());
    }

    #[test]
    fn trailing_url_comma_branch_bypasses_descriptor_tokenization_and_reports_only_extra_commas() {
        let raw = DocumentString::from("a,,, 2x");
        let lexed = tokenize(&raw);
        assert_eq!(lexed.candidates[0].url, range(0, 1));
        assert!(lexed.candidates[0].descriptors.is_empty());
        assert_eq!(
            copied(&raw, lexed.candidates[1].url),
            DocumentString::from("2x")
        );
        assert_eq!(
            lexed.diagnostics,
            [parse_diagnostic(
                HtmlSrcsetParseError::MultipleTrailingCommas,
                range(1, 4),
                Some(0)
            )]
        );
        let parsed = parse_candidates(&raw).unwrap();
        assert_eq!(
            parsed
                .candidates
                .iter()
                .map(HtmlParsedImageCandidate::url)
                .collect::<Vec<_>>(),
            [&DocumentString::from("a"), &DocumentString::from("2x")]
        );
    }

    #[test]
    fn author_duplicate_indices_stay_lexical_when_an_earlier_candidate_is_invalid() {
        let parsed = parse_candidates(&DocumentString::from("bad 0w, a 1x, b 10e-1x")).unwrap();
        let duplicate = parsed
            .author_diagnostics
            .iter()
            .find(|diagnostic| {
                matches!(
                    diagnostic.violation,
                    HtmlSrcsetAuthorViolation::DuplicateDensity { .. }
                )
            })
            .unwrap();
        assert_eq!(
            duplicate.violation,
            HtmlSrcsetAuthorViolation::DuplicateDensity {
                first: 1,
                duplicate: 2
            }
        );
        assert_eq!(duplicate.candidate, Some(2));
        assert_eq!(
            parsed
                .candidates
                .iter()
                .map(HtmlParsedImageCandidate::url)
                .collect::<Vec<_>>(),
            [&DocumentString::from("a"), &DocumentString::from("b")]
        );
    }

    #[test]
    fn height_then_width_is_admitted_without_exposing_height_as_a_final_descriptor() {
        let parsed = parse_candidates(&DocumentString::from("a 123h 456w")).unwrap();
        assert_eq!(
            parsed.diagnostics,
            [parse_diagnostic(
                HtmlSrcsetParseError::FutureHeightDescriptor,
                range(2, 6),
                Some(0)
            )]
        );
        assert_eq!(
            parsed.candidates[0].descriptor(),
            &HtmlParsedSrcsetDescriptor::Width(
                HtmlSrcsetWidth::new(HtmlNonNegativeInteger::from(456)).unwrap()
            )
        );
        assert!(parsed.author_fields.is_none());
        assert!(
            parsed
                .author_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.violation
                    == HtmlSrcsetAuthorViolation::FutureHeightDescriptor)
        );
    }
}
