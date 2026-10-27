//! Checked media construction, lexical backing, and canonical authored serialization.
use crate::component::values::CssComponentValueErrorKind;
use crate::component::values::serialize::{CssCanonicalBuilder, CssCanonicalToken};
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssSerializedValue, CssValueOrigin, CssValueTokenRef,
};
use crate::query::media::{
    CssMediaFeatureQuery, CssMediaLeaf, CssMediaQuery, CssMediaQueryList, CssMediaQueryModifier,
    CssMediaRange, CssMediaRangeRef, CssMediaRatio, CssMediaType, CssOrientation, CssScanMode,
};
use crate::query::{CssCondition, CssConditionKind, CssQueryComparison};
use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::serialization::{CssSerializationError, CssSerializationLimits};
use crate::source::CssSourcePosition;
use std::fmt;

/// A media query, condition or custom-media definition that failed checked
/// construction through its grammar, recovered input or a resource.
///
/// The failure is read through the `is_*` predicates and located by [`Self::origin`]; a
/// rejected component is exposed by [`Self::component_error`] and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssMediaConstructionError {
    kind: MediaConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MediaConstructionKind {
    InvalidQueryGrammar { origin: CssValueOrigin },
    InvalidConditionGrammar { origin: CssValueOrigin },
    InvalidName { origin: CssValueOrigin },
    InvalidPrelude { origin: CssValueOrigin },
    AmbiguousMediaBody { origin: CssValueOrigin },
    RecoveredInput { origin: CssValueOrigin },
    Component(CssComponentValueError),
}
impl CssMediaConstructionError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether the components do not form a media query.
        is_invalid_query_grammar => MediaConstructionKind::InvalidQueryGrammar { .. },
        /// Returns whether the components do not form a media condition.
        is_invalid_condition_grammar => MediaConstructionKind::InvalidConditionGrammar { .. },
        /// Returns whether a custom-media name is not a `<dashed-ident>`.
        is_invalid_name => MediaConstructionKind::InvalidName { .. },
        /// Returns whether a custom-media prelude does not satisfy its grammar.
        is_invalid_prelude => MediaConstructionKind::InvalidPrelude { .. },
        /// Returns whether a custom-media body is ambiguous between its media and boolean
        /// forms.
        is_ambiguous_media_body => MediaConstructionKind::AmbiguousMediaBody { .. },
        /// Returns whether an implicitly closed component was supplied.
        is_recovered_input => MediaConstructionKind::RecoveredInput { .. },
    }
    /// Returns the rejected component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&CssComponentValueError> {
        match &self.kind {
            MediaConstructionKind::Component(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the responsible original or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match &self.kind {
            MediaConstructionKind::InvalidQueryGrammar { origin }
            | MediaConstructionKind::InvalidConditionGrammar { origin }
            | MediaConstructionKind::InvalidName { origin }
            | MediaConstructionKind::InvalidPrelude { origin }
            | MediaConstructionKind::AmbiguousMediaBody { origin }
            | MediaConstructionKind::RecoveredInput { origin } => origin,
            MediaConstructionKind::Component(error) => error.origin(),
        }
    }
}
impl From<MediaConstructionKind> for CssMediaConstructionError {
    fn from(kind: MediaConstructionKind) -> Self {
        Self { kind }
    }
}
impl fmt::Display for CssMediaConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid authored media construction: ")?;
        f.write_str(match &self.kind {
            MediaConstructionKind::InvalidQueryGrammar { .. } => {
                "the components do not form a media query"
            }
            MediaConstructionKind::InvalidConditionGrammar { .. } => {
                "the components do not form a media condition"
            }
            MediaConstructionKind::InvalidName { .. } => "the custom-media name is invalid",
            MediaConstructionKind::InvalidPrelude { .. } => "the custom-media prelude is invalid",
            MediaConstructionKind::AmbiguousMediaBody { .. } => {
                "the custom-media body is ambiguous"
            }
            MediaConstructionKind::RecoveredInput { .. } => {
                "an implicitly closed component is not admitted"
            }
            MediaConstructionKind::Component(_) => "a component is invalid",
        })
    }
}
impl std::error::Error for CssMediaConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<CssComponentValueError> for CssMediaConstructionError {
    fn from(error: CssComponentValueError) -> Self {
        Self {
            kind: MediaConstructionKind::Component(error),
        }
    }
}
/// A lexical media emission failure, before its public classification.
///
/// Construction maps it to its own errors; public serializers map it to
/// [`CssSerializationError`] through [`media_error`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MediaSerializationFailure {
    Component(CssComponentValueError),
    RecoveredNever { origin: CssValueOrigin },
}
impl From<CssComponentValueError> for MediaSerializationFailure {
    fn from(error: CssComponentValueError) -> Self {
        Self::Component(error)
    }
}

/// Why a structurally valid media feature has unknown truth instead of being a recognized feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssUnknownMediaFeatureReason {
    /// The feature name is not a recognized media feature.
    UnknownName,
    /// The name is recognized but the value does not match that feature's grammar.
    InvalidValue,
    /// The name is recognized but the operation is not allowed for it, such as a range comparison on a discrete feature.
    InvalidOperation,
}
/// Borrowed view of the authored shape of a [`CssUnknownMediaFeature`].
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssUnknownMediaFeatureRef<'a> {
    /// The feature was written as a name alone.
    Boolean,
    /// The feature was written with a value or range; the operands are kept as authored component values.
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
    /// Returns the authored feature name text.
    pub fn name(&self) -> &str {
        &self.syntax.name_text
    }
    /// Returns the original parenthesized feature component.
    pub fn component(&self) -> &CssComponentValue {
        &self.syntax.component
    }
    /// Returns the origin of the feature component.
    pub fn origin(&self) -> &CssValueOrigin {
        self.component().origin()
    }
    /// Returns the source position of the feature, or `None` when its origin was not parsed from source.
    pub fn position(&self) -> Option<CssSourcePosition> {
        parsed_position(self.origin())
    }
    /// Returns the feature's original source text, or `None` when it was not parsed from source.
    pub fn authored(&self) -> Option<&str> {
        let origin = self.component().parsed_origin()?;
        origin.source().as_str().get(
            origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value(),
        )
    }
    /// Returns why the feature has unknown truth.
    pub const fn reason(&self) -> CssUnknownMediaFeatureReason {
        self.reason
    }
    /// Returns whether the feature was a bare name or a range, with its authored operands.
    pub fn view(&self) -> CssUnknownMediaFeatureRef<'_> {
        match &self.syntax.shape {
            MediaFeatureShape::Boolean => CssUnknownMediaFeatureRef::Boolean,
            MediaFeatureShape::Range(range) => CssUnknownMediaFeatureRef::Range(range),
        }
    }
    /// Serializes authored media syntax as tokens with their original origins.
    ///
    /// A recovered member fails the whole operation. `limits.max_css_bytes()`
    /// caps the output; the other component limits do not apply.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] for a recovered member or an exhausted
    /// byte limit; no partial output is returned.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        let origin = || self.component().origin().clone();
        with_media_stack(component_depth(self.component()) >= 64, origin, || {
            let mut out = CssCanonicalBuilder::new(limits.max_css_bytes());
            emit_feature(&mut out, &self.syntax, &[], None, MediaOutput::Authored)?;
            Ok(out.finish()?)
        })
        .map_err(media_error)
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
    /// Constructs a media query from authored component values.
    ///
    /// # Errors
    ///
    /// Returns [`CssMediaConstructionError`] when the components do not form one media query, contain an implicitly closed component, or violate `limits`.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssMediaConstructionError> {
        crate::parser::queries::construct_media_query(values, limits)
    }
    /// Serializes authored media syntax as tokens with their original origins.
    ///
    /// A recovered member fails the whole operation. `limits.max_css_bytes()`
    /// caps the output; the other component limits do not apply.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] for a recovered member or an exhausted
    /// byte limit; no partial output is returned.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        with_media_stack(
            query_is_deep(self),
            || self.origin().clone(),
            || {
                let mut out = CssCanonicalBuilder::new(limits.max_css_bytes());
                emit_query(&mut out, self)?;
                Ok(out.finish()?)
            },
        )
        .map_err(media_error)
    }

    /// Serializes canonical CSSOM text, projecting an ignored grammar member to
    /// `not all`. Authored syntax, diagnostics and origins remain unchanged.
    /// Valid unknown and opaque syntax is retained without host evaluation.
    ///
    /// Shares the specified-value input, projection and generated-byte policy.
    /// Queries, conditions and their backing components charge one node each;
    /// recovery additionally charges two projected keyword nodes.
    pub fn serialize_cssom(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        self.capture_cssom(&mut SpecifiedSerializationContext::new(limits))
    }

    /// Compares canonical CSSOM bytes case-sensitively, without media evaluation
    /// or treating authored model equality as CSSOM equality.
    ///
    /// Both serializations consume one cumulative budget, including their bytes.
    pub fn cssom_equals(
        &self,
        other: &Self,
        limits: CssSerializationLimits,
    ) -> Result<bool, CssSerializationError> {
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
    ) -> Result<CssSerializedValue, CssSerializationError> {
        charge_query(context, self)?;
        with_media_stack(
            query_is_deep(self),
            || self.origin().clone(),
            || {
                let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
                emit_query_with_mode(&mut out, self, MediaOutput::Cssom)?;
                Ok(out.finish()?)
            },
        )
        .map_err(media_error)
    }
}
impl CssCondition<CssMediaLeaf> {
    /// Constructs a media condition from authored component values.
    ///
    /// # Errors
    ///
    /// Returns [`CssMediaConstructionError`] when the components do not form one media condition, contain an implicitly closed component, or violate `limits`.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssMediaConstructionError> {
        crate::parser::queries::construct_media_condition(values, limits)
    }
    /// Serializes authored media syntax as tokens with their original origins.
    ///
    /// A recovered member fails the whole operation. `limits.max_css_bytes()`
    /// caps the output; the other component limits do not apply.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] for a recovered member or an exhausted
    /// byte limit; no partial output is returned.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        with_media_stack(
            condition_is_deep(self),
            || self.origin().clone(),
            || {
                let mut out = CssCanonicalBuilder::new(limits.max_css_bytes());
                emit_condition(&mut out, self)?;
                Ok(out.finish()?)
            },
        )
        .map_err(media_error)
    }
}
impl CssMediaQueryList {
    /// Serializes authored media syntax as tokens with their original origins.
    ///
    /// A recovered member fails the whole operation. `limits.max_css_bytes()`
    /// caps the output; the other component limits do not apply.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] for a recovered member or an exhausted
    /// byte limit; no partial output is returned.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        let origin = || deep_list_origin(self);
        with_media_stack(self.queries().iter().any(query_is_deep), origin, || {
            if let Some(CssMediaQuery::Never(value)) = self
                .queries()
                .iter()
                .find(|q| matches!(q, CssMediaQuery::Never(_)))
            {
                return Err(MediaSerializationFailure::RecoveredNever {
                    origin: value.origin().clone(),
                });
            }
            let mut out = CssCanonicalBuilder::new(limits.max_css_bytes());
            emit_list(&mut out, self, MediaOutput::Authored)?;
            Ok(out.finish()?)
        })
        .map_err(media_error)
    }

    /// Preserves list order and projects each ignored grammar member to `not all`.
    /// The empty list emits empty text. No diagnostics or authored nodes are removed.
    ///
    /// Charges the list and all members to one cumulative resource policy.
    pub fn serialize_cssom(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        self.capture_cssom(&mut SpecifiedSerializationContext::new(limits))
    }

    /// Returns separate canonical member captures in list order under one budget.
    /// Duplicates and recovered members remain present; no separators are emitted.
    ///
    /// Charges the collection aggregate, every query projection and all captured
    /// bytes cumulatively. Failure returns no partial collection.
    pub fn serialize_cssom_members(
        &self,
        limits: CssSerializationLimits,
    ) -> Result<Vec<CssSerializedValue>, CssSerializationError> {
        let mut context = SpecifiedSerializationContext::new(limits);
        charge_node(&mut context, &CssValueOrigin::Programmatic)?;
        let mut captures = Vec::new();
        for query in self.queries() {
            let capture = query.capture_cssom(&mut context)?;
            context
                .charge_captured_bytes(capture.as_css().len())
                .map_err(|error| resource(error, query.origin()))?;
            captures.try_reserve(1).map_err(|_| {
                resource(
                    CssSerializationError::new(
                        CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                    ),
                    query.origin(),
                )
            })?;
            captures.push(capture);
        }
        Ok(captures)
    }

    /// Owns the list aggregate and selected member projections without allocating text.
    pub(crate) fn charge_cssom(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> Result<(), CssSerializationError> {
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
    ) -> Result<CssSerializedValue, CssSerializationError> {
        self.capture_with_mode(context, MediaOutput::Cssom)
    }

    /// Generic specified rule-graph projection under the same charging policy:
    /// recovered members become `not all` and the authored `only` prefix remains.
    pub(crate) fn capture_projected(
        &self,
        context: &mut SpecifiedSerializationContext,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        self.capture_with_mode(context, MediaOutput::Projected)
    }

    fn capture_with_mode(
        &self,
        context: &mut SpecifiedSerializationContext,
        mode: MediaOutput,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        self.charge_cssom(context)?;
        if context.output_suppressed() {
            return CssCanonicalBuilder::new(0)
                .finish()
                .map_err(|error| media_error(error.into()));
        }
        let origin = || deep_list_origin(self);
        with_media_stack(self.queries().iter().any(query_is_deep), origin, || {
            let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
            emit_list(&mut out, self, mode)?;
            Ok(out.finish()?)
        })
        .map_err(media_error)
    }
}

/// `Authored` rejects recovered members and keeps every authored prefix.
/// `Projected` is the generic specified rule-graph projection: it shares CSSOM's
/// `not all` recovery and canonical names but keeps the authored `only` prefix.
/// `Cssom` follows "serialize a media query", which never emits `only`.
#[derive(Clone, Copy)]
enum MediaOutput {
    Authored,
    Projected,
    Cssom,
}

fn emit_list(
    out: &mut CssCanonicalBuilder,
    list: &CssMediaQueryList,
    mode: MediaOutput,
) -> Result<(), MediaSerializationFailure> {
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
) -> Result<(), MediaSerializationFailure> {
    emit_query_with_mode(out, query, MediaOutput::Authored)
}

pub(crate) fn emit_query_cssom(
    out: &mut CssCanonicalBuilder,
    query: &CssMediaQuery,
) -> Result<(), MediaSerializationFailure> {
    emit_query_with_mode(out, query, MediaOutput::Cssom)
}

/// Generic specified projection: recovered members become `not all`, while the
/// authored `only` prefix is retained.
pub(crate) fn emit_query_projected(
    out: &mut CssCanonicalBuilder,
    query: &CssMediaQuery,
) -> Result<(), MediaSerializationFailure> {
    emit_query_with_mode(out, query, MediaOutput::Projected)
}

fn emit_query_with_mode(
    out: &mut CssCanonicalBuilder,
    query: &CssMediaQuery,
    mode: MediaOutput,
) -> Result<(), MediaSerializationFailure> {
    match query {
        CssMediaQuery::Never(value) => {
            if matches!(mode, MediaOutput::Authored) {
                return Err(MediaSerializationFailure::RecoveredNever {
                    origin: value.origin().clone(),
                });
            }
            out.push_grammar(CssCanonicalToken::Ident("not"), value.origin())?;
            space(out, value.origin())?;
            out.push_grammar(CssCanonicalToken::Ident("all"), value.origin())?;
        }
        CssMediaQuery::Condition(value) => emit_condition_with_mode(out, value, mode)?,
        CssMediaQuery::Typed(value) => {
            // CSSOM "serialize a media query" emits `not` for a negated query and
            // never emits `only` (#1378); authored output keeps the authored prefix.
            let modifier = value.modifier().filter(|modifier| {
                !matches!(mode, MediaOutput::Cssom) || *modifier == CssMediaQueryModifier::Not
            });
            if let Some(modifier) = modifier {
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
            let emit_type =
                typed_query_emits_type(value.condition().is_none(), modifier, value.media_type());
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
/// Step 3/4 of CSSOM "serialize a media query": the type is emitted alone when
/// there is no condition, and before `and` only when it is not `all` or the
/// query keeps an emitted prefix.
fn typed_query_emits_type(
    no_condition: bool,
    emitted_modifier: Option<CssMediaQueryModifier>,
    media_type: CssMediaType,
) -> bool {
    no_condition || emitted_modifier.is_some() || media_type != CssMediaType::All
}
fn emit_condition(
    out: &mut CssCanonicalBuilder,
    condition: &CssCondition<CssMediaLeaf>,
) -> Result<(), MediaSerializationFailure> {
    emit_condition_with_mode(out, condition, MediaOutput::Authored)
}

fn emit_condition_with_mode(
    out: &mut CssCanonicalBuilder,
    condition: &CssCondition<CssMediaLeaf>,
    mode: MediaOutput,
) -> Result<(), MediaSerializationFailure> {
    match condition.kind() {
        CssConditionKind::Leaf(CssMediaLeaf::CustomMediaReference(value)) => {
            out.push_component(value.component())?
        }
        CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(value)) => {
            out.push_component(value.component())?
        }
        CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(value)) => {
            emit_feature(out, &value.syntax, &[], None, mode)?;
        }
        CssConditionKind::Leaf(CssMediaLeaf::Feature(feature)) => {
            let MediaConditionSyntax::Feature(syntax) = condition.media_syntax() else {
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
        CssConditionKind::Parenthesized(inner) => {
            let MediaConditionSyntax::Group { closing } = condition.media_syntax() else {
                unreachable!("group lexical syntax")
            };
            out.push_grammar(CssCanonicalToken::OpenParen, condition.origin())?;
            emit_condition_with_mode(out, inner, mode)?;
            out.push_grammar(CssCanonicalToken::CloseParen, closing)?;
        }
        CssConditionKind::Not(inner) => {
            out.push_grammar(CssCanonicalToken::Ident("not"), condition.origin())?;
            space(out, condition.origin())?;
            emit_condition_with_mode(out, inner, mode)?;
        }
        CssConditionKind::And(children) | CssConditionKind::Or(children) => {
            let MediaConditionSyntax::Junction(operators) = condition.media_syntax() else {
                unreachable!("junction lexical syntax")
            };
            let keyword = if matches!(condition.kind(), CssConditionKind::And(_)) {
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
    } else if matches!(mode, MediaOutput::Cssom | MediaOutput::Projected) {
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
    let closing = match syntax.component.view() {
        CssComponentValueRef::Block(block) => {
            out.push_grammar(CssCanonicalToken::OpenParen, syntax.component.origin())?;
            block.closing_origin()
        }
        CssComponentValueRef::Function(function) => {
            out.push_grammar(
                CssCanonicalToken::Function(function.name()),
                syntax.component.origin(),
            )?;
            function.closing_origin()
        }
        _ => unreachable!("checked feature enclosure"),
    };
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
    out.push_grammar(CssCanonicalToken::CloseParen, closing)
}

fn resource(error: CssSerializationError, origin: &CssValueOrigin) -> CssSerializationError {
    error.at_origin(origin)
}

/// Classifies a media emission failure. Exhausted bytes and capacity are
/// resource failures at the component's origin; any other component failure
/// and a recovered member are unserializable boundaries.
pub(crate) fn media_error(error: MediaSerializationFailure) -> CssSerializationError {
    match error {
        MediaSerializationFailure::Component(component) => {
            let kind = match component.kind {
                CssComponentValueErrorKind::ByteLimit => {
                    CssSpecifiedValueSerializationErrorKind::ByteLimit
                }
                CssComponentValueErrorKind::CapacityOverflow => {
                    CssSpecifiedValueSerializationErrorKind::CapacityOverflow
                }
                _ => {
                    return CssSerializationError::component(
                        CssSpecifiedValueSerializationErrorKind::UnserializableBoundary,
                        component,
                    );
                }
            };
            resource(CssSerializationError::new(kind), component.origin())
        }
        MediaSerializationFailure::RecoveredNever { origin } => {
            CssSerializationError::recovered_media(&origin)
        }
    }
}

fn charge_node(
    context: &mut SpecifiedSerializationContext,
    origin: &CssValueOrigin,
) -> Result<(), CssSerializationError> {
    context
        .charge_nodes(1)
        .map_err(|error| resource(error, origin))
}

// Bound normalization scratch before allocating a lowercase copy. An identifier's
// escaped output cannot be shorter than its decoded UTF-8 spelling.
fn check_identifier_size(
    context: &SpecifiedSerializationContext,
    name: &str,
    origin: &CssValueOrigin,
) -> Result<(), CssSerializationError> {
    if !context.output_suppressed() && name.len() > context.remaining_bytes() {
        return Err(resource(
            CssSerializationError::new(CssSpecifiedValueSerializationErrorKind::ByteLimit),
            origin,
        ));
    }
    Ok(())
}

fn charge_query(
    context: &mut SpecifiedSerializationContext,
    query: &CssMediaQuery,
) -> Result<(), CssSerializationError> {
    enum Visit<'a> {
        Condition(&'a CssCondition<CssMediaLeaf>),
        Conditions(&'a [CssCondition<CssMediaLeaf>]),
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
            // Shared by the CSSOM and projected writers; a kept `only` prefix may
            // emit the type, so check it whenever either writer can.
            if typed_query_emits_type(
                value.condition().is_none(),
                value.modifier(),
                value.media_type(),
            ) {
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
                    CssConditionKind::Leaf(CssMediaLeaf::CustomMediaReference(value)) => {
                        pending.push(Visit::Component(value.component()));
                    }
                    CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(value)) => {
                        pending.push(Visit::Component(value.component()));
                    }
                    CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(value)) => {
                        check_identifier_size(
                            context,
                            &value.syntax.name_text,
                            value.syntax.name.origin(),
                        )?;
                        pending.push(Visit::Component(value.component()));
                    }
                    CssConditionKind::Leaf(CssMediaLeaf::Feature(_)) => {
                        let MediaConditionSyntax::Feature(syntax) = condition.media_syntax() else {
                            unreachable!("known atom lexical syntax")
                        };
                        pending.push(Visit::Component(&syntax.component));
                    }
                    CssConditionKind::Parenthesized(inner) | CssConditionKind::Not(inner) => {
                        pending.push(Visit::Condition(inner));
                    }
                    CssConditionKind::And(children) | CssConditionKind::Or(children) => {
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

/// The rejection of a `<media-query-list>` member that holds no media query.
/// Such a member has no authored token to name (trivia is never responsible),
/// so the rejection carries the empty member's position: its whitespace (a
/// member of its own, Syntax 3 §5.4.10), or for a member with no token at all
/// the comma that ends it, or the one before it. `None` when
/// `items[start..end]` holds a component.
pub(crate) fn empty_media_member_error(
    items: &[CssComponentValue],
    start: usize,
    end: usize,
) -> Option<CssMediaConstructionError> {
    let member = &items[start..end];
    if !member
        .iter()
        .all(crate::query::supports::construction::trivia)
    {
        return None;
    }
    let origin = member
        .first()
        .or_else(|| items.get(end))
        .or_else(|| start.checked_sub(1).and_then(|index| items.get(index)))
        .map_or(CssValueOrigin::Programmatic, |value| value.origin().clone());
    Some(CssMediaConstructionError::from(
        crate::query::media::construction::MediaConstructionKind::InvalidQueryGrammar { origin },
    ))
}

pub(crate) fn check_canonical_limit(
    query: &CssMediaQuery,
    max_bytes: usize,
) -> Result<(), MediaSerializationFailure> {
    let mut out = CssCanonicalBuilder::new(max_bytes);
    emit_query(&mut out, query)?;
    out.finish()?;
    Ok(())
}

/// Runs deep media work on the shared bounded parser thread. If that thread is
/// unavailable the operation fails with a component `ParserUnavailable` error
/// at `origin`, never recursing deeply on the caller's stack.
pub(crate) fn with_media_stack<T: Send, E: Send + From<CssComponentValueError>>(
    deep: bool,
    origin: impl FnOnce() -> CssValueOrigin,
    action: impl FnOnce() -> Result<T, E> + Send,
) -> Result<T, E> {
    crate::parser::bounded_execution::media(deep, origin, action)
}

/// The origin of a list's first deep query, for a refused deep operation.
pub(crate) fn deep_list_origin(list: &CssMediaQueryList) -> CssValueOrigin {
    list.queries()
        .iter()
        .find(|query| query_is_deep(query))
        .map_or(CssValueOrigin::Programmatic, |query| query.origin().clone())
}
pub(crate) fn query_is_deep(query: &CssMediaQuery) -> bool {
    match query {
        CssMediaQuery::Condition(value) => condition_is_deep(value),
        CssMediaQuery::Typed(value) => value.condition().is_some_and(condition_is_deep),
        CssMediaQuery::Never(_) => false,
    }
}
fn condition_is_deep(root: &CssCondition<CssMediaLeaf>) -> bool {
    let mut pending = vec![(root, 0usize)];
    while let Some((value, depth)) = pending.pop() {
        if depth >= 64 {
            return true;
        }
        match value.kind() {
            CssConditionKind::Leaf(CssMediaLeaf::CustomMediaReference(_)) => {}
            CssConditionKind::Parenthesized(v) | CssConditionKind::Not(v) => {
                pending.push((v, depth + 1))
            }
            CssConditionKind::And(v) | CssConditionKind::Or(v) => {
                pending.extend(v.conditions().iter().map(|v| (v, depth + 1)))
            }
            CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(v)) => {
                if component_depth(v.component()) >= 64 {
                    return true;
                }
            }
            CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(v)) => {
                if component_depth(v.component()) >= 64 {
                    return true;
                }
            }
            CssConditionKind::Leaf(CssMediaLeaf::Feature(_)) => {
                if let MediaConditionSyntax::Feature(syntax) = value.media_syntax()
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
impl CssMediaQuery {
    /// Parses exactly one media query, replacing rejected input with a never-matching query.
    ///
    /// The parser is not read: neither its document mode nor its component limits apply to this front, and only the fixed nesting ceiling bounds the input. [`Self::parse_medium`] reads the parser's document mode.
    #[expect(
        unused_variables,
        reason = "this grammar does not read the parser configuration"
    )]
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<CssMediaQuery> {
        crate::parser::fragments::parse_media_query(source.as_ref())
    }

    /// Parses the CSSOM single-query boundary through the owning media-query list parser.
    ///
    /// Exactly one list member is returned, including a diagnosed ignored member
    /// whose CSSOM serialization is `not all`. Empty or multiple-member lists return
    /// `None`. Diagnostics retain their original ordering, spans and recovery actions;
    /// this operation performs no host evaluation and does not erase unknown syntax.
    ///
    /// ```
    /// use surgeist_css::query::media::CssMediaQuery;
    /// use surgeist_css::CssParser;
    /// assert!(CssMediaQuery::parse_medium(&CssParser::default(), "screen, print").syntax().is_none());
    /// let report = CssMediaQuery::parse_medium(&CssParser::default(), "???");
    /// assert!(!report.is_clean());
    /// assert_eq!(
    ///     report.syntax().as_ref().unwrap().serialize_cssom(Default::default()).unwrap().as_css(),
    ///     "not all"
    /// );
    /// ```
    #[must_use]
    pub fn parse_medium(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<Option<CssMediaQuery>> {
        crate::parser::fragments::parse_cssom_media_query(source.as_ref(), parser.context())
    }
}
impl CssMediaQueryList {
    /// Parses a complete media query list with independent recovery for each comma member.
    ///
    /// An empty list is valid. Malformed members become `Never`, while grammatically
    /// valid unknown features retain unknown truth. This operation
    /// performs no matching or contextual evaluation.
    ///
    /// The parser supplies the document mode of the retained parse source.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<CssMediaQueryList> {
        crate::parser::fragments::parse_media_query_list_snapshot(
            &crate::source::CssSourceSnapshot::new(source.as_ref()),
            parser.context(),
        )
    }

    /// Parses actual UTF16 media input through CSS preprocessing and the ordinary
    /// media-query-list grammar, retaining the complete original input.
    ///
    /// # Errors
    ///
    /// Returns the preparation refusal or the caller's interruption, with no
    /// partial report.
    pub fn parse_utf16<E>(
        parser: &crate::CssParser,
        original: std::sync::Arc<Vec<u16>>,
        limits: crate::source::CssUtf16InputLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<crate::CssParseReport<CssMediaQueryList>, crate::source::CssUtf16InputError<E>>
    {
        crate::source::utf16_input::parse_utf16(original, limits, poll, |source| {
            crate::parser::fragments::parse_media_query_list_snapshot(source, parser.context())
        })
    }
}
