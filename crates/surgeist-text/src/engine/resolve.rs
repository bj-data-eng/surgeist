// Font request resolution adapts the bounded family/setting mechanisms from
// Parley 0.9.0 src/resolve/mod.rs (Copyright 2021 the Parley Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT

use fontique::{Collection, FamilyId, FontStyle, FontWeight, FontWidth, GenericFamily, Language};
use parlance::{FontFamilyName, FontFeature, FontVariation};

use crate::{
    Brush, Decoration, Error, ErrorCode, LineHeight, Range, Result, Source, TextNumericError,
    TextNumericPhase, ValidatedStyle,
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
}

/// Canonical font-only fields shared by a body style and glyph-free context.
/// This is not a paint/lexical style populated with placeholder values.
pub(crate) struct ResolvedFontStyle {
    pub(crate) font_family: Vec<FamilyId>,
    pub(crate) font_size: f32,
    pub(crate) font_width: FontWidth,
    pub(crate) font_style: FontStyle,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_variations: Vec<FontVariation>,
    pub(crate) locale: Option<Language>,
    pub(crate) line_height: LineHeight,
}

#[derive(Clone, Debug)]
pub(crate) struct AnalyzedStyleRun {
    pub(crate) owner: AnalyzedStyleOwner,
    pub(crate) style: ValidatedStyle,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AnalyzedStyleOwner {
    Source(Range),
    Control(usize),
    Scope(usize),
}
impl AnalyzedStyleRun {
    pub(crate) fn source_range(&self) -> Option<Range> {
        match self.owner {
            AnalyzedStyleOwner::Source(range) => Some(range),
            AnalyzedStyleOwner::Control(_) | AnalyzedStyleOwner::Scope(_) => None,
        }
    }
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
                if let AnalyzedStyleOwner::Source(source) = &mut last.owner {
                    source.end = range.end;
                }
            } else {
                result.push(AnalyzedStyleRun {
                    owner: AnalyzedStyleOwner::Source(range),
                    style: style.clone(),
                });
            }
        }
    }
    if result.is_empty() {
        result.push(AnalyzedStyleRun {
            owner: AnalyzedStyleOwner::Source(Range::new(extent.start, extent.start)),
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
    resolved_fields(
        style,
        validated.font_locale(),
        families,
        features,
        variations,
        scale,
    )
}

fn resolved_fields(
    style: &crate::Style,
    locale: Option<Language>,
    families: Vec<FamilyId>,
    features: Vec<FontFeature>,
    variations: Vec<FontVariation>,
    scale: f32,
) -> Result<ResolvedStyle> {
    // Keep ordinary body scaling/error order intact while sharing the actual
    // font-field owner with glyph-free receipts.
    let font_size = scaled(style.size, scale, true, "font size")?;
    let underline = scaled_decoration(style.underline, scale)?;
    let strikethrough = scaled_decoration(style.strikethrough, scale)?;
    let line_height = scaled_line_height(style.line_height, scale)?;
    let font = resolved_font_fields(style, locale, families, variations, font_size, line_height);
    Ok(ResolvedStyle {
        font_family: font.font_family,
        font_size: font.font_size,
        font_width: font.font_width,
        font_style: font.font_style,
        font_weight: font.font_weight,
        font_variations: font.font_variations,
        font_features: features,
        locale: font.locale,
        brush: style.brush,
        underline,
        strikethrough,
        line_height: font.line_height,
        word_spacing: scaled_logical(style.word_spacing, scale, false, "word spacing")?,
        letter_spacing: scaled_logical(style.letter_spacing, scale, false, "letter spacing")?,
    })
}

fn resolved_font_fields(
    style: &crate::Style,
    locale: Option<Language>,
    families: Vec<FamilyId>,
    variations: Vec<FontVariation>,
    font_size: f32,
    line_height: LineHeight,
) -> ResolvedFontStyle {
    ResolvedFontStyle {
        font_family: families,
        font_size,
        font_width: style.font.width.into(),
        font_style: style.font.slant.into(),
        font_weight: style.font.weight.into(),
        font_variations: variations,
        locale,
        line_height,
    }
}

/// The same actual used-height operation for body and glyph-free receipts. The
/// caller supplies its owning phase; no sibling-expanded line extent enters it.
pub(crate) fn font_line_height(
    size: f32,
    height: LineHeight,
    normal: f64,
    phase: TextNumericPhase,
) -> Result<f64> {
    let (left, right) = match height {
        LineHeight::Absolute(value) => return Ok(value),
        LineHeight::FontSizeRelative(value) => (value, f64::from(size)),
        LineHeight::MetricsRelative(value) => (value, normal),
    };
    let product = left * right;
    let category = if !product.is_finite() {
        Some(TextNumericError::NonFinite)
    } else if left != 0.0 && right != 0.0 && product == 0.0 {
        Some(if left > 0.0 && right > 0.0 {
            TextNumericError::PositiveLost
        } else {
            TextNumericError::NonZeroLost
        })
    } else {
        None
    };
    if let Some(category) = category {
        return Err(super::text_numeric_error(
            "line height",
            phase,
            category,
            product,
        ));
    }
    Ok(product)
}

/// Glyph-free captured styles borrow the original request. Only canonical
/// family ids/settings are materialized, under the feature transaction. The
/// ordinary body resolver above keeps its existing parser and matching policy.
pub(crate) fn resolve_scope(
    collection: &mut Collection,
    style: &crate::Style,
    scale: f32,
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
) -> Result<ResolvedFontStyle> {
    budget.work(3)?;
    budget.work(style.locale.as_ref().map_or(0, String::len))?;
    let locale = crate::style::scope_style_locale(style)?;
    let mut families = Vec::new();
    if style.font.family.is_empty() {
        for id in collection.generic_families(GenericFamily::SansSerif) {
            scope_push(budget, &mut families, id)?;
        }
    } else {
        for source in &style.font.family {
            budget.work(source.len())?;
            // FontFamilyName::parse borrows its named input, including quoted
            // names. This is the same first-entry rule as parse_font_families.
            let family = FontFamilyName::parse(source)
                .ok_or_else(|| Error::new(ErrorCode::InvalidStyle, "invalid scope font family"))?;
            match family {
                FontFamilyName::Named(name) => {
                    if let Some(id) = collection.family_id(&name) {
                        scope_push(budget, &mut families, id)?;
                    }
                }
                FontFamilyName::Generic(generic) => {
                    for id in collection.generic_families(generic) {
                        scope_push(budget, &mut families, id)?;
                    }
                }
            }
        }
    }
    let variations_source = scope_settings_source(&style.font.variations, budget)?;
    let mut variations: Vec<FontVariation> = Vec::new();
    if let Some(source) = &variations_source {
        budget.work(
            source
                .len()
                .checked_mul(2)
                .ok_or_else(scope_work_overflow)?,
        )?;
        let source = std::str::from_utf8(source).expect("joined UTF-8 source strings");
        if source.trim().is_empty() {
            return Err(setting_error());
        }
        for setting in FontVariation::parse_css_list(source) {
            let setting = setting.map_err(|_| setting_error())?;
            // Stable insertion preserves duplicate authored order without the
            // hidden scratch allocation of a stable slice sort.
            let at = crate::processing::contextual::alignment::operations::partition_point(
                &variations,
                Some(&mut *budget),
                |old| old.tag <= setting.tag,
            )?;
            budget.add_records(1)?;
            budget.reserve(&mut variations, 1)?;
            // Reserve meters relocation independently. Insertion then moves
            // exactly this suffix and writes its one new authored setting.
            budget.work(
                (variations.len() - at)
                    .checked_add(1)
                    .ok_or_else(scope_work_overflow)?,
            )?;
            variations.insert(at, setting);
        }
        if variations.is_empty() {
            return Err(setting_error());
        }
    }
    scope_drop_source(variations_source, budget)?;
    let font_size = scaled(style.size, scale, true, "font size")?;
    let line_height = scaled_line_height(style.line_height, scale)?;
    Ok(resolved_font_fields(
        style,
        locale,
        families,
        variations,
        font_size,
        line_height,
    ))
}

fn scope_work_overflow() -> Error {
    super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
        kind: crate::ScopeAlignmentLimitKind::StructuralWork,
    })
}

fn scope_push<T>(
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
    values: &mut Vec<T>,
    value: T,
) -> Result<()> {
    budget.work(1)?;
    budget.add_records(1)?;
    budget.reserve(values, 1)?;
    values.push(value);
    Ok(())
}

fn scope_settings_source(
    values: &[String],
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
) -> Result<Option<Vec<u8>>> {
    if values.is_empty() {
        return Ok(None);
    }
    budget.work(values.len())?;
    let kind = crate::ScopeAlignmentLimitKind::PayloadBytes;
    let overflow =
        || super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow { kind });
    let mut len = values
        .len()
        .checked_sub(1)
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(overflow)?;
    for value in values {
        len = len.checked_add(value.len()).ok_or_else(overflow)?;
    }
    budget.work(
        len.checked_add(values.len())
            .ok_or_else(scope_work_overflow)?,
    )?;
    let mut joined = Vec::new();
    budget.reserve(&mut joined, len)?;
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            joined.extend_from_slice(b", ");
        }
        joined.extend_from_slice(value.as_bytes());
    }
    Ok(Some(joined))
}

fn scope_drop_source(
    source: Option<Vec<u8>>,
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
) -> Result<()> {
    if let Some(source) = source {
        let bytes = source.capacity();
        drop(source);
        budget.release_bytes(bytes)?;
    }
    Ok(())
}

pub(crate) fn drop_scope_style(
    style: ResolvedFontStyle,
    budget: &mut crate::processing::contextual::alignment::ScopeAlignmentBudget,
) -> Result<()> {
    use super::scope_alignment::accounting::bytes;
    let mut total = 0usize;
    for size in [bytes(&style.font_family)?, bytes(&style.font_variations)?] {
        total = total.checked_add(size).ok_or_else(|| {
            super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
            })
        })?;
    }
    drop(style);
    budget.release_bytes(total)
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

/// Canonical source-style row; control rows never acquire source membership.
pub(crate) fn source_style(styles: &[AnalyzedStyleRun], at: usize) -> usize {
    // Analysis appends genuine control rows after the ordered source rows.
    // Search only source membership; neither an empty source nor end affinity
    // may accidentally borrow a control's unrelated style.
    let source_count = styles.partition_point(|row| row.source_range().is_some());
    styles[..source_count]
        .partition_point(|row| row.source_range().is_some_and(|range| range.end <= at))
        .min(source_count - 1)
}
