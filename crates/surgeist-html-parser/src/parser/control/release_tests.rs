//! Release helpers refused for a state that cannot arise from a valid parse.
//! Each forces the state through crate-private fields and asserts the exact
//! latched stop, that nothing was credited, and that the owner's own release
//! path still drains the controller afterwards.
use super::*;

const FOREIGN: HtmlParseStop = HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl);
const MISMATCH: HtmlParseStop =
    HtmlParseStop::InternalInvariant(HtmlParseInvariant::CapacityMismatch);
/// A release the ledger refuses.
const UNDERFLOW: HtmlParseStop =
    HtmlParseStop::Accounting(surgeist_html::accounting::HtmlAccountingFault::ReleaseUnderflow);

fn control() -> ParseControl {
    ParseControl::new(
        crate::parser::tree_builder::tests::limits(100),
        crate::HtmlParseCancellation::new(),
    )
}
fn current(c: &ParseControl, resource: HtmlParseResource) -> u64 {
    c.usage().resource(resource).current()
}
fn current_of(usage: &HtmlParseUsage, resource: HtmlParseResource) -> u64 {
    usage.resource(resource).current()
}
#[test]
fn provider_release_for_another_controller_latches_foreign_control_and_credits_nothing() {
    let mut c = control();
    let other = control();
    c.acquire_retained_provider(8, 2, 1).unwrap();
    let before = c.usage().clone();
    assert_eq!(
        c.release_retained_provider(&other.identity(), 8, 2, 1),
        Err(FOREIGN)
    );
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), before);
    let owner = c.identity();
    assert_eq!(c.release_retained_provider(&owner, 8, 2, 1), Ok(()));
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::PeakTransientBytes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::AuxiliaryEntries,
    ] {
        assert_eq!(current(&c, resource), 0, "{resource:?}");
    }
    assert_eq!(c.stop(), Some(FOREIGN));
}

#[test]
fn prepared_release_beyond_the_admitted_amount_latches_underflow_and_credits_nothing() {
    let mut c = control();
    c.acquire_retained_provider(8, 2, 1).unwrap();
    let before = c.usage().clone();
    assert_eq!(
        c.commit_prepared_release(&[
            (HtmlParseResource::AuxiliaryEntries, 1),
            (HtmlParseResource::ManagedUtf16Units, 3),
        ]),
        Err(UNDERFLOW)
    );
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), before, "no partial credit");
    assert_eq!(
        c.commit_prepared_release(&[
            (HtmlParseResource::AuxiliaryEntries, 1),
            (HtmlParseResource::ManagedUtf16Units, 2),
        ]),
        Ok(())
    );
    assert_eq!(current(&c, HtmlParseResource::AuxiliaryEntries), 0);
}

// Characterization of existing behaviour after the release paths became
// fallible; these were not RED against any earlier commit.
// Characterization, as above. The vector capacity must equal the bank's real
// one, so the overflow is reached through the shared ticket's byte count.
fn text(units: &[u16]) -> surgeist_html::values::DocumentString {
    surgeist_html::values::DocumentString::from_utf16(units.to_vec())
}

/// A pending ticket describing `value`, as a valid append leaves it.
fn ticket_for(
    c: &mut ParseControl,
    value: &surgeist_html::values::DocumentString,
) -> DocumentStringAllocation {
    let mut ticket = c.prepare_text_append_allocation().unwrap();
    ticket.vector_capacity = value.retained_units();
    ticket.units = value.len() as u64;
    ticket
}

#[test]
fn restoring_another_controllers_text_ticket_is_refused() {
    let mut c = control();
    let mut other = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut other, &value);
    let before = c.usage().clone();
    assert_eq!(
        c.restore_text_append_ticket(&mut ticket, &value),
        Err(FOREIGN)
    );
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), before);
    assert!(ticket.backing.is_none(), "the ticket is left as it was");
}

#[test]
fn restoring_a_text_ticket_that_already_has_backing_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.backing = Some(value.block().downgrade());
    let before = c.usage().clone();
    assert_eq!(
        c.restore_text_append_ticket(&mut ticket, &value),
        Err(MISMATCH)
    );
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), before);
}

#[test]
fn restoring_a_text_ticket_with_fewer_units_than_the_value_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.units = 0;
    let before = c.usage().clone();
    // The ticket's own check refuses before any release reaches the ledger.
    let underflow = HtmlParseStop::InternalInvariant(HtmlParseInvariant::ReleaseUnderflow);
    assert_eq!(
        c.restore_text_append_ticket(&mut ticket, &value),
        Err(underflow)
    );
    assert_eq!(c.stop(), Some(underflow));
    assert_eq!(c.usage(), before);
    assert!(ticket.backing.is_none());
}

#[test]
fn restoring_more_discarded_units_than_are_admitted_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.units = 5;
    let before = c.usage().clone();
    assert_eq!(
        c.restore_text_append_ticket(&mut ticket, &value),
        Err(UNDERFLOW)
    );
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), before);
    assert!(ticket.backing.is_none());
}

#[test]
fn restoring_a_text_ticket_whose_capacity_differs_from_the_value_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.vector_capacity += 1;
    let before = c.usage().clone();
    assert_eq!(
        c.restore_text_append_ticket(&mut ticket, &value),
        Err(MISMATCH)
    );
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), before);
    assert!(ticket.backing.is_none());
}

/// A growth recorded against another controller's ticket is a ticket under the
/// wrong controller, which is the one fault the foreign-control label names.
#[test]
fn recording_growth_against_another_controllers_ticket_reports_foreign_control() {
    let capacity = crate::construction::text_append_tests::observed_capacity();
    let mut c = control();
    let mut other = control();
    let mut ticket = other.prepare_text_append_allocation().unwrap();
    let before = c.usage().clone();
    assert_eq!(
        c.record_text_append_capacity(&mut ticket, capacity, capacity.capacity()),
        Err(FOREIGN)
    );
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), before, "nothing is recorded for the ticket");
}

/// A growth recorded against a ticket that is already bound to a backing is a
/// ticket whose allocation state differs from what a pending append holds. The
/// ticket belongs to this controller, so it is not a foreign-control fault.
#[test]
fn recording_growth_against_a_bound_ticket_reports_the_capacity_mismatch() {
    let capacity = crate::construction::text_append_tests::observed_capacity();
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.backing = Some(value.block().downgrade());
    let before = c.usage().clone();
    assert_eq!(
        c.record_text_append_capacity(&mut ticket, capacity, capacity.capacity()),
        Err(MISMATCH)
    );
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), before, "nothing is recorded for the ticket");
}

#[test]
fn retaining_a_failed_text_ticket_bound_to_other_backing_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let other = text(&[66]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.backing = Some(other.block().downgrade());
    assert_eq!(
        c.retain_failed_text_append_ticket(&mut ticket, &value),
        Err(MISMATCH)
    );
    assert_eq!(c.stop(), Some(MISMATCH));
}

#[test]
fn text_domain_commit_beyond_the_buffered_amount_latches_underflow_and_changes_nothing() {
    let mut c = control();
    let mut ticket = c.prepare_text_append_allocation().unwrap();
    ticket.units = 3;
    let before = c.usage().clone();
    assert_eq!(c.commit_text_append_domain(&mut ticket), Err(UNDERFLOW));
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), before);
    assert_eq!(ticket.resource, HtmlParseResource::BufferedUtf16Units);
}

#[test]
fn text_unit_admission_that_overflows_the_ticket_count_is_a_representation_stop() {
    let mut c = control();
    let mut ticket = c.prepare_text_append_allocation().unwrap();
    ticket.units = u64::MAX;
    let exhausted = HtmlParseStop::RepresentationExhausted {
        resource: HtmlParseResource::BufferedUtf16Units,
    };
    let before = c.usage().clone();
    assert_eq!(c.prepare_text_append_units(&mut ticket, 1), Err(exhausted));
    assert_eq!(c.stop(), Some(exhausted));
    assert_eq!(
        current(&c, HtmlParseResource::BufferedUtf16Units),
        current_of(&before, HtmlParseResource::BufferedUtf16Units),
        "nothing is admitted for the overflowing count"
    );
    assert_eq!(ticket.units, u64::MAX);
}

#[test]
fn reserving_a_plan_that_shrinks_the_allocation_is_refused_before_any_change() {
    let mut c = control();
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    let before = c.usage().clone();
    let mut vector = Vec::<u32>::new();
    let plan = ReservationPlan {
        additional: 1,
        needed: 1,
        old_bytes: 10,
        new_bytes: 5,
    };
    assert_eq!(
        c.reserve(&mut vector, plan, &mut plan.old_bytes.clone()),
        Err(MISMATCH)
    );
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), before);
    assert_eq!(vector.capacity(), 0);
}

#[test]
fn a_shared_release_beyond_the_admitted_capacity_hands_back_the_arc_and_ticket() {
    let mut c = control();
    let (shared, ticket) = c.share_owned(7_u32).unwrap();
    let bytes = ticket.payload_bytes();
    c.set(HtmlParseResource::ManagedCapacityBytes, bytes - 1);
    let (stop, shared, ticket) = c.release_shared(shared, ticket).unwrap_err().into_parts();
    assert_eq!(stop, UNDERFLOW);
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(
        current(&c, HtmlParseResource::ManagedCapacityBytes),
        bytes - 1
    );
    // With the ledger restored, the same Arc and ticket are accepted.
    c.set(HtmlParseResource::ManagedCapacityBytes, bytes);
    assert_eq!(c.release_shared(shared, ticket).unwrap(), 7);
    assert_eq!(current(&c, HtmlParseResource::ManagedCapacityBytes), 0);
}

#[test]
fn a_shared_admission_whose_charge_vanished_still_reports_the_post_poll_stop() {
    let mut c = control();
    let bytes = c.admit_shared_layout::<u32>().unwrap();
    c.set(HtmlParseResource::ManagedCapacityBytes, 0);
    c.cancellation.cancel();
    assert_eq!(
        c.confirm_shared_admission(bytes),
        Err(HtmlParseStop::Cancelled)
    );
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}
