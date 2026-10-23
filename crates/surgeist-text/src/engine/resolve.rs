// Font request resolution adapts the bounded family/setting mechanisms from
// Parley 0.9.0 src/resolve/mod.rs (Copyright 2021 the Parley Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT

use fontique::{Collection, FamilyId, FontStyle, FontWeight, FontWidth, GenericFamily, Language};
use parlance::{FontFamilyName, FontFeature, FontVariation};

use crate::{
    Brush, Decoration, Error, ErrorCode, LineHeight, OverflowWrap, Range, Result, Source,
    TextNumericError, TextNumericPhase, ValidatedStyle, Wrap,
};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedStyleRun {
    pub(crate) style: ResolvedStyle,
}

/// One complete effective style, resolved against the captured catalogue.
/// Authored span order has already selected this row; shaping does not replay it.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedStyle {
    pub(crate) font_family: Vec<FamilyId>,
    pub(crate) font_size: f32,
    pub(crate) font_width: FontWidth,
    pub(crate) font_style: FontStyle,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_variations: Vec<FontVariation>,
    pub(crate) font_features: Vec<FontFeature>,
    pub(crate) locale: Option<Language>,
    pub(crate) brush: Brush,
    pub(crate) underline: Decoration,
    pub(crate) strikethrough: Decoration,
    pub(crate) line_height: LineHeight,
    pub(crate) word_spacing: f64,
    pub(crate) letter_spacing: f64,
    pub(crate) overflow_wrap: OverflowWrap,
    pub(crate) wrap: Wrap,
}

#[derive(Clone, Debug)]
pub(crate) struct AnalyzedStyleRun {
    pub(crate) range: Range,
    pub(crate) style: ValidatedStyle,
}

pub(crate) fn analyze_styles(
    source: &Source,
    spans: &[crate::source_model::ValidatedSpan],
    default_style: &ValidatedStyle,
    paragraph: &crate::protocol::paragraph::ParagraphData,
) -> Vec<AnalyzedStyleRun> {
    let extent = paragraph.extent_in(source);
    let mut boundaries = vec![extent.start, extent.end];
    for span in spans {
        boundaries.push(span.range().start.clamp(extent.start, extent.end));
        boundaries.push(span.range().end.clamp(extent.start, extent.end));
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut result: Vec<AnalyzedStyleRun> = Vec::new();
    for endpoints in boundaries.windows(2) {
        for range in paragraph.intersections(Range::new(endpoints[0], endpoints[1])) {
            let style = spans
                .iter()
                .rev()
                .find(|span| span.range().start <= range.start && span.range().end >= range.end)
                .map_or(default_style, |span| span.style());
            // A transparent child hole contributes no parent style transition.
            // Equal actual styles therefore retain one shaping row across it;
            // paragraph coverage continues to own the original-piece authority.
            if paragraph.coverage.is_some()
                && let Some(last) = result.last_mut()
                && last.style.authored() == style.authored()
            {
                last.range.end = range.end;
            } else {
                result.push(AnalyzedStyleRun {
                    range,
                    style: style.clone(),
                });
            }
        }
    }
    if result.is_empty() {
        result.push(AnalyzedStyleRun {
            range: Range::new(extent.start, extent.start),
            style: default_style.clone(),
        });
    }
    result
}

pub(crate) fn resolve(
    collection: &mut Collection,
    styles: &[AnalyzedStyleRun],
    scale: f32,
) -> Result<Vec<ResolvedStyleRun>> {
    styles
        .iter()
        .map(|run| {
            Ok(ResolvedStyleRun {
                style: resolve_style(collection, &run.style, scale)?,
            })
        })
        .collect()
}

/// Resolves the actual captured base style even when no body span wins it.
/// The returned row joins the canonical preparation style table.
pub(crate) fn resolve_default(
    collection: &mut Collection,
    style: &ValidatedStyle,
    scale: f32,
) -> Result<ResolvedStyleRun> {
    Ok(ResolvedStyleRun {
        style: resolve_style(collection, style, scale)?,
    })
}

fn resolve_style(
    collection: &mut Collection,
    validated: &ValidatedStyle,
    scale: f32,
) -> Result<ResolvedStyle> {
    let style = validated.authored();
    let mut families = Vec::new();
    if let Some(request) = validated.parsed_font_family() {
        for family in request {
            match family {
                FontFamilyName::Named(name) => {
                    if let Some(id) = collection.family_id(name) {
                        families.push(id);
                    }
                }
                FontFamilyName::Generic(generic) => {
                    families.extend(collection.generic_families(*generic))
                }
            }
        }
    } else {
        families.extend(collection.generic_families(GenericFamily::SansSerif));
    }
    let mut features = parse_features(validated)?;
    let mut variations = parse_variations(validated)?;
    // Stable tag ordering preserves the actual last authored duplicate override.
    features.sort_by_key(|setting| setting.tag);
    variations.sort_by_key(|setting| setting.tag);
    Ok(ResolvedStyle {
        font_family: families,
        font_size: scaled(style.size, scale, true, "font size")?,
        font_width: style.font.width.into(),
        font_style: style.font.slant.into(),
        font_weight: style.font.weight.into(),
        font_variations: variations,
        font_features: features,
        locale: validated.font_locale(),
        brush: style.brush,
        underline: scaled_decoration(style.underline, scale)?,
        strikethrough: scaled_decoration(style.strikethrough, scale)?,
        line_height: scaled_line_height(style.line_height, scale)?,
        word_spacing: scaled_logical(style.word_spacing, scale, false, "word spacing")?,
        letter_spacing: scaled_logical(style.letter_spacing, scale, false, "letter spacing")?,
        overflow_wrap: style.overflow_wrap,
        wrap: style.wrap,
    })
}

fn parse_features(style: &ValidatedStyle) -> Result<Vec<FontFeature>> {
    style.parsed_font_features().map_or_else(
        || Ok(Vec::new()),
        |source| {
            FontFeature::parse_css_list(source)
                .map(|setting| setting.map_err(|_| setting_error()))
                .collect()
        },
    )
}

fn parse_variations(style: &ValidatedStyle) -> Result<Vec<FontVariation>> {
    style.parsed_font_variations().map_or_else(
        || Ok(Vec::new()),
        |source| {
            FontVariation::parse_css_list(source)
                .map(|setting| setting.map_err(|_| setting_error()))
                .collect()
        },
    )
}

fn setting_error() -> Error {
    Error::new(ErrorCode::InvalidStyle, "font setting is invalid")
}

fn scaled_decoration(decoration: Decoration, scale: f32) -> Result<Decoration> {
    use crate::{DecorationOffset, DecorationThickness};
    let offset = match decoration.offset() {
        DecorationOffset::Auto => DecorationOffset::Auto,
        DecorationOffset::Absolute(value) => DecorationOffset::try_absolute(scaled_logical(
            value.get(),
            scale,
            false,
            "decoration offset",
        )?)?,
    };
    let thickness =
        match decoration.thickness() {
            DecorationThickness::Auto => DecorationThickness::Auto,
            DecorationThickness::Absolute(value) => DecorationThickness::try_absolute(
                scaled_logical(value.get(), scale, true, "decoration thickness")?,
            )?,
        };
    Ok(if decoration.enabled() {
        Decoration::solid()
    } else {
        Decoration::none()
    }
    .with_offset(offset)
    .with_thickness(thickness)
    .with_brush(decoration.brush()))
}

fn scaled_line_height(value: LineHeight, scale: f32) -> Result<LineHeight> {
    Ok(match value {
        LineHeight::Absolute(value) => {
            LineHeight::Absolute(scaled_logical(value, scale, true, "line height")?)
        }
        other => other,
    })
}

fn scaled(value: f32, scale: f32, positive: bool, field: &'static str) -> Result<f32> {
    let result = value * scale;
    let category = if !result.is_finite() {
        Some(TextNumericError::NonFinite)
    } else if positive && result <= 0.0 {
        Some(TextNumericError::PositiveLost)
    } else if value != 0.0 && result == 0.0 {
        Some(TextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(category) = category {
        return Err(super::text_numeric_error(
            field,
            TextNumericPhase::Preparation,
            category,
            f64::from(value) * f64::from(scale),
        ));
    }
    Ok(result)
}

fn scaled_logical(value: f64, scale: f32, positive: bool, field: &'static str) -> Result<f64> {
    let result = value * f64::from(scale);
    let category = if !result.is_finite() {
        Some(TextNumericError::NonFinite)
    } else if positive && result <= 0.0 {
        Some(TextNumericError::PositiveLost)
    } else if value != 0.0 && result == 0.0 {
        Some(TextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(category) = category {
        return Err(super::text_numeric_error(
            field,
            TextNumericPhase::Preparation,
            category,
            result,
        ));
    }
    Ok(result)
}
