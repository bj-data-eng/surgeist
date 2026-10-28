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
    ShapingInput,
    analysis::{
        AnalysisDataSources,
        cluster::{Char, CharCluster, Status},
        is_hard_separator,
    },
    convert,
    layout::data::{
        ClusterContent, GlyphData, PreparedCluster, PreparedRun, RunMetrics, ShapedData,
        ShapedItem, Whitespace,
    },
    lru_cache::LruCache,
    resolve::ResolvedStyle,
};
use crate::{
    Error, ErrorCode, ErrorDetail, FontData, FontSynthesis, LineHeight, Range, Result,
    TextNumericError, TextNumericPhase,
};

pub(crate) struct ShapeContext {
    data: LruCache<cache::ResourceKey, surgeist_shaper::ShaperData>,
    instances: LruCache<cache::InstanceKey, surgeist_shaper::ShaperInstance>,
    plans: LruCache<cache::PlanKey, surgeist_shaper::ShapePlan>,
    buffer: Option<surgeist_shaper::UnicodeBuffer>,
    cluster: CharCluster,
}

#[cfg(test)]
impl ShapeContext {
    pub(crate) fn cache_hits(&self) -> [usize; 3] {
        [self.data.hits(), self.instances.hits(), self.plans.hits()]
    }
}

impl Default for ShapeContext {
    fn default() -> Self {
        Self {
            data: LruCache::new(16),
            instances: LruCache::new(16),
            plans: LruCache::new(16),
            buffer: Some(surgeist_shaper::UnicodeBuffer::new()),
            cluster: CharCluster::default(),
        }
    }
}

#[derive(Clone)]
struct SelectedFont {
    font: SelectedResource,
    attrs: Attributes,
}

#[derive(Clone)]
enum SelectedResource {
    Matched(QueryFont),
    Control(FontData),
}

/// Exact pre-mapping font-query input and its genuinely realized resource.
/// Source offsets are deliberately absent: mapping belongs to each projection.
#[derive(Clone, Debug)]
pub(crate) struct FontQueryWitness {
    chars: Vec<Char>,
    map_len: usize,
    emoji: bool,
    normalize: bool,
    pub(crate) run: PreparedRun,
}
impl FontQueryWitness {
    fn matches(
        &self,
        cluster: &CharCluster,
        style: &ResolvedStyle,
        script: Script,
        styles: &[super::resolve::ResolvedStyleRun],
    ) -> bool {
        self.chars == cluster.chars
            && self.map_len == cluster.map_len
            && self.emoji == cluster.is_emoji
            && self.normalize == cluster.force_normalize
            && self.run.script == script
            && same_shaping_style(style, &styles[self.run.style].style)
    }
}

/// Provider-owned Unicode work shared by every font run in this actual domain.
pub(crate) struct JoiningDomain {
    atoms: IndexRange<usize>,
    script: Script,
    level: u8,
    source: surgeist_shaper::JoiningSource,
}

/// The original-only operation retains its actual captured scalar assignments;
/// a selected substring does not invent another catalogue query prerequisite.
pub(crate) fn retain_original_fonts(
    prepared: &super::PreparedData,
    projection: &crate::projection::SourceProjection,
    originals: &[Option<usize>],
) -> Result<Vec<crate::protocol::prepared::RetainedAssignment>> {
    let mut result: Vec<crate::protocol::prepared::RetainedAssignment> = Vec::new();
    let mut prior_run = None;
    let mut prior_original = None;
    for (atom, original) in originals.iter().enumerate() {
        if prepared.options.tabs() == crate::TabPreparation::CssStops
            && projection.atoms[atom].code_point == 9
        {
            continue;
        }
        let original = original.ok_or_else(|| {
            Error::new(
                ErrorCode::LayoutFailed,
                "original assignment has no source scalar",
            )
        })?;
        let run = prepared
            .shaped
            .runs
            .partition_point(|r| r.atoms.end <= original);
        let captured = prepared
            .shaped
            .runs
            .get(run)
            .filter(|r| r.atoms.contains(&original))
            .ok_or_else(no_font)?;
        if prior_run == Some(run)
            && prior_original == original.checked_sub(1)
            && let Some(last) = result.last_mut()
        {
            last.atoms.end = atom + 1;
            last.run.atoms.end = atom + 1;
        } else {
            let mut captured = captured.clone();
            captured.atoms = atom..atom + 1;
            result.push(crate::protocol::prepared::RetainedAssignment {
                atoms: atom..atom + 1,
                run: captured,
            });
        }
        prior_run = Some(run);
        prior_original = Some(original);
    }
    Ok(result)
}

/// Replay only an exact genuinely executed matching input. The new projection,
/// source mapping and provider glyph output are never borrowed from a witness.
pub(crate) fn assign_fonts(
    prepared: &super::PreparedData,
    projection: &crate::projection::SourceProjection,
    analysis: &super::analysis::Analysis,
    scopes: &[crate::protocol::prepared::presentation::DirectionalScope],
) -> Result<Vec<crate::protocol::prepared::RetainedAssignment>> {
    let input = ShapingInput {
        projection,
        analysis,
        styles: &prepared.styles,
        boxes: prepared.analyzed.processed.source().boxes(),
        font_catalogue: &prepared.font_catalogue,
        paragraph: &prepared.analyzed.paragraph,
        css_tabs: prepared.options.tabs() == crate::TabPreparation::CssStops,
    };
    let mut cluster = CharCluster::default();
    let mut assignments: Vec<crate::protocol::prepared::RetainedAssignment> = Vec::new();
    let mut script = analysis
        .infos
        .iter()
        .map(|(i, _)| i.script)
        .find(|&s| real_script(s))
        .unwrap_or(Script::Latin);
    let mut first = 0;
    while first < projection.atoms.len() {
        if input.css_tabs && projection.atoms[first].code_point == 9 {
            first += 1;
            continue;
        }
        let style = analysis.infos[first].1;
        if real_script(analysis.infos[first].0.script) {
            script = analysis.infos[first].0.script;
        }
        let mut end = first + 1;
        while end < projection.atoms.len() {
            let info = analysis.infos[end];
            let next = if real_script(info.0.script) {
                info.0.script
            } else {
                script
            };
            if (input.css_tabs && projection.atoms[end].code_point == 9)
                || scopes[end] != scopes[first]
                || (analysis.levels[end] & 1) != (analysis.levels[first] & 1)
                || next != script
                || !same_shaping_style(
                    &prepared.styles[style].style,
                    &prepared.styles[info.1].style,
                )
                || projection.barrier_atoms.contains(&end)
            {
                break;
            }
            end += 1;
        }
        for atoms in analysis
            .graphemes
            .iter()
            .filter(|g| g.end > first && g.start < end)
            .flat_map(|g| font_spans(projection, g.start.max(first)..g.end.min(end)))
        {
            fill_cluster(&input, atoms.clone(), &mut cluster);
            let witness = prepared
                .shaped
                .font_queries
                .iter()
                .chain(
                    prepared
                        .indication_fonts
                        .iter()
                        .flat_map(|a| a.font_queries.iter()),
                )
                .find(|w| {
                    w.matches(
                        &cluster,
                        &prepared.styles[style].style,
                        script,
                        &prepared.styles,
                    )
                })
                .ok_or_else(|| {
                    crate::hyphenation::error(
                        crate::HyphenationError::UncapturedIndicationFontQuery,
                    )
                })?;
            let mut run = witness.run.clone();
            run.style = style;
            run.atoms = atoms.clone();
            run.level = match scopes[first] {
                crate::protocol::prepared::presentation::DirectionalScope::Outer(level) => level,
                crate::protocol::prepared::presentation::DirectionalScope::Local(_, level) => level,
            };
            if let Some(prior) = assignments.last_mut()
                && prior.atoms.end == atoms.start
                && !projection.barrier_atoms.contains(&atoms.start)
                && scopes[prior.atoms.start] == scopes[first]
                && prior.run.font == run.font
                && prior.run.coords == run.coords
                && prior.run.synthesis == run.synthesis
                && prior.run.script == script
                && same_shaping_style(
                    &prepared.styles[prior.run.style].style,
                    &prepared.styles[style].style,
                )
            {
                prior.atoms.end = atoms.end;
                prior.run.atoms.end = atoms.end;
            } else {
                assignments.push(crate::protocol::prepared::RetainedAssignment { atoms, run });
            }
        }
        first = end;
    }
    Ok(assignments)
}

/// Real fallback spans may divide a presentation grapheme without changing it.
fn font_spans(
    projection: &crate::projection::SourceProjection,
    range: IndexRange<usize>,
) -> impl Iterator<Item = IndexRange<usize>> + '_ {
    let mut first = range.start;
    std::iter::from_fn(move || {
        if first == range.end {
            return None;
        }
        let control = crate::control::is_visible(projection.atoms[first].code_point);
        let mut end = first + 1;
        while end < range.end
            && crate::control::is_visible(projection.atoms[end].code_point) == control
        {
            end += 1;
        }
        let span = first..end;
        first = end;
        Some(span)
    })
}

fn symbol_positions(
    projection: &crate::projection::SourceProjection,
    atoms: IndexRange<usize>,
) -> Result<Vec<u32>> {
    projection.atoms[atoms]
        .iter()
        .enumerate()
        .filter(|(_, a)| crate::control::is_visible(a.code_point))
        .map(|(i, _)| {
            u32::try_from(i)
                .map_err(|_| super::shaping_refused(surgeist_shaper::ShapingRefusal::GrowthLimit))
        })
        .collect()
}

fn provider_script(script: Script) -> surgeist_shaper::Script {
    convert::script_to_shaper(convert::script_to_fontique(script, &AnalysisDataSources))
}

fn provider_direction(level: u8) -> surgeist_shaper::Direction {
    if level & 1 == 0 {
        surgeist_shaper::Direction::LeftToRight
    } else {
        surgeist_shaper::Direction::RightToLeft
    }
}

fn joining_source_error(error: surgeist_shaper::JoiningSourceError) -> Error {
    super::shaping_refused(match error {
        surgeist_shaper::JoiningSourceError::Refused(reason) => reason,
        surgeist_shaper::JoiningSourceError::UnsupportedDirection => {
            surgeist_shaper::ShapingRefusal::JoiningProfile
        }
        surgeist_shaper::JoiningSourceError::OrdinarySymbol(error) => match error {
            surgeist_shaper::OrdinarySymbolInputError::GrowthLimit => {
                surgeist_shaper::ShapingRefusal::GrowthLimit
            }
            surgeist_shaper::OrdinarySymbolInputError::WorkLimit => {
                surgeist_shaper::ShapingRefusal::WorkLimit
            }
            _ => surgeist_shaper::ShapingRefusal::OrdinarySymbolInput,
        },
    })
}

impl PartialEq for SelectedFont {
    fn eq(&self, other: &Self) -> bool {
        self.attrs == other.attrs
            && match (&self.font, &other.font) {
                (SelectedResource::Matched(left), SelectedResource::Matched(right)) => {
                    let (left_blob, _) = left.blob.clone().into_raw_parts();
                    let (right_blob, _) = right.blob.clone().into_raw_parts();
                    std::sync::Arc::ptr_eq(&left_blob, &right_blob)
                        && left.index == right.index
                        && left.synthesis == right.synthesis
                }
                (SelectedResource::Control(left), SelectedResource::Control(right)) => {
                    left.face().is_same_face(&right.face())
                        && left.catalogue_identity() == right.catalogue_identity()
                }
                _ => false,
            }
    }
}

pub(crate) fn shape(
    catalogue: &mut crate::catalogue::Catalogue,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
) -> Result<ShapedData> {
    shape_impl(catalogue, input, scratch, false)
}

/// Actual matching/instance capture without a redundant glyph-shaping operation.
pub(crate) fn capture_fonts(
    catalogue: &mut crate::catalogue::Catalogue,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
) -> Result<Vec<FontQueryWitness>> {
    shape_impl(catalogue, input, scratch, true).map(|output| output.font_queries)
}

fn shape_impl(
    catalogue: &mut crate::catalogue::Catalogue,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
    capture_only: bool,
) -> Result<ShapedData> {
    let intrinsic = if input
        .projection
        .atoms
        .iter()
        .any(|a| crate::control::is_visible(a.code_point))
    {
        Some(catalogue.control_font()?)
    } else {
        None
    };
    let mut query = catalogue.query();
    let mut output = ShapedData::default();
    let mut queries = Vec::new();
    let sources = AnalysisDataSources;
    let barriers: BTreeSet<_> = input.projection.barrier_atoms.iter().copied().collect();
    let count = input.projection.atoms.len();
    // Empty/default-font metrics are an explicit preparation-profile operation.
    // Body shaping never infers that policy from source content or body style.
    if count != 0 {
        let mut item_start = 0;
        let mut script = input
            .analysis
            .infos
            .iter()
            .map(|(info, _)| info.script)
            .find(|script| real_script(*script))
            .unwrap_or(Script::Latin);
        while item_start < count {
            if input.css_tabs && input.projection.atoms[item_start].code_point == 9 {
                append_tab(&input, item_start, None, &mut output);
                item_start += 1;
                continue;
            }
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
                if (input.css_tabs && input.projection.atoms[item_end].code_point == 9)
                    || input.analysis.levels[item_end] != level
                    || next_script != script
                    || !same_shaping_style(style, &input.styles[next_style].style)
                    || barriers.contains(&item_end)
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
            let context = bounded_context(item_start..item_end, 0..count, &barriers);
            for range in input.analysis.graphemes[grapheme_start..grapheme_end]
                .iter()
                .flat_map(|grapheme| {
                    font_spans(
                        input.projection,
                        grapheme.start.max(item_start)..grapheme.end.min(item_end),
                    )
                })
            {
                fill_cluster(&input, range.clone(), &mut scratch.cluster);
                queries.push((
                    range.clone(),
                    scratch.cluster.chars.clone(),
                    scratch.cluster.map_len,
                    scratch.cluster.is_emoji,
                    scratch.cluster.force_normalize,
                ));
                let selected =
                    if crate::control::is_visible(input.projection.atoms[range.start].code_point) {
                        SelectedFont {
                            font: SelectedResource::Control(
                                intrinsic
                                    .as_ref()
                                    .expect("eligible controls admitted their font")
                                    .clone(),
                            ),
                            attrs: Attributes::default(),
                        }
                    } else {
                        select_font(&mut query, style, script, &mut scratch.cluster, &sources)
                            .ok_or_else(no_font)?
                    };
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
                        capture_only,
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
                capture_only,
            )?;
            item_start = item_end;
        }
    }
    for (atoms, chars, map_len, emoji, normalize) in queries {
        let run = output
            .runs
            .iter()
            .find(|run| run.atoms.start <= atoms.start && run.atoms.end >= atoms.end)
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "font query has no realized instance",
                )
            })?
            .clone();
        output.font_queries.push(FontQueryWitness {
            chars,
            map_len,
            emoji,
            normalize,
            run,
        });
    }
    if !capture_only {
        install_mixed_order(&mut output, &input);
    }
    Ok(output)
}

/// Queries and retains one actual zero-atom font strut under the original
/// preparation. The chosen style is an actual canonical style-table row.
pub(crate) fn default_metrics(
    mut query: Query<'_>,
    input: &ShapingInput<'_>,
    style_index: usize,
    scratch: &mut ShapeContext,
) -> Result<PreparedRun> {
    let style = &input.styles[style_index].style;
    configure_query(
        &mut query,
        style,
        Script::Latin,
        false,
        &AnalysisDataSources,
    );
    let mut selected = None;
    query.matches_with(|font| {
        selected = Some(SelectedFont {
            font: SelectedResource::Matched(font.clone()),
            attrs: attributes(style),
        });
        QueryStatus::Stop
    });
    let mut output = ShapedData::default();
    append_selected(
        input,
        SelectedSegment {
            atoms: 0..0,
            style: style_index,
            script: Script::Latin,
            level: input.analysis.base_level,
            font: selected.ok_or_else(no_font)?,
            context: 0..0,
        },
        scratch,
        &mut output,
        false,
    )?;
    Ok(output
        .empty_run
        .expect("the actual zero-atom operation produces its metrics row"))
}

/// A selected line reshapes with its retained original faces and actual selected
/// boundary context. It never queries the current catalogue or redoes fallback.
pub(crate) fn finalize(
    selected: crate::protocol::input::PresentationInput<'_>,
    scratch: &mut ShapeContext,
) -> Result<(
    ShapedData,
    std::sync::Arc<crate::protocol::prepared::SelectedMapping>,
)> {
    use crate::protocol::prepared::PresentationUnitKind;
    let prepared = &selected.prepared;
    let presentation = &selected.presentation;
    let input = ShapingInput {
        projection: &presentation.projection,
        analysis: &presentation.analysis,
        styles: &prepared.styles,
        boxes: prepared.analyzed.processed.source().boxes(),
        font_catalogue: &prepared.font_catalogue,
        paragraph: &prepared.analyzed.paragraph,
        css_tabs: prepared.options.tabs() == crate::TabPreparation::CssStops,
    };
    let mut output = ShapedData::default();
    let native = selected.metric_policy == crate::protocol::selection::LineMetricPolicy::Native;
    if native {
        // This is the actual base-style strut retained by Native+DefaultFont.
        // AllocatedContributions has no such row and never borrows body metrics.
        output.empty_run = prepared.shaped.empty_run.clone();
    }
    if input.projection.atoms.is_empty() {
        if !native {
            let boundary = &prepared.tables.boundaries[selected.start];
            output.empty_run = prepared.shaped.empty_run.clone().or_else(|| {
                prepared
                    .shaped
                    .runs
                    .iter()
                    .find(|run| run.atoms.contains(&boundary.atom))
                    .or_else(|| prepared.shaped.runs.last())
                    .cloned()
            });
        }
    } else {
        let computed = selected.computed;
        let barriers = &selected.boundaries.presentation;
        let mut selected_domains: Vec<JoiningDomain> = Vec::new();
        let mut original_domains: Vec<JoiningDomain> = Vec::new();
        for assignment in &presentation.assignments {
            let run = &assignment.run;
            let extent = bounded_context(
                assignment.atoms.clone(),
                0..input.projection.atoms.len(),
                barriers,
            );
            let mut context =
                RunContext::new(input.projection, assignment.atoms.clone(), extent.clone());
            if computed {
                let seed =
                    crate::protocol::boundary::original_seed(prepared, presentation, assignment)?;
                let original_extent = selected.boundaries.original_extent(prepared, &seed);
                let original_index = if let Some(index) =
                    original_domains.iter().position(|domain| {
                        domain.atoms == original_extent
                            && domain.script == run.script
                            && domain.level == run.level & 1
                    }) {
                    index
                } else {
                    let projection = &prepared.analyzed.projection;
                    let bytes = atom_bytes(projection, original_extent.clone());
                    let source = surgeist_shaper::JoiningSource::analyze(
                        &projection.text[bytes],
                        provider_script(run.script),
                        provider_direction(run.level),
                        &symbol_positions(projection, original_extent.clone())?,
                    )
                    .map_err(joining_source_error)?;
                    original_domains.push(JoiningDomain {
                        atoms: original_extent,
                        script: run.script,
                        level: run.level & 1,
                        source,
                    });
                    original_domains.len() - 1
                };
                let original = &original_domains[original_index];
                let index = if let Some(i) = selected_domains.iter().position(|d| {
                    d.atoms == extent && d.script == run.script && d.level == (run.level & 1)
                }) {
                    i
                } else {
                    let bytes = atom_bytes(input.projection, extent.clone());
                    let source = surgeist_shaper::JoiningSource::analyze(
                        &input.projection.text[bytes],
                        provider_script(run.script),
                        provider_direction(run.level),
                        &symbol_positions(input.projection, extent.clone())?,
                    )
                    .map_err(joining_source_error)?;
                    selected_domains.push(JoiningDomain {
                        atoms: extent,
                        script: run.script,
                        level: run.level & 1,
                        source,
                    });
                    selected_domains.len() - 1
                };
                let domain = &selected_domains[index];
                let original_map = presentation.originals[assignment.atoms.clone()]
                    .iter()
                    .map(|value| {
                        value
                            .map(|i| {
                                i.checked_sub(original.atoms.start)
                                    .filter(|_| original.atoms.contains(&i))
                                    .ok_or_else(|| {
                                        super::shaping_refused(
                                            surgeist_shaper::ShapingRefusal::JoiningCorrespondence,
                                        )
                                    })
                            })
                            .transpose()
                    })
                    .collect::<Result<Vec<_>>>()?;
                let presentation_map = assignment
                    .atoms
                    .clone()
                    .map(|i| i - domain.atoms.start)
                    .collect();
                context.joining = Some(JoiningRun {
                    original: &original.source,
                    original_map,
                    presentation: &domain.source,
                    presentation_map,
                });
            } else {
                let captured = presentation.originals[assignment.atoms.clone()]
                    .iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                if let (Some(&first), Some(&last)) = (captured.first(), captured.last()) {
                    context = RunContext::new(
                        &prepared.analyzed.projection,
                        first..last + 1,
                        bounded_context(
                            first..last + 1,
                            0..prepared.analyzed.projection.atoms.len(),
                            &selected.boundaries.original,
                        ),
                    );
                }
            }
            append_retained_run(
                &input,
                assignment.atoms.clone(),
                run,
                run.script,
                context,
                scratch,
                &mut output,
            )?;
        }
    }
    if input.css_tabs {
        for atom in 0..input.projection.atoms.len() {
            if input.projection.atoms[atom].code_point == 9 {
                let style = input.analysis.infos[atom].1;
                let font = if let Some(run) = prepared.tab_metrics[style]
                    .as_ref()
                    .and_then(|metric| metric.font.as_ref())
                {
                    let mut run = run.clone();
                    run.atoms = atom..atom + 1;
                    let index = output.runs.len();
                    output.runs.push(run);
                    Some(index)
                } else {
                    None
                };
                append_tab(&input, atom, font, &mut output);
            }
        }
        output.clusters.sort_by_key(|cluster| cluster.atoms.start);
    }
    // Each real atom receives one extracted cluster; ligature components lend
    // their actual shared glyph association rather than duplicating glyphs.
    let mut clusters = output.clusters.iter().enumerate().peekable();
    for (unit, value) in presentation.units.iter().enumerate() {
        match value.kind {
            PresentationUnitKind::Atomic { occurrence } => {
                if input.boxes[occurrence].kind() == crate::InlineBoxKind::InFlow {
                    output.items.push(ShapedItem::InlineBox {
                        occurrence,
                        level: value.level,
                    });
                }
            }
            _ if value.atoms.is_empty() => output.items.push(ShapedItem::Anchor { unit }),
            _ => {
                while let Some(&(index, cluster)) = clusters.peek() {
                    if cluster.atoms.start >= value.atoms.end {
                        break;
                    }
                    if cluster.atoms.start >= value.atoms.start {
                        output.items.push(ShapedItem::TextCluster(index));
                    }
                    clusters.next();
                }
            }
        }
    }
    Ok((output, std::sync::Arc::clone(&presentation.mapping)))
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
                font: SelectedResource::Matched(font.clone()),
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

fn install_mixed_order(data: &mut ShapedData, input: &ShapingInput<'_>) {
    data.items
        .extend((0..data.clusters.len()).map(ShapedItem::TextCluster));
    let mut positions = vec![(0, 0); input.boxes.len()];
    for (order, anchor) in input.paragraph.anchor_order.iter().enumerate() {
        if let crate::protocol::paragraph::SemanticAnchorItem::Atomic(occurrence) = anchor.item {
            positions[occurrence] = (
                super::analysis::anchor_atom(order, input.paragraph, input.boxes, input.projection),
                order,
            );
            data.items.push(ShapedItem::InlineBox {
                occurrence,
                level: input.analysis.box_levels[occurrence],
            });
        }
    }
    data.items.sort_by_key(|item| match *item {
        ShapedItem::TextCluster(cluster) => (data.clusters[cluster].atoms.start, 1, cluster),
        ShapedItem::InlineBox { occurrence, .. } => {
            (positions[occurrence].0, 0, positions[occurrence].1)
        }
        ShapedItem::Anchor { unit, .. } => (0, 0, unit),
    });
}

/// Borrowed pre/post strings from one genuine projection/context domain. The
/// shaper sees every bounded character and reports its own context limits.
struct RunContext<'a> {
    before: &'a str,
    after: &'a str,
    joining: Option<JoiningRun<'a>>,
}

struct JoiningRun<'a> {
    original: &'a surgeist_shaper::JoiningSource,
    original_map: Vec<Option<usize>>,
    presentation: &'a surgeist_shaper::JoiningSource,
    presentation_map: Vec<usize>,
}
impl<'a> RunContext<'a> {
    fn new(
        projection: &'a crate::projection::SourceProjection,
        atoms: IndexRange<usize>,
        context: IndexRange<usize>,
    ) -> Self {
        let bytes = atom_bytes(projection, atoms);
        let extent = atom_bytes(projection, context);
        Self {
            before: &projection.text[extent.start..bytes.start],
            after: &projection.text[bytes.end..extent.end],
            joining: None,
        }
    }
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
    suggested_variations: Vec<(surgeist_shaper::Tag, f32)>,
    retained_coords: Option<&'a [i16]>,
    context: RunContext<'a>,
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
    capture_only: bool,
) -> Result<()> {
    let selected = segment.font;
    let (font, synthesis, suggested_variations) = match selected.font {
        SelectedResource::Matched(selected) => {
            let synthesis = FontSynthesis::resolve(
                input.styles[segment.style].style.font_size,
                selected.synthesis.skew(),
                selected.synthesis.embolden(),
            )?;
            let variations = selected.synthesis.variation_settings().to_vec();
            (
                FontData::from_blob(selected.blob, selected.index, input.font_catalogue.clone()),
                synthesis,
                variations,
            )
        }
        SelectedResource::Control(font) => (font, FontSynthesis::default(), Vec::new()),
    };
    append_request(
        RunRequest {
            input,
            atoms: segment.atoms.clone(),
            style: segment.style,
            script: segment.script,
            level: segment.level,
            font: &font,
            attrs: selected.attrs,
            synthesis,
            suggested_variations,
            retained_coords: None,
            context: RunContext::new(input.projection, segment.atoms.clone(), segment.context),
        },
        scratch,
        output,
        capture_only,
    )
}

fn append_retained_run(
    input: &ShapingInput<'_>,
    atoms: IndexRange<usize>,
    run: &PreparedRun,
    script: Script,
    context: RunContext<'_>,
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
        false,
    )
}

fn append_request(
    mut request: RunRequest<'_, '_>,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
    capture_only: bool,
) -> Result<()> {
    let style = &request.input.styles[request.style].style;
    let resource = cache::ResourceKey(request.font.clone());
    let font_ref = surgeist_shaper::FontRef::from_index(request.font.bytes(), request.font.index())
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
            surgeist_shaper::ShaperInstance::from_coords(
                &font_ref,
                coords
                    .iter()
                    .copied()
                    .map(surgeist_shaper::NormalizedCoord::from_bits),
            )
        } else {
            surgeist_shaper::ShaperInstance::from_variations(
                &font_ref,
                key.suggested_variations
                    .iter()
                    .map(|(tag, value)| surgeist_shaper::Variation {
                        tag: *tag,
                        value: *value,
                    })
                    .chain(
                        key.variations
                            .iter()
                            .map(|value| surgeist_shaper::Variation {
                                tag: surgeist_shaper::Tag::new(&value.tag.to_bytes()),
                                value: value.value,
                            }),
                    ),
            )
        }
    });
    let data = scratch
        .data
        .entry(resource, || surgeist_shaper::ShaperData::new(&font_ref));
    let shaper = data
        .shaper(&font_ref)
        .instance(Some(instance))
        .point_size(Some(style.font_size))
        .build();
    if capture_only {
        let (run, _, _) = resolved_run(&request, shaper.coords())?;
        output.runs.push(run);
        return Ok(());
    }
    let direction = if request.level & 1 == 0 {
        surgeist_shaper::Direction::LeftToRight
    } else {
        surgeist_shaper::Direction::RightToLeft
    };
    let script = convert::script_to_shaper(convert::script_to_fontique(
        request.script,
        &AnalysisDataSources,
    ));
    let language = style
        .locale
        .as_ref()
        .map(|locale| {
            locale
                .language()
                .parse::<surgeist_shaper::Language>()
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
            surgeist_shaper::Feature::new(
                surgeist_shaper::Tag::new(&feature.tag.to_bytes()),
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
            surgeist_shaper::ShapePlan::new(
                &shaper,
                direction,
                Some(script),
                language.as_ref(),
                &features,
            )
        },
    );
    let ordinary_symbol_positions =
        symbol_positions(request.input.projection, request.atoms.clone())?;
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
    if let Err(error) = buffer.set_ordinary_symbol_positions(&ordinary_symbol_positions) {
        scratch.buffer = Some(buffer);
        return Err(super::shaping_refused(match error {
            surgeist_shaper::OrdinarySymbolInputError::GrowthLimit => {
                surgeist_shaper::ShapingRefusal::GrowthLimit
            }
            surgeist_shaper::OrdinarySymbolInputError::WorkLimit => {
                surgeist_shaper::ShapingRefusal::WorkLimit
            }
            _ => surgeist_shaper::ShapingRefusal::OrdinarySymbolInput,
        }));
    }
    let continuation_mark = request
        .input
        .analysis
        .graphemes
        .iter()
        .any(|g| g.start < request.atoms.start && request.atoms.start < g.end);
    buffer.set_flags(if continuation_mark {
        surgeist_shaper::BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE
    } else {
        surgeist_shaper::BufferFlags::empty()
    });
    buffer.set_pre_context(request.context.before);
    buffer.set_post_context(request.context.after);
    buffer.set_direction(direction);
    buffer.set_script(script);
    if let Some(language) = language {
        buffer.set_language(language);
    }
    // The provider retains its own validated compact map. Release these
    // explicitly bounded adapter inputs before normalizing or shaping.
    if let Some(joining) = request.context.joining.take()
        && let Err(error) = buffer.set_joining_sources(
            joining.original,
            &joining.original_map,
            joining.presentation,
            &joining.presentation_map,
        )
    {
        scratch.buffer = Some(buffer);
        return Err(super::shaping_refused(match error {
            surgeist_shaper::JoiningInputError::GrowthLimit => {
                surgeist_shaper::ShapingRefusal::GrowthLimit
            }
            _ => surgeist_shaper::ShapingRefusal::JoiningInput,
        }));
    }
    let glyph_buffer = shaper.shape_with_plan(plan, buffer, &features);
    // This is the actual returned invocation's status. No glyph, position,
    // metric or immutable result row is observed before this fence.
    if let surgeist_shaper::ShapingStatus::Refused(reason) = glyph_buffer.shaping_status() {
        scratch.buffer = Some(glyph_buffer.clear());
        return Err(super::shaping_refused(reason));
    }
    let result = extract_run(&request, &glyph_buffer, shaper.coords(), output);
    scratch.buffer = Some(glyph_buffer.clear());
    result
}

fn atom_bytes(
    projection: &crate::projection::SourceProjection,
    atoms: IndexRange<usize>,
) -> IndexRange<usize> {
    if atoms.is_empty() {
        return 0..0;
    }
    projection.atoms[atoms.start].glyph_bytes.start..projection.atoms[atoms.end - 1].glyph_bytes.end
}

fn bounded_context(
    run: IndexRange<usize>,
    mut allowed: IndexRange<usize>,
    barriers: &BTreeSet<usize>,
) -> IndexRange<usize> {
    if run.is_empty() {
        return 0..0;
    }
    if let Some(&anchor) = barriers.range(..=run.start).next_back() {
        allowed.start = allowed.start.max(anchor);
    }
    if let Some(&anchor) = barriers.range(run.end..).next() {
        allowed.end = allowed.end.min(anchor);
    }
    allowed
}

/// Metric attribution does not split otherwise compatible joint shaping.
fn resolved_line_height(
    style: &ResolvedStyle,
    ascent: f64,
    descent: f64,
    leading: f64,
) -> Result<f64> {
    match style.line_height {
        LineHeight::Absolute(value) => Ok(value),
        LineHeight::FontSizeRelative(value) => {
            logical_multiply("line height", value, f64::from(style.font_size))
        }
        LineHeight::MetricsRelative(value) => {
            logical_multiply("line height", value, ascent + descent + leading)
        }
    }
}

fn resolved_run(
    request: &RunRequest<'_, '_>,
    coords: &[surgeist_shaper::NormalizedCoord],
) -> Result<(PreparedRun, f64, f64)> {
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
    let line_height = resolved_line_height(style, ascent, descent, leading)?;
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
        atoms: request.atoms.clone(),
        metrics: run_metrics,
    };
    Ok((run, scale, leading))
}

fn extract_run(
    request: &RunRequest<'_, '_>,
    glyph_buffer: &surgeist_shaper::GlyphBuffer,
    coords: &[surgeist_shaper::NormalizedCoord],
    output: &mut ShapedData,
) -> Result<()> {
    // Receipt admission is part of the same invocation fence, including empty
    // glyph output. Each buffer.add above emits exactly one scalar per atom,
    // so these input ordinals map directly to this request's local atom range.
    let receipt = glyph_buffer
        .default_ignorable_receipt()
        .map_err(super::shaping_refused)?;
    let input = request.input;
    let (run, scale, leading) = resolved_run(request, coords)?;
    let font_ref = skrifa::FontRef::from_index(request.font.bytes(), request.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid selected font face"))?;
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
    let zero_input = |local: usize| {
        let ordinal = u32::try_from(local).expect("invocation scalar indices were admitted");
        receipt
            .residual_input_positions()
            .binary_search(&ordinal)
            .is_ok()
            || receipt
                .deleted_input_positions()
                .binary_search(&ordinal)
                .is_ok()
    };
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
        let cluster_source = original_extent(&input.projection.atoms[atom_start..atom_end]);
        let first_glyph = output.glyphs.len();
        let mut advance = 0.0_f64;
        let provider_glyphs = groups.get(&start).cloned().unwrap_or(0..0);
        for glyph_index in provider_glyphs {
            let ordinal = u32::try_from(glyph_index).map_err(|_| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "provider glyph index exceeds receipt range",
                )
            })?;
            if receipt
                .residual_glyph_positions()
                .binary_search(&ordinal)
                .is_ok()
            {
                continue;
            }
            let info = &infos[glyph_index];
            let position = &positions[glyph_index];
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
        let eligible_components = (start..end).filter(|local| !zero_input(*local)).count();
        let last_eligible = (start..end).rfind(|local| !zero_input(*local));
        if eligible_components == 0 && advance != 0.0 {
            return Err(Error::new(
                ErrorCode::LayoutFailed,
                "provider normal advance has no eligible original input",
            ));
        }
        let component_advance = if eligible_components == 0 {
            0.0
        } else {
            advance / eligible_components as f64
        };
        let mut distributed_base = 0.0;
        for atom in atom_start..atom_end {
            let original = &input.projection.atoms[atom];
            let (info, style_index) = input.analysis.infos[atom];
            let whitespace = whitespace(original.code_point);
            let local = atom - request.atoms.start;
            let excluded = zero_input(local);
            let base_advance = if excluded {
                0.0
            } else if Some(local) == last_eligible {
                advance - distributed_base
            } else {
                component_advance
            };
            distributed_base += base_advance;
            let style = &input.styles[style_index].style;
            let letter_spacing = if excluded { 0.0 } else { style.letter_spacing };
            let word_spacing = if !excluded && whitespace.is_space_or_nbsp() {
                style.word_spacing
            } else {
                0.0
            };
            let with_letter_spacing = logical_add("letter spacing", base_advance, letter_spacing)?;
            let cluster_advance = logical_add("word spacing", with_letter_spacing, word_spacing)?;
            output.clusters.push(PreparedCluster {
                line_height: resolved_line_height(
                    style,
                    run.metrics.ascent,
                    run.metrics.descent,
                    leading,
                )?,
                shaped_advance: base_advance,
                logical_adjustments: [letter_spacing, word_spacing, 0.0],
                source: original.source,
                atoms: atom..atom + 1,
                provider_group: atom_start..atom_end,
                glyphs: if atom == atom_start {
                    glyphs.clone()
                } else {
                    glyphs.end..glyphs.end
                },
                content: ClusterContent::Provider(run_index),
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

fn original_extent(atoms: &[super::OriginalAtom]) -> Range {
    let Some(first) = atoms.first() else {
        return Range::new(0, 0);
    };
    atoms.iter().skip(1).fold(first.source, |extent, atom| {
        Range::new(
            extent.start.min(atom.source.start),
            extent.end.max(atom.source.end),
        )
    })
}

fn whitespace(code_point: u32) -> Whitespace {
    match code_point {
        0x20 => Whitespace::Space,
        0xa0 => Whitespace::NoBreakSpace,
        0x09 => Whitespace::Tab,
        _ if is_hard_separator(code_point) => Whitespace::Newline,
        _ => Whitespace::None,
    }
}

/// Native first-available family probe. Raw registrations have unrestricted
/// unicode ranges. Glyph presence (including U+0020) does not select the face.
pub(crate) fn automatic_hyphen(
    mut query: Query<'_>,
    style: &ResolvedStyle,
    catalogue: &crate::FontCatalogueIdentity,
) -> Result<(crate::Text, FontData)> {
    query.set_families(style.font_family.iter().copied().map(QueryFamily::Id));
    query.set_attributes(attributes(style));
    let mut indication = None;
    query.matches_with(|font| {
        indication = Some((
            if font
                .charmap()
                .and_then(|map| map.map('\u{2010}'))
                .is_some_and(|glyph| glyph != 0)
            {
                "\u{2010}"
            } else {
                "-"
            },
            FontData::from_blob(font.blob.clone(), font.index, catalogue.clone()),
        ));
        QueryStatus::Stop
    });
    indication
        .map(|(text, font)| (crate::Text::from(text), font))
        .ok_or_else(|| crate::hyphenation::error(crate::HyphenationError::MissingPrimaryFont))
}

fn append_tab(input: &ShapingInput<'_>, atom: usize, font: Option<usize>, output: &mut ShapedData) {
    let original = &input.projection.atoms[atom];
    let (info, style) = input.analysis.infos[atom];
    output.clusters.push(PreparedCluster {
        // The admitted tab font was measured under this tab's actual style.
        line_height: font.map_or(0.0, |run| output.runs[run].metrics.line_height),
        shaped_advance: 0.0,
        logical_adjustments: [0.0; 3],
        source: original.source,
        atoms: atom..atom + 1,
        provider_group: atom..atom + 1,
        glyphs: output.glyphs.len()..output.glyphs.len(),
        content: ClusterContent::CssTab(font),
        style,
        level: input.analysis.levels[atom],
        advance: 0.0,
        boundary: info.boundary,
        whitespace: Whitespace::Tab,
        is_whitespace: true,
        ligature_start: false,
        ligature_component: false,
    });
}

/// Internal hypothetical character operation. No authored source or occurrence is created.
pub(crate) fn measure_character(
    mut query: Query<'_>,
    input: ShapingInput<'_>,
    style_index: usize,
    ch: char,
    direction: crate::ParagraphBaseDirection,
    scratch: &mut ShapeContext,
) -> Result<Option<(f64, PreparedRun)>> {
    // Independent hypothetical character analysis: Common uses the existing
    // isolated Latin fallback. SPACE WS follows the actual block base, while
    // ZERO EN is requested with its even/LTR resolved level.
    let level = u8::from(direction == crate::ParagraphBaseDirection::RightToLeft);
    let direction = provider_direction(level);
    let style = &input.styles[style_index].style;
    configure_query(
        &mut query,
        style,
        Script::Latin,
        false,
        &AnalysisDataSources,
    );
    let mut selected = None;
    query.matches_with(|font| {
        if font
            .charmap()
            .and_then(|map| map.map(ch))
            .is_some_and(|id| id != 0)
        {
            selected = Some(SelectedFont {
                font: SelectedResource::Matched(font.clone()),
                attrs: attributes(style),
            });
            QueryStatus::Stop
        } else {
            QueryStatus::Continue
        }
    });
    let Some(font) = selected else {
        return Ok(None);
    };
    let mut output = ShapedData::default();
    append_selected(
        &input,
        SelectedSegment {
            atoms: 0..0,
            style: style_index,
            script: Script::Latin,
            level,
            font,
            context: 0..0,
        },
        scratch,
        &mut output,
        false,
    )?;
    let run = output
        .empty_run
        .expect("zero-atom metric operation retains its actual face");
    let font = surgeist_shaper::FontRef::from_index(run.font.bytes(), run.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid metric face"))?;
    let instance = surgeist_shaper::ShaperInstance::from_coords(
        &font,
        run.coords
            .iter()
            .copied()
            .map(surgeist_shaper::NormalizedCoord::from_bits),
    );
    let data = surgeist_shaper::ShaperData::new(&font);
    let shaper = data
        .shaper(&font)
        .instance(Some(&instance))
        .point_size(Some(style.font_size))
        .build();
    let features: Vec<_> = style
        .font_features
        .iter()
        .map(|f| {
            surgeist_shaper::Feature::new(
                surgeist_shaper::Tag::new(&f.tag.to_bytes()),
                u32::from(f.value),
                ..,
            )
        })
        .collect();
    let language = style
        .locale
        .as_ref()
        .map(|locale| locale.language().parse::<surgeist_shaper::Language>())
        .transpose()
        .map_err(|_| Error::new(ErrorCode::InvalidStyle, "invalid metric language"))?;
    let plan = surgeist_shaper::ShapePlan::new(
        &shaper,
        direction,
        Some(provider_script(Script::Latin)),
        language.as_ref(),
        &features,
    );
    let mut buffer = surgeist_shaper::UnicodeBuffer::new();
    buffer.add(ch, 0);
    buffer.set_direction(direction);
    buffer.set_script(provider_script(Script::Latin));
    if let Some(language) = language {
        buffer.set_language(language);
    }
    let buffer = shaper.shape_with_plan(&plan, buffer, &features);
    if let surgeist_shaper::ShapingStatus::Refused(reason) = buffer.shaping_status() {
        return Err(super::shaping_refused(reason));
    }
    let advance = buffer
        .glyph_positions()
        .iter()
        .try_fold(0.0, |total, position| {
            logical_add(
                "tab character advance",
                total,
                f64::from(position.x_advance),
            )
        })?
        * f64::from(style.font_size)
        / f64::from(
            font.head()
                .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "metric font lacks head"))?
                .units_per_em(),
        );
    if !advance.is_finite() {
        return Err(super::text_numeric_error(
            "tab character advance",
            TextNumericPhase::Preparation,
            TextNumericError::NonFinite,
            advance,
        ));
    }
    Ok(Some((advance, run)))
}
