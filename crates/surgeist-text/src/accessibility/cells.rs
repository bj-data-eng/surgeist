//! Native AX joins over actual original Cell members and executed contributions.
//! This is projection storage, not a Scope allocation transaction or a segmenter.
use super::*;
use crate::engine::layout::data::{ClusterContent, FinalizedRunFrame};
use crate::protocol::paragraph::FlowOriginKey;
use crate::protocol::prepared::cell_sources::{
    CellCorrespondence, CellPartSource, CellSourceVisit,
};
use crate::transform::relation::OriginalPiece;

pub(super) fn capacity() -> Error {
    Error::new(
        ErrorCode::LayoutFailed,
        "accessibility projection allocation was refused",
    )
}
pub(super) fn reserve<T>(values: &mut Vec<T>, additional: usize) -> Result<()> {
    values.len().checked_add(additional).ok_or_else(capacity)?;
    values.try_reserve(additional).map_err(|_| capacity())
}
pub(super) fn push<T>(values: &mut Vec<T>, value: T) -> Result<()> {
    reserve(values, 1)?;
    values.push(value);
    Ok(())
}
fn filled<T: Clone>(count: usize, value: T) -> Result<Vec<T>> {
    let mut values = Vec::new();
    reserve(&mut values, count)?;
    values.resize(count, value);
    Ok(values)
}
fn mapping(cell: usize, range: Range) -> Error {
    ax_error(ErrorDetail::UnsupportedAccessibilityCellMapping { cell, range })
}
struct Group {
    active: bool,
    line: Option<usize>,
    units: Vec<usize>,
}

/// Attach actual Cell ownership and word facts to existing original AX units.
/// No new source boundaries or hull-derived members are introduced.
pub(super) fn descriptors(
    analyzed: &crate::engine::AnalyzedData,
    layout: Option<&Layout>,
    mut descriptors: Vec<NodeDescriptor>,
) -> Result<Vec<NodeDescriptor>> {
    let Some(output) = analyzed
        .combination
        .as_deref()
        .filter(|output| !output.bindings.is_empty())
    else {
        return Ok(descriptors);
    };
    descriptors.sort_by_key(NodeDescriptor::key);
    descriptors.dedup();
    let mut groups = Vec::new();
    for _ in &output.bindings {
        push(
            &mut groups,
            Group {
                active: layout.is_none(),
                line: None,
                units: Vec::new(),
            },
        )?;
    }
    if let Some(layout) = layout {
        for (line, placed) in layout.inner.lines.iter().enumerate() {
            for cell in &placed.finalized.data.cells {
                let group = &mut groups[cell.binding];
                if group.line.is_some() {
                    return Err(mapping(
                        cell.binding,
                        placed.finalized.selected.presentation.units[cell.unit].source,
                    ));
                }
                group.active = true;
                group.line = Some(line);
            }
        }
    }
    let mut owners = filled(descriptors.len(), None)?;
    let mut covered = filled(descriptors.len(), Vec::<Range>::new())?;
    // AX units are ordered disjoint source intervals. Distribute each genuine
    // original part to its actual intersections; repeated ancestry adds no unit.
    for (cell, binding) in output.bindings.iter().enumerate() {
        if !groups[cell].active {
            continue;
        }
        let group = &output.plan.groups[binding.group];
        for part in &output.plan.parts[group.parts.clone()] {
            for piece in &part.pieces {
                if piece.aggregate.is_empty() {
                    continue;
                }
                let first = descriptors
                    .partition_point(|descriptor| descriptor.range.end <= piece.aggregate.start);
                for (offset, descriptor) in descriptors[first..].iter().enumerate() {
                    if descriptor.range.start >= piece.aggregate.end {
                        break;
                    }
                    if descriptor.range.is_empty() {
                        continue;
                    }
                    let index = first + offset;
                    if owners[index].is_some_and(|old| old != cell) {
                        return Err(mapping(cell, descriptor.range));
                    }
                    owners[index] = Some(cell);
                    push(
                        &mut covered[index],
                        Range::new(
                            descriptor.range.start.max(piece.aggregate.start),
                            descriptor.range.end.min(piece.aggregate.end),
                        ),
                    )?;
                }
            }
        }
    }
    for (index, owner) in owners.iter().enumerate() {
        let Some(cell) = *owner else {
            continue;
        };
        covered[index].sort_unstable_by_key(|range| (range.start, range.end));
        let mut at = descriptors[index].range.start;
        for piece in &covered[index] {
            if piece.start > at {
                return Err(mapping(cell, descriptors[index].range));
            }
            at = at.max(piece.end);
        }
        if at != descriptors[index].range.end {
            return Err(mapping(cell, descriptors[index].range));
        }
        descriptors[index].cell = Some(cell);
        descriptors[index].cell_line = groups[cell].line;
        push(&mut groups[cell].units, index)?;
    }
    let mut aggregate = filled(groups.len(), false)?;
    let source = analyzed.processed.source().text().code_units();
    for (cell, group) in groups.iter().enumerate() {
        let Some(&first) = group.units.first() else {
            continue;
        };
        let last = *group.units.last().expect("nonempty Cell AX membership");
        if last - first + 1 != group.units.len() {
            return Err(mapping(
                cell,
                Range::new(descriptors[first].range.start, descriptors[last].range.end),
            ));
        }
        for pair in group.units.windows(2) {
            if descriptors[pair[0]].range.end != descriptors[pair[1]].range.start {
                return Err(mapping(
                    cell,
                    Range::new(descriptors[first].range.start, descriptors[last].range.end),
                ));
            }
        }
        descriptors[first].word_start = true;
        if let Some(next) = descriptors
            .iter_mut()
            .skip(last + 1)
            .find(|next| !next.range.is_empty())
        {
            next.word_start = true;
        }
        // A source-only per-unit LF run would introduce a false implicit next
        // paragraph/word. Aggregate only that real Cell, preserving every unit.
        aggregate[cell] = group.units[..group.units.len() - 1].iter().any(|&index| {
            source[descriptors[index].range.start..descriptors[index].range.end]
                .last()
                .is_some_and(|unit| matches!(*unit, 0x0a | 0x0d))
        });
    }
    let mut result = Vec::new();
    let mut input = descriptors.into_iter().peekable();
    while let Some(mut descriptor) = input.next() {
        if let Some(cell) = descriptor.cell.filter(|&cell| aggregate[cell]) {
            while input.peek().is_some_and(|next| next.cell == Some(cell)) {
                let next = input.next().expect("peeked Cell carrier member");
                descriptor.range.end = next.range.end;
                if descriptor.characters.end != next.characters.start {
                    return Err(mapping(cell, descriptor.range));
                }
                descriptor.characters.end = next.characters.end;
            }
        }
        push(&mut result, descriptor)?;
    }
    Ok(result)
}

/// One ordered coverage pass over actual contribution ranges. A missing original
/// member prevents a uniform geometry-derived metadata claim; it never borrows
/// the neighboring member's contribution. Scratch belongs to Native AX.
pub(super) fn metadata_complete(characters: &[Range], pieces: &[&Fragment]) -> Result<bool> {
    let mut ranges = Vec::new();
    reserve(&mut ranges, pieces.len())?;
    ranges.extend(pieces.iter().map(|piece| piece.range));
    ranges.sort_unstable_by_key(|range| (range.start, range.end));
    let mut next = 0;
    let mut end = 0;
    for unit in characters {
        while next < ranges.len() && ranges[next].start < unit.end {
            end = end.max(ranges[next].end);
            next += 1;
        }
        if end <= unit.start {
            return Ok(false);
        }
    }
    Ok(true)
}

fn origin_key(origin: FlowOriginKey) -> (u8, usize) {
    match origin {
        FlowOriginKey::CapturedEvent(index) => (0, index),
        FlowOriginKey::ExactOrdinal(index) => (1, index),
    }
}
fn piece_key(piece: &&OriginalPiece) -> (usize, usize, (u8, usize), usize, usize) {
    (
        piece.aggregate.start,
        piece.aggregate.end,
        origin_key(piece.origin),
        piece.local.start,
        piece.local.end,
    )
}
struct AtomSource<'a> {
    atom: usize,
    pieces: &'a [OriginalPiece],
}

/// Native projection of actual finalized Cell rows. Never consume Range EGC
/// closure: only positive original part incidence can lend a contribution.
pub(super) fn fragments(
    layout: &Layout,
    locales: &LocaleIndex,
    out: &mut Vec<Fragment>,
) -> Result<()> {
    for (line_index, placed) in layout.inner.lines.iter().enumerate() {
        let data = &placed.finalized.data;
        let selected = &placed.finalized.selected;
        let mut by_cell = filled(data.cells.len(), Vec::<usize>::new())?;
        for (cluster, row) in data.clusters.iter().enumerate() {
            if let FinalizedRunFrame::Cell(cell) = row.frame {
                push(&mut by_cell[cell], cluster)?;
            }
        }
        let mut metadata = filled(data.font_runs.len(), None::<Metadata>)?;
        for (cell_index, cell) in data.cells.iter().enumerate() {
            // Explicit Native AX owner; this is not a fallback from a failed
            // Scope ledger. The correspondence owns no new font/provider work.
            let mut native = None;
            let correspondence = CellCorrespondence::new(
                &selected.prepared,
                &selected.presentation,
                &selected.request.segment,
                cell.unit,
                &mut native,
            )?;
            let unit = &selected.presentation.units[cell.unit];
            let mut atoms = Vec::new();
            correspondence.visit_sources(&mut native, |source, _| {
                let CellSourceVisit::Part(part) = source else {
                    return Ok(());
                };
                if part.source() != CellPartSource::Original || part.presentation().is_empty() {
                    return Ok(());
                }
                let offset = selected.presentation.selected_relation.atom_events
                    [unit.atoms.clone()]
                .binary_search(&part.event())
                .map_err(|_| mapping(cell.binding, unit.source))?;
                push(
                    &mut atoms,
                    AtomSource {
                        atom: unit.atoms.start + offset,
                        pieces: &part.part().pieces,
                    },
                )
            })?;
            atoms.sort_unstable_by_key(|row| row.atom);
            let mut native_spans = Vec::new();
            for span in &cell.tabs {
                push(&mut native_spans, span.cluster)?;
            }
            native_spans.sort_unstable();
            for &index in &by_cell[cell_index] {
                let cluster = &data.clusters[index];
                if cluster.disposition == crate::SpacingSlotDisposition::DiscardedAtLineEdge {
                    continue;
                }
                let provider = match cluster.content {
                    ClusterContent::Provider(run) => Some(run),
                    ClusterContent::CssTab(_) => {
                        if native_spans.binary_search(&index).is_err() {
                            return Err(mapping(cell.binding, cluster.source));
                        }
                        None
                    }
                };
                let first = atoms.partition_point(|row| row.atom < cluster.provider_atoms.start);
                let mut pieces: Vec<&OriginalPiece> = Vec::new();
                for row in &atoms[first..] {
                    if row.atom >= cluster.provider_atoms.end {
                        break;
                    }
                    reserve(&mut pieces, row.pieces.len())?;
                    pieces.extend(row.pieces.iter());
                }
                pieces.sort_unstable_by_key(piece_key);
                pieces.dedup_by(|a, b| piece_key(a) == piece_key(b));
                if pieces.is_empty() {
                    continue;
                }
                crate::engine::layout::data::cell_geometry::qualify_cluster_rect(
                    data,
                    cluster,
                    placed.orientation,
                    Some(placed),
                    crate::TextNumericPhase::Accessibility,
                )?;
                let bounds = crate::engine::layout::data::cell_geometry::placed_cluster_rect(
                    placed, cluster,
                );
                for piece in pieces {
                    let (rtl, font) = if let Some(run) = provider {
                        if metadata[run].is_none() {
                            metadata[run] = Some(Metadata::from_run(&data.font_runs[run], None));
                        }
                        let mut actual = metadata[run]
                            .as_ref()
                            .expect("resolved AX run metadata")
                            .clone();
                        actual.locale = locales.uniform(piece.aggregate);
                        (data.font_runs[run].level & 1 != 0, Some(actual))
                    } else {
                        (cluster.level & 1 != 0, None)
                    };
                    push(
                        out,
                        Fragment {
                            range: piece.aggregate,
                            bounds,
                            line: line_index,
                            rtl,
                            orientation: crate::LineOrientation::Horizontal,
                            word: false,
                            metadata: font,
                        },
                    )?;
                }
            }
            correspondence.drop(&mut native)?;
        }
    }
    Ok(())
}
