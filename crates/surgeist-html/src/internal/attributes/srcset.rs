//! HTML srcset candidate constraints and literal recovery, with symbolic URLs.
//! Sizes CSS, URL resolution, source-set normalization and image selection are
//! separate operations. WHATWG and selected WebKit attribution is in the notice.

use super::*;
use crate::internal::microsyntax::numeric::{
    self, HtmlBinary64Rounding, HtmlScientificDecimalKey, ScientificDecimalKeyError,
};
use crate::{
    internal::microsyntax::numeric::HtmlFloatingPointNumber,
    internal::microsyntax::numeric::HtmlNonNegativeInteger, internal::values::OrderedAttributes,
};
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
        if units
            .iter()
            .any(|&u| crate::internal::microsyntax::tokens::is_ascii_whitespace(u))
        {
            return Err(HtmlSrcsetUrlTokenError::AsciiWhitespace);
        }
        if units.first() == Some(&44) || units.last() == Some(&44) {
            return Err(HtmlSrcsetUrlTokenError::EdgeComma);
        }
        Ok(Self { value })
    }
    /// The exact token text in UTF-16 code units.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}
/// Why text cannot be a srcset image candidate URL token. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSrcsetUrlTokenError {
    /// The text is empty, but the standard requires a nonempty URL.
    Empty,
    /// The text contains an ASCII whitespace code unit, which would end the token.
    AsciiWhitespace,
    /// The text starts or ends with U+002C COMMA, which the standard forbids for candidate URLs
    /// (HTML877d #srcset-attributes).
    EdgeComma,
}
impl fmt::Display for HtmlSrcsetUrlTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "srcset image candidate URL is empty",
            Self::AsciiWhitespace => "srcset image candidate URL contains ASCII whitespace",
            Self::EdgeComma => "srcset image candidate URL starts or ends with a comma",
        })
    }
}
impl Error for HtmlSrcsetUrlTokenError {}

/// Positive exact width; no machine-integer magnitude cap is imposed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetWidth {
    value: HtmlNonNegativeInteger,
}
impl HtmlSrcsetWidth {
    /// Accepts a width greater than zero.
    ///
    /// Returns [`HtmlSrcsetWidthError`] when `value` is zero.
    pub fn new(value: HtmlNonNegativeInteger) -> Result<Self, HtmlSrcsetWidthError> {
        if value.is_zero() {
            Err(HtmlSrcsetWidthError)
        } else {
            Ok(Self { value })
        }
    }
    /// The exact width, always greater than zero.
    pub fn value(&self) -> &HtmlNonNegativeInteger {
        &self.value
    }
}
/// A width descriptor must be greater than zero (HTML877d #srcset-attributes), and zero was
/// supplied.
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
#[derive(Clone, Debug)]
pub struct HtmlSrcsetDensity {
    syntax: DocumentString,
    key: HtmlScientificDecimalKey,
    /// The binary64 rounding of the spelling, checked together with the key. It is a function
    /// of the spelling, so equality does not compare it.
    rounding: HtmlBinary64Rounding,
}
impl PartialEq for HtmlSrcsetDensity {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.key == other.key
    }
}
impl Eq for HtmlSrcsetDensity {}
impl HtmlSrcsetDensity {
    /// Checks complete author grammar and mathematical positivity before rounding.
    /// Uses ordinary native allocations, not a resource/cancellation guarantee.
    pub fn from_decimal_syntax(
        syntax: impl Into<DocumentString>,
    ) -> Result<Self, HtmlSrcsetDensityError> {
        let syntax = syntax.into();
        let decimal =
            numeric::strict_decimal_units(syntax.code_units()).map_err(|error| match error {
                ScientificDecimalKeyError::Syntax(_) => HtmlSrcsetDensityError::InvalidSyntax,
                ScientificDecimalKeyError::RepresentationExhausted => {
                    HtmlSrcsetDensityError::RepresentationExhausted
                }
            })?;
        if !decimal.key.is_positive() {
            return Err(HtmlSrcsetDensityError::Nonpositive);
        }
        Ok(Self {
            syntax,
            key: decimal.key,
            rounding: decimal.rounding,
        })
    }
    /// The exact decimal spelling that was checked, in UTF-16 code units.
    pub fn syntax(&self) -> &DocumentString {
        &self.syntax
    }
    /// Source finite binary64 rounding; underflow returns canonical zero.
    pub fn to_floating_point_number(
        &self,
    ) -> Result<HtmlFloatingPointNumber, HtmlSrcsetDensityProjectionError> {
        match self.rounding {
            HtmlBinary64Rounding::Finite(value) => Ok(value),
            HtmlBinary64Rounding::Overflow => Err(HtmlSrcsetDensityProjectionError),
        }
    }
}
impl TryFrom<HtmlFloatingPointNumber> for HtmlSrcsetDensity {
    type Error = HtmlSrcsetDensityError;
    fn try_from(value: HtmlFloatingPointNumber) -> Result<Self, Self::Error> {
        Self::from_decimal_syntax(value.to_plain_decimal_string())
    }
}
/// Why text cannot be a srcset pixel density. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSrcsetDensityError {
    /// The text is not a valid floating-point number.
    InvalidSyntax,
    /// The text is a valid number that is not greater than zero.
    Nonpositive,
    /// The exact decimal key of the text exceeds the checked representation bounds on this
    /// platform.
    RepresentationExhausted,
}
impl fmt::Display for HtmlSrcsetDensityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSyntax => "srcset density is not a valid floating-point number",
            Self::Nonpositive => "srcset density is not greater than zero",
            Self::RepresentationExhausted => {
                "srcset density size cannot be represented on this platform"
            }
        })
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

/// The optional descriptor of one image candidate string (HTML877d #srcset-attributes).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetDescriptor {
    /// No descriptor is written. For uniqueness checks the standard treats this as a `1x` density.
    Unspecified,
    /// A width descriptor, written as a positive integer followed by `w`.
    Width(HtmlSrcsetWidth),
    /// A pixel density descriptor, written as a positive number followed by `x`.
    Density(HtmlSrcsetDensity),
}
/// One image candidate: a URL token and its descriptor. Construction does not check the list-level
/// rules, which [`HtmlSrcsetValue::new`] enforces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetCandidate {
    url: HtmlSrcsetUrlToken,
    descriptor: HtmlSrcsetDescriptor,
}
impl HtmlSrcsetCandidate {
    /// Pairs a URL token with a descriptor.
    pub fn new(url: HtmlSrcsetUrlToken, descriptor: HtmlSrcsetDescriptor) -> Self {
        Self { url, descriptor }
    }
    /// The candidate's URL token.
    pub fn url(&self) -> &HtmlSrcsetUrlToken {
        &self.url
    }
    /// The candidate's descriptor.
    pub fn descriptor(&self) -> &HtmlSrcsetDescriptor {
        &self.descriptor
    }
}

/// Established HTML list constraints; external URL/resource conformance is separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSrcsetAuthorViolation {
    /// The list has no image candidate strings, but a present `srcset` must have at least one.
    EmptyList,
    /// A comma appears without an image candidate string on one side of it.
    MissingCandidate,
    /// The URL part of a candidate is not a valid URL token.
    InvalidUrlToken,
    /// A descriptor is neither a valid width (`w`) nor a valid density (`x`) descriptor.
    InvalidDescriptor,
    /// A width or density descriptor is valid in form but is zero or negative.
    NonpositiveDescriptor,
    /// A candidate carries more than one descriptor.
    MultipleDescriptors,
    /// A candidate uses the `h` height descriptor, which the standard reserves and does not accept.
    FutureHeightDescriptor,
    /// Two candidates have the same width descriptor value.
    DuplicateWidth {
        /// The index of the earlier candidate with this width.
        first: usize,
        /// The index of the later candidate that repeats it.
        duplicate: usize,
    },
    /// Two candidates have the same density; a candidate without a descriptor counts as `1x`.
    DuplicateDensity {
        /// The index of the earlier candidate with this density.
        first: usize,
        /// The index of the later candidate that repeats it.
        duplicate: usize,
    },
    /// Some candidates have a width descriptor and others do not, but the standard requires all or
    /// none.
    MixedWidthDescriptors,
}
impl fmt::Display for HtmlSrcsetAuthorViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyList => f.write_str("srcset has no image candidate strings"),
            Self::MissingCandidate => {
                f.write_str("srcset has a comma without an image candidate string")
            }
            Self::InvalidUrlToken => {
                f.write_str("srcset image candidate URL is not a valid URL token")
            }
            Self::InvalidDescriptor => {
                f.write_str("srcset image candidate descriptor is not a valid width or density")
            }
            Self::NonpositiveDescriptor => {
                f.write_str("srcset image candidate descriptor is not greater than zero")
            }
            Self::MultipleDescriptors => {
                f.write_str("srcset image candidate has more than one descriptor")
            }
            Self::FutureHeightDescriptor => {
                f.write_str("srcset image candidate uses the unsupported height descriptor")
            }
            Self::DuplicateWidth { first, duplicate } => write!(
                f,
                "srcset image candidate {duplicate} repeats the width of candidate {first}"
            ),
            Self::DuplicateDensity { first, duplicate } => write!(
                f,
                "srcset image candidate {duplicate} repeats the density of candidate {first}"
            ),
            Self::MixedWidthDescriptors => f.write_str(
                "srcset has a candidate without a width descriptor beside width descriptors",
            ),
        }
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
    /// Builds a list from candidates.
    ///
    /// Returns the first author violation: `EmptyList` for no candidates, otherwise the earliest
    /// violation of the standard's uniqueness and width-descriptor rules.
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
    /// The candidates in authored order.
    pub fn candidates(&self) -> &[HtmlSrcsetCandidate] {
        &self.candidates
    }
    /// Unresolved symbolic URL/resource conformance, not HTML grammar failure.
    pub fn external_conformance(&self) -> HtmlSrcsetExternalConformance {
        HtmlSrcsetExternalConformance::Unchecked
    }
    /// Serializes the list as `url`, then an optional ` 123w` or ` 1.5x` descriptor, with
    /// candidates separated by a comma and a space. Densities keep their authored spelling.
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
    /// Emits a no-namespace `srcset` attribute carrying the serialization.
    pub fn into_srcset_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Srcset.qualified_name(),
            self.as_document_string(),
        )
    }
    /// Emits a no-namespace `imagesrcset` attribute carrying the serialization.
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
    let one = HtmlScientificDecimalKey::one();
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
    /// The offset of the first UTF-16 code unit of the range.
    pub fn start(self) -> usize {
        self.start
    }
    /// The offset one past the last UTF-16 code unit of the range.
    pub fn end(self) -> usize {
        self.end
    }
}
/// The descriptor of a candidate recovered by the standard's parsing algorithm (HTML877d
/// #parsing-a-srcset-attribute).
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlParsedSrcsetDescriptor {
    /// No descriptor was recovered.
    Unspecified,
    /// A width descriptor was recovered; it is greater than zero.
    Width(HtmlSrcsetWidth),
    /// A pixel density descriptor was recovered, as the finite binary64 value the standard's
    /// number-parsing rules give.
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
    /// The recovered URL in UTF-16 code units, with removed trailing commas excluded.
    pub fn url(&self) -> &DocumentString {
        &self.url
    }
    /// The recovered descriptor.
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
/// A parse error that the standard's algorithm for parsing a srcset attribute reports (HTML877d
/// #parsing-a-srcset-attribute), plus one recovery rule for density overflow. The enum is
/// `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSrcsetParseError {
    /// The splitting loop that precedes a candidate collected at least one U+002C COMMA among the
    /// ASCII whitespace it skips. Any collected comma is an error, so a single comma at the
    /// start of the list (`,a`) or set apart from the previous candidate by whitespace triggers
    /// it as well as several in a row.
    SkippedCommas,
    /// A candidate URL is followed by more than one trailing comma, all of which are removed.
    MultipleTrailingCommas,
    /// A candidate uses the `h` future-compatibility descriptor.
    FutureHeightDescriptor,
    /// A candidate is rejected and ignored because its descriptors are invalid, repeated or
    /// conflicting.
    RejectedCandidate,
    /// A density descriptor is a valid number that overflows finite binary64, so the candidate is
    /// dropped. This replaces `RejectedCandidate` for that candidate.
    DensityConversionOverflow,
}
impl fmt::Display for HtmlSrcsetParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SkippedCommas => "srcset has a comma where an image candidate was expected",
            Self::MultipleTrailingCommas => {
                "srcset image candidate URL is followed by more than one trailing comma"
            }
            Self::FutureHeightDescriptor => {
                "srcset image candidate uses the unsupported height descriptor"
            }
            Self::RejectedCandidate => "srcset image candidate is rejected and ignored",
            Self::DensityConversionOverflow => "srcset density overflows the finite binary64 range",
        })
    }
}
impl Error for HtmlSrcsetParseError {}
/// A recoverable parse error with the raw range where it occurred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetParseDiagnostic {
    error: HtmlSrcsetParseError,
    range: HtmlSrcsetRawRange,
    candidate: Option<usize>,
}
impl HtmlSrcsetParseDiagnostic {
    /// Which parse error occurred.
    pub fn error(&self) -> HtmlSrcsetParseError {
        self.error
    }
    /// The range of the offending text in the raw value.
    pub fn range(&self) -> HtmlSrcsetRawRange {
        self.range
    }
    /// Index among all lexical candidates, including rejected candidates.
    pub fn candidate_index(&self) -> Option<usize> {
        self.candidate
    }
}
/// A violation of the standard's srcset authoring requirements with its location. A violation does
/// not stop literal recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSrcsetAuthorDiagnostic {
    violation: HtmlSrcsetAuthorViolation,
    range: HtmlSrcsetRawRange,
    candidate: Option<usize>,
}
impl HtmlSrcsetAuthorDiagnostic {
    /// Which authoring requirement is violated.
    pub fn violation(&self) -> HtmlSrcsetAuthorViolation {
        self.violation
    }
    /// The range of the offending text in the raw value.
    pub fn range(&self) -> HtmlSrcsetRawRange {
        self.range
    }
    /// The index of the candidate involved among all lexical candidates, including rejected ones.
    /// It is `None` for `EmptyList` and for a `MissingCandidate` that reports commas skipped
    /// before a candidate or a separator that ends the list. It is `Some` for every other
    /// violation, including the `MissingCandidate` for several trailing commas after a URL,
    /// which carries the index that URL's candidate has.
    pub fn candidate_index(&self) -> Option<usize> {
        self.candidate
    }
}
/// The author verdict for a `srcset` or `imagesrcset` value on its own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetAuthorState {
    /// The attribute is absent.
    Missing,
    /// The attribute is present and satisfies the HTML list constraints. URL and resource
    /// requirements stay unchecked.
    HtmlConstraintsConforming,
    /// The attribute is present and has at least one author diagnostic.
    Nonconforming,
    /// The rule does not apply to the supplied element.
    Unapplied,
}
/// Whether literal candidate recovery ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetProcessingState {
    /// The rule applies but the attribute is absent, so there are no candidates.
    MissingAttribute,
    /// The rule applies and the attribute is present; the report carries the recovered candidates,
    /// possibly none.
    Candidates,
    /// The rule does not apply to the supplied element.
    Unapplied,
}
/// Whether URL and resource requirements that this crate does not validate are relevant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetExternalConformance {
    /// The attribute is absent or has no candidates, so there is nothing to check.
    NotRequired,
    /// At least one candidate exists, and its URL and referenced resource requirements stay
    /// unchecked.
    Unchecked,
    /// The rule does not apply to the supplied element.
    Unapplied,
}

/// Actual raw observations supplied by an adapter; no topology is inferred.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetParentObservation {
    /// The adapter did not observe the parent, so picture ancestry is unknown.
    Unavailable,
    /// The adapter observed that the element has no parent.
    Absent,
    /// The adapter observed that the parent is this element.
    Element(QualifiedName),
}
/// Actual following sibling image observation, distinct from unavailable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetFollowingImageObservation {
    /// The adapter did not observe the following sibling, so image requirements that depend on it
    /// are unknown.
    Unavailable,
    /// The adapter observed that no following image element exists.
    Absent,
    /// The adapter observed this following element.
    Element {
        /// The following element's name.
        name: QualifiedName,
        /// The following element's attributes in order.
        attributes: OrderedAttributes,
    },
}
/// Surrounding raw facts; these are observations, not a qualified semantic proof.
/// Target raw is derived from the same attributes to avoid contradictory inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetContext {
    /// The context of an `img` element.
    Img {
        /// The `img` element's attributes in order. The `srcset`, `src` and `sizes` values come
        /// from here.
        attributes: OrderedAttributes,
    },
    /// The context of a `source` element.
    Source {
        /// The `source` element's attributes in order. The `srcset` and `sizes` values come from
        /// here.
        attributes: OrderedAttributes,
        /// What the adapter observed about the parent element.
        parent: HtmlSrcsetParentObservation,
        /// What the adapter observed about the following sibling image, used for the `sizes="auto"`
        /// allowance.
        following_image: HtmlSrcsetFollowingImageObservation,
    },
    /// The context of a `link` element.
    Link {
        /// The `link` element's attributes in order. The `imagesrcset`, `imagesizes`, `rel` and
        /// `as` values come from here.
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
/// A requirement that surrounding attributes or elements place on a `srcset` or `imagesrcset`
/// (HTML877d #srcset-attributes for `img` and `source`, HTML877d #attr-link-imagesrcset and
/// HTML877d #attr-link-imagesizes for `link`). The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSrcsetContextViolation {
    /// An `img` element has neither `src` nor `srcset`.
    SrcOrSrcsetRequired,
    /// A `source` element with `srcset` has a parent that is not a `picture` element.
    SourceParentNotPicture,
    /// A `source` element in a `picture` lacks `srcset`.
    SourceSrcsetRequired,
    /// A `srcset` with width descriptors lacks a `sizes` attribute.
    SizesRequired,
    /// A `sizes` attribute, or an `imagesizes` attribute on a `link`, is present but not every
    /// candidate has a width descriptor.
    WidthDescriptorsRequiredForSizes,
    /// An `imagesrcset` with width descriptors lacks an `imagesizes` attribute.
    ImagesizesRequired,
    /// An `imagesrcset` is present on a `link` whose `rel` lacks the `preload` keyword.
    PreloadRelRequired,
    /// An `imagesrcset` is present on a `link` whose `as` attribute is not the `image` keyword.
    ImageAsStateRequired,
}
impl fmt::Display for HtmlSrcsetContextViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SrcOrSrcsetRequired => "img element requires a src or srcset attribute",
            Self::SourceParentNotPicture => {
                "source element with a srcset attribute requires a picture parent"
            }
            Self::SourceSrcsetRequired => "source element in a picture requires a srcset attribute",
            Self::SizesRequired => "srcset with width descriptors requires a sizes attribute",
            Self::WidthDescriptorsRequiredForSizes => {
                "sizes attribute requires every srcset candidate to use a width descriptor"
            }
            Self::ImagesizesRequired => {
                "imagesrcset with width descriptors requires an imagesizes attribute"
            }
            Self::PreloadRelRequired => "imagesrcset requires a preload link relationship",
            Self::ImageAsStateRequired => "imagesrcset requires the image destination",
        })
    }
}
impl Error for HtmlSrcsetContextViolation {}
/// The author verdict for the requirements that surrounding attributes and elements place on the
/// value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSrcsetContextAuthorState {
    /// No requirement applies in the supplied context.
    NotRequired,
    /// Some applicable requirement cannot be decided, for example because the `sizes` grammar or
    /// the parent was not checked.
    Unchecked,
    /// Every applicable requirement was decided and holds.
    Conforming,
    /// At least one requirement is violated; `context_violations` lists them.
    Nonconforming,
    /// The rule does not apply to the supplied element.
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
    /// The exact element name that was supplied, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        self.input.element()
    }
    /// The exact supplied attribute (name, prefix and value), or `None` when it was absent.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.input.raw_attribute()
    }
    /// The exact raw value in UTF-16 code units, distinguishing an absent attribute from an empty
    /// one.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.input.raw()
    }
    /// Whether the attribute was absent, present and empty, or present and nonempty.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.input.observation()
    }
    /// Whether the rule applies to the supplied element and context.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.input.applicability()
    }
    /// The standard attribute this report is about, `srcset` or `imagesrcset`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.input.attribute()
    }
    /// The rows of the pinned specification's attribute index that this rule reproduces.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.input.inventory_rows()
    }
    /// The URL of the attribute's element-specific definition in the pinned HTML Standard snapshot.
    pub fn source_clause(&self) -> &'static str {
        self.input.source_clause()
    }
    /// The URL of the pinned standard's srcset grammar.
    pub fn grammar_source_clause(&self) -> &'static str {
        GRAMMAR
    }
    /// The URL of the pinned standard's algorithm for parsing a srcset attribute.
    pub fn processing_source_clause(&self) -> &'static str {
        PROCESSING
    }
    /// The URL of the WebKit source that this crate follows when a density overflows binary64. The
    /// pinned standard does not define that recovery.
    pub fn overflow_recovery_source_clause(&self) -> &'static str {
        OVERFLOW_SOURCE
    }
    /// The length of the raw value in UTF-16 code units, as admitted against the per-value cap.
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    /// The supplied surrounding facts, or `None` when the report was made without a context.
    pub fn context(&self) -> Option<&HtmlSrcsetContext> {
        self.context.as_ref()
    }
    /// The candidate list as a checked native value. It is `Some` only when the attribute is
    /// present and has no author diagnostic.
    pub fn author_fields(&self) -> Option<&HtmlSrcsetValue> {
        self.author_fields.as_ref()
    }
    /// The author verdict for the value alone, without the surrounding context.
    pub fn author(&self) -> HtmlSrcsetAuthorState {
        self.author
    }
    /// Every author diagnostic in the order found.
    pub fn author_diagnostics(&self) -> &[HtmlSrcsetAuthorDiagnostic] {
        &self.author_diagnostics
    }
    /// Whether URL and resource requirements remain unchecked.
    pub fn external_conformance(&self) -> HtmlSrcsetExternalConformance {
        self.external
    }
    /// Whether literal candidate recovery ran for the supplied element.
    pub fn processing(&self) -> HtmlSrcsetProcessingState {
        self.processing
    }
    /// The candidates recovered by the standard's parsing algorithm, or `None` when the attribute
    /// is absent or the rule does not apply. Rejected candidates are absent.
    pub fn candidates(&self) -> Option<&[HtmlParsedImageCandidate]> {
        self.candidates.as_deref()
    }
    /// The parse errors the recovery reported, in the order found.
    pub fn diagnostics(&self) -> &[HtmlSrcsetParseDiagnostic] {
        &self.diagnostics
    }
    /// The author verdict for the requirements that surrounding attributes place on the value.
    pub fn context_author(&self) -> HtmlSrcsetContextAuthorState {
        self.context_author
    }
    /// The surrounding requirements that are violated.
    pub fn context_violations(&self) -> &[HtmlSrcsetContextViolation] {
        &self.context_violations
    }
}
/// Why srcset interpretation could not produce a report. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSrcsetInterpretationError {
    /// The raw value or a surrounding raw attribute that a context rule reads exceeded the
    /// per-value admission cap; the payload is that refusal.
    RawAdmission(HtmlValueAdmissionError),
    /// An internal size cannot be represented on this platform.
    RepresentationExhausted,
}
impl fmt::Display for HtmlSrcsetInterpretationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RawAdmission(_) => "srcset value was refused by raw-value admission",
            Self::RepresentationExhausted => {
                "srcset interpretation size cannot be represented on this platform"
            }
        })
    }
}
impl Error for HtmlSrcsetInterpretationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RawAdmission(error) => Some(error),
            Self::RepresentationExhausted => None,
        }
    }
}
impl From<HtmlValueAdmissionError> for HtmlSrcsetInterpretationError {
    fn from(error: HtmlValueAdmissionError) -> Self {
        Self::RawAdmission(error)
    }
}

/// The attributes of a source's following image that the sizes rule reads.
const FOLLOWING_IMAGE_ATTRIBUTES: [&str; 2] = ["loading", "sizes"];

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
    /// The attributes of the context, besides the target value, that the context rules read:
    /// the sizes list, `src` for an image, and `rel` and `as` for a link.
    fn context_attributes(self) -> &'static [&'static str] {
        match self {
            Self::Img => &["sizes", "src"],
            Self::Source => &["sizes"],
            Self::Link => &["imagesizes", "rel", "as"],
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
fn attribute<'a, A: crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized>(
    attributes: &'a A,
    name: &str,
) -> Option<A::Attribute<'a>> {
    find_attribute(attributes, &Namespace::none(), &DocumentString::from(name))
}

impl HtmlValueInterpreter {
    /// Interprets literal img candidates; surrounding author facts stay unchecked.
    pub fn img_srcset(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.srcset(Role::Img, element, raw, None)
    }
    /// Interprets literal source candidates without guessing picture ancestry.
    pub fn source_srcset(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.srcset(Role::Source, element, raw, None)
    }
    /// Interprets literal link candidates without executing preload or selection.
    pub fn link_imagesrcset(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.srcset(Role::Link, element, raw, None)
    }
    /// Context target content and surrounding facts come from one raw attribute set. The target
    /// value, `sizes` and `src` obey the per-value cap; other attributes are not read or admitted.
    pub fn img_srcset_with_context(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.context_srcset(Role::Img, element, context)
    }
    /// Interprets a `source` element's `srcset` together with its attributes, parent and following
    /// image. The target value, `sizes` and the following image's `loading` and `sizes` obey the
    /// same per-value cap; other attributes are not read or admitted.
    pub fn source_srcset_with_context(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.context_srcset(Role::Source, element, context)
    }
    /// Interprets a `link` element's `imagesrcset` together with its attributes. The target value,
    /// `imagesizes`, `rel` and `as` obey the same per-value cap; other attributes are not read or
    /// admitted.
    pub fn link_imagesrcset_with_context(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.context_srcset(Role::Link, element, context)
    }
    fn context_srcset(
        &self,
        role: Role,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        context: &HtmlSrcsetContext,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        let name = match role {
            Role::Link => "imagesrcset",
            _ => "srcset",
        };
        self.srcset(
            role,
            element,
            attribute(context.attributes(), name).as_ref(),
            Some(context),
        )
    }
    /// Admits, in their order in `attributes`, the values of the attributes named in `read`.
    fn admit_read_attributes(
        &self,
        read: &[&str],
        attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
    ) -> Result<(), HtmlValueAdmissionError> {
        for attribute in attributes_iter(attributes) {
            let name = attribute.name();
            if name.namespace_uri().is_none()
                && read.iter().any(|read| {
                    name.local_name()
                        .code_units()
                        .iter()
                        .copied()
                        .eq(read.encode_utf16())
                })
            {
                self.admit_attribute_raw(Some(attribute.value()))?;
            }
        }
        Ok(())
    }
    fn srcset(
        &self,
        role: Role,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: Option<&HtmlSrcsetContext>,
    ) -> Result<HtmlSrcsetReport, HtmlSrcsetInterpretationError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        // The surrounding raw values that the rules read use the same per-value standalone
        // admission, in the order of the attributes of the context and then of the following
        // image. Other attributes are never read, so they are skipped.
        if let Some(context) = context {
            self.admit_read_attributes(role.context_attributes(), context.attributes())?;
            if let HtmlSrcsetContext::Source {
                following_image: HtmlSrcsetFollowingImageObservation::Element { attributes, .. },
                ..
            } = context
            {
                self.admit_read_attributes(&FOLLOWING_IMAGE_ATTRIBUTES, attributes)?;
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
        )?;
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
) -> Result<(), HtmlSrcsetInterpretationError> {
    let Some(context) = context else {
        result.context_author = if !present && matches!(role, Role::Link) {
            HtmlSrcsetContextAuthorState::NotRequired
        } else {
            HtmlSrcsetContextAuthorState::Unchecked
        };
        return Ok(());
    };
    let attributes = context.attributes();
    let mut unchecked = false;
    let mut required = false;
    let sizes_name = if matches!(role, Role::Link) {
        "imagesizes"
    } else {
        "sizes"
    };
    if attribute(attributes, sizes_name).is_some() {
        // A present value must be a valid source size list, a grammar no owner
        // validates yet; it can never be certified here, even when empty.
        unchecked = true;
    }
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
                    crate::internal::microsyntax::tokens::split_space_tokens(rel.value())
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
                            if !allows_auto_sizes(interpreter, name, attributes)? {
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
    Ok(())
}
fn allows_auto_sizes(
    interpreter: &HtmlValueInterpreter,
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
    attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
) -> Result<bool, HtmlValueAdmissionError> {
    // Reuse the existing enum owner; no CSS sizes evaluation is required here.
    let loading = interpreter.loading(name, attribute(attributes, "loading").as_ref())?;
    if loading.intrinsic() != Some(&HtmlLoading::Lazy) {
        return Ok(false);
    }
    Ok(attribute(attributes, "sizes").is_some_and(|sizes| {
        ascii_keyword(sizes.value(), "auto")
            || sizes.value().code_units().get(..5).is_some_and(|units| {
                ascii_keyword(&DocumentString::from_utf16(units.to_vec()), "auto,")
            })
    }))
}

/// One descriptor token: its whole range, the range before its last unit, and that last unit.
/// The tokenizer builds it from at least one consumed unit, so the last unit always exists.
#[derive(Clone, Copy, Debug)]
struct LexicalDescriptor {
    range: HtmlSrcsetRawRange,
    prefix: HtmlSrcsetRawRange,
    suffix: u16,
}
/// A descriptor token under construction: the offset of its first unit and of its last
/// consumed unit, and that last unit.
#[derive(Clone, Copy, Debug)]
struct DescriptorSpan {
    start: usize,
    last: usize,
    suffix: u16,
}
impl DescriptorSpan {
    fn begin(position: usize, unit: u16) -> Self {
        Self {
            start: position,
            last: position,
            suffix: unit,
        }
    }
    fn extended(self, position: usize, unit: u16) -> Self {
        Self {
            last: position,
            suffix: unit,
            ..self
        }
    }
    fn consume(span: Option<Self>, position: usize, unit: u16) -> Self {
        span.map_or_else(
            || Self::begin(position, unit),
            |span| span.extended(position, unit),
        )
    }
    /// The finished token, ending where the tokenizer stopped consuming it.
    fn finish(self, end: usize) -> LexicalDescriptor {
        LexicalDescriptor {
            range: range(self.start, end),
            prefix: range(self.start, self.last),
            suffix: self.suffix,
        }
    }
}
#[derive(Clone, Debug)]
struct LexicalCandidate {
    range: HtmlSrcsetRawRange,
    url: HtmlSrcsetRawRange,
    descriptors: Vec<LexicalDescriptor>,
}
struct LexicalResult {
    candidates: Vec<LexicalCandidate>,
    diagnostics: Vec<HtmlSrcsetParseDiagnostic>,
    author_diagnostics: Vec<HtmlSrcsetAuthorDiagnostic>,
}
#[derive(Clone, Copy)]
enum DescriptorState {
    /// Between the units of a descriptor, with the descriptor so far, if any unit was consumed.
    InDescriptor(Option<DescriptorSpan>),
    /// Inside parentheses, which always belong to a descriptor that has begun.
    InParens(DescriptorSpan),
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
        while units.get(position).is_some_and(|&u| {
            crate::internal::microsyntax::tokens::is_ascii_whitespace(u) || u == 44
        }) {
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
            .is_some_and(|&u| !crate::internal::microsyntax::tokens::is_ascii_whitespace(u))
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
                .is_some_and(|&u| crate::internal::microsyntax::tokens::is_ascii_whitespace(u))
            {
                position += 1;
            }
            let mut state = DescriptorState::InDescriptor(None);
            loop {
                let current = units.get(position).copied();
                match state {
                    DescriptorState::InDescriptor(span) => match current {
                        None => {
                            if let Some(span) = span {
                                descriptors.push(span.finish(position));
                            }
                            break;
                        }
                        Some(44) => {
                            if let Some(span) = span {
                                descriptors.push(span.finish(position));
                            }
                            last_separator = Some(position);
                            position += 1;
                            break;
                        }
                        Some(u) if crate::internal::microsyntax::tokens::is_ascii_whitespace(u) => {
                            if let Some(span) = span {
                                descriptors.push(span.finish(position));
                            }
                            state = DescriptorState::AfterDescriptor;
                        }
                        Some(40) => {
                            state = DescriptorState::InParens(DescriptorSpan::consume(
                                span, position, 40,
                            ));
                        }
                        Some(u) => {
                            state = DescriptorState::InDescriptor(Some(DescriptorSpan::consume(
                                span, position, u,
                            )));
                        }
                    },
                    DescriptorState::InParens(span) => match current {
                        None => {
                            descriptors.push(span.finish(position));
                            break;
                        }
                        Some(41) => {
                            state =
                                DescriptorState::InDescriptor(Some(span.extended(position, 41)));
                        }
                        Some(u) => state = DescriptorState::InParens(span.extended(position, u)),
                    },
                    DescriptorState::AfterDescriptor => match current {
                        None => break,
                        Some(u) if crate::internal::microsyntax::tokens::is_ascii_whitespace(u) => {
                        }
                        _ => {
                            state = DescriptorState::InDescriptor(None);
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
        for &descriptor in &lexical.descriptors {
            let descriptor_range = descriptor.range;
            let suffix = descriptor.suffix;
            let prefix = copied(raw, descriptor.prefix);
            // A width or height value is a nonempty run of ASCII digits and nothing else.
            let integer = if matches!(suffix, 119 | 104) {
                HtmlNonNegativeInteger::from_decimal_digits(&prefix).ok()
            } else {
                None
            };
            match (suffix, integer) {
                (119 | 104, Some(value)) => {
                    let is_width = suffix == 119;
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
                (120, _) => {
                    let decimal = match numeric::strict_decimal_units(prefix.code_units()) {
                        Ok(decimal) => decimal,
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
                    let rounding = decimal.rounding;
                    if decimal.key.is_positive() {
                        if lexical.descriptors.len() == 1 {
                            author_descriptor =
                                Some(HtmlSrcsetDescriptor::Density(HtmlSrcsetDensity {
                                    syntax: prefix,
                                    key: decimal.key,
                                    rounding,
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
                    // Source recovery consumes the descriptor's valid numeric prefix, which the
                    // strict grammar has just accepted whole, so both read the same token.
                    match rounding {
                        HtmlBinary64Rounding::Finite(value) if value.value() >= 0.0 => {
                            density = Some(value);
                        }
                        HtmlBinary64Rounding::Finite(_) => failed = true,
                        HtmlBinary64Rounding::Overflow => {
                            diagnostics.push(parse_diagnostic(
                                HtmlSrcsetParseError::DensityConversionOverflow,
                                descriptor_range,
                                Some(index),
                            ));
                            failed = true;
                            overflow = true;
                        }
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

    /// The binary64 projection of a density is the source recovery conversion of its spelling:
    /// the checked spelling is a valid number whole, so recovery reads the same token.
    #[test]
    fn a_density_projects_as_the_recovery_conversion_of_its_spelling() {
        for spelling in [
            "1",
            "1.5",
            "0.25",
            ".5",
            "1e3",
            "1E-3",
            "000.0100",
            "123456789.987654321",
            "1.7976931348623157e308",
            "4.9e-324",
            "1e-400",
            "1e999",
            "9e999999",
        ] {
            let density = HtmlSrcsetDensity::from_decimal_syntax(spelling).unwrap();
            match numeric::parse_floating_point_number(&DocumentString::from(spelling)) {
                Ok(value) => {
                    assert_eq!(density.to_floating_point_number(), Ok(value), "{spelling}")
                }
                Err(error) => {
                    assert_eq!(
                        error,
                        crate::internal::microsyntax::numeric::HtmlNumericParseError::FloatingPointOverflow
                    );
                    assert_eq!(
                        density.to_floating_point_number(),
                        Err(HtmlSrcsetDensityProjectionError),
                        "{spelling}"
                    );
                }
            }
        }
        for rejected in ["", "0", "-1", "0.0e5", "+1", "1x", " 1", "2."] {
            assert!(
                HtmlSrcsetDensity::from_decimal_syntax(rejected).is_err(),
                "{rejected:?}"
            );
        }
        // Equality compares the spelling and the exact key, not the projection derived from them.
        assert_eq!(
            HtmlSrcsetDensity::from_decimal_syntax("1").unwrap(),
            HtmlSrcsetDensity::from_decimal_syntax("1").unwrap()
        );
        assert_ne!(
            HtmlSrcsetDensity::from_decimal_syntax("1").unwrap(),
            HtmlSrcsetDensity::from_decimal_syntax("1.0").unwrap()
        );
    }

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
            .map(|&d| copied(&raw, d.range))
            .collect();
        assert_eq!(
            descriptors,
            [
                DocumentString::from("2w"),
                DocumentString::from("(foo, bar)")
            ]
        );
        assert_eq!(lexed.candidates[0].descriptors[0].range, range(2, 4));
        assert_eq!(lexed.candidates[0].descriptors[1].range, range(6, 16));
        // Each token splits into the units before its last unit, and that last unit.
        assert_eq!(lexed.candidates[0].descriptors[0].prefix, range(2, 3));
        assert_eq!(lexed.candidates[0].descriptors[0].suffix, u16::from(b'w'));
        assert_eq!(lexed.candidates[0].descriptors[1].prefix, range(6, 15));
        assert_eq!(lexed.candidates[0].descriptors[1].suffix, u16::from(b')'));
        assert_eq!(
            copied(&raw, lexed.candidates[1].url),
            DocumentString::from("b")
        );
        assert_eq!(
            copied(&raw, lexed.candidates[1].descriptors[0].range),
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
