//! Real Details delivery abandoned after callback entry.
use super::*;

const SOURCE: surgeist_dom::DomElementTaskSourceKind =
    surgeist_dom::DomElementTaskSourceKind::DomManipulation;

fn run(
    fixture: &mut DetailsDispatchDropFixture,
) -> Result<DomElementTaskProgress, DomElementTaskRuntimeError> {
    DomElementTaskRuntime::run_next(
        &mut fixture.session.dom(),
        SOURCE,
        &mut fixture.dispatcher,
        &mut fixture.context,
        details_run_limits(),
        &cancel(),
    )
}

fn started_details() -> (DetailsDispatchDropFixture, DomElementTask) {
    let mut fixture = details_dispatch_drop_fixture(true);
    let original = fixture
        .context
        .held
        .owner
        .pending_task(SOURCE, 0)
        .unwrap()
        .unwrap();
    assert!(matches!(
        run(&mut fixture),
        Err(DomElementTaskRuntimeError::Dispatch(
            DomEventDispatchError::Abandoned
        ))
    ));
    assert_eq!(fixture.context.calls, ["prefix"]);
    assert_eq!(
        fixture.context.held.owner.pending_task(SOURCE, 0).unwrap(),
        Some(original)
    );
    (fixture, original)
}

#[test]
fn destroyed_target_does_not_consume_started_abandoned_details_task() {
    let (mut fixture, original) = started_details();
    let target = fixture.target;
    let mut batch = DomBatch::new();
    batch.detach(target);
    batch.destroy(target);
    apply(&mut fixture.session, batch);
    assert!(matches!(
        fixture.session.snapshot().node(target),
        Err(DomError::StaleNode)
    ));
    let stamp = fixture.session.snapshot().stamp();
    let retry = run(&mut fixture);
    let pending = fixture.context.held.owner.pending_task(SOURCE, 0).unwrap();
    assert_eq!(fixture.context.calls, ["prefix"]);
    assert_eq!(fixture.session.snapshot().stamp(), stamp);
    assert!(
        matches!(
            retry,
            Err(DomElementTaskRuntimeError::Dispatch(
                DomEventDispatchError::Abandoned
            ))
        ),
        "the started task must retain its abandonment after target retirement, got {retry:?} with pending {pending:?}"
    );
    assert_eq!(pending, Some(original));
    let state = fixture.context.held.owner.state().unwrap();
    assert!(state.borrow().tasks[0].receipt.is_none());
}

#[test]
fn later_details_toggle_does_not_coalesce_away_started_abandoned_task() {
    let (mut fixture, original) = started_details();
    let target = fixture.target;
    close(&mut fixture.session, target);
    let owner = &mut fixture.context.held.owner;
    owner.ingest(&mut fixture.session.dom(), &cancel()).unwrap();
    let first = owner.pending_task(SOURCE, 0).unwrap().unwrap();
    let second = owner.pending_task(SOURCE, 1).unwrap();
    assert_eq!(fixture.context.calls, ["prefix"]);
    assert_eq!(
        first.id(),
        original.id(),
        "the partially delivered row stays at the head"
    );
    let successor = second.expect("a later real Details toggle was queued independently");
    assert_ne!(successor.id(), original.id());
    assert_eq!(successor.id().target(), target);
    assert!(matches!(
        run(&mut fixture),
        Err(DomElementTaskRuntimeError::Dispatch(
            DomEventDispatchError::Abandoned
        ))
    ));
    assert_eq!(fixture.context.calls, ["prefix"]);
    assert_eq!(
        fixture.context.held.owner.pending_task(SOURCE, 1).unwrap(),
        Some(successor)
    );
}
