//! Private producer-consumer handoff ordering, not a claimed public schedule.
//! Authentic canonical parent permissions remain caller-owned until after real
//! Runtime child cleanup; no queued row, phase or completion proof is fabricated.
use super::*;

fn late_handoff(parent_captures: usize) {
    assert!(matches!(parent_captures, 1 | 2));
    let cancel = AtomicBool::new(false);
    let (mut session, mut hosted, top, body, iframe) = fixture();
    edit(
        &mut session,
        DomSourceEdit::insert_before(iframe, body, None),
    );
    assert_eq!(drain(&mut session, &mut hosted, top).unwrap(), 0);
    let initial = session.snapshot();
    let child = initial
        .node(iframe)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let nav = initial.hosted_navigable(child).unwrap();
    let child_document = nav.active_document().unwrap();
    let child_global = nav.global();
    let owner = hosted.state().unwrap();

    // A real producer capability may be retained before its consumer handoff.
    // Capture while the child is still live. The original canonical operation
    // owns its captured list, Work and reserved obligations throughout this
    // delay; Runtime has not yet received any of these parent permissions.
    let mut held = Vec::new();
    let mut parent_operations = Vec::new();
    {
        let mut state = owner.borrow_mut();
        state.reserve_invocation(&cancel).unwrap();
        let mut dom = session.dom();
        for _ in 0..parent_captures {
            let permission = dom
                .raw_mut()
                .begin_hosted_top_level_destruction(&mut state.source, top.navigable(), &cancel)
                .unwrap();
            let request = permission.request(dom.raw()).unwrap();
            assert_eq!(request.document(), top.document());
            assert_eq!(request.global(), top.global());
            assert_eq!(request.child_documents().len(), 1);
            assert_eq!(request.child_documents()[0].document(), child_document);
            assert_eq!(request.child_documents()[0].global(), child_global);
            assert!(!parent_operations.contains(&permission.operation()));
            parent_operations.push(permission.operation());
            held.push(permission);
        }
        assert!(state.destruction.invocations.is_empty());
        assert!(state.destruction.cleanups.is_empty());
    }

    // Use actual public Source removal and the normal Runtime task pump. This
    // does not service or register any of the privately held parent waiters.
    edit(&mut session, DomSourceEdit::detach(iframe));
    let mut executed = Vec::new();
    let mut child_idle = false;
    for _ in 0..1024 {
        match hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap()
        {
            DomHostedTaskProgress::Executed {
                operation,
                document,
                global,
                kind: DomHostedTaskKind::Destruction,
            } => executed.push((operation, document, global)),
            DomHostedTaskProgress::Idle => {
                child_idle = true;
                break;
            }
            _ => {}
        }
    }
    assert!(
        child_idle,
        "actual child cleanup must complete before handoff"
    );
    assert_eq!(executed.len(), 1);
    let (child_cleanup, destroyed_document, destroyed_global) = executed[0];
    assert_eq!(destroyed_document, child_document);
    assert_eq!(destroyed_global, child_global);
    let after_child = session.snapshot();
    assert!(
        !after_child
            .document(child_document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        after_child
            .hosted_navigable(child)
            .unwrap()
            .active_document(),
        None
    );
    assert_eq!(
        after_child
            .node(iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    let work_before_handoff = {
        let mut state = owner.borrow_mut();
        assert!(state.destruction.invocations.is_empty());
        assert!(state.destruction.cleanups.is_empty());
        assert_eq!(
            state.find_global(child_global, &cancel, || Ok(())),
            Err(DomHostedRuntimeError::TargetNotFound)
        );
        let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
        assert!(group.traversal.rows.is_empty());
        state.work
    };

    // Hand over those SAME original permissions only now. This is not removal
    // or restoration of Runtime rows: no parent row existed before this point.
    // Original reserve/retain/registration methods own all actual new custody.
    {
        let mut state = owner.borrow_mut();
        let mut dom = session.dom();
        for permission in held {
            let request = permission.request(dom.raw()).unwrap();
            assert_eq!(request.child_documents().len(), 1);
            assert_eq!(request.child_documents()[0].document(), child_document);
            assert_eq!(request.child_documents()[0].global(), child_global);
            state.reserve_invocation(&cancel).unwrap();
            state.retain_invocation(permission);
        }
        state
            .register_descendants(&mut dom, top.navigable(), &cancel)
            .unwrap();
        let HostedState {
            source,
            destruction,
            ..
        } = &mut *state;
        let mut late_children = 0;
        for row in &mut destruction.invocations {
            if !row.info.is_some_and(|info| info.document == child_document) {
                continue;
            }
            late_children += 1;
            let info = row.info.unwrap();
            assert_eq!(info.global, child_global);
            assert_eq!(
                info.cleanup, child_cleanup,
                "late waiter must retain the original successful cleanup identity"
            );
            assert!(row.parent.is_some_and(|id| parent_operations.contains(&id)));
            // The original canonical producer supplies actual terminal proof
            // before any attempt to use the now-destroyed Global or task loop.
            let cleanup = row
                .permission
                .cleanup(dom.raw_mut(), source, &cancel)
                .unwrap();
            let proof = cleanup.destruction_proof().unwrap();
            assert_eq!(proof.document(), child_document);
            assert_eq!(proof.global(), child_global);
        }
        assert_eq!(late_children, parent_captures);
        assert!(state.work >= work_before_handoff);
    }

    let mut completed_parents = Vec::new();
    let mut settled = false;
    for _ in 0..1024 {
        let progress = hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap();
        match progress {
            DomHostedTaskProgress::Executed {
                operation,
                document,
                global,
                kind: DomHostedTaskKind::Destruction,
            } => executed.push((operation, document, global)),
            DomHostedTaskProgress::Complete { operation }
                if parent_operations.contains(&operation) =>
            {
                assert!(!completed_parents.contains(&operation));
                completed_parents.push(operation);
            }
            _ => {}
        }
        let state = owner.borrow();
        if completed_parents.len() == parent_captures
            && state.destruction.invocations.is_empty()
            && state.destruction.cleanups.is_empty()
        {
            assert!(state.hosts[0].top_levels.is_empty());
            assert!(state.work >= work_before_handoff);
            settled = true;
            break;
        }
    }
    assert!(
        settled,
        "late original waiters must finish within the finite pump bound"
    );
    assert_eq!(
        executed
            .iter()
            .filter(|(_, document, _)| *document == child_document)
            .count(),
        1
    );
    assert_eq!(
        executed
            .iter()
            .filter(|(_, document, _)| *document == top.document())
            .count(),
        1
    );
    assert_eq!(
        executed.len(),
        2,
        "only the real child and parent destructor bodies execute"
    );
    assert_eq!(
        after_child.document(child_document).unwrap().navigable(),
        None
    );
    assert_eq!(
        initial.hosted_navigable(child).unwrap().active_document(),
        Some(child_document)
    );
}

#[test]
fn actual_completed_child_satisfies_late_parent_handoff_without_old_global_or_second_cleanup() {
    late_handoff(1);
}

#[test]
fn actual_completed_child_satisfies_two_independent_late_parent_captures_once() {
    late_handoff(2);
}
