use super::*;
use crate::trusted_types_csp::{AttributePolicies, AttributePolicyError, PolicyFactory};
use std::convert::Infallible;
use surgeist_bindings_js::conversion::{ConversionAccount, ResourceTable, WorkTable};
use surgeist_bindings_js::{TrustedPolicyCallback, TrustedTypeKind};
use surgeist_dom::{DocumentString, DomSourceAttributeVerificationOutcome, DomSourceProgress};

pub(super) fn default_factory_retains_the_real_guest_callback() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = enable(session, target, cancel);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let mut factory = PolicyFactory::<TrustedPolicyCallback, ()>::default();
        let name = DocumentString::from("default");
        let attempt = factory
            .begin(&name, &[], &mut || Ok::<_, Infallible>(()))
            .unwrap();
        let admitted = attempt.admit(|_| Ok::<_, Infallible>(())).unwrap();
        let guest = scope.evaluate(
            &"(value, type, sink) => value + (type === 'TrustedScript' && sink === 'Element onclick' ? '!' : '?')"
                .encode_utf16()
                .collect::<Vec<_>>(),
        )?;
        let policy = Rc::new(
            scope
                .retain_trusted_policy_callback(&guest, |_| Ok::<_, Error>(()))?
                .unwrap(),
        );
        admitted.publish(&policy);
        let retained = factory.default_policy().unwrap();
        assert!(Rc::ptr_eq(retained, &policy));
        drop(policy);
        let account = ConversionAccount::standalone(
            scope,
            WorkTable::splat(1_000),
            ResourceTable::splat(1_000),
        )?;
        let result = account.run(scope, |conversion| {
            conversion.invoke_default_policy(
                retained,
                TrustedTypeKind::Script,
                &[0xd800, 0],
                &"Element onclick".encode_utf16().collect::<Vec<_>>(),
            )
        })?;
        assert_eq!(
            result.get().as_ref().unwrap().as_code_units(),
            &[0xd800, 0, b'!' as u16]
        );
        Ok(())
    })
    .unwrap();
}

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
