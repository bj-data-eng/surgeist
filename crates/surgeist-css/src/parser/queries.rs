mod features;
use features::{
    AdmittedMediaFeature, classify_generic_media_feature, media_numeric_error,
    parse_generic_media_feature,
};

pub(crate) fn construct_when_media_feature(
    component: CssComponentValue,
    limits: CssComponentValueLimits,
) -> Result<crate::CssWhenMediaFeature, crate::CssWhenConstructionError> {
    features::construct_when_media_feature(component, limits)
}
#[cfg(test)]
use cssparser::ParserInput;
use cssparser::{
    BasicParseErrorKind, Delimiter, ParseError, Parser, Token, match_ignore_ascii_case,
};

#[cfg(test)]
use super::recovery::StyleContextCaptures;
use super::recovery::{
    RecoveryState, comma_member_span, first_non_trivia_position, recovery_action_for_error,
};
use crate::error::{
    CssFeatureId, Error, basic, from_parse_error, invalid_syntax, is_nesting_limit_error,
    unsupported_value_at, with_media_query_context,
};
use crate::media::{MediaConditionSyntax, MediaFeatureShape, MediaTypedSyntax};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::syntax::*;
use crate::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssValueOrigin,
};

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
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<CssMediaQueryList, ParseError<'i, Error>> {
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
    diagnostics: &mut Vec<crate::CssRecoveryDiagnostic>,
    recovery: &RecoveryState,
) -> std::result::Result<ParsedMediaQueryList, ParseError<'i, Error>> {
    if input.is_exhausted() {
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
        let member_start = input.position().byte_index();
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            let openings = check_media_member_components(source, member, recovery)?;
            let query = parse_media_query(
                source,
                member,
                &NumericInputContext::parsed(recovery.source_snapshot()),
            )?;
            member.expect_exhausted()?;
            Ok((query, openings))
        });
        let member_end = input.position().byte_index();
        let comma_start = member_end;
        let following_comma = match input.next().cloned() {
            Ok(Token::Comma) => {
                comma_origins.push(CssValueOrigin::Parsed(
                    crate::CssParsedOrigin::from_range(
                        recovery.source_snapshot(),
                        comma_start..input.position().byte_index(),
                    )
                    .expect("parsed comma"),
                ));
                Some((comma_start, input.position().byte_index()))
            }
            Err(error) if matches!(error.kind, BasicParseErrorKind::EndOfInput) => None,
            Ok(token) => {
                return Err(with_media_query_context(
                    input.new_unexpected_token_error(token),
                    None,
                ));
            }
            Err(error) => return Err(with_media_query_context(error.into(), None)),
        };

        match result {
            Ok((query, openings)) => {
                queries.push(query);
                implicit_closures.extend(openings);
            }
            Err(error) => {
                let action = recovery_action_for_error(
                    &error,
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
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
                    return Err(error);
                };
                if span.start() == span.end() {
                    return Err(error);
                }
                let position = first_non_trivia_position(source, member_start, member_end);
                let error = from_parse_error(source, error);
                let Some(diagnostic) = crate::CssRecoveryDiagnostic::new(error, span, action)
                else {
                    return Err(with_media_query_context(
                        invalid_syntax(
                            input.current_source_location(),
                            "invalid media-query recovery provenance",
                        ),
                        None,
                    ));
                };
                diagnostics.push(diagnostic);
                queries.push(CssMediaQuery::Never(CssNeverMediaQuery::new(
                    crate::CssParsedOrigin::from_range(
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
        preceding_comma = Some(comma);
    }
    Ok(ParsedMediaQueryList {
        queries: CssMediaQueryList::with_comma_origins(queries, comma_origins),
        implicit_closures,
    })
}

#[cfg(test)]
pub(crate) fn parse_media_query_list_for_test(
    source: &str,
) -> std::result::Result<CssMediaQueryList, Error> {
    let working_source = crate::tokenization::prepare(source);
    let mut input = ParserInput::new(&working_source);
    let mut parser = Parser::new(&mut input);
    let mut diagnostics = Vec::new();
    let recovery = RecoveryState::at_depth(source, 0, StyleContextCaptures::default());
    let list = parse_media_query_list(source, &mut parser, &mut diagnostics, &recovery)
        .map_err(|error| from_parse_error(source, error))?;
    if let Some(diagnostic) = diagnostics.into_iter().next() {
        return Err(diagnostic.error().clone());
    }
    if !parser.is_exhausted() {
        return Err(from_parse_error(
            source,
            invalid_syntax(
                parser.current_source_location(),
                "unexpected token after media query list",
            ),
        ));
    }
    Ok(list)
}

pub(super) fn parse_media_query<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMediaQuery, ParseError<'i, Error>> {
    parse_media_query_inner(
        source,
        input,
        &MediaInput {
            numeric,
            limits: CssComponentValueLimits::default(),
        },
    )
}
fn parse_media_query_inner<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssMediaQuery, ParseError<'i, Error>> {
    match input.try_parse(|p| parse_typed_media_query(source, p, numeric)) {
        Ok(query) => return Ok(CssMediaQuery::Typed(query)),
        Err(error) if media_committed_error(&error) => return Err(error),
        Err(_) => {}
    }
    parse_media_condition(source, input, numeric).map(CssMediaQuery::Condition)
}
fn parse_typed_media_query<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssTypedMediaQuery, ParseError<'i, Error>> {
    let start = input.state();
    let modifier = input.try_parse(parse_media_query_modifier).ok();
    let modifier_component = if modifier.is_some() {
        let end = input.state();
        input.reset(&start);
        let component = numeric
            .collect(input)
            .map_err(|e| media_numeric_error(source, input, numeric, e))?;
        input.reset(&end);
        Some(component)
    } else {
        None
    };
    let type_start = input.state();
    let media_type = parse_media_type(source, input, numeric)?;
    let type_end = input.state();
    input.reset(&type_start);
    let component = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    input.reset(&type_end);
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
        modifier: modifier_component,
        media_type: component,
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

fn parse_media_query_modifier<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMediaQueryModifier, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "not" => Ok(CssMediaQueryModifier::Not),
        "only" => Ok(CssMediaQueryModifier::Only),
        _ => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported media query modifier `{ident}`"),
        )),
    }
}

enum ParsedMediaType {
    Known(CssMediaType),
    Unknown(CssUnknownMediaType),
}

fn parse_media_type<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> std::result::Result<ParsedMediaType, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let position = first_non_trivia_parser_position(input);
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
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
        "layer" | "not" | "and" | "only" | "or" => Err(unsupported_value_at(
            location,
            None,
            format!("reserved media type `{ident}`"),
        )),
        _ => Ok(ParsedMediaType::Unknown(CssUnknownMediaType::new(
            source
                .get(position.byte_offset().value()..input.position().byte_index())
                .unwrap_or(ident.as_ref()),
            numeric.origin_at(position.byte_offset().value()).expect("media type token origin"),
        ))),
    }
}

fn parse_media_condition<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> std::result::Result<CssMediaCondition, ParseError<'i, Error>> {
    parse_media_condition_with_or(source, input, true, numeric)
}

fn parse_media_condition_without_or<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> std::result::Result<CssMediaCondition, ParseError<'i, Error>> {
    parse_media_condition_with_or(source, input, false, numeric)
}

fn take_media_keyword<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
    keyword: &str,
) -> Result<Option<CssValueOrigin>, ParseError<'i, Error>> {
    let start = input.state();
    input.skip_whitespace();
    let offset = input.position().byte_index();
    if input
        .try_parse(|p| p.expect_ident_matching(keyword))
        .is_ok()
    {
        Ok(Some(
            numeric
                .origin_at(offset)
                .expect("media keyword original origin"),
        ))
    } else {
        input.reset(&start);
        Ok(None)
    }
}
fn parse_media_condition_with_or<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    allow_or: bool,
    numeric: &MediaInput<'_>,
) -> Result<CssMediaCondition, ParseError<'i, Error>> {
    if let Some(origin) = take_media_keyword(input, numeric, "not")? {
        return Ok(CssMediaCondition::new(
            CssMediaConditionKind::Not(Box::new(parse_media_in_parens(source, input, numeric)?)),
            origin,
            MediaConditionSyntax::Not,
        ));
    }
    let first = parse_media_in_parens(source, input, numeric)?;
    let origin = first.origin().clone();
    let and = take_media_keyword(input, numeric, "and")?;
    let (keyword, operator) = if let Some(operator) = and {
        ("and", Some(operator))
    } else if allow_or {
        ("or", take_media_keyword(input, numeric, "or")?)
    } else {
        ("and", None)
    };
    let Some(operator) = operator else {
        return Ok(first);
    };
    let mut operators = vec![operator];
    let mut children = vec![first, parse_media_in_parens(source, input, numeric)?];
    while let Some(operator) = take_media_keyword(input, numeric, keyword)? {
        operators.push(operator);
        children.push(parse_media_in_parens(source, input, numeric)?);
    }
    let children = CssMediaConditionList::new(children);
    let kind = if keyword == "and" {
        CssMediaConditionKind::And(children)
    } else {
        CssMediaConditionKind::Or(children)
    };
    Ok(CssMediaCondition::new(
        kind,
        origin,
        MediaConditionSyntax::Junction(operators),
    ))
}
fn parse_media_in_parens<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssMediaCondition, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let component = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let end = input.state();
    let origin = component.origin().clone();
    let closing = match component.view() {
        CssComponentValueRef::Function(_) => None,
        CssComponentValueRef::Block(block) if block.kind() == crate::CssBlockKind::Parenthesis => {
            Some(block.closing_origin().clone())
        }
        _ => {
            return Err(invalid_syntax(
                start.source_location(),
                "expected media parenthesis or function",
            ));
        }
    };
    if let Some(closing) = closing {
        input.reset(&start);
        input.expect_parenthesis_block().map_err(basic)?;
        let candidate = input.parse_nested_block(|p| {
            let initial = p.state();
            match p.try_parse(|p| {
                let v = parse_media_condition(source, p, numeric)?;
                p.expect_exhausted()?;
                Ok(v)
            }) {
                Ok(inner) => {
                    return Ok(Some((
                        CssMediaConditionKind::Parenthesized(Box::new(inner)),
                        MediaConditionSyntax::Group { closing },
                    )));
                }
                Err(e) if media_committed_error(&e) => return Err(e),
                Err(_) => {}
            }
            p.reset(&initial);
            let syntax = match parse_generic_media_feature(source, p, numeric, component.clone()) {
                Ok(value) => value,
                Err(e) if media_committed_error(&e) => return Err(e),
                Err(_) => {
                    // The original complete component was validated above. Consume this
                    // speculative parser before retaining that component as opaque syntax.
                    while p.next_including_whitespace_and_comments().is_ok() {}
                    return Ok(None);
                }
            };
            if syntax.name_text.starts_with("--") {
                if !matches!(syntax.shape, MediaFeatureShape::Boolean) {
                    return Err(crate::error::custom_media_context_error(
                        initial.source_location(),
                        &syntax.name_text,
                    ));
                }
                let name = crate::CssCustomMediaName::try_from_component(syntax.name.clone())
                    .expect("selected extension identifier");
                return Ok(Some((
                    CssMediaConditionKind::CustomMediaReference(
                        crate::CssCustomMediaReference::new(name, component.clone()),
                    ),
                    MediaConditionSyntax::Enclosed,
                )));
            }
            let admitted = classify_generic_media_feature(source, p, numeric, &initial, syntax)?;
            Ok(Some(match admitted {
                AdmittedMediaFeature::Feature(feature, syntax) => (
                    CssMediaConditionKind::Feature(feature),
                    MediaConditionSyntax::Feature(syntax),
                ),
                AdmittedMediaFeature::Unknown(feature) => (
                    CssMediaConditionKind::UnknownFeature(feature),
                    MediaConditionSyntax::Enclosed,
                ),
            }))
        })?;
        input.reset(&end);
        if let Some((kind, syntax)) = candidate {
            return Ok(CssMediaCondition::new(kind, origin, syntax));
        }
    }
    input.reset(&end);
    let enclosed = CssGeneralEnclosed::try_from_component(component)
        .map_err(|_| invalid_syntax(start.source_location(), "invalid media enclosure"))?;
    Ok(CssMediaCondition::new(
        CssMediaConditionKind::GeneralEnclosed(enclosed),
        origin,
        MediaConditionSyntax::Enclosed,
    ))
}
fn first_non_trivia_parser_position(input: &mut Parser<'_, '_>) -> crate::CssSourcePosition {
    let initial = input.state();
    let position = loop {
        let token_start = input.state();
        match input.next_including_whitespace_and_comments() {
            Ok(Token::WhiteSpace(_) | Token::Comment(_)) => {}
            Ok(_) => {
                break crate::CssSourcePosition::from_cssparser(
                    token_start.position(),
                    token_start.source_location(),
                );
            }
            Err(_) => {
                break crate::CssSourcePosition::from_cssparser(
                    input.position(),
                    input.current_source_location(),
                );
            }
        }
    };
    input.reset(&initial);
    position
}

struct MediaInput<'a> {
    numeric: &'a NumericInputContext<'a>,
    limits: CssComponentValueLimits,
}
impl MediaInput<'_> {
    fn collect(
        &self,
        input: &mut Parser<'_, '_>,
    ) -> Result<CssComponentValue, crate::CssNumericConstructionError> {
        self.numeric.collect(input)
    }
    fn origin_at(&self, offset: usize) -> Option<CssValueOrigin> {
        self.numeric.origin_at(offset)
    }
    fn next_origin(&self, input: &mut Parser<'_, '_>) -> CssValueOrigin {
        input.skip_whitespace();
        self.origin_at(input.position().byte_index())
            .expect("checked media cursor origin")
    }
    fn admit(
        &self,
        values: CssComponentValues,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::CssNumericConstructionError> {
        self.numeric.admit_with_limits(values, root, self.limits)
    }
    fn error_location(
        &self,
        error: &crate::CssNumericConstructionError,
        fallback: cssparser::SourceLocation,
        offset: usize,
    ) -> cssparser::SourceLocation {
        self.numeric.error_location(error, fallback, offset)
    }
    fn between<'i>(
        &self,
        input: &mut Parser<'i, '_>,
        start: &cssparser::ParserState,
    ) -> Result<CssComponentValues, ParseError<'i, Error>> {
        let end = input.state();
        let items = match self.numeric.ordinary() {
            NumericInputContext::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            NumericInputContext::Parsed(snapshot) => {
                input.reset(start);
                let mut items = Vec::new();
                while input.position().byte_index() < end.position().byte_index() {
                    items.push(
                        CssComponentValue::collect_from_parser(input, snapshot).map_err(|e| {
                            crate::error::invalid_component_value(
                                input.current_source_location(),
                                e,
                            )
                        })?,
                    );
                }
                input.reset(&end);
                items
            }
            NumericInputContext::Components(values, serialized) => {
                let paths = serialized
                    .component_paths_in_range(
                        start.position().byte_index()..end.position().byte_index(),
                    )
                    .ok_or_else(|| {
                        invalid_syntax(
                            start.source_location(),
                            "media value is not a complete component slice",
                        )
                    })?;
                paths
                    .into_iter()
                    .map(|path| {
                        values
                            .component_at_path(path)
                            .expect("serialized original component path")
                            .clone()
                    })
                    .collect()
            }
        };
        CssComponentValues::try_new(items)
            .map_err(|e| crate::error::invalid_component_value(input.current_source_location(), e))
    }
}
fn media_committed_error(error: &ParseError<'_, Error>) -> bool {
    media_terminal_error(error)
        || matches!(&error.kind, cssparser::ParseErrorKind::Custom(error) if matches!(error.kind(), crate::ErrorKind::InvalidMediaQuery(detail) if detail.is_committed_context()))
}
pub(super) fn media_terminal_error(error: &ParseError<'_, Error>) -> bool {
    is_nesting_limit_error(error)
        || matches!(&error.kind,cssparser::ParseErrorKind::Custom(error) if matches!(error.kind(),crate::ErrorKind::InvalidComponentValue(_)))
}
pub(super) fn check_media_member_components<'i>(
    source: &str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<Vec<usize>, ParseError<'i, Error>> {
    match recovery.check_comma_member_components(source, input, "baseline.media.query-list") {
        Ok(openings) => Ok(openings),
        Err(error) if is_nesting_limit_error(&error) => Err(error),
        Err(error) => {
            let start = input.state();
            let mut component_error = None;
            while !input.is_exhausted() {
                if let Err(detail) =
                    CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
                {
                    component_error = Some(crate::error::invalid_component_value(
                        input.current_source_location(),
                        detail,
                    ));
                    break;
                }
            }
            input.reset(&start);
            Err(component_error.unwrap_or(error))
        }
    }
}
pub(super) fn check_media_import_components<'i>(
    source: &str,
    input: &mut Parser<'i, '_>,
    recovery: &RecoveryState,
) -> Result<(), ParseError<'i, Error>> {
    match recovery.check_specialized_components(source, input, "baseline.media.query-list") {
        Ok(_) => Ok(()),
        Err(error) if is_nesting_limit_error(&error) => Err(error),
        Err(error) => {
            let start = input.state();
            let result = (|| {
                while !input.is_exhausted() {
                    CssComponentValue::collect_from_parser(input, recovery.source_snapshot())
                        .map_err(|detail| {
                            crate::error::invalid_component_value(
                                input.current_source_location(),
                                detail,
                            )
                        })?;
                }
                Err(error)
            })();
            input.reset(&start);
            result
        }
    }
}
pub(crate) fn construct_media_query(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssMediaQuery, CssMediaConstructionError> {
    crate::media::with_media_stack(values.nesting_depth() >= 64, move || {
        construct_media(values, limits, false)
    })
}
pub(crate) fn construct_media_condition(
    values: CssComponentValues,
    limits: CssComponentValueLimits,
) -> Result<CssMediaCondition, CssMediaConstructionError> {
    match crate::media::with_media_stack(values.nesting_depth() >= 64, move || {
        construct_media(values, limits, true)
    })? {
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
        return Err(CssMediaConstructionError::RecoveredInput {
            origin: origin.clone(),
        });
    }
    let serialized = values.serialize_with_limit(limits.max_css_bytes())?;
    let source = serialized.as_css();
    let numeric = NumericInputContext::components(&values, &serialized);
    let context = MediaInput {
        numeric: &numeric,
        limits,
    };
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = cssparser::ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let result = (|| {
        let query = if condition_only {
            parse_media_condition(source, &mut input, &context).map(CssMediaQuery::Condition)?
        } else {
            parse_media_query_inner(source, &mut input, &context)?
        };
        input.expect_exhausted()?;
        Ok(query)
    })();
    let query = result.map_err(|error: ParseError<'_, Error>| {
        if let cssparser::ParseErrorKind::Custom(error) = &error.kind
            && let crate::ErrorKind::InvalidComponentValue(component) = error.kind()
        {
            return CssMediaConstructionError::Component(component.as_ref().clone());
        }
        let error = from_parse_error(source, error);
        let origin = serialized
            .value_origin_at(error.position().byte_offset().value())
            .cloned()
            .unwrap_or(CssValueOrigin::Programmatic);
        if condition_only {
            CssMediaConstructionError::InvalidConditionGrammar { origin }
        } else {
            CssMediaConstructionError::InvalidQueryGrammar { origin }
        }
    })?;
    crate::media::check_canonical_limit(&query, limits.max_css_bytes()).map_err(
        |error| match error {
            CssMediaSerializationError::Component(error) => {
                CssMediaConstructionError::Component(error)
            }
            CssMediaSerializationError::RecoveredNever { origin } => {
                CssMediaConstructionError::RecoveredInput { origin }
            }
        },
    )?;
    Ok(query)
}
