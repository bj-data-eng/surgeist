//! Executed width candidates on the normal composition's retained instances.
use super::*;
use crate::engine::layout::data::{CellArenaOwnership, OrdinalMap};
use crate::transform::relation::ComponentOwner;
use surgeist_shaper::{WidthFeature, WidthInputOutcome, WidthObservationCoverage};

#[derive(Clone)]
struct OriginalRanges {
    runs: IndexRange<usize>,
    clusters: IndexRange<usize>,
    glyphs: IndexRange<usize>,
    providers: IndexRange<usize>,
    relations: IndexRange<usize>,
    atoms: IndexRange<usize>,
}
pub(super) struct Trial {
    pub(super) data: ShapedData,
    original: OriginalRanges,
}

struct InputRole {
    unit: usize,
    attached_mark: bool,
    default_ignorable: bool,
}
impl InputRole {
    fn inherits_width(&self) -> bool {
        self.attached_mark && !self.default_ignorable
    }
    fn donates_base(&self) -> bool {
        !self.attached_mark
    }
}

struct Lineage {
    atoms: IndexRange<usize>,
    roles: Vec<InputRole>,
    units: usize,
}
impl Lineage {
    fn new(analyzed: &AnalyzedData, input: &Input, ledger: &mut Ledger) -> Result<Self> {
        use icu_properties::{
            CodePointMapData, CodePointSetData,
            props::{DefaultIgnorableCodePoint, GeneralCategory, GeneralCategoryGroup},
        };
        let categories = CodePointMapData::<GeneralCategory>::new();
        let default_ignorables = CodePointSetData::new::<DefaultIgnorableCodePoint>();
        let is_mark = |cp| GeneralCategoryGroup::Mark.contains(categories.get32(cp));
        let output = analyzed.combination.as_ref().expect("actual source cell");
        let used = &output.plan.used[output.bindings[input.binding].group];
        let mut roles = Vec::new();
        let mut atom = input.atoms.start;
        for (unit_index, unit) in used.units.iter().enumerate() {
            let mut has_base = false;
            for &part in &used.parts[unit.clone()] {
                account::work(ledger, 1)?;
                let normal = output.plan.parts[part].normal;
                if !output.plan.visible[part] {
                    continue;
                }
                if atom >= input.atoms.end {
                    return Err(unsupported());
                }
                let emission = analyzed.transform.emission_for_atom(atom);
                if emission.selection_part != Some(part) {
                    return Err(unsupported());
                }
                let component = &analyzed.transform.components[emission.component];
                let original_mark = match component.owner {
                    ComponentOwner::Original(row) => {
                        let row = &analyzed.transform.rows[row];
                        let pieces = &analyzed.transform.pieces[row.pieces.clone()];
                        let source_pieces = &output.plan.parts[part].pieces;
                        account::work(
                            ledger,
                            pieces.len().checked_add(1).ok_or_else(work_overflow)?,
                        )?;
                        !pieces.is_empty()
                            && pieces.len() == source_pieces.len()
                            && pieces.iter().zip(source_pieces).all(|(a, b)| {
                                a.origin == b.origin
                                    && a.aggregate == b.aggregate
                                    && a.local == b.local
                            })
                            && is_mark(row.code_point)
                    }
                    ComponentOwner::Wbr(_) | ComponentOwner::Virtual(_) => false,
                };
                // Both the actual input ancestry AND completed used scalar must
                // remain marks. A transformed letter cannot inherit base action.
                let mark = original_mark && is_mark(normal);
                account::work(ledger, 1)?;
                let default_ignorable = default_ignorables.contains32(normal);
                // A surviving Normal DI root needs its own width action. Keep
                // its genuine attachment distinct from permission to inherit,
                // so a covered attached DI cannot become a false base donor.
                account::push(
                    ledger,
                    &mut roles,
                    InputRole {
                        unit: unit_index,
                        attached_mark: mark && has_base,
                        default_ignorable,
                    },
                )?;
                if !is_mark(normal) {
                    has_base = true;
                }
                atom += 1;
            }
        }
        if atom != input.atoms.end {
            return Err(unsupported());
        }
        Ok(Self {
            atoms: input.atoms.clone(),
            roles,
            units: used.units.len(),
        })
    }
    fn drop(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.roles)
    }
}

pub(in crate::engine::shape) struct Observation<'a> {
    pub(in crate::engine::shape) feature: WidthFeature,
    lineage: &'a Lineage,
    outcomes: Vec<Option<WidthInputOutcome>>,
    mark_only: Vec<bool>,
    scope: usize,
    source: Range,
}
impl Observation<'_> {
    pub(in crate::engine::shape) fn record(
        &mut self,
        atoms: IndexRange<usize>,
        glyphs: &surgeist_shaper::GlyphBuffer,
        ledger: &mut Ledger,
    ) -> Result<()> {
        let receipt = glyphs
            .width_feature_receipt()
            .map_err(crate::engine::shaping_refused)?
            .ok_or_else(|| {
                Error::new(ErrorCode::LayoutFailed, "demanded width receipt is missing")
            })?;
        if receipt.feature() != self.feature || receipt.input_len() != atoms.len() {
            return Err(Error::new(
                ErrorCode::LayoutFailed,
                "width receipt does not describe its actual invocation",
            ));
        }
        if let WidthObservationCoverage::Unqualified(path) = receipt.coverage() {
            return Err(crate::transform::combined::refused(
                crate::TextCombineError::UnqualifiedWidthObservation {
                    scope: self.scope,
                    source: self.source,
                    feature: self.feature,
                    path,
                },
            ));
        }
        let components = glyphs
            .component_receipt()
            .map_err(crate::engine::shaping_refused)?
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "width invocation lacks component custody",
                )
            })?;
        if components.input_len() != atoms.len() {
            return Err(invalid_output(components.input_len(), atoms.len()));
        }
        for local in 0..atoms.len() {
            account::work(ledger, 1)?;
            let evidence = receipt
                .input(local)
                .ok_or_else(|| invalid_output(local, atoms.len()))?;
            let index = atoms.start + local - self.lineage.atoms.start;
            if self.outcomes[index].is_some() {
                return Err(unsupported());
            }
            self.outcomes[index] = Some(evidence.outcome());
        }
        for glyph in 0..glyphs.glyph_infos().len() {
            let members = components
                .components(glyph)
                .ok_or_else(|| invalid_output(glyph, glyphs.glyph_infos().len()))?;
            account::work(
                ledger,
                members
                    .len()
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(work_overflow)?,
            )?;
            let mut marks = components.generated(glyph) == Some(false) && !members.is_empty();
            for &member in members {
                let local = member as usize;
                if local >= atoms.len() {
                    return Err(invalid_output(local, atoms.len()));
                }
                marks &= self.lineage.roles[atoms.start + local - self.lineage.atoms.start]
                    .attached_mark;
            }
            if !marks {
                for &member in members {
                    self.mark_only[atoms.start + member as usize - self.lineage.atoms.start] =
                        false;
                }
            }
        }
        Ok(())
    }
    fn covered(&self, ledger: &mut Ledger) -> Result<bool> {
        let mut bases = account::filled(ledger, self.lineage.units, false)?;
        let mut positive = false;
        for (index, outcome) in self.outcomes.iter().enumerate() {
            account::work(ledger, 1)?;
            let outcome = outcome.ok_or_else(|| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "incomplete width invocation inventory",
                )
            })?;
            if outcome == WidthInputOutcome::Covered {
                positive = true;
                if self.lineage.roles[index].donates_base() {
                    bases[self.lineage.roles[index].unit] = true;
                }
            }
        }
        let mut covered = positive;
        for (index, outcome) in self.outcomes.iter().enumerate() {
            account::work(ledger, 1)?;
            covered &= matches!(
                outcome,
                Some(WidthInputOutcome::Covered | WidthInputOutcome::NoSurvivingOperand)
            ) || (self.lineage.roles[index].inherits_width()
                && self.mark_only[index]
                && bases[self.lineage.roles[index].unit]);
        }
        account::drop_vec(ledger, bases)?;
        Ok(covered)
    }
    fn drop(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.outcomes)?;
        account::drop_vec(ledger, self.mark_only)
    }
}
fn work_overflow() -> Error {
    scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
        kind: crate::ScopeAlignmentLimitKind::StructuralWork,
    })
}

pub(super) fn choose(
    normal: &ShapedData,
    input: &Input,
    normal_payload: CellPayload,
    analyzed: &AnalyzedData,
    horizontal: &ShapingInput<'_>,
    scratch: &mut ShapeContext,
    ledger: &mut Ledger,
) -> Result<(CellPayload, Option<Trial>)> {
    let CellPayload::Shaped { advance, runs, .. } = &normal_payload else {
        return Ok((normal_payload, None));
    };
    if advance.end - advance.start <= input.extent {
        return Ok((normal_payload, None));
    }
    let CellPayload::Shaped {
        runs: original_runs,
        clusters,
        glyphs,
        scope_relations,
        ..
    } = &normal_payload
    else {
        unreachable!("measured actual provider input")
    };
    let provider_start =
        operations::partition_point(&normal.provider_groups, ledger.as_mut(), |row| {
            row.run < original_runs.start
        })?;
    let provider_end =
        operations::partition_point(&normal.provider_groups, ledger.as_mut(), |row| {
            row.run < original_runs.end
        })?;
    let original = OriginalRanges {
        runs: original_runs.clone(),
        clusters: clusters.clone(),
        glyphs: glyphs.clone(),
        providers: provider_start..provider_end,
        relations: scope_relations.clone(),
        atoms: input.atoms.clone(),
    };
    let lineage = Lineage::new(analyzed, input, ledger)?;
    let output = analyzed.combination.as_ref().expect("actual cell plan");
    let group = &output.plan.groups[output.bindings[input.binding].group];
    account::work(ledger, group.parts.len())?;
    let source = group
        .parts
        .clone()
        .fold(None, |hull: Option<Range>, p| {
            let r = output.plan.parts[p].source;
            Some(hull.map_or(r, |old| {
                Range::new(old.start.min(r.start), old.end.max(r.end))
            }))
        })
        .expect("selected cell has actual parts");
    let mut best: Option<(CellPayload, Trial)> = None;
    for feature in [
        WidthFeature::HalfWidth,
        WidthFeature::ThirdWidth,
        WidthFeature::QuarterWidth,
    ] {
        account::header::<OrdinalMap<crate::engine::spacing::character::Exclusion>>(ledger)?;
        let mut data = ShapedData {
            cell_arenas: CellArenaOwnership::Owned,
            ..ShapedData::default()
        };
        let mut observation = Observation {
            feature,
            lineage: &lineage,
            outcomes: account::filled(ledger, input.atoms.len(), None)?,
            mark_only: account::filled(ledger, input.atoms.len(), true)?,
            scope: group.scope,
            source,
        };
        for run in &normal.runs[runs.clone()] {
            super::super::append_width_run(
                horizontal,
                run,
                input.atoms.clone(),
                &mut observation,
                scratch,
                &mut data,
                ledger,
            )?;
        }
        let covered = observation.covered(ledger)?;
        observation.drop(ledger)?;
        if !covered {
            drop_trial(
                Trial {
                    data,
                    original: original.clone(),
                },
                ledger,
            )?;
            continue;
        }
        let trial_input = Input {
            binding: input.binding,
            metrics: input.metrics,
            level: input.level,
            extent: input.extent,
            before: input.before,
            after: input.after,
            atoms: input.atoms.clone(),
            events: input.events.clone(),
            scope_relations: Some(0..data.scope_relations.len()),
        };
        let payload = super::normal_payload(&data, &trial_input, ledger)?;
        let width = payload_width(&payload);
        let better = best.as_ref().is_none_or(|(old, _)| {
            let previous = payload_width(old);
            match (width <= input.extent, previous <= input.extent) {
                (true, true) => width > previous,
                (true, false) => true,
                (false, true) => false,
                (false, false) => width < previous,
            }
        });
        if better {
            if let Some((old, trial)) = best.take() {
                drop_payload(old, ledger)?;
                drop_trial(trial, ledger)?;
            }
            best = Some((
                payload,
                Trial {
                    data,
                    original: original.clone(),
                },
            ));
        } else {
            drop_payload(payload, ledger)?;
            drop_trial(
                Trial {
                    data,
                    original: original.clone(),
                },
                ledger,
            )?;
        }
    }
    lineage.drop(ledger)?;
    if let Some((mut payload, trial)) = best {
        drop_payload(normal_payload, ledger)?;
        compress(&mut payload, input.extent)?;
        Ok((payload, Some(trial)))
    } else {
        let mut payload = normal_payload;
        compress(&mut payload, input.extent)?;
        Ok((payload, None))
    }
}
fn payload_width(payload: &CellPayload) -> f64 {
    match payload {
        CellPayload::Shaped { advance, .. } => advance.end - advance.start,
        CellPayload::NoProviderInput => 0.0,
    }
}
fn compress(payload: &mut CellPayload, extent: f64) -> Result<()> {
    if let CellPayload::Shaped { advance, frame, .. } = payload {
        let width = logical_add("cell occupied advance", advance.end, -advance.start)?;
        let scale = if width > extent { extent / width } else { 1.0 };
        frame.x_scale = scale;
        frame.x_offset = logical_add(
            "cell horizontal centering",
            (extent - scale * width) / 2.0,
            -scale * advance.start,
        )?;
    }
    Ok(())
}
fn drop_payload(payload: CellPayload, ledger: &mut Ledger) -> Result<()> {
    if let CellPayload::Shaped { positions, .. } = payload {
        account::drop_vec(ledger, positions)?;
    }
    Ok(())
}
pub(super) fn drop_trial(trial: Trial, ledger: &mut Ledger) -> Result<()> {
    let ShapedData {
        runs,
        clusters,
        glyphs,
        provider_groups,
        scope_glyph_custody,
        scope_relations,
        character_exclusions,
        ..
    } = trial.data;
    let mut runs = runs;
    for row in runs.drain(..) {
        account::drop_vec(ledger, row.coords)?;
    }
    account::drop_vec(ledger, runs)?;
    account::drop_vec(ledger, clusters)?;
    account::drop_vec(ledger, glyphs)?;
    account::drop_vec(ledger, provider_groups)?;
    account::drop_vec(ledger, scope_glyph_custody)?;
    account::drop_vec(ledger, scope_relations)?;
    let map = std::sync::Arc::try_unwrap(character_exclusions).expect("private trial exclusions");
    account::drop_vec(ledger, map.rows)?;
    if let Some(budget) = ledger {
        budget.release_bytes(std::mem::size_of::<
            OrdinalMap<crate::engine::spacing::character::Exclusion>,
        >())?;
    }
    Ok(())
}

struct Rebuilt<T> {
    rows: Vec<T>,
    old: Vec<Option<usize>>,
    cuts: Vec<Option<usize>>,
    replacements: Vec<Option<IndexRange<usize>>>,
}
impl<T> Rebuilt<T> {
    fn index(&self, old: usize) -> Result<usize> {
        self.old.get(old).copied().flatten().ok_or_else(|| {
            Error::new(
                ErrorCode::LayoutFailed,
                "reference enters replaced native payload",
            )
        })
    }
    fn range(&self, range: IndexRange<usize>) -> Result<IndexRange<usize>> {
        if range.is_empty() {
            let cut = self
                .cuts
                .get(range.start)
                .copied()
                .flatten()
                .ok_or_else(|| {
                    Error::new(ErrorCode::LayoutFailed, "missing native payload frontier")
                })?;
            Ok(cut..cut)
        } else {
            Ok(self.index(range.start)?..self.index(range.end - 1)? + 1)
        }
    }
    fn trial(&self, trial: usize) -> Result<IndexRange<usize>> {
        self.replacements[trial]
            .clone()
            .ok_or_else(|| Error::new(ErrorCode::LayoutFailed, "missing winning payload slice"))
    }
    fn drop_maps(self, ledger: &mut Ledger) -> Result<Vec<T>> {
        account::drop_vec(ledger, self.old)?;
        account::drop_vec(ledger, self.cuts)?;
        account::drop_vec(ledger, self.replacements)?;
        Ok(self.rows)
    }
}
fn rebuild<T>(
    mut previous: Vec<T>,
    trials: &mut [Option<Trial>],
    original: fn(&Trial) -> IndexRange<usize>,
    take: fn(&mut Trial) -> Vec<T>,
    mut map: impl FnMut(T, Option<usize>) -> Result<T>,
    mut discard: impl FnMut(T, &mut Ledger) -> Result<()>,
    ledger: &mut Ledger,
) -> Result<Rebuilt<T>> {
    let mut result = Rebuilt {
        rows: Vec::new(),
        old: account::filled(ledger, previous.len(), None)?,
        cuts: account::filled(
            ledger,
            previous.len().checked_add(1).ok_or_else(work_overflow)?,
            None,
        )?,
        replacements: account::filled(ledger, trials.len(), None)?,
    };
    {
        let mut source = previous.drain(..).enumerate().peekable();
        let mut cursor = 0;
        for (index, trial) in trials.iter_mut().enumerate() {
            let Some(trial) = trial else {
                continue;
            };
            let range = original(trial);
            if range.start < cursor || range.end < range.start || range.end > result.old.len() {
                return Err(unsupported());
            }
            while cursor < range.start {
                let (old, value) = source.next().ok_or_else(unsupported)?;
                result.cuts[old] = Some(result.rows.len());
                result.old[old] = Some(result.rows.len());
                account::work(ledger, 8)?; // bounded fixed index rewrites before the move
                account::push(ledger, &mut result.rows, map(value, None)?)?;
                cursor += 1;
            }
            result.cuts[range.start] = Some(result.rows.len());
            while cursor < range.end {
                let (_, value) = source.next().ok_or_else(unsupported)?;
                account::work(ledger, 1)?;
                discard(value, ledger)?;
                cursor += 1;
            }
            let start = result.rows.len();
            let mut replacement = take(trial);
            for value in replacement.drain(..) {
                account::work(ledger, 8)?;
                account::push(ledger, &mut result.rows, map(value, Some(index))?)?;
            }
            account::drop_vec(ledger, replacement)?;
            result.replacements[index] = Some(start..result.rows.len());
            result.cuts[range.end] = Some(result.rows.len());
        }
        for (old, value) in source {
            result.cuts[old] = Some(result.rows.len());
            result.old[old] = Some(result.rows.len());
            account::work(ledger, 8)?;
            account::push(ledger, &mut result.rows, map(value, None)?)?;
        }
    }
    result.cuts[result.old.len()] = Some(result.rows.len());
    account::drop_vec(ledger, previous)?;
    Ok(result)
}
fn mapped_run(index: usize, trial: Option<usize>, runs: &Rebuilt<PreparedRun>) -> Result<usize> {
    if let Some(trial) = trial {
        Ok(runs.trial(trial)?.start + index)
    } else {
        runs.index(index)
    }
}
fn mapped_range<T>(
    range: IndexRange<usize>,
    trial: Option<usize>,
    rows: &Rebuilt<T>,
) -> Result<IndexRange<usize>> {
    if let Some(trial) = trial {
        let start = rows.trial(trial)?.start;
        Ok(start + range.start..start + range.end)
    } else {
        rows.range(range)
    }
}

/// One consuming assembly in canonical run/atom order. No obsolete normal or
/// losing-trial rows remain, and every shared backing has one actual owner.
pub(super) fn assemble(
    output: &mut ShapedData,
    trials: &mut [Option<Trial>],
    ledger: &mut Ledger,
) -> Result<()> {
    if !trials.iter().any(Option::is_some) {
        return Ok(());
    }
    let runs = rebuild(
        std::mem::take(&mut output.runs),
        trials,
        |t| t.original.runs.clone(),
        |t| std::mem::take(&mut t.data.runs),
        |row, _| Ok(row),
        |row, ledger| account::drop_vec(ledger, row.coords),
        ledger,
    )?;
    let glyphs = rebuild(
        std::mem::take(&mut output.glyphs),
        trials,
        |t| t.original.glyphs.clone(),
        |t| std::mem::take(&mut t.data.glyphs),
        |row, _| Ok(row),
        |_, _| Ok(()),
        ledger,
    )?;
    let custody = if output.scope_glyph_custody.is_empty()
        && trials
            .iter()
            .flatten()
            .all(|trial| trial.data.scope_glyph_custody.is_empty())
    {
        None
    } else {
        Some(rebuild(
            std::mem::take(&mut output.scope_glyph_custody),
            trials,
            |t| t.original.glyphs.clone(),
            |t| std::mem::take(&mut t.data.scope_glyph_custody),
            |row, _| Ok(row),
            |_, _| Ok(()),
            ledger,
        )?)
    };
    let providers = rebuild(
        std::mem::take(&mut output.provider_groups),
        trials,
        |t| t.original.providers.clone(),
        |t| std::mem::take(&mut t.data.provider_groups),
        |mut row, trial| {
            row.run = mapped_run(row.run, trial, &runs)?;
            row.glyphs = mapped_range(row.glyphs, trial, &glyphs)?;
            Ok(row)
        },
        |_, _| Ok(()),
        ledger,
    )?;
    let clusters = rebuild(
        std::mem::take(&mut output.clusters),
        trials,
        |t| t.original.clusters.clone(),
        |t| std::mem::take(&mut t.data.clusters),
        |mut row, trial| {
            row.content = match row.content {
                ClusterContent::Provider(run) => {
                    ClusterContent::Provider(mapped_run(run, trial, &runs)?)
                }
                ClusterContent::CssTab(run) => ClusterContent::CssTab(
                    run.map(|run| mapped_run(run, trial, &runs)).transpose()?,
                ),
            };
            row.glyphs = mapped_range(row.glyphs, trial, &glyphs)?;
            row.provider_receipt = row
                .provider_receipt
                .map(|index| {
                    if let Some(trial) = trial {
                        Ok(providers.trial(trial)?.start + index)
                    } else {
                        providers.index(index)
                    }
                })
                .transpose()?;
            Ok(row)
        },
        |_, _| Ok(()),
        ledger,
    )?;
    let relations = rebuild(
        std::mem::take(&mut output.scope_relations),
        trials,
        |t| t.original.relations.clone(),
        |t| std::mem::take(&mut t.data.scope_relations),
        |row, _| Ok(row),
        |_, _| Ok(()),
        ledger,
    )?;
    for invocation in &mut output.character_invocations {
        account::work(ledger, 1)?;
        invocation.run = runs.index(invocation.run)?;
        for constraint in &mut invocation.constraints {
            account::work(ledger, 1)?;
            constraint.run = runs.index(constraint.run)?;
        }
    }
    for (index, cell) in output.cells.iter_mut().enumerate() {
        let trial = trials[index].is_some().then_some(index);
        if let CellPayload::Shaped {
            runs: cell_runs,
            clusters: cell_clusters,
            glyphs: cell_glyphs,
            scope_relations,
            positions,
            ..
        } = &mut cell.payload
        {
            account::work(ledger, 4)?;
            *cell_runs = mapped_range(cell_runs.clone(), trial, &runs)?;
            *cell_clusters = mapped_range(cell_clusters.clone(), trial, &clusters)?;
            *cell_glyphs = mapped_range(cell_glyphs.clone(), trial, &glyphs)?;
            *scope_relations = mapped_range(scope_relations.clone(), trial, &relations)?;
            for position in positions {
                account::work(ledger, 1)?;
                position.cluster = if let Some(trial) = trial {
                    clusters.trial(trial)?.start + position.cluster
                } else {
                    clusters.index(position.cluster)?
                };
            }
        }
    }
    // The ordinary and retained normal entries stay canonical; replaced normal
    // invocation contributions are removed, not unioned with the executed winner.
    let original_map = std::sync::Arc::get_mut(&mut output.character_exclusions)
        .expect("unpublished preparation owns its exclusions");
    let mut previous = std::mem::take(&mut original_map.rows);
    let mut rows = Vec::new();
    {
        let mut old = previous.drain(..).peekable();
        for trial in trials.iter_mut().flatten() {
            let atoms = trial.original.atoms.clone();
            while old.peek().is_some_and(|row| row.0 < atoms.start) {
                account::push(ledger, &mut rows, old.next().expect("observed actual key"))?;
            }
            while old.peek().is_some_and(|row| row.0 < atoms.end) {
                account::work(ledger, 1)?;
                old.next();
            }
            let map = std::sync::Arc::get_mut(&mut trial.data.character_exclusions)
                .expect("private trial exclusions");
            for row in map.rows.drain(..) {
                if !atoms.contains(&row.0) {
                    return Err(unsupported());
                }
                account::push(ledger, &mut rows, row)?;
            }
        }
        for row in old {
            account::push(ledger, &mut rows, row)?;
        }
    }
    account::drop_vec(ledger, previous)?;
    original_map.rows = rows;
    output.runs = runs.drop_maps(ledger)?;
    output.glyphs = glyphs.drop_maps(ledger)?;
    if let Some(custody) = custody {
        output.scope_glyph_custody = custody.drop_maps(ledger)?;
    }
    output.provider_groups = providers.drop_maps(ledger)?;
    output.clusters = clusters.drop_maps(ledger)?;
    output.scope_relations = relations.drop_maps(ledger)?;
    for trial in trials.iter_mut().filter_map(Option::take) {
        drop_trial(trial, ledger)?;
    }
    Ok(())
}
