//! Occurrence-qualified observations of an actual finalized logical prefix.

use std::{collections::BTreeMap, ops::Range as Indices, sync::Arc};

use crate::engine::layout::data::{ShapedData, ShapedItem, Whitespace};
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
    pub fn alternative(&self) -> Option<super::PreparedAlternative> {
        let row = &self.line.inner.data.prefix.groups[self.index];
        self.line.inner.selected.presentation.units[row.units.clone()]
            .iter()
            .any(|unit| matches!(unit.kind, PresentationUnitKind::Replacement { .. }))
            .then(|| self.line.inner.selected.request.alternative.clone())
            .flatten()
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
    groups: Vec<SharedUnit>,
    unit_cuts: Vec<Indices<usize>>,
    participants: Vec<Option<Indices<usize>>>,
    origins: BTreeMap<usize, Vec<OriginSpan>>,
    participant_indices: BTreeMap<(u8, usize), usize>,
}

impl PrefixData {
    pub(crate) fn new(
        selected: &SelectedData,
        shaped: &ShapedData,
        atomics: &BTreeMap<usize, &super::AtomicAllocation>,
        receipts: Vec<f64>,
    ) -> Self {
        let units = &selected.presentation.units;
        let mut result = Self {
            receipts,
            unit_cuts: vec![0..0; units.len()],
            participants: vec![None; selected.request.participants.len()],
            ..Self::default()
        };
        for (i, participant) in selected.request.participants.iter().enumerate() {
            result
                .participant_indices
                .insert(participant_key(participant), i);
        }
        let replacement_owner = selected
            .request
            .alternative
            .as_ref()
            .and_then(|alternative| {
                let definition =
                    selected.prepared.tables.alternatives[alternative.index].definition;
                selected
                    .request
                    .participants
                    .iter()
                    .position(|p| match (definition, p) {
                        (
                            super::paragraph::ParagraphReplacementDefinition::Control {
                                control,
                                ..
                            },
                            PreparedParticipant::Control(handle),
                        ) => selected.prepared.tables.controls[handle.index].occurrence == control,
                        (
                            super::paragraph::ParagraphReplacementDefinition::Source {
                                source, ..
                            },
                            PreparedParticipant::Text(handle),
                        ) => selected.prepared.tables.units[handle.index].source == source,
                        _ => false,
                    })
            });
        let mut owner = 0;
        let mut item_units = Vec::with_capacity(shaped.items.len());
        let mut values = Vec::with_capacity(shaped.items.len());
        for item in &shaped.items {
            let unit = match *item {
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
            item_units.push(unit);
            values.push(match *item {
                ShapedItem::TextCluster(index)
                    if shaped.clusters[index].whitespace != Whitespace::Newline =>
                {
                    shaped.clusters[index].advance
                }
                ShapedItem::InlineBox { occurrence, .. } => match atomics[&occurrence].allocation {
                    super::AtomicLineAllocation::InFlow { flow, .. } => flow.advance,
                    super::AtomicLineAllocation::OutOfFlow => 0.0,
                },
                _ => 0.0,
            });
        }
        let mut item = 0;
        for (index, unit) in units.iter().enumerate() {
            let begin = item;
            while item < item_units.len() && item_units[item] == index {
                item += 1;
            }
            result.unit_cuts[index] = begin..item;
            let participant = unit.participant.or_else(|| {
                matches!(unit.kind, PresentationUnitKind::Replacement { .. })
                    .then_some(replacement_owner)
                    .flatten()
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
                result.origins.entry(origin).or_default().push(OriginSpan {
                    source: unit.source,
                    actual_source: unit.source,
                    unit: index,
                    cuts,
                });
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
                        result.origins.entry(origin).or_default().push(OriginSpan {
                            source,
                            actual_source: unit.source,
                            unit: index,
                            cuts: cuts.clone(),
                        });
                    }
                }
            }
        }
        let count = selected.presentation.projection.atoms.len();
        let mut ends: Vec<_> = (1..=count).collect();
        for range in selected
            .presentation
            .analysis
            .graphemes
            .iter()
            .chain(shaped.clusters.iter().map(|c| &c.provider_group))
        {
            if !range.is_empty() {
                ends[range.start] = ends[range.start].max(range.end);
            }
        }
        let mut atom_items = vec![0; count];
        for (item, value) in shaped.items.iter().enumerate() {
            if let ShapedItem::TextCluster(i) = *value {
                for atom in shaped.clusters[i].atoms.clone() {
                    atom_items[atom] = item;
                }
            }
        }
        let mut atom = 0;
        while atom < count {
            let start = atom;
            let mut end = ends[atom];
            while atom < end {
                end = end.max(ends[atom]);
                atom += 1;
            }
            let first = atom_items[start];
            let last = atom_items[end - 1] + 1;
            let atoms = &selected.presentation.projection.atoms[start..end];
            let source = atoms.iter().fold(Range::new(usize::MAX, 0), |r, a| {
                Range::new(r.start.min(a.source.start), r.end.max(a.source.end))
            });
            let advance = values[first..last]
                .iter()
                .fold(Contribution::Value(0.0), |a, &v| a.add(v));
            result.groups.push(SharedUnit {
                atoms: start..end,
                items: first..last,
                units: item_units[first]..item_units[last - 1] + 1,
                source,
                presentation: Range::new(
                    atoms[0].presentation.start,
                    atoms[atoms.len() - 1].presentation.end,
                ),
                advance,
            });
        }
        result
    }
}

impl FinalizedLine {
    /// Observe the actual prefix at an original occurrence seam. Strictly
    /// interior seams count their whole joint unit without becoming legal stops.
    pub fn prefix_at(&self, seam: &TextSeam) -> Result<FinalizedPrefixView<'_>> {
        let data = &self.inner.data.prefix;
        let selected = &self.inner.selected;
        let invalid = || protocol_error(ProtocolError::OutsideFinalizedLine);
        let (cut, atom) = match seam {
            TextSeam::BeforeParticipant(p) | TextSeam::AfterParticipant(p) => {
                if match p {
                    PreparedParticipant::Text(p) => p.prepared_text(),
                    PreparedParticipant::Atomic(p) => p.prepared_text(),
                    PreparedParticipant::Control(p) => p.prepared_text(),
                } != self.prepared_text()
                {
                    return Err(protocol_error(ProtocolError::ForeignSelectedLine));
                }
                let index = *data
                    .participant_indices
                    .get(&participant_key(p))
                    .ok_or_else(invalid)?;
                let row = data.participants[index].as_ref().ok_or_else(invalid)?;
                (
                    if matches!(seam, TextSeam::BeforeParticipant(_)) {
                        row.start
                    } else {
                        row.end
                    },
                    None,
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
                let index = rows.partition_point(|row| row.source.end < at);
                let row = rows
                    .get(index)
                    .filter(|row| row.source.start <= at && at <= row.source.end)
                    .ok_or_else(invalid)?;
                let interior = row.actual_source.start < at && at < row.actual_source.end;
                let atom = (interior && !selected.presentation.units[row.unit].atoms.is_empty())
                    .then(|| selected.presentation.units[row.unit].atoms.start);
                (
                    if at == row.actual_source.start {
                        row.cuts.start
                    } else if interior {
                        data.unit_cuts[row.unit].start
                    } else {
                        row.cuts.end
                    },
                    atom,
                )
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
        })
    }
}

fn participant_key(participant: &PreparedParticipant) -> (u8, usize) {
    match participant {
        PreparedParticipant::Text(value) => (0, value.index),
        PreparedParticipant::Atomic(value) => (1, value.index),
        PreparedParticipant::Control(value) => (2, value.index),
    }
}

#[cfg(test)]
mod tests;
