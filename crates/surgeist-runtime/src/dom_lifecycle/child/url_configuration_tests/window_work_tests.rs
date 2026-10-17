//! Actual first Frame NativeExecution preflight, before borrowed execution.
use super::*;
use crate::dom_lifecycle::watcher::NativeWindow;
use surgeist_dom::{DomSourceLease, DomSourceProgress, NodeId};

fn frame_fixture() -> (DomSession, HostedRuntime, NodeId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut session = fixture();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel).unwrap()
    };
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(top.document(), None);
    let frame = batch.create_element_ns(&closed, Namespace::html(), "frame");
    let frame = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&frame)
        .unwrap()
        .unwrap();
    (session, hosted, frame, body)
}
fn reach_native_execution(
    lease: &mut DomSourceLease<'_>,
    hosted: &HostedRuntime,
) -> DomHostedChildRequest {
    let cancel = AtomicBool::new(false);
    let child = match lease.advance(&cancel).unwrap() {
        DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) => child,
        other => panic!("real Frame child hook required: {other:?}"),
    };
    loop {
        lease
            .process_hosted_child_attributes(&child, &cancel)
            .unwrap();
        match lease.hosted_child_step(&child).unwrap() {
            DomHostedChildStep::Prepare(_) => {
                hosted.prepare_source_child(lease, &child, &cancel).unwrap()
            }
            DomHostedChildStep::PermissionsPolicy(request) => {
                let policy::Decision::Complete(value) =
                    hosted.policy_decision(request, None, &cancel).unwrap()
                else {
                    panic!("actual absent-src Frame policy needs no URL provider");
                };
                hosted
                    .seal_child_policy(&child, request, value, &cancel)
                    .unwrap();
                hosted
                    .accept_child_publication(lease, &child, true, &cancel)
                    .unwrap();
            }
            DomHostedChildStep::NativeExecution(request) => {
                assert_eq!(request.parent_global(), Some(child.parent_global()));
                return child;
            }
            _ => panic!("unexpected phase before authentic native execution"),
        }
    }
}
fn scaffold_and_canonical_replacement_work() -> (u64, u64) {
    let (mut session, hosted, frame, body) = frame_fixture();
    let cancel = AtomicBool::new(false);
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .begin_source_edit(
            DomSourceEdit::insert_before(frame, body, None),
            source_limits(20_000_000),
            &cancel,
        )
        .unwrap();
    let _child = reach_native_execution(&mut lease, &hosted);
    let scaffold = lease.status().spent_work();
    let owner = hosted.state().unwrap();
    let state = owner.borrow();
    let agent = &state.hosts[0].top_levels[0]
        .group
        .as_ref()
        .unwrap()
        .clusters[0]
        .agent;
    assert_eq!(agent.globals.len(), 1);
    assert_eq!(agent.windows.rows.len(), 1);
    assert_eq!(agent.windows.rows.capacity(), 1);
    // Calibrate the existing canonical producer admission only, with the
    // actual full candidate capacity alive under its original relative guard.
    // No Runtime lookup, retained traversal, Window move or execution is run.
    let guard = lease
        .retain_hosted_runtime_payload(&state.source, 2 * size_of::<NativeWindow>(), &cancel)
        .unwrap();
    let mut replacement = Vec::<NativeWindow>::new();
    replacement.try_reserve_exact(2).unwrap();
    assert_eq!(replacement.capacity(), 2);
    let producer = lease.status().spent_work() - scaffold;
    drop(replacement);
    drop(guard);
    (scaffold, producer)
}
#[test]
fn first_frame_window_preflight_source_work_exact_and_one_short_preserve_real_native_execution() {
    let (scaffold, producer) = scaffold_and_canonical_replacement_work();
    // New preflight: parent Global lookup4; current child lookup1; retained
    // host/composition/cluster/Window plus two Settings fields6; origin match1;
    // Window growth entry1 and the one actual old-Window move1 =14.
    const RUNTIME_STEPS: u64 = 14;
    for short in [true, false] {
        let (mut session, hosted, frame, body) = frame_fixture();
        let cancel = AtomicBool::new(false);
        let cap = scaffold + producer + RUNTIME_STEPS - u64::from(short);
        let owner = hosted.state().unwrap();
        let request = {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .begin_source_edit(
                    DomSourceEdit::insert_before(frame, body, None),
                    source_limits(cap),
                    &cancel,
                )
                .unwrap();
            let child = reach_native_execution(&mut lease, &hosted);
            assert_eq!(lease.status().spent_work(), scaffold);
            let before = lease.snapshot();
            let identity = match lease.hosted_child_step(&child).unwrap() {
                DomHostedChildStep::NativeExecution(request) => (
                    request.operation(),
                    request.browsing_context(),
                    request.parent_global(),
                ),
                _ => panic!("original NativeExecution must remain current"),
            };
            let result = hosted.prepare_child_window(&mut lease, &child, &cancel);
            if short {
                match result {
                    Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                        DomError::SourceWorkLimit { limit },
                    ))) => assert_eq!(limit, cap),
                    Err(error) => panic!("one-short wrong preflight error: {error:?}"),
                    Ok(()) => panic!("one-short Source work admitted actual Window replacement"),
                }
            } else {
                result.unwrap();
            }
            assert_eq!(lease.status().spent_work(), cap);
            let state = owner.borrow();
            let agent = &state.hosts[0].top_levels[0]
                .group
                .as_ref()
                .unwrap()
                .clusters[0]
                .agent;
            assert_eq!(agent.windows.rows.len(), 1);
            assert_eq!(agent.windows.rows.capacity(), if short { 1 } else { 2 });
            assert_eq!(agent.globals.len(), 1);
            assert!(state.children[0].global.is_none());
            assert!(state.children[0].publication.is_none());
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            match lease.hosted_child_step(&child).unwrap() {
                DomHostedChildStep::NativeExecution(request) => {
                    assert_eq!(
                        (
                            request.operation(),
                            request.browsing_context(),
                            request.parent_global()
                        ),
                        identity
                    )
                }
                _ => panic!("refusal/preparation must preserve the original execution request"),
            }
            child
        };
        // Reacquire the same canonical phase/issued identity with raised
        // cumulative work, then run the ordinary service to its real result.
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(request.operation(), source_limits(20_000_000), &cancel)
            .unwrap();
        assert!(matches!(
            hosted
                .service_source_child(&mut lease, &request, None, &cancel)
                .unwrap(),
            SourceProgress::Serviced
        ));
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        let state = owner.borrow();
        let agent = &state.hosts[0].top_levels[0]
            .group
            .as_ref()
            .unwrap()
            .clusters[0]
            .agent;
        assert_eq!(agent.windows.rows.len(), 2);
        assert_eq!(agent.globals.len(), 2);
        assert!(
            lease
                .snapshot()
                .node(frame)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable()
                .unwrap()
                .is_some()
        );
    }
}
