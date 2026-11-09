//! Appending units to the unique backing of a registered document string.
use super::*;
use surgeist_html::values::DocumentString;

/// A refused extension. The string is handed back with its ticket still
/// describing it exactly: spare capacity the refusal had already reserved
/// stays reserved and recorded, and no unit was appended.
pub(crate) struct StringExtensionRefused {
    pub(crate) stop: HtmlParseStop,
    pub(crate) value: DocumentString,
}

impl ParseControl {
    /// Appends `units` to the end of a registered string in place.
    ///
    /// The string must be the only holder of its backing, as the registry
    /// keeps a retired raw value: a second holder is refused, because the
    /// units would then be visible through it. A full backing grows to the
    /// geometric capacity of `try_push_geometric`, so repeated appends copy
    /// each unit a constant number of times. Growth is admitted against the
    /// capacity limits before it is reserved, each copied segment is charged
    /// before it is written, and the units are admitted to the ticket's
    /// resource before the first is written, so nothing can refuse once the
    /// first unit is in.
    pub(crate) fn extend_managed_document_string(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        value: DocumentString,
        units: &[u16],
    ) -> Result<DocumentString, StringExtensionRefused> {
        if let Err(stop) = self.admit_string_extension(allocation, &value) {
            return Err(StringExtensionRefused { stop, value });
        }
        let mut shared = Arc::clone(value.shared_code_units());
        drop(value);
        // The ticket's Weak is the only other pointer to the backing. It goes
        // while the backing is mutated and returns on every path.
        drop(allocation.backing.take());
        let result = match Arc::get_mut(&mut shared) {
            Some(vector) => self.extend_vector(allocation, vector, units),
            None => self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::SharedAllocationAliased,
            )),
        };
        allocation.backing = Some(Arc::downgrade(&shared));
        let value = DocumentString::from_shared_code_units(shared);
        match result {
            Ok(()) => Ok(value),
            Err(stop) => Err(StringExtensionRefused { stop, value }),
        }
    }

    /// The fixed checks and charges that precede the suspension of the ticket.
    fn admit_string_extension(
        &mut self,
        allocation: &DocumentStringAllocation,
        value: &DocumentString,
    ) -> Result<(), HtmlParseStop> {
        self.require_document_string(allocation, value)?;
        if allocation.resource != HtmlParseResource::ManagedUtf16Units {
            return self.fail(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::CapacityMismatch,
            ));
        }
        self.charge_many(&[
            // The unique-holder check and the restored Weak.
            (HtmlParseWorkCategory::ProviderCall, 2),
            (
                HtmlParseWorkCategory::CopyByte,
                const { (size_of::<u64>() + size_of::<Option<Weak<Vec<u16>>>>()) as u64 },
            ),
            (HtmlParseWorkCategory::RecordWrite, 2),
        ])
    }

    fn extend_vector(
        &mut self,
        allocation: &mut DocumentStringAllocation,
        vector: &mut Vec<u16>,
        units: &[u16],
    ) -> Result<(), HtmlParseStop> {
        let added =
            u64::try_from(units.len()).or_else(|_| self.representation(allocation.resource))?;
        let Some(total) = allocation.units.checked_add(added) else {
            return self.representation(allocation.resource);
        };
        let plan = match self.geometric_plan(vector, units.len()) {
            Ok(plan) => plan,
            Err(stop) => return self.fail(stop),
        };
        if let Some(plan) = plan {
            let Some(bytes) = plan.old_bytes.checked_add(plan.new_bytes) else {
                return self.representation(HtmlParseResource::ManagedCapacityBytes);
            };
            self.charge_many(&[
                (HtmlParseWorkCategory::ReserveByte, bytes),
                (HtmlParseWorkCategory::ProviderCall, 1),
            ])?;
            let reserved = self.reserve(vector, plan, &mut allocation.vector_admitted);
            // The ticket describes the allocation as it is, refused or not; its
            // admitted bytes change only when the ledger admits the record.
            allocation.vector_capacity = vector.capacity();
            allocation.data_pointer = vector.as_ptr() as usize;
            reserved?;
        }
        const SEGMENT: usize = WORK_QUANTUM / size_of::<u16>();
        for segment in units.chunks(SEGMENT) {
            self.charge(
                HtmlParseWorkCategory::CopyByte,
                std::mem::size_of_val(segment) as u64,
            )?;
        }
        self.acquire(allocation.resource, added)?;
        vector.extend_from_slice(units);
        allocation.units = total;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlParseLimitsBuilder;

    fn control() -> ParseControl {
        ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        )
    }

    fn current(c: &ParseControl, resource: HtmlParseResource) -> u64 {
        c.usage().resource(resource).current()
    }

    /// A registered string of `units`, as the registry holds a raw value: frozen
    /// from a buffer grown a unit at a time and moved to the managed domain.
    fn registered(
        c: &mut ParseControl,
        units: &[u16],
    ) -> (DocumentString, DocumentStringAllocation) {
        let mut buffer = OwnedVec::new(HtmlParseResource::BufferedUtf16Units).unwrap();
        for &unit in units {
            buffer.try_push_geometric(c, unit).unwrap();
        }
        let (value, mut allocation) = c.freeze_document_string(buffer).unwrap();
        c.move_document_string_to_managed(&mut allocation, &value)
            .unwrap();
        (value, allocation)
    }

    fn work(c: &ParseControl, category: HtmlParseWorkCategory) -> u64 {
        c.usage().work(category)
    }

    #[test]
    fn units_that_fit_the_capacity_are_copied_without_a_reservation() {
        let mut c = control();
        let (value, mut allocation) = registered(&mut c, &[1, 2, 3, 4, 5]);
        // Capacities 1, 2, 4 and 8 hold five units.
        assert_eq!(value.shared_code_units().capacity(), 8);
        let reserved = work(&c, HtmlParseWorkCategory::ReserveByte);
        let copied = work(&c, HtmlParseWorkCategory::CopyByte);
        let bytes = current(&c, HtmlParseResource::ManagedCapacityBytes);
        let value = c
            .extend_managed_document_string(&mut allocation, value, &[6, 7, 8])
            .ok()
            .unwrap();
        assert_eq!(value.code_units(), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(value.shared_code_units().capacity(), 8);
        assert_eq!(work(&c, HtmlParseWorkCategory::ReserveByte), reserved);
        // The three units, six bytes, and the ticket's fixed fields.
        assert_eq!(
            work(&c, HtmlParseWorkCategory::CopyByte) - copied,
            6 + (size_of::<u64>() + size_of::<Option<Weak<Vec<u16>>>>()) as u64
        );
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 8);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), bytes);
        c.require_document_string(&allocation, &value).unwrap();
        drop(value);
        c.release_document_string(allocation).unwrap();
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
    }

    #[test]
    fn a_full_backing_grows_geometrically_and_keeps_its_ticket_exact() {
        let mut c = control();
        let (mut value, mut allocation) = registered(&mut c, &[]);
        let reserved = work(&c, HtmlParseWorkCategory::ReserveByte);
        for unit in 0..40_u16 {
            value = c
                .extend_managed_document_string(&mut allocation, value, &[unit])
                .ok()
                .unwrap();
            c.require_document_string(&allocation, &value).unwrap();
        }
        let expected: Vec<u16> = (0..40).collect();
        assert_eq!(value.code_units(), expected);
        // One unit at a time from nothing: capacities 1, 2, 4, 8, 16, 32 and 64.
        assert_eq!(value.shared_code_units().capacity(), 64);
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 40);
        assert_eq!(
            current(&c, HtmlParseResource::ManagedCapacityBytes),
            64 * 2 + size_of::<Vec<u16>>() as u64
        );
        // Each growth reserves the bytes it replaces and the bytes it makes:
        // 2 * (0 + 1 + 2 + 4 + 8 + 16 + 32) and 2 * (1 + 2 + 4 + 8 + 16 + 32 + 64).
        assert_eq!(
            work(&c, HtmlParseWorkCategory::ReserveByte) - reserved,
            2 * 63 + 2 * 127
        );
        drop(value);
        c.release_document_string(allocation).unwrap();
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
    }

    #[test]
    fn a_refused_extension_hands_back_the_string_and_a_ticket_that_still_releases() {
        // Room for the five units and two more, one short of the three appended.
        let limits = crate::parser::tree_builder::tests::limits(100)
            .into_builder()
            .managed_utf16_units(7)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let (value, mut allocation) = registered(&mut c, &[1, 2, 3, 4, 5]);
        let refused = c
            .extend_managed_document_string(&mut allocation, value, &[6, 7, 8])
            .err()
            .unwrap();
        assert!(matches!(
            refused.stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedUtf16Units,
                limit: 7,
                ..
            }
        ));
        assert_eq!(c.stop(), Some(refused.stop));
        assert_eq!(refused.value.code_units(), [1, 2, 3, 4, 5]);
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 5);
        drop(refused.value);
        c.release_document_string(allocation).unwrap();
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
    }

    #[test]
    fn a_backing_another_holder_shares_is_refused_unchanged() {
        let mut c = control();
        let (value, mut allocation) = registered(&mut c, &[1, 2]);
        let other = value.clone();
        let refused = c
            .extend_managed_document_string(&mut allocation, value, &[3])
            .err()
            .unwrap();
        let aliased = HtmlParseStop::InternalInvariant(HtmlParseInvariant::SharedAllocationAliased);
        assert_eq!(refused.stop, aliased);
        assert_eq!(c.stop(), Some(aliased));
        assert_eq!(refused.value.code_units(), [1, 2]);
        assert_eq!(other.code_units(), [1, 2]);
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 2);
        drop(other);
        drop(refused.value);
        c.release_document_string(allocation).unwrap();
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
    }

    fn limited(
        configure: impl FnOnce(HtmlParseLimitsBuilder) -> HtmlParseLimitsBuilder,
    ) -> ParseControl {
        let limits = configure(crate::parser::tree_builder::tests::limits(100).into_builder())
            .finish()
            .unwrap();
        ParseControl::new(limits, crate::HtmlParseCancellation::new())
    }

    #[test]
    fn a_capacity_limit_met_while_the_backing_grows_hands_back_the_string_and_a_ticket_that_releases()
     {
        // Five units in a backing of eight hold sixteen bytes and the fixed wrapper. Nine more
        // units need sixteen slots, thirty-two bytes, which the limit cannot hold beside the
        // wrapper.
        let wrapper = size_of::<Vec<u16>>() as u64;
        let mut c = limited(|builder| builder.managed_capacity_bytes(16 + wrapper + 15));
        let (value, mut allocation) = registered(&mut c, &[1, 2, 3, 4, 5]);
        let before = current(&c, HtmlParseResource::ManagedCapacityBytes);
        let refused = c
            .extend_managed_document_string(&mut allocation, value, &[6; 9])
            .err()
            .unwrap();
        assert!(matches!(
            refused.stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }
        ));
        assert_eq!(c.stop(), Some(refused.stop));
        // Nothing was reserved or appended, so the ticket still names the value as it was.
        assert_eq!(refused.value.code_units(), [1, 2, 3, 4, 5]);
        assert_eq!(refused.value.shared_code_units().capacity(), 8);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), before);
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 5);
        drop(refused.value);
        c.release_document_string(allocation).unwrap();
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
    }

    #[test]
    fn a_cancelled_control_refuses_an_extension_before_it_changes_anything() {
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(
            crate::parser::tree_builder::tests::limits(100),
            cancellation.clone(),
        );
        let (value, mut allocation) = registered(&mut c, &[1, 2, 3]);
        cancellation.cancel();
        let refused = c
            .extend_managed_document_string(&mut allocation, value, &[4])
            .err()
            .unwrap();
        assert_eq!(refused.stop, HtmlParseStop::Cancelled);
        assert_eq!(refused.value.code_units(), [1, 2, 3]);
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 3);
        drop(refused.value);
        c.release_document_string(allocation).unwrap();
        assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
        assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
    }

    /// Cuts the work of one extension at every credit. The extension that completes is the one
    /// whose cut holds all its work, and every shorter cut hands the string back with a ticket
    /// that releases to zero.
    fn assert_every_work_cut_is_exact(initial: &[u16], appended: &[u16]) {
        let registered_work = {
            let mut c = control();
            let (value, allocation) = registered(&mut c, initial);
            let used = current(&c, HtmlParseResource::WorkCredits);
            drop(value);
            c.release_document_string(allocation).unwrap();
            used
        };
        let total = {
            let mut c = control();
            let (value, mut allocation) = registered(&mut c, initial);
            let extended = c
                .extend_managed_document_string(&mut allocation, value, appended)
                .ok()
                .unwrap();
            let total = current(&c, HtmlParseResource::WorkCredits) - registered_work;
            drop(extended);
            c.release_document_string(allocation).unwrap();
            total
        };
        assert!(total > 0);
        let mut expected: Vec<u16> = initial.to_vec();
        expected.extend_from_slice(appended);
        for room in 0..=total {
            let mut c = limited(|builder| builder.work_credits(registered_work + room));
            let (value, mut allocation) = registered(&mut c, initial);
            let result = c.extend_managed_document_string(&mut allocation, value, appended);
            let value = match result {
                Ok(value) => {
                    assert_eq!(room, total, "a cut of {room} credits completes");
                    assert_eq!(value.code_units(), expected);
                    value
                }
                Err(refused) => {
                    assert!(
                        room < total,
                        "the whole work of {total} credits is admitted"
                    );
                    assert!(matches!(
                        refused.stop,
                        HtmlParseStop::LimitExceeded {
                            resource: HtmlParseResource::WorkCredits,
                            ..
                        }
                    ));
                    assert_eq!(refused.value.code_units(), initial, "cut of {room}");
                    refused.value
                }
            };
            drop(value);
            c.release_document_string(allocation).unwrap();
            assert_eq!(current(&c, HtmlParseResource::ManagedUtf16Units), 0);
            assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
        }
    }

    #[test]
    fn every_work_cut_of_an_extension_within_the_capacity_is_exact() {
        assert_every_work_cut_is_exact(&[1, 2, 3, 4, 5], &[6, 7]);
    }

    #[test]
    fn every_work_cut_of_an_extension_that_grows_the_backing_is_exact() {
        assert_every_work_cut_is_exact(&[1, 2, 3, 4, 5, 6, 7, 8], &[9; 300]);
    }

    #[test]
    fn a_refused_record_after_an_extension_reserve_leaves_the_ticket_releasable() {
        let mut c = control();
        let (value, mut allocation) = registered(&mut c, &[1, 2, 3, 4, 5]);
        let held = current(&c, HtmlParseResource::ManagedCapacityBytes);
        // Room for the planned growth, not for the capacity the provider reports.
        c.test_limit(HtmlParseResource::ManagedCapacityBytes, held + 16_000);
        c.overshoot_next_capacity_record(1_000_000);
        let refused = c
            .extend_managed_document_string(&mut allocation, value, &[0; 20])
            .err()
            .unwrap();
        assert!(matches!(
            refused.stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }
        ));
        drop(refused.value);
        c.release_document_string(allocation).unwrap();
        crate::parser::tree_builder::tests::assert_released(&c);
    }
}
