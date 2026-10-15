//! The original document-string ticket while HTML exclusively appends its backing.
use super::*;
use crate::bounded::HtmlConstructionTextAppendCapacity as Capacity;

impl ParseControl {
    /// Prepaid suspension/restoration of this ticket's Weak. The actual native
    /// Arc stays with HTML; a pointer never grants mutation authority.
    pub(crate) fn suspend_text_append_ticket(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.require_document_string(allocation, value)?;
        self.prepare_text_append_restore()?;
        self.suspend_text_append_weak(allocation)?;
        Ok(())
    }
    fn prepare_text_append_restore(&mut self) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (
                WorkCategory::CopyByte,
                (size_of::<u64>() + size_of::<usize>() + size_of::<Option<Weak<Vec<u16>>>>())
                    as u64,
            ),
            (WorkCategory::RecordWrite, 3),
        ])
    }
    fn suspend_text_append_weak(
        &mut self,
        allocation: &mut DocumentStringAllocation,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (
                WorkCategory::CopyByte,
                size_of::<Option<Weak<Vec<u16>>>>() as u64,
            ),
            (WorkCategory::RecordWrite, 1),
        ])?;
        drop(allocation.backing.take());
        Ok(())
    }
    /// Prepares the actual empty pending wrapper before HTML constructs it.
    pub(crate) fn prepare_text_append_allocation(
        &mut self,
    ) -> Result<DocumentStringAllocation, ParseStop> {
        self.prepare_text_append_restore()?;
        self.charge_many(&[
            (WorkCategory::ProviderCall, 1),
            (
                WorkCategory::CopyByte,
                (size_of::<DocumentStringAllocation>() + 2 * size_of::<usize>()) as u64,
            ),
            (WorkCategory::RecordWrite, 3),
        ])?;
        let wrapper_bytes =
            self.admit_document_string_payload(HtmlParseResource::BufferedUtf16Units)?;
        Ok(DocumentStringAllocation {
            owner: self.identity.clone(),
            units: 0,
            vector_capacity: 0,
            wrapper_bytes,
            resource: HtmlParseResource::BufferedUtf16Units,
            data_pointer: 0,
            backing: None,
        })
    }
    /// Bind the real newly created backing once, then suspend its actual Weak
    /// before HTML starts bounded prefix writes. Numeric observations alone do
    /// not issue an allocation ticket for arbitrary caller-owned backing.
    pub(crate) fn bind_text_append_pending(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (
                WorkCategory::CopyByte,
                size_of::<Option<Weak<Vec<u16>>>>() as u64,
            ),
            (WorkCategory::RecordWrite, 1),
        ])?;
        self.bind_document_string(allocation, value)?;
        self.suspend_text_append_weak(allocation)
    }
    pub(crate) fn prepare_text_append_reserve(
        &mut self,
        allocation: &DocumentStringAllocation,
        before: Capacity,
        required: usize,
    ) -> Result<usize, ParseStop> {
        self.require_suspended_text_append(allocation, before)?;
        let Some(plan) = self.geometric_capacity::<u16>(
            before.initialized_len(),
            before.capacity(),
            required - before.initialized_len(),
        )?
        else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        self.charge_many(&[
            (WorkCategory::ReserveByte, plan.old_bytes + plan.new_bytes),
            // HTML's descriptor charges its provider and native fields.
            (WorkCategory::CopyByte, (2 * size_of::<usize>()) as u64),
            (WorkCategory::RecordWrite, 2),
        ])?;
        self.preflight_construction_capacity(plan.old_bytes, plan.new_bytes)?;
        Ok((plan.new_bytes / size_of::<u16>() as u64) as usize)
    }
    fn require_suspended_text_append(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: Capacity,
    ) -> Result<(), ParseStop> {
        self.charge(WorkCategory::RecordVisit, 2)?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        if allocation.backing.is_some()
            || allocation.vector_capacity != value.capacity()
            || allocation.units != value.initialized_len() as u64
            || (allocation.data_pointer != 0 && allocation.data_pointer != value.data_address())
        {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        }
        Ok(())
    }
    /// Records real growth before post-provider polling; refusal reports the
    /// same bank, while successful overshoot stays owned and releasable.
    pub(crate) fn record_text_append_capacity(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: Capacity,
        expected_capacity: usize,
    ) -> Result<(), ParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) || allocation.backing.is_some() {
            return self.fail(ParseStop::InternalInvariant(Invariant::ForeignControl));
        }
        let old = allocation.vector_capacity as u64 * size_of::<u16>() as u64;
        let actual = value.payload_bytes() as u64;
        let current = self
            .usage
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let Some(total) = current.checked_sub(old).and_then(|v| v.checked_add(actual)) else {
            return self.fail(ParseStop::InternalInvariant(Invariant::CapacityMismatch));
        };
        let Some(overlap) = current.checked_add(actual) else {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        };
        allocation.vector_capacity = value.capacity();
        allocation.data_pointer = value.data_address();
        self.set(HtmlParseResource::ManagedCapacityBytes, total);
        self.set(HtmlParseResource::PeakTransientBytes, overlap);
        self.set(HtmlParseResource::PeakTransientBytes, total);
        self.usage.actual_capacity_bytes = total;
        self.usage.actual_capacity_high_water = self.usage.actual_capacity_high_water.max(total);
        if value.capacity() > expected_capacity {
            return self.fail(ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: (expected_capacity * size_of::<u16>()) as u64,
                current: old,
                requested_delta: actual.saturating_sub(old),
            });
        }
        Ok(())
    }
    /// Admission immediately precedes each bounded actual prefix/tail copy.
    pub(crate) fn prepare_text_append_units(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        units: usize,
    ) -> Result<(), ParseStop> {
        self.charge_many(&[
            (WorkCategory::CopyByte, size_of::<u64>() as u64),
            (WorkCategory::RecordWrite, 1),
        ])?;
        self.acquire(allocation.resource, units as u64)?;
        allocation.units += units as u64;
        Ok(())
    }
    /// Final domain admission before native publication; no provider follows.
    pub(crate) fn commit_text_append_domain(
        &mut self,
        allocation: &mut DocumentStringAllocation,
    ) -> Result<(), ParseStop> {
        self.checkpoint()?;
        if allocation.resource == HtmlParseResource::BufferedUtf16Units {
            let managed = self
                .usage
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current;
            let buffered = self
                .usage
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current;
            let Some(total) = managed.checked_add(allocation.units) else {
                return self.representation(HtmlParseResource::ManagedUtf16Units);
            };
            self.check_value(
                HtmlParseResource::ManagedUtf16Units,
                total,
                allocation.units,
            )?;
            self.set(
                HtmlParseResource::BufferedUtf16Units,
                buffered
                    .checked_sub(allocation.units)
                    .expect("actual buffered append units"),
            );
            self.set(HtmlParseResource::ManagedUtf16Units, total);
            allocation.resource = HtmlParseResource::ManagedUtf16Units;
        }
        Ok(())
    }
    pub(crate) fn retain_failed_text_append_ticket(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) {
        if allocation.backing.is_none() {
            self.restore_text_append_ticket(allocation, value);
        } else {
            self.validate_document_string(allocation, value)
                .expect("actual pending backing already bound before stop");
        }
    }
    /// Prepaid commit/recovery restoration. Release only discarded initialized
    /// tail units; retain capacity growth, work and immutable first-stop usage.
    pub(crate) fn restore_text_append_ticket(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &crate::DocumentString,
    ) {
        assert!(Arc::ptr_eq(&allocation.owner, &self.identity));
        assert!(allocation.backing.is_none());
        let discarded = allocation
            .units
            .checked_sub(value.len() as u64)
            .expect("append initialized admitted units");
        self.cleanup_storage(allocation.resource, discarded, 0)
            .expect("actual rolled-back tail");
        allocation.units = value.len() as u64;
        assert_eq!(
            allocation.vector_capacity,
            value.shared_code_units().capacity()
        );
        allocation.data_pointer = value.code_units().as_ptr() as usize;
        allocation.backing = Some(Arc::downgrade(value.shared_code_units()));
    }
}
