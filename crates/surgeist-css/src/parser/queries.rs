#[cfg(test)]
use cssparser::ParserInput;
use cssparser::{
    BasicParseErrorKind, Delimiter, ParseError, Parser, ToCss, Token, match_ignore_ascii_case,
};

#[cfg(test)]
use super::recovery::StyleContextCaptures;
use super::recovery::{
    RecoveryState, comma_member_span, first_non_trivia_position, recovery_action_for_error,
};
use super::variables::collect_authored_declaration_value;
use crate::error::{
    CssFeatureId, Error, basic, from_parse_error, invalid_syntax, is_nesting_limit_error,
    unsupported_value_at, with_media_query_context,
};
use crate::media::{MediaConditionSyntax, MediaFeatureShape, MediaFeatureSyntax, MediaTypedSyntax};
use crate::media_features::{MediaRangeState, MediaValueFamily};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::syntax::*;
use crate::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssValueOrigin, CssValueTokenRef,
};

pub(super) static IMPLEMENTED_MEDIA: &[CssFeatureId] = &[
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
    let mut input = ParserInput::new(source);
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

pub(crate) fn parse_container_condition<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContainerCondition, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("not"))
        .is_ok()
    {
        return Ok(CssContainerCondition::Not(Box::new(
            parse_container_condition_atom(input)?,
        )));
    }

    let first = parse_container_condition_atom(input)?;

    if input
        .try_parse(|input| input.expect_ident_matching("and"))
        .is_ok()
    {
        let mut conditions = vec![first, parse_container_condition_atom(input)?];
        while input
            .try_parse(|input| input.expect_ident_matching("and"))
            .is_ok()
        {
            conditions.push(parse_container_condition_atom(input)?);
        }
        return Ok(CssContainerCondition::And(CssContainerConditionList::new(
            conditions,
        )));
    }

    if input
        .try_parse(|input| input.expect_ident_matching("or"))
        .is_ok()
    {
        let mut conditions = vec![first, parse_container_condition_atom(input)?];
        while input
            .try_parse(|input| input.expect_ident_matching("or"))
            .is_ok()
        {
            conditions.push(parse_container_condition_atom(input)?);
        }
        return Ok(CssContainerCondition::Or(CssContainerConditionList::new(
            conditions,
        )));
    }

    Ok(first)
}

#[cfg(test)]
pub(crate) fn parse_container_condition_for_test(
    source: &str,
) -> std::result::Result<CssContainerCondition, Error> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let condition =
        parse_container_condition(&mut parser).map_err(|error| from_parse_error(source, error))?;
    if !parser.is_exhausted() {
        return Err(from_parse_error(
            source,
            invalid_syntax(
                parser.current_source_location(),
                "unexpected token after container condition",
            ),
        ));
    }
    Ok(condition)
}

fn parse_container_condition_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContainerCondition, ParseError<'i, Error>> {
    if let Ok(style) = input.try_parse(parse_container_style_query) {
        return Ok(CssContainerCondition::Style(style));
    }

    input.expect_parenthesis_block().map_err(basic)?;
    input.parse_nested_block(|input| {
        if let Ok(condition) =
            input.try_parse(|input| input.parse_entirely(parse_container_condition))
        {
            return Ok(condition);
        }

        let feature = parse_container_feature_query(input)?;
        if !input.is_exhausted() {
            return Err(invalid_syntax(
                input.current_source_location(),
                "unexpected token in container feature query",
            ));
        }
        Ok(CssContainerCondition::Feature(feature))
    })
}

fn parse_container_feature_query<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContainerFeatureQuery, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let Some(feature_name) = ContainerFeatureName::parse(&ident) else {
        return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported container feature `{ident}`"),
        ));
    };

    match feature_name {
        ContainerFeatureName::Width(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssContainerFeatureQuery::Width(CssRangeFeature::new(
                comparison, value,
            )))
        }
        ContainerFeatureName::Height(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssContainerFeatureQuery::Height(CssRangeFeature::new(
                comparison, value,
            )))
        }
        ContainerFeatureName::InlineSize(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssContainerFeatureQuery::InlineSize(CssRangeFeature::new(
                comparison, value,
            )))
        }
        ContainerFeatureName::BlockSize(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssContainerFeatureQuery::BlockSize(CssRangeFeature::new(
                comparison, value,
            )))
        }
        ContainerFeatureName::AspectRatio(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_ratio(input)?;
            Ok(CssContainerFeatureQuery::AspectRatio(CssRangeFeature::new(
                comparison, value,
            )))
        }
        ContainerFeatureName::Orientation => {
            input.expect_colon().map_err(basic)?;
            parse_orientation(input).map(CssContainerFeatureQuery::Orientation)
        }
    }
}

fn parse_container_style_query<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContainerStyleQuery, ParseError<'i, Error>> {
    input.expect_function_matching("style").map_err(basic)?;
    input.parse_nested_block(|input| {
        let location = input.current_source_location();
        let name = input.expect_ident_cloned().map_err(basic)?;
        let Some(name) = CssCustomPropertyName::try_new(name.to_string()) else {
            return Err(invalid_syntax(
                location,
                "container style queries only support custom properties",
            ));
        };

        if input.is_exhausted() {
            return Ok(CssContainerStyleQuery::CustomPropertyPresence(name));
        }

        input.expect_colon().map_err(basic)?;
        let (value, _) = collect_authored_declaration_value(input)?;
        if value.as_css().trim().is_empty() {
            return Err(invalid_syntax(
                input.current_source_location(),
                "container style query custom property value must not be empty",
            ));
        }

        Ok(CssContainerStyleQuery::CustomPropertyValue { name, value })
    })
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
        Err(error) if media_terminal_error(&error) => return Err(error),
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
                Err(e) if media_terminal_error(&e) => return Err(e),
                Err(_) => {}
            }
            p.reset(&initial);
            let mut syntax =
                match parse_generic_media_feature(source, p, numeric, component.clone()) {
                    Ok(value) => value,
                    Err(e) if media_terminal_error(&e) => return Err(e),
                    Err(_) => {
                        // The original complete component was validated above. Consume this
                        // speculative parser before retaining that component as opaque syntax.
                        while p.next_including_whitespace_and_comments().is_ok() {}
                        return Ok(None);
                    }
                };
            let known = known_generic_name(&syntax);
            if let Some(name) = known {
                syntax.canonical_name = Some(syntax.name_text.to_ascii_lowercase());
                let discrete_range = name.id.family() == MediaValueFamily::Discrete
                    && match &syntax.shape {
                        MediaFeatureShape::Boolean => false,
                        MediaFeatureShape::Range(r) => {
                            !matches!(r.view(), CssMediaRangeRef::Plain { .. })
                        }
                    };
                if discrete_range {
                    return Ok(Some((
                        CssMediaConditionKind::UnknownFeature(CssUnknownMediaFeature::new(
                            syntax,
                            CssUnknownMediaFeatureReason::InvalidOperation,
                        )),
                        MediaConditionSyntax::Enclosed,
                    )));
                }
                p.reset(&initial);
                match parse_media_feature_query(source, p, numeric) {
                    Ok(feature) if p.is_exhausted() => {
                        return Ok(Some((
                            CssMediaConditionKind::Feature(feature),
                            MediaConditionSyntax::Feature(Box::new(syntax)),
                        )));
                    }
                    Err(e) if media_terminal_error(&e) => return Err(e),
                    _ => {}
                }
                Ok(Some((
                    CssMediaConditionKind::UnknownFeature(CssUnknownMediaFeature::new(
                        syntax,
                        CssUnknownMediaFeatureReason::InvalidValue,
                    )),
                    MediaConditionSyntax::Enclosed,
                )))
            } else {
                Ok(Some((
                    CssMediaConditionKind::UnknownFeature(CssUnknownMediaFeature::new(
                        syntax,
                        CssUnknownMediaFeatureReason::UnknownName,
                    )),
                    MediaConditionSyntax::Enclosed,
                )))
            }
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
fn known_generic_name(syntax: &MediaFeatureSyntax) -> Option<MediaFeatureName> {
    if let Some(id) = CssMediaFeatureKind::from_name(&syntax.name_text) {
        return Some(MediaFeatureName { id, prefix: None });
    }
    if matches!(&syntax.shape,MediaFeatureShape::Range(range) if matches!(range.view(),CssMediaRangeRef::Plain{..}))
    {
        MediaFeatureName::parse(&syntax.name_text)
    } else {
        None
    }
}
fn parse_generic_media_feature<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
    component: CssComponentValue,
) -> Result<MediaFeatureSyntax, ParseError<'i, Error>> {
    let initial = input.state();
    let first = input.try_parse(|p| generic_feature_first(source, p, numeric, component.clone()));
    match &first {
        Ok(v) if known_generic_name(v).is_some() => return first,
        Err(e) if media_terminal_error(e) => return first,
        _ => {}
    }
    input.reset(&initial);
    let second = input.try_parse(|p| generic_value_first(source, p, numeric, component));
    if let Err(e) = &second
        && media_terminal_error(e)
    {
        return second;
    }
    match (first, second) {
        (_, Ok(second)) if known_generic_name(&second).is_some() => Ok(second),
        (Ok(first), _) => {
            while input.next_including_whitespace_and_comments().is_ok() {}
            Ok(first)
        }
        (_, other) => other,
    }
}
fn generic_feature_first<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
    component: CssComponentValue,
) -> Result<MediaFeatureSyntax, ParseError<'i, Error>> {
    input.skip_whitespace();
    let name_start = input.state();
    let name_text = input.expect_ident_cloned().map_err(basic)?.to_string();
    input.reset(&name_start);
    let name = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let mut separators = if input.is_exhausted() {
        Vec::new()
    } else {
        vec![vec![numeric.next_origin(input)]]
    };
    let shape = if input.is_exhausted() {
        MediaFeatureShape::Boolean
    } else if input.try_parse(Parser::expect_colon).is_ok() {
        MediaFeatureShape::Range(CssMediaRange::new(MediaRangeState::Plain {
            value: generic_media_value(source, input, numeric)?,
        }))
    } else {
        let (comparison, origins) = generic_comparison(input, numeric)?;
        separators[0] = origins;
        MediaFeatureShape::Range(CssMediaRange::new(MediaRangeState::FeatureFirst {
            comparison,
            value: generic_media_value(source, input, numeric)?,
        }))
    };
    input.expect_exhausted()?;
    Ok(MediaFeatureSyntax {
        component,
        name,
        name_text,
        canonical_name: None,
        shape,
        separators,
    })
}
fn generic_value_first<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
    component: CssComponentValue,
) -> Result<MediaFeatureSyntax, ParseError<'i, Error>> {
    let left = generic_media_value(source, input, numeric)?;
    let (comparison, origins) = generic_comparison(input, numeric)?;
    let mut separators = vec![origins];
    input.skip_whitespace();
    let name_start = input.state();
    let name_text = input.expect_ident_cloned().map_err(basic)?.to_string();
    input.reset(&name_start);
    let name = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let state = if input.is_exhausted() {
        MediaRangeState::ValueFirst {
            value: left,
            comparison,
        }
    } else {
        let (second, origins) = generic_comparison(input, numeric)?;
        separators.push(origins);
        let right = generic_media_value(source, input, numeric)?;
        use CssQueryComparison::{GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual};
        match (comparison, second) {
            (LessThan | LessThanOrEqual, LessThan | LessThanOrEqual) => {
                MediaRangeState::Ascending {
                    left,
                    left_inclusive: comparison == LessThanOrEqual,
                    right,
                    right_inclusive: second == LessThanOrEqual,
                }
            }
            (GreaterThan | GreaterThanOrEqual, GreaterThan | GreaterThanOrEqual) => {
                MediaRangeState::Descending {
                    left,
                    left_inclusive: comparison == GreaterThanOrEqual,
                    right,
                    right_inclusive: second == GreaterThanOrEqual,
                }
            }
            _ => {
                return Err(invalid_syntax(
                    input.current_source_location(),
                    "invalid chained media comparison",
                ));
            }
        }
    };
    input.expect_exhausted()?;
    Ok(MediaFeatureSyntax {
        component,
        name,
        name_text,
        canonical_name: None,
        shape: MediaFeatureShape::Range(CssMediaRange::new(state)),
        separators,
    })
}
fn generic_media_value<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssComponentValues, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let component = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let can_ratio = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(n)) => !negative_literal(n),
        CssComponentValueRef::Token(
            CssValueTokenRef::Dimension { .. } | CssValueTokenRef::Ident(_),
        ) => false,
        CssComponentValueRef::Function(_) => {
            let values = CssComponentValues::try_new(vec![component.clone()]).map_err(|e| {
                crate::error::invalid_component_value(input.current_source_location(), e)
            })?;
            let expression = numeric
                .admit(values, CalculationRoot::NamedDimensionOrNumber)
                .map_err(|e| media_numeric_error(source, input, numeric, e))?;
            expression.result_type() == CssCalculationType::Number
        }
        _ => {
            return Err(invalid_syntax(
                start.source_location(),
                "invalid generic media value",
            ));
        }
    };
    if can_ratio && input.try_parse(|p| p.expect_delim('/')).is_ok() {
        let denominator = parse_media_numeric(source, input, numeric, CalculationRoot::Number)
            .map(CssNumberCalculation::from_expression)?;
        if media_literal_number(denominator.components()).is_some_and(negative_literal) {
            return Err(invalid_syntax(
                start.source_location(),
                "negative ratio denominator",
            ));
        }
    }
    numeric.between(input, &start)
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

fn parse_media_feature_query<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssMediaFeatureQuery, ParseError<'i, Error>> {
    let initial = input.state();
    let first_name = input.try_parse(Parser::expect_ident_cloned).ok();
    let name = if let Some(name) = first_name.as_deref().and_then(MediaFeatureName::parse) {
        if input.is_exhausted() {
            return name
                .boolean_kind()
                .map(CssMediaFeatureQuery::Boolean)
                .ok_or_else(|| {
                    invalid_syntax(
                        input.current_source_location(),
                        "prefixed media features require a value",
                    )
                });
        }
        name
    } else {
        // The first operand precedes the feature name. Consume its component shape
        // only to discover the named domain; the domain parser reads it after reset.
        input.reset(&initial);
        let _ = numeric
            .collect(input)
            .map_err(|error| media_numeric_error(source, input, numeric, error))?;
        if input.try_parse(|p| p.expect_delim('/')).is_ok() {
            let _ = numeric
                .collect(input)
                .map_err(|error| media_numeric_error(source, input, numeric, error))?;
        }
        parse_media_comparison(input)?;
        let ident = input.expect_ident_cloned().map_err(basic)?;
        MediaFeatureName::parse(&ident).ok_or_else(|| {
            unsupported_value_at(
                input.current_source_location(),
                None,
                "unknown media range feature",
            )
        })?
    };
    input.reset(&initial);
    match name.id.family() {
        MediaValueFamily::Length => {
            let range = parse_media_range(input, name, |p| {
                parse_media_numeric(source, p, numeric, CalculationRoot::Length)
                    .map(CssLengthCalculation::from_expression)
                    .map(CssMediaLength::new)
            })?;
            Ok(match name.id {
                CssMediaFeatureKind::Width => CssMediaFeatureQuery::Width(range),
                CssMediaFeatureKind::Height => CssMediaFeatureQuery::Height(range),
                CssMediaFeatureKind::DeviceWidth => CssMediaFeatureQuery::DeviceWidth(range),
                CssMediaFeatureKind::DeviceHeight => CssMediaFeatureQuery::DeviceHeight(range),
                _ => unreachable!("length feature catalog"),
            })
        }
        MediaValueFamily::Integer => {
            let range = parse_media_range(input, name, |p| {
                parse_media_numeric(source, p, numeric, CalculationRoot::Integer)
                    .map(CssIntegerCalculation::from_expression)
                    .map(CssMediaInteger::new)
            })?;
            Ok(match name.id {
                CssMediaFeatureKind::Color => CssMediaFeatureQuery::Color(range),
                CssMediaFeatureKind::ColorIndex => CssMediaFeatureQuery::ColorIndex(range),
                CssMediaFeatureKind::Monochrome => CssMediaFeatureQuery::Monochrome(range),
                CssMediaFeatureKind::HorizontalViewportSegments => {
                    CssMediaFeatureQuery::HorizontalViewportSegments(range)
                }
                CssMediaFeatureKind::VerticalViewportSegments => {
                    CssMediaFeatureQuery::VerticalViewportSegments(range)
                }
                _ => unreachable!("integer feature catalog"),
            })
        }
        MediaValueFamily::Ratio => {
            let range = parse_media_range(input, name, |p| parse_media_ratio(source, p, numeric))?;
            Ok(match name.id {
                CssMediaFeatureKind::AspectRatio => CssMediaFeatureQuery::AspectRatio(range),
                CssMediaFeatureKind::DeviceAspectRatio => {
                    CssMediaFeatureQuery::DeviceAspectRatio(range)
                }
                _ => unreachable!("ratio feature catalog"),
            })
        }
        MediaValueFamily::Resolution => parse_media_range(input, name, |p| {
            let initial = p.state();
            if p.try_parse(|p| p.expect_ident_matching("infinite")).is_ok() {
                p.reset(&initial);
                numeric
                    .collect(p)
                    .map(CssMediaResolution::infinite)
                    .map_err(|error| media_numeric_error(source, p, numeric, error))
            } else {
                parse_media_numeric(source, p, numeric, CalculationRoot::Resolution)
                    .map(CssResolutionCalculation::from_expression)
                    .map(CssMediaResolution::numeric)
            }
        })
        .map(CssMediaFeatureQuery::Resolution),
        MediaValueFamily::Discrete => {
            input.expect_ident().map_err(basic)?;
            input.expect_colon().map_err(basic)?;
            parse_media_discrete(source, input, numeric, name.id)
        }
    }
}

fn parse_media_comparison<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssQueryComparison, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let symbol = match token {
        Token::Delim(v @ ('<' | '>' | '=')) => v,
        _ => return Err(invalid_syntax(location, "expected media comparison")),
    };
    if symbol == '=' {
        return Ok(CssQueryComparison::Equal);
    }
    let after = input.state();
    let inclusive = loop {
        match input.next_including_whitespace_and_comments() {
            Ok(Token::Comment(_)) => {}
            Ok(Token::Delim('=')) => break true,
            _ => {
                input.reset(&after);
                break false;
            }
        }
    };
    Ok(match (symbol, inclusive) {
        ('<', false) => CssQueryComparison::LessThan,
        ('<', true) => CssQueryComparison::LessThanOrEqual,
        ('>', false) => CssQueryComparison::GreaterThan,
        ('>', true) => CssQueryComparison::GreaterThanOrEqual,
        _ => unreachable!(),
    })
}

fn parse_media_range<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    name: MediaFeatureName,
    mut value: impl FnMut(&mut Parser<'i, 't>) -> Result<T, ParseError<'i, Error>>,
) -> Result<CssMediaRange<T>, ParseError<'i, Error>> {
    let initial = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned)
        && MediaFeatureName::parse(&ident).is_some_and(|actual| actual == name)
    {
        if input.try_parse(Parser::expect_colon).is_ok() {
            let value = value(input)?;
            return Ok(CssMediaRange::new(match name.prefix {
                None => MediaRangeState::Plain { value },
                Some(RangePrefix::Min) => MediaRangeState::Min { value },
                Some(RangePrefix::Max) => MediaRangeState::Max { value },
            }));
        }
        if name.prefix.is_some() {
            return Err(invalid_syntax(
                input.current_source_location(),
                "prefixed range needs colon",
            ));
        }
        let comparison = parse_media_comparison(input)?;
        return value(input)
            .map(|value| CssMediaRange::new(MediaRangeState::FeatureFirst { comparison, value }));
    }
    input.reset(&initial);
    if name.prefix.is_some() {
        return Err(invalid_syntax(
            input.current_source_location(),
            "prefixed range needs colon",
        ));
    }
    let left = value(input)?;
    let first = parse_media_comparison(input)?;
    input.expect_ident_matching(name.id.name()).map_err(basic)?;
    if input.is_exhausted() {
        return Ok(CssMediaRange::new(MediaRangeState::ValueFirst {
            value: left,
            comparison: first,
        }));
    }
    let second = parse_media_comparison(input)?;
    let right = value(input)?;
    use CssQueryComparison::{GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual};
    let state = match (first, second) {
        (LessThan | LessThanOrEqual, LessThan | LessThanOrEqual) => MediaRangeState::Ascending {
            left,
            left_inclusive: first == LessThanOrEqual,
            right,
            right_inclusive: second == LessThanOrEqual,
        },
        (GreaterThan | GreaterThanOrEqual, GreaterThan | GreaterThanOrEqual) => {
            MediaRangeState::Descending {
                left,
                left_inclusive: first == GreaterThanOrEqual,
                right,
                right_inclusive: second == GreaterThanOrEqual,
            }
        }
        _ => {
            return Err(invalid_syntax(
                input.current_source_location(),
                "media range chain requires matching inequality directions",
            ));
        }
    };
    Ok(CssMediaRange::new(state))
}

fn media_numeric_error<'i>(
    source: &str,
    input: &Parser<'i, '_>,
    numeric: &MediaInput<'_>,
    error: crate::CssNumericConstructionError,
) -> ParseError<'i, Error> {
    if let Some(component) = error.component_error() {
        // No component at a grammar boundary is an absent operand, not a bad
        // authored token. Other component failures remain terminal and typed.
        let absent = component.kind() == crate::CssComponentValueErrorKind::InvalidToken
            && matches!(component.origin(), CssValueOrigin::Parsed(origin) if origin.span().start()==origin.span().end());
        if !absent {
            return crate::error::invalid_component_value(
                input.current_source_location(),
                component.clone(),
            );
        }
    }
    let _ = source;
    unsupported_value_at(
        numeric.error_location(
            &error,
            input.current_source_location(),
            input.position().byte_index(),
        ),
        None,
        "invalid typed media numeric value",
    )
}
fn parse_media_numeric<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
    root: CalculationRoot,
) -> Result<CssCalculationExpression, ParseError<'i, Error>> {
    let component = numeric
        .collect(input)
        .map_err(|error| media_numeric_error(source, input, numeric, error))?;
    let values = crate::CssComponentValues::try_new(vec![component])
        .map_err(|_| invalid_syntax(input.current_source_location(), "invalid media component"))?;
    numeric
        .admit(values, root)
        .map_err(|error| media_numeric_error(source, input, numeric, error))
}
fn media_literal_number(
    value: &crate::CssComponentValues,
) -> Option<crate::CssNumericTokenRef<'_>> {
    value.items().iter().find_map(|c| match c.view() {
        crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Number(n)) => Some(n),
        _ => None,
    })
}
fn negative_literal(number: crate::CssNumericTokenRef<'_>) -> bool {
    let text = number.representation();
    text.starts_with('-')
        && text
            .split(['e', 'E'])
            .next()
            .expect("numeric mantissa")
            .bytes()
            .any(|b| matches!(b, b'1'..=b'9'))
}
fn parse_media_ratio<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssMediaRatio, ParseError<'i, Error>> {
    let operand = |input: &mut Parser<'i, 't>| {
        let value = CssNumberCalculation::from_expression(parse_media_numeric(
            source,
            input,
            numeric,
            CalculationRoot::Number,
        )?);
        if media_literal_number(value.components()).is_some_and(negative_literal) {
            return Err(unsupported_value_at(
                input.current_source_location(),
                None,
                "negative media ratio operand",
            ));
        }
        Ok(value)
    };
    let numerator = operand(input)?;
    let denominator_is_omitted = input.try_parse(|p| p.expect_delim('/')).is_err();
    let denominator = if denominator_is_omitted {
        CssNumberCalculation::try_from_components(
            crate::CssComponentValues::try_new(vec![
                crate::CssComponentValue::try_number("1").expect("default denominator"),
            ])
            .expect("single default denominator"),
        )
        .expect("number default denominator")
    } else {
        operand(input)?
    };
    Ok(CssMediaRatio::new(
        numerator,
        denominator,
        denominator_is_omitted,
    ))
}

fn parse_media_grid<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
) -> Result<CssMediaGrid, ParseError<'i, Error>> {
    let calculation = CssIntegerCalculation::from_expression(parse_media_numeric(
        source,
        input,
        numeric,
        CalculationRoot::Integer,
    )?);
    let literal = if let Some(n) = media_literal_number(calculation.components()) {
        let digits = n
            .representation()
            .trim_start_matches(['+', '-'])
            .trim_start_matches('0');
        Some(if digits.is_empty() {
            CssGridMode::Bitmap
        } else if !n.representation().starts_with('-') && digits == "1" {
            CssGridMode::Grid
        } else {
            return Err(unsupported_value_at(
                input.current_source_location(),
                None,
                "grid literal must be zero or one",
            ));
        })
    } else {
        None
    };
    Ok(CssMediaGrid::new(calculation, literal))
}

fn parse_media_discrete<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    numeric: &MediaInput<'_>,
    id: CssMediaFeatureKind,
) -> Result<CssMediaFeatureQuery, ParseError<'i, Error>> {
    match id {
        CssMediaFeatureKind::Orientation => {
            parse_orientation(input).map(CssMediaFeatureQuery::Orientation)
        }
        CssMediaFeatureKind::Scan => parse_scan_mode(input).map(CssMediaFeatureQuery::Scan),
        CssMediaFeatureKind::PrefersColorScheme => {
            parse_color_scheme_preference(input).map(CssMediaFeatureQuery::PrefersColorScheme)
        }
        CssMediaFeatureKind::PrefersReducedMotion => {
            parse_reduced_motion_preference(input).map(CssMediaFeatureQuery::PrefersReducedMotion)
        }
        CssMediaFeatureKind::PrefersReducedTransparency => {
            parse_reduced_transparency_preference(input)
                .map(CssMediaFeatureQuery::PrefersReducedTransparency)
        }
        CssMediaFeatureKind::PrefersContrast => {
            parse_contrast_preference(input).map(CssMediaFeatureQuery::PrefersContrast)
        }
        CssMediaFeatureKind::ForcedColors => {
            parse_forced_colors_mode(input).map(CssMediaFeatureQuery::ForcedColors)
        }
        CssMediaFeatureKind::Hover => {
            parse_hover_capability(input).map(CssMediaFeatureQuery::Hover)
        }
        CssMediaFeatureKind::AnyHover => {
            parse_hover_capability(input).map(CssMediaFeatureQuery::AnyHover)
        }
        CssMediaFeatureKind::Pointer => {
            parse_pointer_capability(input).map(CssMediaFeatureQuery::Pointer)
        }
        CssMediaFeatureKind::AnyPointer => {
            parse_pointer_capability(input).map(CssMediaFeatureQuery::AnyPointer)
        }
        CssMediaFeatureKind::DisplayMode => {
            parse_display_mode(input).map(CssMediaFeatureQuery::DisplayMode)
        }
        CssMediaFeatureKind::Grid => {
            parse_media_grid(source, input, numeric).map(CssMediaFeatureQuery::Grid)
        }
        CssMediaFeatureKind::Update => parse_discrete_ident(input, id.name(), |ident| match ident
            .to_ascii_lowercase()
            .as_str()
        {
            "none" => Some(CssMediaUpdate::None),
            "slow" => Some(CssMediaUpdate::Slow),
            "fast" => Some(CssMediaUpdate::Fast),
            _ => None,
        })
        .map(CssMediaFeatureQuery::Update),
        CssMediaFeatureKind::OverflowBlock => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaOverflowBlock::None),
                "scroll" => Some(CssMediaOverflowBlock::Scroll),
                "paged" => Some(CssMediaOverflowBlock::Paged),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::OverflowBlock),
        CssMediaFeatureKind::OverflowInline => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaOverflowInline::None),
                "scroll" => Some(CssMediaOverflowInline::Scroll),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::OverflowInline),
        CssMediaFeatureKind::ColorGamut => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "srgb" => Some(CssMediaColorGamut::Srgb),
                "p3" => Some(CssMediaColorGamut::P3),
                "rec2020" => Some(CssMediaColorGamut::Rec2020),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::ColorGamut),
        CssMediaFeatureKind::VideoColorGamut => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "srgb" => Some(CssMediaColorGamut::Srgb),
                "p3" => Some(CssMediaColorGamut::P3),
                "rec2020" => Some(CssMediaColorGamut::Rec2020),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::VideoColorGamut),
        CssMediaFeatureKind::DynamicRange => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "standard" => Some(CssMediaDynamicRange::Standard),
                "high" => Some(CssMediaDynamicRange::High),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::DynamicRange),
        CssMediaFeatureKind::VideoDynamicRange => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "standard" => Some(CssMediaDynamicRange::Standard),
                "high" => Some(CssMediaDynamicRange::High),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::VideoDynamicRange),
        CssMediaFeatureKind::EnvironmentBlending => {
            parse_discrete_ident(input, id.name(), |ident| {
                match ident.to_ascii_lowercase().as_str() {
                    "opaque" => Some(CssMediaEnvironmentBlending::Opaque),
                    "additive" => Some(CssMediaEnvironmentBlending::Additive),
                    "subtractive" => Some(CssMediaEnvironmentBlending::Subtractive),
                    _ => None,
                }
            })
            .map(CssMediaFeatureQuery::EnvironmentBlending)
        }
        CssMediaFeatureKind::InvertedColors => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaInvertedColors::None),
                "inverted" => Some(CssMediaInvertedColors::Inverted),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::InvertedColors),
        CssMediaFeatureKind::NavControls => parse_discrete_ident(input, id.name(), |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaNavigationControls::None),
                "back" => Some(CssMediaNavigationControls::Back),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::NavControls),
        CssMediaFeatureKind::Scripting => {
            parse_discrete_ident(input, id.name(), |ident| {
                match ident.to_ascii_lowercase().as_str() {
                    "none" => Some(CssMediaScripting::None),
                    "initial-only" => Some(CssMediaScripting::InitialOnly),
                    "enabled" => Some(CssMediaScripting::Enabled),
                    _ => None,
                }
            })
            .map(CssMediaFeatureQuery::Scripting)
        }
        CssMediaFeatureKind::PrefersReducedData => {
            parse_discrete_ident(input, id.name(), |ident| {
                match ident.to_ascii_lowercase().as_str() {
                    "no-preference" => Some(CssMediaReducedDataPreference::NoPreference),
                    "reduce" => Some(CssMediaReducedDataPreference::Reduce),
                    _ => None,
                }
            })
            .map(CssMediaFeatureQuery::PrefersReducedData)
        }
        _ => unreachable!("discrete media feature catalog"),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RangePrefix {
    Min,
    Max,
}

#[derive(Clone, Copy, PartialEq)]
struct MediaFeatureName {
    id: CssMediaFeatureKind,
    prefix: Option<RangePrefix>,
}

#[derive(Clone, Copy)]
enum ContainerFeatureName {
    Width(Option<RangePrefix>),
    Height(Option<RangePrefix>),
    InlineSize(Option<RangePrefix>),
    BlockSize(Option<RangePrefix>),
    AspectRatio(Option<RangePrefix>),
    Orientation,
}

impl ContainerFeatureName {
    fn parse(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "width" => Self::Width(None),
            "min-width" => Self::Width(Some(RangePrefix::Min)),
            "max-width" => Self::Width(Some(RangePrefix::Max)),
            "height" => Self::Height(None),
            "min-height" => Self::Height(Some(RangePrefix::Min)),
            "max-height" => Self::Height(Some(RangePrefix::Max)),
            "inline-size" => Self::InlineSize(None),
            "min-inline-size" => Self::InlineSize(Some(RangePrefix::Min)),
            "max-inline-size" => Self::InlineSize(Some(RangePrefix::Max)),
            "block-size" => Self::BlockSize(None),
            "min-block-size" => Self::BlockSize(Some(RangePrefix::Min)),
            "max-block-size" => Self::BlockSize(Some(RangePrefix::Max)),
            "aspect-ratio" => Self::AspectRatio(None),
            "min-aspect-ratio" => Self::AspectRatio(Some(RangePrefix::Min)),
            "max-aspect-ratio" => Self::AspectRatio(Some(RangePrefix::Max)),
            "orientation" => Self::Orientation,
            _ => return None,
        })
    }
}

impl MediaFeatureName {
    fn parse(name: &str) -> Option<Self> {
        if let Some(id) = CssMediaFeatureKind::from_name(name) {
            return Some(Self { id, prefix: None });
        }
        let lower = name.to_ascii_lowercase();
        let (name, prefix) = if let Some(v) = lower.strip_prefix("min-") {
            (v, RangePrefix::Min)
        } else {
            (lower.strip_prefix("max-")?, RangePrefix::Max)
        };
        let id = CssMediaFeatureKind::from_name(name)?;
        (id.family() != MediaValueFamily::Discrete).then_some(Self {
            id,
            prefix: Some(prefix),
        })
    }
    fn boolean_kind(self) -> Option<CssMediaFeatureKind> {
        self.prefix.is_none().then_some(self.id)
    }
}

fn parse_range_feature_comparison<'i, 't>(
    input: &mut Parser<'i, 't>,
    prefix: Option<RangePrefix>,
) -> std::result::Result<Option<CssQueryComparison>, ParseError<'i, Error>> {
    if input.try_parse(Parser::expect_colon).is_ok() {
        return Ok(Some(match prefix {
            Some(RangePrefix::Min) => CssQueryComparison::GreaterThanOrEqual,
            Some(RangePrefix::Max) => CssQueryComparison::LessThanOrEqual,
            None => CssQueryComparison::Equal,
        }));
    }

    if prefix.is_some() {
        return Err(invalid_syntax(
            input.current_source_location(),
            "prefixed media range features require colon syntax",
        ));
    }

    parse_query_comparison(input).map(Some)
}

fn parse_query_comparison<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssQueryComparison, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let Token::Delim(delim) = input.next().map_err(basic)? else {
        return Err(invalid_syntax(
            location,
            "expected media feature comparison",
        ));
    };
    let delim = *delim;

    match delim {
        '<' if input.try_parse(|input| input.expect_delim('=')).is_ok() => {
            Ok(CssQueryComparison::LessThanOrEqual)
        }
        '<' => Ok(CssQueryComparison::LessThan),
        '>' if input.try_parse(|input| input.expect_delim('=')).is_ok() => {
            Ok(CssQueryComparison::GreaterThanOrEqual)
        }
        '>' => Ok(CssQueryComparison::GreaterThan),
        '=' => Ok(CssQueryComparison::Equal),
        _ => Err(invalid_syntax(
            location,
            "expected media feature comparison",
        )),
    }
}

fn parse_query_length<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssQueryLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, unit, .. } => {
            let Some(unit) = CssLengthUnit::from_css_unit(unit) else {
                return Err(unsupported_value_at(
                    location,
                    None,
                    format!("unknown media query length unit `{unit}`"),
                ));
            };
            CssQueryLength::try_new(*value, unit).ok_or_else(|| {
                unsupported_value_at(location, None, "unsupported media query length")
            })
        }
        Token::Number { value, .. } if *value == 0.0 => Ok(CssQueryLength::unitless_zero()),
        token => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported media query length `{}`", token.to_css_string()),
        )),
    }
}

fn parse_ratio<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRatio, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let numerator = match input.next().map_err(basic)? {
        Token::Number { value, .. } => *value,
        token => {
            return Err(unsupported_value_at(
                location,
                None,
                format!("unsupported query ratio `{}`", token.to_css_string()),
            ));
        }
    };

    input.expect_delim('/').map_err(basic)?;

    let denominator_location = input.current_source_location();
    let denominator = match input.next().map_err(basic)? {
        Token::Number { value, .. } => *value,
        token => {
            return Err(unsupported_value_at(
                denominator_location,
                None,
                format!("unsupported query ratio `{}`", token.to_css_string()),
            ));
        }
    };

    CssRatio::try_new(numerator, denominator)
        .ok_or_else(|| unsupported_value_at(location, None, "unsupported query ratio"))
}

fn parse_orientation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOrientation, ParseError<'i, Error>> {
    parse_discrete_ident(input, "orientation", |ident| {
        match_ignore_ascii_case! { ident,
            "portrait" => Some(CssOrientation::Portrait),
            "landscape" => Some(CssOrientation::Landscape),
            _ => None,
        }
    })
}

fn parse_scan_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssScanMode, ParseError<'i, Error>> {
    parse_discrete_ident(input, "scan", |ident| {
        match_ignore_ascii_case! { ident,
            "progressive" => Some(CssScanMode::Progressive),
            "interlace" => Some(CssScanMode::Interlace),
            _ => None,
        }
    })
}

fn parse_color_scheme_preference<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorSchemePreference, ParseError<'i, Error>> {
    parse_discrete_ident(input, "prefers-color-scheme", |ident| {
        match_ignore_ascii_case! { ident,
            "light" => Some(CssColorSchemePreference::Light),
            "dark" => Some(CssColorSchemePreference::Dark),
            _ => None,
        }
    })
}

fn parse_reduced_motion_preference<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssReducedMotionPreference, ParseError<'i, Error>> {
    parse_discrete_ident(input, "prefers-reduced-motion", |ident| {
        match_ignore_ascii_case! { ident,
            "reduce" => Some(CssReducedMotionPreference::Reduce),
            "no-preference" => Some(CssReducedMotionPreference::NoPreference),
            _ => None,
        }
    })
}

fn parse_reduced_transparency_preference<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssReducedTransparencyPreference, ParseError<'i, Error>> {
    parse_discrete_ident(input, "prefers-reduced-transparency", |ident| {
        match_ignore_ascii_case! { ident,
            "reduce" => Some(CssReducedTransparencyPreference::Reduce),
            "no-preference" => Some(CssReducedTransparencyPreference::NoPreference),
            _ => None,
        }
    })
}

fn parse_contrast_preference<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContrastPreference, ParseError<'i, Error>> {
    parse_discrete_ident(input, "prefers-contrast", |ident| {
        match_ignore_ascii_case! { ident,
            "no-preference" => Some(CssContrastPreference::NoPreference),
            "more" => Some(CssContrastPreference::More),
            "less" => Some(CssContrastPreference::Less),
            "custom" => Some(CssContrastPreference::Custom),
            _ => None,
        }
    })
}

fn parse_forced_colors_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssForcedColorsMode, ParseError<'i, Error>> {
    parse_discrete_ident(input, "forced-colors", |ident| {
        match_ignore_ascii_case! { ident,
            "none" => Some(CssForcedColorsMode::None),
            "active" => Some(CssForcedColorsMode::Active),
            _ => None,
        }
    })
}

fn parse_hover_capability<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssHoverCapability, ParseError<'i, Error>> {
    parse_discrete_ident(input, "hover", |ident| {
        match_ignore_ascii_case! { ident,
            "none" => Some(CssHoverCapability::None),
            "hover" => Some(CssHoverCapability::Hover),
            _ => None,
        }
    })
}

fn parse_pointer_capability<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPointerCapability, ParseError<'i, Error>> {
    parse_discrete_ident(input, "pointer", |ident| {
        match_ignore_ascii_case! { ident,
            "none" => Some(CssPointerCapability::None),
            "coarse" => Some(CssPointerCapability::Coarse),
            "fine" => Some(CssPointerCapability::Fine),
            _ => None,
        }
    })
}

fn parse_display_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssDisplayMode, ParseError<'i, Error>> {
    parse_discrete_ident(input, "display-mode", |ident| {
        match_ignore_ascii_case! { ident,
            "fullscreen" => Some(CssDisplayMode::Fullscreen),
            "standalone" => Some(CssDisplayMode::Standalone),
            "minimal-ui" => Some(CssDisplayMode::MinimalUi),
            "browser" => Some(CssDisplayMode::Browser),
            "picture-in-picture" => Some(CssDisplayMode::PictureInPicture),
            _ => None,
        }
    })
}

fn parse_discrete_ident<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    feature: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> std::result::Result<T, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    parse(&ident).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            format!("unsupported {feature} value `{ident}`"),
        )
    })
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
        let items = match self.numeric {
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
    let mut parser_input = cssparser::ParserInput::new(source);
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

fn generic_comparison<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &MediaInput<'_>,
) -> Result<(CssQueryComparison, Vec<CssValueOrigin>), ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let comparison = parse_media_comparison(input)?;
    let components = numeric.between(input, &start)?;
    let origins = components
        .items()
        .iter()
        .filter(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Delim(_))
            )
        })
        .map(|c| c.origin().clone())
        .collect();
    Ok((comparison, origins))
}
