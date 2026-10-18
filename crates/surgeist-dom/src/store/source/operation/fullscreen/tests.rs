//! DOM issuer/constructor boundary only. This fixture models trusted native
//! publication inputs; it does not certify Runtime Global construction or any
//! browser orientation/viewport service. Runtime separately tests composition.
use super::*;
use crate::{
    DomHostedCreationStep, DomHostedEntryInitialization, DomHostedLimits, DomHostedRecordLimits,
    DomInitialPermissionsPolicy, DomPermissionsPolicyValue, DomSourceEdit,
};

fn policy(outputs: usize) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 100_000, 64, outputs, 64, 4)
}
fn fixture() -> (Dom, crate::DomHostedSource, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(64, 100_000, 256, 128, 128, 16, 4_000_000)
            .unwrap()
            .with_fullscreen_exits(4)
            .unwrap()
            .with_fullscreen_events(4)
            .unwrap(),
    )
    .unwrap();
    let mut source = dom
        .register_hosted_source(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                10_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let host = source.issue_host(&cancel).unwrap();
    let mut preparation = Some(source.prepare_top_level(host, &cancel).unwrap());
    let mut prepared = Some(source.seal_top_level(&mut preparation, &cancel).unwrap());
    let mut lease = dom
        .begin_hosted_top_level(&mut source, &mut prepared, &cancel)
        .unwrap();
    loop {
        let publication = match lease.step(&dom).unwrap() {
            DomHostedCreationStep::PermissionsPolicy(request) => {
                let mut publication = Some(
                    source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel,
                        )
                        .unwrap(),
                );
                lease
                    .accept_permissions_policy(&mut dom, &mut source, &mut publication, &cancel)
                    .unwrap();
                continue;
            }
            DomHostedCreationStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel).unwrap();
                source
                    .seal_native_execution(request, global, 0, &cancel)
                    .unwrap()
            }
            DomHostedCreationStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel).unwrap()
            }
            DomHostedCreationStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 100, &cancel).unwrap()
            }
            DomHostedCreationStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel).unwrap();
                let navigable = source.issue_navigable(request, &cancel).unwrap();
                let uuid = |last| {
                    let mut out = [0u16; 36];
                    for (target, unit) in out
                        .iter_mut()
                        .zip("00000000-0000-4000-8000-000000000000".encode_utf16())
                    {
                        *target = unit;
                    }
                    out[35] = last;
                    out
                };
                let mut initialization = Some(DomHostedEntryInitialization::new(
                    uuid(u16::from(b'1')),
                    uuid(u16::from(b'2')),
                ));
                source
                    .seal_initial_entry(request, entry, navigable, &mut initialization, &cancel)
                    .unwrap()
            }
            DomHostedCreationStep::Complete(_) => break,
        };
        lease
            .accept_publication(&mut dom, &mut source, &mut Some(publication), &cancel)
            .unwrap();
    }
    let document = lease
        .finish(&mut dom, &mut source, &cancel)
        .unwrap()
        .document();
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(document, crate::Namespace::none(), "fullscreen-subject");
    batch.insert_before(&subject, parent, None);
    batch.fullscreen_element_model(&subject);
    let subject = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    (dom, source, document, subject)
}

#[test]
fn actual_handoff_and_outer_tail_rollback_share_the_original_cumulative_work_cap() {
    let cancel = AtomicBool::new(false);
    let (mut dom, source, document, subject) = fixture();
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::detach(subject), policy(1024), &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("real reached exit handoff");
    };
    let operation = request.operation();
    let invocation = request.invocation();
    let _job = lease
        .prepare_fullscreen_exit_handoff(&request, &source, &cancel)
        .unwrap()
        .publish();
    let prefix = lease.snapshot();
    assert_eq!(prefix.node(subject).unwrap().parent(), None);
    assert!(
        !prefix
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(lease.dom.fullscreen_exits.len(), 1);
    let outputs = lease.dom.source_operation.as_ref().unwrap().outputs;
    let mut retry = lease
        .dom
        .resume_source_edit(operation, policy(outputs), &cancel)
        .unwrap();
    let before = retry.status().spent_work();
    assert_eq!(
        retry.advance(&cancel).unwrap_err(),
        DomError::SourceOutputLimit
    );
    let after = retry.status().spent_work();
    assert!(after > before);
    assert_eq!(
        retry.dom.fullscreen_exits[invocation.slot]
            .work
            .load(Ordering::Relaxed),
        after
    );
    assert_eq!(retry.snapshot().stamp(), prefix.stamp());
    assert_eq!(
        retry
            .dom
            .state
            .node(retry.dom.owner, document.node())
            .unwrap()
            .fullscreen_events
            .len(),
        1
    );
    // Repeated genuine final-output refusal spends this one original budget.
    // It never queues another event or invocation and cannot renew either meter.
    let mut exhausted = false;
    for _ in 0..policy(outputs).work() {
        match retry.advance(&cancel) {
            Err(DomError::SourceOutputLimit) => {}
            Err(DomError::SourceWorkLimit { limit }) => {
                assert_eq!(limit, policy(outputs).work());
                exhausted = true;
                break;
            }
            other => panic!("original unfinished tail: {other:?}"),
        }
    }
    assert!(exhausted);
    assert_eq!(retry.status().spent_work(), policy(outputs).work());
    let retained = &retry.dom.fullscreen_exits[invocation.slot];
    assert_eq!(
        retained.step(&cancel),
        Err(DomError::SourceWorkLimit {
            limit: policy(outputs).work()
        })
    );
    assert_eq!(
        retained.work.load(Ordering::Relaxed),
        retry.status().spent_work()
    );
    assert_eq!(retry.dom.fullscreen_exits.len(), 1);
    assert_eq!(retry.snapshot().stamp(), prefix.stamp());
}

#[test]
fn dropped_prepared_handoff_restores_its_prefix_and_reuses_only_the_same_exposed_invocation() {
    let cancel = AtomicBool::new(false);
    let (mut dom, source, document, subject) = fixture();
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::exit_fullscreen(document),
            policy(1024),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual request");
    };
    let operation = request.operation();
    let generation = lease.dom.next_fullscreen_exit;
    drop(
        lease
            .prepare_fullscreen_exit_handoff(&request, &source, &cancel)
            .unwrap(),
    );
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(lease.dom.fullscreen_exits.is_empty());
    assert!(
        lease
            .dom
            .state
            .node(lease.dom.owner, document.node())
            .unwrap()
            .fullscreen_events
            .is_empty()
    );
    drop(lease);
    let mut retry = dom
        .resume_source_edit(operation, policy(1024), &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(again)) =
        retry.advance(&cancel).unwrap()
    else {
        panic!("same retained request");
    };
    assert_eq!(again, request);
    assert_eq!(retry.dom.next_fullscreen_exit, generation);
    let mut job = retry
        .prepare_fullscreen_exit_handoff(&again, &source, &cancel)
        .unwrap()
        .publish();
    let DomSourceProgress::Complete(done) = retry.advance(&cancel).unwrap() else {
        panic!("synchronous return");
    };
    assert_eq!(
        done.fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::Pending(request.invocation()))
    );
    drop(retry);
    assert_eq!(
        dom.resume_fullscreen_exit(&source, &mut job, &cancel)
            .unwrap()
            .id(),
        request.invocation()
    );
    assert!(
        dom.snapshot()
            .node(subject)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag(),
        "connected subject has no step7 unfullscreen"
    );
    let mut stale = request.invocation();
    stale.generation -= 1;
    assert_eq!(
        dom.fullscreen_exit_status(stale),
        Err(DomError::StaleFullscreenExit)
    );
    assert_eq!(dom.next_fullscreen_exit, generation);
}

#[test]
fn identity_admission_and_generation_exhaustion_refuse_before_handoff_or_prefix() {
    let cancel = AtomicBool::new(false);
    for exhausted in [false, true] {
        let (mut dom, _source, document, _) = fixture();
        let before = dom.snapshot();
        let mut limits = policy(1024);
        if exhausted {
            dom.next_fullscreen_exit = u64::MAX;
        } else {
            limits.identities = 0;
        }
        let next = dom.next_fullscreen_exit;
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::exit_fullscreen(document), limits, &cancel)
            .unwrap();
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            if exhausted {
                DomError::RepresentationExhausted
            } else {
                DomError::SourceIdentityLimit
            }
        );
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        assert!(lease.dom.fullscreen_exits.is_empty());
        assert_eq!(lease.dom.next_fullscreen_exit, next);
        assert!(!lease.status().has_published());
    }
}
