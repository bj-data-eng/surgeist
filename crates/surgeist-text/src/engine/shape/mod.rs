// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Adapted from Parley 0.9.0 shape/mod.rs. Original atoms and owned resources
// replace scalar-source/compressed carriers; actual provider status gates rows.

mod cache;
mod cell;
mod font_instance;
pub(crate) use font_instance::{CanonicalFontInstance, scope_font};
use font_instance::{ResourceSelection, canonical_instance, selected_resource};
#[cfg(test)]
mod tests;
mod vertical;

#[cfg(test)]
#[path = "selected_basis_tests.rs"]
mod selected_basis_tests;

#[cfg(test)]
thread_local! {
    static PROVIDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
fn reset_provider_calls() {
    PROVIDER_CALLS.set(0);
}
#[cfg(test)]
fn provider_calls() -> usize {
    PROVIDER_CALLS.get()
}
#[cfg(test)]
fn observe_provider_call() {
    PROVIDER_CALLS.set(PROVIDER_CALLS.get() + 1);
}

use std::{collections::BTreeSet, ops::Range as IndexRange};

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
        ClusterContent, GlyphData, HorizontalRunMetrics, PreparedCluster, PreparedRun,
        ProviderGroup, RunMetricBasis, RunMetrics, ShapedData, ShapedItem, Whitespace,
    },
    lru_cache::LruCache,
    resolve::ResolvedStyle,
};
use crate::{
    Error, ErrorCode, ErrorDetail, FontData, FontSynthesis, Range, Result, TextNumericError,
    TextNumericPhase,
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
    profile: super::upright::ShapingProfile<'_>,
    scopes: &[crate::protocol::prepared::presentation::DirectionalScope],
) -> Result<Vec<crate::protocol::prepared::RetainedAssignment>> {
    let input = ShapingInput {
        formatting_breaks: &[],
        scope_custody: None,
        characters: None,
        separator_geometry: false,
        baseline_reference: prepared.options.resolved_baseline_reference(),
        projection,
        analysis,
        styles: &prepared.styles,
        boxes: prepared.analyzed.processed.source().boxes(),
        font_catalogue: &prepared.font_catalogue,
        paragraph: &prepared.analyzed.paragraph,
        css_tabs: prepared.options.tabs() == crate::TabPreparation::CssStops,
        profile,
    };
    let mut cluster = CharCluster::default();
    let mut assignments: Vec<crate::protocol::prepared::RetainedAssignment> = Vec::new();
    let mut script = analysis
        .infos
        .iter()
        .map(|(i, _)| i.script)
        .find(|&s| real_script(s))
        .unwrap_or(Script::Latin);
    let barriers = shaping_barriers(&input);
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
                || barriers.contains(&end)
                || input.formatting_breaks.binary_search(&end).is_ok()
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

fn provider_features(
    style: &ResolvedStyle,
    native_isolated: bool,
    role: super::upright::RunRole,
) -> Vec<surgeist_shaper::Feature> {
    let mut features = Vec::with_capacity(style.font_features.len() + usize::from(native_isolated));
    if role == super::upright::RunRole::SidewaysClockwise {
        features.push(surgeist_shaper::Feature::new(
            surgeist_shaper::Tag::new(b"vrtr"),
            1,
            ..,
        ));
    }
    if native_isolated {
        features.push(surgeist_shaper::Feature::new(
            surgeist_shaper::Tag::new(b"isol"),
            1,
            ..,
        ));
    }
    features.extend(style.font_features.iter().map(|feature| {
        surgeist_shaper::Feature::new(
            surgeist_shaper::Tag::new(&feature.tag.to_bytes()),
            u32::from(feature.value),
            ..,
        )
    }));
    features
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
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<ShapedData> {
    shape_impl(catalogue, input, scratch, false, scope_budget, None)
}

pub(crate) fn admit_cells(
    analyzed: &super::AnalyzedData,
    tabs: crate::TabPreparation,
) -> Result<()> {
    cell::admit(analyzed, tabs)
}

/// Execute real native source cells alongside ordinary body shaping. Each
/// normal inner invocation uses its retained horizontal levels and settings.
pub(crate) fn prepare(
    catalogue: &mut crate::catalogue::Catalogue,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
    analyzed: &super::AnalyzedData,
    scopes: &super::scope_alignment::PreparedScopes,
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<ShapedData> {
    if !analyzed
        .combination
        .as_ref()
        .is_some_and(|output| !output.cells.is_empty())
    {
        return shape(catalogue, input, scratch, scope_budget);
    }
    let horizontal = cell::horizontal_input(&input);
    let mut cells = cell::Inputs::new(analyzed, scopes, scope_budget)?;
    let mut output = shape_impl(
        catalogue,
        input,
        scratch,
        false,
        scope_budget,
        Some(&mut cells),
    )?;
    cells.finish(
        &mut output,
        analyzed,
        &horizontal,
        scratch,
        scopes,
        scope_budget,
    )?;
    Ok(output)
}

/// Actual matching/instance capture without a redundant glyph-shaping operation.
pub(crate) fn capture_fonts(
    catalogue: &mut crate::catalogue::Catalogue,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
) -> Result<Vec<FontQueryWitness>> {
    shape_impl(catalogue, input, scratch, true, &mut None, None).map(|output| output.font_queries)
}

fn shape_impl(
    catalogue: &mut crate::catalogue::Catalogue,
    input: ShapingInput<'_>,
    scratch: &mut ShapeContext,
    capture_only: bool,
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
    mut cells: Option<&mut cell::Inputs>,
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
    if !capture_only && scope_budget.is_some() {
        start_scope_output(scope_budget)?;
    }
    let mut queries = Vec::new();
    let sources = AnalysisDataSources;
    let barriers = shaping_barriers(&input);
    let horizontal = ShapingInput {
        formatting_breaks: input.formatting_breaks,
        scope_custody: input.scope_custody,
        separator_geometry: input.separator_geometry,
        characters: None,
        profile: super::upright::ShapingProfile::Horizontal,
        baseline_reference: Some(crate::FontBaseline::Alphabetic),
        projection: input.projection,
        analysis: input.analysis,
        styles: input.styles,
        boxes: input.boxes,
        font_catalogue: input.font_catalogue,
        paragraph: input.paragraph,
        css_tabs: input.css_tabs,
    };
    let count = input.projection.atoms.len();
    // Empty/default-font metrics are an explicit preparation-profile operation.
    // Body shaping never infers that policy from source content or body style.
    if count != 0 {
        let mut item_start = 0;
        let mut script = if let Some(cells) = cells.as_deref() {
            let mut script = Script::Latin;
            for (atom, (info, _)) in input.analysis.infos.iter().enumerate() {
                super::scope_alignment::accounting::work(scope_budget, 1)?;
                if cells.owner(atom).is_none() && real_script(info.script) {
                    script = info.script;
                    break;
                }
            }
            script
        } else {
            input
                .analysis
                .infos
                .iter()
                .map(|(info, _)| info.script)
                .find(|script| real_script(*script))
                .unwrap_or(Script::Latin)
        };
        let mut ordinary_script = script;
        let mut inner_script: Option<(usize, Script)> = None;
        while item_start < count {
            let cell = cells.as_deref().and_then(|cells| cells.owner(item_start));
            script = if let Some(cell) = cell {
                if let Some((_, script)) = inner_script.filter(|(owner, _)| *owner == cell) {
                    script
                } else {
                    cells.as_deref().expect("actual cell input").first_script(
                        cell,
                        input.analysis,
                        scope_budget,
                    )?
                }
            } else {
                ordinary_script
            };
            let input = if cell.is_some() { &horizontal } else { &input };
            if let Some(cell) = cell {
                cells
                    .as_deref_mut()
                    .expect("actual cells")
                    .begin(cell, output.scope_relations.len());
            }
            if input.css_tabs && input.projection.atoms[item_start].code_point == 9 {
                append_tab(input, item_start, None, &mut output);
                if let Some(cell) = cell {
                    cells
                        .as_deref_mut()
                        .expect("actual cells")
                        .end(cell, output.scope_relations.len());
                }
                item_start += 1;
                continue;
            }
            let style_index = input.analysis.infos[item_start].1;
            let style = &input.styles[style_index].style;
            let level = cells
                .as_deref()
                .map_or(input.analysis.levels[item_start], |cells| {
                    cells.level(item_start, input.analysis.levels[item_start])
                });
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
                    || cells
                        .as_deref()
                        .map_or(input.analysis.levels[item_end], |cells| {
                            cells.level(item_end, input.analysis.levels[item_end])
                        })
                        != level
                    || cells
                        .as_deref()
                        .is_some_and(|cells| cells.owner(item_end) != cell)
                    || if let Some(cells) = cells.as_deref() {
                        cells.boundary(item_end, scope_budget)?
                    } else {
                        false
                    }
                    || next_script != script
                    || !same_shaping_style(style, &input.styles[next_style].style)
                    || barriers.contains(&item_end)
                    || crate::processing::contextual::alignment::operations::binary_search_by(
                        input.formatting_breaks,
                        scope_budget.as_mut(),
                        |cut| cut.cmp(&item_end),
                    )?
                    .is_ok()
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
            let domain = if let Some(cells) = cells.as_deref() {
                cells.context(item_start, count, scope_budget)?
            } else {
                0..count
            };
            let context = bounded_context(item_start..item_end, domain, &barriers);
            for range in input.analysis.graphemes[grapheme_start..grapheme_end]
                .iter()
                .flat_map(|grapheme| {
                    font_spans(
                        input.projection,
                        grapheme.start.max(item_start)..grapheme.end.min(item_end),
                    )
                })
            {
                fill_cluster(input, range.clone(), &mut scratch.cluster);
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
                        input,
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
                        scope_budget,
                        capture_only,
                    )?;
                    segment_start = range.start;
                }
                current = Some(selected);
            }
            let selected = current.ok_or_else(no_font)?;
            append_selected(
                input,
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
                scope_budget,
                capture_only,
            )?;
            if let Some(cell) = cell {
                cells
                    .as_deref_mut()
                    .expect("actual cells")
                    .end(cell, output.scope_relations.len());
            }
            if let Some(cell) = cell {
                inner_script = Some((cell, script));
            } else {
                ordinary_script = script;
            }
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
        retain_external_invocations(&mut output);
        if cells.is_none() {
            install_mixed_order(&mut output, &input);
        }
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
        &mut None,
        false,
    )?;
    Ok(output
        .empty_run
        .expect("the actual zero-atom operation produces its metrics row"))
}

/// Establish actual selected output before complete spacing handles are issued.
/// Retained faces/context are used without querying the catalogue or redoing fallback.
pub(crate) fn finalize(
    selected: crate::protocol::input::PresentationInput<'_>,
    scratch: &mut ShapeContext,
    spacing: crate::SpacingPolicy,
    allowance: Option<crate::ScopeAlignmentAllowance>,
    scope_input: super::scope_alignment::PlanInput<'_>,
) -> Result<(
    ShapedData,
    std::sync::Arc<crate::protocol::prepared::SelectedMapping>,
    super::scope_alignment::SelectionPlan,
)> {
    use crate::protocol::prepared::PresentationUnitKind;
    let prepared = &selected.prepared;
    let presentation = &selected.presentation;
    let mut scope_plan =
        super::scope_alignment::selection_plan(prepared, presentation, allowance, scope_input)?;
    let mut scope_budget = scope_plan.budget.take();
    let mut cells = cell::SelectedCells::new(prepared, presentation, &mut scope_budget)?;
    let character_input =
        super::spacing::character::Input::selected(prepared, presentation, &mut scope_budget)?;
    let characters = super::spacing::character::plan(
        character_input,
        spacing == crate::SpacingPolicy::InterCharacter,
        &mut scope_budget,
    )?;
    let mut native_spacing = None;
    let spacing_ledger = if character_input.has_cells() && spacing != crate::SpacingPolicy::None {
        &mut scope_budget
    } else {
        &mut native_spacing
    };
    let mut separator_feedback = false;
    match spacing {
        crate::SpacingPolicy::None | crate::SpacingPolicy::InterCharacter => (),
        crate::SpacingPolicy::Spaces => {
            for unit in &presentation.units {
                super::scope_alignment::accounting::work(spacing_ledger, 1)?;
                separator_feedback |= unit.eligible_space;
            }
        }
        crate::SpacingPolicy::InterWord => super::spacing::visit_outer_candidates(
            presentation,
            &selected.boundaries.presentation,
            spacing_ledger,
            |_, _| {
                separator_feedback = true;
                Ok(())
            },
        )?,
    }
    let input = ShapingInput {
        formatting_breaks: &scope_plan.breaks,
        scope_custody: scope_plan.custody(prepared),
        characters: Some(&characters),
        separator_geometry: separator_feedback
            || super::spacing::authored_demand(&presentation.analysis, &prepared.styles),
        baseline_reference: prepared.options.resolved_baseline_reference(),
        projection: &presentation.projection,
        analysis: &presentation.analysis,
        styles: &prepared.styles,
        boxes: prepared.analyzed.processed.source().boxes(),
        font_catalogue: &prepared.font_catalogue,
        paragraph: &prepared.analyzed.paragraph,
        css_tabs: prepared.options.tabs() == crate::TabPreparation::CssStops,
        profile: super::upright::selected_profile(prepared, presentation),
    };
    let mut output = ShapedData {
        character_requirements: Some(std::sync::Arc::clone(&characters)),
        ..ShapedData::default()
    };
    let native = selected.metric_policy == crate::protocol::selection::LineMetricPolicy::Native;
    if native {
        // This is the actual base-style strut retained by Native+DefaultFont.
        // AllocatedContributions has no such row and never borrows body metrics.
        output.empty_run = prepared.shaped.empty_run.clone();
    }
    if input.projection.atoms.is_empty() && !native {
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
    super::scope_alignment::admit_selected_resources(
        prepared,
        presentation,
        output.empty_run.as_ref(),
        &scope_plan.proof_frames,
        &mut scope_budget,
    )?;
    if scope_budget.is_some() {
        start_scope_output(&mut scope_budget)?;
    }
    if !input.projection.atoms.is_empty() {
        let computed = selected.computed;
        let barriers = &selected.boundaries.presentation;
        let mut selected_domains: Vec<JoiningDomain> = Vec::new();
        let mut original_domains: Vec<JoiningDomain> = Vec::new();
        let mut upright_barriers = shaping_barriers(&input);
        upright_barriers.extend(barriers.iter().copied());
        let mut split_assignments = Vec::new();
        let assignments = if input.is_mixed() || !input.formatting_breaks.is_empty() {
            for assignment in &presentation.assignments {
                let mut first = assignment.atoms.start;
                while first < assignment.atoms.end {
                    let end = next_assignment_end(
                        first,
                        assignment.atoms.end,
                        input.is_mixed(),
                        &upright_barriers,
                        input.formatting_breaks,
                        &mut scope_budget,
                    )?;
                    let mut run = super::scope_alignment::accounting::clone_run(
                        &mut scope_budget,
                        &assignment.run,
                    )?;
                    run.atoms = first..end;
                    super::scope_alignment::accounting::push(
                        &mut scope_budget,
                        &mut split_assignments,
                        crate::protocol::prepared::RetainedAssignment {
                            atoms: first..end,
                            run,
                        },
                    )?;
                    first = end;
                }
            }
            &split_assignments
        } else {
            &presentation.assignments
        };
        let barriers = if input.is_mixed() {
            &upright_barriers
        } else {
            barriers
        };
        let original_input = ShapingInput {
            formatting_breaks: &prepared.scope_alignment.breaks,
            scope_custody: prepared.scope_alignment.custody(&prepared.analyzed),
            profile: super::upright::original_profile(
                prepared.options.orientation(),
                &prepared.vertical_units,
                prepared.analyzed.typography,
                &prepared.analyzed.styles,
            ),
            projection: &prepared.analyzed.projection,
            analysis: &prepared.analyzed.analysis,
            characters: input.characters,
            separator_geometry: input.separator_geometry,
            baseline_reference: input.baseline_reference,
            styles: input.styles,
            boxes: input.boxes,
            font_catalogue: input.font_catalogue,
            paragraph: input.paragraph,
            css_tabs: input.css_tabs,
        };
        let mut original_barriers = shaping_barriers(&original_input);
        original_barriers.extend(selected.boundaries.original.iter().copied());
        for assignment in assignments {
            if cells.assignment(assignment.atoms.clone(), &mut scope_budget)? {
                continue;
            }
            let run = &assignment.run;
            if input.role(assignment.atoms.start) == super::upright::RunRole::Upright {
                let mut first = assignment.atoms.start;
                let ends = upright_barriers
                    .range((
                        std::ops::Bound::Excluded(first),
                        std::ops::Bound::Excluded(assignment.atoms.end),
                    ))
                    .copied()
                    .chain(std::iter::once(assignment.atoms.end));
                for end in ends {
                    let atoms = first..end;
                    let extent = bounded_context(
                        atoms.clone(),
                        0..input.projection.atoms.len(),
                        &upright_barriers,
                    );
                    append_retained_run(
                        &input,
                        atoms.clone(),
                        run,
                        RunContext::new(input.projection, atoms, extent),
                        scratch,
                        &mut output,
                        &mut scope_budget,
                    )?;
                    first = end;
                }
                continue;
            }
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
                let original_extent = if input.is_mixed() {
                    bounded_context(
                        seed.atoms.clone(),
                        selected.boundaries.original_extent(prepared, &seed),
                        &original_barriers,
                    )
                } else {
                    selected.boundaries.original_extent(prepared, &seed)
                };
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
                            if input.is_mixed() {
                                &original_barriers
                            } else {
                                &selected.boundaries.original
                            },
                        ),
                    );
                }
            }
            append_retained_run(
                &input,
                assignment.atoms.clone(),
                run,
                context,
                scratch,
                &mut output,
                &mut scope_budget,
            )?;
        }
        let mut split_bytes = super::scope_alignment::accounting::bytes(&split_assignments)?;
        for row in &split_assignments {
            split_bytes = split_bytes
                .checked_add(super::scope_alignment::accounting::bytes(&row.run.coords)?)
                .ok_or_else(|| {
                    super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                        kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
                    })
                })?;
        }
        drop(split_assignments);
        if let Some(budget) = &mut scope_budget {
            budget.release_bytes(split_bytes)?;
        }
    }
    if input.css_tabs {
        for atom in 0..input.projection.atoms.len() {
            if cells.assignment(atom..atom + 1, &mut scope_budget)? {
                continue;
            }
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
    retain_external_invocations(&mut output);
    cells.retain(prepared, presentation, &mut output, &mut scope_budget)?;
    let funded_word_receipts = spacing != crate::SpacingPolicy::None && !output.cells.is_empty();
    super::spacing::authored_selected(
        selected,
        &mut output,
        separator_feedback,
        funded_word_receipts,
        &mut scope_budget,
    )?;
    // Each real atom receives one extracted cluster; ligature components lend
    // their actual shared glyph association rather than duplicating glyphs.
    let mut clusters = output.clusters.iter().enumerate().peekable();
    for (unit, value) in presentation.units.iter().enumerate() {
        match value.kind {
            PresentationUnitKind::Cell { cell } => {
                cells.push_item(
                    &mut output.items,
                    ShapedItem::Cell(cells.cell(cell)),
                    &mut scope_budget,
                )?;
            }
            PresentationUnitKind::Atomic { occurrence } => {
                if input.boxes[occurrence].kind() == crate::InlineBoxKind::InFlow {
                    cells.push_item(
                        &mut output.items,
                        ShapedItem::InlineBox {
                            occurrence,
                            level: value.level,
                        },
                        &mut scope_budget,
                    )?;
                }
            }
            _ if value.atoms.is_empty() => cells.push_item(
                &mut output.items,
                ShapedItem::Anchor { unit },
                &mut scope_budget,
            )?,
            _ => {
                while let Some(&(index, cluster)) = clusters.peek() {
                    if cluster.atoms.start >= value.atoms.end {
                        break;
                    }
                    if cluster.atoms.start >= value.atoms.start {
                        cells.push_item(
                            &mut output.items,
                            ShapedItem::TextCluster(index),
                            &mut scope_budget,
                        )?;
                    }
                    clusters.next();
                }
            }
        }
    }
    cells.finish(&mut scope_budget)?;
    scope_plan.budget = scope_budget;
    Ok((
        output,
        std::sync::Arc::clone(&presentation.mapping),
        scope_plan,
    ))
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
        ShapedItem::Cell(_) => {
            unreachable!("source cells are installed after ordinary mixed order")
        }
    });
}

/// Borrowed pre/post strings from one genuine projection/context domain. The
/// shaper sees every bounded character and reports its own context limits.
struct RunContext<'a> {
    before: &'a str,
    after: &'a str,
    atoms: IndexRange<usize>,
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
        let extent = atom_bytes(projection, context.clone());
        Self {
            before: &projection.text[extent.start..bytes.start],
            after: &projection.text[bytes.end..extent.end],
            atoms: context,
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

impl RunRequest<'_, '_> {
    fn role(&self) -> super::upright::RunRole {
        if self.atoms.is_empty()
            && matches!(
                self.input.profile,
                super::upright::ShapingProfile::Mixed { .. }
            )
        {
            super::upright::RunRole::Upright
        } else {
            self.input.role(self.atoms.start)
        }
    }
    fn upright(&self) -> bool {
        self.role() == super::upright::RunRole::Upright
    }
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
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
    capture_only: bool,
) -> Result<()> {
    let selected = segment.font;
    let ResourceSelection {
        font,
        synthesis,
        suggested_variations,
    } = selected_resource(
        selected.font,
        input.styles[segment.style].style.font_size,
        input.font_catalogue,
    )?;
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
        scope_budget,
        capture_only,
    )
}

fn append_retained_run(
    input: &ShapingInput<'_>,
    atoms: IndexRange<usize>,
    run: &PreparedRun,
    context: RunContext<'_>,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<()> {
    append_request(
        RunRequest {
            input,
            atoms,
            style: run.style,
            script: run.script,
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
        scope_budget,
        false,
    )
}

/// Width execution consumes the normal run's exact effective instance. Plans
/// remain invocation-local; no adapter key/feature copies escape into a cache.
fn append_width_run(
    input: &ShapingInput<'_>,
    run: &PreparedRun,
    context: IndexRange<usize>,
    observation: &mut cell::width::Observation<'_>,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
    ledger: &mut super::scope_alignment::accounting::Ledger,
) -> Result<()> {
    use super::scope_alignment::accounting as account;
    use crate::processing::contextual::alignment::operations;
    let mut context = context;
    let left =
        operations::partition_point(&input.projection.barrier_atoms, ledger.as_mut(), |cut| {
            *cut <= run.atoms.start
        })?;
    if left != 0 {
        context.start = context.start.max(input.projection.barrier_atoms[left - 1]);
    }
    let right =
        operations::partition_point(&input.projection.barrier_atoms, ledger.as_mut(), |cut| {
            *cut < run.atoms.end
        })?;
    if let Some(cut) = input.projection.barrier_atoms.get(right) {
        context.end = context.end.min(*cut);
    }
    let request = RunRequest {
        input,
        atoms: run.atoms.clone(),
        style: run.style,
        script: run.script,
        level: run.level,
        font: &run.font,
        attrs: run.font_attrs,
        synthesis: run.synthesis,
        suggested_variations: Vec::new(),
        retained_coords: Some(&run.coords),
        context: RunContext::new(input.projection, run.atoms.clone(), context),
    };
    let font = surgeist_shaper::FontRef::from_index(run.font.bytes(), run.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid retained cell font"))?;
    let instance = surgeist_shaper::ShaperInstance::from_coords(
        &font,
        run.coords
            .iter()
            .copied()
            .map(surgeist_shaper::NormalizedCoord::from_bits),
    );
    let data = scratch
        .data
        .entry(cache::ResourceKey(run.font.clone()), || {
            surgeist_shaper::ShaperData::new(&font)
        });
    let style = &input.styles[run.style].style;
    let shaper = data
        .shaper(&font)
        .instance(Some(&instance))
        .point_size(Some(style.font_size))
        .build();
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
    // These width tags are native policy requests, not provider-classified
    // OptionalDefault features. Preserve later authored precedence and order.
    let mut features = Vec::new();
    account::push(
        ledger,
        &mut features,
        surgeist_shaper::Feature::new(observation.feature.tag(), 1, ..),
    )?;
    for feature in &style.font_features {
        account::push(
            ledger,
            &mut features,
            surgeist_shaper::Feature::new(
                surgeist_shaper::Tag::new(&feature.tag.to_bytes()),
                u32::from(feature.value),
                ..,
            ),
        )?;
    }
    let plan = surgeist_shaper::ShapePlan::new(
        &shaper,
        provider_direction(run.level),
        Some(provider_script(run.script)),
        language.as_ref(),
        &features,
    );
    let mut symbols = Vec::new();
    for (local, atom) in input.projection.atoms[run.atoms.clone()].iter().enumerate() {
        account::work(ledger, 1)?;
        if crate::control::is_visible(atom.code_point) {
            account::push(
                ledger,
                &mut symbols,
                u32::try_from(local).map_err(|_| {
                    super::shaping_refused(surgeist_shaper::ShapingRefusal::GrowthLimit)
                })?,
            )?;
        }
    }
    let mut buffer = scratch
        .buffer
        .take()
        .expect("a shape invocation returns its reusable buffer");
    buffer.clear();
    buffer.observe_width_feature(observation.feature);
    let result = (|| {
        let count = request.atoms.len();
        if count
            .checked_sub(1)
            .is_some_and(|last| u32::try_from(last).is_err())
        {
            return Err(Error::new(
                ErrorCode::SourceLengthOverflow,
                "cell input exceeds provider indices",
            ));
        }
        buffer.reserve(count);
        for (local, atom) in input.projection.atoms[request.atoms.clone()]
            .iter()
            .enumerate()
        {
            buffer.add(
                atom.glyph,
                u32::try_from(local).expect("admitted provider ordinal"),
            );
        }
        buffer
            .set_ordinary_symbol_positions(&symbols)
            .map_err(|error| {
                super::shaping_refused(match error {
                    surgeist_shaper::OrdinarySymbolInputError::GrowthLimit => {
                        surgeist_shaper::ShapingRefusal::GrowthLimit
                    }
                    surgeist_shaper::OrdinarySymbolInputError::WorkLimit => {
                        surgeist_shaper::ShapingRefusal::WorkLimit
                    }
                    _ => surgeist_shaper::ShapingRefusal::OrdinarySymbolInput,
                })
            })?;
        let grapheme =
            operations::partition_point(&input.analysis.graphemes, ledger.as_mut(), |g| {
                g.end <= request.atoms.start
            })?;
        let continuation = input
            .analysis
            .graphemes
            .get(grapheme)
            .is_some_and(|g| g.start < request.atoms.start && request.atoms.start < g.end);
        buffer.set_flags(if continuation {
            surgeist_shaper::BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE
        } else {
            surgeist_shaper::BufferFlags::empty()
        });
        buffer.set_pre_context(request.context.before);
        buffer.set_post_context(request.context.after);
        buffer.set_direction(provider_direction(run.level));
        buffer.set_script(provider_script(run.script));
        if let Some(language) = language {
            buffer.set_language(language);
        }
        Ok(())
    })();
    if let Err(error) = result {
        scratch.buffer = Some(buffer);
        account::drop_vec(ledger, symbols)?;
        account::drop_vec(ledger, features)?;
        return Err(error);
    }
    #[cfg(test)]
    observe_provider_call();
    let glyphs = shaper.shape_with_plan(&plan, buffer, &features);
    let result = (|| {
        if let surgeist_shaper::ShapingStatus::Refused(reason) = glyphs.shaping_status() {
            return Err(super::shaping_refused(reason));
        }
        observation.record(request.atoms.clone(), &glyphs, ledger)?;
        extract_run(&request, &glyphs, shaper.coords(), output, ledger)
    })();
    scratch.buffer = Some(glyphs.clear());
    let symbols_dropped = account::drop_vec(ledger, symbols);
    let features_dropped = account::drop_vec(ledger, features);
    result?;
    symbols_dropped?;
    features_dropped
}

fn append_request(
    mut request: RunRequest<'_, '_>,
    scratch: &mut ShapeContext,
    output: &mut ShapedData,
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
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
        canonical_instance(&font_ref, &key, request.retained_coords)
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
        let (run, _, _) = resolved_run(&request, shaper.coords(), None)?;
        output.runs.push(run);
        return Ok(());
    }
    if request.upright() && request.atoms.is_empty() {
        let (run, _, _) = resolved_run(&request, shaper.coords(), None)?;
        output.empty_run = Some(run);
        return Ok(());
    }
    let direction = if request.upright() {
        if request.input.is_mixed() && request.level & 1 != 0 {
            surgeist_shaper::Direction::BottomToTop
        } else {
            surgeist_shaper::Direction::TopToBottom
        }
    } else {
        provider_direction(request.level)
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
    let native_isolated = request.upright()
        && matches!(request.script, Script::Arabic | Script::Syriac)
        && request
            .input
            .vertical_units()
            .iter()
            .filter_map(|unit| unit.upright())
            .any(|unit| unit.isolate() && unit.atoms.contains(&request.atoms.start));
    let mut features = provider_features(style, native_isolated, request.role());
    let mut funded_features = false;
    if let Some(graph) = request.input.characters {
        let masks = graph.masks(request.atoms.clone(), scope_budget)?;
        if graph.funded() && !masks.is_empty() {
            if let Some(budget) = scope_budget.as_mut() {
                budget.work(features.len())?;
                budget.add_records(features.len())?;
                budget.retain_bytes(super::scope_alignment::accounting::bytes(&features)?)?;
            }
            funded_features = true;
        }
        for range in &masks {
            for tag in [b"liga", b"clig", b"dlig", b"hlig"] {
                u32::try_from(range.start)
                    .and_then(|_| u32::try_from(range.end))
                    .map_err(|_| {
                        Error::new(ErrorCode::SourceLengthOverflow, "optional feature range")
                    })?;
                let feature = surgeist_shaper::Feature::optional_default(
                    surgeist_shaper::Tag::new(tag),
                    0,
                    range.clone(),
                );
                if funded_features {
                    super::scope_alignment::accounting::push(scope_budget, &mut features, feature)?;
                } else {
                    features.push(feature);
                }
            }
        }
        if graph.funded() {
            super::scope_alignment::accounting::drop_vec(scope_budget, masks)?;
        }
    }
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
    let observe_components = if request
        .input
        .characters
        .is_some_and(|requirements| requirements.observe)
    {
        true
    } else if let Some(custody) = request.input.scope_custody {
        match custody.observes(request.atoms.clone(), scope_budget) {
            Ok(observes) => observes,
            Err(error) => {
                // This gate owns the taken, already-cleared reusable buffer.
                // A feature-work refusal must preserve the same return fence.
                scratch.buffer = Some(buffer);
                return Err(error);
            }
        }
    } else {
        false
    };
    if observe_components {
        buffer.observe_components();
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
    // Formatting cuts split emitted bodies, not genuine Unicode joining
    // lookaround. These strings insert no glyphs into GSUB/GPOS operands.
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
    if request.upright() {
        let shaped = vertical::shape(&request, &shaper, plan, buffer, &features);
        let result = match shaped {
            Ok(glyph_buffer) => {
                let result = extract_vertical_run(
                    &request,
                    &glyph_buffer,
                    shaper.coords(),
                    output,
                    scope_budget,
                );
                scratch.buffer = Some(glyph_buffer.clear());
                result
            }
            Err(error) => {
                // The provider consumes refused invocations and returns no
                // accepted buffer. Retain a fresh reusable input for next time.
                scratch.buffer = Some(surgeist_shaper::UnicodeBuffer::new());
                Err(error)
            }
        };
        if funded_features {
            super::scope_alignment::accounting::drop_vec(scope_budget, features)?;
        }
        return result;
    }
    #[cfg(test)]
    observe_provider_call();
    let glyph_buffer = shaper.shape_with_plan(plan, buffer, &features);
    // This is the actual returned invocation's status. No glyph, position,
    // metric or immutable result row is observed before this fence.
    if let surgeist_shaper::ShapingStatus::Refused(reason) = glyph_buffer.shaping_status() {
        scratch.buffer = Some(glyph_buffer.clear());
        return Err(super::shaping_refused(reason));
    }
    let result = extract_run(
        &request,
        &glyph_buffer,
        shaper.coords(),
        output,
        scope_budget,
    );
    scratch.buffer = Some(glyph_buffer.clear());
    if funded_features {
        super::scope_alignment::accounting::drop_vec(scope_budget, features)?;
    }
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

fn next_assignment_end(
    first: usize,
    end: usize,
    mixed: bool,
    native: &BTreeSet<usize>,
    formatting: &[usize],
    budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<usize> {
    let native = if mixed {
        native
            .range((
                std::ops::Bound::Excluded(first),
                std::ops::Bound::Excluded(end),
            ))
            .next()
            .copied()
            .unwrap_or(end)
    } else {
        end
    };
    let next = crate::processing::contextual::alignment::operations::partition_point(
        formatting,
        budget.as_mut(),
        |&cut| cut <= first,
    )?;
    Ok(native.min(
        formatting
            .get(next)
            .copied()
            .filter(|&cut| cut < end)
            .unwrap_or(end),
    ))
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

fn shaping_barriers(input: &ShapingInput<'_>) -> BTreeSet<usize> {
    let mut barriers: BTreeSet<_> = input.projection.barrier_atoms.iter().copied().collect();
    let mut prior_upright = None;
    for unit in input.vertical_units() {
        let upright = unit.upright();
        let atoms = unit.atoms();
        if prior_upright.is_some_and(|prior| prior != upright.is_some()) {
            barriers.insert(atoms.start);
        }
        if upright.is_some_and(|unit| unit.isolate()) {
            barriers.insert(atoms.start);
            barriers.insert(atoms.end);
        }
        prior_upright = Some(upright.is_some());
    }
    barriers
}

/// Metric attribution does not split otherwise compatible joint shaping.
pub(super) fn resolved_line_height(style: &ResolvedStyle, normal_extent: f64) -> Result<f64> {
    super::resolve::font_line_height(
        style.font_size,
        style.line_height,
        normal_extent,
        TextNumericPhase::Shaping,
    )
}

fn component_line_height(
    style: &ResolvedStyle,
    metrics: &RunMetrics,
    normal_extent: f64,
) -> Result<f64> {
    match metrics.basis {
        RunMetricBasis::Horizontal(_) => resolved_line_height(style, normal_extent),
        RunMetricBasis::Upright {
            normal_extent_design,
            design_scale,
        } => vertical::line_height(style, normal_extent_design, design_scale),
    }
}

fn resolved_run(
    request: &RunRequest<'_, '_>,
    coords: &[surgeist_shaper::NormalizedCoord],
    ledger: Option<&mut super::scope_alignment::accounting::Ledger>,
) -> Result<(PreparedRun, f64, f64)> {
    let input = request.input;
    let style = &input.styles[request.style].style;
    let stored_coords: Vec<_> = if let Some(ledger) = ledger {
        super::scope_alignment::accounting::collect(
            ledger,
            coords.iter().map(|coord| coord.to_bits()),
        )?
    } else {
        coords.iter().map(|coord| coord.to_bits()).collect()
    };
    let instance = crate::EffectiveFontInstance::qualify_synthesized(
        request.font.face(),
        style.font_size,
        &stored_coords,
        request.synthesis,
    )?;
    if request.upright() {
        return vertical::resolved_run(request, instance, &stored_coords);
    }
    let metrics = instance.metrics(input.font_catalogue)?;
    let font_ref = skrifa::FontRef::from_index(request.font.bytes(), request.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid selected font face"))?;
    let raw_metrics = skrifa::metrics::Metrics::new(
        &font_ref,
        skrifa::prelude::Size::new(style.font_size),
        coords,
    );
    let leading = metrics.leading();
    // Normal extent is independent of the named reference. Retain the raw
    // calculation for every component rather than canceling two rebased values.
    let normal_extent = metrics.ascent() - metrics.descent() + leading;
    let line_height = resolved_line_height(style, normal_extent)?;
    let baseline_reference = match input.baseline_reference {
        Some(crate::FontBaseline::Central)
            if request.role() == super::upright::RunRole::SidewaysClockwise =>
        {
            Some(crate::FontBaseline::Central)
        }
        Some(crate::FontBaseline::Alphabetic) => Some(crate::FontBaseline::Alphabetic),
        _ => None,
    };
    let font_origin_offset = if let Some(reference) = baseline_reference {
        let script = crate::FontBaselineScript::from_script(request.script).ok_or_else(|| {
            Error::new(
                ErrorCode::InvalidFontMetrics,
                "unavailable actual baseline script",
            )
            .with_detail(ErrorDetail::FontMetricTable {
                table: crate::FontMetricTable::Base,
                reason: crate::FontMetricTableError::BaselineScriptUnavailable,
            })
        })?;
        let baseline = instance.baseline(
            input.font_catalogue,
            script,
            crate::FontBaselineAxis::HorizontalY,
            reference,
        )?;
        // The raw owner supplies the named HorizontalY coordinate or its
        // qualified synthesis; malformed consumed data returned above.
        let crate::FontBaselineOutcome::Available(value) = baseline.outcome() else {
            unreachable!("horizontal used baseline measurement always has a value")
        };
        value.coordinate()
    } else {
        0.0
    };
    let ascent = super::layout::add(
        "used font ascent",
        TextNumericPhase::Shaping,
        metrics.ascent(),
        -font_origin_offset,
    )?;
    let descent = super::layout::add(
        "used font descent",
        TextNumericPhase::Shaping,
        -metrics.descent(),
        font_origin_offset,
    )?;
    let underline = raw_metrics.underline;
    let raw_underline_top = underline.map_or(f64::from(metrics.units_per_em()) / 18.0, |value| {
        f64::from(value.offset)
    });
    let underline_thickness = underline.map_or(f64::from(metrics.units_per_em()) / 18.0, |value| {
        f64::from(value.thickness)
    });
    let underline_top = super::layout::add(
        "used underline top",
        TextNumericPhase::Shaping,
        raw_underline_top,
        -font_origin_offset,
    )?;
    let strike_center = super::layout::add(
        "used strike center",
        TextNumericPhase::Shaping,
        metrics.strikeout().center_offset(),
        -font_origin_offset,
    )?;
    let run_metrics = RunMetrics {
        ascent,
        descent,
        line_height,
        basis: RunMetricBasis::Horizontal(HorizontalRunMetrics {
            font_origin_offset,
            underline_top,
            underline_thickness,
            strike_center,
            strike_thickness: metrics.strikeout().thickness(),
        }),
    };
    for (field, value) in [
        ("ascent", ascent),
        ("descent", descent),
        ("leading", leading),
        ("normal extent", normal_extent),
        ("font origin offset", font_origin_offset),
        ("line height", line_height),
        ("underline top", underline_top),
        ("underline thickness", underline_thickness),
        ("strike center", strike_center),
        ("x height", raw_metrics.x_height.map_or(0.0, f64::from)),
        ("cap height", raw_metrics.cap_height.map_or(0.0, f64::from)),
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
        glyph_basis: match request.role() {
            super::upright::RunRole::SidewaysClockwise => crate::GlyphBasis::Clockwise,
            super::upright::RunRole::SidewaysCounterclockwise => {
                crate::GlyphBasis::Counterclockwise
            }
            super::upright::RunRole::Horizontal | super::upright::RunRole::Upright => {
                crate::GlyphBasis::Unrotated
            }
        },
        metrics: run_metrics,
    };
    Ok((run, scale, normal_extent))
}

// These compact frame facts are an alignment consumer of the real provider
// receipt. They do not widen generic spacing membership or manufacture roots.
fn start_scope_output(budget: &mut super::scope_alignment::accounting::Ledger) -> Result<()> {
    super::scope_alignment::accounting::header::<Vec<super::scope_alignment::GlyphCustody>>(
        budget,
    )?;
    super::scope_alignment::accounting::header::<Vec<super::scope_alignment::ScopeRelation>>(budget)
}

fn scope_group<I: Iterator<Item = u32> + Clone>(
    request: &RunRequest<'_, '_>,
    custody: super::scope_alignment::CustodyInput<'_>,
    members: I,
    unqualified: bool,
    output: &mut ShapedData,
    budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<()> {
    let mut first_members = members.clone();
    loop {
        // Charge the actual iterator probe before inspecting its member/end.
        super::scope_alignment::accounting::work(budget, 1)?;
        let Some(first) = first_members.next() else {
            break;
        };
        let first = first as usize;
        if first >= request.atoms.len() {
            return Err(invalid_output(first, request.atoms.len()));
        }
        let first_owners = custody.input_owners(request.atoms.start + first, budget)?;
        for owner in first_owners {
            super::scope_alignment::accounting::work(budget, 1)?;
            let owner = *owner;
            let first = custody.target(owner, budget)?;
            let mut second_members = members.clone();
            loop {
                super::scope_alignment::accounting::work(budget, 1)?;
                let Some(second) = second_members.next() else {
                    break;
                };
                let second = second as usize;
                if second >= request.atoms.len() {
                    return Err(invalid_output(second, request.atoms.len()));
                }
                let second_owners = custody.input_owners(request.atoms.start + second, budget)?;
                for owner in second_owners {
                    super::scope_alignment::accounting::work(budget, 1)?;
                    let owner = *owner;
                    let second = custody.target(owner, budget)?;
                    if first >= second {
                        continue;
                    }
                    let relation = super::scope_alignment::ScopeRelation {
                        first,
                        second,
                        unqualified,
                    };
                    super::scope_alignment::accounting::work(budget, output.scope_relations.len())?;
                    if !output.scope_relations.contains(&relation) {
                        super::scope_alignment::accounting::push(
                            budget,
                            &mut output.scope_relations,
                            relation,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn retain_scope_relations(
    request: &RunRequest<'_, '_>,
    receipt: Option<&surgeist_shaper::ComponentReceipt<'_>>,
    glyph_count: usize,
    residual: &[u32],
    output: &mut ShapedData,
    budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<()> {
    let Some(custody) = request.input.scope_custody else {
        return Ok(());
    };
    if !custody.observes(request.atoms.clone(), budget)? {
        return Ok(());
    };
    if let Some(receipt) = receipt
        && receipt.input_len() != request.atoms.len()
    {
        return Err(invalid_output(receipt.input_len(), request.atoms.len()));
    }
    let mut unqualified = receipt
        .is_none_or(|receipt| receipt.coverage() != surgeist_shaper::ComponentCoverage::Complete);
    let mut surviving = false;
    for glyph in 0..glyph_count {
        super::scope_alignment::accounting::work(budget, 1)?;
        let ordinal = u32::try_from(glyph).map_err(|_| invalid_output(glyph, glyph_count))?;
        if crate::processing::contextual::alignment::operations::binary_search_by(
            residual,
            budget.as_mut(),
            |value| value.cmp(&ordinal),
        )?
        .is_ok()
        {
            continue;
        }
        surviving = true;
        if let Some(receipt) = receipt {
            let members = receipt
                .components(glyph)
                .ok_or_else(|| invalid_output(glyph, glyph_count))?;
            scope_group(
                request,
                custody,
                members.iter().copied(),
                false,
                output,
                budget,
            )?;
            if members.is_empty() {
                super::scope_alignment::accounting::work(budget, 1)?;
                if let Some(context) = receipt.generated_context(glyph) {
                    scope_group(
                        request,
                        custody,
                        context.iter().copied(),
                        false,
                        output,
                        budget,
                    )?;
                    unqualified |= context.is_empty();
                } else {
                    unqualified = true;
                }
            }
        }
    }
    if !surviving {
        return Ok(());
    };
    if let Some(receipt) = receipt {
        for ligature in receipt.ligatures() {
            super::scope_alignment::accounting::work(budget, 1)?;
            if !ligature.glyph_indices().is_empty() {
                scope_group(
                    request,
                    custody,
                    ligature.components().iter().copied(),
                    false,
                    output,
                    budget,
                )?;
            }
        }
        for attachment in receipt.attachments() {
            super::scope_alignment::accounting::work(budget, 1)?;
            scope_group(
                request,
                custody,
                attachment
                    .dependent_components()
                    .iter()
                    .chain(attachment.base_components())
                    .copied(),
                false,
                output,
                budget,
            )?;
        }
    }
    if unqualified {
        let members = (0..request.atoms.len())
            .map(|local| u32::try_from(local).expect("provider input ordinals admitted"));
        scope_group(request, custody, members, true, output, budget)?;
    }
    Ok(())
}

fn scope_glyph_owner(
    custody: super::scope_alignment::CustodyInput<'_>,
    atoms: IndexRange<usize>,
    receipt: Option<&surgeist_shaper::ComponentReceipt<'_>>,
    glyph: usize,
    budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<super::scope_alignment::GlyphCustody> {
    use super::scope_alignment::GlyphCustody as G;
    if !custody.observes(atoms.clone(), budget)? {
        return Ok(G::Unscoped);
    };
    let first_owner = |members: &[u32],
                       budget: &mut super::scope_alignment::accounting::Ledger|
     -> Result<Option<usize>> {
        for member in members {
            super::scope_alignment::accounting::work(budget, 1)?;
            let local = *member as usize;
            if local >= atoms.len() {
                return Err(invalid_output(local, atoms.len()));
            }
            let owners = custody.input_owners(atoms.start + local, budget)?;
            if !owners.is_empty() {
                super::scope_alignment::accounting::work(budget, 1)?;
                return Ok(Some(owners[0]));
            }
        }
        Ok(None)
    };
    if let Some(receipt) = receipt {
        // Querying the actual component/context descriptor is also inspected
        // work; the borrowed member rows are charged separately below.
        super::scope_alignment::accounting::work(budget, 1)?;
        let members = receipt
            .components(glyph)
            .ok_or_else(|| invalid_output(glyph, glyph + 1))?;
        if let Some(owner) = first_owner(members, budget)? {
            return Ok(G::Input(owner));
        }
        super::scope_alignment::accounting::work(budget, 1)?;
        if let Some(context) = receipt.generated_context(glyph)
            && let Some(owner) = first_owner(context, budget)?
        {
            return Ok(G::Context(owner));
        }
    }
    // No membership guarantee is made. Inspect only actual fallback rows until
    // the first contextual owner; the uncertain cohort still governs placement.
    for atom in atoms {
        let owners = custody.input_owners(atom, budget)?;
        if !owners.is_empty() {
            super::scope_alignment::accounting::work(budget, 1)?;
            return Ok(G::UnqualifiedContext(Some(owners[0])));
        }
    }
    Ok(G::UnqualifiedContext(None))
}

fn extract_run(
    request: &RunRequest<'_, '_>,
    glyph_buffer: &surgeist_shaper::GlyphBuffer,
    coords: &[surgeist_shaper::NormalizedCoord],
    output: &mut ShapedData,
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<()> {
    // Receipt admission is part of the same invocation fence, including empty
    // glyph output. Each buffer.add above emits exactly one scalar per atom,
    // so these input ordinals map directly to this request's local atom range.
    let components = glyph_buffer
        .component_receipt()
        .map_err(super::shaping_refused)?;
    if let Some(receipt) = components.as_ref().filter(|_| {
        request
            .input
            .characters
            .is_some_and(|requirements| requirements.observe)
    }) {
        retain_components(request, receipt, glyph_buffer.glyph_infos().len(), output)?;
    } else if request.input.characters.is_some() {
        output
            .character_invocations
            .push(super::spacing::character::Invocation {
                atoms: request.atoms.clone(),
                complete: false,
                path: None,
                run: output.runs.len(),
                components: Vec::new(),
                generated: Vec::new(),
                constraints: Vec::new(),
            });
    }
    let receipt = glyph_buffer
        .default_ignorable_receipt()
        .map_err(super::shaping_refused)?;
    let funded = output.cell_arenas == super::layout::data::CellArenaOwnership::Owned;
    let values = glyph_buffer
        .glyph_positions()
        .iter()
        .map(|position| ProviderPosition {
            advance: f64::from(position.x_advance),
            x: f64::from(position.x_offset),
            y: -f64::from(position.y_offset),
        });
    let positions: Vec<_> = if funded {
        super::scope_alignment::accounting::collect(scope_budget, values)?
    } else {
        values.collect()
    };
    let result = extract_output(
        request,
        glyph_buffer.glyph_infos(),
        &positions,
        receipt,
        coords,
        components.as_ref(),
        GlyphWriter {
            data: output,
            scope_budget,
        },
    );
    let dropped = if funded {
        super::scope_alignment::accounting::drop_vec(scope_budget, positions)
    } else {
        drop(positions);
        Ok(())
    };
    result?;
    dropped
}

struct ProviderPosition {
    advance: f64,
    x: f64,
    y: f64,
}

fn extract_vertical_run(
    request: &RunRequest<'_, '_>,
    glyph_buffer: &surgeist_shaper::VerticalGlyphBuffer,
    coords: &[surgeist_shaper::NormalizedCoord],
    output: &mut ShapedData,
    scope_budget: &mut super::scope_alignment::accounting::Ledger,
) -> Result<()> {
    let components = glyph_buffer
        .component_receipt()
        .map_err(super::shaping_refused)?;
    if let Some(receipt) = components.as_ref().filter(|_| {
        request
            .input
            .characters
            .is_some_and(|requirements| requirements.observe)
    }) {
        retain_components(request, receipt, glyph_buffer.glyph_infos().len(), output)?;
    } else if request.input.characters.is_some() {
        output
            .character_invocations
            .push(super::spacing::character::Invocation {
                atoms: request.atoms.clone(),
                complete: false,
                path: None,
                run: output.runs.len(),
                components: Vec::new(),
                generated: Vec::new(),
                constraints: Vec::new(),
            });
    }
    let receipt = glyph_buffer
        .default_ignorable_receipt()
        .map_err(super::shaping_refused)?;
    let positions: Vec<_> = glyph_buffer
        .glyph_positions()
        .map(|position| ProviderPosition {
            advance: -position.y_advance,
            x: position.x_offset,
            y: -position.y_offset,
        })
        .collect();
    extract_output(
        request,
        glyph_buffer.glyph_infos(),
        &positions,
        receipt,
        coords,
        components.as_ref(),
        GlyphWriter {
            data: output,
            scope_budget,
        },
    )
}

fn retain_components(
    request: &RunRequest<'_, '_>,
    receipt: &surgeist_shaper::ComponentReceipt<'_>,
    glyph_count: usize,
    output: &mut ShapedData,
) -> Result<()> {
    if receipt.input_len() != request.atoms.len() {
        return Err(invalid_output(receipt.input_len(), request.atoms.len()));
    }
    let mapped = |members: &[u32]| -> Result<Vec<usize>> {
        members
            .iter()
            .map(|&i| {
                let local = i as usize;
                if local >= request.atoms.len() {
                    Err(invalid_output(local, request.atoms.len()))
                } else {
                    Ok(request.atoms.start + local)
                }
            })
            .collect()
    };
    let mut components = Vec::new();
    let mut generated = Vec::new();
    for i in 0..glyph_count {
        components.push(mapped(
            receipt
                .components(i)
                .ok_or_else(|| invalid_output(i, glyph_count))?,
        )?);
        if receipt.generated(i) == Some(true) {
            generated.push(mapped(
                receipt
                    .generated_context(i)
                    .ok_or_else(|| invalid_output(i, glyph_count))?,
            )?);
        }
    }
    let mut constraints = Vec::new();
    for relation in receipt.ligatures() {
        constraints.push(super::spacing::character::Constraint {
            run: output.runs.len(),
            components: mapped(relation.components())?,
            required: relation
                .provenance()
                .iter()
                .any(|p| p.role() == surgeist_shaper::FeatureRole::Required),
            kind: crate::SpacingConstraintKind::Ligature,
            provenance: relation.provenance().to_vec(),
        });
    }
    for relation in receipt.attachments() {
        let mut members = mapped(relation.dependent_components())?;
        members.extend(mapped(relation.base_components())?);
        members.sort_unstable();
        members.dedup();
        constraints.push(super::spacing::character::Constraint {
            run: output.runs.len(),
            components: members,
            required: false,
            kind: match relation.kind() {
                surgeist_shaper::ComponentAttachmentKind::Mark => {
                    crate::SpacingConstraintKind::MarkAttachment
                }
                surgeist_shaper::ComponentAttachmentKind::Cursive => {
                    crate::SpacingConstraintKind::CursiveAttachment
                }
            },
            provenance: Vec::new(),
        });
    }
    if receipt.joining_context_use() == surgeist_shaper::JoiningContextUse::ContextualActions {
        retain_external_joining(request, output.runs.len(), &mut constraints);
    }
    output
        .character_invocations
        .push(super::spacing::character::Invocation {
            atoms: request.atoms.clone(),
            complete: receipt.coverage() == surgeist_shaper::ComponentCoverage::Complete,
            path: match receipt.coverage() {
                surgeist_shaper::ComponentCoverage::Complete => None,
                surgeist_shaper::ComponentCoverage::Unqualified(path) => Some(path),
            },
            run: output.runs.len(),
            components,
            generated,
            constraints,
        });
    Ok(())
}

/// Retain only actual potential Unicode connections in the bounded context of
/// an invocation that executed joining actions. No script-to-shaper inference
/// or claim about an applied external attachment is made here.
fn retain_external_joining(
    request: &RunRequest<'_, '_>,
    run: usize,
    constraints: &mut Vec<super::spacing::character::Constraint>,
) {
    use icu_properties::{CodePointMapData, props::JoiningType};
    let types = CodePointMapData::<JoiningType>::new();
    let mut before = None;
    for atom in request.context.atoms.clone() {
        let joining = types.get32(request.input.projection.atoms[atom].code_point);
        if matches!(joining, JoiningType::Transparent | JoiningType::JoinCausing) {
            continue;
        }
        if let Some((previous, previous_type)) = before
            && matches!(
                previous_type,
                JoiningType::DualJoining | JoiningType::LeftJoining
            )
            && matches!(
                joining,
                JoiningType::DualJoining | JoiningType::RightJoining
            )
            && request.atoms.contains(&previous) != request.atoms.contains(&atom)
        {
            constraints.push(super::spacing::character::Constraint {
                run,
                components: vec![previous, atom],
                required: false,
                kind: crate::SpacingConstraintKind::ExternalJoiningContext {
                    body: request.atoms.clone(),
                    context: request.context.atoms.clone(),
                },
                provenance: Vec::new(),
            });
        }
        // Nonjoining characters, including ZWNJ, replace the previous endpoint
        // and therefore break connectivity rather than being skipped.
        before = Some((atom, joining));
    }
}

/// A source context scalar only constrains this result when another actual
/// emitted invocation owns it. Selected-away context is not another line unit.
fn retain_external_invocations(output: &mut ShapedData) {
    let bodies: Vec<_> = output
        .character_invocations
        .iter()
        .map(|invocation| (invocation.run, invocation.atoms.clone()))
        .collect();
    for invocation in &mut output.character_invocations {
        invocation.constraints.retain(|constraint| {
            let crate::SpacingConstraintKind::ExternalJoiningContext { body, .. } =
                &constraint.kind
            else {
                return true;
            };
            constraint.components.iter().all(|atom| {
                body.contains(atom)
                    || bodies
                        .iter()
                        .any(|(run, atoms)| *run != invocation.run && atoms.contains(atom))
            })
        });
    }
}

/// One stored output and its operation ledger. Glyph custody is committed in
/// the same extraction order as the actual glyph, including scoped reservation.
struct GlyphWriter<'a> {
    data: &'a mut ShapedData,
    scope_budget: &'a mut super::scope_alignment::accounting::Ledger,
}
impl GlyphWriter<'_> {
    fn push_glyph(
        &mut self,
        custody: Option<super::scope_alignment::GlyphCustody>,
        glyph: impl FnOnce() -> Result<GlyphData>,
    ) -> Result<()> {
        if let Some(owner) = custody {
            super::scope_alignment::accounting::push(
                self.scope_budget,
                &mut self.data.scope_glyph_custody,
                owner,
            )?;
        }
        // Preserve provider validation and reservation order: numeric glyph
        // construction follows custody admission, exactly as ordinary extraction.
        if self.data.cell_arenas == super::layout::data::CellArenaOwnership::Owned {
            super::scope_alignment::accounting::push(
                self.scope_budget,
                &mut self.data.glyphs,
                glyph()?,
            )?;
        } else {
            self.data.glyphs.push(glyph()?);
        }
        Ok(())
    }
}

fn extract_output(
    request: &RunRequest<'_, '_>,
    infos: &[surgeist_shaper::GlyphInfo],
    positions: &[ProviderPosition],
    receipt: surgeist_shaper::DefaultIgnorableReceipt<'_>,
    coords: &[surgeist_shaper::NormalizedCoord],
    components: Option<&surgeist_shaper::ComponentReceipt<'_>>,
    mut writer: GlyphWriter<'_>,
) -> Result<()> {
    let input = request.input;
    let funded = writer.data.cell_arenas == super::layout::data::CellArenaOwnership::Owned;
    let (run, scale, normal_extent) =
        resolved_run(request, coords, funded.then_some(&mut *writer.scope_budget))?;
    let font_ref = skrifa::FontRef::from_index(request.font.bytes(), request.font.index())
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "invalid selected font face"))?;
    if request.atoms.is_empty() {
        writer.data.empty_run = Some(run);
        return Ok(());
    }
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
    retain_scope_relations(
        request,
        components,
        infos.len(),
        receipt.residual_glyph_positions(),
        writer.data,
        writer.scope_budget,
    )?;
    let max_glyph = font_ref
        .maxp()
        .map_err(|_| Error::new(ErrorCode::FontLoadFailed, "font lacks valid glyph bounds"))?
        .num_glyphs();
    let count = request.atoms.len();
    // Preserve the completed invocation's semantic exclusions for spacing.
    // Input ordinals are body scalars, exactly one per emitted selected atom.
    for (ordinals, kind) in [
        (
            receipt.residual_input_positions(),
            super::spacing::character::Exclusion::Residual,
        ),
        (
            receipt.deleted_input_positions(),
            super::spacing::character::Exclusion::Deleted,
        ),
    ] {
        for &ordinal in ordinals {
            let local = ordinal as usize;
            if local >= count {
                return Err(invalid_output(local, count));
            }
            if funded {
                super::scope_alignment::accounting::insert(
                    writer.scope_budget,
                    std::sync::Arc::get_mut(&mut writer.data.character_exclusions)
                        .expect("private trial exclusion owner"),
                    request.atoms.start + local,
                    kind,
                )?;
            } else {
                std::sync::Arc::make_mut(&mut writer.data.character_exclusions)
                    .insert(request.atoms.start + local, kind);
            }
        }
    }
    let zero_input =
        |local: usize, ledger: &mut super::scope_alignment::accounting::Ledger| -> Result<bool> {
            let ordinal = u32::try_from(local).expect("invocation scalar indices were admitted");
            if !funded {
                return Ok(receipt
                    .residual_input_positions()
                    .binary_search(&ordinal)
                    .is_ok()
                    || receipt
                        .deleted_input_positions()
                        .binary_search(&ordinal)
                        .is_ok());
            }
            use crate::processing::contextual::alignment::operations;
            if operations::binary_search_by(
                receipt.residual_input_positions(),
                ledger.as_mut(),
                |value| value.cmp(&ordinal),
            )?
            .is_ok()
            {
                return Ok(true);
            }
            Ok(operations::binary_search_by(
                receipt.deleted_input_positions(),
                ledger.as_mut(),
                |value| value.cmp(&ordinal),
            )?
            .is_ok())
        };
    // One grouping algorithm for native and funded Cell extraction. The actual
    // cluster-key rows are visible Vec storage, including funded sorting scratch.
    let mut groups = Vec::new();
    let mut next = 0;
    while next < infos.len() {
        if funded {
            super::scope_alignment::accounting::work(writer.scope_budget, 1)?;
        }
        let cluster = infos[next].cluster as usize;
        if cluster >= count {
            return Err(invalid_output(cluster, count));
        }
        let first = next;
        next += 1;
        while next < infos.len() {
            if funded {
                super::scope_alignment::accounting::work(writer.scope_budget, 1)?;
            }
            if infos[next].cluster as usize != cluster {
                break;
            }
            next += 1;
        }
        if funded {
            super::scope_alignment::accounting::push(
                writer.scope_budget,
                &mut groups,
                (cluster, first..next),
            )?;
        } else {
            groups.push((cluster, first..next));
        }
    }
    if funded {
        crate::processing::contextual::alignment::operations::order_by(
            &mut groups,
            writer.scope_budget.as_mut(),
            |a, b| a.0.cmp(&b.0),
        )?;
    } else {
        groups.sort_unstable_by_key(|row| row.0);
    }
    for pair in groups.windows(2) {
        if funded {
            super::scope_alignment::accounting::work(writer.scope_budget, 1)?;
        }
        if pair[0].0 == pair[1].0 {
            return Err(invalid_output(pair[0].0, pair[1].0));
        }
    }
    let mut starts: Vec<_> = if funded {
        super::scope_alignment::accounting::collect(
            writer.scope_budget,
            groups.iter().map(|row| row.0),
        )?
    } else {
        groups.iter().map(|row| row.0).collect()
    };
    for cut in [0, count] {
        if funded {
            super::scope_alignment::accounting::push(writer.scope_budget, &mut starts, cut)?;
        } else {
            starts.push(cut);
        }
    }
    if funded {
        crate::processing::contextual::alignment::operations::order_by(
            &mut starts,
            writer.scope_budget.as_mut(),
            Ord::cmp,
        )?;
        super::scope_alignment::accounting::work(writer.scope_budget, starts.len())?;
    } else {
        starts.sort_unstable();
    }
    starts.dedup();
    let run_index = writer.data.runs.len();
    let first_run_glyph = writer.data.glyphs.len();
    // Each actual provider cluster is extracted in logical order; its glyphs
    // retain the provider's visual order. Component geometry remains distinct.
    for pair in starts.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let atom_start = request.atoms.start + start;
        let atom_end = request.atoms.start + end;
        if funded {
            // Admit this separate source-span walk, including its first seed,
            // before original_extent inspects any of the actual atom rows.
            super::scope_alignment::accounting::work(writer.scope_budget, end - start)?;
        }
        let cluster_source = original_extent(&input.projection.atoms[atom_start..atom_end]);
        let first_glyph = writer.data.glyphs.len();
        let mut advance = 0.0_f64;
        let found = if funded {
            crate::processing::contextual::alignment::operations::binary_search_by(
                &groups,
                writer.scope_budget.as_mut(),
                |row| row.0.cmp(&start),
            )?
        } else {
            groups.binary_search_by_key(&start, |row| row.0)
        };
        let provider_glyphs = found
            .ok()
            .map(|index| groups[index].1.clone())
            .unwrap_or(0..0);
        for glyph_index in provider_glyphs {
            let ordinal = u32::try_from(glyph_index).map_err(|_| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "provider glyph index exceeds receipt range",
                )
            })?;
            let residual = if funded {
                crate::processing::contextual::alignment::operations::binary_search_by(
                    receipt.residual_glyph_positions(),
                    writer.scope_budget.as_mut(),
                    |value| value.cmp(&ordinal),
                )?
                .is_ok()
            } else {
                receipt
                    .residual_glyph_positions()
                    .binary_search(&ordinal)
                    .is_ok()
            };
            if residual {
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
            let glyph_advance = canonical("glyph advance", position.advance * scale)?;
            advance += f64::from(glyph_advance);
            let custody = request
                .input
                .scope_custody
                .map(|custody| {
                    scope_glyph_owner(
                        custody,
                        request.atoms.clone(),
                        components,
                        glyph_index,
                        writer.scope_budget,
                    )
                })
                .transpose()?;
            writer.push_glyph(custody, || {
                Ok(GlyphData {
                    id: info.glyph_id,
                    x: canonical("glyph x offset", position.x * scale)?,
                    y: canonical("glyph y offset", position.y * scale)?,
                    advance: glyph_advance,
                    source: cluster_source,
                    style: input.analysis.infos[atom_start].1,
                })
            })?;
        }
        let glyphs = first_glyph..writer.data.glyphs.len();
        let provider_receipt = input
            .separator_geometry
            .then_some(writer.data.provider_groups.len());
        if input.separator_geometry {
            let group = ProviderGroup {
                run: run_index,
                atoms: atom_start..atom_end,
                glyphs: glyphs.clone(),
                advance,
            };
            if funded {
                super::scope_alignment::accounting::push(
                    writer.scope_budget,
                    &mut writer.data.provider_groups,
                    group,
                )?;
            } else {
                writer.data.provider_groups.push(group);
            }
        }
        let components = end - start;
        let mut eligible_components = 0;
        let mut last_eligible = None;
        for local in start..end {
            if funded {
                super::scope_alignment::accounting::work(writer.scope_budget, 1)?;
            }
            if !zero_input(local, writer.scope_budget)? {
                eligible_components += 1;
                last_eligible = Some(local);
            }
        }
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
            let excluded = zero_input(local, writer.scope_budget)?;
            let base_advance = if excluded {
                0.0
            } else if Some(local) == last_eligible {
                advance - distributed_base
            } else {
                component_advance
            };
            distributed_base += base_advance;
            let style = &input.styles[style_index].style;

            let cluster = PreparedCluster {
                line_height: component_line_height(style, &run.metrics, normal_extent)?,
                shaped_advance: base_advance,
                logical_adjustments: [0.0, 0.0, 0.0],
                source: original.source,
                atoms: atom..atom + 1,
                provider_group: atom_start..atom_end,
                provider_receipt,
                glyphs: if atom == atom_start {
                    glyphs.clone()
                } else {
                    glyphs.end..glyphs.end
                },
                content: ClusterContent::Provider(run_index),
                style: style_index,
                level: request.level,
                advance: base_advance,
                boundary: info.boundary,
                whitespace,
                is_whitespace: whitespace != Whitespace::None,
                ligature_start: !glyphs.is_empty() && components > 1 && atom == atom_start,
                ligature_component: !glyphs.is_empty() && atom != atom_start,
            };
            if funded {
                super::scope_alignment::accounting::push(
                    writer.scope_budget,
                    &mut writer.data.clusters,
                    cluster,
                )?;
            } else {
                writer.data.clusters.push(cluster);
            }
        }
    }
    if input.is_mixed() && request.upright() && writer.data.glyphs.len() != first_run_glyph {
        let mut seen = BTreeSet::new();
        for atom in request.atoms.clone() {
            let style = input.analysis.infos[atom].1;
            if !writer.data.character_exclusions.contains_key(&atom) && seen.insert(style) {
                super::upright::admit_mixed_decoration(input, style)?;
            }
        }
    }
    if funded {
        super::scope_alignment::accounting::push(writer.scope_budget, &mut writer.data.runs, run)?;
        super::scope_alignment::accounting::drop_vec(writer.scope_budget, groups)?;
        super::scope_alignment::accounting::drop_vec(writer.scope_budget, starts)?;
    } else {
        writer.data.runs.push(run);
    }
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
        provider_receipt: None,
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
    mut input: ShapingInput<'_>,
    style_index: usize,
    ch: char,
    direction: crate::ParagraphBaseDirection,
    content: crate::UprightContent,
    scratch: &mut ShapeContext,
) -> Result<Option<(f64, PreparedRun)>> {
    // Independent hypothetical character analysis: Common uses the existing
    // isolated Latin fallback. SPACE WS follows the actual block base, while
    // ZERO EN is requested with its even/LTR resolved level.
    if input.is_mixed() {
        input.profile = if super::analysis::unit_properties::vertical_orientation(u32::from(ch))
            == icu_properties::props::VerticalOrientation::Rotated
        {
            super::upright::ShapingProfile::SidewaysClockwise
        } else {
            super::upright::ShapingProfile::Upright {
                units: &[],
                provenance: super::upright::UprightProvenance::Original,
            }
        };
    }
    let level = u8::from(direction == crate::ParagraphBaseDirection::RightToLeft);
    let direction = if input.role(0) == super::upright::RunRole::Upright {
        surgeist_shaper::Direction::TopToBottom
    } else {
        provider_direction(level)
    };
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
        &mut None,
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
    let features = provider_features(style, false, input.role(0));
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
    let advances: Vec<_> = if input.role(0) == super::upright::RunRole::Upright {
        let effective = crate::EffectiveFontInstance::qualify_synthesized(
            run.font.face(),
            run.font_size,
            &run.coords,
            run.synthesis,
        )?;
        let buffer = vertical::Operation {
            shaper: &shaper,
            plan: &plan,
            features: &features,
        }
        .shape_with_instance(
            &effective,
            input.font_catalogue,
            Script::Latin,
            || content,
            buffer,
        )?;
        let receipt = buffer
            .default_ignorable_receipt()
            .map_err(super::shaping_refused)?;
        buffer
            .glyph_positions()
            .enumerate()
            .filter_map(|(index, position)| {
                (!receipt.residual_glyph_positions().contains(&(index as u32)))
                    .then_some(-position.y_advance)
            })
            .collect()
    } else {
        let buffer = shaper.shape_with_plan(&plan, buffer, &features);
        if let surgeist_shaper::ShapingStatus::Refused(reason) = buffer.shaping_status() {
            return Err(super::shaping_refused(reason));
        }
        let receipt = buffer
            .default_ignorable_receipt()
            .map_err(super::shaping_refused)?;
        buffer
            .glyph_positions()
            .iter()
            .enumerate()
            .filter_map(|(index, position)| {
                (!receipt.residual_glyph_positions().contains(&(index as u32)))
                    .then_some(f64::from(position.x_advance))
            })
            .collect()
    };
    let advance = advances.into_iter().try_fold(0.0, |total, advance| {
        logical_add("tab character advance", total, advance)
    })? * f64::from(style.font_size)
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
    let advance = if ch == ' ' {
        super::spacing::hypothetical_space(advance, style.letter_spacing, style.word_spacing)?
    } else {
        advance
    };
    Ok(Some((advance, run)))
}
