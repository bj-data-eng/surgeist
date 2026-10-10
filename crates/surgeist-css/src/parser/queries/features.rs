//! Feature shapes, known-domain admission, and when-media leaf construction.

use super::super::query_components::{
    RangePrefix, media_literal_number, negative_literal, query_comparison, query_plain_range,
    query_range_chain,
};
use super::{MediaInput, media_committed_error, media_terminal_error};
use crate::error::{Error, basic, invalid_syntax, unexpected_at};
use crate::media::{MediaFeatureShape, MediaFeatureSyntax};
use crate::media_features::{MediaRangeState, MediaValueFamily};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::syntax::*;
use crate::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssValueOrigin, CssValueTokenRef,
};
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

pub(super) enum AdmittedMediaFeature {
    Feature(CssMediaFeatureQuery, Box<MediaFeatureSyntax>),
    Unknown(CssUnknownMediaFeature),
}
pub(super) fn classify_generic_media_feature<'i>(
    source: &str,
    input: &mut Parser<'i, '_>,
    numeric: &MediaInput<'_>,
    initial: &cssparser::ParserState,
    mut syntax: MediaFeatureSyntax,
) -> Result<AdmittedMediaFeature, ParseError<'i, Error>> {
    let reason = if let Some(name) = known_generic_name(&syntax) {
        syntax.canonical_name = Some(syntax.name_text.to_ascii_lowercase());
        let discrete_range = name.id.family() == MediaValueFamily::Discrete
            && matches!(&syntax.shape, MediaFeatureShape::Range(range)
                if !matches!(range.view(), CssMediaRangeRef::Plain { .. }));
        if discrete_range {
            CssUnknownMediaFeatureReason::InvalidOperation
        } else {
            input.reset(initial);
            match parse_media_feature_query(source, input, numeric) {
                Ok(feature) if input.is_exhausted() => {
                    return Ok(AdmittedMediaFeature::Feature(feature, Box::new(syntax)));
                }
                Err(error) if media_committed_error(&error) => return Err(error),
                _ => {}
            }
            CssUnknownMediaFeatureReason::InvalidValue
        }
    } else {
        CssUnknownMediaFeatureReason::UnknownName
    };
    while input.next_including_whitespace_and_comments().is_ok() {}
    Ok(AdmittedMediaFeature::Unknown(CssUnknownMediaFeature::new(
        syntax, reason,
    )))
}
/// Exact single-feature admission over the actual media() enclosure. Unlike
/// media-in-parens, an extension identifier is an unknown boolean feature.
pub(super) fn construct_when_media_feature(
    component: CssComponentValue,
    limits: CssComponentValueLimits,
) -> Result<crate::CssWhenMediaFeature, crate::CssWhenConstructionError> {
    use crate::{CssWhenConstructionError, CssWhenMediaFeatureKind};
    let invalid = || CssWhenConstructionError::InvalidMediaFeatureGrammar {
        origin: component.origin().clone(),
    };
    if !matches!(component.view(), CssComponentValueRef::Function(function) if function.name().eq_ignore_ascii_case("media"))
    {
        return Err(invalid());
    }
    let values = CssComponentValues::try_new_with_limits(vec![component.clone()], limits)?;
    let serialized = values.serialize_with_limit(limits.max_css_bytes())?;
    let numeric = NumericInputContext::components(&values, &serialized);
    let numeric = MediaInput {
        numeric: &numeric,
        limits,
    };
    let source = serialized.as_css();
    let working_source = crate::tokenization::prepare(source);
    let mut parser_input = cssparser::ParserInput::new(&working_source);
    let mut input = Parser::new(&mut parser_input);
    let result: Result<_, ParseError<'_, Error>> = (|| {
        input.expect_function_matching("media")?;
        input.parse_nested_block(|input| {
            let initial = input.state();
            let syntax = parse_generic_media_feature(source, input, &numeric, component.clone())?;
            classify_generic_media_feature(source, input, &numeric, &initial, syntax)
        })
    })();
    let admitted = result.map_err(|error| {
        if let cssparser::ParseErrorKind::Custom(error) = &error.kind
            && let crate::ErrorKind::InvalidComponentValue(component) = error.kind()
        {
            return CssWhenConstructionError::Component(component.as_ref().clone());
        }
        invalid()
    })?;
    let kind = match admitted {
        AdmittedMediaFeature::Feature(feature, _) => CssWhenMediaFeatureKind::Feature(feature),
        AdmittedMediaFeature::Unknown(feature) => CssWhenMediaFeatureKind::UnknownFeature(feature),
    };
    Ok(crate::CssWhenMediaFeature::new(kind, component))
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
pub(super) fn parse_generic_media_feature<'i, 't>(
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
                return Err(invalid_syntax(input.current_source_location()));
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
            return Err(invalid_syntax(start.source_location()));
        }
    };
    if can_ratio && input.try_parse(|p| p.expect_delim('/')).is_ok() {
        let denominator = parse_media_numeric(source, input, numeric, CalculationRoot::Number)
            .map(CssNumberCalculation::from_expression)?;
        if media_literal_number(denominator.components()).is_some_and(negative_literal) {
            return Err(invalid_syntax(start.source_location()));
        }
    }
    numeric.between(input, &start)
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
                .ok_or_else(|| invalid_syntax(input.current_source_location()));
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
        MediaFeatureName::parse(&ident)
            .ok_or_else(|| unexpected_at(input.current_source_location()))?
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
        _ => return Err(invalid_syntax(location)),
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
    Ok(query_comparison(symbol, inclusive))
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
            return Ok(CssMediaRange::new(query_plain_range(value, name.prefix)));
        }
        if name.prefix.is_some() {
            return Err(invalid_syntax(input.current_source_location()));
        }
        let comparison = parse_media_comparison(input)?;
        return value(input)
            .map(|value| CssMediaRange::new(MediaRangeState::FeatureFirst { comparison, value }));
    }
    input.reset(&initial);
    if name.prefix.is_some() {
        return Err(invalid_syntax(input.current_source_location()));
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
    let state = query_range_chain(left, first, right, second)
        .ok_or_else(|| invalid_syntax(input.current_source_location()))?;
    Ok(CssMediaRange::new(state))
}

pub(super) fn media_numeric_error<'i>(
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
    unexpected_at(numeric.error_location(
        &error,
        input.current_source_location(),
        input.position().byte_index(),
    ))
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
        .map_err(|_| invalid_syntax(input.current_source_location()))?;
    numeric
        .admit(values, root)
        .map_err(|error| media_numeric_error(source, input, numeric, error))
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
            return Err(unexpected_at(input.current_source_location()));
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
            return Err(unexpected_at(input.current_source_location()));
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
        CssMediaFeatureKind::Update => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaUpdate::None),
                "slow" => Some(CssMediaUpdate::Slow),
                "fast" => Some(CssMediaUpdate::Fast),
                _ => None,
            })
            .map(CssMediaFeatureQuery::Update)
        }
        CssMediaFeatureKind::OverflowBlock => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaOverflowBlock::None),
                "scroll" => Some(CssMediaOverflowBlock::Scroll),
                "paged" => Some(CssMediaOverflowBlock::Paged),
                _ => None,
            })
            .map(CssMediaFeatureQuery::OverflowBlock)
        }
        CssMediaFeatureKind::OverflowInline => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaOverflowInline::None),
                "scroll" => Some(CssMediaOverflowInline::Scroll),
                _ => None,
            })
            .map(CssMediaFeatureQuery::OverflowInline)
        }
        CssMediaFeatureKind::ColorGamut => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "srgb" => Some(CssMediaColorGamut::Srgb),
                "p3" => Some(CssMediaColorGamut::P3),
                "rec2020" => Some(CssMediaColorGamut::Rec2020),
                _ => None,
            })
            .map(CssMediaFeatureQuery::ColorGamut)
        }
        CssMediaFeatureKind::VideoColorGamut => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "srgb" => Some(CssMediaColorGamut::Srgb),
                "p3" => Some(CssMediaColorGamut::P3),
                "rec2020" => Some(CssMediaColorGamut::Rec2020),
                _ => None,
            })
            .map(CssMediaFeatureQuery::VideoColorGamut)
        }
        CssMediaFeatureKind::DynamicRange => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "standard" => Some(CssMediaDynamicRange::Standard),
                "high" => Some(CssMediaDynamicRange::High),
                _ => None,
            })
            .map(CssMediaFeatureQuery::DynamicRange)
        }
        CssMediaFeatureKind::VideoDynamicRange => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "standard" => Some(CssMediaDynamicRange::Standard),
                "high" => Some(CssMediaDynamicRange::High),
                _ => None,
            })
            .map(CssMediaFeatureQuery::VideoDynamicRange)
        }
        CssMediaFeatureKind::EnvironmentBlending => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "opaque" => Some(CssMediaEnvironmentBlending::Opaque),
                "additive" => Some(CssMediaEnvironmentBlending::Additive),
                "subtractive" => Some(CssMediaEnvironmentBlending::Subtractive),
                _ => None,
            })
            .map(CssMediaFeatureQuery::EnvironmentBlending)
        }
        CssMediaFeatureKind::InvertedColors => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaInvertedColors::None),
                "inverted" => Some(CssMediaInvertedColors::Inverted),
                _ => None,
            })
            .map(CssMediaFeatureQuery::InvertedColors)
        }
        CssMediaFeatureKind::NavControls => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaNavigationControls::None),
                "back" => Some(CssMediaNavigationControls::Back),
                _ => None,
            })
            .map(CssMediaFeatureQuery::NavControls)
        }
        CssMediaFeatureKind::Scripting => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaScripting::None),
                "initial-only" => Some(CssMediaScripting::InitialOnly),
                "enabled" => Some(CssMediaScripting::Enabled),
                _ => None,
            })
            .map(CssMediaFeatureQuery::Scripting)
        }
        CssMediaFeatureKind::PrefersReducedData => {
            parse_discrete_ident(input, |ident| match ident.to_ascii_lowercase().as_str() {
                "no-preference" => Some(CssMediaReducedDataPreference::NoPreference),
                "reduce" => Some(CssMediaReducedDataPreference::Reduce),
                _ => None,
            })
            .map(CssMediaFeatureQuery::PrefersReducedData)
        }
        _ => unreachable!("discrete media feature catalog"),
    }
}

#[derive(Clone, Copy, PartialEq)]
struct MediaFeatureName {
    id: CssMediaFeatureKind,
    prefix: Option<RangePrefix>,
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

fn parse_orientation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOrientation, ParseError<'i, Error>> {
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse_discrete_ident(input, |ident| {
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
    parse: impl FnOnce(&str) -> Option<T>,
) -> std::result::Result<T, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    parse(&ident).ok_or_else(|| unexpected_at(location))
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
