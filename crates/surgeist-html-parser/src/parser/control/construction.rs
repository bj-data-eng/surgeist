//! The parser's original ledger applied to the banks HTML's construction storage grows.
use super::*;

use surgeist_html::bounded::construction::{
    HtmlConstructionBank, HtmlConstructionBankGrowth as Growth,
};

impl ParseControl {
    /// The last checkpoint before the shared owner's ready commit. Only
    /// ordinary/owning edge releases leave the parser's logical graph here;
    /// payload string/attribute retention remains with its existing tickets.
    pub(crate) fn commit_construction_logical(
        &mut self,
        owner: &ControlIdentity,
        requests: &[(HtmlParseResource, u64)],
        edge_remove: u64,
    ) -> Result<(), HtmlParseStop> {
        if requests
            .iter()
            .all(|(resource, _)| *resource == HtmlParseResource::ManagedEdges)
        {
            let mut addition = 0_u64;
            for &(_, delta) in requests {
                addition = addition
                    .checked_add(delta)
                    .ok_or_else(|| self.stop_representation(HtmlParseResource::ManagedEdges))?;
            }
            return self.commit_construction_edges(owner, addition, edge_remove);
        }
        self.require_identity(owner)?;
        // The requests are checked against the caps before the removed edges are
        // credited, so the removal is not netted into the request.
        self.check_acquire_many(requests)?;
        let (mut changes, count) = self.storage_batch(requests)?;
        let Some(admitted) = changes.get_mut(..count) else {
            return Err(self.unexpected_change());
        };
        // The admitted state is the requests with the removal credited: the
        // edges are acquired net of it, or released when it is larger.
        let mut batch = [Change::Release(HtmlResource::ManagedEdges, 0); STORAGE_RESOURCES + 1];
        let mut length = 0_usize;
        let mut edges = None;
        for change in admitted.iter().copied() {
            match change {
                Change::Acquire(HtmlResource::ManagedEdges, delta) => edges = Some(delta),
                other => {
                    if let Some(slot) = batch.get_mut(length) {
                        *slot = other;
                        length = length.saturating_add(1);
                    }
                }
            }
        }
        let added = edges.unwrap_or(0);
        let net = if added >= edge_remove {
            Change::Acquire(
                HtmlResource::ManagedEdges,
                added.saturating_sub(edge_remove),
            )
        } else {
            Change::Release(
                HtmlResource::ManagedEdges,
                edge_remove.saturating_sub(added),
            )
        };
        if let Some(slot) = batch.get_mut(length) {
            *slot = net;
            length = length.saturating_add(1);
        }
        let Some(batch) = batch.get(..length) else {
            return Err(self.unexpected_change());
        };
        self.admit_resources(batch, Site::PLAIN)
    }
    /// The capacity a bank grows to. A bank of the build rows that already has room grows by
    /// the parser's amortized plan, which never reserves spare capacity the limits cannot hold,
    /// and one that has none yet takes exactly the room needed; the vector of interned names grows
    /// the same way. A string of a leaf takes its length, the shared arena of a publication holds
    /// one, and the slots of the name table and the allocation of one name entry take exactly
    /// what they need.
    pub(crate) fn bank_growth_target(&mut self, growth: Growth) -> Result<usize, HtmlParseStop> {
        let minimum = growth.minimum_capacity();
        match growth.bank() {
            HtmlConstructionBank::NodeRows
            | HtmlConstructionBank::Depths
            | HtmlConstructionBank::DepthIndex
            | HtmlConstructionBank::DepthScratch
            | HtmlConstructionBank::Associations
            | HtmlConstructionBank::Leaves
            | HtmlConstructionBank::AttributeRows
            | HtmlConstructionBank::AttributeRuns
            | HtmlConstructionBank::Names => {
                self.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let before = growth.before();
                if before.capacity() == 0 {
                    return Ok(minimum);
                }
                match self.geometric_sized(
                    growth.element_size(),
                    before.initialized_len(),
                    before.capacity(),
                    minimum.saturating_sub(before.initialized_len()),
                    usize::MAX,
                ) {
                    Ok(Some(plan)) => Ok(before
                        .initialized_len()
                        .checked_add(plan.additional)
                        .map_or(minimum, |planned| planned.max(minimum))),
                    Ok(None) => Ok(minimum),
                    Err(stop) => self.fail(stop),
                }
            }
            HtmlConstructionBank::TextArena => {
                self.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                Ok(Self::text_growth_target(
                    growth.before().capacity(),
                    minimum,
                ))
            }
            HtmlConstructionBank::Text
            | HtmlConstructionBank::TextBlock
            | HtmlConstructionBank::PublishedRows
            | HtmlConstructionBank::PublishedChildren
            | HtmlConstructionBank::PublishedAttributes
            | HtmlConstructionBank::PublishedRuns
            | HtmlConstructionBank::PublishedAssociations
            | HtmlConstructionBank::PublishedLeaves
            | HtmlConstructionBank::Wrapper
            | HtmlConstructionBank::NameSlots
            | HtmlConstructionBank::NameEntry => Ok(minimum),
        }
    }

    /// The bytes a bank of `capacity` elements holds, with the wrapper of a string.
    fn bank_bytes(growth: Growth, capacity: usize) -> Option<u64> {
        capacity
            .checked_mul(growth.element_size())
            .and_then(|bytes| bytes.checked_add(growth.wrapper_bytes()))
            .and_then(|bytes| u64::try_from(bytes).ok())
    }

    /// Admits the growth of one bank to `chosen` elements before it is reserved: the overlap of
    /// the old and the new layout is charged as reserved bytes, and the capacity, transient and
    /// layout limits are checked. Returns the bytes of the new layout.
    pub(crate) fn prepare_bank_growth(
        &mut self,
        growth: Growth,
        chosen: usize,
    ) -> Result<u64, HtmlParseStop> {
        self.checkpoint()?;
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let Some(new) = chosen
            .checked_mul(growth.element_size())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .and_then(|_| Self::bank_bytes(growth, chosen))
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        // A bank that has no allocation yet holds no wrapper either.
        let old = if growth.before().capacity() == 0 {
            Some(0)
        } else {
            Self::bank_bytes(growth, growth.before().capacity())
        };
        let Some(old) = old else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(overlap) = old.checked_add(new) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, overlap),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        self.preflight_construction_capacity(old, new)?;
        Ok(new)
    }

    /// Records the capacity a reservation reached in place of the bytes the bank held before it,
    /// and moves `held`, the bytes the parse holds for banks of this kind, by the difference.
    /// `old` is the bytes recorded before and `reached` the capacity of the bank now. The
    /// allocation exists whatever its size, so an overshoot of the `expected` layout is recorded
    /// and then stops the parse.
    pub(crate) fn record_bank_growth(
        &mut self,
        old: u64,
        actual: u64,
        expected: u64,
        held: &mut u64,
    ) -> Result<(), HtmlParseStop> {
        let current = self.live(HtmlParseResource::ManagedCapacityBytes)?;
        if current
            .checked_sub(old)
            .and_then(|v| v.checked_add(actual))
            .is_none()
        {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        }
        if current.checked_add(actual).is_none() {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        }
        let recorded = self.record_capacity(old, actual, expected);
        // Only a record the ledger admitted changes the bytes held; a refused one keeps the old.
        let Some(rest) = held.checked_sub(old) else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        };
        *held = rest.saturating_add(recorded.as_ref().copied().unwrap_or(old));
        recorded?;
        if actual > expected {
            return self.fail(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: expected,
                current: old,
                requested_delta: actual.saturating_sub(old),
            });
        }
        Ok(())
    }

    /// The bytes a record of a growth holds, as the ledger counts them.
    pub(crate) fn bank_record_bytes(
        growth: Growth,
        capacity: usize,
        first: bool,
    ) -> Result<u64, HtmlParseStop> {
        if first && capacity == 0 {
            return Ok(0);
        }
        Self::bank_bytes(growth, capacity).ok_or(HtmlParseStop::InternalInvariant(
            HtmlParseInvariant::CapacityMismatch,
        ))
    }

    /// Credits the bytes the parse holds for the banks of HTML's storage and for the published
    /// arena. This is the only place they are credited.
    pub(crate) fn release_banks(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        self.cleanup_capacity(bytes)
    }
    pub(super) fn preflight_construction_capacity(
        &mut self,
        old: u64,
        new: u64,
    ) -> Result<(), HtmlParseStop> {
        self.preflight_capacity(old, new, false)
    }
    pub(crate) fn construction_allocation_failed(&mut self, layout_bytes: usize) -> HtmlParseStop {
        self.latch_stop(HtmlParseStop::AllocationFailed {
            resource: HtmlParseResource::ManagedCapacityBytes,
            layout_bytes: layout_bytes as u64,
        })
    }
}
