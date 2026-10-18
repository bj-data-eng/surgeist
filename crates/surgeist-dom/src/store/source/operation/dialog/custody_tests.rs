//! Real Source event issuance plus owner-only fixed-atom dependency aliases.
use super::tests::{fixture, limits, request};
use super::*;

#[test]
fn owner_dependency_keeps_original_event_atoms_once_after_source_finishes() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _executor, _, subject, _) = fixture();
    let event = request(&mut dom, subject);
    let dependency = dom
        .source_operation
        .as_ref()
        .unwrap()
        .event_custody
        .clone()
        .unwrap();
    assert!(Arc::ptr_eq(dependency.work(), &event.work));
    let source = dependency.source();
    assert!(Arc::ptr_eq(source.work(), &event.work.work));
    let ledger = event.work.storage.ledger().clone();
    let before_alias = ledger.total();
    let alias = dependency.clone();
    assert_eq!(ledger.total(), before_alias);
    drop(alias);
    assert_eq!(ledger.total(), before_alias);

    // The real prestart cancellation completes this original Source without
    // inventing a dispatch receipt or publishing the requested Dialog change.
    {
        let mut lease = dom
            .resume_source_edit(event.operation(), limits(), &cancel)
            .unwrap();
        lease.request_cancel();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert!(dom.pending_source_edit().is_none());
    let after_source = ledger.total();
    drop(event); // An external request owns no record/page/fixed-atom debit.
    assert_eq!(ledger.total(), after_source);

    let fixed_event = size_of::<EventWork>()
        + 2 * size_of::<AtomicUsize>()
        + size_of::<formation::EventWorkCustody>()
        + custody::SharedDebit::administration_usage().bytes();
    drop(dependency);
    assert_eq!(after_source.bytes() - ledger.total().bytes(), fixed_event);
    assert_eq!(after_source.units(), ledger.total().units());
    // The separately held original counter remains controlled by the exact
    // identity alias, with no replacement counter or loaded-value copy.
    assert!(source.work().load(Ordering::Relaxed) > 0);
}
