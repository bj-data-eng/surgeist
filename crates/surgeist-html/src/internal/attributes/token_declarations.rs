//! Local accesskey, blocking and link icon-size declarations.
//! Ordinary allocation and per-original-value UTF-16 admission apply; no
//! physical allocation ledger, work/cancellation budget or runtime effect.

use super::*;
use crate::internal::microsyntax::tokens::declared_keyword;
use crate::{
    internal::microsyntax::numeric::HtmlNonNegativeInteger,
    internal::microsyntax::tokens::HtmlDuplicateTokenError,
    internal::microsyntax::tokens::HtmlOrderedTokenSet,
    internal::microsyntax::tokens::HtmlSpaceSeparatedTokens,
    internal::microsyntax::tokens::HtmlSpaceToken,
    internal::microsyntax::tokens::HtmlTokenComparison,
    internal::microsyntax::tokens::HtmlUnorderedTokenSet,
    internal::values::interpretation::HtmlValueRecoveryState,
    internal::values::interpretation::HtmlValueReport,
};
use std::{error::Error, fmt};

/// A local author error; raw tokens remain in the common report independently.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlTokenDeclarationError {
    /// Two tokens were identical under the attribute's comparison; the payload names the repeat.
    Duplicate(HtmlDuplicateTokenError),
    /// An `accesskey` token is not exactly one Unicode code point.
    AccessKeyLength {
        /// The zero-based index of the offending token among the attribute's tokens.
        token_index: usize,
    },
    /// A `blocking` token is not exactly the lowercase `render` keyword. The author check is case
    /// sensitive, so `RENDER` is refused here even though explicit render detection ignores case.
    InvalidBlockingToken {
        /// The zero-based index of the offending token among the attribute's tokens.
        token_index: usize,
    },
    /// A `sizes` token is not `any` or a valid `WIDTHxHEIGHT` icon size.
    InvalidIconSize {
        /// The zero-based index of the offending token among the attribute's tokens.
        token_index: usize,
        /// The reason the token is not a valid icon size.
        reason: HtmlIconSizeError,
    },
}
impl fmt::Display for HtmlTokenDeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(_) => f.write_str("token declaration contains a duplicate token"),
            Self::AccessKeyLength { token_index } => write!(
                f,
                "accesskey token at index {token_index} is not exactly one code point"
            ),
            Self::InvalidBlockingToken { token_index } => write!(
                f,
                "blocking token at index {token_index} is not a recognized keyword"
            ),
            Self::InvalidIconSize { token_index, .. } => {
                write!(f, "icon size token at index {token_index} is invalid")
            }
        }
    }
}
impl From<HtmlDuplicateTokenError> for HtmlTokenDeclarationError {
    fn from(error: HtmlDuplicateTokenError) -> Self {
        Self::Duplicate(error)
    }
}
impl Error for HtmlTokenDeclarationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Duplicate(error) => Some(error),
            Self::InvalidIconSize { reason, .. } => Some(reason),
            Self::AccessKeyLength { .. } | Self::InvalidBlockingToken { .. } => None,
        }
    }
}

/// The author-conformance verdict for an `accesskey`, `blocking` or `sizes` token list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTokenDeclarationAuthorState {
    /// The attribute is absent, which is conforming.
    Missing,
    /// The attribute is present and every token and the set as a whole satisfy the local rules.
    Conforming,
    /// The attribute is present but breaks a local rule; the payload names the first rule checked
    /// that failed. Other violations stay in the report.
    Nonconforming(HtmlTokenDeclarationError),
    /// The rule does not apply to the supplied element.
    Unapplied,
}

/// A checked ordered unique set of exact single-code-point access-key tokens.
/// This proves no keyboard correspondence, assigned key or activation behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAccessKeys {
    tokens: HtmlOrderedTokenSet,
}
impl HtmlAccessKeys {
    /// Builds an ordered set of access keys.
    ///
    /// Returns `Duplicate` when two tokens are identical code unit for code unit (comparison is
    /// case-sensitive). Otherwise returns `AccessKeyLength` for the first token that is not exactly
    /// one code point.
    pub fn new(
        tokens: impl IntoIterator<Item = HtmlSpaceToken>,
    ) -> Result<Self, HtmlTokenDeclarationError> {
        let tokens = HtmlOrderedTokenSet::new(HtmlTokenComparison::CaseSensitive, tokens)?;
        for (token_index, token) in tokens.tokens().iter().enumerate() {
            if !single_code_point(token) {
                return Err(HtmlTokenDeclarationError::AccessKeyLength { token_index });
            }
        }
        Ok(Self { tokens })
    }
    /// The access keys in source order, each a single code point.
    pub fn tokens(&self) -> &[HtmlSpaceToken] {
        self.tokens.tokens()
    }
    /// Emits exact spellings in source order, joined by single ASCII spaces.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Accesskey.qualified_name(),
            self.tokens.as_document_string(),
        )
    }
}
fn single_code_point(token: &HtmlSpaceToken) -> bool {
    token.value().code_points().take(2).count() == 1
}

/// The explicit pinned blocking-token set. Empty does not exclude implicit
/// potentially-render-blocking behavior or establish current render blocking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlBlockingTokens {
    /// No `render` token is present: the attribute is absent, empty, or holds only other tokens.
    /// This does not exclude implicit render blocking.
    Empty,
    /// The attribute holds the `render` token.
    Render,
}
impl HtmlBlockingTokens {
    /// Emits a no-namespace `blocking` attribute whose value is empty or `render`.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Blocking.qualified_name(),
            match self {
                Self::Empty => "",
                Self::Render => "render",
            },
        )
    }
}

/// Icon dimensions are exact positive integers in raw/device pixels.
/// Their presence in the linked resource is a separate author condition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBitmapIconSize {
    width: HtmlNonNegativeInteger,
    height: HtmlNonNegativeInteger,
}
/// Why text is not a valid icon size. The enum is `#[non_exhaustive]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlIconSizeError {
    /// A bitmap size has a width or height of zero, which the standard does not allow.
    ZeroDimension,
    /// A dimension starts with the digit zero.
    LeadingZero,
    /// The text does not contain exactly one `x` or `X` separating the dimensions.
    Separator,
    /// A dimension is empty or contains something other than ASCII digits.
    DimensionDigits,
}
impl fmt::Display for HtmlIconSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroDimension => "icon size has a zero width or height",
            Self::LeadingZero => "icon size dimension has a leading zero",
            Self::Separator => "icon size dimensions are not separated by a single 'x' or 'X'",
            Self::DimensionDigits => "icon size dimension is not a sequence of ASCII digits",
        })
    }
}
impl Error for HtmlIconSizeError {}
impl HtmlBitmapIconSize {
    /// Builds a bitmap size from exact dimensions.
    ///
    /// Returns `ZeroDimension` when `width` or `height` is zero.
    pub fn new(
        width: HtmlNonNegativeInteger,
        height: HtmlNonNegativeInteger,
    ) -> Result<Self, HtmlIconSizeError> {
        if width.is_zero() || height.is_zero() {
            return Err(HtmlIconSizeError::ZeroDimension);
        }
        Ok(Self { width, height })
    }
    /// The width in pixels, always greater than zero.
    pub fn width(&self) -> &HtmlNonNegativeInteger {
        &self.width
    }
    /// The height in pixels, always greater than zero.
    pub fn height(&self) -> &HtmlNonNegativeInteger {
        &self.height
    }
    fn token(&self) -> HtmlSpaceToken {
        HtmlSpaceToken::integer_pair(&self.width, declared_keyword!("x"), &self.height)
    }
}
/// One icon size keyword (HTML877d #rel-icon).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIconSize {
    /// The `any` keyword: the resource is a scalable icon such as an SVG image.
    Scalable,
    /// A bitmap size in raw pixels, written `WIDTHxHEIGHT`.
    Bitmap(HtmlBitmapIconSize),
}
impl HtmlIconSize {
    fn token(&self) -> HtmlSpaceToken {
        match self {
            Self::Scalable => HtmlSpaceToken::declared(declared_keyword!("any")),
            Self::Bitmap(size) => size.token(),
        }
    }
}
/// A checked locally conforming unordered icon-size set. Serialization order
/// is retained as a spelling choice; equality ignores that order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconSizes {
    tokens: HtmlUnorderedTokenSet,
}
impl HtmlIconSizes {
    /// Builds a set of icon sizes.
    ///
    /// Returns the repeat when two sizes serialize to the same token under ASCII case-insensitive
    /// comparison.
    pub fn new(
        sizes: impl IntoIterator<Item = HtmlIconSize>,
    ) -> Result<Self, HtmlDuplicateTokenError> {
        Ok(Self {
            tokens: HtmlUnorderedTokenSet::new(
                HtmlTokenComparison::AsciiCaseInsensitive,
                sizes.into_iter().map(|size| size.token()),
            )?,
        })
    }
    /// The size tokens in the chosen serialization order.
    pub fn tokens(&self) -> &[HtmlSpaceToken] {
        self.tokens.tokens()
    }
    /// Emits only locally conforming sizes; rel/resource conditions are unchecked.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Sizes.qualified_name(),
            self.tokens.as_document_string(),
        )
    }
}

/// Shared exact input observations composed from the established attribute and
/// common token report owners. There is no aggregate conformance Boolean.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlTokenDeclarationInput {
    input: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>>,
    author: HtmlTokenDeclarationAuthorState,
    duplicate: Option<HtmlDuplicateTokenError>,
    rel: Option<Attribute>,
    rel_observed: bool,
}
impl HtmlTokenDeclarationInput {
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
    /// Whether the attribute's rule applies to the supplied element.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.input.applicability()
    }
    /// The standard attribute this report is about.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.input.attribute()
    }
    /// The rows of the pinned specification's attribute index that this rule reproduces.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.input.inventory_rows()
    }
    /// The URL of the attribute's definition in the pinned HTML Standard snapshot.
    pub fn source_clause(&self) -> &'static str {
        self.input.source_clause()
    }
    /// The length of the raw value in UTF-16 code units, as admitted against the per-value cap.
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    /// None means the rule was unapplied; applicable absence has a common report.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>> {
        self.common.as_ref()
    }
    /// The local author verdict. A duplicate token may also be present in `duplicate` when another
    /// violation is reported here.
    pub fn author(&self) -> &HtmlTokenDeclarationAuthorState {
        &self.author
    }
    /// Unique-set failure remains observable even when another token is invalid.
    pub fn duplicate(&self) -> Option<&HtmlDuplicateTokenError> {
        self.duplicate.as_ref()
    }
    /// The wider author-context verdict, which depends on the element and companion attributes.
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        self.input.context_author()
    }
    /// The supplied `script` element facts, even when the rule is unapplied.
    pub fn script_context(&self) -> Option<HtmlScriptAttributeContext> {
        self.input.script_context()
    }
    /// Actual selected rel from the complete companion attribute set.
    pub fn rel_attribute(&self) -> Option<&Attribute> {
        self.rel.as_ref()
    }
    /// Whether the evaluation was given a companion attribute set from which `rel` was selected,
    /// whether or not `rel` was present.
    pub fn rel_observed(&self) -> bool {
        self.rel_observed
    }
    /// The URL of the pinned standard's definition of the supplied context facts: the `script`
    /// element or the `link` `rel` attribute. It is `None` when neither was supplied.
    pub fn context_source_clause(&self) -> Option<&'static str> {
        if self.script_context().is_some() {
            Some(SCRIPT_SOURCE)
        } else if self.rel_observed {
            Some(source!("attr-link-rel"))
        } else {
            None
        }
    }
    fn tokens(&self) -> &[HtmlSpaceToken] {
        match self.common.as_ref().map(HtmlValueReport::recovery) {
            Some(HtmlValueRecoveryState::Value(tokens)) => tokens.tokens(),
            _ => &[],
        }
    }
}
/// The `accesskey` token list with the single-code-point candidates the standard processes.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlAccessKeyReport {
    input: HtmlTokenDeclarationInput,
}
impl HtmlAccessKeyReport {
    /// The shared exact observations of the attribute.
    pub fn input(&self) -> &HtmlTokenDeclarationInput {
        &self.input
    }
    /// Ordered source candidates, including repetitions, before keyboard checks.
    /// Applicable absence has no content candidates, but host fallback is possible.
    pub fn candidate_tokens(&self) -> impl Iterator<Item = &HtmlSpaceToken> {
        self.input
            .tokens()
            .iter()
            .filter(|token| single_code_point(token))
    }
    /// The URL of the pinned standard's keyboard shortcut processing model.
    pub fn processing_source_clause(&self) -> &'static str {
        source!("keyboard-shortcuts-processing-model")
    }
}
/// The `blocking` token list with its explicit processing set.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlBlockingReport {
    input: HtmlTokenDeclarationInput,
    explicit: Option<HtmlBlockingTokens>,
}
impl HtmlBlockingReport {
    /// The shared exact observations of the attribute.
    pub fn input(&self) -> &HtmlTokenDeclarationInput {
        &self.input
    }
    /// None is unapplied. Missing applicable content produces Empty.
    pub fn explicit_tokens(&self) -> Option<HtmlBlockingTokens> {
        self.explicit
    }
    /// The URL of the pinned standard's blocking tokens set.
    pub fn processing_source_clause(&self) -> &'static str {
        source!("blocking-tokens-set")
    }
}
/// The result of reading one `sizes` token as an icon size keyword.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconSizeKeyword {
    token_index: usize,
    result: Result<HtmlIconSize, HtmlIconSizeError>,
}
impl HtmlIconSizeKeyword {
    /// The zero-based index of the token among all tokens of the attribute.
    pub fn token_index(&self) -> usize {
        self.token_index
    }
    /// The token read as an icon size, or the reason it is not one.
    pub fn result(&self) -> Result<&HtmlIconSize, HtmlIconSizeError> {
        self.result.as_ref().map_err(|&e| e)
    }
}
/// The `sizes` token list with the per-keyword reading of each token.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlIconSizesReport {
    input: HtmlTokenDeclarationInput,
    keywords: Option<Vec<HtmlIconSizeKeyword>>,
}
impl HtmlIconSizesReport {
    /// The shared exact observations of the attribute.
    pub fn input(&self) -> &HtmlTokenDeclarationInput {
        &self.input
    }
    /// Per-keyword outcomes preserve source order and repetitions. None is
    /// missing or unapplied; Some(empty) is a specified zero-token value.
    pub fn keywords(&self) -> Option<&[HtmlIconSizeKeyword]> {
        self.keywords.as_deref()
    }
    /// The URL of the pinned standard's definition of the `icon` link type.
    pub fn processing_source_clause(&self) -> &'static str {
        source!("rel-icon")
    }
    /// No linked-resource availability fact is supplied by this local operation.
    pub fn resource_conformance(&self) -> HtmlAttributeContextAuthorState {
        if self.input.applicability() != HtmlAttributeApplicability::Applicable {
            HtmlAttributeContextAuthorState::Unapplied
        } else if self.input.raw_attribute().is_none() {
            HtmlAttributeContextAuthorState::NotRequired
        } else {
            HtmlAttributeContextAuthorState::Unchecked
        }
    }
}

/// Borrowed canonical sizes-token grammar. Dimension ranges retain arbitrary
/// precision and exact authored UTF-16; callers choose materialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIconSizeShape {
    /// The token is `any`, compared ASCII case-insensitively.
    Scalable,
    /// The token is a `WIDTHxHEIGHT` bitmap size.
    Bitmap {
        /// The offsets of the width digits in the token, in UTF-16 code units. The range is
        /// nonempty and starts with a nonzero digit.
        width: std::ops::Range<usize>,
        /// The offsets of the height digits in the token, in UTF-16 code units. The range is
        /// nonempty and starts with a nonzero digit.
        height: std::ops::Range<usize>,
    },
}
/// The grammar stage; each stage owns the data it requires.
#[derive(Clone, Copy, Debug)]
enum IconSizeStage {
    /// Compare the whole token with `any`.
    Any,
    /// Locate the only `x` separator.
    Separator {
        separator: Option<usize>,
        multiple: bool,
    },
    /// Check the width digits before this separator.
    Width(usize),
    /// Check the height digits after this separator.
    Height(usize),
}
/// A stepper that reads one icon size token over a borrowed slice of UTF-16 code units, one bounded
/// action per step. It performs no allocation.
#[derive(Clone, Debug)]
pub struct HtmlIconSizeCursor<'a> {
    units: &'a [u16],
    stage: IconSizeStage,
    index: usize,
    result: Option<Result<HtmlIconSizeShape, HtmlIconSizeError>>,
}
impl<'a> HtmlIconSizeCursor<'a> {
    // Seven bounded grammar actions: phase/range selection, original-unit read,
    // ASCII classification, error selection, position update, result/transition.
    // No dimension materialization is hidden behind this borrowed operation.
    /// The work units that one [`HtmlIconSizeCursor::step`] charges.
    pub const STEP_WORK: usize = 7;
    /// Starts reading `units`, a single token, before its first step.
    pub fn new(units: &'a [u16]) -> Self {
        Self {
            units,
            stage: IconSizeStage::Any,
            index: 0,
            result: None,
        }
    }
    /// Whether the grammar has produced a result.
    pub fn is_complete(&self) -> bool {
        self.result.is_some()
    }
    /// The grammar result once complete, or `None` while steps remain.
    pub fn result(&self) -> Option<&Result<HtmlIconSizeShape, HtmlIconSizeError>> {
        self.result.as_ref()
    }
    /// Steps until the grammar completes and returns its result.
    fn finish(mut self) -> Result<HtmlIconSizeShape, HtmlIconSizeError> {
        loop {
            if let Some(result) = self.result.take() {
                return result;
            }
            self.step();
        }
    }
    /// Performs one bounded grammar action. Callers stop stepping once
    /// [`HtmlIconSizeCursor::is_complete`] returns true.
    pub fn step(&mut self) {
        match self.stage {
            IconSizeStage::Any => {
                if self.units.len() != 3 {
                    self.stage = IconSizeStage::Separator {
                        separator: None,
                        multiple: false,
                    };
                    self.index = 0;
                } else if self.index == 3 {
                    self.result = Some(Ok(HtmlIconSizeShape::Scalable));
                } else if ascii_case_insensitive_unit(self.units[self.index], b"any"[self.index]) {
                    self.index += 1;
                } else {
                    self.stage = IconSizeStage::Separator {
                        separator: None,
                        multiple: false,
                    };
                    self.index = 0;
                }
            }
            IconSizeStage::Separator {
                separator,
                multiple,
            } => {
                if self.index == self.units.len() {
                    match (separator, multiple) {
                        (Some(separator), false) => {
                            self.stage = IconSizeStage::Width(separator);
                            self.index = 0;
                        }
                        _ => self.result = Some(Err(HtmlIconSizeError::Separator)),
                    }
                } else {
                    let unit = self.units[self.index];
                    if unit == 120 || unit == 88 {
                        self.stage = IconSizeStage::Separator {
                            separator: separator.or(Some(self.index)),
                            multiple: multiple || separator.is_some(),
                        };
                    }
                    self.index += 1;
                }
            }
            IconSizeStage::Width(separator) | IconSizeStage::Height(separator) => {
                let width = matches!(self.stage, IconSizeStage::Width(_));
                let (start, end) = if width {
                    (0, separator)
                } else {
                    (separator + 1, self.units.len())
                };
                if self.index == 0 {
                    if start == end {
                        self.result = Some(Err(HtmlIconSizeError::DimensionDigits));
                        return;
                    }
                    if self.units[start] == 48 {
                        self.result = Some(Err(HtmlIconSizeError::LeadingZero));
                        return;
                    }
                }
                if start + self.index == end {
                    if width {
                        self.stage = IconSizeStage::Height(separator);
                        self.index = 0;
                    } else {
                        self.result = Some(Ok(HtmlIconSizeShape::Bitmap {
                            width: 0..separator,
                            height: separator + 1..self.units.len(),
                        }));
                    }
                } else if (48..=57).contains(&self.units[start + self.index]) {
                    self.index += 1;
                } else {
                    self.result = Some(Err(HtmlIconSizeError::DimensionDigits));
                }
            }
        }
    }
}
fn icon_size(token: &HtmlSpaceToken) -> Result<HtmlIconSize, HtmlIconSizeError> {
    let units = token.value().code_units();
    match HtmlIconSizeCursor::new(units).finish() {
        Err(error) => Err(error),
        Ok(HtmlIconSizeShape::Scalable) => Ok(HtmlIconSize::Scalable),
        Ok(HtmlIconSizeShape::Bitmap { width, height }) => {
            // The cursor recognized each range as a run of ASCII digits; reading it checks that again.
            let dimension = |range: std::ops::Range<usize>| {
                units
                    .get(range)
                    .and_then(|digits| {
                        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(
                            digits.to_vec(),
                        ))
                        .ok()
                    })
                    .ok_or(HtmlIconSizeError::DimensionDigits)
            };
            HtmlBitmapIconSize::new(dimension(width)?, dimension(height)?).map(HtmlIconSize::Bitmap)
        }
    }
}

/// The three token-list attributes this owner declares.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclarationRole {
    Accesskey,
    Blocking,
    Sizes,
}
impl DeclarationRole {
    fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::Accesskey => KnownHtmlAttribute::Accesskey,
            Self::Blocking => KnownHtmlAttribute::Blocking,
            Self::Sizes => KnownHtmlAttribute::Sizes,
        }
    }
}

fn rule(role: DeclarationRole, element: Option<KnownHtmlElement>) -> RuleMetadata {
    let (rows, source, context): (&'static [u16], &'static str, Context) = match role {
        DeclarationRole::Accesskey => (&[4], source!("the-accesskey-attribute"), Context::Global),
        DeclarationRole::Blocking => (
            &[18],
            match element {
                Some(KnownHtmlElement::Link) => source!("attr-link-blocking"),
                Some(KnownHtmlElement::Script) => source!("attr-script-blocking"),
                Some(KnownHtmlElement::Style) => source!("attr-style-blocking"),
                _ => source!("blocking-attributes"),
            },
            Context::Elements(&[
                KnownHtmlElement::Link,
                KnownHtmlElement::Script,
                KnownHtmlElement::Style,
            ]),
        ),
        DeclarationRole::Sizes => (
            &[144],
            source!("attr-link-sizes"),
            Context::Elements(&[KnownHtmlElement::Link]),
        ),
    };
    RuleMetadata {
        attribute: role.attribute(),
        rows,
        source,
        context,
        vocabulary_source: None,
    }
}

impl HtmlValueInterpreter {
    fn declaration_input(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        role: DeclarationRole,
        required_role: Option<KnownHtmlElement>,
    ) -> Result<HtmlTokenDeclarationInput, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut input = report(
            rule(role, KnownHtmlElement::from_name(element)),
            element,
            raw,
        );
        if input.applicability == HtmlAttributeApplicability::Applicable
            && required_role.is_some_and(|role| KnownHtmlElement::from_name(element) != Some(role))
        {
            input.applicability = HtmlAttributeApplicability::DifferentContext;
            input.context_author = HtmlAttributeContextAuthorState::Unapplied;
        }
        let applicable = input.applicability == HtmlAttributeApplicability::Applicable;
        let common = if applicable {
            Some(self.space_separated_tokens(raw.map(HtmlAttributeInput::value))?)
        } else {
            None
        };
        Ok(HtmlTokenDeclarationInput {
            input,
            common,
            author: if applicable {
                if raw.is_some() {
                    HtmlTokenDeclarationAuthorState::Conforming
                } else {
                    HtmlTokenDeclarationAuthorState::Missing
                }
            } else {
                HtmlTokenDeclarationAuthorState::Unapplied
            },
            duplicate: None,
            rel: None,
            rel_observed: false,
        })
    }
    /// Local unique single-code-point author checking and ordered candidate
    /// filtering. Does not assign keyboard shortcuts or execute activation.
    pub fn accesskey(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlAccessKeyReport, HtmlValueAdmissionError> {
        let mut input = self.declaration_input(element, raw, DeclarationRole::Accesskey, None)?;
        if raw.is_some() && input.common.is_some() {
            input.duplicate = HtmlOrderedTokenSet::new(
                HtmlTokenComparison::CaseSensitive,
                input.tokens().iter().cloned(),
            )
            .err();
            let length = input
                .tokens()
                .iter()
                .position(|token| !single_code_point(token));
            if let Some(token_index) = length {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::AccessKeyLength { token_index },
                );
            } else if let Some(error) = &input.duplicate {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::Duplicate(error.clone()),
                );
            }
        }
        Ok(HtmlAccessKeyReport { input })
    }
    /// Exact authored render tokens and the independently case-folded explicit
    /// processing set. Link/script author context remains unchecked when present.
    pub fn blocking(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        self.blocking_observed(element, raw, None)
    }
    fn blocking_observed(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        required_role: Option<KnownHtmlElement>,
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        let mut input =
            self.declaration_input(element, raw, DeclarationRole::Blocking, required_role)?;
        if input.common.is_none() {
            return Ok(HtmlBlockingReport {
                input,
                explicit: None,
            });
        }
        let explicit = if input.tokens().iter().any(|t| {
            crate::internal::elements::style::blocking_token_is_render_with(
                t.value().code_units(),
                || Ok::<(), std::convert::Infallible>(()),
            )
            .unwrap_or_else(|never| match never {})
        }) {
            HtmlBlockingTokens::Render
        } else {
            HtmlBlockingTokens::Empty
        };
        if raw.is_some() {
            input.duplicate = HtmlUnorderedTokenSet::new(
                HtmlTokenComparison::CaseSensitive,
                input.tokens().iter().cloned(),
            )
            .err();
            if let Some(token_index) = input
                .tokens()
                .iter()
                .position(|t| t.value().code_units() != [114, 101, 110, 100, 101, 114])
            {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::InvalidBlockingToken { token_index },
                );
            } else if let Some(error) = &input.duplicate {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::Duplicate(error.clone()),
                );
            }
            if matches!(
                KnownHtmlElement::from_name(element),
                Some(KnownHtmlElement::Link | KnownHtmlElement::Script)
            ) {
                input.input.context_author = HtmlAttributeContextAuthorState::Unchecked;
            }
        }
        Ok(HtmlBlockingReport {
            input,
            explicit: Some(explicit),
        })
    }
    /// Retains supplied actual script kind/placement without deriving it from
    /// type/src. Only external classic/module allow authored blocking presence.
    pub fn blocking_for_script(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        context: HtmlScriptAttributeContext,
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        let mut result = self.blocking_observed(element, raw, Some(KnownHtmlElement::Script))?;
        result.input.input.supplied_context = SuppliedContext::Script(context);
        if result.input.input.applicability == HtmlAttributeApplicability::Applicable
            && raw.is_some()
        {
            result.input.input.context_author = if matches!(
                context,
                HtmlScriptAttributeContext::ExternalClassic
                    | HtmlScriptAttributeContext::ExternalModule
            ) {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::ScriptContextForbidsAttribute,
                )
            };
        }
        Ok(result)
    }
    /// Observes the actual blocking and rel slots of a complete attribute set;
    /// membership only, not whole rel conformance or rendering behavior.
    pub fn blocking_for_link(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        let raw = selected(attributes, KnownHtmlAttribute::Blocking);
        let rel = selected(attributes, KnownHtmlAttribute::Rel);
        self.admit_attribute_raw(raw.as_ref().map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(rel.as_ref().map(HtmlAttributeInput::value))?;
        let mut result =
            self.blocking_observed(element, raw.as_ref(), Some(KnownHtmlElement::Link))?;
        link_context(
            &mut result.input,
            rel.as_ref(),
            &["stylesheet", "expect"],
            HtmlAttributeContextConformanceError::BlockingRequiresStylesheetOrExpectRel,
        );
        if result.input.applicability() != HtmlAttributeApplicability::Applicable {
            result.explicit = None;
        }
        Ok(result)
    }
    /// Checks only link icon-size tokens. Original raw length is admitted before
    /// role checking. Ordinary allocation; no image sizing/hit testing or resource
    /// availability certification. Context stays independent of token validity.
    pub fn icon_sizes(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlIconSizesReport, HtmlValueAdmissionError> {
        self.icon_sizes_observed(element, raw, None)
    }
    fn icon_sizes_observed(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        required_role: Option<KnownHtmlElement>,
    ) -> Result<HtmlIconSizesReport, HtmlValueAdmissionError> {
        let mut input =
            self.declaration_input(element, raw, DeclarationRole::Sizes, required_role)?;
        let keywords = if input.common.is_some() && raw.is_some() {
            input.duplicate = HtmlUnorderedTokenSet::new(
                HtmlTokenComparison::AsciiCaseInsensitive,
                input.tokens().iter().cloned(),
            )
            .err();
            let keywords: Vec<_> = input
                .tokens()
                .iter()
                .enumerate()
                .map(|(token_index, token)| HtmlIconSizeKeyword {
                    token_index,
                    result: icon_size(token),
                })
                .collect();
            if let Some((token_index, reason)) = keywords.iter().find_map(|k| {
                k.result
                    .as_ref()
                    .err()
                    .map(|&reason| (k.token_index, reason))
            }) {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::InvalidIconSize {
                        token_index,
                        reason,
                    },
                );
            } else if let Some(error) = &input.duplicate {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::Duplicate(error.clone()),
                );
            }
            input.input.context_author = HtmlAttributeContextAuthorState::Unchecked;
            Some(keywords)
        } else {
            None
        };
        Ok(HtmlIconSizesReport { input, keywords })
    }
    /// Checks icon/apple-touch-icon membership from the complete actual link
    /// attributes, retaining original rel. Resource availability stays unchecked.
    pub fn icon_sizes_for_link(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
    ) -> Result<HtmlIconSizesReport, HtmlValueAdmissionError> {
        let raw = selected(attributes, KnownHtmlAttribute::Sizes);
        let rel = selected(attributes, KnownHtmlAttribute::Rel);
        self.admit_attribute_raw(raw.as_ref().map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(rel.as_ref().map(HtmlAttributeInput::value))?;
        let mut result =
            self.icon_sizes_observed(element, raw.as_ref(), Some(KnownHtmlElement::Link))?;
        link_context(
            &mut result.input,
            rel.as_ref(),
            &["icon", "apple-touch-icon"],
            HtmlAttributeContextConformanceError::IconSizesRequireIconRel,
        );
        if result.input.applicability() != HtmlAttributeApplicability::Applicable {
            result.keywords = None;
        }
        Ok(result)
    }
}
fn selected<A: crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized>(
    attributes: &A,
    name: KnownHtmlAttribute,
) -> Option<A::Attribute<'_>> {
    let name = name.qualified_name();
    find_attribute(attributes, name.namespace(), name.local_name())
}
fn link_context(
    input: &mut HtmlTokenDeclarationInput,
    rel: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    required: &[&str],
    error: HtmlAttributeContextConformanceError,
) {
    input.rel = rel.map(capture_attribute);
    input.rel_observed = true;
    if input.input.applicability != HtmlAttributeApplicability::Applicable {
        return;
    }
    if KnownHtmlElement::from_name(input.element()) != Some(KnownHtmlElement::Link) {
        input.input.applicability = HtmlAttributeApplicability::DifferentContext;
        input.input.context_author = HtmlAttributeContextAuthorState::Unapplied;
        input.author = HtmlTokenDeclarationAuthorState::Unapplied;
        input.common = None;
        input.duplicate = None;
        return;
    }
    if input.raw_attribute().is_some() {
        let matches = rel.is_some_and(|rel| {
            crate::internal::microsyntax::tokens::space_token_slices(rel.value()).any(|token| {
                required
                    .iter()
                    .any(|keyword| crate::internal::elements::rel::ascii_keyword(token, keyword))
            })
        });
        input.input.context_author = if matches {
            HtmlAttributeContextAuthorState::Conforming
        } else {
            HtmlAttributeContextAuthorState::Nonconforming(error)
        };
    }
}
