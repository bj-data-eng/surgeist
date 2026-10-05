//! Checked media construction, lexical backing, and canonical authored serialization.
use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
use crate::specified_serialization::SpecifiedSerializationContext;
use crate::*;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaConstructionError {
    InvalidQueryGrammar { origin: CssValueOrigin },
    InvalidConditionGrammar { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
}
impl CssMediaConstructionError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::InvalidQueryGrammar { origin }
            | Self::InvalidConditionGrammar { origin }
            | Self::RecoveredInput { origin } => origin,
            Self::Component(error) => error.origin(),
        }
    }
}
impl fmt::Display for CssMediaConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid authored media construction: {self:?}")
    }
}
impl std::error::Error for CssMediaConstructionError {}
impl From<CssComponentValueError> for CssMediaConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaSerializationError {
    Component(CssComponentValueError),
    RecoveredNever { origin: CssValueOrigin },
}
impl CssMediaSerializationError {
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Component(error) => error.origin(),
            Self::RecoveredNever { origin } => origin,
        }
    }
}
impl fmt::Display for CssMediaSerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "media serialization failed: {self:?}")
    }
}
impl std::error::Error for CssMediaSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(error) => Some(error),
            Self::RecoveredNever { .. } => None,
        }
    }
}
impl From<CssComponentValueError> for CssMediaSerializationError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}

/// A canonical CSSOM component failure or exhausted cumulative resource.
/// Ignored grammar members have defined `not all` output and are not failures.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaCssomSerializationError {
    Media(CssMediaSerializationError),
    Resource {
        error: CssSpecifiedValueSerializationError,
        origin: CssValueOrigin,
    },
}

impl CssMediaCssomSerializationError {
    /// Returns original source or explicit programmatic provenance.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Media(error) => error.origin(),
            Self::Resource { origin, .. } => origin,
        }
    }
}

impl fmt::Display for CssMediaCssomSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Media(error) => error.fmt(formatter),
            Self::Resource { error, .. } => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CssMediaCssomSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Media(error) => error,
            Self::Resource { error, .. } => error,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUnknownMediaFeatureReason {
    UnknownName,
    InvalidValue,
    InvalidOperation,
}
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssUnknownMediaFeatureRef<'a> {
    Boolean,
    Range(&'a CssMediaRange<CssComponentValues>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MediaFeatureShape {
    Boolean,
    Range(CssMediaRange<CssComponentValues>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MediaFeatureSyntax {
    pub(crate) component: CssComponentValue,
    pub(crate) name: CssComponentValue,
    pub(crate) name_text: String,
    pub(crate) canonical_name: Option<String>,
    pub(crate) shape: MediaFeatureShape,
    pub(crate) separators: Vec<Vec<CssValueOrigin>>,
}
/// A structurally valid feature whose name, value, or operation has unknown truth.
#[derive(Clone, Debug, PartialEq)]
pub struct CssUnknownMediaFeature {
    syntax: Box<MediaFeatureSyntax>,
    reason: CssUnknownMediaFeatureReason,
}
impl CssUnknownMediaFeature {
    pub(crate) fn new(syntax: MediaFeatureSyntax, reason: CssUnknownMediaFeatureReason) -> Self {
        Self {
            syntax: Box::new(syntax),
            reason,
        }
    }
    pub fn name(&self) -> &str {
        &self.syntax.name_text
    }
    pub fn component(&self) -> &CssComponentValue {
        &self.syntax.component
    }
    pub fn origin(&self) -> &CssValueOrigin {
        self.component().origin()
    }
    pub fn position(&self) -> Option<CssSourcePosition> {
        parsed_position(self.origin())
    }
    pub fn authored(&self) -> Option<&str> {
        let origin = self.component().parsed_origin()?;
        origin.source().as_str().get(
            origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value(),
        )
    }
    pub const fn reason(&self) -> CssUnknownMediaFeatureReason {
        self.reason
    }
    pub fn view(&self) -> CssUnknownMediaFeatureRef<'_> {
        match &self.syntax.shape {
            MediaFeatureShape::Boolean => CssUnknownMediaFeatureRef::Boolean,
            MediaFeatureShape::Range(range) => CssUnknownMediaFeatureRef::Range(range),
        }
    }
    pub fn serialize(&self) -> Result<CssSerializedValue, CssMediaSerializationError> {
        with_media_stack(component_depth(self.component()) >= 64, || {
            let mut out = CssCanonicalBuilder::new(usize::MAX);
            emit_feature(&mut out, &self.syntax, &[], None, MediaOutput::Authored)?;
            Ok(out.finish()?)
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MediaConditionSyntax {
    Feature(Box<MediaFeatureSyntax>),
    Enclosed,
    Group { closing: CssValueOrigin },
    Not,
    Junction(Vec<CssValueOrigin>),
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MediaTypedSyntax {
    pub(crate) modifier: Option<CssComponentValue>,
    pub(crate) media_type: CssComponentValue,
    pub(crate) conjunction: Option<CssValueOrigin>,
}
pub(crate) const fn parsed_position(origin: &CssValueOrigin) -> Option<CssSourcePosition> {
    match origin {
        CssValueOrigin::Parsed(v) => Some(v.span().start()),
        _ => None,
    }
}

impl CssMediaQuery {
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssMediaConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssMediaConstructionError> {
        crate::parser::construct_media_query(values, limits)
    }
    pub fn serialize(&self) -> Result<CssSerializedValue, CssMediaSerializationError> {
        with_media_stack(query_is_deep(self), || {
            let mut out = CssCanonicalBuilder::new(usize::MAX);
            emit_query(&mut out, self)?;
            Ok(out.finish()?)
        })
    }

    /// Serializes canonical CSSOM text, projecting an ignored grammar member to
    /// `not all`. Authored syntax, diagnostics and origins remain unchanged.
    /// Valid unknown and opaque syntax is retained without host evaluation.
    pub fn serialize_cssom(&self) -> Result<CssSerializedValue, CssMediaCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares the specified-value input, projection and generated-byte policy.
    /// Queries, conditions and their backing components charge one node each;
    /// recovery additionally charges two projected keyword nodes.
    pub fn serialize_cssom_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<CssSerializedValue, CssMediaCssomSerializationError> {
        self.capture_cssom(&mut SpecifiedSerializationContext::new(limits))
    }

    /// Compares canonical CSSOM bytes case-sensitively, without media evaluation
    /// or treating authored model equality as CSSOM equality.
    pub fn cssom_equals(&self, other: &Self) -> Result<bool, CssMediaCssomSerializationError> {
        self.cssom_equals_with_limits(other, CssSpecifiedValueSerializationLimits::default())
    }

    /// Both serializations consume one cumulative budget, including their bytes.
    pub fn cssom_equals_with_limits(
        &self,
        other: &Self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<bool, CssMediaCssomSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let first = self.capture_cssom(&mut context)?;
        let mut bytes = String::new();
        context
            .append(&mut bytes, first.as_css())
            .map_err(|error| resource(error, self.origin()))?;
        let second = other.capture_cssom(&mut context)?;
        Ok(first.as_css() == second.as_css())
    }

    /// Captures bounded bytes without charging final emission. An enclosing
    /// writer appends the result through this same context, charging bytes once.
    pub(crate) fn capture_cssom(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> Result<CssSerializedValue, CssMediaCssomSerializationError> {
        charge_query(context, self)?;
        with_media_stack(query_is_deep(self), || {
            let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
            emit_query_with_mode(&mut out, self, MediaOutput::Cssom)?;
            Ok(out.finish()?)
        })
        .map_err(bounded_error)
    }
}
impl CssMediaCondition {
    pub fn try_from_components(
        values: CssComponentValues,
    ) -> Result<Self, CssMediaConstructionError> {
        Self::try_from_components_with_limits(values, CssComponentValueLimits::default())
    }
    pub fn try_from_components_with_limits(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssMediaConstructionError> {
        crate::parser::construct_media_condition(values, limits)
    }
    pub fn serialize(&self) -> Result<CssSerializedValue, CssMediaSerializationError> {
        with_media_stack(condition_is_deep(self), || {
            let mut out = CssCanonicalBuilder::new(usize::MAX);
            emit_condition(&mut out, self)?;
            Ok(out.finish()?)
        })
    }
}
impl CssMediaQueryList {
    pub fn serialize(&self) -> Result<CssSerializedValue, CssMediaSerializationError> {
        with_media_stack(self.queries().iter().any(query_is_deep), || {
            if let Some(CssMediaQuery::Never(value)) = self
                .queries()
                .iter()
                .find(|q| matches!(q, CssMediaQuery::Never(_)))
            {
                return Err(CssMediaSerializationError::RecoveredNever {
                    origin: value.origin().clone(),
                });
            }
            let mut out = CssCanonicalBuilder::new(usize::MAX);
            emit_list(&mut out, self, MediaOutput::Authored)?;
            Ok(out.finish()?)
        })
    }

    /// Preserves list order and projects each ignored grammar member to `not all`.
    /// The empty list emits empty text. No diagnostics or authored nodes are removed.
    pub fn serialize_cssom(&self) -> Result<CssSerializedValue, CssMediaCssomSerializationError> {
        self.serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges the list and all members to one cumulative resource policy.
    pub fn serialize_cssom_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<CssSerializedValue, CssMediaCssomSerializationError> {
        self.capture_cssom(&mut SpecifiedSerializationContext::new(limits))
    }

    /// Owns the list aggregate and selected member projections without allocating text.
    pub(crate) fn charge_cssom(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> Result<(), CssMediaCssomSerializationError> {
        charge_node(context, &CssValueOrigin::Programmatic)?;
        for query in self.queries() {
            charge_query(context, query)?;
        }
        Ok(())
    }

    /// An enclosing writer charges captured output bytes through the same context.
    pub(crate) fn capture_cssom(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> Result<CssSerializedValue, CssMediaCssomSerializationError> {
        self.charge_cssom(context)?;
        if context.output_suppressed() {
            return CssCanonicalBuilder::new(0)
                .finish()
                .map_err(|error| bounded_error(error.into()));
        }
        with_media_stack(self.queries().iter().any(query_is_deep), || {
            let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
            emit_list(&mut out, self, MediaOutput::Cssom)?;
            Ok(out.finish()?)
        })
        .map_err(bounded_error)
    }
}

#[derive(Clone, Copy)]
enum MediaOutput {
    Authored,
    Cssom,
}

fn emit_list(
    out: &mut CssCanonicalBuilder,
    list: &CssMediaQueryList,
    mode: MediaOutput,
) -> Result<(), CssMediaSerializationError> {
    for (index, query) in list.queries().iter().enumerate() {
        if index > 0 {
            let origin = list
                .comma_origins
                .get(index - 1)
                .unwrap_or(&CssValueOrigin::Programmatic);
            out.push_grammar(CssCanonicalToken::Comma, origin)?;
            space(out, origin)?;
        }
        emit_query_with_mode(out, query, mode)?;
    }
    Ok(())
}
fn space(
    out: &mut CssCanonicalBuilder,
    origin: &CssValueOrigin,
) -> Result<(), CssComponentValueError> {
    out.push_grammar(CssCanonicalToken::Whitespace, origin)
}
pub(crate) fn emit_query(
    out: &mut CssCanonicalBuilder,
    query: &CssMediaQuery,
) -> Result<(), CssMediaSerializationError> {
    emit_query_with_mode(out, query, MediaOutput::Authored)
}

pub(crate) fn emit_query_cssom(
    out: &mut CssCanonicalBuilder,
    query: &CssMediaQuery,
) -> Result<(), CssMediaSerializationError> {
    emit_query_with_mode(out, query, MediaOutput::Cssom)
}

fn emit_query_with_mode(
    out: &mut CssCanonicalBuilder,
    query: &CssMediaQuery,
    mode: MediaOutput,
) -> Result<(), CssMediaSerializationError> {
    match query {
        CssMediaQuery::Never(value) => {
            if matches!(mode, MediaOutput::Authored) {
                return Err(CssMediaSerializationError::RecoveredNever {
                    origin: value.origin().clone(),
                });
            }
            out.push_grammar(CssCanonicalToken::Ident("not"), value.origin())?;
            space(out, value.origin())?;
            out.push_grammar(CssCanonicalToken::Ident("all"), value.origin())?;
        }
        CssMediaQuery::Condition(value) => emit_condition_with_mode(out, value, mode)?,
        CssMediaQuery::Typed(value) => {
            if let Some(modifier) = value.modifier() {
                let origin = value
                    .syntax
                    .modifier
                    .as_ref()
                    .expect("typed modifier token")
                    .origin();
                out.push_grammar(
                    CssCanonicalToken::Ident(match modifier {
                        CssMediaQueryModifier::Not => "not",
                        CssMediaQueryModifier::Only => "only",
                    }),
                    origin,
                )?;
                space(out, origin)?;
            }
            let emit_type = value.condition().is_none()
                || value.modifier().is_some()
                || value.media_type() != CssMediaType::All;
            if emit_type {
                let name = match value.syntax.media_type.view() {
                    CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => name,
                    _ => unreachable!("checked media type identifier"),
                };
                out.push_grammar(
                    CssCanonicalToken::Ident(&name.to_ascii_lowercase()),
                    value.syntax.media_type.origin(),
                )?;
            }
            if let Some(condition) = value.condition() {
                if emit_type {
                    let origin = value
                        .syntax
                        .conjunction
                        .as_ref()
                        .expect("typed conjunction token");
                    space(out, origin)?;
                    out.push_grammar(CssCanonicalToken::Ident("and"), origin)?;
                    space(out, origin)?;
                }
                emit_condition_with_mode(out, condition, mode)?;
            }
        }
    }
    Ok(())
}
fn emit_condition(
    out: &mut CssCanonicalBuilder,
    condition: &CssMediaCondition,
) -> Result<(), CssMediaSerializationError> {
    emit_condition_with_mode(out, condition, MediaOutput::Authored)
}

fn emit_condition_with_mode(
    out: &mut CssCanonicalBuilder,
    condition: &CssMediaCondition,
    mode: MediaOutput,
) -> Result<(), CssMediaSerializationError> {
    match condition.kind() {
        CssMediaConditionKind::CustomMediaReference(value) => {
            out.push_component(value.component())?
        }
        CssMediaConditionKind::GeneralEnclosed(value) => out.push_component(value.component())?,
        CssMediaConditionKind::UnknownFeature(value) => {
            emit_feature(out, &value.syntax, &[], None, mode)?;
        }
        CssMediaConditionKind::Feature(feature) => {
            let MediaConditionSyntax::Feature(syntax) = &condition.syntax else {
                unreachable!("known atom lexical syntax")
            };
            let defaults = match feature {
                CssMediaFeatureQuery::AspectRatio(range)
                | CssMediaFeatureQuery::DeviceAspectRatio(range) => ratio_defaults(range),
                _ => Vec::new(),
            };
            let keyword = match feature {
                CssMediaFeatureQuery::Orientation(CssOrientation::Portrait) => Some("portrait"),
                CssMediaFeatureQuery::Orientation(CssOrientation::Landscape) => Some("landscape"),
                CssMediaFeatureQuery::Scan(CssScanMode::Progressive) => Some("progressive"),
                CssMediaFeatureQuery::Scan(CssScanMode::Interlace) => Some("interlace"),
                _ => None,
            };
            emit_feature(out, syntax, &defaults, keyword, mode)?;
        }
        CssMediaConditionKind::Parenthesized(inner) => {
            let MediaConditionSyntax::Group { closing } = &condition.syntax else {
                unreachable!("group lexical syntax")
            };
            out.push_grammar(CssCanonicalToken::OpenParen, condition.origin())?;
            emit_condition_with_mode(out, inner, mode)?;
            out.push_grammar(CssCanonicalToken::CloseParen, closing)?;
        }
        CssMediaConditionKind::Not(inner) => {
            out.push_grammar(CssCanonicalToken::Ident("not"), condition.origin())?;
            space(out, condition.origin())?;
            emit_condition_with_mode(out, inner, mode)?;
        }
        CssMediaConditionKind::And(children) | CssMediaConditionKind::Or(children) => {
            let MediaConditionSyntax::Junction(operators) = &condition.syntax else {
                unreachable!("junction lexical syntax")
            };
            let keyword = if matches!(condition.kind(), CssMediaConditionKind::And(_)) {
                "and"
            } else {
                "or"
            };
            for (i, child) in children.conditions().iter().enumerate() {
                if i > 0 {
                    let origin = &operators[i - 1];
                    space(out, origin)?;
                    out.push_grammar(CssCanonicalToken::Ident(keyword), origin)?;
                    space(out, origin)?;
                }
                emit_condition_with_mode(out, child, mode)?;
            }
        }
    }
    Ok(())
}
fn ratio_defaults(range: &CssMediaRange<CssMediaRatio>) -> Vec<bool> {
    match range.view() {
        CssMediaRangeRef::Plain { value }
        | CssMediaRangeRef::Min { value }
        | CssMediaRangeRef::Max { value }
        | CssMediaRangeRef::FeatureFirst { value, .. }
        | CssMediaRangeRef::ValueFirst { value, .. } => vec![value.denominator_is_omitted()],
        CssMediaRangeRef::Ascending { left, right, .. }
        | CssMediaRangeRef::Descending { left, right, .. } => vec![
            left.denominator_is_omitted(),
            right.denominator_is_omitted(),
        ],
    }
}
fn emit_name(
    out: &mut CssCanonicalBuilder,
    syntax: &MediaFeatureSyntax,
    mode: MediaOutput,
) -> Result<(), CssComponentValueError> {
    if let Some(name) = &syntax.canonical_name {
        out.push_grammar(CssCanonicalToken::Ident(name), syntax.name.origin())
    } else if matches!(mode, MediaOutput::Cssom) {
        out.push_grammar(
            CssCanonicalToken::Ident(&syntax.name_text.to_ascii_lowercase()),
            syntax.name.origin(),
        )
    } else {
        out.push_component(&syntax.name)
    }
}
fn emit_comparison(
    out: &mut CssCanonicalBuilder,
    comparison: CssQueryComparison,
    origins: &[CssValueOrigin],
) -> Result<(), CssComponentValueError> {
    let origin = &origins[0];
    space(out, origin)?;
    let (symbol, inclusive) = match comparison {
        CssQueryComparison::LessThan => ('<', false),
        CssQueryComparison::LessThanOrEqual => ('<', true),
        CssQueryComparison::Equal => ('=', false),
        CssQueryComparison::GreaterThanOrEqual => ('>', true),
        CssQueryComparison::GreaterThan => ('>', false),
    };
    out.push_grammar(CssCanonicalToken::Delim(symbol), origin)?;
    if inclusive {
        out.push_grammar(
            CssCanonicalToken::Delim('='),
            origins.get(1).unwrap_or(origin),
        )?;
    }
    space(out, origin)
}
fn emit_value(
    out: &mut CssCanonicalBuilder,
    values: &CssComponentValues,
    default_denominator: bool,
) -> Result<(), CssComponentValueError> {
    let items = values.items();
    let first = items
        .iter()
        .position(|c| {
            !matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .unwrap_or(items.len());
    let end = items
        .iter()
        .rposition(|c| {
            !matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .map_or(first, |v| v + 1);
    let items = &items[first..end];
    for (i, component) in items.iter().enumerate() {
        if matches!(
            component.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
        ) && (items.get(i.wrapping_sub(1)).is_some_and(is_slash)
            || items.get(i + 1).is_some_and(is_slash))
        {
            continue;
        }
        if is_slash(component) {
            space(out, component.origin())?;
            out.push_component(component)?;
            space(out, component.origin())?;
        } else {
            out.push_component(component)?;
        }
    }
    if default_denominator {
        let origin = CssValueOrigin::Programmatic;
        space(out, &origin)?;
        out.push_grammar(CssCanonicalToken::Delim('/'), &origin)?;
        space(out, &origin)?;
        out.push_component(&CssComponentValue::try_number("1")?)?;
    }
    Ok(())
}
fn is_slash(component: &CssComponentValue) -> bool {
    matches!(
        component.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Delim('/'))
    )
}
fn emit_feature(
    out: &mut CssCanonicalBuilder,
    syntax: &MediaFeatureSyntax,
    defaults: &[bool],
    keyword: Option<&str>,
    mode: MediaOutput,
) -> Result<(), CssComponentValueError> {
    let CssComponentValueRef::Block(block) = syntax.component.view() else {
        unreachable!("feature parenthesis")
    };
    out.push_grammar(CssCanonicalToken::OpenParen, syntax.component.origin())?;
    let default = |index| defaults.get(index).copied().unwrap_or(false);
    match &syntax.shape {
        MediaFeatureShape::Boolean => emit_name(out, syntax, mode)?,
        MediaFeatureShape::Range(range) => match range.view() {
            CssMediaRangeRef::Plain { value }
            | CssMediaRangeRef::Min { value }
            | CssMediaRangeRef::Max { value } => {
                emit_name(out, syntax, mode)?;
                out.push_grammar(CssCanonicalToken::Colon, &syntax.separators[0][0])?;
                space(out, &syntax.separators[0][0])?;
                if let Some(keyword) = keyword {
                    let origin = value
                        .items()
                        .iter()
                        .find(|component| {
                            matches!(
                                component.view(),
                                CssComponentValueRef::Token(CssValueTokenRef::Ident(_))
                            )
                        })
                        .expect("checked discrete feature has a value")
                        .origin();
                    out.push_grammar(CssCanonicalToken::Ident(keyword), origin)?;
                } else {
                    emit_value(out, value, default(0))?;
                }
            }
            CssMediaRangeRef::FeatureFirst { comparison, value } => {
                emit_name(out, syntax, mode)?;
                emit_comparison(out, comparison, &syntax.separators[0])?;
                emit_value(out, value, default(0))?;
            }
            CssMediaRangeRef::ValueFirst { value, comparison } => {
                emit_value(out, value, default(0))?;
                emit_comparison(out, comparison, &syntax.separators[0])?;
                emit_name(out, syntax, mode)?;
            }
            CssMediaRangeRef::Ascending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            }
            | CssMediaRangeRef::Descending {
                left,
                left_inclusive,
                right,
                right_inclusive,
            } => {
                let ascending = matches!(range.view(), CssMediaRangeRef::Ascending { .. });
                let comparison = |inclusive| match (ascending, inclusive) {
                    (true, true) => CssQueryComparison::LessThanOrEqual,
                    (true, false) => CssQueryComparison::LessThan,
                    (false, true) => CssQueryComparison::GreaterThanOrEqual,
                    (false, false) => CssQueryComparison::GreaterThan,
                };
                emit_value(out, left, default(0))?;
                emit_comparison(out, comparison(left_inclusive), &syntax.separators[0])?;
                emit_name(out, syntax, mode)?;
                emit_comparison(out, comparison(right_inclusive), &syntax.separators[1])?;
                emit_value(out, right, default(1))?;
            }
        },
    }
    out.push_grammar(CssCanonicalToken::CloseParen, block.closing_origin())
}

fn resource(
    error: CssSpecifiedValueSerializationError,
    origin: &CssValueOrigin,
) -> CssMediaCssomSerializationError {
    CssMediaCssomSerializationError::Resource {
        error,
        origin: origin.clone(),
    }
}

pub(crate) fn bounded_error(error: CssMediaSerializationError) -> CssMediaCssomSerializationError {
    if let CssMediaSerializationError::Component(component) = &error {
        let kind = match component.kind() {
            CssComponentValueErrorKind::ByteLimit => {
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            }
            CssComponentValueErrorKind::CapacityOverflow => {
                CssSpecifiedValueSerializationErrorKind::CapacityOverflow
            }
            _ => return CssMediaCssomSerializationError::Media(error),
        };
        return resource(
            CssSpecifiedValueSerializationError::new(kind),
            component.origin(),
        );
    }
    CssMediaCssomSerializationError::Media(error)
}

fn charge_node(
    context: &mut SpecifiedSerializationContext,
    origin: &CssValueOrigin,
) -> Result<(), CssMediaCssomSerializationError> {
    context
        .charge_input(1)
        .and_then(|()| context.charge_projection(1))
        .map_err(|error| resource(error, origin))
}

// Bound normalization scratch before allocating a lowercase copy. An identifier's
// escaped output cannot be shorter than its decoded UTF-8 spelling.
fn check_identifier_size(
    context: &SpecifiedSerializationContext,
    name: &str,
    origin: &CssValueOrigin,
) -> Result<(), CssMediaCssomSerializationError> {
    if !context.output_suppressed() && name.len() > context.remaining_bytes() {
        return Err(resource(
            CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            origin,
        ));
    }
    Ok(())
}

fn charge_query(
    context: &mut SpecifiedSerializationContext,
    query: &CssMediaQuery,
) -> Result<(), CssMediaCssomSerializationError> {
    enum Visit<'a> {
        Condition(&'a CssMediaCondition),
        Conditions(&'a [CssMediaCondition]),
        Component(&'a CssComponentValue),
        Components(&'a [CssComponentValue]),
    }
    charge_node(context, query.origin())?;
    let mut pending = Vec::new();
    match query {
        CssMediaQuery::Never(value) => {
            context
                .charge_projection(2)
                .map_err(|error| resource(error, value.origin()))?;
        }
        CssMediaQuery::Condition(condition) => pending.push(Visit::Condition(condition)),
        CssMediaQuery::Typed(value) => {
            if value.condition().is_none()
                || value.modifier().is_some()
                || value.media_type() != CssMediaType::All
            {
                let CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) =
                    value.syntax.media_type.view()
                else {
                    unreachable!("checked media type identifier")
                };
                check_identifier_size(context, name, value.syntax.media_type.origin())?;
            }
            pending.push(Visit::Component(&value.syntax.media_type));
            if let Some(modifier) = &value.syntax.modifier {
                pending.push(Visit::Component(modifier));
            }
            if let Some(condition) = value.condition() {
                pending.push(Visit::Condition(condition));
            }
        }
    }
    while let Some(visit) = pending.pop() {
        match visit {
            Visit::Conditions(conditions) => {
                if let Some((first, rest)) = conditions.split_first() {
                    pending.push(Visit::Conditions(rest));
                    pending.push(Visit::Condition(first));
                }
            }
            Visit::Components(components) => {
                if let Some((first, rest)) = components.split_first() {
                    pending.push(Visit::Components(rest));
                    pending.push(Visit::Component(first));
                }
            }
            Visit::Condition(condition) => {
                charge_node(context, condition.origin())?;
                match condition.kind() {
                    CssMediaConditionKind::CustomMediaReference(value) => {
                        pending.push(Visit::Component(value.component()));
                    }
                    CssMediaConditionKind::GeneralEnclosed(value) => {
                        pending.push(Visit::Component(value.component()));
                    }
                    CssMediaConditionKind::UnknownFeature(value) => {
                        check_identifier_size(
                            context,
                            &value.syntax.name_text,
                            value.syntax.name.origin(),
                        )?;
                        pending.push(Visit::Component(value.component()));
                    }
                    CssMediaConditionKind::Feature(_) => {
                        let MediaConditionSyntax::Feature(syntax) = &condition.syntax else {
                            unreachable!("known atom lexical syntax")
                        };
                        pending.push(Visit::Component(&syntax.component));
                    }
                    CssMediaConditionKind::Parenthesized(inner)
                    | CssMediaConditionKind::Not(inner) => {
                        pending.push(Visit::Condition(inner));
                    }
                    CssMediaConditionKind::And(children) | CssMediaConditionKind::Or(children) => {
                        pending.push(Visit::Conditions(children.conditions()));
                    }
                }
            }
            Visit::Component(component) => {
                charge_node(context, component.origin())?;
                match component.view() {
                    CssComponentValueRef::Function(value) => {
                        pending.push(Visit::Components(value.values().items()));
                    }
                    CssComponentValueRef::Block(value) => {
                        pending.push(Visit::Components(value.values().items()));
                    }
                    CssComponentValueRef::Token(_) | CssComponentValueRef::Comment(_) => {}
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn check_canonical_limit(
    query: &CssMediaQuery,
    max_bytes: usize,
) -> Result<(), CssMediaSerializationError> {
    let mut out = CssCanonicalBuilder::new(max_bytes);
    emit_query(&mut out, query)?;
    out.finish()?;
    Ok(())
}

pub(crate) fn with_media_stack<T: Send>(deep: bool, action: impl FnOnce() -> T + Send) -> T {
    if !deep {
        return action();
    }
    std::thread::scope(|scope| {
        let task = std::thread::Builder::new()
            .name("surgeist-css-media".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, action)
            .expect("bounded media parser thread must be available");
        match task.join() {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}
pub(crate) fn query_is_deep(query: &CssMediaQuery) -> bool {
    match query {
        CssMediaQuery::Condition(value) => condition_is_deep(value),
        CssMediaQuery::Typed(value) => value.condition().is_some_and(condition_is_deep),
        CssMediaQuery::Never(_) => false,
    }
}
fn condition_is_deep(root: &CssMediaCondition) -> bool {
    let mut pending = vec![(root, 0usize)];
    while let Some((value, depth)) = pending.pop() {
        if depth >= 64 {
            return true;
        }
        match value.kind() {
            CssMediaConditionKind::CustomMediaReference(_) => {}
            CssMediaConditionKind::Parenthesized(v) | CssMediaConditionKind::Not(v) => {
                pending.push((v, depth + 1))
            }
            CssMediaConditionKind::And(v) | CssMediaConditionKind::Or(v) => {
                pending.extend(v.conditions().iter().map(|v| (v, depth + 1)))
            }
            CssMediaConditionKind::GeneralEnclosed(v) => {
                if component_depth(v.component()) >= 64 {
                    return true;
                }
            }
            CssMediaConditionKind::UnknownFeature(v) => {
                if component_depth(v.component()) >= 64 {
                    return true;
                }
            }
            CssMediaConditionKind::Feature(_) => {
                if let MediaConditionSyntax::Feature(syntax) = &value.syntax
                    && component_depth(&syntax.component) >= 64
                {
                    return true;
                }
            }
        }
    }
    false
}
pub(crate) fn component_depth(component: &CssComponentValue) -> u32 {
    match component.view() {
        CssComponentValueRef::Function(v) => v.values().nesting_depth() + 1,
        CssComponentValueRef::Block(v) => v.values().nesting_depth() + 1,
        _ => 0,
    }
}
