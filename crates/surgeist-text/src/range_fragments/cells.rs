//! Inner semantic EGC closure over authenticated retained Cell correspondence.
use super::*;
use crate::engine::layout::data::FinalizedCell;
use crate::protocol::paragraph::FlowOriginKey;
use crate::protocol::prepared::cell_sources::{
    CellCorrespondence, CellPartOccurrence, CellPartSource, CellSourceVisit, visit_original_part,
};
use crate::transform::combined::{FixedGroup, SelectionPart};
use crate::transform::relation::{GeneratedSeparatorOwner, RelationEvent};
use metric_geometry::{MetricUnit, MetricUnitIndex};

fn invariant(message: &'static str) -> Error {
    Error::new(ErrorCode::LayoutFailed, message)
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Choice {
    None,
    One(usize),
    Ambiguous,
}
impl Choice {
    fn include(&mut self, unit: usize) {
        *self = match *self {
            Self::None => Self::One(unit),
            Self::One(old) if old == unit => Self::One(old),
            _ => Self::Ambiguous,
        };
    }
}
struct Events {
    first: usize,
    last: usize,
    prefix_end: usize,
    unit: usize,
}
fn visit_event_units(
    events: &[Events],
    event: usize,
    ledger: &mut Ledger,
    mut visit: impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    let first =
        operations::partition_point(events, ledger.as_mut(), |row| row.prefix_end <= event)?;
    let end = operations::partition_point(events, ledger.as_mut(), |row| row.first < event)?;
    if first >= end {
        return Ok(());
    }
    for row in &events[first..end] {
        account::work(ledger, 1)?;
        if row.first < event && event < row.last {
            visit(row.unit)?;
        }
    }
    Ok(())
}
struct Closure<'a> {
    group: &'a FixedGroup,
    parts: Vec<Option<CellPartOccurrence<'a>>>,
    membership: Vec<Choice>,
    selection: Vec<Option<usize>>,
    metric_units: Vec<MetricUnit>,
    metric_index: Vec<MetricUnitIndex>,
    members: Vec<Vec<usize>>,
    removed: Vec<Vec<usize>>,
    selected: Vec<bool>,
    selected_removed: Vec<bool>,
}
impl<'a> Closure<'a> {
    fn part(&self, index: usize) -> CellPartOccurrence<'a> {
        self.parts[index].expect("complete authenticated Cell part inventory")
    }
    fn new(
        layout: &'a Layout,
        line_index: usize,
        cell_index: usize,
        cell: &FinalizedCell,
        key: FlowOriginKey,
        local: Range,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let placed = &layout.inner.lines[line_index];
        let selected_line = &placed.finalized.selected;
        let prepared = &layout.inner.prepared;
        let output = prepared
            .analyzed
            .combination
            .as_ref()
            .ok_or_else(|| invariant("placed Cell has no retained source combination"))?;
        let correspondence = CellCorrespondence::new(
            prepared,
            &selected_line.presentation,
            &selected_line.request.segment,
            cell.unit,
            ledger,
        )?;
        if !std::ptr::eq(correspondence.binding(), &output.bindings[cell.binding]) {
            return Err(invariant(
                "placed Cell does not own its selected correspondence",
            ));
        }
        let group = correspondence.group();
        let plan = &output.plan;
        let used = &plan.used[correspondence.binding().group];
        let mut parts = account::filled(ledger, group.parts.len(), None)?;
        correspondence.visit_sources(ledger, |source, ledger| {
            if let CellSourceVisit::Part(part) = source {
                account::work(ledger, 1)?;
                let at = part.part_index() - group.parts.start;
                if parts[at].replace(part).is_some() {
                    return Err(invariant("Cell part has duplicate selected correspondence"));
                }
            }
            Ok(())
        })?;
        correspondence.drop(ledger)?;
        for part in &parts {
            account::work(ledger, 1)?;
            if part.is_none() {
                return Err(invariant("Cell part has no selected correspondence"));
            }
        }
        let mut direct = account::filled(ledger, parts.len(), None)?;
        let mut metric_units = Vec::new();
        let mut events = Vec::new();
        let projection = &selected_line.presentation.projection;
        for (unit, range) in used.units.iter().enumerate() {
            account::work(ledger, 1)?;
            let mut presentation: Option<Range> = None;
            let mut atoms: Option<std::ops::Range<usize>> = None;
            let mut cut = None;
            let mut first_event = None;
            let mut last_event = None;
            for &part in &used.parts[range.clone()] {
                account::work(ledger, 1)?;
                let at = part - group.parts.start;
                if direct[at].replace(unit).is_some() {
                    return Err(invariant("Cell part belongs to multiple semantic units"));
                }
                let occurrence = parts[at].expect("authenticated part");
                let span = occurrence.presentation();
                let span = Range::new(span.start, span.end);
                presentation = Some(presentation.map_or(span, |old| {
                    Range::new(old.start.min(span.start), old.end.max(span.end))
                }));
                first_event = Some(
                    first_event
                        .map_or(occurrence.event(), |old: usize| old.min(occurrence.event())),
                );
                last_event = Some(
                    last_event.map_or(occurrence.event(), |old: usize| old.max(occurrence.event())),
                );
                let start =
                    operations::partition_point(&projection.atoms, ledger.as_mut(), |atom| {
                        atom.presentation.end <= span.start
                    })?;
                if span.is_empty() {
                    cut.get_or_insert(start);
                } else {
                    let count = operations::partition_point(
                        &projection.atoms[start..],
                        ledger.as_mut(),
                        |atom| atom.presentation.start < span.end,
                    )?;
                    let end = start + count;
                    if start == end
                        || projection.atoms[start].presentation.start != span.start
                        || projection.atoms[end - 1].presentation.end != span.end
                    {
                        return Err(invariant(
                            "Cell semantic part lost its actual selected atom span",
                        ));
                    }
                    atoms = Some(
                        atoms.map_or(start..end, |old| old.start.min(start)..old.end.max(end)),
                    );
                }
            }
            let presentation =
                presentation.ok_or_else(|| invariant("Cell semantic unit has no actual parts"))?;
            let atoms = if let Some(atoms) = atoms {
                atoms
            } else {
                let cut =
                    cut.ok_or_else(|| invariant("Cell zero unit has no actual selected cut"))?;
                cut..cut
            };
            account::push(
                ledger,
                &mut metric_units,
                MetricUnit {
                    atoms,
                    presentation,
                },
            )?;
            account::push(
                ledger,
                &mut events,
                Events {
                    first: first_event.expect("nonempty semantic unit"),
                    last: last_event.expect("nonempty semantic unit"),
                    prefix_end: 0,
                    unit,
                },
            )?;
        }
        let mut metric_index = Vec::new();
        for (unit, row) in metric_units.iter().enumerate() {
            account::work(ledger, 1)?;
            if !row.atoms.is_empty() {
                let first = &projection.atoms[row.atoms.start];
                let last = &projection.atoms[row.atoms.end - 1];
                account::push(
                    ledger,
                    &mut metric_index,
                    MetricUnitIndex {
                        unit,
                        atoms: row.atoms.clone(),
                        presentation: Range::new(first.presentation.start, last.presentation.end),
                    },
                )?;
            }
        }
        operations::order_by(&mut metric_index, ledger.as_mut(), |a, b| {
            (a.atoms.start, a.atoms.end, a.unit).cmp(&(b.atoms.start, b.atoms.end, b.unit))
        })?;
        let mut previous = None;
        for row in &metric_index {
            account::work(ledger, 1)?;
            if previous.is_some_and(|end| row.atoms.start < end) {
                return Err(invariant(
                    "distinct Cell semantic units share selected atoms",
                ));
            }
            previous = Some(row.atoms.end);
        }
        operations::order_by(&mut events, ledger.as_mut(), |a, b| {
            (a.first, a.last, a.unit).cmp(&(b.first, b.last, b.unit))
        })?;
        let mut prefix = 0;
        for event in &mut events {
            account::work(ledger, 1)?;
            prefix = prefix.max(event.last);
            event.prefix_end = prefix;
        }
        let mut selection = account::filled(ledger, parts.len(), None)?;
        let mut selection_choice = account::filled(ledger, group.units.len(), Choice::None)?;
        for selection_unit in group.units.clone() {
            account::work(ledger, 1)?;
            let target = &mut selection_choice[selection_unit - group.units.start];
            for part in plan.units[selection_unit].parts.clone() {
                account::work(ledger, 1)?;
                let at = part - group.parts.start;
                if selection[at].replace(selection_unit).is_some() {
                    return Err(invariant("Cell part has duplicate pre-used membership"));
                }
                if let Some(unit) = direct[at] {
                    target.include(unit);
                }
            }
        }
        let mut membership = account::filled(ledger, parts.len(), Choice::None)?;
        let mut positive = account::filled(ledger, parts.len(), false)?;
        for (at, slot) in parts.iter().enumerate() {
            account::work(ledger, 1)?;
            let part = slot.expect("authenticated part");
            let selection_unit = selection[at]
                .ok_or_else(|| invariant("Cell part lacks its pre-used membership"))?;
            let mut choice = if let Some(unit) = direct[at] {
                Choice::One(unit)
            } else {
                selection_choice[selection_unit - group.units.start]
            };
            if direct[at].is_none() {
                visit_event_units(&events, part.event(), ledger, |unit| {
                    choice.include(unit);
                    Ok(())
                })?;
            }
            membership[at] = choice;
            if part.source() == CellPartSource::Original {
                for piece in &part.part().pieces {
                    account::work(ledger, 1)?;
                    positive[at] |= piece.origin == key && overlap(piece.local, local).is_some();
                }
            }
        }
        let ambiguous = |at: usize| {
            failure(SourceRangeFragmentError::AmbiguousCellUnit {
                line: line_index,
                cell: cell_index,
                selection_unit: selection[at].expect("actual source selection unit"),
            })
        };
        let mut selected = account::filled(ledger, used.units.len(), false)?;
        let mut selected_removed = account::filled(ledger, group.units.len(), false)?;
        for (at, &positive) in positive.iter().enumerate() {
            account::work(ledger, 1)?;
            if !positive {
                continue;
            }
            match membership[at] {
                Choice::One(unit) => selected[unit] = true,
                Choice::None => {
                    selected_removed
                        [selection[at].expect("pre-used membership") - group.units.start] = true
                }
                Choice::Ambiguous => return Err(ambiguous(at)),
            }
        }
        let mut affected = account::filled(ledger, group.units.len(), false)?;
        for (at, unit) in direct.iter().enumerate() {
            account::work(ledger, 1)?;
            if let Some(unit) = *unit {
                affected[selection[at].expect("pre-used membership") - group.units.start] |=
                    selected[unit];
            }
        }
        for (at, &choice) in membership.iter().enumerate() {
            account::work(ledger, 1)?;
            if choice != Choice::Ambiguous {
                continue;
            }
            let mut demanded =
                affected[selection[at].expect("pre-used membership") - group.units.start];
            visit_event_units(
                &events,
                parts[at].expect("authenticated part").event(),
                ledger,
                |unit| {
                    demanded |= selected[unit];
                    Ok(())
                },
            )?;
            if demanded {
                return Err(ambiguous(at));
            }
        }
        let mut members = account::filled(ledger, used.units.len(), Vec::new())?;
        let mut removed = account::filled(ledger, group.units.len(), Vec::new())?;
        for (at, &choice) in membership.iter().enumerate() {
            account::work(ledger, 1)?;
            match choice {
                Choice::One(unit) => account::push(ledger, &mut members[unit], at)?,
                Choice::None => account::push(
                    ledger,
                    &mut removed[selection[at].expect("pre-used membership") - group.units.start],
                    at,
                )?,
                Choice::Ambiguous => (),
            }
        }
        account::drop_vec(ledger, direct)?;
        account::drop_vec(ledger, events)?;
        account::drop_vec(ledger, selection_choice)?;
        account::drop_vec(ledger, positive)?;
        account::drop_vec(ledger, affected)?;
        Ok(Self {
            group,
            parts,
            membership,
            selection,
            metric_units,
            metric_index,
            members,
            removed,
            selected,
            selected_removed,
        })
    }
    fn drop(self, ledger: &mut Ledger) -> Result<()> {
        for mut rows in [self.members, self.removed] {
            for row in &mut rows {
                account::drop_vec(ledger, std::mem::take(row))?;
            }
            account::drop_vec(ledger, rows)?;
        }
        account::drop_vec(ledger, self.parts)?;
        account::drop_vec(ledger, self.membership)?;
        account::drop_vec(ledger, self.selection)?;
        account::drop_vec(ledger, self.metric_units)?;
        account::drop_vec(ledger, self.metric_index)?;
        account::drop_vec(ledger, self.selected)?;
        account::drop_vec(ledger, self.selected_removed)
    }
}

#[derive(Clone, Copy)]
struct OwnedAssociation {
    input: (u8, usize),
    association: OriginIntersection,
}
fn original_input(
    layout: &Layout,
    line: usize,
    part: CellPartOccurrence<'_>,
    ledger: &mut Ledger,
) -> Result<(u8, usize)> {
    account::work(ledger, 1)?;
    if let SelectionPart::CollapsedSpace(record) = part.part().reference {
        return Ok((1, record));
    }
    let selected = &layout.inner.lines[line].finalized.selected.presentation;
    let SelectedRelationEvent::Base { event } = selected.selected_relation.events[part.event()]
    else {
        return Err(invariant(
            "Cell original part has no base selected relation",
        ));
    };
    let map = &layout.inner.prepared.analyzed.transform;
    let row = match map.events()[event] {
        RelationEvent::Text { atom } => map.original_row_for_atom(atom),
        RelationEvent::Removal { row, .. } => Some(row),
        _ => None,
    }
    .ok_or_else(|| invariant("Cell original part has no authenticated input ancestry"))?;
    Ok((0, row))
}
fn associations(
    mut rows: Vec<OwnedAssociation>,
    ledger: &mut Ledger,
) -> Result<Vec<OriginIntersection>> {
    operations::order_by(&mut rows, ledger.as_mut(), |a, b| {
        (association_key(&a.association), a.input).cmp(&(association_key(&b.association), b.input))
    })?;
    account::work(ledger, rows.len())?;
    account::work(ledger, rows.len())?;
    rows.dedup_by(|a, b| {
        a.input == b.input && association_key(&a.association) == association_key(&b.association)
    });
    let result = account::collect(ledger, rows.iter().map(|row| row.association))?;
    account::drop_vec(ledger, rows)?;
    Ok(result)
}
struct Metadata {
    selected: Vec<OriginIntersection>,
    expanded: Vec<OriginIntersection>,
    controls: Vec<usize>,
    virtuals: Vec<usize>,
}
fn metadata(
    layout: &Layout,
    line: usize,
    closure: &Closure<'_>,
    members: &[usize],
    origin: &PreparedOrigin,
    local: Range,
    ledger: &mut Ledger,
) -> Result<Metadata> {
    let mut selected = Vec::new();
    let mut expanded = Vec::new();
    let mut controls = Vec::new();
    let mut virtuals = Vec::new();
    for &member in members {
        account::work(ledger, 1)?;
        let part = closure.part(member);
        match part.source() {
            CellPartSource::Original => {
                let input = original_input(layout, line, part, ledger)?;
                visit_original_part(
                    &layout.inner.prepared,
                    part.part(),
                    ledger,
                    |association, ledger| {
                        let piece = association.source.ok_or_else(|| {
                            invariant("Cell original lost its source association")
                        })?;
                        account::push(
                            ledger,
                            &mut expanded,
                            OwnedAssociation {
                                input,
                                association: piece,
                            },
                        )?;
                        if piece.origin == origin.index
                            && let Some(clip) = overlap(piece.local, local)
                        {
                            let aggregate = Range::new(
                                piece.aggregate.start + clip.start - piece.local.start,
                                piece.aggregate.start + clip.end - piece.local.start,
                            );
                            let association =
                                source_intersection(layout, piece.origin, aggregate, ledger)?;
                            account::push(
                                ledger,
                                &mut selected,
                                OwnedAssociation { input, association },
                            )?;
                        }
                        Ok(())
                    },
                )?;
            }
            CellPartSource::Generated(GeneratedSeparatorOwner::Control(control)) => {
                account::push(ledger, &mut controls, control)?
            }
            CellPartSource::Generated(GeneratedSeparatorOwner::Virtual(owner)) => {
                account::push(ledger, &mut virtuals, owner)?
            }
        }
    }
    for rows in [&mut controls, &mut virtuals] {
        operations::order_by(rows, ledger.as_mut(), Ord::cmp)?;
        dedup_sorted(rows, ledger)?;
    }
    let public_controls = account::collect(
        ledger,
        controls
            .iter()
            .map(|&control| layout.inner.prepared.tables.control_rows[control]),
    )?;
    account::drop_vec(ledger, controls)?;
    Ok(Metadata {
        selected: associations(selected, ledger)?,
        expanded: associations(expanded, ledger)?,
        controls: public_controls,
        virtuals,
    })
}
fn fragments(
    indexes: &LineIndexes,
    metadata: &Metadata,
    layout: &Layout,
    ledger: &mut Ledger,
) -> Result<Vec<usize>> {
    let mut result = Vec::new();
    for source in &metadata.expanded {
        account::work(ledger, 1)?;
        indexes.append_original(source.origin, source.aggregate, &mut result, ledger)?;
    }
    for &public in &metadata.controls {
        account::work(ledger, 1)?;
        let control = layout.inner.prepared.tables.controls[public].occurrence;
        indexes.append_owner(FragmentOwner::Control(control), &mut result, ledger)?;
    }
    for &owner in &metadata.virtuals {
        account::work(ledger, 1)?;
        indexes.append_owner(FragmentOwner::VirtualSeparator(owner), &mut result, ledger)?;
    }
    operations::order_by(&mut result, ledger.as_mut(), Ord::cmp)?;
    dedup_sorted(&mut result, ledger)?;
    Ok(result)
}

struct CellGeometry {
    providers: Vec<Vec<std::ops::Range<usize>>>,
    tabs: Vec<Vec<usize>>,
    spans: Vec<(usize, usize)>,
    cache: GeometryCache,
}
impl CellGeometry {
    fn new(
        layout: &Layout,
        line: usize,
        cell: usize,
        closure: &mut Closure<'_>,
        indexes: &LineIndexes,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let data = &layout.inner.lines[line].finalized.data;
        let mut providers = account::filled(ledger, closure.metric_units.len(), Vec::new())?;
        let mut tabs = account::filled(ledger, closure.metric_units.len(), Vec::new())?;
        let mut native = account::filled(ledger, closure.metric_units.len(), false)?;
        let rows = indexes.cell_clusters(cell, ledger)?;
        let units = MetricUnits::Cell(&closure.metric_units, &closure.metric_index);
        for &(_, index) in rows {
            account::work(ledger, 1)?;
            let cluster = &data.clusters[index];
            if matches!(
                cluster.content,
                crate::engine::layout::data::ClusterContent::CssTab(_)
            ) {
                units.visit_provider_units(&cluster.provider_atoms, ledger, |unit, ledger| {
                    native[unit] = true;
                    account::push(ledger, &mut tabs[unit], index)
                })?;
            } else if cluster.content.font_run().is_none() {
                let mut demanded = None;
                units.visit_provider_units(&cluster.provider_atoms, ledger, |unit, _| {
                    if closure.selected[unit] {
                        demanded = Some(unit);
                    }
                    Ok(())
                })?;
                if let Some(unit) = demanded {
                    return Err(failure(SourceRangeFragmentError::MissingCellReference {
                        line,
                        cell,
                        unit,
                    }));
                }
            }
        }
        // This is the actual font-operation index. A native TAB contributes to
        // its own executed layout, never to font-provider share cardinality.
        account::work(ledger, closure.metric_index.len())?;
        account::work(ledger, closure.metric_index.len())?;
        closure.metric_index.retain(|row| !native[row.unit]);
        account::drop_vec(ledger, native)?;
        let units = MetricUnits::Cell(&closure.metric_units, &closure.metric_index);
        let mut next = 0;
        while next < rows.len() {
            account::work(ledger, 1)?;
            let first = rows[next].1;
            let cluster = &data.clusters[first];
            next += 1;
            if !matches!(
                cluster.content,
                crate::engine::layout::data::ClusterContent::Provider(_)
            ) {
                continue;
            }
            let mut end = first + 1;
            while next < rows.len() {
                account::work(ledger, 1)?;
                let index = rows[next].1;
                let candidate = &data.clusters[index];
                if index != end
                    || candidate.provider_atoms != cluster.provider_atoms
                    || candidate.content != cluster.content
                    || candidate.frame != cluster.frame
                {
                    break;
                }
                end += 1;
                next += 1;
            }
            units.visit_provider_units(&cluster.provider_atoms, ledger, |unit, ledger| {
                account::push(ledger, &mut providers[unit], first..end)
            })?;
        }
        // Finalization preserves the selected shaped cluster ordinals; each
        // retained native span names that same actual cluster, not a TAB ordinal.
        let mut spans = Vec::new();
        for (index, span) in data.cells[cell].tabs.iter().enumerate() {
            account::work(ledger, 1)?;
            account::push(ledger, &mut spans, (span.cluster, index))?;
        }
        operations::order_by(&mut spans, ledger.as_mut(), Ord::cmp)?;
        Ok(Self {
            providers,
            tabs,
            spans,
            cache: GeometryCache::default(),
        })
    }
    fn pieces<'a>(
        &mut self,
        layout: &'a Layout,
        line_index: usize,
        cell_index: usize,
        closure: &Closure<'_>,
        unit: usize,
        ledger: &mut Ledger,
    ) -> Result<Vec<GeometryPiece<'a>>> {
        use crate::engine::layout::data::cell_geometry;
        let placed = &layout.inner.lines[line_index];
        let data = &placed.finalized.data;
        let cell = &data.cells[cell_index];
        let missing = || {
            failure(SourceRangeFragmentError::MissingCellReference {
                line: line_index,
                cell: cell_index,
                unit,
            })
        };
        let mut result = Vec::new();
        let metrics = range_geometry(
            &data.clusters,
            MetricUnits::Cell(&closure.metric_units, &closure.metric_index),
            unit,
            &self.providers[unit],
            &mut self.cache,
            ledger,
            || {
                failure(SourceRangeFragmentError::UnsupportedCellPartition {
                    line: line_index,
                    cell: cell_index,
                    unit,
                })
            },
        )?;
        for metric in &metrics {
            account::work(ledger, 1)?;
            let cluster = &data.clusters[metric.clusters.start];
            let font = cluster
                .content
                .font_run()
                .and_then(|run| data.font_runs.get(run))
                .ok_or_else(&missing)?;
            if font.metrics.horizontal().is_none() {
                return Err(missing());
            }
            account::work(ledger, font.coords.len())?;
            let instance = EffectiveFontInstance::qualify_synthesized(
                font.font.face(),
                font.font_size,
                &font.coords,
                font.synthesis,
            )?;
            let inner = cell.frame.ok_or_else(&missing)?;
            let origin = cell_geometry::placed_layout_point(
                placed,
                cell,
                crate::Point::new(0.0, inner.baseline),
                TextNumericPhase::SourceRangeFragments,
            )?;
            let bounds = cell_geometry::placed_metric_rect(
                placed,
                cell,
                (metric.start, metric.end),
                font.metrics,
                TextNumericPhase::SourceRangeFragments,
            )?;
            account::push(
                ledger,
                &mut result,
                GeometryPiece::CellFont(CellFontPiece {
                    clusters: metric.clusters.clone(),
                    start: metric.start,
                    end: metric.end,
                    bounds,
                    instance,
                    ascent: font.metrics.ascent,
                    descent: font.metrics.descent,
                    frame: SourceRangeCellMetricFrame {
                        origin,
                        x_scale: inner.x_scale,
                    },
                }),
            )?;
        }
        account::drop_vec(ledger, metrics)?;
        for &index in &self.tabs[unit] {
            account::work(ledger, 1)?;
            let cluster = &data.clusters[index];
            let crate::engine::layout::data::ClusterContent::CssTab(_) = cluster.content else {
                return Err(missing());
            };
            let at = operations::binary_search_by(&self.spans, ledger.as_mut(), |row| {
                row.0.cmp(&index)
            })?
            .map_err(|_| missing())?;
            let span_index = self.spans[at].1;
            let span = &cell.tabs[span_index];
            if span.cluster != index || !cluster.provider_atoms.contains(&span.atom) {
                return Err(missing());
            }
            let PresentationUnitKind::Cell {
                cell: prepared_cell,
            } = placed.finalized.selected.presentation.units[cell.unit].kind
            else {
                return Err(missing());
            };
            let original = layout
                .inner
                .prepared
                .shaped
                .cells
                .get(prepared_cell)
                .ok_or_else(&missing)?;
            if original.metrics >= layout.inner.prepared.scope_alignment.metrics.len() {
                return Err(missing());
            }
            let transform = cell.transform.ok_or_else(&missing)?;
            let origin = cell_geometry::placed_layout_point(
                placed,
                cell,
                crate::Point::new(0.0, 0.0),
                TextNumericPhase::SourceRangeFragments,
            )?;
            cell_geometry::qualify_cluster_rect(
                data,
                cluster,
                placed.orientation,
                Some(placed),
                TextNumericPhase::SourceRangeFragments,
            )?;
            let bounds = cell_geometry::placed_cluster_rect(placed, cluster);
            account::push(
                ledger,
                &mut result,
                GeometryPiece::CellTab(CellTabPiece {
                    span: span_index,
                    start: span.before,
                    end: span.after,
                    amount: span.amount,
                    bounds,
                    frame: SourceRangeCellLayoutFrame {
                        origin,
                        x_scale: transform.x_scale,
                        extent: original.extent,
                    },
                    metrics: original.metrics,
                }),
            )?;
        }
        Ok(result)
    }
    fn drop(mut self, ledger: &mut Ledger) -> Result<()> {
        for rows in &mut self.providers {
            account::drop_vec(ledger, std::mem::take(rows))?;
        }
        for rows in &mut self.tabs {
            account::drop_vec(ledger, std::mem::take(rows))?;
        }
        account::drop_vec(ledger, self.providers)?;
        account::drop_vec(ledger, self.tabs)?;
        account::drop_vec(ledger, self.spans)?;
        self.cache.drop_scratch(ledger)
    }
}

/// The original occurrence and interval admitted by the placed Range query.
pub(super) struct Query<'a> {
    pub(super) origin: &'a PreparedOrigin,
    pub(super) key: FlowOriginKey,
    pub(super) local: Range,
}

pub(super) fn append<'a>(
    layout: &'a Layout,
    line_index: usize,
    query: Query<'_>,
    indexes: &LineIndexes,
    result: &mut Vec<Unit<'a>>,
    ledger: &mut Ledger,
) -> Result<()> {
    let Query { origin, key, local } = query;
    let placed = &layout.inner.lines[line_index];
    for (cell_index, cell) in placed.finalized.data.cells.iter().enumerate() {
        account::work(ledger, 1)?;
        if !placed
            .finalized
            .source_cell(cell_index)
            .intersects_original_range_with(key, local, || account::work(ledger, 1))?
        {
            continue;
        }
        let mut closure = Closure::new(layout, line_index, cell_index, cell, key, local, ledger)?;
        let mut geometry = CellGeometry::new(
            layout,
            line_index,
            cell_index,
            &mut closure,
            indexes,
            ledger,
        )?;
        for (unit, &selected) in closure.selected.iter().enumerate() {
            account::work(ledger, 1)?;
            if !selected {
                continue;
            }
            let members = &closure.members[unit];
            let metadata = metadata(layout, line_index, &closure, members, origin, local, ledger)?;
            if metadata.selected.is_empty() {
                return Err(invariant(
                    "selected Cell EGC has no actual requested original",
                ));
            }
            let fragments = fragments(indexes, &metadata, layout, ledger)?;
            let pieces = geometry.pieces(layout, line_index, cell_index, &closure, unit, ledger)?;
            account::push(
                ledger,
                result,
                Unit {
                    line: line_index,
                    identity: UnitIdentity::Cell {
                        cell: cell_index,
                        kind: SourceRangeCellUnitKind::Semantic { ordinal: unit },
                    },
                    order: (
                        cell.unit,
                        *members
                            .first()
                            .ok_or_else(|| invariant("selected Cell EGC has no members"))?,
                    ),
                    selected: metadata.selected,
                    expanded: metadata.expanded,
                    fragments,
                    pieces,
                    controls: metadata.controls,
                    virtuals: metadata.virtuals,
                    alternatives: Vec::new(),
                    defining: Vec::new(),
                    attributed_cuts: Vec::new(),
                    presentation: closure.metric_units[unit].presentation,
                },
            )?;
        }
        for (selection, &selected) in closure.selected_removed.iter().enumerate() {
            account::work(ledger, 1)?;
            if !selected {
                continue;
            }
            let members = &closure.removed[selection];
            let metadata = metadata(layout, line_index, &closure, members, origin, local, ledger)?;
            if metadata.selected.is_empty() {
                return Err(invariant(
                    "removed Cell unit has no actual requested original",
                ));
            }
            let fragments = fragments(indexes, &metadata, layout, ledger)?;
            let mut presentation: Option<Range> = None;
            for &member in members {
                account::work(ledger, 1)?;
                let actual = closure.part(member).presentation();
                let actual = Range::new(actual.start, actual.end);
                presentation = Some(presentation.map_or(actual, |old| {
                    Range::new(old.start.min(actual.start), old.end.max(actual.end))
                }));
            }
            account::push(
                ledger,
                result,
                Unit {
                    line: line_index,
                    identity: UnitIdentity::Cell {
                        cell: cell_index,
                        kind: SourceRangeCellUnitKind::Removed {
                            ordinal: closure.group.units.start + selection,
                        },
                    },
                    order: (
                        cell.unit,
                        *members
                            .first()
                            .ok_or_else(|| invariant("removed Cell unit has no members"))?,
                    ),
                    selected: metadata.selected,
                    expanded: metadata.expanded,
                    fragments,
                    pieces: Vec::new(),
                    controls: metadata.controls,
                    virtuals: metadata.virtuals,
                    alternatives: Vec::new(),
                    defining: Vec::new(),
                    attributed_cuts: Vec::new(),
                    presentation: presentation
                        .ok_or_else(|| invariant("removed Cell unit has no actual source cut"))?,
                },
            )?;
        }
        geometry.drop(ledger)?;
        closure.drop(ledger)?;
    }
    Ok(())
}
