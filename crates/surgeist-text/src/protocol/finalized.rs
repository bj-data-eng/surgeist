//! Selected-request spacing and readback over actual immutable finalized rows.

use std::sync::Arc;

use super::{
    AtomicSlotFeedback, FinalizedLine, LineFeedback, PreparedParticipant, ProtocolError,
    SelectedLine, SpacingAllocations, SpacingSlot, protocol_error,
};
use crate::{Result, TextNumericError, TextNumericPhase};

impl SelectedLine {
    /// Complete receipt-qualified candidates from this exact retained basis.
    /// Actual geometry and expansion admission come from finalized feedback.
    /// Every handle retains the selected allocation, including its preparation.
    pub fn spacing_slots(&self) -> impl ExactSizeIterator<Item = SpacingSlot> + '_ {
        (0..self.inner.spacing_slots.len()).map(|index| SpacingSlot {
            selected: Arc::clone(&self.inner),
            index,
        })
    }
}

impl SpacingSlot {
    #[must_use]
    pub fn selected_line(&self) -> SelectedLine {
        SelectedLine {
            inner: Arc::clone(&self.selected),
        }
    }

    /// Every actual participating selected capability, in content order.
    pub fn participants(&self) -> impl Iterator<Item = &PreparedParticipant> {
        self.selected.spacing_slots[self.index]
            .participants
            .iter()
            .map(|&index| &self.selected.request.participants[index])
    }

    /// Exact units on either side of the actual visual boundary.
    #[must_use]
    pub fn target(&self) -> super::SpacingSlotTarget {
        let unit = |side| super::SpacingUnit {
            selected: Arc::clone(&self.selected),
            slot: self.index,
            side,
        };
        match self.selected.spacing_slots[self.index].target {
            super::selection::SpacingTargetData::Separator { .. } => {
                super::SpacingSlotTarget::Separator(unit(0))
            }
            super::selection::SpacingTargetData::Boundary { .. } => {
                super::SpacingSlotTarget::Boundary {
                    visual_before: unit(0),
                    visual_after: unit(1),
                }
            }
        }
    }
}

impl SelectedLine {
    /// Execute allocation geometry over this exact retained selected basis.
    /// No provider, System, catalogue, or alternative boundary is consulted.
    pub fn finalize(&self, allocations: super::LineAllocations) -> Result<FinalizedLine> {
        self.finalize_allocations(allocations, None)
    }

    /// Execute within a caller-reduced captured feature allowance. Retained
    /// Text input participates in the same transaction before allocation.
    pub fn finalize_with_scope_allowance(
        &self,
        allocations: super::LineAllocations,
        allowance: crate::ScopeAlignmentAllowance,
    ) -> Result<FinalizedLine> {
        self.finalize_allocations(allocations, Some(allowance))
    }

    fn finalize_allocations(
        &self,
        allocations: super::LineAllocations,
        allowance: Option<crate::ScopeAlignmentAllowance>,
    ) -> Result<FinalizedLine> {
        let mut scopes = crate::engine::scope_alignment::allocation_frames(
            &self.inner,
            allocations.scopes,
            allowance,
            &allocations.spacing,
        )?;
        let amounts = allocation_amounts(self, allocations.spacing, &mut scopes.budget)?;
        let inner =
            crate::engine::layout::finalize_selected(Arc::clone(&self.inner), amounts, scopes)?;
        Ok(FinalizedLine {
            inner: Arc::new(inner),
        })
    }
}

fn allocation_amounts(
    selected: &SelectedLine,
    allocations: SpacingAllocations,
    budget: &mut Option<crate::processing::contextual::alignment::ScopeAlignmentBudget>,
) -> Result<Vec<f64>> {
    // The common frame transaction has already admitted this real typed input
    // while it coexisted with frame scratch. Retain that exact storage until
    // consuming the descriptor Vec actually drops its allocation.
    let input_retention = if budget.is_some() {
        Some(crate::engine::scope_alignment::spacing_input_retention(
            &allocations,
        )?)
    } else {
        None
    };
    let count = selected.inner.spacing_slots.len();
    let mut amounts = Vec::new();
    if let Some(budget) = budget.as_mut() {
        budget.add_records(count)?;
        budget.work(count)?;
        budget.reserve(&mut amounts, count)?;
    }
    amounts.resize(count, 0.0);
    let SpacingAllocations::Complete(allocations) = allocations else {
        return Ok(amounts);
    };
    let mut assigned = Vec::new();
    if let Some(budget) = budget.as_mut() {
        budget.add_records(count)?;
        budget.work(count)?;
        budget.reserve(&mut assigned, count)?;
    }
    assigned.resize(count, false);
    for allocation in allocations {
        if !Arc::ptr_eq(&selected.inner, &allocation.slot.selected) {
            return Err(protocol_error(ProtocolError::ForeignSelectedLine));
        }
        let slot = allocation.slot.index;
        let Some(assigned) = assigned.get_mut(slot) else {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        };
        if *assigned {
            return Err(protocol_error(ProtocolError::DuplicateSpacingSlot { slot }));
        }
        if !allocation.amount.is_finite()
            || (matches!(
                selected.inner.request.context.spacing,
                super::SpacingPolicy::InterWord | super::SpacingPolicy::InterCharacter
            ) && allocation.amount < 0.0)
        {
            return Err(crate::error::text_numeric_error(
                "selected spacing allocation",
                TextNumericPhase::Finalization,
                if allocation.amount.is_finite() {
                    TextNumericError::OutOfRange
                } else {
                    TextNumericError::NonFinite
                },
                allocation.amount,
            ));
        }
        *assigned = true;
        amounts[slot] = allocation.amount;
    }
    if let (Some(budget), Some(retention)) = (budget.as_mut(), input_retention) {
        budget.release_bytes(retention.payload_bytes())?;
    }
    if let Some(slot) = assigned.iter().position(|assigned| !*assigned) {
        return Err(protocol_error(ProtocolError::MissingSpacingSlot { slot }));
    }
    let assigned_bytes = assigned.capacity();
    drop(assigned);
    if let Some(budget) = budget.as_mut() {
        budget.release_bytes(assigned_bytes)?;
    }
    Ok(amounts)
}

impl FinalizedLine {
    /// Replace spacing allocations from the immutable selected shape while
    /// retaining the executed scope frames. No provider shaping, font matching
    /// or subtraction from rounded output occurs.
    pub fn with_spacing(&self, allocations: SpacingAllocations) -> Result<FinalizedLine> {
        self.replace_spacing(allocations, None)
    }

    /// Replay spacing while retaining scope placement under a reduced feature
    /// allowance, including coexistence with this borrowed finalized result.
    pub fn with_spacing_with_scope_allowance(
        &self,
        allocations: SpacingAllocations,
        allowance: crate::ScopeAlignmentAllowance,
    ) -> Result<FinalizedLine> {
        self.replace_spacing(allocations, Some(allowance))
    }

    fn replace_spacing(
        &self,
        allocations: SpacingAllocations,
        allowance: Option<crate::ScopeAlignmentAllowance>,
    ) -> Result<FinalizedLine> {
        let mut scopes =
            crate::engine::scope_alignment::replay_frames(&self.inner, allowance, &allocations)?;
        let amounts = allocation_amounts(&self.selected_line(), allocations, &mut scopes.budget)?;
        let inner = crate::engine::layout::refinalize_selected(&self.inner, amounts, scopes)?;
        Ok(FinalizedLine {
            inner: Arc::new(inner),
        })
    }

    /// The exact immutable request which produced these finalized rows.
    #[must_use]
    pub fn selected_line(&self) -> SelectedLine {
        SelectedLine {
            inner: Arc::clone(&self.inner.selected),
        }
    }

    /// Actual line-local output. Fit and deliberate overflow belong to the consumer.
    #[must_use]
    pub fn feedback(&self) -> LineFeedback {
        let metrics = self.inner.data.metrics;
        LineFeedback {
            advance: metrics.advance,
            occupied_start: metrics.occupied_start,
            occupied_end: metrics.occupied_end,
            trailing_whitespace: metrics.trailing_whitespace,
            hanging_start: metrics.hanging_start,
            hanging_end: metrics.hanging_end,
            ascent: metrics.ascent,
            descent: metrics.descent,
            line_height: metrics.line_height,
            baseline: metrics.baseline,
        }
    }

    /// Actual visual slot positions and retained canonical box/flow relations.
    pub fn atomic_slots(&self) -> impl ExactSizeIterator<Item = AtomicSlotFeedback> + '_ {
        self.inner.data.boxes.iter().map(|item| AtomicSlotFeedback {
            occurrence: item.atomic.clone(),
            origin: item.slot,
            scope_displacement: item.scope_displacement,
            allocation: match item.geometry {
                crate::engine::layout::data::FinalizedBoxGeometry::InFlow {
                    flow,
                    box_geometry,
                    alignment,
                    fit,
                    ..
                } => super::AtomicLineAllocation::InFlow {
                    fit,
                    flow,
                    box_geometry,
                    alignment,
                },
                crate::engine::layout::data::FinalizedBoxGeometry::OutOfFlow => {
                    super::AtomicLineAllocation::OutOfFlow
                }
            },
        })
    }
}
