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
        (node_work, 20_000, 4),
        &mut Some(registry),
    )
}

fn projection(report: &DocumentGuestAccountingReport) -> &Observation {
    let [root, projection] = report.scopes.as_slice() else {
        panic!("expected one adapter and one projection scope");
    };
    assert_eq!(root.name, "runtime-guest");
    assert_eq!(projection.name, "node-projection");
    assert_eq!(projection.parent, Some(root.scope));
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
    const NATIVE_WATCH_WORK: u64 = 5 + 2 * usize::BITS as u64 + 1;
    let mut control = host(ProfileMode::Off, NATIVE_WATCH_WORK);
    let events = control.guest_events().unwrap();
    let before = (events.budget.work().unwrap(), events.budget.used());
    open(&mut control).unwrap();
    let work = events.budget.work().unwrap() - before.0;
    let bytes = events.budget.used() - before.1;
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
                event_limit: 2,
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
        let events = host.guest_events().unwrap();
        let spare = events.limits.storage_bytes - events.budget.used() - bytes;
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
            events.limits.storage_bytes as u64
        );
        assert_eq!(
            current.usage.live(DocumentGuestResource::BackingBytes),
            bytes as u64
        );
        assert_eq!(report.events.len(), if detailed { 2 } else { 0 });
        assert_eq!(report.dropped_events > 0, detailed);

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
            0
        );
        assert!(projection(&after).usage.work(DocumentGuestWork::Adapter) > 0);
        assert_eq!(
            projection(&after).first_refusal.is_some(),
            native == NATIVE_WATCH_WORK
        );
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
