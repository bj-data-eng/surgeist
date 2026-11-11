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
