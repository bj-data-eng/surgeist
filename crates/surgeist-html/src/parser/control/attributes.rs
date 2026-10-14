//! Original parser policy and tickets for HTML-owned exclusive attribute edits.
use super::*;
use crate::bounded::{
    HtmlConstructionAttributeCapacity as Capacity, HtmlConstructionAttributeWork as Work,
};
impl ParseControl {
    #[cfg(test)]
    pub(crate) fn test_limit(&mut self, resource: HtmlParseResource, limit: u64) {
        self.limits[resource as usize] = limit;
    }
    #[cfg(test)]
    pub(crate) fn observe_construction_attribute_append(&mut self) {
        self.observe_attribute_append(1);
    }
    pub(crate) fn cleanup_auxiliary_entries(&mut self, entries: u64) -> Result<(), ParseStop> {
        self.cleanup_storage(HtmlParseResource::AuxiliaryEntries, entries, 0)
    }
    pub(crate) fn admit_attribute_work(&mut self, work: Work) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::RecordVisit, work.record_visits as u64),
            (WorkCategory::ProviderCall, work.provider_calls as u64),
            (WorkCategory::InspectByte, work.inspected_bytes as u64),
            (WorkCategory::CopyByte, work.copied_bytes as u64),
            (WorkCategory::RecordWrite, work.record_writes as u64),
        ])?;
        if work.atomic_layout_bytes != 0 {
            self.admit_atomic_layout(work.atomic_layout_bytes)?;
        }
        if work.auxiliary_entries != 0 {
            self.acquire(
                HtmlParseResource::AuxiliaryEntries,
                work.auxiliary_entries as u64,
            )?;
        }
        Ok(())
    }
    pub(crate) fn prepare_controlled_attribute_edit(
        &mut self,
        values: &Arc<Vec<crate::Attribute>>,
        shared: &SharedAllocation<Vec<crate::Attribute>>,
        vector: &VectorCapacity<crate::Attribute>,
    ) -> Result<(), ParseStop> {
        self.prepare_attribute_edit(values, shared, vector)
    }
    pub(crate) fn suspend_attribute_weak(
        &mut self,
        shared: &mut SharedAllocation<Vec<crate::Attribute>>,
    ) {
        drop(std::mem::replace(&mut shared.shared, Weak::new()));
    }
    pub(crate) fn restore_attribute_weak(
        &mut self,
        values: &Arc<Vec<crate::Attribute>>,
        shared: &mut SharedAllocation<Vec<crate::Attribute>>,
    ) {
        if shared.shared.as_ptr() != Arc::as_ptr(values) {
            shared.shared = Arc::downgrade(values);
        }
    }
    pub(crate) fn require_incoming_attribute_vector(
        &mut self,
        values: &Vec<crate::Attribute>,
        allocation: &VectorCapacity<crate::Attribute>,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordVisit, 2),
        ])?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if values.capacity() != allocation.capacity || values.is_empty() {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        Ok(())
    }
    pub(crate) fn prepare_controlled_attribute_reserve(
        &mut self,
        before: Capacity,
        minimum: usize,
        vector: &VectorCapacity<crate::Attribute>,
    ) -> Result<usize, ParseStop> {
        self.checkpoint()?;
        self.charge(WorkCategory::RecordVisit, 2)?;
        if !Arc::ptr_eq(&vector.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if vector.capacity != before.capacity {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        let Some(new) = minimum
            .checked_mul(size_of::<crate::Attribute>())
            .filter(|v| *v <= isize::MAX as usize)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let old = before.capacity * size_of::<crate::Attribute>();
        let Some(overlap_bytes) = old.checked_add(new) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (WorkCategory::ReserveByte, overlap_bytes as u64),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        self.preflight_construction_capacity(old as u64, new as u64)?;
        Ok(new)
    }
    pub(crate) fn record_controlled_attribute_capacity(
        &mut self,
        after: Capacity,
        vector: &mut VectorCapacity<crate::Attribute>,
        expected: usize,
    ) -> Result<(), ParseStop> {
        let old = vector.capacity * size_of::<crate::Attribute>();
        let actual = after.capacity * size_of::<crate::Attribute>();
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(total) = current
            .checked_sub(old as u64)
            .and_then(|v| v.checked_add(actual as u64))
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        vector.capacity = after.capacity;
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        let Some(overlap) = current.checked_add(actual as u64) else {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        };
        self.set(HtmlParseResource::PeakTransientBytes, overlap);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        if actual > expected {
            return self.fail(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: expected as u64,
                current: old as u64,
                requested_delta: (actual - old) as u64,
            });
        }
        Ok(())
    }
}
