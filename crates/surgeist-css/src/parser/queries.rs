pub(crate) mod features;
use features::{
    AdmittedMediaFeature, classify_generic_media_feature, media_numeric_error,
    parse_generic_media_feature,
};

pub(crate) fn construct_when_media_feature(
    component: CssComponentValue,
    limits: CssComponentValueLimits,
) -> Result<crate::query::CssWhenMediaFeature, crate::query::CssConditionConstructionError> {
    features::construct_when_media_feature(component, limits)
}
use cssparser::{ParseError, Parser};
use surgeist_css_parser::stream::StopToken;
use surgeist_css_parser::{Token, match_ignore_ascii_case};

use super::boolean_condition::{BooleanConditionGrammar, parse_boolean_condition};
use super::descriptor_stream::{
    Value, ValueError, at_final_comma, byte_offset, collect_scope, consume_remaining,
    expect_exhausted, from_here, location, native, parse_scope,
};
use super::recovery::{
    RecoveryState, comma_member_span, first_non_trivia_position, recovery_action_for_error,
};
use super::syntax_types::{Mark, Site, Stream, SyntaxError};
use crate::component::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssValueOrigin,
};
use crate::diagnostic::error::{
    from_parse_error, invalid_syntax, is_nesting_limit_error, unexpected_at,
    unexpected_syntax_token, with_media_query_context,
};
use crate::diagnostic::{CssError, CssFeatureId};
use crate::numeric::expression::CssCalculationExpression;
use crate::numeric::expression::{CalculationRoot, NumericInputContext};
use crate::query::CssGeneralEnclosed;
use crate::query::media::construction::{
    MediaConditionSyntax, MediaFeatureShape, MediaTypedSyntax,
};
use crate::query::media::{
    CssMediaConstructionError, CssMediaLeaf, CssMediaQuery, CssMediaQueryList,
    CssMediaQueryModifier, CssMediaType, CssNeverMediaQuery, CssTypedMediaQuery,
    CssUnknownMediaType,
};
use crate::query::{CssCondition, CssConditionKind, CssConditionList};

pub(super) static IMPLEMENTED_MEDIA: &[CssFeatureId] = &[
    CssFeatureId::new("ext.media.custom-media"),
    CssFeatureId::new("baseline.media.type"),
    CssFeatureId::new("official.media.query-list-core"),
    CssFeatureId::new("ext.media.condition-syntax"),
    CssFeatureId::new("ext.media.malformed-member-never"),
    CssFeatureId::new("official.media.feature.width"),
    CssFeatureId::new("official.media.feature.height"),
    CssFeatureId::new("official.media.feature.device-width"),
    CssFeatureId::new("official.media.feature.device-height"),
    CssFeatureId::new("official.media.feature.aspect-ratio"),
    CssFeatureId::new("official.media.feature.device-aspect-ratio"),
    CssFeatureId::new("official.media.feature.resolution"),
    CssFeatureId::new("ext.media.resolution.dppx"),
    CssFeatureId::new("official.media.feature.color"),
    CssFeatureId::new("official.media.feature.color-index"),
    CssFeatureId::new("official.media.feature.monochrome"),
    CssFeatureId::new("official.media.feature.scan"),
    CssFeatureId::new("official.media.feature.grid"),
    CssFeatureId::new("ext.media.range.width"),
    CssFeatureId::new("ext.media.range.height"),
    CssFeatureId::new("ext.media.range.resolution"),
    CssFeatureId::new("ext.media.range.color"),
    CssFeatureId::new("ext.media.range.monochrome"),
    CssFeatureId::new("official.media.feature.orientation"),
    CssFeatureId::new("ext.media.hover"),
    CssFeatureId::new("ext.media.any-hover"),
    CssFeatureId::new("ext.media.pointer"),
    CssFeatureId::new("ext.media.any-pointer"),
    CssFeatureId::new("ext.media.prefers-color-scheme"),
    CssFeatureId::new("ext.media.prefers-reduced-motion"),
    CssFeatureId::new("ext.media.prefers-reduced-transparency"),
    CssFeatureId::new("ext.media.prefers-contrast"),
    CssFeatureId::new("ext.media.forced-colors"),
    CssFeatureId::new("ext.media.display-mode"),
    CssFeatureId::new("ext.media.horizontal-viewport-segments"),
    CssFeatureId::new("ext.media.vertical-viewport-segments"),
    CssFeatureId::new("ext.media.update"),
    CssFeatureId::new("ext.media.overflow-block"),
    CssFeatureId::new("ext.media.overflow-inline"),
    CssFeatureId::new("ext.media.color-gamut"),
    CssFeatureId::new("ext.media.video-color-gamut"),
    CssFeatureId::new("ext.media.dynamic-range"),
    CssFeatureId::new("ext.media.video-dynamic-range"),
    CssFeatureId::new("ext.media.environment-blending"),
    CssFeatureId::new("ext.media.inverted-colors"),
    CssFeatureId::new("ext.media.nav-controls"),
    CssFeatureId::new("ext.media.scripting"),
    CssFeatureId::new("ext.media.prefers-reduced-data"),
];

pub(super) static IMPLEMENTED_CONTAINER_EXTENSIONS: &[CssFeatureId] = &[
    CssFeatureId::new("baseline.container.condition"),
    CssFeatureId::new("baseline.container.size-feature"),
];

pub(crate) fn parse_media_query_list<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<CssMediaQueryList, ParseError<'i, CssError>> {
    parse_media_query_list_with_closures(source, input, diagnostics, recovery)
        .map(|parsed| parsed.queries)
}

/// Query syntax and tentative EOF closures, before the owning rule survives.
pub(super) struct ParsedMediaQueryList {
    pub(super) queries: CssMediaQueryList,
    pub(super) implicit_closures: Vec<usize>,
}

pub(super) fn parse_media_query_list_with_closures<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<ParsedMediaQueryList, ParseError<'i, CssError>> {
    parse_scope(input, recovery.source_snapshot(), |stream, numeric| {
        media_query_list(source, stream, numeric, diagnostics, recovery)
    })
}

pub(super) fn media_query_list<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    diagnostics: &mut Vec<crate::diagnostic::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> Value<'i, ParsedMediaQueryList> {
    if input.only_whitespace_remains() {
        return Ok(ParsedMediaQueryList {
            queries: CssMediaQueryList::new(Vec::new()),
            implicit_closures: Vec::new(),
        });
    }

    let mut queries = Vec::new();
    let mut implicit_closures = Vec::new();
    let mut comma_origins = Vec::new();
    let mut preceding_comma = None;
    loop {
        let member_start = byte_offset(numeric, input.site());
        let result = input.within_list_until(StopToken::Comma, |member| {
            let openings = check_media_member_components(source, member, numeric, recovery)?;
            let query = parse_media_query(source, member, numeric)?;
            expect_exhausted(member)?;
            Ok((query, openings))
        });
        let member_end = byte_offset(numeric, input.site());
        let comma_start = member_end;
        let at = input.site();
        let following_comma = match input.consume_token() {
            Ok(Token::Comma) => {
                let comma_end = byte_offset(numeric, input.site());
                comma_origins.push(CssValueOrigin::Parsed(
                    crate::source::CssParsedOrigin::from_range(
                        recovery.source_snapshot(),
                        comma_start..comma_end,
                    )
                    .expect("parsed comma"),
                ));
                Some((comma_start, comma_end))
            }
            Err(error) if error.is_end_of_stream() => None,
            Ok(token) => {
                return Err(SyntaxError::custom(
                    at,
                    with_media_query_context(
                        unexpected_syntax_token(location(numeric, at), token),
                        None,
                    ),
                ));
            }
            Err(error) => return Err(SyntaxError::from(error)),
        };

        match result {
            Ok((query, openings)) => {
                queries.push(query);
                implicit_closures.extend(openings);
            }
            Err(error) => {
                let error = native(numeric, &error);
                let action = recovery_action_for_error(
                    &error,
                    crate::diagnostic::CssRecoveryAction::ReplaceMediaQueryWithNever,
                );
                let error = if media_terminal_error(&error) {
                    error
                } else {
                    with_media_query_context(error, None)
                };
                let Some(span) = comma_member_span(
                    source,
                    member_start,
                    member_end,
                    following_comma,
                    preceding_comma,
                ) else {
                    return Err(SyntaxError::custom(input.site(), error));
                };
                if span.start() == span.end() {
                    return Err(SyntaxError::custom(input.site(), error));
                }
                let position = first_non_trivia_position(source, member_start, member_end);
                let error = from_parse_error(source, error);
                let Some(diagnostic) =
                    crate::diagnostic::CssRecoveryDiagnostic::new(error, span, action)
                else {
                    return Err(SyntaxError::custom(
                        input.site(),
                        with_media_query_context(
                            invalid_syntax(location(numeric, input.site())),
                            None,
                        ),
                    ));
                };
                diagnostics.push(diagnostic);
                queries.push(CssMediaQuery::Never(CssNeverMediaQuery::new(
                    crate::source::CssParsedOrigin::from_range(
                        recovery.source_snapshot(),
                        position.byte_offset().value()..member_end,
                    )
                    .expect("media member origin"),
                )));
            }
        }

        let Some(comma) = following_comma else {
            break;
        };
        if at_final_comma(input) {
            break;
        }
        preceding_comma = Some(comma);
    }
    Ok(ParsedMediaQueryList {
        queries: CssMediaQueryList::with_comma_origins(queries, comma_origins),
        implicit_closures,
    })
}

/// The media query that `input` holds to its end, with the openings its
/// components leave to close at the end of the input.
pub(super) fn parse_media_query_fragment<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    recovery: &RecoveryState,
) -> Result<(CssMediaQuery, Vec<usize>), ParseError<'i, CssError>> {
    parse_scope(input, recovery.source_snapshot(), |stream, numeric| {
        let openings = check_media_member_components(source, stream, numeric, recovery)?;
        let query = parse_media_query(source, stream, numeric)?;
        expect_exhausted(stream)?;
        Ok((query, openings))
    })
}

pub(super) fn parse_media_query<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Value<'i, CssMediaQuery> {
    parse_media_query_inner(
        source,
        input,
        &MediaInput {
            numeric,
            limits: CssComponentValueLimits::default(),
        },
    )
}
fn parse_media_query_inner<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssMediaQuery> {
    let typed = match input.attempt(|p| parse_typed_media_query(source, p, numeric)) {
        Ok(query) => return Ok(CssMediaQuery::Typed(query)),
        Err(error) if media_committed(&error) => return Err(error),
        Err(error) => error,
    };
    // Media Queries 4 §3: when neither form matches, the alternative that
    // reached further names the responsible token (`screen and` fails at its
    // end as a typed query, not at `screen` as a condition).
    parse_media_condition(source, input, numeric)
        .map(CssMediaQuery::Condition)
        .map_err(|condition| {
            if numeric.reach(&typed) > numeric.reach(&condition) {
                typed
            } else {
                condition
            }
        })
}
fn parse_typed_media_query<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssTypedMediaQuery> {
    let start = input.mark();
    let modifier = input
        .attempt(|input| parse_media_query_modifier(input, numeric))
        .ok();
    let modifier_component = if modifier.is_some() {
        let end = input.mark();
        input.restore(start);
        let component = numeric
            .collect(input)
            .map_err(|e| media_numeric_error(source, input, numeric, e))?;
        input.restore(end);
        Some(component)
    } else {
        None
    };
    let type_start = input.mark();
    let media_type = parse_media_type(source, input, numeric)?;
    let type_end = input.mark();
    input.restore(type_start);
    let component = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    input.restore(type_end);
    let conjunction = take_media_keyword(input, numeric, "and")?;
    let condition = if conjunction.is_some() {
        Some(parse_media_condition_without_or(source, input, numeric)?)
    } else {
        None
    };
    let origin = modifier_component
        .as_ref()
        .unwrap_or(&component)
        .origin()
        .clone();
    let syntax = MediaTypedSyntax {
        modifier: modifier_component.map(std::borrow::Cow::into_owned),
        media_type: component.into_owned(),
        conjunction,
    };
    Ok(match media_type {
        ParsedMediaType::Known(ty) => {
            CssTypedMediaQuery::new(modifier, ty, condition, origin, syntax)
        }
        ParsedMediaType::Unknown(ty) => {
            CssTypedMediaQuery::new_unknown(modifier, ty, condition, origin, syntax)
        }
    })
}

fn parse_media_query_modifier<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssMediaQueryModifier> {
    let at = input.site();
    let ident = from_here(input, Stream::consume_ident)?;
    match_ignore_ascii_case! { ident,
        "not" => Ok(CssMediaQueryModifier::Not),
        "only" => Ok(CssMediaQueryModifier::Only),
        _ => Err(numeric.unexpected(at)),
    }
}

enum ParsedMediaType {
    Known(CssMediaType),
    Unknown(CssUnknownMediaType),
}

fn parse_media_type<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, ParsedMediaType> {
    let at = input.site();
    let first = first_non_trivia_site(input);
    let ident = from_here(input, Stream::consume_ident)?;
    match_ignore_ascii_case! { ident,
        "all" => Ok(ParsedMediaType::Known(CssMediaType::All)),
        "aural" => Ok(ParsedMediaType::Known(CssMediaType::Aural)),
        "braille" => Ok(ParsedMediaType::Known(CssMediaType::Braille)),
        "embossed" => Ok(ParsedMediaType::Known(CssMediaType::Embossed)),
        "handheld" => Ok(ParsedMediaType::Known(CssMediaType::Handheld)),
        "projection" => Ok(ParsedMediaType::Known(CssMediaType::Projection)),
        "screen" => Ok(ParsedMediaType::Known(CssMediaType::Screen)),
        "speech" => Ok(ParsedMediaType::Known(CssMediaType::Speech)),
        "tty" => Ok(ParsedMediaType::Known(CssMediaType::Tty)),
        "tv" => Ok(ParsedMediaType::Known(CssMediaType::Tv)),
        "print" => Ok(ParsedMediaType::Known(CssMediaType::Print)),
        "layer" | "not" | "and" | "only" | "or" => Err(numeric.unexpected(at)),
        _ => Ok(ParsedMediaType::Unknown(CssUnknownMediaType::new(
            source
                .get(numeric.offset(first)..numeric.offset(input.site()))
                .unwrap_or(ident),
            numeric.origin_of(first).expect("media type token origin"),
        ))),
    }
}

fn parse_media_condition<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssCondition<CssMediaLeaf>> {
    parse_media_condition_with_or(source, input, true, numeric)
}

fn parse_media_condition_without_or<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssCondition<CssMediaLeaf>> {
    parse_media_condition_with_or(source, input, false, numeric)
}

fn take_media_keyword<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
    keyword: &str,
) -> Value<'i, Option<CssValueOrigin>> {
    let start = input.mark();
    input.discard_whitespace();
    let at = input.site();
    if input.attempt(|p| p.consume_keyword(keyword)).is_ok() {
        Ok(Some(
            numeric
                .origin_of(at)
                .expect("media keyword original origin"),
        ))
    } else {
        input.restore(start);
        Ok(None)
    }
}
fn parse_media_condition_with_or<'i>(
    source: &str,
    input: &mut Stream<'i>,
    allow_or: bool,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssCondition<CssMediaLeaf>> {
    parse_boolean_condition(
        &mut MediaConditionGrammar {
            source,
            input,
            numeric,
        },
        allow_or,
    )
}
// Media Queries 4 §3 `<media-condition>`: each operator keyword keeps its
// original origin, and a combined condition starts at its first operand.
struct MediaConditionGrammar<'s, 'i, 'n> {
    source: &'s str,
    input: &'s mut Stream<'i>,
    numeric: &'s MediaInput<'n>,
}
impl<'i> BooleanConditionGrammar for MediaConditionGrammar<'_, 'i, '_> {
    type Operator = CssValueOrigin;
    type Node = CssCondition<CssMediaLeaf>;
    type Error = ValueError<'i>;

    fn keyword(&mut self, keyword: &'static str) -> Result<Option<CssValueOrigin>, Self::Error> {
        take_media_keyword(self.input, self.numeric, keyword)
    }
    fn operand(&mut self) -> Result<CssCondition<CssMediaLeaf>, Self::Error> {
        parse_media_in_parens(self.source, self.input, self.numeric)
    }
    // The enclosing query owns what follows a media condition.
    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn negation(
        &mut self,
        operator: CssValueOrigin,
        operand: CssCondition<CssMediaLeaf>,
    ) -> CssCondition<CssMediaLeaf> {
        CssCondition::media(
            CssConditionKind::Not(Box::new(operand)),
            operator,
            MediaConditionSyntax::Not,
        )
    }
    fn single(&mut self, operand: CssCondition<CssMediaLeaf>) -> CssCondition<CssMediaLeaf> {
        operand
    }
    fn junction(
        &mut self,
        conjunction: bool,
        operators: Vec<CssValueOrigin>,
        operands: Vec<CssCondition<CssMediaLeaf>>,
    ) -> CssCondition<CssMediaLeaf> {
        let origin = operands[0].origin().clone();
        let operands = CssConditionList::new(operands);
        CssCondition::media(
            if conjunction {
                CssConditionKind::And(operands)
            } else {
                CssConditionKind::Or(operands)
            },
            origin,
            MediaConditionSyntax::Junction(operators),
        )
    }
}
fn parse_media_in_parens<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssCondition<CssMediaLeaf>> {
    input.discard_whitespace();
    let start = input.mark();
    let component = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let end = input.mark();
    let origin = component.origin().clone();
    let closing = match component.view() {
        CssComponentValueRef::Function(_) => None,
        CssComponentValueRef::Block(block)
            if block.kind() == crate::component::CssBlockKind::Parenthesis =>
        {
            Some(block.closing_origin().clone())
        }
        _ => {
            return Err(numeric.invalid(start.site()));
        }
    };
    let mut component = Some(component);
    if let Some(closing) = closing {
        input.restore(start);
        match input.consume_after_whitespace() {
            Ok(Token::OpenParen) => {}
            Ok(token) => return Err(start.site().unexpected(token)),
            Err(error) => return Err(error.into()),
        }
        let candidate = input.within_block(|p| {
            let initial = p.mark();
            match p.attempt(|p| {
                let v = parse_media_condition(source, p, numeric)?;
                expect_exhausted(p)?;
                Ok(v)
            }) {
                Ok(inner) => {
                    return Ok(Some((
                        CssConditionKind::Parenthesized(Box::new(inner)),
                        MediaConditionSyntax::Group { closing },
                    )));
                }
                Err(e) if media_committed(&e) => return Err(e),
                Err(_) => {}
            }
            p.restore(initial);
            let syntax = match parse_generic_media_feature(source, p, numeric) {
                Ok(value) => value,
                Err(e) if media_committed(&e) => return Err(e),
                Err(_) => {
                    // The original complete component was validated above. Consume this
                    // speculative parser before retaining that component as opaque syntax.
                    consume_remaining(p);
                    return Ok(None);
                }
            };
            if syntax.name_text.starts_with("--") {
                // MQ5 §3: a dashed name is only a boolean custom-media
                // reference, so as a feature name that name is responsible,
                // wherever the range form places it.
                if !matches!(syntax.shape, MediaFeatureShape::Boolean) {
                    return Err(SyntaxError::custom(
                        syntax.name_site,
                        crate::diagnostic::error::custom_media_context_error(
                            numeric.location(syntax.name_site),
                            &syntax.name_text,
                        ),
                    ));
                }
                let name = crate::query::media::CssCustomMediaName::try_from_component(syntax.name)
                    .expect("selected extension identifier");
                return Ok(Some((
                    CssConditionKind::Leaf(CssMediaLeaf::CustomMediaReference(
                        crate::query::media::CssCustomMediaReference::new(
                            name,
                            component
                                .take()
                                .expect("selected custom media enclosure")
                                .into_owned(),
                        ),
                    )),
                    MediaConditionSyntax::Enclosed,
                )));
            }
            let syntax = syntax.into_syntax(
                component
                    .take()
                    .expect("selected media feature enclosure")
                    .into_owned(),
            );
            let admitted = classify_generic_media_feature(source, p, numeric, initial, syntax)?;
            Ok(Some(match admitted {
                AdmittedMediaFeature::Feature(feature, syntax) => (
                    CssConditionKind::Leaf(CssMediaLeaf::Feature(feature)),
                    MediaConditionSyntax::Feature(syntax),
                ),
                AdmittedMediaFeature::Unknown(feature) => (
                    CssConditionKind::Leaf(CssMediaLeaf::UnknownFeature(feature)),
                    MediaConditionSyntax::Enclosed,
                ),
            }))
        })?;
        input.restore(end);
        if let Some((kind, syntax)) = candidate {
            return Ok(CssCondition::media(kind, origin, syntax));
        }
    }
    input.restore(end);
    let enclosed = CssGeneralEnclosed::try_from_component(
        component.expect("opaque media enclosure").into_owned(),
    )
    .map_err(|_| numeric.invalid(start.site()))?;
    Ok(CssCondition::media(
        CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(enclosed)),
        origin,
        MediaConditionSyntax::Enclosed,
    ))
}
/// The boundary of the first token that is not whitespace or a comment; the end
/// of the input when there is none. The stream does not move.
fn first_non_trivia_site<'i>(input: &Stream<'i>) -> Site<'i> {
    let mut probe = input.clone();
    probe.discard_whitespace();
    probe.site()
}

pub(super) struct MediaInput<'a> {
    pub(super) numeric: &'a NumericInputContext<'a>,
    pub(super) limits: CssComponentValueLimits,
}
impl<'a> MediaInput<'a> {
    pub(super) fn collect<'i>(
        &self,
        input: &mut Stream<'i>,
    ) -> Result<std::borrow::Cow<'i, CssComponentValue>, crate::numeric::CssNumericConstructionError>
    {
        self.numeric.collect(input)
    }
    /// The origin of the token the boundary `site` is at.
    pub(super) fn origin_of(&self, site: Site<'_>) -> Option<CssValueOrigin> {
        use crate::syntax_consumption::DocumentExt as _;
        site.node().map(|node| site.document().origin(node))
    }
    /// The byte of the text the boundary `site` is at.
    pub(super) fn offset(&self, site: Site<'_>) -> usize {
        byte_offset(self.numeric, site)
    }
    pub(super) fn location(&self, site: Site<'_>) -> cssparser::SourceLocation {
        location(self.numeric, site)
    }
    /// A failure of the grammar at `at`, naming no token.
    pub(super) fn unexpected<'i>(&self, at: Site<'i>) -> ValueError<'i> {
        SyntaxError::custom(at, unexpected_at(self.location(at)))
    }
    /// A syntax failure at `at`.
    pub(super) fn invalid<'i>(&self, at: Site<'i>) -> ValueError<'i> {
        SyntaxError::custom(at, invalid_syntax(self.location(at)))
    }
    /// How far into the text a failure got: where it is, as a reader counts.
    pub(super) fn reach(&self, error: &ValueError<'_>) -> (u32, u32) {
        let location = native(self.numeric, error).location;
        (location.line, location.column)
    }
    pub(super) fn next_origin(&self, input: &mut Stream<'_>) -> CssValueOrigin {
        input.discard_whitespace();
        self.origin_of(input.site())
            .expect("checked media cursor origin")
    }
    pub(super) fn admit<'input>(
        &self,
        values: impl Into<crate::numeric::expression::CalculationComponents<'input>>,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::numeric::CssNumericConstructionError> {
        self.numeric.admit_with_limits(values, root, self.limits)
    }
    pub(super) fn error_location(
        &self,
        error: &crate::numeric::CssNumericConstructionError,
        fallback: cssparser::SourceLocation,
        offset: usize,
    ) -> cssparser::SourceLocation {
        self.numeric.error_location(error, fallback, offset)
    }
    /// The components of the stream between the mark `start` and where it is now.
    pub(super) fn between<'i>(
        &self,
        input: &Stream<'i>,
        start: Mark<'i>,
    ) -> Value<'i, CssComponentValues> {
        let first = start.site();
        let end = input.site();
        let document = first.document();
        let nodes = document
            .list(first.list())
            .get(first.index()..end.index())
            .unwrap_or(&[]);
        crate::component::values::parse::promote_run(document, nodes).map_err(|e| {
            SyntaxError::custom(
                end,
                crate::diagnostic::error::invalid_component_value(self.location(end), e),
            )
        })
    }
}
fn media_committed_error(error: &ParseError<'_, CssError>) -> bool {
    media_terminal_error(error)
        || matches!(&error.kind, cssparser::ParseErrorKind::Custom(error) if matches!(error.kind(), crate::diagnostic::error::ErrorKind::InvalidMediaQuery(detail) if detail.is_committed_context()))
}
pub(super) fn media_terminal_error(error: &ParseError<'_, CssError>) -> bool {
    is_nesting_limit_error(error)
        || matches!(&error.kind,cssparser::ParseErrorKind::Custom(error) if matches!(error.kind(),crate::diagnostic::error::ErrorKind::InvalidComponentValue(_)))
}
/// [`media_committed_error`] for the failure of a stream grammar.
pub(super) fn media_committed(error: &ValueError<'_>) -> bool {
    error.custom_value().is_some_and(media_committed_error)
}
/// [`media_terminal_error`] for the failure of a stream grammar.
pub(super) fn media_terminal(error: &ValueError<'_>) -> bool {
    error.custom_value().is_some_and(media_terminal_error)
}
/// Checks the components of the comma-separated member that `input` holds, and
/// returns the openings that close at the end of the input.
pub(super) fn check_media_member_components<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    recovery: &RecoveryState,
) -> Value<'i, Vec<usize>> {
    let start = byte_offset(numeric, input.site());
    match recovery.check_comma_member_components_from(
        source,
        start,
        "official.media.query-list-core",
    ) {
        Ok(openings) => Ok(openings),
        Err(error) if is_nesting_limit_error(&error) => {
            Err(SyntaxError::custom(input.site(), error))
        }
        Err(error) => {
            let component_error = collect_scope(input, numeric)
                .err()
                .map(|(detail, at)| crate::diagnostic::error::invalid_component_value(at, detail));
            Err(SyntaxError::custom(
                input.site(),
                component_error.unwrap_or(error),
            ))
        }
    }
}
/// Checks the components of the prelude of an `@import` that `input` holds from
/// where it is.
pub(super) fn check_media_import_components<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    recovery: &RecoveryState,
) -> Value<'i, ()> {
    match recovery.check_specialized_components_from(
        source,
        byte_offset(numeric, input.site()),
        "official.media.query-list-core",
    ) {
        Ok(_) => Ok(()),
        Err(error) => {
            if is_nesting_limit_error(&error) {
                return Err(SyntaxError::custom(input.site(), error));
            }
            let component_error = collect_scope(input, numeric)
                .err()
                .map(|(detail, at)| crate::diagnostic::error::invalid_component_value(at, detail));
            Err(SyntaxError::custom(
                input.site(),
                component_error.unwrap_or(error),
            ))
        }
    }
}
/// Whether component construction needs the bounded thread, and the
/// responsible component if that thread is unavailable.
fn deep_media_components(values: &CssComponentValues) -> (bool, CssValueOrigin) {
    let deep = values.nesting_depth() >= 64;
    let origin = if deep {
        super::bounded_execution::deep_component_origin(values.items())
    } else {
        CssValueOrigin::Programmatic
    };
    (deep, origin)
}
pub(crate) fn construct_media_query(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssMediaQuery, CssMediaConstructionError> {
    let (deep, origin) = deep_media_components(&values);
    crate::query::media::construction::with_media_stack(
        deep,
        || origin,
        move || construct_media(values, limits, false),
    )
}
pub(crate) fn construct_media_condition(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssCondition<CssMediaLeaf>, CssMediaConstructionError> {
    let (deep, origin) = deep_media_components(&values);
    match crate::query::media::construction::with_media_stack(
        deep,
        || origin,
        move || construct_media(values, limits, true),
    )? {
        CssMediaQuery::Condition(condition) => Ok(condition),
        _ => unreachable!("condition construction context"),
    }
}
fn construct_media(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
    condition_only: bool,
) -> Result<CssMediaQuery, CssMediaConstructionError> {
    values.validate_with_limits(limits)?;
    if let Some(origin) = values.first_implicit_origin() {
        return Err(CssMediaConstructionError::from(
            crate::query::media::construction::MediaConstructionKind::RecoveredInput {
                origin: origin.clone(),
            },
        ));
    }
    let serialized = values.serialize_tokens(limits)?;
    let source = serialized.as_css();
    let numeric = NumericInputContext::components(&values, &serialized);
    let context = MediaInput {
        numeric: &numeric,
        limits,
    };
    let document = crate::syntax_consumption::checked_document(
        values.items(),
        crate::syntax_consumption::SyntaxInputLimits {
            max_depth: limits.max_nesting_depth(),
            max_components: limits.max_components(),
            // Public source/output byte admission remains owned by values and the
            // checked writer. Do not reinterpret it as an input spelling metric.
            max_known_spelling_bytes: usize::MAX,
        },
        0,
    )?;
    let mut input = document.root();
    let result = (|| {
        let query = if condition_only {
            parse_media_condition(source, &mut input, &context).map(CssMediaQuery::Condition)?
        } else {
            parse_media_query_inner(source, &mut input, &context)?
        };
        expect_exhausted(&mut input)?;
        Ok(query)
    })();
    let query = result.map_err(|error: ValueError<'_>| {
        let error = native(&numeric, &error);
        if let cssparser::ParseErrorKind::Custom(error) = &error.kind
            && let crate::diagnostic::error::ErrorKind::InvalidComponentValue(component) =
                error.kind()
        {
            return CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::Component(
                    component.as_ref().clone(),
                ),
            );
        }
        let error = from_parse_error(source, error);
        let origin = serialized
            .value_origin_at(error.position().byte_offset().value())
            .cloned()
            .unwrap_or(CssValueOrigin::Programmatic);
        if condition_only {
            CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::InvalidConditionGrammar {
                    origin,
                },
            )
        } else {
            CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::InvalidQueryGrammar {
                    origin,
                },
            )
        }
    })?;
    crate::query::media::construction::check_canonical_limit(&query, limits.max_css_bytes())
        .map_err(|error| match error {
            crate::query::media::construction::MediaSerializationFailure::Component(error) => {
                CssMediaConstructionError::from(
                    crate::query::media::construction::MediaConstructionKind::Component(error),
                )
            }
            crate::query::media::construction::MediaSerializationFailure::RecoveredNever {
                origin,
            } => CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::RecoveredInput { origin },
            ),
        })?;
    Ok(query)
}
