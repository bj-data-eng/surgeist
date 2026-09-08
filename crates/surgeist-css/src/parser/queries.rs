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
    CssFeatureId, Error, basic, from_parse_error, invalid_syntax, unsupported_value_at,
    with_media_query_context,
};
use crate::syntax::*;

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
    if input.is_exhausted() {
        return Ok(CssMediaQueryList::new(Vec::new()));
    }

    let mut queries = Vec::new();
    let mut preceding_comma = None;
    loop {
        let member_start = input.position().byte_index();
        let result = input.parse_until_before(Delimiter::Comma, |member| {
            let _ = recovery.check_specialized_components(
                source,
                member,
                "baseline.media.query-list",
            )?;
            parse_media_query(source, member)
        });
        let member_end = input.position().byte_index();
        let comma_start = member_end;
        let following_comma = match input.next().cloned() {
            Ok(Token::Comma) => Some((comma_start, input.position().byte_index())),
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
            Ok(query) => queries.push(query),
            Err(error) => {
                let action = recovery_action_for_error(
                    &error,
                    crate::CssRecoveryAction::ReplaceMediaQueryWithNever,
                );
                let error = if action == crate::CssRecoveryAction::StopAtNestingLimit {
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
                queries.push(CssMediaQuery::Never(CssNeverMediaQuery::new(position)));
            }
        }

        let Some(comma) = following_comma else {
            break;
        };
        preceding_comma = Some(comma);
    }
    Ok(CssMediaQueryList::new(queries))
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
    if input
        .try_parse(|input| input.expect_ident_matching("not"))
        .is_ok()
    {
        return Ok(CssContainerCondition::Not(Box::new(
            parse_container_condition_atom(input)?,
        )));
    }

    if let Ok(style) = input.try_parse(parse_container_style_query) {
        return Ok(CssContainerCondition::Style(style));
    }

    input.expect_parenthesis_block().map_err(basic)?;
    let feature = input.parse_nested_block(|input| {
        let feature = parse_container_feature_query(input)?;
        if !input.is_exhausted() {
            return Err(invalid_syntax(
                input.current_source_location(),
                "unexpected token in container feature query",
            ));
        }
        Ok(feature)
    })?;
    Ok(CssContainerCondition::Feature(feature))
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

fn parse_media_query<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMediaQuery, ParseError<'i, Error>> {
    let position = first_non_trivia_parser_position(input);
    if let Ok(query) = input.try_parse(|input| parse_typed_media_query(source, input, position)) {
        return Ok(CssMediaQuery::Typed(query));
    }

    parse_media_condition(source, input).map(CssMediaQuery::Condition)
}

fn parse_typed_media_query<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
    position: crate::CssSourcePosition,
) -> std::result::Result<CssTypedMediaQuery, ParseError<'i, Error>> {
    let modifier = input.try_parse(parse_media_query_modifier).ok();
    let media_type = parse_media_type(source, input)?;
    let condition = if input
        .try_parse(|input| input.expect_ident_matching("and"))
        .is_ok()
    {
        Some(parse_media_condition(source, input)?)
    } else {
        None
    };

    Ok(match media_type {
        ParsedMediaType::Known(media_type) => {
            CssTypedMediaQuery::new(modifier, media_type, condition, position)
        }
        ParsedMediaType::Unknown(media_type) => {
            CssTypedMediaQuery::new_unknown(modifier, media_type, condition, position)
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
            position,
        ))),
    }
}

fn parse_media_condition<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMediaCondition, ParseError<'i, Error>> {
    let position = first_non_trivia_parser_position(input);
    let first = parse_media_condition_atom(source, input)?;

    if input
        .try_parse(|input| input.expect_ident_matching("and"))
        .is_ok()
    {
        let mut conditions = vec![first, parse_media_condition_atom(source, input)?];
        while input
            .try_parse(|input| input.expect_ident_matching("and"))
            .is_ok()
        {
            conditions.push(parse_media_condition_atom(source, input)?);
        }
        return Ok(CssMediaCondition::new(
            CssMediaConditionKind::And(CssMediaConditionList::new(conditions)),
            position,
        ));
    }

    if input
        .try_parse(|input| input.expect_ident_matching("or"))
        .is_ok()
    {
        let mut conditions = vec![first, parse_media_condition_atom(source, input)?];
        while input
            .try_parse(|input| input.expect_ident_matching("or"))
            .is_ok()
        {
            conditions.push(parse_media_condition_atom(source, input)?);
        }
        return Ok(CssMediaCondition::new(
            CssMediaConditionKind::Or(CssMediaConditionList::new(conditions)),
            position,
        ));
    }

    Ok(first)
}

fn parse_media_condition_atom<'i, 't>(
    source: &str,
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMediaCondition, ParseError<'i, Error>> {
    let position = first_non_trivia_parser_position(input);
    if input
        .try_parse(|input| input.expect_ident_matching("not"))
        .is_ok()
    {
        return Ok(CssMediaCondition::new(
            CssMediaConditionKind::Not(Box::new(parse_media_condition_atom(source, input)?)),
            position,
        ));
    }

    let expression_start = position.byte_offset().value();
    input.expect_parenthesis_block().map_err(basic)?;
    let parsed = input.parse_nested_block(|input| {
        let initial = input.state();
        match parse_media_feature_query(input) {
            Ok(feature) if input.is_exhausted() => Ok(ParsedMediaConditionAtom::Feature(feature)),
            Ok(_) => {
                let location = input.current_source_location();
                input.reset(&initial);
                parse_defined_false_media_reason(input)
                    .map(ParsedMediaConditionAtom::DefinedFalse)
                    .ok_or_else(|| {
                        invalid_syntax(location, "unexpected token in media feature query")
                    })
            }
            Err(error) => {
                input.reset(&initial);
                if let Some(reason) = parse_defined_false_media_reason(input) {
                    Ok(ParsedMediaConditionAtom::DefinedFalse(reason))
                } else {
                    Err(error)
                }
            }
        }
    })?;
    let kind = match parsed {
        ParsedMediaConditionAtom::Feature(feature) => CssMediaConditionKind::Feature(feature),
        ParsedMediaConditionAtom::DefinedFalse(reason) => {
            let expression_end = input.position().byte_index();
            let authored = source
                .get(expression_start..expression_end)
                .unwrap_or_default();
            CssMediaConditionKind::DefinedFalse(CssDefinedFalseMediaCondition::new(
                authored, reason, position,
            ))
        }
    };
    Ok(CssMediaCondition::new(kind, position))
}

enum ParsedMediaConditionAtom {
    Feature(CssMediaFeatureQuery),
    DefinedFalse(CssDefinedFalseMediaReason),
}

fn parse_defined_false_media_reason(
    input: &mut Parser<'_, '_>,
) -> Option<CssDefinedFalseMediaReason> {
    let ident = input.expect_ident_cloned().ok()?;
    if ident.eq_ignore_ascii_case("scripting") {
        return None;
    }

    let feature_name = MediaFeatureName::parse(&ident);
    if feature_name.is_some_and(|name| !name.is_mq3()) {
        return None;
    }
    if input.is_exhausted() {
        return feature_name
            .is_none()
            .then_some(CssDefinedFalseMediaReason::UnknownFeature);
    }

    let has_value_separator = match feature_name {
        Some(name) if name.is_range() => {
            parse_range_feature_comparison(input, name.prefix()).is_ok()
        }
        Some(_) | None => input.expect_colon().is_ok(),
    };
    if !has_value_separator || input.is_exhausted() {
        return None;
    }

    while input.next_including_whitespace_and_comments().is_ok() {}
    Some(if feature_name.is_some() {
        CssDefinedFalseMediaReason::UnknownValue
    } else {
        CssDefinedFalseMediaReason::UnknownFeature
    })
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
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMediaFeatureQuery, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let Some(feature_name) = MediaFeatureName::parse(&ident) else {
        return Err(with_media_query_context(
            unsupported_value_at(
                location,
                None,
                format!("unsupported media feature `{ident}`"),
            ),
            Some(ident.as_ref()),
        ));
    };

    if input.is_exhausted() {
        return feature_name
            .boolean_kind()
            .map(CssMediaFeatureQuery::Boolean)
            .ok_or_else(|| {
                invalid_syntax(
                    input.current_source_location(),
                    "prefixed media features require a value",
                )
            });
    }

    match feature_name {
        MediaFeatureName::Width(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssMediaFeatureQuery::Width(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::Height(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssMediaFeatureQuery::Height(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::DeviceWidth(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssMediaFeatureQuery::DeviceWidth(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::DeviceHeight(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_query_length(input)?;
            Ok(CssMediaFeatureQuery::DeviceHeight(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::AspectRatio(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_media_ratio(input)?;
            Ok(CssMediaFeatureQuery::AspectRatio(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::DeviceAspectRatio(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_media_ratio(input)?;
            Ok(CssMediaFeatureQuery::DeviceAspectRatio(
                CssRangeFeature::new(comparison, value),
            ))
        }
        MediaFeatureName::Resolution(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_resolution(input)?;
            Ok(CssMediaFeatureQuery::Resolution(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::Color(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_non_negative_integer(input)?;
            Ok(CssMediaFeatureQuery::Color(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::ColorIndex(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_non_negative_integer(input)?;
            Ok(CssMediaFeatureQuery::ColorIndex(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::Monochrome(prefix) => {
            let comparison = parse_range_feature_comparison(input, prefix)?;
            let value = parse_non_negative_integer(input)?;
            Ok(CssMediaFeatureQuery::Monochrome(CssRangeFeature::new(
                comparison, value,
            )))
        }
        MediaFeatureName::Orientation => {
            input.expect_colon().map_err(basic)?;
            parse_orientation(input).map(CssMediaFeatureQuery::Orientation)
        }
        MediaFeatureName::Scan => {
            input.expect_colon().map_err(basic)?;
            parse_scan_mode(input).map(CssMediaFeatureQuery::Scan)
        }
        MediaFeatureName::Grid => {
            input.expect_colon().map_err(basic)?;
            parse_grid_mode(input).map(CssMediaFeatureQuery::Grid)
        }
        MediaFeatureName::PrefersColorScheme => {
            input.expect_colon().map_err(basic)?;
            parse_color_scheme_preference(input).map(CssMediaFeatureQuery::PrefersColorScheme)
        }
        MediaFeatureName::PrefersReducedMotion => {
            input.expect_colon().map_err(basic)?;
            parse_reduced_motion_preference(input).map(CssMediaFeatureQuery::PrefersReducedMotion)
        }
        MediaFeatureName::PrefersReducedTransparency => {
            input.expect_colon().map_err(basic)?;
            parse_reduced_transparency_preference(input)
                .map(CssMediaFeatureQuery::PrefersReducedTransparency)
        }
        MediaFeatureName::PrefersContrast => {
            input.expect_colon().map_err(basic)?;
            parse_contrast_preference(input).map(CssMediaFeatureQuery::PrefersContrast)
        }
        MediaFeatureName::ForcedColors => {
            input.expect_colon().map_err(basic)?;
            parse_forced_colors_mode(input).map(CssMediaFeatureQuery::ForcedColors)
        }
        MediaFeatureName::Hover => {
            input.expect_colon().map_err(basic)?;
            parse_hover_capability(input).map(CssMediaFeatureQuery::Hover)
        }
        MediaFeatureName::AnyHover => {
            input.expect_colon().map_err(basic)?;
            parse_hover_capability(input).map(CssMediaFeatureQuery::AnyHover)
        }
        MediaFeatureName::Pointer => {
            input.expect_colon().map_err(basic)?;
            parse_pointer_capability(input).map(CssMediaFeatureQuery::Pointer)
        }
        MediaFeatureName::AnyPointer => {
            input.expect_colon().map_err(basic)?;
            parse_pointer_capability(input).map(CssMediaFeatureQuery::AnyPointer)
        }
        MediaFeatureName::DisplayMode => {
            input.expect_colon().map_err(basic)?;
            parse_display_mode(input).map(CssMediaFeatureQuery::DisplayMode)
        }
    }
}

#[derive(Clone, Copy)]
enum RangePrefix {
    Min,
    Max,
}

#[derive(Clone, Copy)]
enum MediaFeatureName {
    Width(Option<RangePrefix>),
    Height(Option<RangePrefix>),
    DeviceWidth(Option<RangePrefix>),
    DeviceHeight(Option<RangePrefix>),
    AspectRatio(Option<RangePrefix>),
    DeviceAspectRatio(Option<RangePrefix>),
    Resolution(Option<RangePrefix>),
    Color(Option<RangePrefix>),
    ColorIndex(Option<RangePrefix>),
    Monochrome(Option<RangePrefix>),
    Orientation,
    Scan,
    Grid,
    PrefersColorScheme,
    PrefersReducedMotion,
    PrefersReducedTransparency,
    PrefersContrast,
    ForcedColors,
    Hover,
    AnyHover,
    Pointer,
    AnyPointer,
    DisplayMode,
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
        Some(match name.to_ascii_lowercase().as_str() {
            "width" => Self::Width(None),
            "min-width" => Self::Width(Some(RangePrefix::Min)),
            "max-width" => Self::Width(Some(RangePrefix::Max)),
            "height" => Self::Height(None),
            "min-height" => Self::Height(Some(RangePrefix::Min)),
            "max-height" => Self::Height(Some(RangePrefix::Max)),
            "device-width" => Self::DeviceWidth(None),
            "min-device-width" => Self::DeviceWidth(Some(RangePrefix::Min)),
            "max-device-width" => Self::DeviceWidth(Some(RangePrefix::Max)),
            "device-height" => Self::DeviceHeight(None),
            "min-device-height" => Self::DeviceHeight(Some(RangePrefix::Min)),
            "max-device-height" => Self::DeviceHeight(Some(RangePrefix::Max)),
            "aspect-ratio" => Self::AspectRatio(None),
            "min-aspect-ratio" => Self::AspectRatio(Some(RangePrefix::Min)),
            "max-aspect-ratio" => Self::AspectRatio(Some(RangePrefix::Max)),
            "device-aspect-ratio" => Self::DeviceAspectRatio(None),
            "min-device-aspect-ratio" => Self::DeviceAspectRatio(Some(RangePrefix::Min)),
            "max-device-aspect-ratio" => Self::DeviceAspectRatio(Some(RangePrefix::Max)),
            "resolution" => Self::Resolution(None),
            "min-resolution" => Self::Resolution(Some(RangePrefix::Min)),
            "max-resolution" => Self::Resolution(Some(RangePrefix::Max)),
            "color" => Self::Color(None),
            "min-color" => Self::Color(Some(RangePrefix::Min)),
            "max-color" => Self::Color(Some(RangePrefix::Max)),
            "color-index" => Self::ColorIndex(None),
            "min-color-index" => Self::ColorIndex(Some(RangePrefix::Min)),
            "max-color-index" => Self::ColorIndex(Some(RangePrefix::Max)),
            "monochrome" => Self::Monochrome(None),
            "min-monochrome" => Self::Monochrome(Some(RangePrefix::Min)),
            "max-monochrome" => Self::Monochrome(Some(RangePrefix::Max)),
            "orientation" => Self::Orientation,
            "scan" => Self::Scan,
            "grid" => Self::Grid,
            "prefers-color-scheme" => Self::PrefersColorScheme,
            "prefers-reduced-motion" => Self::PrefersReducedMotion,
            "prefers-reduced-transparency" => Self::PrefersReducedTransparency,
            "prefers-contrast" => Self::PrefersContrast,
            "forced-colors" => Self::ForcedColors,
            "hover" => Self::Hover,
            "any-hover" => Self::AnyHover,
            "pointer" => Self::Pointer,
            "any-pointer" => Self::AnyPointer,
            "display-mode" => Self::DisplayMode,
            _ => return None,
        })
    }

    fn boolean_kind(self) -> Option<CssMediaFeatureKind> {
        Some(match self {
            Self::Width(None) => CssMediaFeatureKind::Width,
            Self::Height(None) => CssMediaFeatureKind::Height,
            Self::DeviceWidth(None) => CssMediaFeatureKind::DeviceWidth,
            Self::DeviceHeight(None) => CssMediaFeatureKind::DeviceHeight,
            Self::AspectRatio(None) => CssMediaFeatureKind::AspectRatio,
            Self::DeviceAspectRatio(None) => CssMediaFeatureKind::DeviceAspectRatio,
            Self::Resolution(None) => CssMediaFeatureKind::Resolution,
            Self::Color(None) => CssMediaFeatureKind::Color,
            Self::ColorIndex(None) => CssMediaFeatureKind::ColorIndex,
            Self::Monochrome(None) => CssMediaFeatureKind::Monochrome,
            Self::Orientation => CssMediaFeatureKind::Orientation,
            Self::Scan => CssMediaFeatureKind::Scan,
            Self::Grid => CssMediaFeatureKind::Grid,
            Self::Width(Some(_))
            | Self::Height(Some(_))
            | Self::DeviceWidth(Some(_))
            | Self::DeviceHeight(Some(_))
            | Self::AspectRatio(Some(_))
            | Self::DeviceAspectRatio(Some(_))
            | Self::Resolution(Some(_))
            | Self::Color(Some(_))
            | Self::ColorIndex(Some(_))
            | Self::Monochrome(Some(_))
            | Self::PrefersColorScheme
            | Self::PrefersReducedMotion
            | Self::PrefersReducedTransparency
            | Self::PrefersContrast
            | Self::ForcedColors
            | Self::Hover
            | Self::AnyHover
            | Self::Pointer
            | Self::AnyPointer
            | Self::DisplayMode => return None,
        })
    }

    fn is_mq3(self) -> bool {
        !matches!(
            self,
            Self::PrefersColorScheme
                | Self::PrefersReducedMotion
                | Self::PrefersReducedTransparency
                | Self::PrefersContrast
                | Self::ForcedColors
                | Self::Hover
                | Self::AnyHover
                | Self::Pointer
                | Self::AnyPointer
                | Self::DisplayMode
        )
    }

    fn is_range(self) -> bool {
        matches!(
            self,
            Self::Width(_)
                | Self::Height(_)
                | Self::DeviceWidth(_)
                | Self::DeviceHeight(_)
                | Self::AspectRatio(_)
                | Self::DeviceAspectRatio(_)
                | Self::Resolution(_)
                | Self::Color(_)
                | Self::ColorIndex(_)
                | Self::Monochrome(_)
        )
    }

    fn prefix(self) -> Option<RangePrefix> {
        match self {
            Self::Width(prefix)
            | Self::Height(prefix)
            | Self::DeviceWidth(prefix)
            | Self::DeviceHeight(prefix)
            | Self::AspectRatio(prefix)
            | Self::DeviceAspectRatio(prefix)
            | Self::Resolution(prefix)
            | Self::Color(prefix)
            | Self::ColorIndex(prefix)
            | Self::Monochrome(prefix) => prefix,
            Self::Orientation
            | Self::Scan
            | Self::Grid
            | Self::PrefersColorScheme
            | Self::PrefersReducedMotion
            | Self::PrefersReducedTransparency
            | Self::PrefersContrast
            | Self::ForcedColors
            | Self::Hover
            | Self::AnyHover
            | Self::Pointer
            | Self::AnyPointer
            | Self::DisplayMode => None,
        }
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

fn parse_media_ratio<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMediaRatio, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let numerator = parse_positive_integer(input, "media query ratio")?;
    input.expect_delim('/').map_err(basic)?;
    let denominator = parse_positive_integer(input, "media query ratio")?;
    CssMediaRatio::try_new(numerator, denominator)
        .ok_or_else(|| unsupported_value_at(location, None, "unsupported media query ratio"))
}

fn parse_positive_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    domain: &str,
) -> std::result::Result<u32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => u32::try_from(*value)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| unsupported_value_at(location, None, format!("unsupported {domain}"))),
        token => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported {domain} `{}`", token.to_css_string()),
        )),
    }
}

fn parse_resolution<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssResolution, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, unit, .. } => {
            let unit = match_ignore_ascii_case! { unit,
                "dpi" => CssResolutionUnit::Dpi,
                "dpcm" => CssResolutionUnit::Dpcm,
                "dppx" => CssResolutionUnit::Dppx,
                _ => return Err(unsupported_value_at(
                    location,
                    None,
                    format!("unknown media query resolution unit `{unit}`"),
                )),
            };
            CssResolution::try_new(*value, unit).ok_or_else(|| {
                unsupported_value_at(location, None, "unsupported media query resolution")
            })
        }
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "unsupported media query resolution `{}`",
                token.to_css_string()
            ),
        )),
    }
}

fn parse_non_negative_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssNonNegativeInteger, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => u32::try_from(*value)
            .map(CssNonNegativeInteger::new)
            .map_err(|_| unsupported_value_at(location, None, "unsupported negative integer")),
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "unsupported media query integer `{}`",
                token.to_css_string()
            ),
        )),
    }
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

fn parse_grid_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGridMode, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(0), ..
        } => Ok(CssGridMode::Bitmap),
        Token::Number {
            int_value: Some(1), ..
        } => Ok(CssGridMode::Grid),
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "unsupported grid value `{}`; expected 0 or 1",
                token.to_css_string()
            ),
        )),
    }
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
