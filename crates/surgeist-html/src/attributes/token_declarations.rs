//! Local accesskey, blocking and link icon-size declarations.
//! Ordinary allocation and per-original-value UTF-16 admission apply; no
//! physical allocation ledger, work/cancellation budget or runtime effect.

use super::*;
use crate::{
    HtmlDuplicateTokenError, HtmlNonNegativeInteger, HtmlOrderedTokenSet, HtmlSpaceSeparatedTokens,
    HtmlSpaceToken, HtmlTokenComparison, HtmlUnorderedTokenSet, HtmlValueRecoveryState,
    HtmlValueReport, OrderedAttributes,
};
use std::{error::Error, fmt};

/// A local author error; raw tokens remain in the common report independently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTokenDeclarationError {
    Duplicate(HtmlDuplicateTokenError),
    AccessKeyLength {
        token_index: usize,
    },
    InvalidBlockingToken {
        token_index: usize,
    },
    InvalidIconSize {
        token_index: usize,
        reason: HtmlIconSizeError,
    },
}
impl fmt::Display for HtmlTokenDeclarationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTML token declaration: {self:?}")
    }
}
impl Error for HtmlTokenDeclarationError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTokenDeclarationAuthorState {
    Missing,
    Conforming,
    Nonconforming(HtmlTokenDeclarationError),
    Unapplied,
}

/// A checked ordered unique set of exact single-code-point access-key tokens.
/// This proves no keyboard correspondence, assigned key or activation behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAccessKeys {
    tokens: HtmlOrderedTokenSet,
}
impl HtmlAccessKeys {
    pub fn new(
        tokens: impl IntoIterator<Item = HtmlSpaceToken>,
    ) -> Result<Self, HtmlTokenDeclarationError> {
        let tokens = HtmlOrderedTokenSet::new(HtmlTokenComparison::CaseSensitive, tokens)
            .map_err(HtmlTokenDeclarationError::Duplicate)?;
        for (token_index, token) in tokens.tokens().iter().enumerate() {
            if !single_code_point(token) {
                return Err(HtmlTokenDeclarationError::AccessKeyLength { token_index });
            }
        }
        Ok(Self { tokens })
    }
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
    Empty,
    Render,
}
impl HtmlBlockingTokens {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlIconSizeError {
    ZeroDimension,
    LeadingZero,
    Separator,
    DimensionDigits,
}
impl fmt::Display for HtmlIconSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid HTML icon size: {self:?}")
    }
}
impl Error for HtmlIconSizeError {}
impl HtmlBitmapIconSize {
    pub fn new(
        width: HtmlNonNegativeInteger,
        height: HtmlNonNegativeInteger,
    ) -> Result<Self, HtmlIconSizeError> {
        if width.is_zero() || height.is_zero() {
            return Err(HtmlIconSizeError::ZeroDimension);
        }
        Ok(Self { width, height })
    }
    pub fn width(&self) -> &HtmlNonNegativeInteger {
        &self.width
    }
    pub fn height(&self) -> &HtmlNonNegativeInteger {
        &self.height
    }
    fn token(&self) -> HtmlSpaceToken {
        HtmlSpaceToken::new(format!(
            "{}x{}",
            self.width.decimal_digits(),
            self.height.decimal_digits()
        ))
        .expect("positive integer digits and x form one space token")
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlIconSize {
    Scalable,
    Bitmap(HtmlBitmapIconSize),
}
impl HtmlIconSize {
    fn token(&self) -> HtmlSpaceToken {
        match self {
            Self::Scalable => HtmlSpaceToken::new("any").expect("literal space token"),
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
    pub fn input_code_units(&self) -> usize {
        self.input.input_code_units()
    }
    /// None means the rule was unapplied; applicable absence has a common report.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>> {
        self.common.as_ref()
    }
    pub fn author(&self) -> &HtmlTokenDeclarationAuthorState {
        &self.author
    }
    /// Unique-set failure remains observable even when another token is invalid.
    pub fn duplicate(&self) -> Option<&HtmlDuplicateTokenError> {
        self.duplicate.as_ref()
    }
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        self.input.context_author()
    }
    pub fn script_context(&self) -> Option<HtmlScriptAttributeContext> {
        self.input.script_context()
    }
    /// Actual selected rel from the complete companion attribute set.
    pub fn rel_attribute(&self) -> Option<&Attribute> {
        self.rel.as_ref()
    }
    pub fn rel_observed(&self) -> bool {
        self.rel_observed
    }
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
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlAccessKeyReport {
    input: HtmlTokenDeclarationInput,
}
impl HtmlAccessKeyReport {
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
    pub fn processing_source_clause(&self) -> &'static str {
        source!("keyboard-shortcuts-processing-model")
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlBlockingReport {
    input: HtmlTokenDeclarationInput,
    explicit: Option<HtmlBlockingTokens>,
}
impl HtmlBlockingReport {
    pub fn input(&self) -> &HtmlTokenDeclarationInput {
        &self.input
    }
    /// None is unapplied. Missing applicable content produces Empty.
    pub fn explicit_tokens(&self) -> Option<HtmlBlockingTokens> {
        self.explicit
    }
    pub fn processing_source_clause(&self) -> &'static str {
        source!("blocking-tokens-set")
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlIconSizeKeyword {
    token_index: usize,
    result: Result<HtmlIconSize, HtmlIconSizeError>,
}
impl HtmlIconSizeKeyword {
    pub fn token_index(&self) -> usize {
        self.token_index
    }
    pub fn result(&self) -> Result<&HtmlIconSize, HtmlIconSizeError> {
        self.result.as_ref().map_err(|&e| e)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlIconSizesReport {
    input: HtmlTokenDeclarationInput,
    keywords: Option<Vec<HtmlIconSizeKeyword>>,
}
impl HtmlIconSizesReport {
    pub fn input(&self) -> &HtmlTokenDeclarationInput {
        &self.input
    }
    /// Per-keyword outcomes preserve source order and repetitions. None is
    /// missing or unapplied; Some(empty) is a specified zero-token value.
    pub fn keywords(&self) -> Option<&[HtmlIconSizeKeyword]> {
        self.keywords.as_deref()
    }
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
pub(crate) enum IconSizeShape {
    Scalable,
    Bitmap {
        width: std::ops::Range<usize>,
        height: std::ops::Range<usize>,
    },
}
pub(crate) struct IconSizeCursor<'a> {
    units: &'a [u16],
    stage: u8,
    index: usize,
    separator: Option<usize>,
    multiple: bool,
    result: Option<Result<IconSizeShape, HtmlIconSizeError>>,
}
impl<'a> IconSizeCursor<'a> {
    // Seven bounded grammar actions: phase/range selection, original-unit read,
    // ASCII classification, error selection, position update, result/transition.
    // No dimension materialization is hidden behind this borrowed operation.
    pub(crate) const STEP_WORK: usize = 7;
    pub(crate) fn new(units: &'a [u16]) -> Self {
        Self {
            units,
            stage: 0,
            index: 0,
            separator: None,
            multiple: false,
            result: None,
        }
    }
    pub(crate) fn is_complete(&self) -> bool {
        self.result.is_some()
    }
    pub(crate) fn result(&self) -> Option<&Result<IconSizeShape, HtmlIconSizeError>> {
        self.result.as_ref()
    }
    pub(crate) fn step(&mut self) {
        match self.stage {
            0 => {
                if self.units.len() != 3 {
                    self.stage = 1;
                    self.index = 0;
                } else if self.index == 3 {
                    self.result = Some(Ok(IconSizeShape::Scalable));
                } else if crate::rel::fold(self.units[self.index]) == u16::from(b"any"[self.index])
                {
                    self.index += 1;
                } else {
                    self.stage = 1;
                    self.index = 0;
                }
            }
            1 => {
                if self.index == self.units.len() {
                    if self.multiple || self.separator.is_none() {
                        self.result = Some(Err(HtmlIconSizeError::Separator));
                    } else {
                        self.stage = 2;
                        self.index = 0;
                    }
                } else {
                    let unit = self.units[self.index];
                    if unit == 120 || unit == 88 {
                        if self.separator.is_some() {
                            self.multiple = true;
                        } else {
                            self.separator = Some(self.index);
                        }
                    }
                    self.index += 1;
                }
            }
            2 | 3 => {
                let separator = self.separator.expect("separator phase established");
                let (start, end) = if self.stage == 2 {
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
                    if self.stage == 2 {
                        self.stage = 3;
                        self.index = 0;
                    } else {
                        self.result = Some(Ok(IconSizeShape::Bitmap {
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
            _ => unreachable!("closed cursor stages"),
        }
    }
}
fn icon_size(token: &HtmlSpaceToken) -> Result<HtmlIconSize, HtmlIconSizeError> {
    let units = token.value().code_units();
    let mut cursor = IconSizeCursor::new(units);
    while !cursor.is_complete() {
        cursor.step();
    }
    match cursor.result().expect("complete sizes grammar") {
        Err(error) => Err(*error),
        Ok(IconSizeShape::Scalable) => Ok(HtmlIconSize::Scalable),
        Ok(IconSizeShape::Bitmap { width, height }) => {
            let dimension = |range: std::ops::Range<usize>| {
                HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from_utf16(
                    units[range].to_vec(),
                ))
                .expect("canonical positive decimal proof")
            };
            HtmlBitmapIconSize::new(dimension(width.clone()), dimension(height.clone()))
                .map(HtmlIconSize::Bitmap)
        }
    }
}

fn rule(attribute: KnownHtmlAttribute, element: Option<KnownHtmlElement>) -> RuleMetadata {
    let (rows, source, context): (&'static [u16], &'static str, Context) = match attribute {
        KnownHtmlAttribute::Accesskey => {
            (&[4], source!("the-accesskey-attribute"), Context::Global)
        }
        KnownHtmlAttribute::Blocking => (
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
        KnownHtmlAttribute::Sizes => (
            &[144],
            source!("attr-link-sizes"),
            Context::Elements(&[KnownHtmlElement::Link]),
        ),
        _ => unreachable!("only declaration attribute rules enter this private owner"),
    };
    RuleMetadata {
        attribute,
        rows,
        source,
        context,
        vocabulary_source: None,
    }
}

impl HtmlValueInterpreter {
    fn declaration_input(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        attribute: KnownHtmlAttribute,
        required_role: Option<KnownHtmlElement>,
    ) -> Result<HtmlTokenDeclarationInput, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut input = report(
            rule(attribute, KnownHtmlElement::from_name(element)),
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
            Some(self.space_separated_tokens(raw.map(Attribute::value))?)
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAccessKeyReport, HtmlValueAdmissionError> {
        let mut input =
            self.declaration_input(element, raw, KnownHtmlAttribute::Accesskey, None)?;
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        self.blocking_observed(element, raw, None)
    }
    fn blocking_observed(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        required_role: Option<KnownHtmlElement>,
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        let mut input =
            self.declaration_input(element, raw, KnownHtmlAttribute::Blocking, required_role)?;
        if input.common.is_none() {
            return Ok(HtmlBlockingReport {
                input,
                explicit: None,
            });
        }
        let explicit = if input
            .tokens()
            .iter()
            .any(|t| ascii_keyword_units(t.value().code_units(), "render"))
        {
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
        element: &QualifiedName,
        attributes: &OrderedAttributes,
    ) -> Result<HtmlBlockingReport, HtmlValueAdmissionError> {
        let raw = selected(attributes, KnownHtmlAttribute::Blocking);
        let rel = selected(attributes, KnownHtmlAttribute::Rel);
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        self.admit_attribute_raw(rel.map(Attribute::value))?;
        let mut result = self.blocking_observed(element, raw, Some(KnownHtmlElement::Link))?;
        link_context(
            &mut result.input,
            rel,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlIconSizesReport, HtmlValueAdmissionError> {
        self.icon_sizes_observed(element, raw, None)
    }
    fn icon_sizes_observed(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        required_role: Option<KnownHtmlElement>,
    ) -> Result<HtmlIconSizesReport, HtmlValueAdmissionError> {
        let mut input =
            self.declaration_input(element, raw, KnownHtmlAttribute::Sizes, required_role)?;
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
            if let Some(keyword) = keywords.iter().find(|k| k.result.is_err()) {
                input.author = HtmlTokenDeclarationAuthorState::Nonconforming(
                    HtmlTokenDeclarationError::InvalidIconSize {
                        token_index: keyword.token_index,
                        reason: *keyword.result.as_ref().unwrap_err(),
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
        element: &QualifiedName,
        attributes: &OrderedAttributes,
    ) -> Result<HtmlIconSizesReport, HtmlValueAdmissionError> {
        let raw = selected(attributes, KnownHtmlAttribute::Sizes);
        let rel = selected(attributes, KnownHtmlAttribute::Rel);
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        self.admit_attribute_raw(rel.map(Attribute::value))?;
        let mut result = self.icon_sizes_observed(element, raw, Some(KnownHtmlElement::Link))?;
        link_context(
            &mut result.input,
            rel,
            &["icon", "apple-touch-icon"],
            HtmlAttributeContextConformanceError::IconSizesRequireIconRel,
        );
        if result.input.applicability() != HtmlAttributeApplicability::Applicable {
            result.keywords = None;
        }
        Ok(result)
    }
}
fn selected(attributes: &OrderedAttributes, name: KnownHtmlAttribute) -> Option<&Attribute> {
    let name = name.qualified_name();
    attributes.get(name.namespace(), name.local_name())
}
fn link_context(
    input: &mut HtmlTokenDeclarationInput,
    rel: Option<&Attribute>,
    required: &[&str],
    error: HtmlAttributeContextConformanceError,
) {
    input.rel = rel.cloned();
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
            crate::tokens::space_token_slices(rel.value()).any(|token| {
                required
                    .iter()
                    .any(|keyword| crate::rel::ascii_keyword(token, keyword))
            })
        });
        input.input.context_author = if matches {
            HtmlAttributeContextAuthorState::Conforming
        } else {
            HtmlAttributeContextAuthorState::Nonconforming(error)
        };
    }
}
