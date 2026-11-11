//! Original parser policy and accounting for the growth of HTML-owned attribute lists.
use super::*;

use surgeist_html::bounded::construction::HtmlConstructionAttributeCapacity as Capacity;
impl ParseControl {
    /// The capacity a list that must hold `minimum` attributes grows to: the greater of the
    /// minimum and double the capacity it has. It depends on the list alone, never on a limit,
    /// so the allocation path of an input is the same under every limit.
    pub(crate) fn attribute_growth_target(&self, before: Capacity, minimum: usize) -> usize {
        minimum.max(before.capacity.saturating_mul(2))
    }

    /// Admits the growth of one list from its capacity to `target` attributes before it is
    /// reserved: the overlap of the old and the new layout is charged as reserved bytes, and the
    /// capacity, transient and layout limits are checked. Returns the bytes of the new layout.
    pub(crate) fn prepare_attribute_growth(
        &mut self,
        before: Capacity,
        target: usize,
    ) -> Result<u64, HtmlParseStop> {
        self.checkpoint()?;
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let Some(new) = target
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
        Ok(new as u64)
    }

    /// Records the capacity a reservation reached in place of the bytes the list held before it,
    /// and moves `admitted`, the bytes the parse holds for all attribute lists, by the difference.
    /// The allocation exists whatever its size, so an overshoot of the `expected` layout is
    /// recorded and then stops the parse.
    pub(crate) fn record_attribute_growth(
        &mut self,
        before: Capacity,
        after: Capacity,
        expected: u64,
        admitted: &mut u64,
    ) -> Result<(), HtmlParseStop> {
        let Some(actual) = after
            .capacity
            .checked_mul(size_of::<surgeist_html::values::Attribute>())
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(old) = before
            .capacity
            .checked_mul(size_of::<surgeist_html::values::Attribute>())
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let (old, actual) = (old as u64, actual as u64);
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
        let Some(held) = admitted.checked_sub(old) else {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        };
        *admitted = held.saturating_add(recorded.as_ref().copied().unwrap_or(old));
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

    /// Takes the accounting of a list the parse built and handed to HTML's storage: the bytes of
    /// its capacity and of the shared wrapper the storage holds it in, which the parse releases
    /// with the lists it holds.
    pub(crate) fn absorb_attribute_list(
        &mut self,
        capacity: VectorCapacity<surgeist_html::values::Attribute>,
        shared: SharedAllocation<Vec<surgeist_html::values::Attribute>>,
    ) -> Result<u64, HtmlParseStop> {
        if !Arc::ptr_eq(&capacity.owner, &self.identity)
            || !Arc::ptr_eq(&shared.owner, &self.identity)
        {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        match capacity.admitted.checked_add(shared.bytes) {
            Some(bytes) => Ok(bytes),
            None => self.representation(HtmlParseResource::ManagedCapacityBytes),
        }
    }

    /// Releases the bytes the parse holds for attribute lists.
    pub(crate) fn release_attribute_lists(&mut self, bytes: u64) -> Result<(), HtmlParseStop> {
        self.cleanup_capacity(bytes)
    }
}
