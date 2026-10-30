//! Occurrence-qualified observations of an actual finalized logical prefix.

use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;
use std::{ops::Range as Indices, sync::Arc};

use crate::engine::layout::data::{OrdinalMap, ShapedData, ShapedItem, Whitespace};
use crate::{ErrorDetail, Range, Result, SourcePosition, TextNumericError, TextNumericPhase};

use super::prepared::PresentationUnitKind;
use super::selection::SelectedData;
use super::{FinalizedLine, PreparedOrigin, PreparedParticipant, ProtocolError, protocol_error};

/// An original occurrence seam, independently of legal caret or line boundaries.
/// `local` is an offset in the retained origin's original Source; the query
/// revalidates it against that occurrence's admitted local range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextSeam {
    Origin {
        origin: PreparedOrigin,
        local: SourcePosition,
    },
    BeforeParticipant(PreparedParticipant),
    AfterParticipant(PreparedParticipant),
}

/// A line-owned prefix observation. The seam is cheaply retained independently
/// of the caller's descriptor lifetime. No seam is moved by whole-unit accounting.
#[derive(Debug)]
pub struct FinalizedPrefixView<'a> {
    line: &'a FinalizedLine,
    seam: TextSeam,
    cut: usize,
    interior: Option<usize>,
    retained_flow: bool,
    in_flow_content: bool,
}

impl FinalizedPrefixView<'_> {
    pub fn seam(&self) -> &TextSeam {
        &self.seam
    }
    pub fn finalized_line(&self) -> &FinalizedLine {
        self.line
    }
    /// Actual logical fold receipt; excludes alignment and physical translation.
    pub fn advance(&self) -> f64 {
        self.line.inner.data.prefix.receipts[self.cut]
    }
    /// Whether the actual original occurrence prefix contains retained flow
    /// participation. This is independent of whole-unit width accounting: a
    /// later part charged by an interior seam does not become preceding content.
    /// Retained zero/signed text and allocated zero struts count; discarded text,
    /// empty/deleted occurrences and out-of-flow or neutral roles add nothing.
    /// This does not establish visibility, positive extent or a generated line.
    pub fn has_retained_flow_participation(&self) -> bool {
        self.retained_flow
    }
    /// Actual retained in-flow content before this exact original seam. Unlike
    /// retained participation, vertical control struts do not establish content.
    pub fn has_in_flow_content(&self) -> bool {
        self.in_flow_content
    }
    /// Some only when the original seam is strictly inside the joint unit.
    pub fn interior_unit(&self) -> Option<FinalizedSharedUnitView<'_>> {
        self.interior.map(|index| FinalizedSharedUnitView {
            line: self.line,
            index,
        })
    }
}

/// The smallest candidate-local grouping closed under grapheme and provider
/// cluster association. Its extent is correspondence, not a break capability.
#[derive(Debug)]
pub struct FinalizedSharedUnitView<'a> {
    line: &'a FinalizedLine,
    index: usize,
}
impl FinalizedSharedUnitView<'_> {
    pub fn finalized_line(&self) -> &FinalizedLine {
        self.line
    }
    /// Enclosing original extent; holes are not authored contribution.
    /// Use `source_pieces` to inspect actual original atom coverage.
    pub fn source_range(&self) -> Range {
        self.line.inner.data.prefix.groups[self.index].source
    }
    pub fn source_pieces(&self) -> impl Iterator<Item = Range> + '_ {
        let group = &self.line.inner.data.prefix.groups[self.index];
        self.line.inner.selected.presentation.projection.atoms[group.atoms.clone()]
            .iter()
            .map(|atom| atom.source)
            .filter(|range| !range.is_empty())
    }
    pub fn presentation_range(&self) -> super::PresentationRange {
        {
            let range = self.line.inner.data.prefix.groups[self.index].presentation;
            super::PresentationRange {
                start: range.start,
                end: range.end,
            }
        }
    }
    /// Independently folded contribution, never a subtraction of prefixes.
    pub fn advance(&self) -> Result<f64> {
        self.line.inner.data.prefix.groups[self.index]
            .advance
            .value()
    }
    pub fn alternatives(&self) -> impl Iterator<Item = super::PreparedAlternative> + '_ {
        let row = &self.line.inner.data.prefix.groups[self.index];
        let mut seen = std::collections::BTreeSet::new();
        self.line.inner.selected.presentation.units[row.units.clone()]
            .iter()
            .filter_map(move |unit| {
                if let PresentationUnitKind::Replacement { alternative, .. } = unit.kind
                    && seen.insert(alternative)
                {
                    Some(super::PreparedAlternative {
                        prepared: Arc::clone(&self.line.inner.selected.prepared),
                        index: alternative,
                    })
                } else {
                    None
                }
            })
    }
}

#[derive(Clone, Copy, Debug)]
struct NumericFailure {
    field: &'static str,
    phase: TextNumericPhase,
    category: TextNumericError,
    value: f64,
}
#[derive(Clone, Copy, Debug)]
enum Contribution {
    Value(f64),
    Failure(NumericFailure),
}
impl Contribution {
    fn add(self, delta: f64) -> Self {
        let Self::Value(base) = self else { return self };
        match crate::engine::layout::add(
            "shared unit advance",
            TextNumericPhase::Finalization,
            base,
            delta,
        ) {
            Ok(value) => Self::Value(value),
            Err(error) => match error.detail() {
                Some(ErrorDetail::InvalidTextNumeric {
                    field,
                    phase,
                    category,
                    value,
                }) => Self::Failure(NumericFailure {
                    field,
                    phase: *phase,
                    category: *category,
                    value: *value,
                }),
                _ => unreachable!("logical add produces only its typed numeric failure"),
            },
        }
    }
    fn value(self) -> Result<f64> {
        match self {
            Self::Value(value) => Ok(value),
            Self::Failure(fact) => Err(crate::error::text_numeric_error(
                fact.field,
                fact.phase,
                fact.category,
                fact.value,
            )),
        }
    }
}
#[derive(Clone, Debug)]
struct SharedUnit {
    atoms: Indices<usize>,
    items: Indices<usize>,
    units: Indices<usize>,
    source: Range,
    presentation: Range,
    advance: Contribution,
}
#[derive(Clone, Debug)]
struct OriginSpan {
    source: Range,
    actual_source: Range,
    unit: usize,
    cuts: Indices<usize>,
}

/// Every receipt comes from the existing successful line-metrics fold. Optional
/// unit arithmetic failures remain local facts, and never poison finalization.
#[derive(Clone, Debug, Default)]
pub(crate) struct PrefixData {
    receipts: Vec<f64>,
    pub(super) has_in_flow_content: bool,
    groups: Vec<SharedUnit>,
    unit_cuts: Vec<Indices<usize>>,
    retained_prefix: Vec<bool>,
    content_prefix: Vec<bool>,
    retained_sources: Vec<Vec<Range>>,
    content_sources: Vec<Vec<Range>>,
    participants: Vec<Option<Indices<usize>>>,
    origins: OrdinalMap<Vec<OriginSpan>>,
    participant_indices: Vec<((u8, usize), usize)>,
}

impl PrefixData {
    pub(crate) fn new(
        selected: &SelectedData,
        shaped: &ShapedData,
        atomics: &OrdinalMap<&super::AtomicAllocation>,
        controls: &OrdinalMap<&super::ControlAllocation>,
        dispositions: &[super::SpacingSlotDisposition],
        receipts: Vec<f64>,
        ledger: &mut Ledger,
    ) -> Result<Self> {
        let units = &selected.presentation.units;
        let mut result = Self {
            receipts,
            unit_cuts: account::filled(ledger, units.len(), 0..0)?,
            retained_prefix: account::filled(ledger, 1, false)?,
            content_prefix: account::filled(ledger, 1, false)?,
            retained_sources: account::filled(ledger, units.len(), Vec::new())?,
            content_sources: account::filled(ledger, units.len(), Vec::new())?,
            participants: account::filled(ledger, selected.request.participants.len(), None)?,
            ..Self::default()
        };
        for (i, participant) in selected.request.participants.iter().enumerate() {
            account::push(
                ledger,
                &mut result.participant_indices,
                (participant_key(participant), i),
            )?;
        }
        operations::order_by(
            &mut result.participant_indices,
            ledger.as_mut(),
            |left, right| left.cmp(right),
        )?;
        // The prior sparse index retained the last actual occurrence for a
        // repeated key. Preserve that policy without opaque map storage.
        // Each input after the first performs exactly one adjacent probe,
        // including its bounded survivor update and compaction.
        account::work(ledger, result.participant_indices.len().saturating_sub(1))?;
        result.participant_indices.dedup_by(|later, previous| {
            if later.0 == previous.0 {
                previous.1 = later.1;
                true
            } else {
                false
            }
        });
        let replacement_owner = |alternative: usize| {
            let definition = selected.prepared.tables.alternatives[alternative].definition;
            selected
                .request
                .participants
                .iter()
                .position(|p| match (definition, p) {
                    (
                        super::prepared::AlternativeDefinition::Control { control, .. },
                        PreparedParticipant::Control(handle),
                    ) => selected.prepared.tables.controls[handle.index].occurrence == control,
                    (
                        super::prepared::AlternativeDefinition::Source { source, .. },
                        PreparedParticipant::Text(handle),
                    ) => selected.prepared.tables.units[handle.index].source == source,
                    (
                        super::prepared::AlternativeDefinition::Automatic { atom, .. },
                        PreparedParticipant::Text(handle),
                    ) => selected.prepared.tables.units[handle.index]
                        .atoms
                        .contains(&(atom - 1)),
                    _ => false,
                })
        };
        let mut owner = 0;
        let mut item_units = Vec::new();
        let mut values = Vec::new();
        for item in &shaped.items {
            let unit = match *item {
                ShapedItem::Cell(cell) => {
                    let binding = shaped.cells[cell].binding;
                    loop {
                        account::work(ledger, 1)?;
                        if matches!(units[owner].kind, PresentationUnitKind::Cell { cell }
                            if selected.prepared.shaped.cells[cell].binding == binding)
                        {
                            break owner;
                        }
                        owner += 1;
                    }
                }
                ShapedItem::Anchor { unit, .. } => unit,
                ShapedItem::TextCluster(index) => {
                    let atom = shaped.clusters[index].atoms.start;
                    while units[owner].atoms.end <= atom {
                        owner += 1;
                    }
                    owner
                }
                ShapedItem::InlineBox { occurrence, .. } => {
                    while !matches!(units[owner].kind, PresentationUnitKind::Atomic { occurrence: actual } if actual == occurrence)
                    {
                        owner += 1;
                    }
                    owner
                }
            };
            owner = unit;
            let retained = retained_item(
                selected.presentation_input(),
                item,
                atomics,
                controls,
                dispositions,
            );
            if retained && let ShapedItem::TextCluster(index) = *item {
                for source in selected.presentation.projection.atoms
                    [shaped.clusters[index].atoms.clone()]
                .iter()
                .map(|atom| atom.source)
                .filter(|range| !range.is_empty())
                {
                    account::push(ledger, &mut result.retained_sources[unit], source)?;
                }
            }
            let content = in_flow_item(
                selected.presentation_input(),
                shaped,
                item,
                atomics,
                controls,
                dispositions,
            );
            result.has_in_flow_content |= content;
            let next = *result.content_prefix.last().unwrap() || content;
            account::push(ledger, &mut result.content_prefix, next)?;
            if let ShapedItem::Cell(cell) = *item {
                let output = selected
                    .prepared
                    .analyzed
                    .combination
                    .as_deref()
                    .expect("selected cells retain actual source bindings");
                let binding = &output.bindings[shaped.cells[cell].binding];
                let group = &output.plan.groups[binding.group];
                for part in &output.plan.parts[group.parts.clone()] {
                    account::work(ledger, 1)?;
                    for piece in &part.pieces {
                        if retained {
                            account::push(
                                ledger,
                                &mut result.retained_sources[unit],
                                piece.aggregate,
                            )?;
                        }
                        if content {
                            account::push(
                                ledger,
                                &mut result.content_sources[unit],
                                piece.aggregate,
                            )?;
                        }
                    }
                }
            }
            if content && let ShapedItem::TextCluster(index) = *item {
                for source in selected.presentation.projection.atoms
                    [shaped.clusters[index].atoms.clone()]
                .iter()
                .map(|atom| atom.source)
                .filter(|range| !range.is_empty())
                {
                    account::push(ledger, &mut result.content_sources[unit], source)?;
                }
            }
            let next = *result.retained_prefix.last().unwrap() || retained;
            account::push(ledger, &mut result.retained_prefix, next)?;
            account::push(ledger, &mut item_units, unit)?;
            let value = match *item {
                ShapedItem::Cell(cell) => shaped.cells[cell].extent,
                ShapedItem::TextCluster(index)
                    if shaped.clusters[index].whitespace != Whitespace::Newline =>
                {
                    shaped.clusters[index].advance
                }
                ShapedItem::InlineBox { occurrence, .. } => match atomics[&occurrence].allocation {
                    super::AtomicLineAllocation::InFlow { flow, .. } => {
                        crate::engine::spacing::character::atomic_advance(
                            shaped,
                            occurrence,
                            flow.advance,
                        )
                        .expect("line metrics admitted the actual atomic cell advance")
                    }
                    super::AtomicLineAllocation::OutOfFlow => 0.0,
                },
                ShapedItem::Anchor { unit } => {
                    super::boundary::advance(selected.presentation_input(), unit)
                }
                _ => 0.0,
            };
            account::push(ledger, &mut values, value)?;
        }
        let mut item = 0;
        for (index, unit) in units.iter().enumerate() {
            let begin = item;
            while item < item_units.len() && item_units[item] == index {
                item += 1;
            }
            result.unit_cuts[index] = begin..item;
            let participant = unit.participant.or_else(|| match unit.kind {
                PresentationUnitKind::Replacement { alternative, .. } => {
                    replacement_owner(alternative)
                }
                _ => None,
            });
            if let Some(participant) = participant {
                match &mut result.participants[participant] {
                    Some(row) => {
                        row.start = row.start.min(begin);
                        row.end = row.end.max(item);
                    }
                    row @ None => *row = Some(begin..item),
                }
            }
        }
        let origin_rows = &selected.prepared.analyzed.paragraph.origins;
        for (index, unit) in units.iter().enumerate() {
            let cuts = unit.participant.map_or_else(
                || result.unit_cuts[index].clone(),
                |i| {
                    result.participants[i]
                        .clone()
                        .expect("each actual participant lends its selected presentation bundle")
                },
            );
            if let PresentationUnitKind::EmptyOrigin { origin } = unit.kind {
                append_origin(
                    ledger,
                    &mut result.origins,
                    origin,
                    OriginSpan {
                        source: unit.source,
                        actual_source: unit.source,
                        unit: index,
                        cuts,
                    },
                )?;
            } else if !unit.source.is_empty()
                && !matches!(unit.kind, PresentationUnitKind::Replacement { .. })
            {
                let first =
                    origin_rows.partition_point(|row| row.aggregate.end <= unit.source.start);
                for (origin, row) in origin_rows.iter().enumerate().skip(first) {
                    if row.aggregate.start >= unit.source.end {
                        break;
                    }
                    let source = Range::new(
                        row.aggregate.start.max(unit.source.start),
                        row.aggregate.end.min(unit.source.end),
                    );
                    if !source.is_empty() {
                        append_origin(
                            ledger,
                            &mut result.origins,
                            origin,
                            OriginSpan {
                                source,
                                actual_source: unit.source,
                                unit: index,
                                cuts: cuts.clone(),
                            },
                        )?;
                    }
                }
            }
        }
        let count = selected.presentation.projection.atoms.len();
        let mut atom_items = account::filled(ledger, count, 0)?;
        for (item, value) in shaped.items.iter().enumerate() {
            if let ShapedItem::Cell(_) = *value {
                // The first fold already admitted this exact item's selected
                // unit. Reuse that issuance, including provider-empty cells.
                let unit = &units[item_units[item]];
                account::work(ledger, unit.atoms.len())?;
                for atom in unit.atoms.clone() {
                    atom_items[atom] = item;
                }
            }
            if let ShapedItem::TextCluster(i) = *value {
                for atom in shaped.clusters[i].atoms.clone() {
                    atom_items[atom] = item;
                }
            }
        }
        let groups = super::boundary::groups_accounted(selected, shaped, ledger)?;
        for group in &groups {
            let start = group.start;
            let end = group.end;
            let first = atom_items[start];
            let last = atom_items[end - 1] + 1;
            let atoms = &selected.presentation.projection.atoms[start..end];
            let source = atoms.iter().fold(Range::new(usize::MAX, 0), |r, a| {
                Range::new(r.start.min(a.source.start), r.end.max(a.source.end))
            });
            let advance = values[first..last]
                .iter()
                .fold(Contribution::Value(0.0), |a, &v| a.add(v));
            account::push(
                ledger,
                &mut result.groups,
                SharedUnit {
                    atoms: start..end,
                    items: first..last,
                    units: item_units[first]..item_units[last - 1] + 1,
                    source,
                    presentation: Range::new(
                        atoms[0].presentation.start,
                        atoms[atoms.len() - 1].presentation.end,
                    ),
                    advance,
                },
            )?;
        }
        account::drop_vec(ledger, groups)?;
        account::drop_vec(ledger, atom_items)?;
        account::drop_vec(ledger, item_units)?;
        account::drop_vec(ledger, values)?;
        Ok(result)
    }
}

fn append_origin(
    ledger: &mut Ledger,
    rows: &mut OrdinalMap<Vec<OriginSpan>>,
    origin: usize,
    row: OriginSpan,
) -> Result<()> {
    if rows.get(&origin).is_none() {
        account::insert(ledger, rows, origin, Vec::new())?;
    }
    account::push(ledger, rows.get_mut(&origin).expect("inserted origin"), row)
}

impl PrefixData {
    pub(crate) fn visit_scope_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> Result<()> {
        use crate::engine::scope_alignment::census::AllocationWitness as W;
        visit(W::buffer(&self.receipts, self.receipts.len())?)?;
        visit(W::buffer(&self.groups, self.groups.len())?)?;
        visit(W::buffer(&self.unit_cuts, self.unit_cuts.len())?)?;
        visit(W::buffer(
            &self.retained_prefix,
            self.retained_prefix.len(),
        )?)?;
        visit(W::buffer(&self.content_prefix, self.content_prefix.len())?)?;
        visit(W::buffer(
            &self.retained_sources,
            self.retained_sources.len(),
        )?)?;
        visit(W::buffer(
            &self.content_sources,
            self.content_sources.len(),
        )?)?;
        visit(W::buffer(&self.participants, self.participants.len())?)?;
        visit(W::buffer(&self.origins.rows, self.origins.rows.len())?)?;
        visit(W::buffer(
            &self.participant_indices,
            self.participant_indices.len(),
        )?)?;
        for rows in self.retained_sources.iter().chain(&self.content_sources) {
            visit(W::buffer(rows, rows.len())?)?;
        }
        for (_, rows) in &self.origins.rows {
            visit(W::buffer(rows, rows.len())?)?;
        }
        Ok(())
    }
}

impl FinalizedLine {
    /// Observe the actual prefix at an original occurrence seam. Strictly
    /// interior seams count their whole joint unit without becoming legal stops.
    pub fn prefix_at(&self, seam: &TextSeam) -> Result<FinalizedPrefixView<'_>> {
        let data = &self.inner.data.prefix;
        let selected = &self.inner.selected;
        let invalid = || protocol_error(ProtocolError::OutsideFinalizedLine);
        let (cut, atom, retained_flow, in_flow_content) = match seam {
            TextSeam::BeforeParticipant(p) | TextSeam::AfterParticipant(p) => {
                if match p {
                    PreparedParticipant::Text(p) => p.prepared_text(),
                    PreparedParticipant::Cell(p) => p.prepared_text(),
                    PreparedParticipant::Atomic(p) => p.prepared_text(),
                    PreparedParticipant::Control(p) => p.prepared_text(),
                    PreparedParticipant::ComputedLineBoundary(p) => p.prepared_text(),
                } != self.prepared_text()
                {
                    return Err(protocol_error(ProtocolError::ForeignSelectedLine));
                }
                let key = participant_key(p);
                let index = data
                    .participant_indices
                    .binary_search_by_key(&key, |row| row.0)
                    .ok()
                    .map(|index| data.participant_indices[index].1)
                    .ok_or_else(invalid)?;
                let row = data.participants[index].as_ref().ok_or_else(invalid)?;
                let cut = if matches!(seam, TextSeam::BeforeParticipant(_)) {
                    row.start
                } else {
                    row.end
                };
                (
                    cut,
                    None,
                    data.retained_prefix[cut],
                    data.content_prefix[cut],
                )
            }
            TextSeam::Origin { origin, local } => {
                if !Arc::ptr_eq(&origin.prepared, &selected.prepared) {
                    return Err(protocol_error(ProtocolError::ForeignSelectedLine));
                }
                let original = &selected.prepared.analyzed.paragraph.origins[origin.index];
                if local.get() < original.local.start || local.get() > original.local.end {
                    return Err(protocol_error(ProtocolError::InvalidOriginSeam {
                        position: local.get(),
                    }));
                }
                let at = original.aggregate.start + (local.get() - original.local.start);
                let rows = data.origins.get(&origin.index).ok_or_else(invalid)?;
                // Input rows can emit more than once and in a different
                // semantic order. A source end includes every actual emission
                // of its row; it cannot select the first equal-source output.
                let candidates = || {
                    rows.iter()
                        .filter(|row| row.source.start <= at && at <= row.source.end)
                };
                let row = candidates()
                    .filter(|row| row.actual_source.start < at)
                    .max_by_key(|row| row.cuts.end)
                    .or_else(|| candidates().min_by_key(|row| row.cuts.start))
                    .ok_or_else(invalid)?;
                let interior = row.actual_source.start < at && at < row.actual_source.end;
                let atom = (interior && !selected.presentation.units[row.unit].atoms.is_empty())
                    .then(|| selected.presentation.units[row.unit].atoms.start);
                let cut = if at == row.actual_source.start {
                    row.cuts.start
                } else if interior {
                    data.unit_cuts[row.unit].start
                } else {
                    row.cuts.end
                };
                // Preserve original occurrence order before whole-unit closure.
                // A seam inside an undistributed item may already follow real
                // retained original units, even though its item cut is the start.
                let retained = data.retained_prefix[cut]
                    || interior
                        && data.retained_sources[row.unit]
                            .iter()
                            .any(|source| source.start < at);
                let content = data.content_prefix[cut]
                    || interior
                        && data.content_sources[row.unit]
                            .iter()
                            .any(|source| source.start < at);
                (cut, atom, retained, content)
            }
        };
        let interior = if let Some(atom) = atom {
            let i = data.groups.partition_point(|g| g.atoms.end <= atom);
            data.groups
                .get(i)
                .filter(|g| g.atoms.contains(&atom))
                .map(|_| i)
        } else {
            let i = data.groups.partition_point(|g| g.items.end <= cut);
            data.groups
                .get(i)
                .filter(|g| g.items.start < cut && cut < g.items.end)
                .map(|_| i)
        };
        let cut = if let Some(index) = interior {
            data.groups[index].advance.value()?;
            data.groups[index].items.end
        } else {
            cut
        };
        Ok(FinalizedPrefixView {
            line: self,
            seam: seam.clone(),
            cut,
            interior,
            retained_flow,
            in_flow_content,
        })
    }
}

fn participant_key(participant: &PreparedParticipant) -> (u8, usize) {
    match participant {
        PreparedParticipant::Text(value) => (0, value.index),
        PreparedParticipant::Cell(value) => (4, value.index),
        PreparedParticipant::Atomic(value) => (1, value.index),
        PreparedParticipant::Control(value) => (2, value.index),
        PreparedParticipant::ComputedLineBoundary(value) => (3, value.index),
    }
}

#[cfg(test)]
mod tests;

/// One role-independent predicate shared by finalized prefixes and flow observations.
/// One shared semantic content predicate for finalized and hypothetical flow.
pub(crate) fn in_flow_item(
    input: super::input::PresentationInput<'_>,
    shaped: &ShapedData,
    item: &ShapedItem,
    atomics: &OrdinalMap<&super::AtomicAllocation>,
    controls: &OrdinalMap<&super::ControlAllocation>,
    dispositions: &[super::SpacingSlotDisposition],
) -> bool {
    let retained = retained_item(input, item, atomics, controls, dispositions);
    match *item {
        ShapedItem::Cell(_) => true,
        ShapedItem::TextCluster(index) => {
            retained && shaped.clusters[index].whitespace != Whitespace::Newline
        }
        ShapedItem::InlineBox { .. } => retained,
        ShapedItem::Anchor { unit } => super::boundary::advance(input, unit) > 0.0,
    }
}

pub(crate) fn retained_item(
    input: super::input::PresentationInput<'_>,
    item: &ShapedItem,
    atomics: &OrdinalMap<&super::AtomicAllocation>,
    controls: &OrdinalMap<&super::ControlAllocation>,
    dispositions: &[super::SpacingSlotDisposition],
) -> bool {
    match *item {
        ShapedItem::Cell(_) => true,
        ShapedItem::TextCluster(index) => !matches!(
            dispositions[index],
            super::SpacingSlotDisposition::DiscardedAtLineEdge
        ),
        ShapedItem::InlineBox { occurrence, .. } => matches!(
            atomics[&occurrence].allocation,
            super::AtomicLineAllocation::InFlow { .. }
        ),
        ShapedItem::Anchor { unit } => match input.presentation.units[unit].kind {
            PresentationUnitKind::Control { occurrence } => {
                controls[&occurrence].strut.is_some() || super::boundary::advance(input, unit) > 0.0
            }
            _ => false,
        },
    }
}
