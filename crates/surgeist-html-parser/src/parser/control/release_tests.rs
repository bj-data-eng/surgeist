//! Release helpers refused for a state that cannot arise from a valid parse.
//! Each forces the state through crate-private fields and asserts the exact
//! latched stop, that nothing was credited, and that the owner's own release
//! path still drains the controller afterwards.
use super::*;
use crate::{Attribute, Namespace, OrderedAttributes, QualifiedName};

const FOREIGN: ParseStop = ParseStop::InternalInvariant(Invariant::ForeignControl);
const MISMATCH: ParseStop = ParseStop::InternalInvariant(Invariant::CapacityMismatch);
const UNDERFLOW: ParseStop = ParseStop::InternalInvariant(Invariant::ReleaseUnderflow);

fn control() -> ParseControl {
    ParseControl::new(
        crate::parser::tree_builder::tests::limits(100),
        crate::HtmlParseCancellation::new(),
    )
}
fn current(c: &ParseControl, resource: HtmlParseResource) -> u64 {
    c.usage().resource(resource).current()
}
fn bank() -> OrderedAttributes {
    OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "class"),
        "k",
    )])
    .unwrap()
}

#[test]
fn provider_release_for_another_controller_latches_foreign_control_and_credits_nothing() {
    let mut c = control();
    let other = control();
    c.acquire_retained_provider(8, 2, 1).unwrap();
    let before = c.usage().clone();
    let _ = c.release_retained_provider(&other.identity(), 8, 2, 1);
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), &before);
    let owner = c.identity();
    let _ = c.release_retained_provider(&owner, 8, 2, 1);
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
    let _ = c.commit_prepared_release(&[
        (HtmlParseResource::AuxiliaryEntries, 1),
        (HtmlParseResource::ManagedUtf16Units, 3),
    ]);
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), &before, "no partial credit");
}

#[test]
fn final_vector_release_of_another_controllers_ticket_is_refused_and_credits_nothing() {
    let mut c = control();
    let mut other = control();
    let (_vector, ticket) = other.new_vector::<Attribute>().unwrap();
    let before = c.usage().clone();
    let _ = c.release_final_vector(ticket, 0);
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), &before);
}

#[test]
fn final_shared_release_of_an_unfinalized_ticket_is_refused_and_credits_nothing() {
    let mut c = control();
    let (shared, ticket) = c.share_owned(7_u32).unwrap();
    let before = c.usage().clone();
    let _ = c.release_final_shared(ticket);
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), &before);
    drop(shared);
}

#[test]
fn final_shared_release_of_another_controllers_ticket_is_refused_and_credits_nothing() {
    let mut c = control();
    let mut other = control();
    let values = bank();
    let (_vector, ticket) = other.final_attribute_tickets(&values).unwrap();
    let before = c.usage().clone();
    let _ = c.release_final_shared(ticket);
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), &before);
}

#[test]
fn final_attribute_release_of_another_controllers_tickets_is_refused() {
    let mut c = control();
    let mut other = control();
    let values = bank();
    let (vector, shared) = other.final_attribute_tickets(&values).unwrap();
    let before = c.usage().clone();
    let _ = c.release_final_attributes(values, vector, shared);
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), &before);
}

#[test]
fn final_attribute_release_of_a_different_bank_is_refused() {
    let mut c = control();
    let values = bank();
    let (vector, shared) = c.final_attribute_tickets(&values).unwrap();
    let before = c.usage().clone();
    let _ = c.release_final_attributes(bank(), vector, shared);
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), &before);
    drop(values);
}

#[test]
fn final_attribute_release_with_a_forged_capacity_is_refused() {
    let mut c = control();
    let values = bank();
    let (mut vector, shared) = c.final_attribute_tickets(&values).unwrap();
    vector.capacity += 1;
    let before = c.usage().clone();
    let _ = c.release_final_attributes(values, vector, shared);
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), &before);
}

fn text(units: &[u16]) -> crate::DocumentString {
    crate::DocumentString::from_utf16(units.to_vec())
}

/// A pending ticket describing `value`, as a valid append leaves it.
fn ticket_for(c: &mut ParseControl, value: &crate::DocumentString) -> DocumentStringAllocation {
    let mut ticket = c.prepare_text_append_allocation().unwrap();
    ticket.vector_capacity = value.shared_code_units().capacity();
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
    let _ = c.restore_text_append_ticket(&mut ticket, &value);
    assert_eq!(c.stop(), Some(FOREIGN));
    assert_eq!(c.usage(), &before);
    assert!(ticket.backing.is_none(), "the ticket is left as it was");
}

#[test]
fn restoring_a_text_ticket_that_already_has_backing_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.backing = Some(Arc::downgrade(value.shared_code_units()));
    let before = c.usage().clone();
    let _ = c.restore_text_append_ticket(&mut ticket, &value);
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), &before);
}

#[test]
fn restoring_a_text_ticket_with_fewer_units_than_the_value_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.units = 0;
    let before = c.usage().clone();
    let _ = c.restore_text_append_ticket(&mut ticket, &value);
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), &before);
    assert!(ticket.backing.is_none());
}

#[test]
fn restoring_more_discarded_units_than_are_admitted_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.units = 5;
    let before = c.usage().clone();
    let _ = c.restore_text_append_ticket(&mut ticket, &value);
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), &before);
    assert!(ticket.backing.is_none());
}

#[test]
fn restoring_a_text_ticket_whose_capacity_differs_from_the_value_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.vector_capacity += 1;
    let before = c.usage().clone();
    let _ = c.restore_text_append_ticket(&mut ticket, &value);
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), &before);
    assert!(ticket.backing.is_none());
}

#[test]
fn retaining_a_failed_text_ticket_bound_to_other_backing_is_refused() {
    let mut c = control();
    let value = text(&[65]);
    let other = text(&[66]);
    let mut ticket = ticket_for(&mut c, &value);
    ticket.backing = Some(Arc::downgrade(other.shared_code_units()));
    let _ = c.retain_failed_text_append_ticket(&mut ticket, &value);
    assert_eq!(c.stop(), Some(MISMATCH));
}

#[test]
fn text_domain_commit_beyond_the_buffered_amount_latches_underflow_and_changes_nothing() {
    let mut c = control();
    let mut ticket = c.prepare_text_append_allocation().unwrap();
    ticket.units = 3;
    let before = c.usage().clone();
    let _ = c.commit_text_append_domain(&mut ticket);
    assert_eq!(c.stop(), Some(UNDERFLOW));
    assert_eq!(c.usage(), &before);
    assert_eq!(ticket.resource, HtmlParseResource::BufferedUtf16Units);
}

#[test]
fn text_unit_admission_that_overflows_the_ticket_count_is_a_representation_stop() {
    let mut c = control();
    let mut ticket = c.prepare_text_append_allocation().unwrap();
    ticket.units = u64::MAX;
    let _ = c.prepare_text_append_units(&mut ticket, 1);
    assert_eq!(
        c.stop(),
        Some(ParseStop::RepresentationExhausted {
            resource: HtmlParseResource::BufferedUtf16Units
        })
    );
}

#[test]
fn reserving_a_plan_that_shrinks_the_allocation_is_refused_before_any_change() {
    let mut c = control();
    c.set(HtmlParseResource::ManagedCapacityBytes, 100);
    let before = c.usage().clone();
    let mut vector = Vec::<u32>::new();
    let plan = ReservationPlan {
        additional: 1,
        old_bytes: 10,
        new_bytes: 5,
    };
    let _ = c.reserve(&mut vector, plan);
    assert_eq!(c.stop(), Some(MISMATCH));
    assert_eq!(c.usage(), &before);
    assert_eq!(vector.capacity(), 0);
}
