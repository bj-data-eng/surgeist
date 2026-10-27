//! Checked authored custom-media definitions and symbolic references.
use crate::component::values::serialize::{CssCanonicalBuilder, CssCanonicalToken};
use crate::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssSerializedValue, CssValueOrigin, CssValueTokenRef,
};
use crate::query::media::custom_media::ident;
use crate::query::media::{CssCustomMediaBody, CssCustomMediaName};
use crate::query::media::{CssMediaConstructionError, CssMediaQuery, CssMediaQueryList};
use crate::serialization::CssSerializationError;
use crate::source::{
    CssRetainedSources, CssSourcePosition, CssSourceRetention, CssSourceRetentionError,
};

impl CssRetainedSources for CssCustomMediaRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        sources.visit(self.data.name.component())?;
        sources.visit(&self.data.origin)?;
        sources.visit(&self.data.body_origin)?;
        sources.visit(&self.data.semicolon)?;
        if let CssCustomMediaBody::Media(value) = &self.data.body {
            sources.visit(value)?;
        }
        Ok(())
    }
}
/// A checked authored statement. Duplicate definitions and cycles remain symbolic.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCustomMediaRule {
    data: Box<CustomMediaRuleData>,
}

#[derive(Clone, Debug, PartialEq)]
struct CustomMediaRuleData {
    name: CssCustomMediaName,
    body: CssCustomMediaBody,
    origin: CssValueOrigin,
    body_origin: CssValueOrigin,
    semicolon: CssValueOrigin,
}
/// A lexical custom-media emission failure, before its public classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CustomMediaSerializationFailure {
    Component(CssComponentValueError),
    Media(crate::query::media::construction::MediaSerializationFailure),
}
/// Classifies a custom-media emission failure: a component failure by its
/// kind and a media failure as media serialization classifies it.
pub(crate) fn custom_media_error(error: CustomMediaSerializationFailure) -> CssSerializationError {
    match error {
        CustomMediaSerializationFailure::Component(error) => CssSerializationError::component(
            crate::component::values::specified_component_kind(&error),
            error,
        ),
        CustomMediaSerializationFailure::Media(error) => {
            crate::query::media::construction::media_error(error)
        }
    }
}
impl From<CssComponentValueError> for CustomMediaSerializationFailure {
    fn from(e: CssComponentValueError) -> Self {
        Self::Component(e)
    }
}
impl From<crate::query::media::construction::MediaSerializationFailure>
    for CustomMediaSerializationFailure
{
    fn from(e: crate::query::media::construction::MediaSerializationFailure) -> Self {
        Self::Media(e)
    }
}
pub(crate) fn trivia(value: &CssComponentValue) -> bool {
    matches!(
        value.view(),
        CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}
pub(crate) fn boolean_keyword(name: &str) -> Option<bool> {
    if name.eq_ignore_ascii_case("true") {
        Some(true)
    } else if name.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}
pub(crate) fn boolean_body(
    values: &[CssComponentValue],
) -> Option<(CssCustomMediaBody, CssValueOrigin)> {
    let mut meaningful = values.iter().filter(|v| !trivia(v));
    let token = meaningful.next()?;
    if meaningful.next().is_some() {
        return None;
    }
    let name = ident(token)?;
    let body = if boolean_keyword(name)? {
        CssCustomMediaBody::True
    } else {
        CssCustomMediaBody::False
    };
    Some((body, token.origin().clone()))
}
impl CssCustomMediaRule {
    /// Checks a typed body without resolving references. Recovered members and a
    /// singleton unmodified true/false media type cannot construct this rule.
    ///
    /// Checks the complete canonical statement against component and byte limits.
    /// Generated at-keyword and semicolon origins are programmatic.
    pub fn try_new(
        name: CssCustomMediaName,
        body: CssCustomMediaBody,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssMediaConstructionError> {
        let deep = matches!(&body,CssCustomMediaBody::Media(list) if list.queries().iter().any(crate::query::media::construction::query_is_deep));
        let origin = match &body {
            CssCustomMediaBody::Media(list) if deep => {
                crate::query::media::construction::deep_list_origin(list)
            }
            _ => CssValueOrigin::Programmatic,
        };
        crate::query::media::construction::with_media_stack(
            deep,
            || origin,
            || {
                let rule = Self::parsed(
                    name,
                    body,
                    CssValueOrigin::Programmatic,
                    CssValueOrigin::Programmatic,
                    CssValueOrigin::Programmatic,
                );
                rule.validate(limits)?;
                Ok(rule)
            },
        )
    }
    /// Admits a prelude (name followed by body), preserving original component origins.
    ///
    /// Checks both supplied prelude components and canonical statement output.
    /// The absent at-keyword has no parsed position; supplied tokens keep theirs.
    pub fn try_from_components(
        values: CssComponentValues,
        limits: CssComponentValueLimits,
    ) -> Result<Self, CssMediaConstructionError> {
        let deep = values.nesting_depth() >= 64;
        let origin = if deep {
            crate::parser::bounded_execution::deep_component_origin(values.items())
        } else {
            CssValueOrigin::Programmatic
        };
        crate::query::media::construction::with_media_stack(
            deep,
            || origin,
            || {
                values.validate_with_limits(limits)?;
                if let Some(origin) = values.first_implicit_origin() {
                    return Err(CssMediaConstructionError::from(
                        crate::query::media::construction::MediaConstructionKind::RecoveredInput {
                            origin: origin.clone(),
                        },
                    ));
                }
                let Some(index) = values.items().iter().position(|v| !trivia(v)) else {
                    return Err(CssMediaConstructionError::from(
                        crate::query::media::construction::MediaConstructionKind::InvalidPrelude {
                            origin: CssValueOrigin::Programmatic,
                        },
                    ));
                };
                let name = CssCustomMediaName::try_from_component(values.items()[index].clone())?;
                let tail = &values.items()[index + 1..];
                let (body, body_origin) = if let Some(boolean) = boolean_body(tail) {
                    boolean
                } else {
                    let mut queries = Vec::new();
                    let mut commas = Vec::new();
                    let mut start = 0;
                    let empty = tail.iter().all(trivia);
                    if !empty {
                        for end in 0..=tail.len() {
                            // Syntax 3 (2026 CRD) §5.4.10: a final comma followed
                            // by no token, only comments, appends no member.
                            if end == tail.len()
                                && start > 0
                                && tail[start..]
                                    .iter()
                                    .all(|v| matches!(v.view(), CssComponentValueRef::Comment(_)))
                            {
                                break;
                            }
                            if end == tail.len()
                                || matches!(
                                    tail[end].view(),
                                    CssComponentValueRef::Token(CssValueTokenRef::Comma)
                                )
                            {
                                if let Some(error) =
                                    crate::query::media::construction::empty_media_member_error(
                                        tail, start, end,
                                    )
                                {
                                    return Err(error);
                                }
                                queries.push(CssMediaQuery::try_from_components(
                                    CssComponentValues::try_new(
                                        tail[start..end].to_vec(),
                                        Default::default(),
                                    )?,
                                    limits,
                                )?);
                                if end < tail.len() {
                                    commas.push(tail[end].origin().clone());
                                }
                                start = end + 1;
                            }
                        }
                    }
                    (
                        CssCustomMediaBody::Media(CssMediaQueryList::with_comma_origins(
                            queries, commas,
                        )),
                        tail.iter()
                            .find(|v| !trivia(v))
                            .map_or(CssValueOrigin::Programmatic, |v| v.origin().clone()),
                    )
                };
                let rule = Self::parsed(
                    name,
                    body,
                    CssValueOrigin::Programmatic,
                    body_origin,
                    CssValueOrigin::Programmatic,
                );
                rule.validate(limits)?;
                Ok(rule)
            },
        )
    }
    pub(crate) fn parsed(
        name: CssCustomMediaName,
        body: CssCustomMediaBody,
        origin: CssValueOrigin,
        body_origin: CssValueOrigin,
        semicolon: CssValueOrigin,
    ) -> Self {
        Self {
            data: Box::new(CustomMediaRuleData {
                name,
                body,
                origin,
                body_origin,
                semicolon,
            }),
        }
    }
    pub const fn name(&self) -> &CssCustomMediaName {
        &self.data.name
    }
    pub const fn body(&self) -> &CssCustomMediaBody {
        &self.data.body
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.data.origin
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(&self.data.origin)
    }
    /// The deep media query responsible for a refused deep serialization.
    fn media_origin(&self) -> CssValueOrigin {
        match &self.data.body {
            CssCustomMediaBody::Media(list) => {
                crate::query::media::construction::deep_list_origin(list)
            }
            _ => self.data.origin.clone(),
        }
    }
    /// Emits canonical authored syntax with original token origins. Any recovered
    /// media member fails the entire operation before an output is returned.
    ///
    /// `limits.max_css_bytes()` caps the output; the other component limits do
    /// not apply.
    ///
    /// # Errors
    ///
    /// Returns [`CssSerializationError`] for a recovered media member or an
    /// exhausted byte limit.
    pub fn serialize_tokens(
        &self,
        limits: CssComponentValueLimits,
    ) -> Result<CssSerializedValue, CssSerializationError> {
        let deep = matches!(&self.data.body,CssCustomMediaBody::Media(list) if list.queries().iter().any(crate::query::media::construction::query_is_deep));
        let origin = || self.media_origin();
        crate::query::media::construction::with_media_stack(deep, origin, || {
            self.emit(limits.max_css_bytes())
        })
        .map_err(custom_media_error)
    }
    /// Generic graph projection leaves the logical rule charge to its caller.
    pub(crate) fn append_specified(
        &self,
        context: &mut crate::serialization::specified_serialization::SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<
        (),
        crate::serialization::specified_rule_serialization::SpecifiedRuleSerializationSource,
    > {
        crate::component::values::charge_specified_components(
            context,
            std::slice::from_ref(self.data.name.component()),
        )?;
        match &self.data.body {
            CssCustomMediaBody::True | CssCustomMediaBody::False => {
                context.charge_nodes(1)?;
            }
            CssCustomMediaBody::Media(list) => list.charge_cssom(context)?,
        }
        if context.output_suppressed() {
            return Ok(());
        }
        let deep = matches!(&self.data.body,CssCustomMediaBody::Media(list) if list.queries().iter().any(crate::query::media::construction::query_is_deep));
        let origin = || self.media_origin();
        let value = crate::query::media::construction::with_media_stack(deep, origin, || {
            let mut out = CssCanonicalBuilder::new(context.remaining_bytes());
            self.emit_into(
                &mut out,
                crate::query::media::construction::emit_query_projected,
            )?;
            Ok::<_, CustomMediaSerializationFailure>(out.finish()?)
        })?;
        context.append(output, value.as_css())?;
        Ok(())
    }
    fn emit(
        &self,
        max_bytes: usize,
    ) -> Result<CssSerializedValue, CustomMediaSerializationFailure> {
        let mut out = CssCanonicalBuilder::new(max_bytes);
        self.emit_into(&mut out, crate::query::media::construction::emit_query)?;
        Ok(out.finish()?)
    }
    fn emit_into(
        &self,
        out: &mut CssCanonicalBuilder,
        emit_query: fn(
            &mut CssCanonicalBuilder,
            &CssMediaQuery,
        )
            -> Result<(), crate::query::media::construction::MediaSerializationFailure>,
    ) -> Result<(), CustomMediaSerializationFailure> {
        out.push_grammar(
            CssCanonicalToken::AtKeyword("custom-media"),
            &self.data.origin,
        )?;
        out.push_grammar(CssCanonicalToken::Whitespace, self.data.name.origin())?;
        out.push_component(self.data.name.component())?;
        match &self.data.body {
            CssCustomMediaBody::True | CssCustomMediaBody::False => {
                out.push_grammar(CssCanonicalToken::Whitespace, &self.data.body_origin)?;
                out.push_grammar(
                    CssCanonicalToken::Ident(
                        if matches!(self.data.body, CssCustomMediaBody::True) {
                            "true"
                        } else {
                            "false"
                        },
                    ),
                    &self.data.body_origin,
                )?;
            }
            CssCustomMediaBody::Media(list) => {
                for (index, query) in list.queries().iter().enumerate() {
                    let origin = if index == 0 {
                        query.origin()
                    } else {
                        list.comma_origins
                            .get(index - 1)
                            .unwrap_or(&CssValueOrigin::Programmatic)
                    };
                    if index > 0 {
                        out.push_grammar(CssCanonicalToken::Comma, origin)?;
                    }
                    out.push_grammar(CssCanonicalToken::Whitespace, origin)?;
                    emit_query(out, query)?;
                }
            }
        }
        out.push_grammar(CssCanonicalToken::Semicolon, &self.data.semicolon)?;
        Ok(())
    }
    fn validate(&self, limits: CssComponentValueLimits) -> Result<(), CssMediaConstructionError> {
        if let CssCustomMediaBody::Media(list) = &self.data.body {
            if let [CssMediaQuery::Typed(query)] = list.queries()
                && query.modifier().is_none()
                && query.condition().is_none()
                && ident(&query.syntax.media_type).is_some_and(|name| {
                    name.eq_ignore_ascii_case("true") || name.eq_ignore_ascii_case("false")
                })
            {
                return Err(CssMediaConstructionError::from(
                    crate::query::media::construction::MediaConstructionKind::AmbiguousMediaBody {
                        origin: query.syntax.media_type.origin().clone(),
                    },
                ));
            }
            if let Some(CssMediaQuery::Never(value)) = list
                .queries()
                .iter()
                .find(|q| matches!(q, CssMediaQuery::Never(_)))
            {
                return Err(CssMediaConstructionError::from(
                    crate::query::media::construction::MediaConstructionKind::RecoveredInput {
                        origin: value.origin().clone(),
                    },
                ));
            }
        }

        let output = self.emit(limits.max_css_bytes()).map_err(|e| match e {
            CustomMediaSerializationFailure::Component(e) => CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::Component(e),
            ),
            CustomMediaSerializationFailure::Media(
                crate::query::media::construction::MediaSerializationFailure::Component(e),
            ) => CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::Component(e),
            ),
            CustomMediaSerializationFailure::Media(
                crate::query::media::construction::MediaSerializationFailure::RecoveredNever {
                    origin,
                },
            ) => CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::RecoveredInput { origin },
            ),
        })?;
        for segment in output.segments() {
            if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) =
                output.value_origin_at(segment.byte_range().start)
            {
                return Err(CssMediaConstructionError::from(
                    crate::query::media::construction::MediaConstructionKind::RecoveredInput {
                        origin: origin.clone(),
                    },
                ));
            }
        }
        // Validate resource consumption only. The original typed graph is never replaced.
        crate::component::values::parse_component_values_with_limits(output.as_css(), limits)
            .map_err(|error| {
                let offset = crate::query::media::construction::parsed_position(error.origin())
                    .map_or(0, |p| p.byte_offset().value());
                CssMediaConstructionError::from(
                    crate::query::media::construction::MediaConstructionKind::Component(
                        CssComponentValueError::new(
                            error.kind,
                            output
                                .value_origin_at(offset)
                                .cloned()
                                .unwrap_or_else(|| self.data.origin.clone()),
                        ),
                    ),
                )
            })?;
        Ok(())
    }
}
