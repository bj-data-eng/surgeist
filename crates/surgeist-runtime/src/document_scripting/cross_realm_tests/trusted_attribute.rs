use super::*;
use crate::trusted_types_csp::{AttributePolicies, AttributePolicyError, PolicyFactory};
use std::convert::Infallible;
use surgeist_bindings_js::conversion::{ConversionAccount, ResourceTable, WorkTable};
use surgeist_bindings_js::{TrustedPolicyCallback, TrustedTypeKind};
use surgeist_dom::{DocumentString, DomSourceAttributeVerificationOutcome, DomSourceProgress};
use surgeist_html::security::csp::{
    HtmlCspError, HtmlCspLayout, HtmlCspProcessorProfile, HtmlCspStorage, HtmlMetaCspOutcome,
    HtmlMetaCspProcessor, HtmlMetaCspRequest,
};

struct StopAfterCspPreparation<'a> {
    processor: &'a mut crate::trusted_types_csp::PolicyProcessor,
    cancel: &'a AtomicBool,
    calls: usize,
}

impl HtmlMetaCspProcessor for StopAfterCspPreparation<'_> {
    type Error = crate::trusted_types_csp::PolicyListCapacity;

    fn profile(&self) -> &HtmlCspProcessorProfile {
        self.processor.profile()
    }

    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>> {
        self.calls += 1;
        let outcome = self.processor.prepare(request, admit, release)?;
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(outcome)
    }
}

pub(super) fn hosted_meta_source_publishes_ordered_csp_for_the_registered_attribute_verifier() {
    use crate::trusted_types_csp::{PolicyDelivery, PolicyProcessor};
    use crate::{DomSourceDriveProgress, DomSourceServices, DomSourceUnavailable};
    use surgeist_dom::{DomDocumentCspState, DomSourceMetaCspError};

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let element = insert_hosted_element(&mut session, target, "div", &cancel);
    let snapshot = session.snapshot();
    let document = target.document();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let mut metas = Vec::new();
    for policy in [
        "trusted-types default; report-uri /ignored",
        "require-trusted-types-for 'script'",
    ] {
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", policy);
        metas.push(meta);
    }
    let formed = session.dom().apply(batch, &cancel).unwrap();
    let mut allow = |_| Ok::<(), Infallible>(());
    session.enable_trusted_types_csp(&cancel).unwrap();
    let selected = session.trusted_types_csp.as_ref().unwrap().clone();
    let mut other = PolicyProcessor::new_with(&mut allow, &mut |_| unreachable!()).unwrap();
    let mut source = session.source_execution();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let mut policy_units = 0;
    for (index, ticket) in metas.iter().enumerate() {
        let meta = formed.created(ticket).unwrap().unwrap();
        let before = session.snapshot().stamp();
        let before_units = session.dom().usage().string_code_units();
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::MetaCsp(request),
            reason: DomSourceUnavailable::MetaCsp,
        } = source
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(meta, head, None),
                SOURCE,
                &mut services,
                &cancel,
            )
            .unwrap()
        else {
            panic!("expected original Meta policy request");
        };
        assert_eq!(session.snapshot().stamp(), before);
        if index == 0 {
            let mut processor = selected.processor.borrow_mut();
            let mut stopped = StopAfterCspPreparation {
                processor: &mut processor,
                cancel: &cancel,
                calls: 0,
            };
            let mut view = session.dom();
            {
                let mut lease = view
                    .resume_source_edit(request.operation(), SOURCE, &cancel)
                    .unwrap();
                assert!(matches!(
                    lease.process_meta_csp(request, &mut stopped, &cancel),
                    Err(DomSourceMetaCspError::Dom(DomError::Cancelled))
                ));
                assert_eq!(lease.snapshot().stamp(), before);
            }
            cancel.store(false, std::sync::atomic::Ordering::Relaxed);
            let mut lease = view
                .resume_source_edit(request.operation(), SOURCE, &cancel)
                .unwrap();
            lease
                .process_meta_csp(request, &mut stopped, &cancel)
                .unwrap();
            assert_eq!(stopped.calls, 1);
        } else {
            assert!(matches!(
                source.service_meta_csp(&mut session.dom(), request, &mut other, SOURCE, &cancel),
                Err(crate::DomSourceRuntimeError::MetaCsp(
                    DomSourceMetaCspError::Protocol(_)
                ))
            ));
            assert_eq!(session.snapshot().stamp(), before);
            session
                .service_trusted_types_meta_csp(request, SOURCE, &cancel)
                .unwrap();
        }
        assert_eq!(session.snapshot().stamp(), before);
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    request.operation(),
                    SOURCE,
                    &mut services,
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let snapshot = session.snapshot();
        assert_eq!(snapshot.node(meta).unwrap().parent(), Some(head));
        let document = snapshot.document(document).unwrap();
        let state = document.metadata().policy_container().csp();
        let DomDocumentCspState::Associated(association) = state else {
            panic!("the real published document must retain the policy");
        };
        assert!(association.latest().is_some());
        let next_units = association
            .retained_layout()
            .unwrap()
            .storage()
            .utf16_units();
        assert_eq!(
            session.dom().usage().string_code_units(),
            before_units - policy_units + next_units
        );
        policy_units = next_units;
        let list = selected
            .processor
            .borrow()
            .document_policies_with(state, &mut allow)
            .unwrap()
            .unwrap();
        assert_eq!(list.len(), index + 1);
        assert!(
            list.iter()
                .all(|policy| policy.delivery() == PolicyDelivery::Meta)
        );
    }

    let host = enable(session, target, cancel.clone());
    let registration = &host.engine.as_ref().unwrap().registration;
    let mut session = host.native.session.borrow_mut();
    let mut view = session.dom();
    let mut lease = view
        .begin_source_edit(
            DomSourceEdit::set_attribute(element, "onclick", "input"),
            SOURCE,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::AttributeVerification(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("expected actual registered attribute verifier");
    };
    let processor = selected.processor.borrow();
    let policies =
        AttributePolicies::read(&lease, request, registration, Some(&processor), &cancel).unwrap();
    assert!(policies.requires_default_policy(&cancel).unwrap());
    let mut mismatch = policies.mismatch();
    let violation = mismatch.next_violation(&cancel).unwrap().unwrap();
    assert_eq!(violation.policy_index(), 1);
    assert_eq!(mismatch.blocked(), Some(true));
    drop(processor);
    lease.request_cancel();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    drop(session);

    let session = host.stop().into_session().unwrap();
    let mut host =
        super::super::guest::tests::attach_public_nodes(session, target, cancel, 5_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let expression: Vec<_> = r#"
            (() => {
                const policy = trustedTypes.createPolicy('default');
                if (policy.name !== 'default' || trustedTypes.defaultPolicy !== policy) return false;
                try { trustedTypes.createPolicy('denied'); }
                catch (error) { return error instanceof TypeError; }
                return false;
            })()
        "#.encode_utf16().collect();
        let result = scope.evaluate(&expression)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).expect("the actual guest factory must use the hosted Document's CSP");
    super::super::guest::tests::assert_queued_policy_name(&host, "denied");
}

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
