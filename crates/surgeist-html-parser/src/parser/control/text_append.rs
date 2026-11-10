//! The original document-string ticket while HTML exclusively appends its backing.
use super::*;
use surgeist_html::bounded::construction::HtmlConstructionTextAppendCapacity as Capacity;

impl ParseControl {
    /// Prepaid suspension/restoration of this ticket's Weak. The actual native
    /// Arc stays with HTML; a pointer never grants mutation authority.
    pub(crate) fn suspend_text_append_ticket(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.require_document_string(allocation, value)?;
        self.prepare_text_append_restore()?;
        self.suspend_text_append_weak(allocation)?;
        Ok(())
    }
    fn prepare_text_append_restore(&mut self) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (HtmlParseWorkCategory::ProviderCall, 1),
            (
                HtmlParseWorkCategory::CopyByte,
                const {
                    (size_of::<u64>() + size_of::<usize>() + size_of::<Option<WeakTextBlock>>())
                        as u64
                },
            ),
            (HtmlParseWorkCategory::RecordWrite, 3),
        ])
    }
    fn suspend_text_append_weak(
        &mut self,
        allocation: &mut DocumentStringAllocation,
    ) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (HtmlParseWorkCategory::ProviderCall, 1),
            (
                HtmlParseWorkCategory::CopyByte,
                size_of::<Option<WeakTextBlock>>() as u64,
            ),
            (HtmlParseWorkCategory::RecordWrite, 1),
        ])?;
        drop(allocation.backing.take());
        Ok(())
    }
    /// Prepares the actual empty pending wrapper before HTML constructs it.
    pub(crate) fn prepare_text_append_allocation(
        &mut self,
    ) -> Result<DocumentStringAllocation, HtmlParseStop> {
        self.prepare_text_append_restore()?;
        self.charge_many(&[
            (HtmlParseWorkCategory::ProviderCall, 1),
            (
                HtmlParseWorkCategory::CopyByte,
                const { (size_of::<DocumentStringAllocation>() + 2 * size_of::<usize>()) as u64 },
            ),
            (HtmlParseWorkCategory::RecordWrite, 3),
        ])?;
        let wrapper_bytes =
            self.admit_document_string_payload(HtmlParseResource::BufferedUtf16Units)?;
        Ok(DocumentStringAllocation {
            owner: self.identity.clone(),
            units: 0,
            vector_capacity: 0,
            vector_admitted: 0,
            wrapper_bytes,
            resource: HtmlParseResource::BufferedUtf16Units,
            data_pointer: 0,
            backing: None,
            shared: false,
        })
    }
    /// Bind the real newly created backing once, then suspend its actual Weak
    /// before HTML starts bounded prefix writes. Numeric observations alone do
    /// not issue an allocation ticket for arbitrary caller-owned backing.
    pub(crate) fn bind_text_append_pending(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.charge_many(&[
            (
                HtmlParseWorkCategory::CopyByte,
                size_of::<Option<WeakTextBlock>>() as u64,
            ),
            (HtmlParseWorkCategory::RecordWrite, 1),
        ])?;
        self.bind_document_string(allocation, value)?;
        self.suspend_text_append_weak(allocation)
    }
    pub(crate) fn prepare_text_append_reserve(
        &mut self,
        allocation: &DocumentStringAllocation,
        before: Capacity,
        required: usize,
    ) -> Result<usize, HtmlParseStop> {
        self.require_suspended_text_append(allocation, before)?;
        // HTML requests a capacity that holds the initialized units.
        let Some(additional) = required.checked_sub(before.initialized_len()) else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        };
        let Some(plan) = self.geometric_capacity::<u16>(
            before.initialized_len(),
            before.capacity(),
            additional,
        )?
        else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        };
        let Some(reserve_bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::ReserveByte, reserve_bytes),
            // HTML's descriptor charges its provider and native fields.
            (
                HtmlParseWorkCategory::CopyByte,
                const { (2 * size_of::<usize>()) as u64 },
            ),
            (HtmlParseWorkCategory::RecordWrite, 2),
        ])?;
        self.preflight_construction_capacity(plan.old_bytes, plan.new_bytes)?;
        const UNIT_BYTES: u64 = size_of::<u16>() as u64;
        Ok((plan.new_bytes / UNIT_BYTES) as usize)
    }
    fn require_suspended_text_append(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: Capacity,
    ) -> Result<(), HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if allocation.backing.is_some()
            || allocation.vector_capacity != value.capacity()
            || allocation.units != value.initialized_len() as u64
            || (allocation.data_pointer != 0 && allocation.data_pointer != value.data_address())
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        Ok(())
    }
    /// Records real growth before post-provider polling. The ticket describes
    /// the backing as it is. A successful overshoot is admitted and released as
    /// admitted; a record the ledger refuses leaves the admitted bytes as they
    /// were, and the refusal is the stop.
    pub(crate) fn record_text_append_capacity(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: Capacity,
        expected_capacity: usize,
    ) -> Result<(), HtmlParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        // Growth is recorded while the ticket is pending; a ticket already
        // bound to a backing is in another allocation state than the one this
        // controller charged, whoever holds it.
        if allocation.backing.is_some() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        const UNIT_BYTES: u64 = size_of::<u16>() as u64;
        let old = allocation.vector_admitted;
        let actual = value.payload_bytes() as u64;
        let current = self.live(HtmlParseResource::ManagedCapacityBytes)?;
        if current
            .checked_sub(old)
            .and_then(|v| v.checked_add(actual))
            .is_none()
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        if current.checked_add(actual).is_none() {
            return self.representation(HtmlParseResource::PeakTransientBytes);
        }
        let recorded = self.record_capacity(
            old,
            actual,
            (expected_capacity as u64).saturating_mul(UNIT_BYTES),
        );
        // The ticket describes the backing as it is; only a record the ledger
        // admitted changes the bytes the ticket releases.
        allocation.vector_capacity = value.capacity();
        allocation.data_pointer = value.data_address();
        allocation.vector_admitted = recorded?;
        if value.capacity() > expected_capacity {
            return self.fail(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                limit: expected_capacity.saturating_mul(size_of::<u16>()) as u64,
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
    ) -> Result<(), HtmlParseStop> {
        let total = u64::try_from(units).ok().and_then(|units| {
            allocation
                .units
                .checked_add(units)
                .map(|total| (units, total))
        });
        let Some((units, total)) = total else {
            return self.representation(allocation.resource);
        };
        self.charge_many(&[
            (HtmlParseWorkCategory::CopyByte, size_of::<u64>() as u64),
            (HtmlParseWorkCategory::RecordWrite, 1),
        ])?;
        self.acquire(allocation.resource, units)?;
        allocation.units = total;
        Ok(())
    }
    /// Final domain admission before native publication; no provider follows.
    pub(crate) fn commit_text_append_domain(
        &mut self,
        allocation: &mut DocumentStringAllocation,
    ) -> Result<(), HtmlParseStop> {
        self.checkpoint()?;
        if allocation.resource == HtmlParseResource::BufferedUtf16Units {
            // The appended units were admitted as buffered text. The managed
            // cap is checked first: the units are released from the buffered
            // total only once they fit.
            self.check_resources(
                &[Change::Acquire(
                    HtmlResource::ManagedUtf16Units,
                    allocation.units,
                )],
                Site::PLAIN,
            )?;
            self.admit_resources(
                &[
                    Change::Release(HtmlResource::BufferedUtf16Units, allocation.units),
                    Change::Acquire(HtmlResource::ManagedUtf16Units, allocation.units),
                ],
                Site::PLAIN,
            )?;
            allocation.resource = HtmlParseResource::ManagedUtf16Units;
        }
        Ok(())
    }
    /// Keeps a failed append's ticket for terminal disposal: an unbound ticket
    /// is restored to the value it describes, a bound one is checked against
    /// it. A refusal latches its stop and leaves the ticket with the caller.
    pub(crate) fn retain_failed_text_append_ticket(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        if allocation.backing.is_none() {
            self.restore_text_append_ticket(allocation, value)
        } else {
            self.validate_document_string(allocation, value)
        }
    }
    /// Prepaid commit/recovery restoration. Release only discarded initialized
    /// tail units; retain capacity growth, work and immutable first-stop usage.
    /// Every precondition is checked before anything changes: a refusal latches
    /// its stop, credits nothing and leaves the ticket with the caller.
    pub(crate) fn restore_text_append_ticket(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        #[cfg(test)]
        if self.take_text_append_restore_fault() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if allocation.backing.is_some() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        let kept = value.len() as u64;
        let Some(discarded) = allocation.units.checked_sub(kept) else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ReleaseUnderflow,
            ));
        };
        if allocation.vector_capacity != value.retained_units() {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        self.cleanup_storage(allocation.resource, discarded, 0)?;
        allocation.units = kept;
        allocation.data_pointer = value.code_units().as_ptr() as usize;
        allocation.backing = Some(value.block().downgrade());
        Ok(())
    }
}
