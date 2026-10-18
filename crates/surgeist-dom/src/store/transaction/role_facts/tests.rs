//! New private index characterizations; original controller REDs live with
//! Style, Dialog, autofocus and fullscreen, rather than this new API.
use super::*;

fn keys() -> (Key, Key, Key) {
    let owner = DomOwnerId::allocate().unwrap();
    let key = |slot, generation| {
        Key::Existing(NodeId {
            owner,
            slot,
            generation,
        })
    };
    (key(2, 1), key(9, u64::MAX), key(9, 1))
}
fn registry() -> (custody::Ledger, bank::CustodyRegistry) {
    let ledger = custody::Ledger::default();
    let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
    (ledger, registry)
}
fn set(
    version: &mut Version,
    role: Role,
    owner: Key,
    member: Key,
    count: usize,
    registry: &bank::CustodyRegistry,
) {
    version
        .set(
            role,
            owner,
            member,
            count,
            &mut Editing {
                registry,
                admit: &mut |_| Ok(()),
                step: &mut || Ok(()),
                changed: false,
            },
        )
        .unwrap();
}

#[test]
fn roles_preserve_occurrences_generations_and_their_independent_reverse_edges() {
    let (ledger, registry) = registry();
    let (document, member, reused) = keys();
    let baseline = ledger.total();
    let mut version = Version::default();
    set(&mut version, Role::TopLayer, document, member, 2, &registry);
    set(
        &mut version,
        Role::PendingTopLayerRemovals,
        document,
        member,
        1,
        &registry,
    );
    set(
        &mut version,
        Role::AutofocusCandidates,
        document,
        reused,
        3,
        &registry,
    );
    assert_eq!(version.value.count(Role::TopLayer, document, member), 2);
    assert_eq!(
        version
            .value
            .count(Role::PendingTopLayerRemovals, document, member),
        1
    );
    assert_eq!(
        version
            .value
            .count(Role::AutofocusCandidates, document, member),
        0
    );
    assert_eq!(version.value.count(Role::TopLayer, document, reused), 0);
    let rows = version.value.inspect_relations();
    assert_eq!(rows.len(), 3);
    assert!(rows.contains(&(Role::TopLayer, document, member, 2)));
    assert!(rows.contains(&(Role::PendingTopLayerRemovals, document, member, 1)));
    assert!(rows.contains(&(Role::AutofocusCandidates, document, reused, 3)));
    for (role, owner, target, count) in rows {
        assert_eq!(
            version.value.reverse[role.index()].count(target, owner),
            count
        );
    }
    let graph = version.value.inspect_graph().unwrap();
    assert_eq!(
        version.value.retained_usage(),
        custody::Usage::new(graph.index_charge().bytes + graph.record_totals().bytes, 0)
    );
    let before = version.clone();
    let mut roles = Roles {
        current: version,
        ..Roles::default()
    };
    let mut candidate = roles.current.clone();
    set(
        &mut candidate,
        Role::TopLayer,
        document,
        member,
        0,
        &registry,
    );
    let inverse = Plan { next: candidate }.commit(&mut roles);
    assert_eq!(roles.count(Role::TopLayer, document, member), 0);
    assert_eq!(
        roles.count(Role::PendingTopLayerRemovals, document, member),
        1
    );
    inverse.restore(&mut roles);
    assert_eq!(roles.count(Role::TopLayer, document, member), 2);
    assert_eq!(
        before.value.inspect_relations(),
        roles.current.value.inspect_relations()
    );
    drop(before);
    drop(roles);
    registry.reconcile_releases();
    assert_eq!(ledger.total(), baseline);
}

#[test]
fn the_first_count_record_has_an_exact_physical_gate_and_one_short_leaves_no_half_edge() {
    for short in [true, false] {
        let (ledger, registry) = registry();
        let (owner, member, _) = keys();
        let mut version = Version::default();
        let baseline = ledger.total();
        // This is the raw Count entry + actual Debit target layout, not an
        // observed high-water price and not a whole-index completion budget.
        let first = Map::count_record_extent();
        let limit = baseline.bytes() + first.bytes() - usize::from(short);
        let calls = std::cell::Cell::new(0);
        let result = version.set(
            Role::TopLayer,
            owner,
            member,
            1,
            &mut Editing {
                registry: &registry,
                admit: &mut |usage| {
                    calls.set(calls.get() + 1);
                    if calls.get() == 1 {
                        assert_eq!(usage, baseline.checked_add(first).unwrap());
                    }
                    if usage.bytes() > limit {
                        Err(DomError::ResourceLimit {
                            resource: DomResource::SourceRetainedBytes,
                            limit,
                        })
                    } else {
                        Ok(())
                    }
                },
                step: &mut || Ok(()),
                changed: false,
            },
        );
        assert_eq!(
            result,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit
            })
        );
        assert_eq!(calls.get(), if short { 1 } else { 2 });
        assert!(version.value.inspect_relations().is_empty());
        assert!(!version.value.has_any(Role::TopLayer, owner));
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        set(&mut version, Role::TopLayer, owner, member, 1, &registry);
        assert_eq!(version.value.count(Role::TopLayer, owner, member), 1);
        drop(version);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }
}

#[test]
fn a_refused_second_direction_drops_the_candidate_and_keeps_the_published_pair() {
    let (ledger, registry) = registry();
    let (owner, member, other) = keys();
    let mut original = Version::default();
    set(
        &mut original,
        Role::OpenDialogs,
        owner,
        member,
        2,
        &registry,
    );
    let expected = original.value.inspect_relations();
    let before = ledger.total();
    let mut completed = false;
    for cutoff in 0..2048 {
        let mut candidate = original.clone();
        let calls = std::cell::Cell::new(0);
        let result = candidate.set(
            Role::OpenDialogs,
            owner,
            other,
            1,
            &mut Editing {
                registry: &registry,
                admit: &mut |_| {
                    let ordinal = calls.get();
                    calls.set(ordinal + 1);
                    if ordinal == cutoff {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                },
                step: &mut || Ok(()),
                changed: false,
            },
        );
        match result {
            Err(DomError::Cancelled) => assert_eq!(calls.get(), cutoff + 1),
            Ok(()) => {
                assert!(calls.get() <= cutoff);
                completed = true;
            }
            other => panic!("actual candidate admission: {other:?}"),
        }
        drop(candidate);
        registry.reconcile_releases();
        assert_eq!(original.value.inspect_relations(), expected);
        assert_eq!(
            original.value.reverse[Role::OpenDialogs.index()].count(member, owner),
            2
        );
        assert_eq!(ledger.total(), before);
        if completed {
            break;
        }
    }
    assert!(completed, "finite copied key paths reach completion");
}
