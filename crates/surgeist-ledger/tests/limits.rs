use surgeist_ledger::{Error, Limits, Operation, Tariffs};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visits, Credits }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes, Entries }
}

fn caps(visits: u64, credits: u64, bytes: u64, entries: u64) -> Limits<Work, Resource> {
    Limits::new(
        WorkTable {
            Visits: visits,
            Credits: credits,
        },
        ResourceTable {
            Bytes: bytes,
            Entries: entries,
        },
    )
}

#[test]
fn a_scope_reports_its_own_caps_and_not_those_of_an_ancestor() {
    let mut operation = Operation::new("root", caps(10, 20, 30, 40), 2).unwrap();
    let root = operation.root();
    let child = operation
        .child(root, "child", caps(1, 2, 3, 4), Tariffs::identity())
        .unwrap();
    assert_eq!(*operation.limits(root).unwrap(), caps(10, 20, 30, 40));
    assert_eq!(*operation.limits(child).unwrap(), caps(1, 2, 3, 4));
    let limits = operation.limits(child).unwrap();
    assert_eq!(limits.work(Work::Credits), 2);
    assert_eq!(limits.live(Resource::Entries), 4);
}

#[test]
fn an_unlimited_scope_reports_the_maximum_for_every_dimension() {
    let operation = Operation::<Work, Resource>::new("root", Limits::unlimited(), 1).unwrap();
    let limits = operation.limits(operation.root()).unwrap();
    for work in [Work::Visits, Work::Credits] {
        assert_eq!(limits.work(work), u64::MAX);
    }
    for resource in [Resource::Bytes, Resource::Entries] {
        assert_eq!(limits.live(resource), u64::MAX);
    }
}

#[test]
fn a_phase_and_a_shared_participant_report_the_same_caps() {
    let mut operation = Operation::new("root", caps(10, 20, 30, 40), 1).unwrap();
    let root = operation.root();
    let in_phase = operation
        .with_phase(root, |phase| *phase.limits(root).unwrap())
        .unwrap();
    assert_eq!(in_phase, caps(10, 20, 30, 40));
    let shared = operation.share();
    assert_eq!(shared.limits(root), Ok(caps(10, 20, 30, 40)));
}

#[test]
fn a_scope_of_another_operation_is_refused_as_foreign() {
    let operation = Operation::<Work, Resource>::new("one", Limits::unlimited(), 1).unwrap();
    let other = Operation::<Work, Resource>::new("two", Limits::unlimited(), 1).unwrap();
    assert!(matches!(
        operation.limits(other.root()),
        Err(Error::ForeignScope)
    ));
}
