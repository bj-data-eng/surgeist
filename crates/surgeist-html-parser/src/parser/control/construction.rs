//! The parser's original ledger applied to actual HTML-owned allocation reports.
use super::*;

use surgeist_html::bounded::construction::{
    HtmlConstructionAllocationSnapshot as Snapshot, HtmlConstructionRetired,
};

/// One actual shared owner's bank, never a caller-supplied capacity or refund.
#[derive(Debug)]
pub(crate) struct ConstructionAllocation {
    owner: Arc<()>,
    snapshot: Snapshot,
}
impl ConstructionAllocation {
    pub(crate) fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
}

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
            return Err(self.fail_invariant(HtmlParseInvariant::LedgerState));
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
            return Err(self.fail_invariant(HtmlParseInvariant::LedgerState));
        };
        self.admit_resources(batch, Site::RELEASE)
    }
    /// Preflight the precise old/new reserve overlap before its HTML provider call.
    pub(crate) fn prepare_construction_reserve(
        &mut self,
        snapshot: &Snapshot,
        minimum_capacity: usize,
        ticket: Option<&ConstructionAllocation>,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        self.require_construction_snapshot(snapshot, ticket)?;
        let Some(new_bytes) = minimum_capacity
            .checked_mul(snapshot.element_size())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let old_bytes = snapshot.payload_bytes() as u64;
        let Some(reserve_bytes) = old_bytes.checked_add(new_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, reserve_bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        self.preflight_construction_capacity(old_bytes, new_bytes)
    }
    /// The record count to request so the HTML-owned bank grows geometrically.
    /// `additional` is returned unchanged when the bank already holds it, and
    /// also when its exact layout cannot be admitted, so the existing reserve
    /// gate reports that layout's concrete limit and stop. The bank is
    /// described by its actual ticket; no bank exists before the first reserve.
    pub(crate) fn geometric_construction_additional(
        &mut self,
        ticket: Option<&ConstructionAllocation>,
        len: usize,
        additional: usize,
    ) -> Result<usize, HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(ticket) = ticket else {
            return Ok(additional);
        };
        let snapshot = ticket.snapshot();
        match self.geometric_sized(
            snapshot.element_size(),
            len,
            snapshot.capacity(),
            additional,
            usize::MAX,
        ) {
            Ok(Some(plan)) => Ok(plan.additional),
            Ok(None) => Ok(additional),
            Err(stop) => self.fail(stop),
        }
    }
    pub(crate) fn prepare_construction_share(
        &mut self,
        payload_bytes: usize,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        let Ok(bytes) = u64::try_from(payload_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, bytes),
            (HtmlParseWorkCategory::ProviderCall, 1),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, bytes),
        ])?;
        self.preflight_construction_capacity(0, bytes)
    }
    pub(super) fn preflight_construction_capacity(
        &mut self,
        old: u64,
        new: u64,
    ) -> Result<(), HtmlParseStop> {
        self.preflight_capacity(old, new, false)
    }
    fn require_construction_snapshot(
        &mut self,
        snapshot: &Snapshot,
        ticket: Option<&ConstructionAllocation>,
    ) -> Result<(), HtmlParseStop> {
        if let Some(ticket) = ticket {
            if !Arc::ptr_eq(&ticket.owner, &self.identity) {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ForeignControl,
                ));
            }
            // Initialized lengths may change at a ready commit; the allocation
            // generation, actual capacity and hidden element layout cannot.
            let expected = &ticket.snapshot;
            if expected.id() != snapshot.id()
                || expected.capacity() != snapshot.capacity()
                || expected.element_size() != snapshot.element_size()
                || expected.element_alignment() != snapshot.element_alignment()
            {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::CapacityMismatch,
                ));
            }
        } else if snapshot.capacity() != 0 || snapshot.payload_bytes() != 0 {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        Ok(())
    }
    /// Records actual provider ownership *before* the post-call checkpoint. A
    /// later refusal leaves the ticket describing exactly the retained bank.
    pub(crate) fn observe_construction_allocation(
        &mut self,
        snapshot: &Snapshot,
        ticket: &mut Option<ConstructionAllocation>,
        expected_capacity_bytes: usize,
    ) -> Result<(), HtmlParseStop> {
        self.record_construction_allocation(snapshot, ticket, expected_capacity_bytes)?;
        self.checkpoint()
    }
    pub(crate) fn construction_allocation_failed(&mut self, layout_bytes: usize) -> HtmlParseStop {
        self.latch_stop(HtmlParseStop::AllocationFailed {
            resource: HtmlParseResource::ManagedCapacityBytes,
            layout_bytes: layout_bytes as u64,
        })
    }
    pub(crate) fn record_construction_allocation(
        &mut self,
        snapshot: &Snapshot,
        ticket: &mut Option<ConstructionAllocation>,
        expected_capacity_bytes: usize,
    ) -> Result<(), HtmlParseStop> {
        let old = ticket
            .as_ref()
            .map_or(0, |value| value.snapshot.payload_bytes() as u64);
        if let Some(previous) = ticket.as_ref() {
            if !Arc::ptr_eq(&previous.owner, &self.identity) {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ForeignControl,
                ));
            }
            if !previous.snapshot.id().same_owner_role(snapshot.id())
                || previous.snapshot.element_size() != snapshot.element_size()
                || previous.snapshot.element_alignment() != snapshot.element_alignment()
            {
                return self.fail(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::CapacityMismatch,
                ));
            }
        }
        let actual = snapshot.payload_bytes() as u64;
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
        self.record_capacity(old, actual, expected_capacity_bytes as u64)?;
        // Snapshot/owner clones are fixed provider calls prepaid by the driver.
        *ticket = Some(ConstructionAllocation {
            owner: self.identity.clone(),
            snapshot: snapshot.clone(),
        });
        if actual > expected_capacity_bytes as u64 {
            return self.fail(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: expected_capacity_bytes as u64,
                current: old,
                requested_delta: actual.saturating_sub(old),
            });
        }
        Ok(())
    }
    pub(crate) fn require_construction_retirement(
        &mut self,
        ticket: &ConstructionAllocation,
        snapshot: &Snapshot,
    ) -> Result<(), HtmlParseStop> {
        self.require_construction_snapshot(snapshot, Some(ticket))?;
        if self.live(HtmlParseResource::ManagedCapacityBytes)? < snapshot.payload_bytes() as u64 {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ReleaseUnderflow,
            ));
        }
        Ok(())
    }
    pub(crate) fn retire_construction_allocation(
        &mut self,
        ticket: ConstructionAllocation,
        receipt: HtmlConstructionRetired,
    ) -> Result<(), HtmlParseStop> {
        self.require_construction_retirement(&ticket, receipt.allocation())?;
        self.cleanup_capacity(receipt.allocation().payload_bytes() as u64)
    }
}
