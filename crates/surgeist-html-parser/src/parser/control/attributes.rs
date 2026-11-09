//! Original parser policy and tickets for HTML-owned exclusive attribute edits.
use super::*;

use surgeist_html::bounded::construction::{
    HtmlConstructionAttributeCapacity as Capacity, HtmlConstructionAttributeWork as Work,
};
impl ParseControl {
    #[cfg(test)]
    pub(crate) fn observe_construction_attribute_append(&mut self) {
        self.observe_attribute_append(1);
    }
    pub(crate) fn cleanup_auxiliary_entries(&mut self, entries: u64) -> Result<(), HtmlParseStop> {
        self.cleanup_storage(HtmlParseResource::AuxiliaryEntries, entries, 0)
    }
    pub(crate) fn admit_attribute_work(&mut self, work: Work) -> Result<(), HtmlParseStop> {
        self.charge_changes(&attribute_edit_work(&work))?;
        if work.atomic_layout_bytes != 0 {
            self.admit_atomic_layout(work.atomic_layout_bytes)?;
        }
        if work.auxiliary_entries != 0 {
            self.acquire_change(attribute_edit_entries(&work))?;
        }
        Ok(())
    }
    pub(crate) fn prepare_controlled_attribute_edit(
        &mut self,
        values: &Arc<Vec<surgeist_html::values::Attribute>>,
        shared: &SharedAllocation<Vec<surgeist_html::values::Attribute>>,
        vector: &VectorCapacity<surgeist_html::values::Attribute>,
    ) -> Result<(), HtmlParseStop> {
        self.prepare_attribute_edit(values, shared, vector)
    }
    pub(crate) fn suspend_attribute_weak(
        &mut self,
        shared: &mut SharedAllocation<Vec<surgeist_html::values::Attribute>>,
    ) {
        drop(std::mem::replace(&mut shared.shared, Weak::new()));
    }
    pub(crate) fn restore_attribute_weak(
        &mut self,
        values: &Arc<Vec<surgeist_html::values::Attribute>>,
        shared: &mut SharedAllocation<Vec<surgeist_html::values::Attribute>>,
    ) {
        if shared.shared.as_ptr() != Arc::as_ptr(values) {
            shared.shared = Arc::downgrade(values);
        }
    }
    pub(crate) fn require_incoming_attribute_vector(
        &mut self,
        values: &Vec<surgeist_html::values::Attribute>,
        allocation: &VectorCapacity<surgeist_html::values::Attribute>,
    ) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (HtmlParseWorkCategory::ProviderCall, 1),
            (HtmlParseWorkCategory::RecordVisit, 2),
        ])?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if values.capacity() != allocation.capacity || values.is_empty() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        Ok(())
    }
    pub(crate) fn prepare_controlled_attribute_reserve(
        &mut self,
        before: Capacity,
        minimum: usize,
        vector: &VectorCapacity<surgeist_html::values::Attribute>,
    ) -> Result<usize, HtmlParseStop> {
        self.checkpoint()?;
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if !Arc::ptr_eq(&vector.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if vector.capacity != before.capacity {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        let Some(new) = minimum
            .checked_mul(size_of::<surgeist_html::values::Attribute>())
            .filter(|v| *v <= isize::MAX as usize)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(old) = before
            .capacity
            .checked_mul(size_of::<surgeist_html::values::Attribute>())
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(overlap_bytes) = old.checked_add(new) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, overlap_bytes as u64),
            (HtmlParseWorkCategory::RecordWrite, 1),
            (HtmlParseWorkCategory::CopyByte, size_of::<usize>() as u64),
        ])?;
        self.preflight_construction_capacity(old as u64, new as u64)?;
        Ok(new)
    }
    pub(crate) fn record_controlled_attribute_capacity(
        &mut self,
        after: Capacity,
        vector: &mut VectorCapacity<surgeist_html::values::Attribute>,
        expected: usize,
    ) -> Result<(), HtmlParseStop> {
        let (Some(old), Some(actual)) = (
            vector
                .capacity
                .checked_mul(size_of::<surgeist_html::values::Attribute>()),
            after
                .capacity
                .checked_mul(size_of::<surgeist_html::values::Attribute>()),
        ) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let current = self.live(HtmlParseResource::ManagedCapacityBytes)?;
        if current
            .checked_sub(old as u64)
            .and_then(|v| v.checked_add(actual as u64))
            .is_none()
        {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        }
        if current.checked_add(actual as u64).is_none() {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        }
        self.record_capacity(old as u64, actual as u64, expected as u64)?;
        vector.capacity = after.capacity;
        if actual > expected {
            return self.fail(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: expected as u64,
                current: old as u64,
                requested_delta: actual.saturating_sub(old) as u64,
            });
        }
        Ok(())
    }
}
