//! Feature shapes, known-domain admission, and when-media leaf construction.

use super::super::descriptor_stream::{
    Value, ValueError, consume_remaining, expect_exhausted, from_here, native,
};
use super::super::query_components::{
    RangePrefix, media_literal_number, negative_literal, query_comparison, query_plain_range,
    query_range_chain,
};
use super::super::syntax_types::{Mark, Site, Stream, SyntaxError};
use super::{MediaInput, media_committed, media_terminal};
use crate::component::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssValueOrigin, CssValueTokenRef,
};
use crate::diagnostic::error::{invalid_component_value, unexpected_at};
use crate::numeric::expression::CssCalculationExpression;
use crate::numeric::expression::{CalculationRoot, NumericInputContext};
use crate::numeric::{
    CssCalculation, CssCalculationType, CssInteger, CssLength, CssNumber, CssResolution,
};
use crate::query::CssQueryComparison;
use crate::query::media::construction::{MediaFeatureShape, MediaFeatureSyntax};
use crate::query::media::media_features::{MediaRangeState, MediaValueFamily};
use crate::query::media::{
    CssColorSchemePreference, CssContrastPreference, CssDisplayMode, CssForcedColorsMode,
    CssGridMode, CssHoverCapability, CssMediaColorGamut, CssMediaDynamicRange,
    CssMediaEnvironmentBlending, CssMediaFeatureKind, CssMediaFeatureQuery, CssMediaGrid,
    CssMediaInteger, CssMediaInvertedColors, CssMediaLength, CssMediaNavigationControls,
    CssMediaOverflowBlock, CssMediaOverflowInline, CssMediaRange, CssMediaRangeRef, CssMediaRatio,
    CssMediaReducedDataPreference, CssMediaResolution, CssMediaScripting, CssMediaUpdate,
    CssOrientation, CssPointerCapability, CssReducedMotionPreference,
    CssReducedTransparencyPreference, CssScanMode, CssUnknownMediaFeature,
    CssUnknownMediaFeatureReason,
};
use surgeist_css_parser::{Token, match_ignore_ascii_case};

pub(super) enum AdmittedMediaFeature {
    Feature(CssMediaFeatureQuery, Box<MediaFeatureSyntax>),
    Unknown(CssUnknownMediaFeature),
}
/// A selected feature shape before its enclosing component is retained.
pub(super) struct ParsedMediaFeature<'i> {
    pub(super) name: CssComponentValue,
    /// Where the feature name starts, for diagnostics naming that name.
    pub(super) name_site: Site<'i>,
    pub(super) name_text: String,
    pub(super) shape: MediaFeatureShape,
    separators: Vec<Vec<CssValueOrigin>>,
}
impl ParsedMediaFeature<'_> {
    pub(super) fn into_syntax(self, component: CssComponentValue) -> MediaFeatureSyntax {
        MediaFeatureSyntax {
            component,
            name: self.name,
            name_text: self.name_text,
            canonical_name: None,
            shape: self.shape,
            separators: self.separators,
        }
    }
}
pub(super) fn classify_generic_media_feature<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
    initial: Mark<'i>,
    mut syntax: MediaFeatureSyntax,
) -> Value<'i, AdmittedMediaFeature> {
    let reason = if let Some(name) = known_generic_name(&syntax.name_text, &syntax.shape) {
        syntax.canonical_name = Some(syntax.name_text.to_ascii_lowercase());
        let discrete_range = name.id.family() == MediaValueFamily::Discrete
            && matches!(&syntax.shape, MediaFeatureShape::Range(range)
                if !matches!(range.view(), CssMediaRangeRef::Plain { .. }));
        if discrete_range {
            CssUnknownMediaFeatureReason::InvalidOperation
        } else {
            input.restore(initial);
            match parse_media_feature_query(source, input, numeric) {
                Ok(feature) if input.only_whitespace_remains() => {
                    return Ok(AdmittedMediaFeature::Feature(feature, Box::new(syntax)));
                }
                Err(error) if media_committed(&error) => return Err(error),
                _ => {}
            }
            CssUnknownMediaFeatureReason::InvalidValue
        }
    } else {
        CssUnknownMediaFeatureReason::UnknownName
    };
    consume_remaining(input);
    Ok(AdmittedMediaFeature::Unknown(CssUnknownMediaFeature::new(
        syntax, reason,
    )))
}
/// Exact single-feature admission over the actual media() enclosure. Unlike
/// media-in-parens, an extension identifier is an unknown boolean feature.
pub(super) fn construct_when_media_feature(
    component: CssComponentValue,
    limits: CssComponentValueLimits,
) -> Result<crate::query::CssWhenMediaFeature, crate::query::CssConditionConstructionError> {
    use crate::query::{CssConditionConstructionError, CssWhenMediaFeatureKind};
    let invalid = || {
        CssConditionConstructionError::from(
            crate::query::when::ConditionConstructionKind::InvalidMediaFeatureGrammar {
                origin: component.origin().clone(),
            },
        )
    };
    if !matches!(component.view(), CssComponentValueRef::Function(function) if function.name().eq_ignore_ascii_case("media"))
    {
        return Err(invalid());
    }
    let values = CssComponentValues::try_new(vec![component.clone()], limits)?;
    let serialized = values.serialize_tokens(limits)?;
    let numeric = NumericInputContext::components(&values, &serialized);
    let media = MediaInput {
        numeric: &numeric,
        limits,
    };
    let source = serialized.as_css();
    let document = crate::syntax_consumption::checked_document(
        values.items(),
        crate::syntax_consumption::SyntaxInputLimits::default(),
        0,
    )?;
    let mut input = document.root();
    let result: Value<'_, _> = (|| {
        input.consume_function_named("media")?;
        input.within_block(|input| {
            let initial = input.mark();
            let syntax =
                parse_generic_media_feature(source, input, &media)?.into_syntax(component.clone());
            classify_generic_media_feature(source, input, &media, initial, syntax)
        })
    })();
    let admitted = result.map_err(|error| {
        let error = native(&numeric, &error);
        if let cssparser::ParseErrorKind::Custom(error) = &error.kind
            && let crate::diagnostic::error::ErrorKind::InvalidComponentValue(component) =
                error.kind()
        {
            return CssConditionConstructionError::from(
                crate::query::when::ConditionConstructionKind::Component(
                    component.as_ref().clone(),
                ),
            );
        }
        invalid()
    })?;
    let kind = match admitted {
        AdmittedMediaFeature::Feature(feature, _) => CssWhenMediaFeatureKind::Feature(feature),
        AdmittedMediaFeature::Unknown(feature) => CssWhenMediaFeatureKind::UnknownFeature(feature),
    };
    Ok(crate::query::CssWhenMediaFeature::new(kind, component))
}
fn known_generic_name(name_text: &str, shape: &MediaFeatureShape) -> Option<MediaFeatureName> {
    if let Some(id) = CssMediaFeatureKind::from_name(name_text) {
        return Some(MediaFeatureName { id, prefix: None });
    }
    if matches!(shape,MediaFeatureShape::Range(range) if matches!(range.view(),CssMediaRangeRef::Plain{..}))
    {
        MediaFeatureName::parse(name_text)
    } else {
        None
    }
}
pub(super) fn parse_generic_media_feature<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, ParsedMediaFeature<'i>> {
    let initial = input.mark();
    let first = input.attempt(|p| generic_feature_first(source, p, numeric));
    match &first {
        Ok(v) if known_generic_name(&v.name_text, &v.shape).is_some() => return first,
        Err(e) if media_terminal(e) => return first,
        _ => {}
    }
    input.restore(initial);
    let second = input.attempt(|p| generic_value_first(source, p, numeric));
    if let Err(e) = &second
        && media_terminal(e)
    {
        return second;
    }
    match (first, second) {
        (_, Ok(second)) if known_generic_name(&second.name_text, &second.shape).is_some() => {
            Ok(second)
        }
        (Ok(first), _) => {
            consume_remaining(input);
            Ok(first)
        }
        (_, other) => other,
    }
}
fn generic_feature_first<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, ParsedMediaFeature<'i>> {
    input.discard_whitespace();
    let name_start = input.mark();
    let name_site = name_start.site();
    let name_text = from_here(input, Stream::consume_ident)?.to_string();
    input.restore(name_start);
    let name = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let mut separators = if input.only_whitespace_remains() {
        Vec::new()
    } else {
        vec![vec![numeric.next_origin(input)]]
    };
    let shape = if input.only_whitespace_remains() {
        MediaFeatureShape::Boolean
    } else if input.attempt(Stream::consume_colon).is_ok() {
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
    expect_exhausted(input)?;
    Ok(ParsedMediaFeature {
        name: name.into_owned(),
        name_site,
        name_text,
        shape,
        separators,
    })
}
fn generic_value_first<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, ParsedMediaFeature<'i>> {
    let left = generic_media_value(source, input, numeric)?;
    let (comparison, origins) = generic_comparison(input, numeric)?;
    let mut separators = vec![origins];
    input.discard_whitespace();
    let name_start = input.mark();
    let name_site = name_start.site();
    let name_text = from_here(input, Stream::consume_ident)?.to_string();
    input.restore(name_start);
    let name = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let state = if input.only_whitespace_remains() {
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
                return Err(numeric.invalid(input.site()));
            }
        }
    };
    expect_exhausted(input)?;
    Ok(ParsedMediaFeature {
        name: name.into_owned(),
        name_site,
        name_text,
        shape: MediaFeatureShape::Range(CssMediaRange::new(state)),
        separators,
    })
}
fn generic_media_value<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssComponentValues> {
    input.discard_whitespace();
    let start = input.mark();
    let component = numeric
        .collect(input)
        .map_err(|e| media_numeric_error(source, input, numeric, e))?;
    let can_ratio = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(n)) => !negative_literal(n),
        CssComponentValueRef::Token(
            CssValueTokenRef::Dimension { .. } | CssValueTokenRef::Ident(_),
        ) => false,
        CssComponentValueRef::Function(_) => {
            let result_type = numeric
                .numeric
                .component_result_type(
                    component.as_ref(),
                    CalculationRoot::NamedDimensionOrNumber,
                    numeric.limits,
                )
                .map_err(|e| media_numeric_error(source, input, numeric, e))?;
            result_type == CssCalculationType::Number
        }
        _ => {
            return Err(numeric.invalid(start.site()));
        }
    };
    if can_ratio && input.attempt(|p| p.consume_delim('/')).is_ok() {
        let denominator = parse_media_numeric(source, input, numeric, CalculationRoot::Number)
            .map(CssCalculation::<CssNumber>::from_expression)?;
        if media_literal_number(denominator.components()).is_some_and(negative_literal) {
            return Err(numeric.invalid(start.site()));
        }
    }
    numeric.between(input, start)
}

fn parse_media_feature_query<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssMediaFeatureQuery> {
    let initial = input.mark();
    let first_name = input.attempt(Stream::consume_ident).ok();
    let name = if let Some(name) = first_name.and_then(MediaFeatureName::parse) {
        if input.only_whitespace_remains() {
            return name
                .boolean_kind()
                .map(CssMediaFeatureQuery::Boolean)
                .ok_or_else(|| numeric.invalid(input.site()));
        }
        name
    } else {
        // The first operand precedes the feature name. Consume its component shape
        // only to discover the named domain; the domain parser reads it after reset.
        input.restore(initial);
        let _ = numeric
            .collect(input)
            .map_err(|error| media_numeric_error(source, input, numeric, error))?;
        if input.attempt(|p| p.consume_delim('/')).is_ok() {
            let _ = numeric
                .collect(input)
                .map_err(|error| media_numeric_error(source, input, numeric, error))?;
        }
        parse_media_comparison(input, numeric)?;
        let keyword_site = input.site();
        let ident = from_here(input, Stream::consume_ident)?;
        MediaFeatureName::parse(ident).ok_or_else(|| numeric.unexpected(keyword_site))?
    };
    input.restore(initial);
    match name.id.family() {
        MediaValueFamily::Length => {
            let range = parse_media_range(input, numeric, name, |p| {
                parse_media_numeric(source, p, numeric, CalculationRoot::Length)
                    .map(CssCalculation::<CssLength>::from_expression)
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
            let range = parse_media_range(input, numeric, name, |p| {
                parse_media_numeric(source, p, numeric, CalculationRoot::Integer)
                    .map(CssCalculation::<CssInteger>::from_expression)
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
            let range = parse_media_range(input, numeric, name, |p| {
                parse_media_ratio(source, p, numeric)
            })?;
            Ok(match name.id {
                CssMediaFeatureKind::AspectRatio => CssMediaFeatureQuery::AspectRatio(range),
                CssMediaFeatureKind::DeviceAspectRatio => {
                    CssMediaFeatureQuery::DeviceAspectRatio(range)
                }
                _ => unreachable!("ratio feature catalog"),
            })
        }
        MediaValueFamily::Resolution => parse_media_range(input, numeric, name, |p| {
            let initial = p.mark();
            if p.attempt(|p| p.consume_keyword("infinite")).is_ok() {
                p.restore(initial);
                numeric
                    .collect(p)
                    .map(|component| CssMediaResolution::infinite(component.into_owned()))
                    .map_err(|error| media_numeric_error(source, p, numeric, error))
            } else {
                parse_media_numeric(source, p, numeric, CalculationRoot::Resolution)
                    .map(CssCalculation::<CssResolution>::from_expression)
                    .map(CssMediaResolution::numeric)
            }
        })
        .map(CssMediaFeatureQuery::Resolution),
        MediaValueFamily::Discrete => {
            from_here(input, Stream::consume_ident)?;
            from_here(input, Stream::consume_colon)?;
            parse_media_discrete(source, input, numeric, name.id)
        }
    }
}

fn parse_media_comparison<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssQueryComparison> {
    let at = input.site();
    let token = input.consume_after_whitespace()?;
    let symbol = match token {
        Token::Delim(v @ ('<' | '>' | '=')) => v,
        _ => return Err(numeric.invalid(at)),
    };
    if symbol == '=' {
        return Ok(CssQueryComparison::Equal);
    }
    let after = input.mark();
    let inclusive = match input.consume_token() {
        Ok(Token::Delim('=')) => true,
        _ => {
            input.restore(after);
            false
        }
    };
    Ok(query_comparison(symbol, inclusive))
}

fn parse_media_range<'i, T>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
    name: MediaFeatureName,
    mut value: impl FnMut(&mut Stream<'i>) -> Value<'i, T>,
) -> Value<'i, CssMediaRange<T>> {
    let initial = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident)
        && MediaFeatureName::parse(ident).is_some_and(|actual| actual == name)
    {
        if input.attempt(Stream::consume_colon).is_ok() {
            let value = value(input)?;
            return Ok(CssMediaRange::new(query_plain_range(value, name.prefix)));
        }
        if name.prefix.is_some() {
            return Err(numeric.invalid(input.site()));
        }
        let comparison = parse_media_comparison(input, numeric)?;
        return value(input)
            .map(|value| CssMediaRange::new(MediaRangeState::FeatureFirst { comparison, value }));
    }
    input.restore(initial);
    if name.prefix.is_some() {
        return Err(numeric.invalid(input.site()));
    }
    let left = value(input)?;
    let first = parse_media_comparison(input, numeric)?;
    from_here(input, |p| p.consume_keyword(name.id.as_str()))?;
    if input.only_whitespace_remains() {
        return Ok(CssMediaRange::new(MediaRangeState::ValueFirst {
            value: left,
            comparison: first,
        }));
    }
    let second = parse_media_comparison(input, numeric)?;
    let right = value(input)?;
    let state = query_range_chain(left, first, right, second)
        .ok_or_else(|| numeric.invalid(input.site()))?;
    Ok(CssMediaRange::new(state))
}

pub(super) fn media_numeric_error<'i>(
    source: &str,
    input: &Stream<'i>,
    numeric: &MediaInput<'_>,
    error: crate::numeric::CssNumericConstructionError,
) -> ValueError<'i> {
    let at = input.site();
    if let Some(component) = error.component_error() {
        // No component at a grammar boundary is an absent operand, not a bad
        // authored token. Other component failures remain terminal and typed.
        let absent = component.kind
            == crate::component::values::CssComponentValueErrorKind::InvalidToken
            && matches!(component.origin(), CssValueOrigin::Parsed(origin) if origin.span().start()==origin.span().end());
        if !absent {
            return SyntaxError::custom(
                at,
                invalid_component_value(numeric.location(at), component.clone()),
            );
        }
    }
    let _ = source;
    SyntaxError::custom(
        at,
        unexpected_at(numeric.error_location(&error, numeric.location(at), numeric.offset(at))),
    )
}
fn parse_media_numeric<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
    root: CalculationRoot,
) -> Value<'i, CssCalculationExpression> {
    let component = numeric
        .collect(input)
        .map_err(|error| media_numeric_error(source, input, numeric, error))?;
    let values = crate::numeric::expression::CalculationComponents::single(component)
        .map_err(|_| numeric.invalid(input.site()))?;
    numeric
        .admit(values, root)
        .map_err(|error| media_numeric_error(source, input, numeric, error))
}
fn parse_media_ratio<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssMediaRatio> {
    let operand = |input: &mut Stream<'i>| {
        let value = CssCalculation::<CssNumber>::from_expression(parse_media_numeric(
            source,
            input,
            numeric,
            CalculationRoot::Number,
        )?);
        if media_literal_number(value.components()).is_some_and(negative_literal) {
            return Err(numeric.unexpected(input.site()));
        }
        Ok(value)
    };
    let numerator = operand(input)?;
    let denominator_is_omitted = input.attempt(|p| p.consume_delim('/')).is_err();
    let denominator = if denominator_is_omitted {
        CssCalculation::<CssNumber>::try_from_components(
            crate::component::CssComponentValues::try_new(
                vec![
                    crate::component::CssComponentValue::try_number("1")
                        .expect("default denominator"),
                ],
                Default::default(),
            )
            .expect("single default denominator"),
            Default::default(),
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

fn parse_media_grid<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssMediaGrid> {
    let calculation = CssCalculation::<CssInteger>::from_expression(parse_media_numeric(
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
            return Err(numeric.unexpected(input.site()));
        })
    } else {
        None
    };
    Ok(CssMediaGrid::new(calculation, literal))
}

fn parse_media_discrete<'i>(
    source: &str,
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
    id: CssMediaFeatureKind,
) -> Value<'i, CssMediaFeatureQuery> {
    match id {
        CssMediaFeatureKind::Orientation => {
            parse_orientation(input, numeric).map(CssMediaFeatureQuery::Orientation)
        }
        CssMediaFeatureKind::Scan => {
            parse_scan_mode(input, numeric).map(CssMediaFeatureQuery::Scan)
        }
        CssMediaFeatureKind::PrefersColorScheme => parse_color_scheme_preference(input, numeric)
            .map(CssMediaFeatureQuery::PrefersColorScheme),
        CssMediaFeatureKind::PrefersReducedMotion => {
            parse_reduced_motion_preference(input, numeric)
                .map(CssMediaFeatureQuery::PrefersReducedMotion)
        }
        CssMediaFeatureKind::PrefersReducedTransparency => {
            parse_reduced_transparency_preference(input, numeric)
                .map(CssMediaFeatureQuery::PrefersReducedTransparency)
        }
        CssMediaFeatureKind::PrefersContrast => {
            parse_contrast_preference(input, numeric).map(CssMediaFeatureQuery::PrefersContrast)
        }
        CssMediaFeatureKind::ForcedColors => {
            parse_forced_colors_mode(input, numeric).map(CssMediaFeatureQuery::ForcedColors)
        }
        CssMediaFeatureKind::Hover => {
            parse_hover_capability(input, numeric).map(CssMediaFeatureQuery::Hover)
        }
        CssMediaFeatureKind::AnyHover => {
            parse_hover_capability(input, numeric).map(CssMediaFeatureQuery::AnyHover)
        }
        CssMediaFeatureKind::Pointer => {
            parse_pointer_capability(input, numeric).map(CssMediaFeatureQuery::Pointer)
        }
        CssMediaFeatureKind::AnyPointer => {
            parse_pointer_capability(input, numeric).map(CssMediaFeatureQuery::AnyPointer)
        }
        CssMediaFeatureKind::DisplayMode => {
            parse_display_mode(input, numeric).map(CssMediaFeatureQuery::DisplayMode)
        }
        CssMediaFeatureKind::Grid => {
            parse_media_grid(source, input, numeric).map(CssMediaFeatureQuery::Grid)
        }
        CssMediaFeatureKind::Update => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaUpdate::None),
                "slow" => Some(CssMediaUpdate::Slow),
                "fast" => Some(CssMediaUpdate::Fast),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::Update),
        CssMediaFeatureKind::OverflowBlock => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaOverflowBlock::None),
                "scroll" => Some(CssMediaOverflowBlock::Scroll),
                "paged" => Some(CssMediaOverflowBlock::Paged),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::OverflowBlock),
        CssMediaFeatureKind::OverflowInline => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaOverflowInline::None),
                "scroll" => Some(CssMediaOverflowInline::Scroll),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::OverflowInline),
        CssMediaFeatureKind::ColorGamut => {
            parse_discrete_ident(input, numeric, |ident| {
                match ident.to_ascii_lowercase().as_str() {
                    "srgb" => Some(CssMediaColorGamut::Srgb),
                    "p3" => Some(CssMediaColorGamut::P3),
                    "rec2020" => Some(CssMediaColorGamut::Rec2020),
                    _ => None,
                }
            })
            .map(CssMediaFeatureQuery::ColorGamut)
        }
        CssMediaFeatureKind::VideoColorGamut => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "srgb" => Some(CssMediaColorGamut::Srgb),
                "p3" => Some(CssMediaColorGamut::P3),
                "rec2020" => Some(CssMediaColorGamut::Rec2020),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::VideoColorGamut),
        CssMediaFeatureKind::DynamicRange => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "standard" => Some(CssMediaDynamicRange::Standard),
                "high" => Some(CssMediaDynamicRange::High),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::DynamicRange),
        CssMediaFeatureKind::VideoDynamicRange => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "standard" => Some(CssMediaDynamicRange::Standard),
                "high" => Some(CssMediaDynamicRange::High),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::VideoDynamicRange),
        CssMediaFeatureKind::EnvironmentBlending => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "opaque" => Some(CssMediaEnvironmentBlending::Opaque),
                "additive" => Some(CssMediaEnvironmentBlending::Additive),
                "subtractive" => Some(CssMediaEnvironmentBlending::Subtractive),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::EnvironmentBlending),
        CssMediaFeatureKind::InvertedColors => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "none" => Some(CssMediaInvertedColors::None),
                "inverted" => Some(CssMediaInvertedColors::Inverted),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::InvertedColors),
        CssMediaFeatureKind::NavControls => {
            parse_discrete_ident(input, numeric, |ident| {
                match ident.to_ascii_lowercase().as_str() {
                    "none" => Some(CssMediaNavigationControls::None),
                    "back" => Some(CssMediaNavigationControls::Back),
                    _ => None,
                }
            })
            .map(CssMediaFeatureQuery::NavControls)
        }
        CssMediaFeatureKind::Scripting => parse_discrete_ident(input, numeric, |ident| match ident
            .to_ascii_lowercase()
            .as_str()
        {
            "none" => Some(CssMediaScripting::None),
            "initial-only" => Some(CssMediaScripting::InitialOnly),
            "enabled" => Some(CssMediaScripting::Enabled),
            _ => None,
        })
        .map(CssMediaFeatureQuery::Scripting),
        CssMediaFeatureKind::PrefersReducedData => parse_discrete_ident(input, numeric, |ident| {
            match ident.to_ascii_lowercase().as_str() {
                "no-preference" => Some(CssMediaReducedDataPreference::NoPreference),
                "reduce" => Some(CssMediaReducedDataPreference::Reduce),
                _ => None,
            }
        })
        .map(CssMediaFeatureQuery::PrefersReducedData),
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

fn parse_orientation<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssOrientation> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "portrait" => Some(CssOrientation::Portrait),
            "landscape" => Some(CssOrientation::Landscape),
            _ => None,
        }
    })
}

fn parse_scan_mode<'i>(input: &mut Stream<'i>, numeric: &MediaInput<'_>) -> Value<'i, CssScanMode> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "progressive" => Some(CssScanMode::Progressive),
            "interlace" => Some(CssScanMode::Interlace),
            _ => None,
        }
    })
}

fn parse_color_scheme_preference<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssColorSchemePreference> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "light" => Some(CssColorSchemePreference::Light),
            "dark" => Some(CssColorSchemePreference::Dark),
            _ => None,
        }
    })
}

fn parse_reduced_motion_preference<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssReducedMotionPreference> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "reduce" => Some(CssReducedMotionPreference::Reduce),
            "no-preference" => Some(CssReducedMotionPreference::NoPreference),
            _ => None,
        }
    })
}

fn parse_reduced_transparency_preference<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssReducedTransparencyPreference> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "reduce" => Some(CssReducedTransparencyPreference::Reduce),
            "no-preference" => Some(CssReducedTransparencyPreference::NoPreference),
            _ => None,
        }
    })
}

fn parse_contrast_preference<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssContrastPreference> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "no-preference" => Some(CssContrastPreference::NoPreference),
            "more" => Some(CssContrastPreference::More),
            "less" => Some(CssContrastPreference::Less),
            "custom" => Some(CssContrastPreference::Custom),
            _ => None,
        }
    })
}

fn parse_forced_colors_mode<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssForcedColorsMode> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "none" => Some(CssForcedColorsMode::None),
            "active" => Some(CssForcedColorsMode::Active),
            _ => None,
        }
    })
}

fn parse_hover_capability<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssHoverCapability> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "none" => Some(CssHoverCapability::None),
            "hover" => Some(CssHoverCapability::Hover),
            _ => None,
        }
    })
}

fn parse_pointer_capability<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssPointerCapability> {
    parse_discrete_ident(input, numeric, |ident| {
        match_ignore_ascii_case! { ident,
            "none" => Some(CssPointerCapability::None),
            "coarse" => Some(CssPointerCapability::Coarse),
            "fine" => Some(CssPointerCapability::Fine),
            _ => None,
        }
    })
}

fn parse_display_mode<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, CssDisplayMode> {
    parse_discrete_ident(input, numeric, |ident| {
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

fn parse_discrete_ident<'i, T>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Value<'i, T> {
    let at = input.site();
    let ident = from_here(input, Stream::consume_ident)?;
    parse(ident).ok_or_else(|| numeric.unexpected(at))
}

fn generic_comparison<'i>(
    input: &mut Stream<'i>,
    numeric: &MediaInput<'_>,
) -> Value<'i, (CssQueryComparison, Vec<CssValueOrigin>)> {
    input.discard_whitespace();
    let start = input.mark();
    let comparison = parse_media_comparison(input, numeric)?;
    let components = numeric.between(input, start)?;
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
