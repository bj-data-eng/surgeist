// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.9.0 shape/mod.rs. Original atoms and owned resources
// replace scalar-source/compressed carriers; actual provider status gates rows.

mod cache;
#[cfg(test)]
mod tests;

use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range as IndexRange,
};

use fontique::{Attributes, Query, QueryFamily, QueryFont, QueryStatus};
use icu_properties::props::Script;
use skrifa::raw::TableProvider;

use super::{
    LineSelection, PreparedData, ShapingInput,
    analysis::{
        AnalysisDataSources,
        cluster::{Char, CharCluster, Status},
    },
    convert,
    layout::data::{
        GlyphData, PreparedCluster, PreparedRun, RunMetrics, ShapedData, ShapedItem, Whitespace,
    },
    lru_cache::LruCache,
    resolve::ResolvedStyle,
};
use crate::{
    Error, ErrorCode, ErrorDetail, FontData, FontSynthesis, InlineBoxKind, LineHeight, Range,
    Result, TextNumericError, TextNumericPhase,
};

pub(crate) struct ShapeContext {
    data: LruCache<cache::ResourceKey, harfrust::ShaperData>,
    instances: LruCache<cache::InstanceKey, harfrust::ShaperInstance>,
    plans: LruCache<cache::PlanKey, harfrust::ShapePlan>,
    buffer: Option<harfrust::UnicodeBuffer>,
    cluster: CharCluster,
}

impl Default for ShapeContext {
    fn default() -> Self {
        Self {
            data: LruCache::new(16),
            instances: LruCache::new(16),
            plans: LruCache::new(16),
            buffer: Some(harfrust::UnicodeBuffer::new()),
            cluster: CharCluster::default(),
        }
    }
}

#[derive(Clone)]
struct SelectedFont {
    font: QueryFont,
    attrs: Attributes,
}

impl PartialEq for SelectedFont {
    fn eq(&self, other: &Self) -> bool {
        let (left, _) = self.font.blob.clone().into_raw_parts();
        let (right, _) = other.font.blob.clone().into_raw_parts();
        std::sync::Arc::ptr_eq(&left, &right)
            && self.font.index == other.font.index
            && self.font.synthesis == other.font.synthesis
            && self.attrs == other.attrs
    }
}

pub(crate) fn shape(
    mut query: Query<'_>,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
) -> Result<ShapedData> {
    let mut output = ShapedData::default();
    let sources = AnalysisDataSources;
    let barriers: BTreeSet<_> = input
        .boxes
        .iter()
        .filter(|item| item.kind() == InlineBoxKind::InFlow)
        .map(|item| item.index())
        .collect();
    let count = input.projection.atoms.len();
    // Actual box-only occurrences provide their own size; they do not require
    // synthetic text or an invented font-metric row.
    if count == 0 && input.boxes.is_empty() {
        let style = &input.styles[0].style;
        configure_query(&mut query, style, Script::Latin, false, &sources);
        let mut selected = None;
        query.matches_with(|font| {
            selected = Some(SelectedFont {
                font: font.clone(),
                attrs: attributes(style),
            });
            QueryStatus::Stop
        });
        let selected = selected.ok_or_else(no_font)?;
        append_selected(
            &input,
            SelectedSegment {
                atoms: 0..0,
                style: 0,
                script: Script::Latin,
                level: input.analysis.base_level,
                font: selected,
                context: 0..0,
            },
            scratch,
            &mut output,
        )?;
    } else if count != 0 {
        let mut item_start = 0;
        let mut script = input
            .analysis
            .infos
            .iter()
            .map(|(info, _)| info.script)
            .find(|script| real_script(*script))
            .unwrap_or(Script::Latin);
        while item_start < count {
            let style_index = input.analysis.infos[item_start].1;
            let style = &input.styles[style_index].style;
            let level = input.analysis.levels[item_start];
            if real_script(input.analysis.infos[item_start].0.script) {
                script = input.analysis.infos[item_start].0.script;
            }
            let mut item_end = item_start + 1;
            while item_end < count {
                let (info, next_style) = input.analysis.infos[item_end];
                let next_script = if real_script(info.script) {
                    info.script
                } else {
                    script
                };
                let anchor = input.projection.atoms[item_end].source.start;
                if input.analysis.levels[item_end] != level
                    || next_script != script
                    || !same_shaping_style(style, &input.styles[next_style].style)
                    || barriers.contains(&anchor)
                {
                    break;
                }
                item_end += 1;
            }
            let mut segment_start = item_start;
            let mut current: Option<SelectedFont> = None;
            let grapheme_start = input
                .analysis
                .graphemes
                .partition_point(|range| range.end <= item_start);
            let grapheme_end = input
                .analysis
                .graphemes
                .partition_point(|range| range.start < item_end);
            let context = bounded_context(&input, item_start..item_end, 0..count, &barriers);
            for grapheme in &input.analysis.graphemes[grapheme_start..grapheme_end] {
                let range = grapheme.start.max(item_start)..grapheme.end.min(item_end);
                fill_cluster(&input, range.clone(), &mut scratch.cluster);
                let selected =
                    select_font(&mut query, style, script, &mut scratch.cluster, &sources)
                        .ok_or_else(no_font)?;
                if current.as_ref().is_some_and(|font| font != &selected) {
                    let old = current.take().expect("the selected segment owns a font");
                    append_selected(
                        &input,
                        SelectedSegment {
                            atoms: segment_start..range.start,
                            style: style_index,
                            script,
                            level,
                            font: old,
                            context: context.clone(),
                        },
                        scratch,
                        &mut output,
                    )?;
                    segment_start = range.start;
                }
                current = Some(selected);
            }
            let selected = current.ok_or_else(no_font)?;
            append_selected(
                &input,
                SelectedSegment {
                    atoms: segment_start..item_end,
                    style: style_index,
                    script,
                    level,
                    font: selected,
                    context: context.clone(),
                },
                scratch,
                &mut output,
            )?;
            item_start = item_end;
        }
    }
    install_mixed_order(&mut output, input.boxes, &input.analysis.box_levels);
    Ok(output)
}

/// A selected line reshapes with its retained original faces and actual selected
/// boundary context. It never queries the current catalogue or redoes fallback.
pub(crate) fn finalize(
    prepared: &PreparedData,
    selection: &LineSelection,
    scratch: &mut ShapeContext,
) -> Result<ShapedData> {
    let input = ShapingInput {
        projection: &prepared.analyzed.projection,
        analysis: &prepared.analyzed.analysis,
        styles: &prepared.styles,
        boxes: prepared.analyzed.source.boxes(),
        font_catalogue: &prepared.font_catalogue,
    };
    let atoms = &prepared.analyzed.projection.atoms;
    let first = atoms.partition_point(|atom| atom.source.end <= selection.source.start);
    let end = atoms.partition_point(|atom| atom.source.start < selection.source.end);
    let mut output = ShapedData::default();
    if first == end {
        output.empty_run = prepared.shaped.empty_run.clone().or_else(|| {
            prepared
                .shaped
                .runs
                .iter()
                .find(|run| {
                    run.source.start <= selection.source.start
                        && run.source.end >= selection.source.end
                })
                .cloned()
        });
    } else {
        let barriers: BTreeSet<_> = input
            .boxes
            .iter()
            .filter(|item| item.kind() == InlineBoxKind::InFlow)
            .map(|item| item.index())
            .collect();
        let run_start = prepared
            .shaped
            .runs
            .partition_point(|run| run.atoms.end <= first);
        let run_end = prepared
            .shaped
            .runs
            .partition_point(|run| run.atoms.start < end);
        for run in &prepared.shaped.runs[run_start..run_end] {
            let range = run.atoms.start.max(first)..run.atoms.end.min(end);
            if range.is_empty() {
                continue;
            }
            let script = run.script;
            let context = bounded_context(&input, range.clone(), first..end, &barriers);
            append_retained_run(&input, range, run, script, context, scratch, &mut output)?;
        }
    }
    let mut items = Vec::new();
    for item in &prepared.shaped.items[selection.items.clone()] {
        match *item {
            ShapedItem::InlineBox { occurrence, level } => {
                items.push(ShapedItem::InlineBox { occurrence, level })
            }
            ShapedItem::TextCluster(_) => {}
        }
    }
    output.items.extend(items);
    install_text_order(&mut output, input.boxes);
    Ok(output)
}

fn attributes(style: &ResolvedStyle) -> Attributes {
    Attributes {
        width: style.font_width,
        weight: style.font_weight,
        style: style.font_style,
    }
}

fn same_shaping_style(a: &ResolvedStyle, b: &ResolvedStyle) -> bool {
    a.font_family == b.font_family
        && a.font_size == b.font_size
        && a.font_width == b.font_width
        && a.font_weight == b.font_weight
        && a.font_style == b.font_style
        && a.locale == b.locale
        && a.font_variations == b.font_variations
        && a.font_features == b.font_features
        && a.word_spacing == b.word_spacing
        && a.letter_spacing == b.letter_spacing
}

fn configure_query(
    query: &mut Query<'_>,
    style: &ResolvedStyle,
    script: Script,
    emoji: bool,
    sources: &AnalysisDataSources,
) {
    let families = style.font_family.iter().copied().map(QueryFamily::Id);
    if emoji {
        query.set_families(families.chain(std::iter::once(QueryFamily::Generic(
            fontique::GenericFamily::Emoji,
        ))));
    } else {
        query.set_families(families);
    }
    query.set_attributes(attributes(style));
    query.set_fallbacks(fontique::FallbackKey::new(
        convert::script_to_fontique(script, sources),
        style.locale.as_ref(),
    ));
}

fn select_font(
    query: &mut Query<'_>,
    style: &ResolvedStyle,
    script: Script,
    cluster: &mut CharCluster,
    sources: &AnalysisDataSources,
) -> Option<SelectedFont> {
    configure_query(query, style, script, cluster.is_emoji, sources);
    let mut selected = None;
    query.matches_with(|font| {
        let Some(charmap) = font.charmap() else {
            return QueryStatus::Continue;
        };
        let status = cluster.map(
            |ch| charmap.map(ch).is_some_and(|glyph| glyph != 0),
            sources,
        );
        if status != Status::Discard || selected.is_none() {
            selected = Some(SelectedFont {
                font: font.clone(),
                attrs: attributes(style),
            });
        }
        if status == Status::Complete {
            QueryStatus::Stop
        } else {
            QueryStatus::Continue
        }
    });
    selected
}

fn fill_cluster(input: &ShapingInput<'_>, range: IndexRange<usize>, cluster: &mut CharCluster) {
    cluster.clear();
    for index in range {
        let (info, _) = input.analysis.infos[index];
        let contributes = info.contributes_to_shaping();
        cluster.map_len += usize::from(contributes);
        cluster.is_emoji |= info.is_emoji_or_pictograph();
        cluster.force_normalize |= info.force_normalize();
        cluster.chars.push(Char {
            ch: input.projection.atoms[index].glyph,
            is_control_character: info.is_control(),
            contributes_to_shaping: contributes,
        });
    }
}

fn real_script(script: Script) -> bool {
    !matches!(script, Script::Common | Script::Unknown | Script::Inherited)
}

fn no_font() -> Error {
    Error::new(
        ErrorCode::NoUsableFontFace,
        "no admitted font face is available for this request",
    )
}

fn invalid_output(start: usize, end: usize) -> Error {
    Error::new(
        ErrorCode::LayoutFailed,
        "shaper returned an invalid source association",
    )
    .with_detail(ErrorDetail::ProviderSourceRange { start, end })
}

fn canonical(field: &'static str, value: f64) -> Result<f32> {
    let category = if !value.is_finite() {
        Some(TextNumericError::NonFinite)
    } else if value.abs() > f64::from(f32::MAX) {
        Some(TextNumericError::OutOfRange)
    } else if value != 0.0 && value as f32 == 0.0 {
        Some(TextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(category) = category {
        return Err(super::text_numeric_error(
            field,
            TextNumericPhase::Shaping,
            category,
            value,
        ));
    }
    Ok(value as f32)
}

fn logical_add(field: &'static str, base: f64, delta: f64) -> Result<f64> {
    let sum = base + delta;
    let category = if !sum.is_finite() {
        Some(TextNumericError::NonFinite)
    } else if delta != 0.0 && sum == base {
        Some(TextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(category) = category {
        return Err(super::text_numeric_error(
            field,
            TextNumericPhase::Shaping,
            category,
            sum,
        ));
    }
    Ok(sum)
}

fn logical_multiply(field: &'static str, left: f64, right: f64) -> Result<f64> {
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
            field,
            TextNumericPhase::Shaping,
            category,
            product,
        ));
    }
    Ok(product)
}

fn install_mixed_order(data: &mut ShapedData, boxes: &[crate::InlineBox], levels: &[u8]) {
    data.items
        .extend((0..boxes.len()).map(|occurrence| ShapedItem::InlineBox {
            occurrence,
            level: levels[occurrence],
        }));
    install_text_order(data, boxes);
}

fn install_text_order(data: &mut ShapedData, boxes: &[crate::InlineBox]) {
    data.items
        .extend((0..data.clusters.len()).map(ShapedItem::TextCluster));
    data.items.sort_by_key(|item| match *item {
        ShapedItem::TextCluster(cluster) => (data.clusters[cluster].source.start, 1, cluster),
        ShapedItem::InlineBox { occurrence, .. } => (boxes[occurrence].index(), 0, occurrence),
    });
}

struct RunRequest<'a, 'input> {
    input: &'a ShapingInput<'input>,
    atoms: IndexRange<usize>,
    style: usize,
    script: Script,
    level: u8,
    font: &'a FontData,
    attrs: Attributes,
    synthesis: FontSynthesis,
    suggested_variations: Vec<(harfrust::Tag, f32)>,
    retained_coords: Option<&'a [i16]>,
    context: IndexRange<usize>,
}

struct SelectedSegment {
    atoms: IndexRange<usize>,
    style: usize,
    script: Script,
    level: u8,
    font: SelectedFont,
    context: IndexRange<usize>,
}

fn append_selected(
    input: &ShapingInput<'_>,
    segment: SelectedSegment,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
) -> Result<()> {
    let selected = segment.font;
    let font = FontData::from_blob(
        selected.font.blob,
        selected.font.index,
        input.font_catalogue.clone(),
    );
    let synthesis = FontSynthesis::resolve(
        input.styles[segment.style].style.font_size,
        selected.font.synthesis.skew(),
        selected.font.synthesis.embolden(),
    )?;
    append_request(
        RunRequest {
            input,
            atoms: segment.atoms,
            style: segment.style,
            script: segment.script,
            level: segment.level,
            font: &font,
            attrs: selected.attrs,
            synthesis,
            suggested_variations: selected.font.synthesis.variation_settings().to_vec(),
            retained_coords: None,
            context: segment.context,
        },
        scratch,
        output,
    )
}

fn append_retained_run(
    input: &ShapingInput<'_>,
    atoms: IndexRange<usize>,
    run: &PreparedRun,
    script: Script,
    context: IndexRange<usize>,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
) -> Result<()> {
    append_request(
        RunRequest {
            input,
            atoms,
            style: run.style,
            script,
            level: run.level,
            font: &run.font,
            attrs: run.font_attrs,
            synthesis: run.synthesis,
            suggested_variations: Vec::new(),
            retained_coords: Some(&run.coords),
            context,
        },
        scratch,
        output,
    )
}

fn append_request(
    request: RunRequest<'_, '_>,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
) -> Result<()> {
    let style = &request.input.styles[request.style].style;
    let resource = cache::ResourceKey(request.font.clone());
    let font_ref = harfrust::FontRef::from_index(request.font.bytes(), request.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid selected font face"))?;
    let key = cache::InstanceKey {
        resource: resource.clone(),
        synthesis: request.synthesis,
        suggested_variations: request.suggested_variations.clone(),
        variations: if request.retained_coords.is_some() {
            Vec::new()
        } else {
            style.font_variations.clone()
        },
        coords: request.retained_coords.unwrap_or_default().to_vec(),
    };
    let instance = scratch.instances.entry(key.clone(), || {
        if let Some(coords) = request.retained_coords {
            harfrust::ShaperInstance::from_coords(
                &font_ref,
                coords
                    .iter()
                    .copied()
                    .map(harfrust::NormalizedCoord::from_bits),
            )
        } else {
            harfrust::ShaperInstance::from_variations(
                &font_ref,
                key.suggested_variations
                    .iter()
                    .map(|(tag, value)| harfrust::Variation {
                        tag: *tag,
                        value: *value,
                    })
                    .chain(key.variations.iter().map(|value| harfrust::Variation {
                        tag: harfrust::Tag::new(&value.tag.to_bytes()),
                        value: value.value,
                    })),
            )
        }
    });
    let data = scratch
        .data
        .entry(resource, || harfrust::ShaperData::new(&font_ref));
    let shaper = data
        .shaper(&font_ref)
        .instance(Some(instance))
        .point_size(Some(style.font_size))
        .build();
    let direction = if request.level & 1 == 0 {
        harfrust::Direction::LeftToRight
    } else {
        harfrust::Direction::RightToLeft
    };
    let script = convert::script_to_harfrust(convert::script_to_fontique(
        request.script,
        &AnalysisDataSources,
    ));
    let language = style
        .locale
        .as_ref()
        .map(|locale| {
            locale
                .language()
                .parse::<harfrust::Language>()
                .map_err(|_| {
                    Error::new(
                        ErrorCode::InvalidStyle,
                        "validated language is not representable by the shaper",
                    )
                })
        })
        .transpose()?;
    let features: Vec<_> = style
        .font_features
        .iter()
        .map(|feature| {
            harfrust::Feature::new(
                harfrust::Tag::new(&feature.tag.to_bytes()),
                u32::from(feature.value),
                ..,
            )
        })
        .collect();
    let mut plan_key = key;
    plan_key.coords = shaper
        .coords()
        .iter()
        .map(|coord| coord.to_bits())
        .collect();
    let plan = scratch.plans.entry(
        cache::PlanKey {
            instance: plan_key,
            direction,
            script,
            language: language.clone(),
            features: features.clone(),
            size_bits: style.font_size.to_bits(),
        },
        || {
            harfrust::ShapePlan::new(
                &shaper,
                direction,
                Some(script),
                language.as_ref(),
                &features,
            )
        },
    );
    let mut buffer = scratch
        .buffer
        .take()
        .expect("a shape invocation returns its reusable buffer");
    buffer.clear();
    let count = request.atoms.len();
    if count
        .checked_sub(1)
        .is_some_and(|last| u32::try_from(last).is_err())
    {
        scratch.buffer = Some(buffer);
        return Err(Error::new(
            ErrorCode::SourceLengthOverflow,
            "shaping input exceeds provider cluster indices",
        ));
    }
    buffer.reserve(count);
    for (local, atom) in request.input.projection.atoms[request.atoms.clone()]
        .iter()
        .enumerate()
    {
        buffer.add(
            atom.glyph,
            u32::try_from(local).expect("local provider index was admitted"),
        );
    }
    let bytes = atom_bytes(request.input, request.atoms.clone());
    let context = context_bytes(
        request.input,
        request.context.clone(),
        request.atoms.clone(),
    );
    buffer.set_pre_context(&request.input.projection.text[context.start..bytes.start]);
    buffer.set_post_context(&request.input.projection.text[bytes.end..context.end]);
    buffer.set_direction(direction);
    buffer.set_script(script);
    if let Some(language) = language {
        buffer.set_language(language);
    }
    let glyph_buffer = shaper.shape_with_plan(plan, buffer, &features);
    // This is the actual returned invocation's status. No glyph, position,
    // metric or immutable result row is observed before this fence.
    if let harfrust::ShapingStatus::Refused(reason) = glyph_buffer.shaping_status() {
        scratch.buffer = Some(glyph_buffer.clear());
        return Err(super::shaping_refused(reason));
    }
    let result = extract_run(&request, &glyph_buffer, shaper.coords(), output);
    scratch.buffer = Some(glyph_buffer.clear());
    result
}

fn atom_bytes(input: &ShapingInput<'_>, atoms: IndexRange<usize>) -> IndexRange<usize> {
    if atoms.is_empty() {
        return 0..0;
    }
    input.projection.atoms[atoms.start].glyph_bytes.start
        ..input.projection.atoms[atoms.end - 1].glyph_bytes.end
}

fn bounded_context(
    input: &ShapingInput<'_>,
    run: IndexRange<usize>,
    mut allowed: IndexRange<usize>,
    barriers: &BTreeSet<usize>,
) -> IndexRange<usize> {
    if run.is_empty() {
        return 0..0;
    }
    let start = input.projection.atoms[run.start].source.start;
    let end = input.projection.atoms[run.end - 1].source.end;
    if let Some(anchor) = barriers.range(..=start).next_back() {
        allowed.start = allowed.start.max(
            input
                .projection
                .atoms
                .partition_point(|atom| atom.source.start < *anchor),
        );
    }
    if let Some(anchor) = barriers.range(end..).next() {
        allowed.end = allowed.end.min(
            input
                .projection
                .atoms
                .partition_point(|atom| atom.source.start < *anchor),
        );
    }
    allowed
}

fn context_bytes(
    input: &ShapingInput<'_>,
    context: IndexRange<usize>,
    _run: IndexRange<usize>,
) -> IndexRange<usize> {
    atom_bytes(input, context)
}

fn extract_run(
    request: &RunRequest<'_, '_>,
    glyph_buffer: &harfrust::GlyphBuffer,
    coords: &[harfrust::NormalizedCoord],
    output: &mut ShapedData,
) -> Result<()> {
    let input = request.input;
    let style = &input.styles[request.style].style;
    let stored_coords: Vec<_> = coords.iter().map(|coord| coord.to_bits()).collect();
    let instance = crate::EffectiveFontInstance::qualify_synthesized(
        request.font.face(),
        style.font_size,
        &stored_coords,
        request.synthesis,
    )?;
    let metrics = instance.metrics(input.font_catalogue)?;
    let font_ref = skrifa::FontRef::from_index(request.font.bytes(), request.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid selected font face"))?;
    let raw_metrics = skrifa::metrics::Metrics::new(
        &font_ref,
        skrifa::prelude::Size::new(style.font_size),
        coords,
    );
    let ascent = metrics.ascent();
    let descent = -metrics.descent();
    let leading = metrics.leading();
    let line_height = match style.line_height {
        LineHeight::Absolute(value) => value,
        LineHeight::FontSizeRelative(value) => {
            logical_multiply("line height", value, f64::from(style.font_size))?
        }
        LineHeight::MetricsRelative(value) => {
            logical_multiply("line height", value, ascent + descent + leading)?
        }
    };
    let underline = raw_metrics.underline;
    let underline_top = underline.map_or(f64::from(metrics.units_per_em()) / 18.0, |value| {
        f64::from(value.offset)
    });
    let underline_thickness = underline.map_or(f64::from(metrics.units_per_em()) / 18.0, |value| {
        f64::from(value.thickness)
    });
    let run_metrics = RunMetrics {
        ascent,
        descent,
        line_height,
        underline_top,
        underline_thickness,
        strike_center: metrics.strikeout().center_offset(),
        strike_thickness: metrics.strikeout().thickness(),
        x_height: raw_metrics.x_height.map_or(0.0, f64::from),
        cap_height: raw_metrics.cap_height.map_or(0.0, f64::from),
    };
    for (field, value) in [
        ("ascent", ascent),
        ("descent", descent),
        ("leading", leading),
        ("line height", line_height),
        ("underline top", underline_top),
        ("underline thickness", underline_thickness),
        ("x height", run_metrics.x_height),
        ("cap height", run_metrics.cap_height),
    ] {
        if !value.is_finite() {
            return Err(super::text_numeric_error(
                field,
                TextNumericPhase::Shaping,
                TextNumericError::NonFinite,
                value,
            ));
        }
    }
    let source = if request.atoms.is_empty() {
        Range::new(0, 0)
    } else {
        Range::new(
            input.projection.atoms[request.atoms.start].source.start,
            input.projection.atoms[request.atoms.end - 1].source.end,
        )
    };
    let scale = f64::from(style.font_size) / f64::from(metrics.units_per_em());
    let run = PreparedRun {
        font: request.font.clone(),
        font_size: style.font_size,
        font_attrs: request.attrs,
        synthesis: request.synthesis,
        coords: stored_coords,
        style: request.style,
        level: request.level,
        script: request.script,
        source,
        atoms: request.atoms.clone(),
        metrics: run_metrics,
    };
    if request.atoms.is_empty() {
        output.empty_run = Some(run);
        return Ok(());
    }
    let infos = glyph_buffer.glyph_infos();
    let positions = glyph_buffer.glyph_positions();
    if infos.len() != positions.len() {
        return Err(Error::new(
            ErrorCode::LayoutFailed,
            "shaper output arrays have different lengths",
        )
        .with_detail(ErrorDetail::ProviderOutputLength {
            infos: infos.len(),
            positions: positions.len(),
        }));
    }
    let max_glyph = font_ref
        .maxp()
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "font lacks valid glyph bounds"))?
        .num_glyphs();
    let count = request.atoms.len();
    let mut groups = BTreeMap::new();
    let mut next = 0;
    while next < infos.len() {
        let cluster = infos[next].cluster as usize;
        if cluster >= count {
            return Err(invalid_output(cluster, count));
        }
        let first = next;
        next += 1;
        while next < infos.len() && infos[next].cluster as usize == cluster {
            next += 1;
        }
        if groups.insert(cluster, first..next).is_some() {
            return Err(invalid_output(cluster, cluster));
        }
    }
    let mut starts: Vec<_> = groups.keys().copied().collect();
    starts.push(0);
    starts.push(count);
    starts.sort_unstable();
    starts.dedup();
    let run_index = output.runs.len();
    // Each actual provider cluster is extracted in logical order; its glyphs
    // retain the provider's visual order. Component geometry remains distinct.
    for pair in starts.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let atom_start = request.atoms.start + start;
        let atom_end = request.atoms.start + end;
        let cluster_source = Range::new(
            input.projection.atoms[atom_start].source.start,
            input.projection.atoms[atom_end - 1].source.end,
        );
        let first_glyph = output.glyphs.len();
        let mut advance = 0.0_f64;
        let provider_glyphs = groups.get(&start).cloned().unwrap_or(0..0);
        for (info, position) in infos[provider_glyphs.clone()]
            .iter()
            .zip(&positions[provider_glyphs])
        {
            if info.glyph_id >= u32::from(max_glyph) {
                return Err(Error::new(
                    ErrorCode::LayoutFailed,
                    "shaper output glyph is outside the actual font",
                )
                .with_detail(ErrorDetail::ProviderGlyphBounds {
                    glyph_id: info.glyph_id,
                    num_glyphs: max_glyph,
                }));
            }
            let glyph_advance = canonical("glyph advance", f64::from(position.x_advance) * scale)?;
            advance += f64::from(glyph_advance);
            output.glyphs.push(GlyphData {
                id: info.glyph_id,
                x: canonical("glyph x offset", f64::from(position.x_offset) * scale)?,
                y: canonical("glyph y offset", -f64::from(position.y_offset) * scale)?,
                advance: glyph_advance,
                source: cluster_source,
                style: input.analysis.infos[atom_start].1,
            });
        }
        let glyphs = first_glyph..output.glyphs.len();
        let components = end - start;
        let component_advance = advance / components as f64;
        let mut distributed_base = 0.0;
        for atom in atom_start..atom_end {
            let original = &input.projection.atoms[atom];
            let (info, style_index) = input.analysis.infos[atom];
            let whitespace = whitespace(original.code_point);
            let base_advance = if atom + 1 == atom_end {
                advance - distributed_base
            } else {
                component_advance
            };
            distributed_base += base_advance;
            let style = &input.styles[style_index].style;
            let with_letter_spacing =
                logical_add("letter spacing", base_advance, style.letter_spacing)?;
            let cluster_advance = if whitespace.is_space_or_nbsp() {
                logical_add("word spacing", with_letter_spacing, style.word_spacing)?
            } else {
                with_letter_spacing
            };
            output.clusters.push(PreparedCluster {
                source: original.source,
                atoms: atom..atom + 1,
                glyphs: if atom == atom_start {
                    glyphs.clone()
                } else {
                    glyphs.end..glyphs.end
                },
                run: run_index,
                style: style_index,
                level: request.level,
                advance: cluster_advance,
                boundary: info.boundary,
                whitespace,
                is_whitespace: whitespace != Whitespace::None,
                ligature_start: !glyphs.is_empty() && components > 1 && atom == atom_start,
                ligature_component: !glyphs.is_empty() && atom != atom_start,
            });
        }
    }
    output.runs.push(run);
    Ok(())
}

fn whitespace(code_point: u32) -> Whitespace {
    match code_point {
        0x20 => Whitespace::Space,
        0xa0 => Whitespace::NoBreakSpace,
        0x09 => Whitespace::Tab,
        0x0a | 0x0d | 0x2028 | 0x2029 => Whitespace::Newline,
        _ => Whitespace::None,
    }
}
