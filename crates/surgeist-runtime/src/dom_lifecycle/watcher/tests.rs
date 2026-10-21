//! Real Source hooks, original Window membership and affine-effect retries.
mod close;
mod request;
mod show_modal;
use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomHostedTaskProgress, DomHostedTopLevel, DomSession,
    DomSessionLimits, DomShutdownOutcome, DomShutdownReason, DomSourceDriveProgress,
    DomSourceEventRuntimeLimits, DomSourceServices, DomStyleRuntimeLimits,
};
use surgeist_dom::*;
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4)
}
fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 4,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: surgeist_css::CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 8,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn hosted_queue_fixture() -> (DomSession, DomHostedTopLevel, NodeId) {
    let cancel = AtomicBool::new(false);
    let dom = Dom::new(
        DomLimits::new(
            128,
            100_000,
            256,
            128,
            128,
            32,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        )
        .with_details_toggle_intentions(8)
        .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel).unwrap()
    };
    let snapshot = session.snapshot();
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let children = snapshot.node(html).unwrap().children();
    assert_eq!(children.len(), 2);
    (session, top, children[1])
}

fn fixture() -> (DomSession, DomHostedTopLevel, NodeId, NodeId) {
    let (mut session, top, body) = hosted_queue_fixture();
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&subject, body, None);
    let subject = session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    (session, top, body, subject)
}
fn apply(session: &mut DomSession, edit: DomSourceEdit) -> DomSourceCompletion {
    let mut policy = |_: &DomStyleSource| None;
    match session
        .source_execution()
        .apply(
            &mut session.dom(),
            edit,
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &AtomicBool::new(false),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Complete(done) => {
            assert!(!done.cancellation_requested());
            done
        }
        other => panic!("real ordinary hook did not complete: {other:?}"),
    }
}
fn pending(lease: &mut DomSourceLease<'_>) -> DomSourceDialogRequest {
    match lease.advance(&AtomicBool::new(false)).unwrap() {
        DomSourceProgress::Request(DomSourceRequest::Dialog(request)) => request,
        other => panic!("actual Hook required: {other:?}"),
    }
}
fn counts(hosted: &HostedRuntime) -> (usize, usize, usize) {
    let owner = hosted.state().unwrap();
    let state = owner.borrow();
    let agent = &state.hosts[0].top_levels[0]
        .group
        .as_ref()
        .unwrap()
        .clusters[0]
        .agent;
    (
        agent.windows.rows.len(),
        agent
            .windows
            .rows
            .iter()
            .map(|window| window.groups.len())
            .sum(),
        agent
            .windows
            .rows
            .iter()
            .flat_map(|window| &window.groups)
            .map(|group| group.members.len())
            .sum(),
    )
}
#[test]
fn actual_window_manager_shares_group_and_releases_each_removed_member_once() {
    let (mut session, top, body, first) = fixture();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let second = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&second, body, None);
    let second = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&second)
        .unwrap()
        .unwrap();
    let hosted = session.hosted_execution();
    assert_eq!(counts(&hosted), (1, 0, 0));
    apply(
        &mut session,
        DomSourceEdit::set_attribute(first, "open", ""),
    );
    let first_id = session
        .snapshot()
        .node(first)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    apply(
        &mut session,
        DomSourceEdit::set_attribute(second, "open", ""),
    );
    let second_id = session
        .snapshot()
        .node(second)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    assert_ne!(first_id, second_id);
    assert_eq!(first_id.global(), top.global());
    assert_eq!(second_id.global(), top.global());
    assert_eq!(counts(&hosted), (1, 1, 2));
    let before = session.snapshot();
    let mut atomic = DomBatch::new();
    atomic.remove_attribute(first, "open");
    assert!(
        matches!(session.dom().apply(atomic,&cancel),Err(DomError::NeedsEventfulOperation { subject:Some(target),action:HtmlDialogAction::DestroyCloseWatcher }) if target==first)
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(counts(&hosted), (1, 1, 2));
    apply(&mut session, DomSourceEdit::remove_attribute(first, "open"));
    assert_eq!(counts(&hosted), (1, 1, 1));
    apply(
        &mut session,
        DomSourceEdit::remove_attribute(second, "open"),
    );
    assert_eq!(counts(&hosted), (1, 0, 0));
    assert!(
        before
            .node(first)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert!(
        !session
            .snapshot()
            .node(first)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
}
#[test]
fn refused_watcher_acceptance_retries_same_real_registration_without_replay() {
    let (mut session, _top, _body, subject) = fixture();
    let cancel = AtomicBool::new(false);
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let (request, location, id) = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .begin_source_edit(
                DomSourceEdit::set_attribute(subject, "open", ""),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let request = pending(&mut lease);
        assert_eq!(request.action(), HtmlDialogAction::EstablishCloseWatcher);
        let location = hosted
            .install_dialog_watcher(&mut lease, &request, &cancel)
            .unwrap()
            .unwrap();
        let id = {
            let mut state = owner.borrow_mut();
            let actual = location.get(&mut state);
            assert!(actual.publication.is_some());
            actual.id
        };
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            hosted.accept_watcher(&mut lease, &request, &location, &cancel),
            Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                DomError::Cancelled
            )))
        ));
        assert!(location.get(&mut owner.borrow_mut()).publication.is_some());
        drop(lease);
        (request, location, id)
    };
    drop(hosted);
    cancel.store(false, Ordering::Relaxed);
    let hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(request.operation(), source_limits(), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_dialog_watcher(&mut lease, &request, &cancel)
                .unwrap()
        );
        assert_eq!(counts(&hosted), (1, 1, 1));
        assert_eq!(location.get(&mut owner.borrow_mut()).id, id);
        assert!(location.get(&mut owner.borrow_mut()).publication.is_none());
        let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
        let done = lease.advance(&cancel).unwrap();
        settlement.finish(&lease.snapshot());
        assert!(matches!(done, DomSourceProgress::Complete(_)));
        assert_eq!(
            location.get(&mut owner.borrow_mut()).phase,
            WatcherPhase::Registered
        );
        drop(lease);
    }
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(id)
    );
}
#[test]
fn retired_window_retains_host_and_whole_session_until_canonical_clear_is_joined() {
    let (mut session, top, _body, subject) = fixture();
    let cancel = AtomicBool::new(false);
    apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    );
    let mut hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let host = owner.borrow().hosts[0].id;
    let task = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let mut complete = false;
    for _ in 0..32 {
        if matches!(hosted.run_next(&mut session.dom(),task.traversable(),&cancel).unwrap(),DomHostedTaskProgress::Complete {operation} if operation==task.operation())
        {
            complete = true;
            break;
        }
    }
    assert!(complete);
    assert_eq!(counts(&hosted), (1, 1, 1));
    assert!(
        owner
            .borrow_mut()
            .find_global(top.global(), &cancel, || Ok(()))
            .is_err()
    );
    assert!(
        owner.borrow().hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent
            .windows
            .rows[0]
            .retired
    );
    assert!(matches!(
        hosted.destroy_host(&mut session.dom(), host, &cancel),
        Err(DomHostedRuntimeError::OperationPending)
    ));
    session = match session.try_shutdown() {
        DomShutdownOutcome::Pending {
            session,
            reason: DomShutdownReason::HostedLifecycle,
        } => session,
        other => panic!("actual retained Window must preserve owner custody: {other:?}"),
    };
    apply(
        &mut session,
        DomSourceEdit::remove_attribute(subject, "open"),
    );
    assert!(owner.borrow().hosts[0].top_levels.is_empty());
    hosted
        .destroy_host(&mut session.dom(), host, &cancel)
        .unwrap();
    drop(owner);
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

#[test]
fn real_removal_replacement_adoption_and_shadow_descendants_drain_original_watchers() {
    for branch in 0..4 {
        let (mut session, top, body, subject) = fixture();
        let cancel = AtomicBool::new(false);
        apply(
            &mut session,
            DomSourceEdit::set_attribute(subject, "open", ""),
        );
        let old = session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        let hosted = session.hosted_execution();
        match branch {
            0 => {
                apply(&mut session, DomSourceEdit::detach(subject));
            }
            1 => {
                let mut batch = DomBatch::new();
                let other = batch.create_element_ns(top.document(), Namespace::html(), "span");
                let other = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .created(&other)
                    .unwrap()
                    .unwrap();
                apply(
                    &mut session,
                    DomSourceEdit::replace_child(other, body, subject),
                );
            }
            2 => {
                let mut batch = DomBatch::new();
                let document = batch.create_document();
                let document = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .document(&document)
                    .unwrap();
                apply(&mut session, DomSourceEdit::adopt(subject, document.node()));
                assert_eq!(
                    session.snapshot().node(subject).unwrap().owner_document(),
                    Some(document)
                );
            }
            3 => {
                let mut batch = DomBatch::new();
                let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
                let shadow =
                    batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
                batch.insert_before(&host, body, None);
                let done = session.dom().apply(batch, &cancel).unwrap();
                let host = done.created(&host).unwrap().unwrap();
                let shadow = done.created(&shadow).unwrap().unwrap();
                apply(
                    &mut session,
                    DomSourceEdit::insert_before(subject, shadow, None),
                );
                assert_eq!(counts(&hosted), (1, 1, 1));
                apply(&mut session, DomSourceEdit::detach(host));
            }
            _ => unreachable!(),
        }
        assert_eq!(counts(&hosted), (1, 0, 0), "branch {branch}");
        assert!(
            !session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
        assert_eq!(
            session.snapshot().relevant_global(subject).unwrap(),
            Some(top.global())
        );
        // Reinsertion is a new real occurrence in the original Window; neither
        // adoption nor a fresh current Document substitutes its Global.
        apply(
            &mut session,
            DomSourceEdit::insert_before(subject, body, None),
        );
        let new = session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        assert_ne!(new, old);
        assert_eq!(new.global(), top.global());
        assert_eq!(counts(&hosted), (1, 1, 1));
        apply(&mut session, DomSourceEdit::detach(subject));
        apply(&mut session, DomSourceEdit::destroy(subject));
        assert_eq!(
            session.snapshot().node(subject).unwrap_err(),
            DomError::StaleNode
        );
        assert_eq!(counts(&hosted), (1, 0, 0));
    }
}

#[test]
fn actual_member_replacement_peak_one_short_refuses_before_registration_and_retries() {
    for short in [true, false] {
        let (mut session, top, body, first) = fixture();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let second = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
        batch.insert_before(&second, body, None);
        let second = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&second)
            .unwrap()
            .unwrap();
        apply(
            &mut session,
            DomSourceEdit::set_attribute(first, "open", ""),
        );
        let mut hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let (base, mut limits) = {
            let mut state = owner.borrow_mut();
            (state.retained_bytes(&cancel).unwrap(), state.limits)
        };
        let first_capacity = {
            let state = owner.borrow();
            state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .groups[0]
                .members
                .capacity()
        };
        assert_eq!(
            first_capacity, 1,
            "real bank from empty has one represented row"
        );
        // Current retained bytes already include the old one-row backing.
        // The owning call forms an actual group candidate header and a separate
        // full two-row replacement before moving either old member.
        let replacement_overlap = 2 * size_of::<Rc<RefCell<Watcher>>>();
        let exact = base
            + size_of::<WatcherGroup>()
            + replacement_overlap.max(watcher_body_bytes() + size_of::<Rc<RefCell<Watcher>>>());
        limits.storage_bytes = exact - usize::from(short);
        hosted
            .set_limits(&mut session.dom(), limits, &cancel)
            .unwrap();
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .begin_source_edit(
                DomSourceEdit::set_attribute(second, "open", ""),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let request = pending(&mut lease);
        let before = lease.snapshot().stamp();
        let installed = hosted.install_dialog_watcher(&mut lease, &request, &cancel);
        let location = if short {
            assert!(matches!(installed, Err(DomHostedRuntimeError::Admission)));
            assert_eq!(counts(&hosted), (1, 1, 1));
            assert_eq!(lease.snapshot().stamp(), before);
            owner.borrow_mut().limits.storage_bytes = exact;
            hosted
                .install_dialog_watcher(&mut lease, &request, &cancel)
                .unwrap()
                .unwrap()
        } else {
            installed.unwrap().unwrap()
        };
        assert_eq!(counts(&hosted), (1, 1, 2));
        hosted
            .accept_watcher(&mut lease, &request, &location, &cancel)
            .unwrap();
        let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
        let result = lease.advance(&cancel).unwrap();
        settlement.finish(&lease.snapshot());
        assert!(matches!(result, DomSourceProgress::Complete(_)));
        assert_eq!(
            location.get(&mut owner.borrow_mut()).phase,
            WatcherPhase::Registered
        );
    }
}

#[test]
fn actual_created_hook_repeats_same_exposed_identity_before_real_watcher_service() {
    let (mut session, top, body, _) = fixture();
    let cancel = AtomicBool::new(false);
    let before = session.snapshot();
    let prior = before.node(body).unwrap().children().to_vec();
    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(16, 16, 16, 4096, 16),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "dialog"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "open"),
                "",
            )])
            .unwrap(),
        )
        .unwrap();
    let fragment = builder.finish();
    assert_eq!(fragment.root().children().len(), 1);
    let source_subject = fragment.root().children().next().unwrap();
    let target = DomNativeHtmlInstallTarget::new(
        before.document(top.document()).unwrap().stamp(),
        body,
        None,
    );
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .begin_native_installation(
            target,
            &fragment,
            DomSourceNativeHtmlInstallLimits::new(
                DomNativeHtmlInstallLimits::new(16, 1_000_000, 16_000_000),
                source_limits(),
            ),
            &cancel,
        )
        .unwrap();
    let request = pending(&mut lease);
    assert_eq!(request.action(), HtmlDialogAction::EstablishCloseWatcher);
    let subject = request.subject();
    assert!(before.node(subject).is_err());
    let first = lease.snapshot();
    assert_eq!(first.node(subject).unwrap().parent(), Some(body));
    assert_eq!(first.node(body).unwrap().children().len(), prior.len() + 1);
    assert_eq!(
        first.node(subject).unwrap().attribute(&"open".into()),
        Some(&"".into())
    );
    assert_eq!(
        first.incomplete_source_operation(),
        Some(request.operation())
    );
    let repeated = pending(&mut lease);
    assert_eq!(repeated, request);
    assert_eq!(lease.snapshot().stamp(), first.stamp());
    assert_eq!(
        lease.snapshot().node(body).unwrap().children(),
        first.node(body).unwrap().children()
    );
    assert_eq!(counts(&hosted), (1, 0, 0));
    assert!(
        hosted
            .service_dialog_watcher(&mut lease, &request, &cancel)
            .unwrap()
    );
    let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
    let done = lease.advance(&cancel).unwrap();
    settlement.finish(&lease.snapshot());
    let DomSourceProgress::Complete(done) = done else {
        panic!("actual installation must finish after the real watcher receipt");
    };
    assert_eq!(
        done.native_installation()
            .unwrap()
            .node_for(&source_subject),
        Some(subject)
    );
    assert_eq!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap()
            .subject(),
        subject
    );
    assert_eq!(counts(&hosted), (1, 1, 1));
    let mut state = owner.borrow_mut();
    assert_eq!(
        state.hosts[0].top_levels[0]
            .group
            .as_mut()
            .unwrap()
            .clusters[0]
            .agent
            .windows
            .rows[0]
            .groups[0]
            .members[0]
            .borrow()
            .phase,
        WatcherPhase::Registered
    );
}

#[test]
fn refused_final_association_publication_retains_actual_receipt_and_same_registration() {
    let (mut session, _top, _body, subject) = fixture();
    let cancel = AtomicBool::new(false);
    let before = session.snapshot().stamp();
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .begin_source_edit(
            DomSourceEdit::set_attribute(subject, "open", ""),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let request = pending(&mut lease);
    let location = hosted
        .install_dialog_watcher(&mut lease, &request, &cancel)
        .unwrap()
        .unwrap();
    hosted
        .accept_watcher(&mut lease, &request, &location, &cancel)
        .unwrap();
    let id = location.get(&mut owner.borrow_mut()).id;
    let prefix = lease.snapshot();
    drop(lease);
    // Quota units are actual published change records, independently observed
    // from the product's ordinary history. One close fits; association+close do not.
    let published = dom.raw().changes_since(before).unwrap().changes().len();
    let ordinary = source_limits();
    let one_short = DomSourceLimits::new(
        ordinary.retained_bytes(),
        ordinary.retained_string_code_units(),
        ordinary.work(),
        ordinary.phase_publications(),
        published + 1,
        ordinary.published_native_identities(),
        ordinary.nested_operations(),
    );
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(request.operation(), one_short, &cancel)
        .unwrap();
    let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
    assert!(matches!(
        lease.advance(&cancel),
        Err(DomError::SourceOutputLimit)
    ));
    settlement.finish(&lease.snapshot());
    assert_eq!(lease.snapshot().stamp(), prefix.stamp());
    assert_eq!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        None
    );
    assert_eq!(location.get(&mut owner.borrow_mut()).id, id);
    assert!(location.get(&mut owner.borrow_mut()).publication.is_none());
    assert_eq!(
        location.get(&mut owner.borrow_mut()).phase,
        WatcherPhase::Registering
    );
    drop(lease);
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(request.operation(), ordinary, &cancel)
        .unwrap();
    let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
    let done = lease.advance(&cancel).unwrap();
    settlement.finish(&lease.snapshot());
    assert!(matches!(done, DomSourceProgress::Complete(_)));
    assert_eq!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(id)
    );
    assert_eq!(counts(&hosted), (1, 1, 1));
    assert_eq!(
        location.get(&mut owner.borrow_mut()).phase,
        WatcherPhase::Registered
    );
}

#[test]
fn retired_window_settlement_work_is_admitted_before_source_close_and_pruning() {
    let (mut session, top, _body, subject) = fixture();
    let cancel = AtomicBool::new(false);
    apply(
        &mut session,
        DomSourceEdit::set_attribute(subject, "open", ""),
    );
    let mut hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let task = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel)
        .unwrap();
    let mut closed = false;
    for _ in 0..32 {
        if matches!(hosted.run_next(&mut session.dom(),task.traversable(),&cancel).unwrap(),DomHostedTaskProgress::Complete {operation} if operation==task.operation())
        {
            closed = true;
            break;
        }
    }
    assert!(closed);
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .begin_source_edit(
            DomSourceEdit::remove_attribute(subject, "open"),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let request = pending(&mut lease);
    let location = hosted
        .install_dialog_watcher(&mut lease, &request, &cancel)
        .unwrap()
        .unwrap();
    hosted
        .accept_watcher(&mut lease, &request, &location, &cancel)
        .unwrap();
    let before = lease.snapshot();
    // Membership/group are already physically removed; one HookRemoval stays
    // in custody. Current scans visit host/composition/cluster/Window4 and
    // second-pass host/composition2 =6. Prepaid finish visits are first-pass
    // headers/window retain6 + custody predicate/drop2 + second-pass host/
    // composition/cluster pruning3 =11. No composition tail rows: total17.
    const REQUIRED: u64 = 17;
    let first_cap = {
        let mut state = owner.borrow_mut();
        let cap = state.work + REQUIRED - 1;
        state.limits.work = cap;
        cap
    };
    assert!(matches!(
        hosted.prepare_watcher_settlement(&lease, &cancel),
        Err(DomHostedRuntimeError::Admission)
    ));
    assert_eq!(owner.borrow().work, first_cap);
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(
        location.get(&mut owner.borrow_mut()).phase,
        WatcherPhase::Removing
    );
    let retry_cap = {
        let mut state = owner.borrow_mut();
        let cap = state.work + REQUIRED;
        state.limits.work = cap;
        cap
    };
    let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
    assert_eq!(
        owner.borrow().work,
        retry_cap,
        "failed-attempt work was not reset"
    );
    let result = lease.advance(&cancel).unwrap();
    assert!(matches!(result, DomSourceProgress::Complete(_)));
    settlement.finish(&lease.snapshot());
    assert_eq!(
        owner.borrow().work,
        retry_cap,
        "no fallible post-close work debit"
    );
    assert!(owner.borrow().hosts[0].top_levels.is_empty());
}

fn dispatcher<C>() -> crate::DomEventDispatcher<C> {
    crate::DomEventDispatcher::new(crate::DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap()
}
fn event_limits() -> crate::DomEventDispatchLimits {
    crate::DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 32,
        failure_records: 32,
        callbacks: 32,
        work: 100_000,
    }
}
fn show_event(
    session: &mut DomSession,
    subject: NodeId,
    limits: DomSourceLimits,
) -> DomSourceEventRequest {
    let mut policy = |_: &DomStyleSource| None;
    match session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(subject),
            limits,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &AtomicBool::new(false),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            ..
        } => request,
        other => panic!("genuine original beforetoggle required: {other:?}"),
    }
}
#[test]
fn active_show_reserves_four_phases_and_known_outputs_before_real_queue_effect() {
    for refusal in [Some(true), Some(false), None] {
        let (mut session, top, _body, subject) = fixture();
        let cancel = AtomicBool::new(false);
        let ordinary = source_limits();
        let bounded = DomSourceLimits::new(
            ordinary.retained_bytes(),
            ordinary.retained_string_code_units(),
            ordinary.work(),
            if refusal == Some(true) { 3 } else { 4 },
            match refusal {
                Some(true) => 4096,
                Some(false) => 5,
                None => 6,
            },
            ordinary.published_native_identities(),
            ordinary.nested_operations(),
        );
        let request = show_event(&mut session, subject, bounded);
        let mut listeners = dispatcher();
        let mut calls = 0usize;
        listeners
            .add_listener(
                &mut session.dom(),
                subject,
                crate::DomEventType::BeforeToggle,
                crate::DomEventCallback::new(|_, _, calls: &mut usize, _| {
                    *calls += 1;
                    Ok(())
                }),
                crate::DomEventListenerOptions::default(),
            )
            .unwrap();
        let serviced = session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut calls,
                crate::DomSourceEventServiceLimits {
                    source: bounded,
                    dispatch: event_limits(),
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            serviced,
            crate::DomSourceEventServiceProgress::Serviced { .. }
        ));
        assert_eq!(calls, 1);
        let mut policy = |_: &DomStyleSource| None;
        if let Some(phase_short) = refusal {
            let before = session.snapshot();
            let error = session
                .source_execution()
                .resume(
                    &mut session.dom(),
                    request.operation(),
                    bounded,
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None,
                    },
                    &cancel,
                )
                .unwrap_err();
            if phase_short {
                assert!(matches!(
                    error,
                    crate::DomSourceDriveError::Source(crate::DomSourceRuntimeError::ElementTasks(
                        crate::DomElementTaskRuntimeError::Dom(DomError::SourcePublicationLimit)
                    ))
                ))
            } else {
                assert!(matches!(
                    error,
                    crate::DomSourceDriveError::Source(crate::DomSourceRuntimeError::ElementTasks(
                        crate::DomElementTaskRuntimeError::Dom(DomError::SourceOutputLimit)
                    ))
                ))
            }
            assert_eq!(session.snapshot().stamp(), before.stamp());
            assert_eq!(
                session
                    .element_task_execution()
                    .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
                    .unwrap(),
                0
            );
            assert_eq!(counts(&session.hosted_execution()), (1, 0, 0));
        }
        // Tracker+Source start2, open attribute+Document list2, watcher state1,
        // and final Source close1. Three of the four phases are published here;
        // the retained synchronous tail still owns the fourth closing phase.
        let exact = DomSourceLimits::new(
            ordinary.retained_bytes(),
            ordinary.retained_string_code_units(),
            ordinary.work(),
            4,
            6,
            ordinary.published_native_identities(),
            ordinary.nested_operations(),
        );
        let next = session
            .source_execution()
            .resume(
                &mut session.dom(),
                request.operation(),
                exact,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        assert!(
            matches!(next,DomSourceDriveProgress::Pending {request:DomSourceRequest::Dialog(ref pending),..} if pending.action()==HtmlDialogAction::RememberFocusedElement)
        );
        assert_eq!(
            session
                .element_task_execution()
                .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
                .unwrap(),
            1
        );
        assert_eq!(counts(&session.hosted_execution()), (1, 1, 1));
        let snapshot = session.snapshot();
        let dialog = snapshot.node(subject).unwrap().as_dialog().unwrap();
        let watcher = dialog.close_watcher().expect("real established watcher");
        assert_eq!(watcher.subject(), subject);
        assert_eq!(watcher.global(), top.global());
        assert_eq!(
            snapshot.node(subject).unwrap().attribute(&"open".into()),
            Some(&"".into())
        );
        assert_eq!(
            snapshot.incomplete_source_operation(),
            Some(request.operation())
        );
        let status = session.dom().pending_source_edit().unwrap();
        assert_eq!(status.operation(), request.operation());
        assert_eq!(status.published_phases(), 3);
        assert!(!status.cancellation_requested());
        let task = session
            .element_task_execution()
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap()
            .dialog_toggle()
            .unwrap();
        assert_eq!(dialog.toggle_task_tracker().unwrap().task(), task.id());
        assert_eq!(task.id().target(), subject);
        assert_eq!(task.old_state(), surgeist_html::HtmlDialogState::Closed);
        assert_eq!(task.new_state(), surgeist_html::HtmlDialogState::Open);
        assert_eq!(task.source(), None);
        let association = task.queue_association().unwrap();
        assert_eq!(association.global(), top.global());
        assert_eq!(association.document(), top.document());
        assert_eq!(calls, 1);
    }
}

#[test]
fn real_child_complete_keeps_registered_window_payload_under_the_original_parent_observer() {
    struct Context {
        source: crate::DomSourceRuntime,
        child: Option<Result<DomSourceDriveProgress, crate::DomSourceDriveError>>,
        parent: Option<DomSourceOperationId>,
        watcher: Option<DomCloseWatcherId>,
        calls: usize,
    }
    let (mut session, top, body, parent) = fixture();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let child = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&child, body, None);
    let child = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&child)
        .unwrap()
        .unwrap();
    let request = show_event(&mut session, parent, source_limits());
    let mut context = Context {
        source: session.source_execution(),
        child: None,
        parent: None,
        watcher: None,
        calls: 0,
    };
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &mut session.dom(),
            parent,
            crate::DomEventType::BeforeToggle,
            crate::DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                context.calls += 1;
                event.prevent_default();
                let mut policy = |_: &DomStyleSource| None;
                context.child = Some(context.source.apply(
                    dom,
                    DomSourceEdit::set_attribute(child, "open", ""),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None,
                    },
                    &AtomicBool::new(false),
                ));
                context.parent = dom.pending_source_edit().map(|status| status.operation());
                context.watcher = dom
                    .snapshot()
                    .node(child)
                    .ok()
                    .and_then(|node| node.as_dialog())
                    .and_then(|dialog| dialog.close_watcher());
                Ok(())
            }),
            crate::DomEventListenerOptions::default(),
        )
        .unwrap();
    let serviced = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut context,
            crate::DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: event_limits(),
            },
            &cancel,
        )
        .unwrap();
    let crate::DomSourceEventServiceProgress::Serviced { receipt } = serviced else {
        panic!("real completed dispatch required")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    assert!(
        matches!(context.child,Some(Ok(DomSourceDriveProgress::Complete(ref done))) if !done.cancellation_requested())
    );
    assert_eq!(context.parent, Some(request.operation()));
    assert_eq!(context.calls, 1);
    let watcher = context
        .watcher
        .expect("actual watcher survives the child close");
    assert_eq!(watcher.subject(), child);
    assert_eq!(watcher.global(), top.global());
    assert_eq!(counts(&session.hosted_execution()), (1, 1, 1));
    let mut policy = |_: &DomStyleSource| None;
    let parent_done = session
        .source_execution()
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    assert!(
        matches!(parent_done,DomSourceDriveProgress::Complete(ref done) if !done.cancellation_requested())
    );
    assert_eq!(
        session
            .snapshot()
            .node(child)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher)
    );
    apply(&mut session, DomSourceEdit::remove_attribute(child, "open"));
    assert_eq!(counts(&session.hosted_execution()), (1, 0, 0));
}

#[test]
fn adopted_dialog_uses_original_window_and_old_or_foreign_requests_cannot_remove_it() {
    let (mut session, first, _body, subject) = fixture();
    let cancel = AtomicBool::new(false);
    let mut hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let old_request = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .begin_source_edit(
                DomSourceEdit::set_attribute(subject, "open", ""),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let request = pending(&mut lease);
        assert!(
            hosted
                .service_dialog_watcher(&mut lease, &request, &cancel)
                .unwrap()
        );
        let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        settlement.finish(&lease.snapshot());
        request
    };
    let old_id = session
        .snapshot()
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    let host = owner.borrow().hosts[0].id;
    let second = hosted
        .create_top_level(&mut session.dom(), host, &cancel)
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(second.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    apply(
        &mut session,
        DomSourceEdit::insert_before(subject, body, None),
    );
    let adopted = session.snapshot();
    let new_id = adopted
        .node(subject)
        .unwrap()
        .as_dialog()
        .unwrap()
        .close_watcher()
        .unwrap();
    assert_ne!(new_id, old_id);
    assert_eq!(new_id.global(), first.global());
    assert_eq!(
        adopted.relevant_global(subject).unwrap(),
        Some(first.global())
    );
    assert_eq!(
        adopted.node(subject).unwrap().owner_document(),
        Some(second.document())
    );
    {
        let mut state = owner.borrow_mut();
        let (h, t, c, w) = state
            .find_window(first.global(), &cancel, || Ok(()))
            .unwrap();
        assert_eq!(
            state.hosts[h].top_levels[t]
                .group
                .as_ref()
                .unwrap()
                .clusters[c]
                .agent
                .windows
                .rows[w]
                .groups[0]
                .members[0]
                .borrow()
                .id,
            new_id
        );
        let (h, t, c, w) = state
            .find_window(second.global(), &cancel, || Ok(()))
            .unwrap();
        assert!(
            state.hosts[h].top_levels[t]
                .group
                .as_ref()
                .unwrap()
                .clusters[c]
                .agent
                .windows
                .rows[w]
                .groups
                .is_empty()
        );
    }
    let (foreign, _top, _body, _) = fixture();
    let foreign_hosted = foreign.hosted_execution();
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .begin_source_edit(
            DomSourceEdit::remove_attribute(subject, "open"),
            source_limits(),
            &cancel,
        )
        .unwrap();
    let request = pending(&mut lease);
    assert_eq!(request.action(), HtmlDialogAction::DestroyCloseWatcher);
    let before = lease.snapshot();
    let work = owner.borrow().work;
    assert!(matches!(
        hosted.install_dialog_watcher(&mut lease, &old_request, &cancel),
        Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
            DomError::InvalidSourceOperation
        )))
    ));
    assert_eq!(owner.borrow().work, work);
    assert!(matches!(
        foreign_hosted.install_dialog_watcher(&mut lease, &request, &cancel),
        Err(DomHostedRuntimeError::Binding)
    ));
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert_eq!(counts(&foreign_hosted), (1, 0, 0));
    assert!(
        hosted
            .service_dialog_watcher(&mut lease, &request, &cancel)
            .unwrap()
    );
    let settlement = hosted.prepare_watcher_settlement(&lease, &cancel).unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    settlement.finish(&lease.snapshot());
    assert_eq!(
        lease
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        None
    );
    assert_eq!(counts(&hosted), (1, 0, 0));
}

// Prepare1 + retry6 + Window lookup4 + retained8 + selection2 + custody
// growth1 + membership predicates/moves4 + transition1. Source's independent
// payload-admission producer join is calibrated separately below.
const ONE_MEMBER_REMOVAL_STEPS: u64 = 27;
fn removal_source_boundary(
    dom: &mut Dom,
    hosted: &HostedRuntime,
    request: &DomSourceDialogRequest,
) -> (u64, u64) {
    let owner = hosted.state().unwrap();
    let source = &owner.borrow().source;
    let cancel = AtomicBool::new(false);
    let before_resume = dom.pending_source_edit().unwrap().spent_work();
    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel)
        .unwrap();
    let resume = lease.status().spent_work() - before_resume;
    let _preparation = lease
        .prepare_dialog_watcher(request, source, &cancel)
        .unwrap()
        .unwrap();
    let before = lease.status().spent_work();
    let guard = lease
        .retain_hosted_runtime_payload(source, size_of::<WatcherCustody>(), &cancel)
        .unwrap();
    let mut bank = Vec::<WatcherCustody>::new();
    bank.try_reserve_exact(1).unwrap();
    assert_eq!(bank.capacity(), 1);
    let work = lease.status().spent_work() - before;
    drop(bank);
    drop(guard);
    (lease.status().spent_work() + resume, work)
}
fn work_limit(work: u64) -> DomSourceLimits {
    let limits = source_limits();
    DomSourceLimits::new(
        limits.retained_bytes(),
        limits.retained_string_code_units(),
        work,
        limits.phase_publications(),
        limits.output_records(),
        limits.published_native_identities(),
        limits.nested_operations(),
    )
}
#[test]
fn original_source_removal_work_exact_and_one_short_precede_real_manager_effect() {
    for short in [true, false] {
        let (mut session, _, _, subject) = fixture();
        let cancel = AtomicBool::new(false);
        apply(
            &mut session,
            DomSourceEdit::set_attribute(subject, "open", ""),
        );
        let id = session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        let hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let request = {
            let mut dom = session.dom();
            let request = {
                let mut lease = dom
                    .raw_mut()
                    .begin_source_edit(
                        DomSourceEdit::remove_attribute(subject, "open"),
                        source_limits(),
                        &cancel,
                    )
                    .unwrap();
                pending(&mut lease)
            };
            // Only this original operation's canonical producer is measured;
            // no Runtime traversal or irreversible manager effect has run.
            let (scaffold, retention) = removal_source_boundary(dom.raw_mut(), &hosted, &request);
            let cap = scaffold + retention + ONE_MEMBER_REMOVAL_STEPS - u64::from(short);
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(request.operation(), work_limit(cap), &cancel)
                .unwrap();
            assert_eq!(lease.status().spent_work(), scaffold);
            let before = lease.snapshot();
            let result = hosted.install_dialog_watcher(&mut lease, &request, &cancel);
            if short {
                match result {
                    Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                        DomError::SourceWorkLimit { limit },
                    ))) => assert_eq!(limit, cap),
                    Err(error) => panic!("one-short wrong error: {error:?}"),
                    Ok(_) => panic!("one-short admitted actual manager removal"),
                }
                assert_eq!(lease.status().spent_work(), cap);
                assert_eq!(counts(&hosted), (1, 1, 1));
                let state = owner.borrow();
                let watcher = &state.hosts[0].top_levels[0]
                    .group
                    .as_ref()
                    .unwrap()
                    .clusters[0]
                    .agent
                    .windows
                    .rows[0]
                    .groups[0]
                    .members[0];
                let watcher = watcher.borrow();
                assert_eq!(watcher.phase, WatcherPhase::Registered);
                assert_eq!(watcher.id, id);
                assert!(watcher.publication.is_none());
            } else {
                let location = result.unwrap().expect("exact original work admits removal");
                assert_eq!(lease.status().spent_work(), cap);
                let mut state = owner.borrow_mut();
                let watcher = location.get(&mut state);
                assert_eq!(watcher.phase, WatcherPhase::Removing);
                assert_eq!(watcher.id, id);
                assert!(watcher.publication.is_some());
            }
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            assert_eq!(
                lease
                    .snapshot()
                    .node(subject)
                    .unwrap()
                    .as_dialog()
                    .unwrap()
                    .close_watcher(),
                Some(id)
            );
            request
        };
        // A new permission/lease raises the same cumulative Source policy.
        // The exact case retries the already-real removal publication; the
        // one-short case performs that removal for the first time.
        let replacement = session.hosted_execution();
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(request.operation(), source_limits(), &cancel)
            .unwrap();
        let location = replacement
            .install_dialog_watcher(&mut lease, &request, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(location.get(&mut owner.borrow_mut()).id, id);
        assert_eq!(
            location.get(&mut owner.borrow_mut()).phase,
            WatcherPhase::Removing
        );
        replacement
            .accept_watcher(&mut lease, &request, &location, &cancel)
            .unwrap();
        let settlement = replacement
            .prepare_watcher_settlement(&lease, &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        settlement.finish(&lease.snapshot());
        assert_eq!(counts(&replacement), (1, 0, 0));
        assert!(
            !lease
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
    }
}

fn is_parent_work_refusal(error: &(dyn std::error::Error + 'static), cap: u64) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(DomError::SourceWorkLimit { limit }) = error.downcast_ref::<DomError>() {
            assert_eq!(*limit, cap);
            return true;
        }
        current = error.source();
    }
    false
}

struct AncestorRemoval {
    hosted: HostedRuntime,
    service: bool,
    calls: usize,
    request: Option<DomSourceDialogRequest>,
    setup_error: Option<DomError>,
    unexpected: Option<String>,
    result: Option<Result<Option<WatcherLocation>, DomHostedRuntimeError>>,
}
fn ancestor_removal_attempt(
    cap: u64,
    service: bool,
) -> (DomSession, Option<DomSourceEventRequest>, AncestorRemoval) {
    let (mut session, top, body, parent) = fixture();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let child = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&child, body, None);
    let child = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&child)
        .unwrap()
        .unwrap();
    apply(
        &mut session,
        DomSourceEdit::set_attribute(child, "open", ""),
    );
    let mut context = AncestorRemoval {
        hosted: session.hosted_execution(),
        service,
        calls: 0,
        request: None,
        setup_error: None,
        unexpected: None,
        result: None,
    };
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &mut session.dom(),
            parent,
            crate::DomEventType::BeforeToggle,
            crate::DomEventCallback::new(move |dom, _, context: &mut AncestorRemoval, event| {
                context.calls += 1;
                event.prevent_default();
                let mut lease = match dom.raw_mut().begin_source_edit(
                    DomSourceEdit::remove_attribute(child, "open"),
                    source_limits(),
                    &AtomicBool::new(false),
                ) {
                    Ok(lease) => lease,
                    Err(error) => {
                        context.setup_error = Some(error);
                        return Ok(());
                    }
                };
                match lease.advance(&AtomicBool::new(false)) {
                    Ok(DomSourceProgress::Request(DomSourceRequest::Dialog(request))) => {
                        if context.service {
                            context.result = Some(context.hosted.install_dialog_watcher(
                                &mut lease,
                                &request,
                                &AtomicBool::new(false),
                            ));
                        }
                        context.request = Some(request);
                    }
                    Err(error) => context.setup_error = Some(error),
                    Ok(other) => context.unexpected = Some(format!("{other:?}")),
                }
                Ok(())
            }),
            crate::DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut policy = |_: &DomStyleSource| None;
    let initial = session.source_execution().apply(
        &mut session.dom(),
        DomSourceEdit::show_dialog(parent),
        work_limit(cap),
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut policy,
            url: None,
        },
        &cancel,
    );
    let outer = match initial {
        Ok(DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(outer),
            ..
        }) => Some(outer),
        Err(crate::DomSourceDriveError::Source(crate::DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit },
        ))) if limit == cap => None,
        other => panic!("only original parent Source work may bound setup: {other:?}"),
    };
    if let Some(outer) = &outer {
        // A retained child prevents outer sealing. This helper intentionally
        // observes only the authenticated child Hook reached inside dispatch.
        let outcome = session.source_execution().service_event(
            &mut session.dom(),
            outer,
            &mut listeners,
            &mut context,
            crate::DomSourceEventServiceLimits {
                source: work_limit(cap),
                dispatch: event_limits(),
            },
            &cancel,
        );
        if let Err(error) = &outcome {
            assert!(
                is_parent_work_refusal(error, cap),
                "unexpected original parent dispatch error: {error:?}"
            );
        }
        if context.request.is_none() {
            if let Some(error) = &context.setup_error {
                assert!(
                    matches!(error, DomError::SourceWorkLimit { limit } if *limit == cap),
                    "wrong child setup error: {error:?}"
                );
            } else {
                assert!(outcome.is_err(), "wrong pre-callback outcome: {outcome:?}");
            }
        }
    }
    assert!(
        context.unexpected.is_none(),
        "unexpected child phase {:?}",
        context.unexpected
    );
    (session, outer, context)
}
#[test]
fn retained_parent_removal_preserves_child_exact_work_cut_and_real_effect() {
    for short in [true, false] {
        let (mut session, outer, mut context) =
            ancestor_removal_attempt(source_limits().work(), false);
        let outer = outer.expect("real retained observing parent");
        let request = context.request.take().expect("actual child Destroy Hook");
        assert_eq!(request.action(), HtmlDialogAction::DestroyCloseWatcher);
        // A retained parent's policy is monotonic beneath its child. The
        // top child's original public resume owns this exact lowerable cut.
        let cap = {
            let mut dom = session.dom();
            let (scaffold, retention) =
                removal_source_boundary(dom.raw_mut(), &context.hosted, &request);
            let cap = scaffold + retention + ONE_MEMBER_REMOVAL_STEPS - u64::from(short);
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(
                    request.operation(),
                    work_limit(cap),
                    &AtomicBool::new(false),
                )
                .unwrap();
            assert_eq!(lease.status().spent_work(), scaffold);
            let basis = lease.snapshot().stamp();
            context.result = Some(context.hosted.install_dialog_watcher(
                &mut lease,
                &request,
                &AtomicBool::new(false),
            ));
            assert_eq!(lease.status().spent_work(), cap);
            assert_eq!(lease.snapshot().stamp(), basis);
            cap
        };
        let status = session.dom().pending_source_edit().unwrap();
        assert_eq!(status.operation(), request.operation());
        assert_eq!(status.parent_operation(), Some(outer.operation()));
        assert!(
            status.spent_work() < source_limits().work(),
            "child's independent allowance remains generous"
        );
        let id = session
            .snapshot()
            .node(request.subject())
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        let owner = context.hosted.state().unwrap();
        if short {
            match context.result.take().expect("actual reached service") {
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::SourceWorkLimit { limit },
                ))) => assert_eq!(limit, cap),
                Err(error) => {
                    panic!("wrong child service refusal under retained parent: {error:?}")
                }
                Ok(_) => panic!("one-short child admitted actual manager removal"),
            }
            assert_eq!(
                owner.borrow().hosts[0].top_levels[0]
                    .group
                    .as_ref()
                    .unwrap()
                    .clusters[0]
                    .agent
                    .windows
                    .rows[0]
                    .groups[0]
                    .members[0]
                    .borrow()
                    .phase,
                WatcherPhase::Registered
            );
        } else {
            let location = context
                .result
                .take()
                .expect("actual reached service")
                .unwrap()
                .unwrap();
            assert_eq!(
                location.get(&mut owner.borrow_mut()).phase,
                WatcherPhase::Removing
            );
            assert!(location.get(&mut owner.borrow_mut()).publication.is_some());
        }
        assert_eq!(context.calls, 1);
        session
            .source_execution()
            .raise_event_limits(
                &mut session.dom(),
                &outer,
                source_limits(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let hosted = session.hosted_execution();
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(
                    request.operation(),
                    source_limits(),
                    &AtomicBool::new(false),
                )
                .unwrap();
            let location = hosted
                .install_dialog_watcher(&mut lease, &request, &AtomicBool::new(false))
                .unwrap()
                .unwrap();
            assert_eq!(location.get(&mut owner.borrow_mut()).id, id);
            hosted
                .accept_watcher(&mut lease, &request, &location, &AtomicBool::new(false))
                .unwrap();
            let settlement = hosted
                .prepare_watcher_settlement(&lease, &AtomicBool::new(false))
                .unwrap();
            assert!(matches!(
                lease.advance(&AtomicBool::new(false)).unwrap(),
                DomSourceProgress::Complete(_)
            ));
            settlement.finish(&lease.snapshot());
        }
        assert_eq!(
            session.dom().pending_source_edit().unwrap().operation(),
            outer.operation()
        );
        assert_eq!(counts(&hosted), (1, 0, 0));
        assert!(
            !session
                .snapshot()
                .node(request.subject())
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
        assert_eq!(context.calls, 1);
    }
}

#[test]
fn finite_parent_removal_work_refusals_name_the_actual_parent_budget() {
    // These fresh owners characterize actual finite policy outcomes; they do
    // not transfer a canonical setup threshold or claim a late one-short cut.
    for cap in [0, 64, 1024, 16_384] {
        let (mut session, outer, mut context) = ancestor_removal_attempt(cap, true);
        let Some(request) = context.request.take() else {
            assert_ne!(
                cap, 16_384,
                "generous finite parent must reach child removal"
            );
            eprintln!("watcher finite parent cap={cap}: setup refused");
            continue;
        };
        assert_ne!(cap, 0, "zero parent work must refuse");
        let outer = outer.expect("actual retained observing parent");
        let status = session.dom().pending_source_edit().unwrap();
        assert_eq!(status.operation(), request.operation());
        assert_eq!(status.parent_operation(), Some(outer.operation()));
        assert_eq!(context.calls, 1);
        let id = session
            .snapshot()
            .node(request.subject())
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher()
            .unwrap();
        let owner = context.hosted.state().unwrap();
        match context.result.take().expect("actual reached service") {
            Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit {
                limit,
            }))) => {
                assert_eq!(limit, cap);
                assert_ne!(cap, 16_384, "generous finite parent must service removal");
                assert_eq!(counts(&context.hosted), (1, 1, 1));
                let state = owner.borrow();
                let record = state.hosts[0].top_levels[0]
                    .group
                    .as_ref()
                    .unwrap()
                    .clusters[0]
                    .agent
                    .windows
                    .rows[0]
                    .groups[0]
                    .members[0]
                    .borrow();
                assert_eq!(record.id, id);
                assert_eq!(record.phase, WatcherPhase::Registered);
                assert!(record.publication.is_none());
                eprintln!("watcher finite parent cap={cap}: removal service refused");
            }
            Err(error) => panic!("wrong finite-parent watcher refusal: {error:?}"),
            Ok(location) => {
                let location = location.expect("real manager result");
                let mut state = owner.borrow_mut();
                let record = location.get(&mut state);
                assert_eq!(record.id, id);
                assert_eq!(record.phase, WatcherPhase::Removing);
                assert!(record.publication.is_some());
                eprintln!("watcher finite parent cap={cap}: actual removal retained");
            }
        }
    }
}

fn second_registration_fixture() -> (DomSession, NodeId) {
    let (mut session, top, body, first) = fixture();
    apply(
        &mut session,
        DomSourceEdit::set_attribute(first, "open", ""),
    );
    let mut batch = DomBatch::new();
    let second = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&second, body, None);
    let second = session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&second)
        .unwrap()
        .unwrap();
    (session, second)
}
fn registration_source_boundary(
    dom: &mut Dom,
    hosted: &HostedRuntime,
    request: &DomSourceDialogRequest,
) -> (u64, u64, u64, u64, u64) {
    let owner = hosted.state().unwrap();
    let source = &owner.borrow().source;
    let cancel = AtomicBool::new(false);
    let before_resume = dom.pending_source_edit().unwrap().spent_work();
    let mut lease = dom
        .resume_source_edit(request.operation(), source_limits(), &cancel)
        .unwrap();
    let scaffold = lease.status().spent_work();
    let resume = scaffold - before_resume;
    let _preparation = lease
        .prepare_dialog_watcher(request, source, &cancel)
        .unwrap()
        .unwrap();
    let prepared = lease.status().spent_work();
    // Calibrate the existing canonical producer joins, never a Runtime bank
    // traversal/move. These guards own the actual candidate header and full
    // replacement capacity through their physical temporary values.
    let candidate_guard = lease
        .retain_hosted_runtime_payload(source, size_of::<WatcherGroup>(), &cancel)
        .unwrap();
    let candidate = WatcherGroup {
        members: Vec::new(),
        payload: None,
    };
    let candidate_work = lease.status().spent_work();
    let replacement_guard = lease
        .retain_hosted_runtime_payload(source, 2 * size_of::<Rc<RefCell<Watcher>>>(), &cancel)
        .unwrap();
    let mut replacement = Vec::<Rc<RefCell<Watcher>>>::new();
    replacement.try_reserve_exact(2).unwrap();
    assert_eq!(replacement.capacity(), 2);
    let replacement_work = lease.status().spent_work();
    let body_guard = lease
        .retain_hosted_runtime_payload(source, watcher_body_bytes(), &cancel)
        .unwrap();
    let mut body = Vec::<u8>::new();
    body.try_reserve_exact(watcher_body_bytes()).unwrap();
    assert_eq!(body.capacity(), watcher_body_bytes());
    let body_work = lease.status().spent_work();
    drop(body);
    drop(body_guard);
    drop(replacement);
    drop(replacement_guard);
    drop(candidate);
    drop(candidate_guard);
    (
        body_work + resume,
        prepared - scaffold,
        candidate_work - prepared,
        replacement_work - candidate_work,
        body_work - replacement_work,
    )
}
#[test]
fn source_member_growth_precharge_one_short_preserves_actual_old_bank_and_exact_replaces_it() {
    for short in [true, false] {
        let (mut session, second) = second_registration_fixture();
        let cancel = AtomicBool::new(false);
        let hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let first = owner.borrow().hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent
            .windows
            .rows[0]
            .groups[0]
            .members[0]
            .borrow()
            .id;
        let mut dom = session.dom();
        let request = {
            let mut lease = dom
                .raw_mut()
                .begin_source_edit(
                    DomSourceEdit::set_attribute(second, "open", ""),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            pending(&mut lease)
        };
        let (scaffold, prepare, candidate, replacement, body) =
            registration_source_boundary(dom.raw_mut(), &hosted, &request);
        // Six retry rows, four Window lookup rows and eight retained visits
        // (including the newly allocated original record body).
        // This full one-member bank then needs entry1 + actual member move1.
        // Short leaves only1 for that2-step movement precharge, before the
        // replacement allocation/admission. Exact also admits the independent
        // producer replacement join and final registration transition1.
        let cap = if short {
            scaffold + prepare + candidate + 18 + 1
        } else {
            scaffold + prepare + candidate + replacement + body + 18 + 2 + 1
        };
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(request.operation(), work_limit(cap), &cancel)
            .unwrap();
        assert_eq!(lease.status().spent_work(), scaffold);
        let result = hosted.install_dialog_watcher(&mut lease, &request, &cancel);
        if short {
            match result {
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::SourceWorkLimit { limit },
                ))) => assert_eq!(limit, cap),
                Err(error) => panic!("wrong growth precharge refusal: {error:?}"),
                Ok(_) => panic!("one-short movement work admitted actual replacement"),
            }
            let state = owner.borrow();
            let members = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .groups[0]
                .members;
            assert_eq!(members.len(), 1);
            assert_eq!(members.capacity(), 1);
            assert_eq!(members[0].borrow().id, first);
            assert_eq!(members[0].borrow().phase, WatcherPhase::Registered);
        } else {
            let location = result.unwrap().unwrap();
            let mut state = owner.borrow_mut();
            let members = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent
                .windows
                .rows[0]
                .groups[0]
                .members;
            assert_eq!(members.len(), 2);
            assert_eq!(members.capacity(), 2);
            assert_eq!(members[0].borrow().id, first);
            assert_eq!(location.get(&mut state).phase, WatcherPhase::Registering);
            assert!(location.get(&mut state).publication.is_some());
        }
        assert_eq!(lease.status().spent_work(), cap);
        assert!(
            !lease
                .snapshot()
                .node(second)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
    }
}
