//! Parser policy and accounting for the text of the non-element fields of HTML's storage: the
//! growth of a text that receives an append, and the units and capacity bytes the parse holds
//! for every such string.
use super::*;

use surgeist_html::bounded::construction::HtmlConstructionTextCapacity as Capacity;

/// What the parse holds for one string of a non-element field: its units, and the bytes of its
/// capacity and of the shared wrapper that holds it. It replaces the ticket of the string, whose
/// release the owner of the strings makes once for all of them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextAccount {
    pub(crate) units: u64,
    pub(crate) bytes: u64,
}

impl ParseControl {
    /// The capacity a text of `capacity` units that must hold `minimum` units grows to: the
    /// greater of the minimum and double the capacity, which is clipped to the units of the
    /// largest layout. It depends on the text alone, never on a limit, so the allocation path
    /// of an input is the same under every limit.
    pub(crate) fn text_growth_target(capacity: usize, minimum: usize) -> usize {
        const LARGEST: usize = isize::MAX as usize / size_of::<u16>();
        minimum.max(capacity.saturating_mul(2).min(LARGEST))
    }

    /// Admits one append of `added` units that leaves a text of `before` holding `minimum`
    /// units, before anything is reserved or recorded. The units are checked against the cap
    /// first, so a refusal names them before any byte resource. A text that is full then grows
    /// to the target: the overlap of the old and the new layout is charged as reserved bytes,
    /// and the capacity, transient and layout limits are checked. Returns the capacity to
    /// reserve and the bytes of its layout, or `None` for a text that has room.
    pub(crate) fn admit_text_append(
        &mut self,
        before: Capacity,
        added: usize,
        minimum: usize,
    ) -> Result<(usize, u64), HtmlParseStop> {
        self.checkpoint()?;
        let Ok(units) = u64::try_from(added) else {
            return self.representation(HtmlParseResource::ManagedUtf16Units);
        };
        self.check_acquire_many(&[(HtmlParseResource::ManagedUtf16Units, units)])?;
        if minimum <= before.capacity() {
            return Ok((before.capacity(), 0));
        }
        let target = Self::text_growth_target(before.capacity(), minimum);
        let expected = self.prepare_text_growth(before, target)?;
        Ok((target, expected))
    }

    /// Admits the growth of one text from its capacity to `target` units before it is
    /// reserved. Returns the bytes of the new layout.
    fn prepare_text_growth(
        &mut self,
        before: Capacity,
        target: usize,
    ) -> Result<u64, HtmlParseStop> {
        self.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let Some(new) = target
            .checked_mul(size_of::<u16>())
            .filter(|v| *v <= isize::MAX as usize)
        else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(old) = before.capacity().checked_mul(size_of::<u16>()) else {
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

    /// Records the capacity a reservation reached in place of the bytes the text held before
    /// it, and moves `admitted`, the bytes the parse holds for all its strings, by the
    /// difference. The allocation exists whatever its size, so an overshoot of the `expected`
    /// layout is recorded and then stops the parse.
    pub(crate) fn record_text_growth(
        &mut self,
        before: Capacity,
        after: Capacity,
        expected: u64,
        admitted: &mut u64,
    ) -> Result<(), HtmlParseStop> {
        let Some(actual) = after.capacity().checked_mul(size_of::<u16>()) else {
            return self.representation(HtmlParseResource::ManagedCapacityBytes);
        };
        let Some(old) = before.capacity().checked_mul(size_of::<u16>()) else {
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

    /// Acquires the units an append adds to a text.
    pub(crate) fn acquire_text_units(&mut self, added: usize) -> Result<(), HtmlParseStop> {
        let Ok(units) = u64::try_from(added) else {
            return self.representation(HtmlParseResource::ManagedUtf16Units);
        };
        self.acquire(HtmlParseResource::ManagedUtf16Units, units)
    }

    /// Moves a string that the tokenizer buffered to the managed units, as the stored string of a
    /// record. The ticket stays with the caller.
    pub(crate) fn manage_text_string(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: &surgeist_html::values::DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.move_document_string_to_managed(allocation, value)
    }

    /// Takes what the ticket of a managed string holds, and ends the ticket: its units and the
    /// bytes of its capacity and wrapper, which the parse releases with the strings it holds.
    /// The string is the only holder of its block afterwards, so it can grow in place.
    pub(crate) fn absorb_text_string(
        &mut self,
        allocation: DocumentStringAllocation,
    ) -> Result<TextAccount, HtmlParseStop> {
        if !Arc::ptr_eq(&allocation.owner, &self.identity) {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl,
            ));
        }
        if allocation.resource != HtmlParseResource::ManagedUtf16Units || allocation.shared {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::NonStorageResource,
            ));
        }
        match allocation
            .vector_admitted
            .checked_add(allocation.wrapper_bytes)
        {
            Some(bytes) => Ok(TextAccount {
                units: allocation.units,
                bytes,
            }),
            None => self.representation(HtmlParseResource::ManagedCapacityBytes),
        }
    }

    /// Releases the units and the bytes the parse holds for its strings.
    pub(crate) fn release_text_accounts(
        &mut self,
        units: u64,
        bytes: u64,
    ) -> Result<(), HtmlParseStop> {
        self.cleanup_storage(HtmlParseResource::ManagedUtf16Units, units, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The units of the largest layout: `isize::MAX` bytes of two-byte units.
    const LARGEST: usize = isize::MAX as usize / 2;

    #[test]
    fn a_text_grows_to_double_its_capacity_or_to_the_room_it_needs() {
        assert_eq!(ParseControl::text_growth_target(0, 1), 1);
        assert_eq!(ParseControl::text_growth_target(1, 2), 2);
        assert_eq!(ParseControl::text_growth_target(4, 5), 8);
        assert_eq!(ParseControl::text_growth_target(4, 20), 20);
        assert_eq!(
            ParseControl::text_growth_target(1 << 20, (1 << 20) + 1),
            2 << 20
        );
    }

    #[test]
    fn the_growth_target_never_exceeds_the_largest_layout_when_the_room_needed_fits() {
        // Double the capacity is beyond the largest layout, and the text needs a few units more.
        let capacity = LARGEST / 2 + 8;
        assert!(capacity.saturating_mul(2) > LARGEST);
        assert_eq!(
            ParseControl::text_growth_target(capacity, capacity + 3),
            LARGEST
        );
        assert_eq!(ParseControl::text_growth_target(LARGEST, LARGEST), LARGEST);
        // The room needed is never reduced: a minimum above the largest layout stays, and the
        // admission of the layout refuses it.
        assert_eq!(
            ParseControl::text_growth_target(capacity, LARGEST + 1),
            LARGEST + 1
        );
        assert_eq!(
            ParseControl::text_growth_target(usize::MAX, usize::MAX),
            usize::MAX
        );
    }
}
