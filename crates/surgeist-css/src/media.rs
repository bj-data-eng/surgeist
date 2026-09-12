//! Checked media construction, lexical backing, and canonical authored serialization.
use crate::component_values::{CssCanonicalBuilder, CssCanonicalToken};
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
impl std::error::Error for CssMediaSerializationError {}
impl From<CssComponentValueError> for CssMediaSerializationError {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
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
            emit_feature(&mut out, &self.syntax, &[])?;
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
            for (index, query) in self.queries().iter().enumerate() {
                if index > 0 {
                    let origin = self
                        .comma_origins
                        .get(index - 1)
                        .unwrap_or(&CssValueOrigin::Programmatic);
                    out.push_grammar(CssCanonicalToken::Comma, origin)?;
                    space(&mut out, origin)?;
                }
                emit_query(&mut out, query)?;
            }
            Ok(out.finish()?)
        })
    }
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
    match query {
        CssMediaQuery::Never(value) => {
            return Err(CssMediaSerializationError::RecoveredNever {
                origin: value.origin().clone(),
            });
        }
        CssMediaQuery::Condition(value) => emit_condition(out, value)?,
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
            if value.media_type() == CssMediaType::Unknown {
                out.push_component(&value.syntax.media_type)?;
            } else {
                out.push_grammar(
                    CssCanonicalToken::Ident(value.media_type().name()),
                    value.syntax.media_type.origin(),
                )?;
            }
            if let Some(condition) = value.condition() {
                let origin = value
                    .syntax
                    .conjunction
                    .as_ref()
                    .expect("typed conjunction token");
                space(out, origin)?;
                out.push_grammar(CssCanonicalToken::Ident("and"), origin)?;
                space(out, origin)?;
                emit_condition(out, condition)?;
            }
        }
    }
    Ok(())
}
fn emit_condition(
    out: &mut CssCanonicalBuilder,
    condition: &CssMediaCondition,
) -> Result<(), CssMediaSerializationError> {
    match condition.kind() {
        CssMediaConditionKind::GeneralEnclosed(value) => out.push_component(value.component())?,
        CssMediaConditionKind::UnknownFeature(value) => emit_feature(out, &value.syntax, &[])?,
        CssMediaConditionKind::Feature(feature) => {
            let MediaConditionSyntax::Feature(syntax) = &condition.syntax else {
                unreachable!("known atom lexical syntax")
            };
            let defaults = match feature {
                CssMediaFeatureQuery::AspectRatio(range)
                | CssMediaFeatureQuery::DeviceAspectRatio(range) => ratio_defaults(range),
                _ => Vec::new(),
            };
            emit_feature(out, syntax, &defaults)?;
        }
        CssMediaConditionKind::Parenthesized(inner) => {
            let MediaConditionSyntax::Group { closing } = &condition.syntax else {
                unreachable!("group lexical syntax")
            };
            out.push_grammar(CssCanonicalToken::OpenParen, condition.origin())?;
            emit_condition(out, inner)?;
            out.push_grammar(CssCanonicalToken::CloseParen, closing)?;
        }
        CssMediaConditionKind::Not(inner) => {
            out.push_grammar(CssCanonicalToken::Ident("not"), condition.origin())?;
            space(out, condition.origin())?;
            emit_condition(out, inner)?;
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
                emit_condition(out, child)?;
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
) -> Result<(), CssComponentValueError> {
    if let Some(name) = &syntax.canonical_name {
        out.push_grammar(CssCanonicalToken::Ident(name), syntax.name.origin())
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
) -> Result<(), CssComponentValueError> {
    let CssComponentValueRef::Block(block) = syntax.component.view() else {
        unreachable!("feature parenthesis")
    };
    out.push_grammar(CssCanonicalToken::OpenParen, syntax.component.origin())?;
    let default = |index| defaults.get(index).copied().unwrap_or(false);
    match &syntax.shape {
        MediaFeatureShape::Boolean => emit_name(out, syntax)?,
        MediaFeatureShape::Range(range) => match range.view() {
            CssMediaRangeRef::Plain { value }
            | CssMediaRangeRef::Min { value }
            | CssMediaRangeRef::Max { value } => {
                emit_name(out, syntax)?;
                out.push_grammar(CssCanonicalToken::Colon, &syntax.separators[0][0])?;
                space(out, &syntax.separators[0][0])?;
                emit_value(out, value, default(0))?;
            }
            CssMediaRangeRef::FeatureFirst { comparison, value } => {
                emit_name(out, syntax)?;
                emit_comparison(out, comparison, &syntax.separators[0])?;
                emit_value(out, value, default(0))?;
            }
            CssMediaRangeRef::ValueFirst { value, comparison } => {
                emit_value(out, value, default(0))?;
                emit_comparison(out, comparison, &syntax.separators[0])?;
                emit_name(out, syntax)?;
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
                emit_name(out, syntax)?;
                emit_comparison(out, comparison(right_inclusive), &syntax.separators[1])?;
                emit_value(out, right, default(1))?;
            }
        },
    }
    out.push_grammar(CssCanonicalToken::CloseParen, block.closing_origin())
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
