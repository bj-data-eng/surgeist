use super::*;
use crate::trusted_types_csp::{AttributePolicies, AttributePolicyError};
use surgeist_dom::{DocumentString, DomSourceAttributeVerificationOutcome, DomSourceProgress};

pub(super) fn hosted_attribute_policy_reads_charge_the_parked_source_and_preserve_exact_input() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target, element) = hosted_element(&cancel, "div");
    let host = enable(session, target, cancel.clone());
    let registration = &host.engine.as_ref().unwrap().registration;
    let mut session = host.native.session.borrow_mut();
    let mut view = session.dom();
    let dom = view.raw_mut();
    let units = DocumentString::from_utf16([0xd800, 0, 0x61]);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(element, "onclick", units.clone()),
            SOURCE,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("expected the actual registered Source sink");
    };
    let stopped = AtomicBool::new(true);
    let spent = lease.status().spent_work();
    assert!(matches!(
        AttributePolicies::read(&lease, request, registration, None, &stopped),
        Err(AttributePolicyError::Dom(DomError::Cancelled))
    ));
    assert_eq!(lease.status().spent_work(), spent);
    {
        let policies =
            AttributePolicies::read(&lease, request, registration, None, &cancel).unwrap();
        assert_eq!(policies.document(), target.document());
        assert!(!policies.requires_default_policy(&cancel).unwrap());
        let mut mismatch = policies.mismatch();
        assert!(matches!(
            mismatch.next_violation(&stopped),
            Err(DomError::Cancelled)
        ));
        assert!(mismatch.next_violation(&cancel).unwrap().is_none());
        assert_eq!(mismatch.blocked(), Some(false));
    }
    assert!(lease.status().spent_work() > spent);
    let original = lease
        .attribute_verification_input(request, registration, &cancel)
        .unwrap()
        .clone();
    assert_eq!(original, units);
    let mut reply = Some(DomSourceAttributeVerificationOutcome::Value(original));
    lease
        .accept_attribute_verification(request, registration, &mut reply, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(
        lease
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&units)
    );
}
