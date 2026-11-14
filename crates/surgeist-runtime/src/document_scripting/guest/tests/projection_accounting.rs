use super::*;
use crate::document_scripting::cross_realm_tests::hosted_session;
use surgeist_ledger::{EventFilter, ProfileMode, ScopeReport};

type Observation = ScopeReport<DocumentGuestWork, DocumentGuestResource>;

fn host(mode: ProfileMode, node_work: u64) -> DocumentScriptHost {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 2,
        nested_dispatches: 2,
        storage_bytes: 32_000,
        work: 10_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    let prepared = PreparedDocumentScriptHost::prepare(session, 2_000_000, cancel)
        .unwrap()
        .with_guest_accounting_profile(mode);
    enable_public_nodes(
        prepared,
        target,
        (node_work, public_node_source_limits(20_000, 64), 4),
        &mut Some(registry),
    )
}

fn projection(report: &DocumentGuestAccountingReport) -> &Observation {
    let [root, projection, capture] = report.scopes.as_slice() else {
        panic!("expected adapter, projection and binding preparation scopes");
    };
    assert_eq!(root.name, "runtime-guest");
    assert_eq!(projection.name, "node-projection");
    assert_eq!(projection.parent, Some(root.scope));
    assert_eq!(capture.name, "binding-preparation");
    assert_eq!(capture.parent, Some(root.scope));
    projection
}

fn open(host: &mut DocumentScriptHost) -> Result<(), DocumentScriptError> {
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let object = scope.evaluate(&units("document.documentElement"))?;
        assert!(matches!(
            scope.kind(&object)?,
            surgeist_bindings_js::ValueKind::Object
        ));
        Ok(())
    })
}

pub(in crate::document_scripting) fn profiles_observe_actual_projection_without_changing_admission()
{
    oversized_profile_setup_returns_the_unpublished_owner();
    const NATIVE_WATCH_WORK: u64 = 5 + 2 * usize::BITS as u64 + 1;
    let mut control = host(ProfileMode::Off, NATIVE_WATCH_WORK);
    let events = control.guest_events().unwrap();
    let before = (events.budget.work().unwrap(), events.budget.used());
    let (result, peak) = events.budget.measure_peak(|| open(&mut control));
    result.unwrap();
    let work = events.budget.work().unwrap() - before.0;
    let bytes = peak - before.1;
    let retained = events.budget.used() - before.1;
    assert!(
        control
            .guest_accounting()
            .unwrap()
            .snapshot()
            .unwrap()
            .is_none()
    );

    drop((control, events));

    for (mode, timed, detailed) in [
        (ProfileMode::Summary { timing: false }, false, false),
        (ProfileMode::Summary { timing: true }, true, false),
        (
            ProfileMode::Detailed {
                timing: true,
                event_limit: 8,
                filter: EventFilter::ScopeName("node-projection"),
            },
            true,
            true,
        ),
    ] {
        let mut host = host(mode, NATIVE_WATCH_WORK);
        let account = host.guest_accounting().unwrap();
        let initial = account.snapshot().unwrap().unwrap();
        let initial_root = &initial.scopes[0];
        let attached = projection(&initial);
        let attached_bytes = attached.usage.live(DocumentGuestResource::BackingBytes);
        assert!(attached_bytes > 0 && attached.phases > 0);
        let events = host.guest_events().unwrap();
        let before_bytes = events.budget.used();
        let spare = events.limits.storage_bytes - before_bytes - bytes;
        let guard = events.reserve(&host.native, spare).unwrap();
        events.budget.set_work_remaining(work);
        open(&mut host).unwrap();
        let report = account.snapshot().unwrap().unwrap();
        let current = projection(&report);
        assert_eq!(report.elapsed.is_some(), timed);
        assert_eq!(current.phase_elapsed.is_some(), timed);
        assert!(current.phases > 0);
        assert!(current.usage.work(DocumentGuestWork::Adapter) > 0);
        assert_eq!(
            report.scopes[0].usage.work(DocumentGuestWork::Adapter)
                - initial_root.usage.work(DocumentGuestWork::Adapter),
            work + 1
        );
        assert_eq!(
            report.scopes[0]
                .usage
                .live(DocumentGuestResource::BackingBytes),
            (before_bytes + spare + retained) as u64
        );
        assert_eq!(events.budget.peak_used(), events.limits.storage_bytes);
        assert_eq!(
            current.usage.live(DocumentGuestResource::BackingBytes),
            attached_bytes + retained as u64
        );
        let capture = &report.scopes[2];
        assert_eq!(
            capture.usage.live(DocumentGuestResource::BackingBytes),
            initial.scopes[2]
                .usage
                .live(DocumentGuestResource::BackingBytes)
        );
        // Compile/evaluate result slots; Window getter frame, receiver, global
        // and reopened Document; Element getter frame, receiver and new root.
        assert_eq!(
            capture.usage.work(DocumentGuestWork::Adapter)
                - initial.scopes[2].usage.work(DocumentGuestWork::Adapter),
            2 + 4 + 3
        );
        assert_eq!(report.events.len(), if detailed { 8 } else { 0 });
        assert_eq!(report.dropped_events > 0, detailed);
        if detailed {
            assert!(report.events.len() > initial.events.len());
            assert!(report.events.iter().all(|event| match event {
                surgeist_ledger::Event::Accepted { scope, .. }
                | surgeist_ledger::Event::Phase { scope, .. } => *scope == current.scope,
                surgeist_ledger::Event::Refused { origin, .. } => *origin == current.scope,
            }));
        }

        drop(guard);
        events.budget.set_work_remaining(work);
        open(&mut host).unwrap();
        let reopened = account.snapshot().unwrap().unwrap();
        let reopened_scope = projection(&reopened);
        assert_eq!(
            reopened_scope
                .usage
                .live(DocumentGuestResource::BackingBytes),
            current.usage.live(DocumentGuestResource::BackingBytes)
        );
        assert!(
            reopened_scope.usage.work(DocumentGuestWork::Adapter)
                > current.usage.work(DocumentGuestWork::Adapter)
        );
        drop((host, events));
        let mut host = self::host(mode, 1_000);
        let account = host.guest_accounting().unwrap();
        let permit = host.permit().unwrap();
        let detached = host
            .in_document(&permit, |scope| {
                let fragment = scope.evaluate(&units("new DocumentFragment()"))?;
                projected_node_id(scope, &fragment)
            })
            .unwrap();
        let live = account.snapshot().unwrap().unwrap();
        let live = projection(&live)
            .usage
            .live(DocumentGuestResource::BackingBytes);
        host.with_native(|dom, _, _| {
            let mut batch = surgeist_dom::DomBatch::new();
            batch.destroy(detached);
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
        })
        .unwrap();
        let retired = account.snapshot().unwrap().unwrap();
        assert!(
            projection(&retired)
                .usage
                .live(DocumentGuestResource::BackingBytes)
                < live
        );

        drop(host);
        let finished = account.snapshot().unwrap().unwrap();
        assert_eq!(
            finished.scopes[0]
                .usage
                .live(DocumentGuestResource::BackingBytes),
            0
        );
        assert_eq!(
            projection(&finished)
                .usage
                .live(DocumentGuestResource::BackingBytes),
            0
        );
    }

    for (native, work_remaining, storage_short) in [
        (NATIVE_WATCH_WORK, work - 1, false),
        (NATIVE_WATCH_WORK, work, true),
        (NATIVE_WATCH_WORK - 1, work, false),
    ] {
        let mut host = host(ProfileMode::Summary { timing: false }, native);
        let account = host.guest_accounting().unwrap();
        let events = host.guest_events().unwrap();
        let spare = events.limits.storage_bytes - events.budget.used() - bytes;
        let guard = events
            .reserve(&host.native, spare + usize::from(storage_short))
            .unwrap();
        events.budget.set_work_remaining(work_remaining);
        let stamp = host.snapshot().unwrap().stamp();
        let initial = account.snapshot().unwrap().unwrap();
        let error = open(&mut host).unwrap_err();
        let DocumentScriptError::Script(Error::HostFailure(failure)) = error else {
            panic!("expected original Runtime projection refusal: {error:?}");
        };
        let cause = failure.cause::<DocumentGuestEventError>();
        if storage_short {
            assert!(matches!(
                cause,
                Some(DocumentGuestEventError::Document(
                    DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                ))
            ));
        } else if native < NATIVE_WATCH_WORK {
            assert!(matches!(cause, Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                    DomError::NodeRetirementWorkLimit { limit }
                ))
            )) if *limit == native));
        } else {
            assert!(matches!(cause, Some(DocumentGuestEventError::WorkLimit)));
        }
        let after = account.snapshot().unwrap().unwrap();
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(
            after.scopes[0]
                .usage
                .live(DocumentGuestResource::BackingBytes),
            initial.scopes[0]
                .usage
                .live(DocumentGuestResource::BackingBytes)
        );
        assert_eq!(
            projection(&after)
                .usage
                .live(DocumentGuestResource::BackingBytes),
            projection(&initial)
                .usage
                .live(DocumentGuestResource::BackingBytes)
        );
        assert!(projection(&after).usage.work(DocumentGuestWork::Adapter) > 0);
        assert!(projection(&after).first_refusal.is_none());
        assert_eq!(
            after.scopes[2].first_refusal.is_some(),
            native == NATIVE_WATCH_WORK,
            "native={native} work={work_remaining} storage_short={storage_short}; preparation={:?}",
            after.scopes[2].first_refusal,
        );
        assert!(after.scopes[0].first_refusal.is_none());
        if let Some(refusal) = after.scopes[2].first_refusal {
            assert_eq!(refusal.scope, after.scopes[0].scope);
            let expected = if storage_short {
                let bytes = host
                    .engine
                    .as_ref()
                    .unwrap()
                    .process
                    .managed_object_storage_bytes()
                    + size_of::<crate::document_scripting::preparation::Backing>();
                surgeist_ledger::Change::Acquire(
                    DocumentGuestResource::BackingBytes,
                    u64::try_from(bytes).unwrap(),
                )
            } else {
                surgeist_ledger::Change::Work(DocumentGuestWork::Adapter, 1)
            };
            assert_eq!(refusal.change, expected);
        }
        drop((guard, failure, host, events));
        let final_report = account.snapshot().unwrap().unwrap();
        assert_eq!(
            final_report.scopes[0]
                .usage
                .live(DocumentGuestResource::BackingBytes),
            0
        );
    }
}

fn oversized_profile_setup_returns_the_unpublished_owner() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 2,
        nested_dispatches: 2,
        storage_bytes: 32_000,
        work: 10_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    let mut registry = Some(registry);
    let stamp = session.snapshot().stamp();
    let prepared = PreparedDocumentScriptHost::prepare(session, 2_000_000, cancel).unwrap();
    let prepared = prepared.with_guest_accounting_profile(ProfileMode::Detailed {
        timing: true,
        event_limit: 100_000,
        filter: EventFilter::All,
    });
    let policy = HtmlScriptingPolicy {
        user_allows: true,
        automation_allows: true,
    };
    let read = units("readChildCount");
    let detach = units("detachIf");
    let hooks = DocumentScriptHooks {
        child_count: &read,
        detach_source_node_if: &detach,
    };
    let limits = DocumentGuestEventLimits {
        storage_bytes: 192 * 1024,
        work: 10_000,
        pending_exception_reports: 4,
        node_retirement: DomNodeRetirementLimits::new(16_000_000, 1_000),
        source_edits: DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 3),
        dispatch: DomEventDispatchLimits {
            path_items: 8,
            snapshot_listeners: 8,
            failure_records: 4,
            callbacks: 8,
            work: 10_000,
        },
    };
    let Err(failure) =
        prepared.enable_with_guest_event_registry(target, policy, hooks, &mut registry, limits)
    else {
        panic!("observer setup exceeds the prepared host's storage");
    };
    assert!(matches!(
        failure.error,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::StorageLimit
        ))
    ));
    assert!(registry.is_some());
    assert!(failure.owner.native.attachment.get().is_none());
    assert_eq!(
        failure.owner.native.session.borrow().snapshot().stamp(),
        stamp
    );
    let host = enable_public_nodes(
        failure
            .owner
            .with_guest_accounting_profile(ProfileMode::Summary { timing: false }),
        target,
        (1_000, public_node_source_limits(20_000, 64), 4),
        &mut registry,
    );
    assert!(registry.is_none());
    assert!(
        host.guest_accounting()
            .unwrap()
            .snapshot()
            .unwrap()
            .is_some()
    );
}
