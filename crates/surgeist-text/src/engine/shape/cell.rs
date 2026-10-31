//! Source-owned Cell admission and retained executed horizontal payloads.

pub(super) mod width;

use super::*;
use crate::engine::layout::data::{CellClusterPosition, CellFrame, CellPayload, PreparedCell};
use crate::engine::spacing::{CellWordExecution, CellWordPlan, SeparatorCandidate};
use crate::engine::{AnalyzedData, scope_alignment};
use crate::processing::contextual::alignment::operations;
use crate::protocol::paragraph::SemanticAnchorItem;
use crate::transform::relation::RelationEvent;
use scope_alignment::accounting::{self as account, Ledger};

pub(super) fn horizontal_input<'a>(input: &ShapingInput<'a>) -> ShapingInput<'a> {
    ShapingInput {
        formatting_breaks: input.formatting_breaks,
        scope_custody: input.scope_custody,
        separator_geometry: input.separator_geometry,
        characters: None,
        profile: crate::engine::upright::ShapingProfile::Horizontal,
        baseline_reference: Some(crate::FontBaseline::Alphabetic),
        projection: input.projection,
        analysis: input.analysis,
        styles: input.styles,
        boxes: input.boxes,
        font_catalogue: input.font_catalogue,
        paragraph: input.paragraph,
        css_tabs: input.css_tabs,
    }
}

fn unsupported() -> Error {
    crate::transform::combined::refused(crate::TextCombineError::UnsupportedCellPreparation)
}

/// Bounded native capability. Empty provider input is established from actual
/// source presentation and control roles, never from font/glyph/ink outcomes.
pub(super) fn admit(analyzed: &AnalyzedData, tabs: crate::TabPreparation) -> Result<()> {
    let Some(output) = &analyzed.combination else {
        return Ok(());
    };
    for binding in &output.bindings {
        if tabs == crate::TabPreparation::CssStops
            && analyzed.projection.atoms[binding.atoms.clone()]
                .iter()
                .any(|atom| atom.code_point == 0x09)
        {
            return Err(unsupported());
        }
        // Source has already resolved inner directional controls into used
        // levels and removed literal control scalars from provider input.
        for event in &analyzed.transform.events[binding.events.clone()] {
            if let RelationEvent::Anchor { order } = event {
                match analyzed.paragraph.anchor_order[*order].item {
                    SemanticAnchorItem::Control(control) => {
                        let kind = &analyzed.paragraph.controls[control].kind;
                        if matches!(
                            kind,
                            crate::ParagraphControlKind::InlineBoundaryStart
                                | crate::ParagraphControlKind::InlineBoundaryEnd
                        ) {
                            return Err(unsupported());
                        }
                    }
                    SemanticAnchorItem::Atomic(_) => return Err(unsupported()),
                    SemanticAnchorItem::ComputedLineBoundary(_)
                    | SemanticAnchorItem::EmptyOrigin(_) => (),
                }
            }
        }
    }
    Ok(())
}

struct Input {
    binding: usize,
    metrics: usize,
    level: u8,
    extent: f64,
    before: f64,
    after: f64,
    atoms: IndexRange<usize>,
    events: IndexRange<usize>,
    scope_relations: Option<IndexRange<usize>>,
}

pub(super) struct Inputs {
    rows: Vec<Input>,
    atom_cells: Vec<Option<usize>>,
    levels: Vec<Option<u8>>,
    atom_events: Vec<Option<usize>>,
    box_events: Vec<Option<(usize, u8)>>,
    boundaries: Vec<usize>,
}

impl Inputs {
    pub(super) fn new(
        analyzed: &AnalyzedData,
        scopes: &scope_alignment::PreparedScopes,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let output = analyzed.combination.as_ref().expect("actual source cells");
        let mut rows = Vec::new();
        let mut atom_cells = account::filled(ledger, analyzed.projection.atoms.len(), None)?;
        let mut levels = account::filled(ledger, analyzed.projection.atoms.len(), None)?;
        let mut atom_events = account::filled(ledger, analyzed.projection.atoms.len(), None)?;
        let mut box_events =
            account::filled(ledger, analyzed.processed.source().boxes().len(), None)?;
        for (event, row) in analyzed.transform.events.iter().enumerate() {
            account::work(ledger, 1)?;
            match row {
                RelationEvent::Text { atom } => atom_events[*atom] = Some(event),
                RelationEvent::Anchor { order } => {
                    if let SemanticAnchorItem::Atomic(index) =
                        analyzed.paragraph.anchor_order[*order].item
                    {
                        box_events[index] = Some((event, analyzed.analysis.box_levels[index]));
                    }
                }
                _ => (),
            }
        }
        let mut boundaries = Vec::new();
        for (binding_index, binding) in output.bindings.iter().enumerate() {
            account::work(ledger, 1)?;
            let group = &output.plan.groups[binding.group];
            let used = &output.plan.used[binding.group];
            let metrics = scope_alignment::cell_metric_index(scopes, group.scope, ledger)?;
            let row = &scopes.metrics[metrics];
            let extent = f64::from(row.font_size);
            // over/under are signed DISTANCES from the actual native reference.
            let midpoint = super::super::layout::add(
                "cell font midpoint",
                TextNumericPhase::Preparation,
                row.over / 2.0,
                -row.under / 2.0,
            )?;
            let before = super::super::layout::add(
                "cell before",
                TextNumericPhase::Preparation,
                extent / 2.0,
                midpoint,
            )?;
            let after = super::super::layout::add(
                "cell after",
                TextNumericPhase::Preparation,
                extent / 2.0,
                -midpoint,
            )?;
            let cell = rows.len();
            let mut atom = binding.atoms.start;
            for (&part, &level) in used.parts.iter().zip(&used.levels) {
                account::work(ledger, 1)?;
                if !output.plan.visible[part] {
                    continue;
                }
                if atom >= binding.atoms.end
                    || analyzed.transform.emission_for_atom(atom).selection_part != Some(part)
                {
                    return Err(unsupported());
                }
                atom_cells[atom] = Some(cell);
                levels[atom] = Some(level);
                atom += 1;
            }
            if atom != binding.atoms.end {
                return Err(unsupported());
            }
            account::push(ledger, &mut boundaries, binding.atoms.start)?;
            account::push(ledger, &mut boundaries, binding.atoms.end)?;
            account::push(
                ledger,
                &mut rows,
                Input {
                    binding: binding_index,
                    metrics,
                    level: analyzed.analysis.cell_levels[binding_index],
                    extent,
                    before,
                    after,
                    atoms: binding.atoms.clone(),
                    events: binding.events.clone(),
                    scope_relations: None,
                },
            )?;
        }
        Ok(Self {
            rows,
            atom_cells,
            levels,
            atom_events,
            box_events,
            boundaries,
        })
    }

    pub(super) fn owner(&self, atom: usize) -> Option<usize> {
        self.atom_cells[atom]
    }

    pub(super) fn first_script(
        &self,
        cell: usize,
        analysis: &crate::engine::analysis::Analysis,
        ledger: &mut Ledger,
    ) -> Result<Script> {
        for atom in self.rows[cell].atoms.clone() {
            account::work(ledger, 1)?;
            let script = analysis.infos[atom].0.script;
            if real_script(script) {
                return Ok(script);
            }
        }
        Ok(Script::Latin)
    }

    pub(super) fn begin(&mut self, cell: usize, relations: usize) {
        self.rows[cell]
            .scope_relations
            .get_or_insert(relations..relations);
    }

    pub(super) fn end(&mut self, cell: usize, relations: usize) {
        self.rows[cell]
            .scope_relations
            .as_mut()
            .expect("actual cell operation")
            .end = relations;
    }

    pub(super) fn level(&self, atom: usize, ordinary: u8) -> u8 {
        self.levels[atom].unwrap_or(ordinary)
    }

    pub(super) fn boundary(&self, atom: usize, ledger: &mut Ledger) -> Result<bool> {
        crate::processing::contextual::alignment::operations::binary_search_by(
            &self.boundaries,
            ledger.as_mut(),
            |cut| cut.cmp(&atom),
        )
        .map(|found| found.is_ok())
    }

    pub(super) fn context(
        &self,
        atom: usize,
        count: usize,
        ledger: &mut Ledger,
    ) -> Result<IndexRange<usize>> {
        if let Some(cell) = self.owner(atom) {
            return Ok(self.rows[cell].atoms.clone());
        }
        let next = crate::processing::contextual::alignment::operations::partition_point(
            &self.boundaries,
            ledger.as_mut(),
            |cut| *cut <= atom,
        )?;
        Ok(next
            .checked_sub(1)
            .and_then(|index| self.boundaries.get(index))
            .copied()
            .unwrap_or(0)..self.boundaries.get(next).copied().unwrap_or(count))
    }

    pub(super) fn finish(
        self,
        output: &mut ShapedData,
        analyzed: &AnalyzedData,
        horizontal: &ShapingInput<'_>,
        scratch: &mut ShapeContext,
        scopes: &scope_alignment::PreparedScopes,
        ledger: &mut Ledger,
    ) -> Result<()> {
        let words = CellWordPlan::new(
            &analyzed.projection,
            &analyzed.analysis,
            horizontal.styles,
            &self.atom_cells,
            self.rows.len(),
            ledger,
        )?;
        let mut trials = Vec::new();
        account::header::<Vec<PreparedCell>>(ledger)?;
        for (index, row) in self.rows.iter().enumerate() {
            account::work(ledger, 1)?;
            let composition = Composition {
                analyzed,
                horizontal,
                words: words.for_cell(index),
            };
            let payload = if row.atoms.is_empty() {
                CellPayload::NoProviderInput
            } else {
                normal_payload(output, row, &composition, ledger)?
            };
            let overwide = match &payload {
                CellPayload::Shaped { advance, .. } => advance.end - advance.start > row.extent,
                CellPayload::NoProviderInput => false,
            };
            if overwide {
                account::adopt_prepared_cell_arenas(ledger, scopes, output)?;
                if trials.is_empty() {
                    trials = account::collect(ledger, (0..self.rows.len()).map(|_| None))?;
                }
            }
            let (payload, trial) = if overwide {
                width::choose(output, row, payload, &composition, scratch, ledger)?
            } else {
                (payload, None)
            };
            if !trials.is_empty() {
                trials[index] = trial;
            }
            account::push(
                ledger,
                &mut output.cells,
                PreparedCell {
                    binding: row.binding,
                    metrics: row.metrics,
                    level: row.level,
                    extent: row.extent,
                    advance: row.extent,
                    before: row.before,
                    after: row.after,
                    payload,
                },
            )?;
        }
        words.drop(ledger)?;
        width::assemble(output, &mut trials, ledger)?;
        account::drop_vec(ledger, trials)?;
        // The cell-active branch did not allocate ordinary mixed items. Build
        // this actual outer arena under its owner before its first reserve.
        account::header::<Vec<ShapedItem>>(ledger)?;
        for (cluster, value) in output.clusters.iter().enumerate() {
            account::work(ledger, 1)?;
            if self.owner(value.atoms.start).is_none() {
                account::push(ledger, &mut output.items, ShapedItem::TextCluster(cluster))?;
            }
        }
        for (occurrence, value) in self.box_events.iter().enumerate() {
            account::work(ledger, 1)?;
            if let Some((_, level)) = *value {
                account::push(
                    ledger,
                    &mut output.items,
                    ShapedItem::InlineBox { occurrence, level },
                )?;
            }
        }
        for cell in 0..output.cells.len() {
            account::push(ledger, &mut output.items, ShapedItem::Cell(cell))?;
        }
        let key = |item: &ShapedItem| match *item {
            ShapedItem::TextCluster(cluster) => self.atom_events
                [output.clusters[cluster].atoms.start]
                .expect("actual source text event"),
            ShapedItem::Cell(cell) => self.rows[cell].events.start,
            ShapedItem::InlineBox { occurrence, .. } => {
                self.box_events[occurrence].expect("actual atomic event").0
            }
            ShapedItem::Anchor { .. } => {
                unreachable!("original shaping does not manufacture selected anchors")
            }
        };
        crate::processing::contextual::alignment::operations::order_by(
            &mut output.items,
            ledger.as_mut(),
            |a, b| key(a).cmp(&key(b)),
        )?;
        account::drop_vec(ledger, self.rows)?;
        account::drop_vec(ledger, self.atom_cells)?;
        account::drop_vec(ledger, self.levels)?;
        account::drop_vec(ledger, self.atom_events)?;
        account::drop_vec(ledger, self.box_events)?;
        account::drop_vec(ledger, self.boundaries)
    }
}

/// Actual selected Cell ownership, separate from its scalar presentation.
pub(super) struct SelectedCells {
    owners: Vec<Option<usize>>,
    retained: Vec<Option<usize>>,
}

impl SelectedCells {
    pub(super) fn new(
        prepared: &super::super::PreparedData,
        presentation: &crate::protocol::prepared::SelectedPresentation,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        use crate::protocol::prepared::PresentationUnitKind;
        if !presentation
            .units
            .iter()
            .any(|unit| matches!(unit.kind, PresentationUnitKind::Cell { .. }))
        {
            return Ok(Self {
                owners: Vec::new(),
                retained: Vec::new(),
            });
        }
        let mut owners = account::filled(ledger, presentation.projection.atoms.len(), None)?;
        for unit in &presentation.units {
            if let PresentationUnitKind::Cell { cell } = unit.kind {
                let source = &prepared.shaped.cells[cell];
                let binding = &prepared
                    .analyzed
                    .combination
                    .as_ref()
                    .expect("actual cell output")
                    .bindings[source.binding];
                if binding.atoms.len() != unit.atoms.len() {
                    return Err(unsupported());
                }
                for (atom, original) in unit.atoms.clone().zip(binding.atoms.clone()) {
                    account::work(ledger, 1)?;
                    if presentation.originals[atom] != Some(original) || owners[atom].is_some() {
                        return Err(unsupported());
                    }
                    owners[atom] = Some(cell);
                }
            }
        }
        Ok(Self {
            owners,
            retained: account::filled(ledger, prepared.shaped.cells.len(), None)?,
        })
    }

    pub(super) fn assignment(&self, atoms: IndexRange<usize>, ledger: &mut Ledger) -> Result<bool> {
        if self.owners.is_empty() {
            return Ok(false);
        }
        let mut owner = None;
        let mut ordinary = false;
        for atom in atoms {
            account::work(ledger, 1)?;
            if let Some(cell) = self.owners[atom] {
                if ordinary || owner.is_some_and(|old| old != cell) {
                    return Err(unsupported());
                }
                owner = Some(cell);
            } else {
                if owner.is_some() {
                    return Err(unsupported());
                }
                ordinary = true;
            }
        }
        Ok(owner.is_some())
    }

    pub(super) fn retain(
        &mut self,
        prepared: &super::super::PreparedData,
        presentation: &crate::protocol::prepared::SelectedPresentation,
        output: &mut ShapedData,
        ledger: &mut Ledger,
    ) -> Result<()> {
        use crate::protocol::prepared::PresentationUnitKind;
        if self.retained.is_empty() {
            return Ok(());
        }
        // Transfer the actual preexisting native shared backing once, before
        // any cell copy can reuse it or grow it. Its later census is shared
        // with this exact admission owner; no fictional per-cell suballocation.
        account::adopt_cell_arenas(ledger, output)?;
        if prepared.shaped.cell_arenas == crate::engine::layout::data::CellArenaOwnership::Owned {
            account::adopt_cell_exclusions(ledger, output)?;
            output.cell_arenas = crate::engine::layout::data::CellArenaOwnership::Owned;
        }
        account::header::<Vec<PreparedCell>>(ledger)?;
        for unit in &presentation.units {
            let PresentationUnitKind::Cell { cell } = unit.kind else {
                continue;
            };
            if self.retained[cell].is_some() {
                return Err(unsupported());
            }
            let original = &prepared.shaped.cells[cell];
            let binding = &prepared
                .analyzed
                .combination
                .as_ref()
                .expect("actual source cell")
                .bindings[original.binding];
            let shift = |range: IndexRange<usize>| -> Result<IndexRange<usize>> {
                if range.start < binding.atoms.start || range.end > binding.atoms.end {
                    return Err(unsupported());
                }
                Ok(unit.atoms.start + range.start - binding.atoms.start
                    ..unit.atoms.start + range.end - binding.atoms.start)
            };
            let payload = match &original.payload {
                CellPayload::NoProviderInput => CellPayload::NoProviderInput,
                CellPayload::Shaped {
                    runs,
                    clusters,
                    glyphs,
                    scope_relations,
                    advance,
                    positions,
                    frame,
                } => {
                    let run_base = output.runs.len();
                    for old in &prepared.shaped.runs[runs.clone()] {
                        let mut run = account::clone_run(ledger, old)?;
                        run.atoms = shift(run.atoms)?;
                        account::push(ledger, &mut output.runs, run)?;
                    }
                    let glyph_base = output.glyphs.len();
                    for glyph in &prepared.shaped.glyphs[glyphs.clone()] {
                        account::push(ledger, &mut output.glyphs, *glyph)?;
                    }
                    if !prepared.shaped.scope_glyph_custody.is_empty() {
                        for custody in &prepared.shaped.scope_glyph_custody[glyphs.clone()] {
                            account::push(ledger, &mut output.scope_glyph_custody, *custody)?;
                        }
                    }
                    let relation_base = output.scope_relations.len();
                    for relation in &prepared.shaped.scope_relations[scope_relations.clone()] {
                        account::push(ledger, &mut output.scope_relations, *relation)?;
                    }
                    let provider_first = operations::partition_point(
                        &prepared.shaped.provider_groups,
                        ledger.as_mut(),
                        |group| group.run < runs.start,
                    )?;
                    let provider_last = operations::partition_point(
                        &prepared.shaped.provider_groups,
                        ledger.as_mut(),
                        |group| group.run < runs.end,
                    )?;
                    let provider_base = output.provider_groups.len();
                    for group in &prepared.shaped.provider_groups[provider_first..provider_last] {
                        account::push(
                            ledger,
                            &mut output.provider_groups,
                            ProviderGroup {
                                run: run_base + group.run - runs.start,
                                atoms: shift(group.atoms.clone())?,
                                glyphs: glyph_base + group.glyphs.start - glyphs.start
                                    ..glyph_base + group.glyphs.end - glyphs.start,
                                advance: group.advance,
                            },
                        )?;
                    }
                    let cluster_base = output.clusters.len();
                    for old in &prepared.shaped.clusters[clusters.clone()] {
                        let mut cluster = old.clone();
                        cluster.atoms = shift(cluster.atoms)?;
                        cluster.provider_group = shift(cluster.provider_group)?;
                        cluster.glyphs = glyph_base + cluster.glyphs.start - glyphs.start
                            ..glyph_base + cluster.glyphs.end - glyphs.start;
                        cluster.content = match cluster.content {
                            ClusterContent::Provider(run) => {
                                ClusterContent::Provider(run_base + run - runs.start)
                            }
                            ClusterContent::CssTab(run) => {
                                ClusterContent::CssTab(run.map(|run| run_base + run - runs.start))
                            }
                        };
                        cluster.provider_receipt = cluster
                            .provider_receipt
                            .map(|receipt| provider_base + receipt - provider_first);
                        account::push(ledger, &mut output.clusters, cluster)?;
                    }
                    let mut copied_positions = Vec::new();
                    for position in positions {
                        account::push(
                            ledger,
                            &mut copied_positions,
                            CellClusterPosition {
                                cluster: cluster_base + position.cluster - clusters.start,
                                pen: position.pen,
                                word: position.word,
                            },
                        )?;
                    }
                    CellPayload::Shaped {
                        runs: run_base..output.runs.len(),
                        clusters: cluster_base..output.clusters.len(),
                        glyphs: glyph_base..output.glyphs.len(),
                        scope_relations: relation_base..output.scope_relations.len(),
                        advance: advance.clone(),
                        positions: copied_positions,
                        frame: *frame,
                    }
                }
            };
            self.retained[cell] = Some(output.cells.len());
            account::push(
                ledger,
                &mut output.cells,
                PreparedCell {
                    binding: original.binding,
                    metrics: original.metrics,
                    level: unit.level,
                    extent: original.extent,
                    // Selection re-evaluates its own exterior boundaries. The
                    // prepared gap buffer is not copied with this fixed payload.
                    advance: original.extent,
                    before: original.before,
                    after: original.after,
                    payload,
                },
            )?;
        }
        // Existing selected consumers use logical cluster order. Move real
        // rows in place, then rebind the retained position indices by their
        // actual selected atom identity; glyph/run arenas are not reshaped.
        let old_atoms = account::collect(
            ledger,
            output.clusters.iter().map(|cluster| cluster.atoms.start),
        )?;
        crate::processing::contextual::alignment::operations::order_by(
            &mut output.clusters,
            ledger.as_mut(),
            |a, b| a.atoms.start.cmp(&b.atoms.start),
        )?;
        let mut new_clusters = account::filled(ledger, presentation.projection.atoms.len(), None)?;
        for (index, cluster) in output.clusters.iter().enumerate() {
            account::work(ledger, 1)?;
            new_clusters[cluster.atoms.start] = Some(index);
        }
        for cell in &mut output.cells {
            if let CellPayload::Shaped {
                clusters,
                positions,
                ..
            } = &mut cell.payload
            {
                let mut first = None;
                let mut last = None;
                for position in positions.iter_mut() {
                    account::work(ledger, 1)?;
                    position.cluster = new_clusters[old_atoms[position.cluster]]
                        .expect("actual selected cell cluster");
                    first = Some(
                        first.map_or(position.cluster, |old: usize| old.min(position.cluster)),
                    );
                    last =
                        Some(last.map_or(position.cluster, |old: usize| old.max(position.cluster)));
                }
                let start = first.expect("actual shaped cell input");
                let end = last.expect("actual shaped cell input") + 1;
                if end - start != positions.len() {
                    return Err(unsupported());
                }
                *clusters = start..end;
            }
        }
        account::drop_vec(ledger, old_atoms)?;
        account::drop_vec(ledger, new_clusters)
    }

    pub(super) fn cell(&self, source: usize) -> usize {
        self.retained[source].expect("actual retained selected cell")
    }

    pub(super) fn push_item(
        &self,
        items: &mut Vec<ShapedItem>,
        item: ShapedItem,
        ledger: &mut Ledger,
    ) -> Result<()> {
        if self.retained.is_empty() {
            items.push(item);
            Ok(())
        } else {
            account::push(ledger, items, item)
        }
    }

    pub(super) fn finish(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.owners)?;
        account::drop_vec(ledger, self.retained)
    }
}

/// Real source and already-resolved horizontal policy for one packet; it never
/// substitutes mapped output or an alternate paragraph into source context.
struct Composition<'a, 'input> {
    analyzed: &'a AnalyzedData,
    horizontal: &'a ShapingInput<'input>,
    words: &'a [SeparatorCandidate],
}

fn normal_payload(
    output: &mut ShapedData,
    input: &Input,
    composition: &Composition<'_, '_>,
    ledger: &mut Ledger,
) -> Result<CellPayload> {
    let first_run = operations::partition_point(&output.runs, ledger.as_mut(), |run| {
        run.atoms.end <= input.atoms.start
    })?;
    let last_run = operations::partition_point(&output.runs, ledger.as_mut(), |run| {
        run.atoms.start < input.atoms.end
    })?;
    let first_cluster =
        operations::partition_point(&output.clusters, ledger.as_mut(), |cluster| {
            cluster.atoms.end <= input.atoms.start
        })?;
    let last_cluster = operations::partition_point(&output.clusters, ledger.as_mut(), |cluster| {
        cluster.atoms.start < input.atoms.end
    })?;
    if first_run == last_run || first_cluster == last_cluster {
        return Err(unsupported());
    }
    let runs = first_run..last_run;
    let clusters = first_cluster..last_cluster;
    let mut glyph_start = None;
    let mut glyph_end = None;
    let mut ascent: Option<f64> = None;
    let mut descent: Option<f64> = None;
    for run in &output.runs[runs.clone()] {
        account::work(ledger, 1)?;
        if run.atoms.start < input.atoms.start || run.atoms.end > input.atoms.end {
            return Err(unsupported());
        }
        ascent = Some(ascent.map_or(run.metrics.ascent, |old| old.max(run.metrics.ascent)));
        descent = Some(descent.map_or(run.metrics.descent, |old| old.max(run.metrics.descent)));
    }
    let execution = CellWordExecution::new(
        composition.analyzed,
        composition.horizontal.styles,
        output,
        input.atoms.clone(),
        clusters.clone(),
        composition.words,
        ledger,
    )?;
    // Provider-cluster operands move as one L2 object. Native extraction may
    // retain several source/caret rows for one ligature; those are not distinct
    // glyph streams to reverse and offset independently.
    let mut groups: Vec<IndexRange<usize>> = Vec::new();
    for index in clusters.clone() {
        account::work(ledger, 1)?;
        let value = &output.clusters[index];
        if let Some(group) = groups.last_mut() {
            let previous = &output.clusters[group.start];
            if previous.provider_group == value.provider_group
                && previous.content == value.content
                && previous.level == value.level
            {
                group.end = index + 1;
                continue;
            }
        }
        account::push(ledger, &mut groups, index..index + 1)?;
    }
    let mut slots = Vec::new();
    let mut minimum_odd: Option<u8> = None;
    let mut maximum = 0;
    for group in &groups {
        account::work(ledger, 1)?;
        let cluster = &output.clusters[group.start];
        let level = crate::BidiLevel::try_new(cluster.level).map_err(|_| unsupported())?;
        maximum = maximum.max(level.get());
        if level.is_rtl() {
            minimum_odd = Some(minimum_odd.map_or(level.get(), |old| old.min(level.get())));
        }
        account::push(
            ledger,
            &mut slots,
            crate::SuppliedInlineParticipant::Text {
                level,
                edge: crate::InlineWhitespaceEdge::Preserve,
            },
        )?;
    }
    let mut order = account::filled(ledger, slots.len(), 0usize)?;
    // Conservative admission for canonical L2: initialize/min/max scans, then
    // at most two slot inspections and one moved row per slot per level. The
    // fourth per-level term covers partition endpoints; this is a bound, not
    // an executed-visit trace. No new permutation algorithm is introduced.
    let overflow = || {
        scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
            kind: crate::ScopeAlignmentLimitKind::StructuralWork,
        })
    };
    let passes = minimum_odd.map_or(0, |minimum| usize::from(maximum - minimum) + 1);
    let factor = passes
        .checked_mul(4)
        .and_then(|value| value.checked_add(3))
        .ok_or_else(overflow)?;
    account::work(
        ledger,
        slots.len().checked_mul(factor).ok_or_else(overflow)?,
    )?;
    crate::SuppliedLine::new(&slots)
        .reorder_into(&mut order)
        .map_err(|_| unsupported())?;
    let mut ranks = if execution.is_empty() {
        Vec::new()
    } else {
        account::filled(ledger, clusters.len(), 0usize)?
    };
    if !ranks.is_empty() {
        let mut rank = 0;
        for local in &order {
            account::work(ledger, 1)?;
            for index in groups[*local].clone() {
                account::work(ledger, 1)?;
                ranks[index - clusters.start] = rank;
                rank += 1;
            }
        }
    }
    let halves = execution.place(
        &mut output.clusters[clusters.clone()],
        clusters.start,
        &ranks,
        ledger,
    )?;
    account::drop_vec(ledger, std::mem::take(&mut ranks))?;
    let mut positions = Vec::new();
    let mut pen = 0.0f64;
    let mut start = 0.0f64;
    let mut end = 0.0f64;
    for local in &order {
        for index in groups[*local].clone() {
            account::work(ledger, 1)?;
            let cluster = &output.clusters[index];
            let word = halves
                .get(index - clusters.start)
                .copied()
                .unwrap_or_default();
            glyph_start = Some(glyph_start.map_or(cluster.glyphs.start, |old: usize| {
                old.min(cluster.glyphs.start)
            }));
            glyph_end = Some(
                glyph_end.map_or(cluster.glyphs.end, |old: usize| old.max(cluster.glyphs.end)),
            );
            account::push(
                ledger,
                &mut positions,
                CellClusterPosition {
                    cluster: index,
                    pen,
                    word,
                },
            )?;
            // Logical edges preserve positive added whitespace even with blank
            // ink. The independent half translates raw provider progression;
            // signed terminal flow is not a substitute for this interval.
            start = start.min(pen);
            end = end.max(pen);
            let mut glyph_pen = logical_add("cell word half", pen, word.half)?;
            start = start.min(glyph_pen);
            end = end.max(glyph_pen);
            for glyph in &output.glyphs[cluster.glyphs.clone()] {
                account::work(ledger, 1)?;
                glyph_pen = logical_add("cell glyph pen", glyph_pen, f64::from(glyph.advance))?;
                start = start.min(glyph_pen);
                end = end.max(glyph_pen);
            }
            pen = logical_add("cell cluster pen", pen, cluster.advance)?;
            start = start.min(pen);
            end = end.max(pen);
        }
    }
    let width = logical_add("cell occupied advance", end, -start)?;
    let x_offset = logical_add(
        "cell horizontal centering",
        (input.extent - width) / 2.0,
        -start,
    )?;
    let baseline = logical_add(
        "cell inner baseline",
        input.extent / 2.0,
        logical_add(
            "cell inner midpoint",
            ascent.expect("actual normal font") / 2.0,
            -descent.expect("actual normal font") / 2.0,
        )?,
    )?;
    account::drop_vec(ledger, halves)?;
    account::drop_vec(ledger, groups)?;
    account::drop_vec(ledger, slots)?;
    account::drop_vec(ledger, order)?;
    Ok(CellPayload::Shaped {
        runs,
        clusters,
        glyphs: glyph_start.expect("actual provider input")
            ..glyph_end.expect("actual provider input"),
        scope_relations: input
            .scope_relations
            .clone()
            .expect("actual normal cell invocation"),
        advance: start..end,
        positions,
        frame: CellFrame {
            x_scale: 1.0,
            x_offset,
            baseline,
        },
    })
}
