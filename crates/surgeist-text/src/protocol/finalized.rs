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
    pub fn finalize(&self, allocations: SpacingAllocations) -> Result<FinalizedLine> {
        let amounts = allocation_amounts(self, allocations)?;
        let inner = crate::engine::layout::finalize_selected(Arc::clone(&self.inner), amounts)?;
        Ok(FinalizedLine {
            inner: Arc::new(inner),
        })
    }
}

fn allocation_amounts(
    selected: &SelectedLine,
    allocations: SpacingAllocations,
) -> Result<Vec<f64>> {
    let count = selected.inner.spacing_slots.len();
    let SpacingAllocations::Complete(allocations) = allocations else {
        return Ok(vec![0.0; count]);
    };
    let mut amounts = vec![None; count];
    for allocation in allocations {
        if !Arc::ptr_eq(&selected.inner, &allocation.slot.selected) {
            return Err(protocol_error(ProtocolError::ForeignSelectedLine));
        }
        let slot = allocation.slot.index;
        let Some(amount) = amounts.get_mut(slot) else {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        };
        if amount.is_some() {
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
        *amount = Some(allocation.amount);
    }
    amounts
        .into_iter()
        .enumerate()
        .map(|(slot, amount)| {
            amount.ok_or_else(|| protocol_error(ProtocolError::MissingSpacingSlot { slot }))
        })
        .collect()
}

impl FinalizedLine {
    /// Replace all selected allocations from this immutable unallocated shape.
    /// No provider shaping, font matching or subtraction from rounded output occurs.
    pub fn with_spacing(&self, allocations: SpacingAllocations) -> Result<FinalizedLine> {
        let amounts = allocation_amounts(&self.selected_line(), allocations)?;
        let inner = crate::engine::layout::refinalize_selected(&self.inner, amounts)?;
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
            allocation: match item.geometry {
                crate::engine::layout::data::FinalizedBoxGeometry::InFlow {
                    flow,
                    box_geometry,
                    alignment,
                    ..
                } => super::AtomicLineAllocation::InFlow {
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
