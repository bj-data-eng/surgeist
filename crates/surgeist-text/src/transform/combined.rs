//! Source-only combination selection, frozen ownership and used presentation.

pub(super) mod selection;
mod used;
use super::{context, execute::Canonical, relation::OriginalPiece};
use crate::{Range, Result, TextCombineUpright, WhiteSpace};
use std::{ops::Range as IndexRange, sync::Arc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectionPart {
    Emission(usize),
    CollapsedSpace(usize),
}
#[derive(Clone, Debug)]
pub(crate) struct Part {
    pub(crate) reference: SelectionPart,
    pub(crate) identity: u32,
    pub(crate) normal: u32,
    pub(crate) source: Range,
    pub(crate) processed: Range,
    pub(crate) scope: usize,
    pub(crate) order: usize,
    pub(crate) mode: WhiteSpace,
    pub(crate) pieces: Vec<OriginalPiece>,
    /// The actual pre-replay emission receipt, retained independently of later
    /// output-local reindexing. Lexical SPACE has its ledger identity instead.
    pub(crate) emission: Option<super::relation::Emission>,
    pub(crate) output: Option<usize>,
}
#[derive(Clone, Debug)]
pub(crate) struct SelectionUnit {
    pub(crate) parts: IndexRange<usize>,
    pub(crate) scope: usize,
    pub(crate) crosses_box: bool,
    pub(crate) digit: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct FixedGroup {
    pub(crate) units: IndexRange<usize>,
    pub(crate) parts: IndexRange<usize>,
    pub(crate) scope: usize,
    pub(crate) parent: Option<usize>,
    pub(crate) mode: TextCombineUpright,
    pub(crate) inner_direction: crate::ParagraphBaseDirection,
}
#[derive(Clone, Debug)]
pub(crate) struct UsedPayload {
    pub(crate) parts: Vec<usize>,
    pub(crate) presentation_parts: Vec<usize>,
    pub(crate) levels: Vec<u8>,
    pub(crate) units: Vec<IndexRange<usize>>,
}
#[derive(Debug)]
pub(crate) struct CombinedSourcePlan {
    pub(crate) parts: Vec<Part>,
    pub(crate) units: Vec<SelectionUnit>,
    pub(crate) groups: Vec<FixedGroup>,
    pub(crate) used: Vec<UsedPayload>,
    /// Genuine source fences, expressed in the immutable part inventory.
    pub(crate) fences: Vec<usize>,
    pub(crate) semantic_fences: Vec<usize>,
    pub(crate) visible: Vec<bool>,
    pub(crate) retained: Vec<bool>,
    pub(crate) replay_stops: Vec<(usize, bool)>,
    pub(crate) replay_unknown: Vec<(usize, usize, usize)>,
    pub(crate) group_for_part: Vec<Option<usize>>,
    pub(crate) lexical_parts: Vec<Option<usize>>,
}
#[derive(Debug)]
pub(crate) struct CombinedOutput {
    pub(crate) plan: Arc<CombinedSourcePlan>,
    pub(crate) units: Vec<usize>,
    pub(crate) cells: Vec<usize>,
    pub(crate) bindings: Vec<CellBinding>,
}

#[derive(Clone, Debug)]
pub(crate) struct CellBinding {
    pub(crate) group: usize,
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) events: IndexRange<usize>,
    pub(crate) participants: IndexRange<usize>,
}

pub(super) fn complete(
    canonical: &mut Canonical<'_>,
    output_for_raw: &[Option<usize>],
) -> Result<Vec<Arc<CombinedOutput>>> {
    if !canonical.context.combination_requested() {
        let count = output_for_raw
            .iter()
            .flatten()
            .copied()
            .max()
            .map_or(0, |last| last + 1);
        let plan = Arc::new(CombinedSourcePlan {
            parts: Vec::new(),
            units: Vec::new(),
            groups: Vec::new(),
            used: Vec::new(),
            fences: Vec::new(),
            semantic_fences: Vec::new(),
            visible: Vec::new(),
            retained: Vec::new(),
            replay_stops: Vec::new(),
            replay_unknown: Vec::new(),
            group_for_part: Vec::new(),
            lexical_parts: Vec::new(),
        });
        return context::collect_rows((0..count).map(|_| {
            Arc::new(CombinedOutput {
                plan: Arc::clone(&plan),
                units: Vec::new(),
                cells: Vec::new(),
                bindings: Vec::new(),
            })
        }));
    }
    let mut plan = selection::select(canonical, output_for_raw)?;
    for (index, part) in plan.parts.iter_mut().enumerate() {
        if let SelectionPart::Emission(id) = part.reference {
            canonical.map.emissions[id].selection_part = Some(index);
            part.emission = Some(canonical.map.emissions[id]);
        }
    }
    used::complete(canonical, &mut plan)?;
    let count = output_for_raw
        .iter()
        .flatten()
        .copied()
        .max()
        .map_or(0, |last| last + 1);
    let plan = Arc::new(plan);
    let mut unit_lists: Vec<Vec<usize>> = context::collect_rows((0..count).map(|_| Vec::new()))?;
    let mut cell_lists: Vec<Vec<usize>> = context::collect_rows((0..count).map(|_| Vec::new()))?;
    for (index, unit) in plan.units.iter().enumerate() {
        if let Some(owner) = plan.parts[unit.parts.start].output {
            context::push(&mut unit_lists[owner], index)?;
        }
    }
    for (index, group) in plan.groups.iter().enumerate() {
        let owner = plan.parts[group.parts.start]
            .output
            .expect("checked selected group authority");
        context::push(&mut cell_lists[owner], index)?;
    }
    context::collect_rows(
        unit_lists
            .into_iter()
            .zip(cell_lists)
            .map(|(units, cells)| {
                Arc::new(CombinedOutput {
                    plan: Arc::clone(&plan),
                    units,
                    cells,
                    bindings: Vec::new(),
                })
            }),
    )
}

impl CombinedOutput {
    /// Retain exact output-local incidence after canonical row/event lowering.
    pub(crate) fn bind(
        &self,
        map: &super::relation::TransformMap,
        lexical_state: &crate::processing::lexical_inventory::LexicalState,
    ) -> Result<Self> {
        if self.cells.is_empty() {
            return Ok(Self {
                plan: Arc::clone(&self.plan),
                units: context::collect_rows(self.units.iter().copied())?,
                cells: Vec::new(),
                bindings: Vec::new(),
            });
        }
        let mut intervals = context::filled(self.cells.len(), None::<IndexRange<usize>>)?;
        let mut atom_prefix = context::filled(
            map.events
                .len()
                .checked_add(1)
                .ok_or_else(context::capacity)?,
            0usize,
        )?;
        for (event_index, event) in map.events.iter().enumerate() {
            atom_prefix[event_index + 1] = atom_prefix[event_index]
                + usize::from(matches!(event, super::relation::RelationEvent::Text { .. }));
            let mut own = |part: usize| {
                if let Some(group) = self.plan.group_for_part[part]
                    && let Ok(local) = self.cells.binary_search(&group)
                {
                    match &mut intervals[local] {
                        Some(range) => range.end = event_index + 1,
                        slot @ None => *slot = Some(event_index..event_index + 1),
                    }
                }
            };
            match event {
                super::relation::RelationEvent::Text { atom } => {
                    if let Some(part) = map.emission_for_atom(*atom).selection_part {
                        own(part);
                    }
                }
                super::relation::RelationEvent::Removal {
                    selection_part: Some(part),
                    ..
                } => own(*part),
                super::relation::RelationEvent::GeneratedSuppression { selection_part, .. } => {
                    own(*selection_part)
                }
                super::relation::RelationEvent::PhaseIDeleted { lexical, .. } => {
                    for (record, _) in lexical.indexed_occurrences(lexical_state) {
                        if let Some(part) = self.plan.lexical_parts[record] {
                            own(part);
                        }
                    }
                }
                _ => (),
            }
        }
        let mut bindings = Vec::new();
        for (local, &group) in self.cells.iter().enumerate() {
            let events = intervals[local]
                .clone()
                .expect("real selected source interval");
            context::push(
                &mut bindings,
                CellBinding {
                    group,
                    atoms: atom_prefix[events.start]..atom_prefix[events.end],
                    participants: map
                        .logical_prefix_at_cut(events.start)
                        .expect("actual source cut")
                        ..map
                            .logical_prefix_at_cut(events.end)
                            .expect("actual source cut"),
                    events,
                },
            )?;
        }
        Ok(Self {
            plan: Arc::clone(&self.plan),
            units: context::collect_rows(self.units.iter().copied())?,
            cells: context::collect_rows(self.cells.iter().copied())?,
            bindings,
        })
    }
}

/// Missing authentic source facts or an unsupported downstream cell capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextCombineError {
    MissingCapture,
    MissingMode {
        scope: usize,
    },
    MissingWritingAxis {
        domain: usize,
    },
    MissingInnerDirection {
        scope: usize,
    },
    MissingScopeContext {
        scope: usize,
    },
    /// Selected cells have completed source semantics, but actual cell/font
    /// preparation is not yet supplied by the native preparation owner.
    UnsupportedCellPreparation,
}

pub(crate) fn refused(reason: TextCombineError) -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::UnsupportedFeature,
        "source text combination is unavailable",
    )
    .with_detail(crate::ErrorDetail::TextCombine { reason })
}
