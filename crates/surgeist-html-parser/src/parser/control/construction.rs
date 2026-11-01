//! The parser's original ledger applied to actual HTML-owned allocation reports.
use super::*;
use crate::bounded::{HtmlConstructionAllocationSnapshot as Snapshot, HtmlConstructionRetired};

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
    ) -> Result<(), ParseStop> {
        if requests
            .iter()
            .all(|(resource, _)| *resource == HtmlParseResource::ManagedEdges)
        {
            let mut addition = 0_u64;
            for &(_, delta) in requests {
                addition = addition.checked_add(delta).ok_or_else(|| {
                    self.representation::<()>(HtmlParseResource::ManagedEdges)
                        .unwrap_err()
                })?;
            }
            return self.commit_construction_edges(owner, addition, edge_remove);
        }
        self.require_identity(owner)?;
        let mut values = self.plan_storage(requests)?;
        let Some(edges) = values[HtmlParseResource::ManagedEdges as usize].checked_sub(edge_remove)
        else {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        };
        values[HtmlParseResource::ManagedEdges as usize] = edges;
        for resource in RESOURCES {
            if resource.storage() {
                self.set(resource, values[resource as usize]);
            }
        }
        Ok(())
    }
    /// Preflight the precise old/new reserve overlap before its HTML provider call.
    pub(crate) fn prepare_construction_reserve(
        &mut self,
        snapshot: &Snapshot,
        minimum_capacity: usize,
        ticket: Option<&ConstructionAllocation>,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        self.charge(WorkCategory::RecordVisit, 2)?;
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
            (WorkCategory::ReserveByte, reserve_bytes),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, size_of::<usize>() as u64),
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
    ) -> Result<usize, ParseStop> {
        self.charge(WorkCategory::RecordVisit, 1)?;
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
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        let Ok(bytes) = u64::try_from(payload_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (WorkCategory::ReserveByte, bytes),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, bytes),
        ])?;
        self.preflight_construction_capacity(0, bytes)
    }
    pub(super) fn preflight_construction_capacity(
        &mut self,
        old: u64,
        new: u64,
    ) -> Result<(), ParseStop> {
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(without_old) = current.checked_sub(old) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        let (Some(total), Some(overlap)) = (without_old.checked_add(new), current.checked_add(new))
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.check_value(
            HtmlParseResource::ManagedCapacityBytes,
            total,
            new.saturating_sub(old),
        )?;
        self.check_value(HtmlParseResource::PeakTransientBytes, overlap, new)?;
        self.check_value(HtmlParseResource::AtomicLayoutBytes, new, new)?;
        self.set(HtmlParseResource::PeakTransientBytes, overlap);
        self.set(HtmlParseResource::AtomicLayoutBytes, new);
        self.usage.largest_atomic_layout = self.usage.largest_atomic_layout.max(new);
        Ok(())
    }
    fn require_construction_snapshot(
        &mut self,
        snapshot: &Snapshot,
        ticket: Option<&ConstructionAllocation>,
    ) -> Result<(), ParseStop> {
        if let Some(ticket) = ticket {
            if !Arc::ptr_eq(&ticket.owner, &self.identity) {
                return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
            // Initialized lengths may change at a ready commit; the allocation
            // generation, actual capacity and hidden element layout cannot.
            let expected = &ticket.snapshot;
            if expected.id() != snapshot.id()
                || expected.capacity() != snapshot.capacity()
                || expected.element_size() != snapshot.element_size()
                || expected.element_alignment() != snapshot.element_alignment()
            {
                return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
            }
        } else if snapshot.capacity() != 0 || snapshot.payload_bytes() != 0 {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
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
    ) -> Result<(), ParseStop> {
        self.record_construction_allocation(snapshot, ticket, expected_capacity_bytes)?;
        self.checkpoint()
    }
    pub(crate) fn construction_allocation_failed(&mut self, layout_bytes: usize) -> ParseStop {
        self.fail::<()>(ParseStop::AllocationFailed {
            resource: HtmlParseResource::ManagedCapacityBytes,
            layout_bytes: layout_bytes as u64,
        })
        .unwrap_err()
    }
    pub(crate) fn record_construction_allocation(
        &mut self,
        snapshot: &Snapshot,
        ticket: &mut Option<ConstructionAllocation>,
        expected_capacity_bytes: usize,
    ) -> Result<(), ParseStop> {
        let old = ticket
            .as_ref()
            .map_or(0, |value| value.snapshot.payload_bytes() as u64);
        if let Some(previous) = ticket.as_ref() {
            if !Arc::ptr_eq(&previous.owner, &self.identity) {
                return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
            }
            if !previous.snapshot.id().same_owner_role(snapshot.id())
                || previous.snapshot.element_size() != snapshot.element_size()
                || previous.snapshot.element_alignment() != snapshot.element_alignment()
            {
                return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
            }
        }
        let actual = snapshot.payload_bytes() as u64;
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(total) = current.checked_sub(old).and_then(|v| v.checked_add(actual)) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(overlap) = current.checked_add(actual) else {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        };
        // Snapshot/owner clones are fixed provider calls prepaid by the driver.
        *ticket = Some(ConstructionAllocation {
            owner: self.identity.clone(),
            snapshot: snapshot.clone(),
        });
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, overlap);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        if actual > expected_capacity_bytes as u64 {
            return self.fail(ParseStop::LimitExceeded {
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
    ) -> Result<(), ParseStop> {
        self.require_construction_snapshot(snapshot, Some(ticket))?;
        if self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            < snapshot.payload_bytes() as u64
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow));
        }
        Ok(())
    }
    pub(crate) fn retire_construction_allocation(
        &mut self,
        ticket: ConstructionAllocation,
        receipt: HtmlConstructionRetired,
    ) -> Result<(), ParseStop> {
        self.require_construction_retirement(&ticket, receipt.allocation())?;
        self.cleanup_capacity(receipt.allocation().payload_bytes() as u64)
    }
}
