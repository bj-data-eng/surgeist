use super::*;
use std::collections::BTreeSet;

fn bytes(value: &[u8]) -> KeyView<'_, 0> {
    KeyView {
        words: [],
        used: 0,
        suffix: Suffix::Utf8(value),
    }
}
fn units(value: &[u16]) -> KeyView<'_, 0> {
    KeyView {
        words: [],
        used: 0,
        suffix: Suffix::Utf16(value),
    }
}
fn registry() -> bank::CustodyRegistry {
    bank::CustodyRegistry::new(&custody::Ledger::default(), |_| Ok(())).unwrap()
}
fn install<L: Copy + Eq>(
    forest: &mut Forest<L>,
    control: &mut Custody<L>,
    root: &mut Root,
    change: Change<L>,
) {
    let old = std::mem::replace(forest, change.forest);
    let old_control = std::mem::replace(control, change.custody);
    *root = change.root;
    drop(old);
    drop(old_control);
}
fn inspect<L: Copy + Eq>(forest: &Forest<L>, root: Root) {
    fn visit<L: Copy + Eq>(
        forest: &Forest<L>,
        index: usize,
        parent: Option<usize>,
        seen: &mut BTreeSet<usize>,
    ) -> BTreeSet<usize> {
        assert!(seen.insert(index), "each raw edge occurs once");
        match forest.node(index).unwrap() {
            Node::Leaf(_) => BTreeSet::from([index]),
            Node::Branch {
                bit,
                children,
                representative,
            } => {
                assert!(parent.is_none_or(|parent| parent < bit));
                let mut leaves = visit(forest, children[0], Some(bit), seen);
                leaves.extend(visit(forest, children[1], Some(bit), seen));
                assert!(
                    leaves.contains(&representative),
                    "representative belongs to this subtree"
                );
                assert!(matches!(forest.node(representative), Some(Node::Leaf(_))));
                leaves
            }
        }
    }
    let mut seen = BTreeSet::new();
    if let Some(root) = root {
        visit(forest, root, None, &mut seen);
    }
    assert_eq!(seen.len(), forest.inspect_summaries().0.occurrences);
    let graph = forest.inspect_graph().unwrap();
    let (totals, index) = forest.inspect_summaries();
    assert_eq!(totals, graph.record_totals());
    assert_eq!(index, graph.index_charge());
    assert_eq!(
        forest.heap_usage().unwrap(),
        custody::Usage::new(graph.record_totals().bytes + graph.index_charge().bytes, 0)
    );
}

#[test]
fn short_absent_query_and_insert_exclude_long_common_prefix_chains() {
    for count in [1, 16, 128, 512] {
        let registry = registry();
        let mut forest = Forest::default();
        let mut control = Custody::default();
        let mut root = None;
        let mut names = Vec::new();
        for length in 1..=count {
            let mut name = b"p".to_vec();
            name.extend(std::iter::repeat_n(b'a', length));
            name.push(b'b');
            names.push(name);
        }
        names.push(b"p".to_vec());
        for index in 0..count {
            let change = forest
                .change(
                    &control,
                    root,
                    &bytes(&names[index]),
                    Some(index),
                    |id| Ok(bytes(&names[id])),
                    Edit {
                        registry: &registry,
                        admit: &mut |_| Ok(()),
                        step: &mut || Ok(()),
                    },
                )
                .unwrap();
            install(&mut forest, &mut control, &mut root, change);
        }
        inspect(&forest, root);
        let mut work = 0;
        let mut step = || {
            work += 1;
            Ok(())
        };
        assert_eq!(
            forest
                .lookup(root, &bytes(b"p"), |id| Ok(bytes(&names[id])), &mut step)
                .unwrap(),
            None
        );
        assert!(work <= 40, "short query count={count}, work={work}");
        let mut work = 0;
        let change = forest
            .change(
                &control,
                root,
                &bytes(b"p"),
                Some(count),
                |id| Ok(bytes(&names[id])),
                Edit {
                    registry: &registry,
                    admit: &mut |_| Ok(()),
                    step: &mut || {
                        work += 1;
                        Ok(())
                    },
                },
            )
            .unwrap();
        assert_eq!(change.previous, None);
        assert!(work <= 40, "short insert count={count}, work={work}");
        install(&mut forest, &mut control, &mut root, change);
        inspect(&forest, root);
        for (id, name) in names.iter().enumerate() {
            assert_eq!(
                forest
                    .lookup(root, &bytes(name), |id| Ok(bytes(&names[id])), &mut || Ok(
                        ()
                    ))
                    .unwrap(),
                Some(id)
            );
        }
    }
}

#[test]
fn exact_utf16_names_preserve_terminators_prefixes_and_lone_surrogates() {
    let registry = registry();
    let mut forest = Forest::default();
    let mut control = Custody::default();
    let mut root = None;
    let names = [
        vec![],
        vec![0],
        vec![0, 0],
        vec![0xd800],
        vec![0xd800, 0],
        vec![0xdc00],
        vec![65],
        vec![97],
    ];
    for (id, name) in names.iter().enumerate() {
        let change = forest
            .change(
                &control,
                root,
                &units(name),
                Some(id),
                |id| Ok(units(&names[id])),
                Edit {
                    registry: &registry,
                    admit: &mut |_| Ok(()),
                    step: &mut || Ok(()),
                },
            )
            .unwrap();
        install(&mut forest, &mut control, &mut root, change);
    }
    inspect(&forest, root);
    for (id, name) in names.iter().enumerate() {
        assert_eq!(
            forest
                .lookup(root, &units(name), |id| Ok(units(&names[id])), &mut || Ok(
                    ()
                ))
                .unwrap(),
            Some(id)
        );
    }
    for absent in [vec![0, 1], vec![0xd800, 1], vec![0xd800, 0, 0], vec![66]] {
        assert_eq!(
            forest
                .lookup(
                    root,
                    &units(&absent),
                    |id| Ok(units(&names[id])),
                    &mut || Ok(())
                )
                .unwrap(),
            None
        );
    }
}

#[test]
fn leaf_replacement_and_retirement_keep_representatives_live_through_slot_reuse() {
    let registry = registry();
    let mut forest = Forest::default();
    let mut control = Custody::default();
    let mut root = None;
    let names = [
        b"a".to_vec(),
        b"ab".to_vec(),
        b"abc".to_vec(),
        b"b".to_vec(),
        b"c".to_vec(),
        b"abc".to_vec(),
        b"fresh".to_vec(),
    ];
    for id in 0..5 {
        let change = forest
            .change(
                &control,
                root,
                &bytes(&names[id]),
                Some(id),
                |id| Ok(bytes(&names[id])),
                Edit {
                    registry: &registry,
                    admit: &mut |_| Ok(()),
                    step: &mut || Ok(()),
                },
            )
            .unwrap();
        install(&mut forest, &mut control, &mut root, change);
    }
    let old = forest.clone();
    let old_root = root;
    // Both old/new representatives remain available during preparation. Once
    // committed, no branch may resolve the retired representative payload.
    let change = forest
        .change(
            &control,
            root,
            &bytes(b"abc"),
            Some(5),
            |id| Ok(bytes(&names[id])),
            Edit {
                registry: &registry,
                admit: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    assert_eq!(change.previous, Some(2));
    install(&mut forest, &mut control, &mut root, change);
    assert_eq!(
        forest
            .lookup(
                root,
                &bytes(b"abc"),
                |id| {
                    assert_ne!(id, 2);
                    Ok(bytes(&names[id]))
                },
                &mut || Ok(())
            )
            .unwrap(),
        Some(5)
    );
    for id in [0, 5, 3, 1, 4] {
        let change = forest
            .change(
                &control,
                root,
                &bytes(&names[id]),
                None,
                |id| Ok(bytes(&names[id])),
                Edit {
                    registry: &registry,
                    admit: &mut |_| Ok(()),
                    step: &mut || Ok(()),
                },
            )
            .unwrap();
        assert_eq!(change.previous, Some(id));
        install(&mut forest, &mut control, &mut root, change);
        inspect(&forest, root);
    }
    let change = forest
        .change(
            &control,
            root,
            &bytes(b"fresh"),
            Some(6),
            |id| Ok(bytes(&names[id])),
            Edit {
                registry: &registry,
                admit: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    install(&mut forest, &mut control, &mut root, change);
    inspect(&forest, root);
    for id in 0..5 {
        assert_eq!(
            old.lookup(
                old_root,
                &bytes(&names[id]),
                |id| Ok(bytes(&names[id])),
                &mut || Ok(())
            )
            .unwrap(),
            Some(id)
        );
    }
}

#[test]
fn refused_actual_admission_releases_candidate_and_same_owner_retry_succeeds() {
    let registry = registry();
    let forest = Forest::default();
    let control = Custody::default();
    let mut proposal = None;
    let result = forest.change(
        &control,
        None,
        &bytes(b"name"),
        Some(0),
        |_| Ok(bytes(b"name")),
        Edit {
            registry: &registry,
            admit: &mut |usage| {
                if usage.bytes() != 0 {
                    let limit = usage.bytes() - 1;
                    proposal = Some((usage, limit));
                    Err(DomError::ResourceLimit {
                        resource: crate::DomResource::SourceRetainedBytes,
                        limit,
                    })
                } else {
                    Ok(())
                }
            },
            step: &mut || Ok(()),
        },
    );
    let (usage, limit) = proposal.expect("actual positive resource proposal");
    assert_eq!(usage.bytes(), limit + 1);
    assert!(
        matches!(result,Err(DomError::ResourceLimit{resource:crate::DomResource::SourceRetainedBytes,limit:actual}) if actual==limit)
    );
    registry.reconcile_releases();
    inspect(&forest, None);
    let retry = forest
        .change(
            &control,
            None,
            &bytes(b"name"),
            Some(0),
            |_| Ok(bytes(b"name")),
            Edit {
                registry: &registry,
                admit: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    inspect(&retry.forest, retry.root);
    assert_eq!(
        retry
            .forest
            .lookup(
                retry.root,
                &bytes(b"name"),
                |_| Ok(bytes(b"name")),
                &mut || Ok(())
            )
            .unwrap(),
        Some(0)
    );
}

#[test]
fn cancellation_after_candidate_allocation_preserves_original_and_allows_retry() {
    let registry = registry();
    let empty = Forest::default();
    let control = Custody::default();
    let initial = empty
        .change(
            &control,
            None,
            &bytes(b"old"),
            Some(0),
            |id| Ok(bytes(if id == 0 { b"old" } else { b"new" })),
            Edit {
                registry: &registry,
                admit: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    let forest = initial.forest;
    let control = initial.custody;
    let root = initial.root;
    let allocated = std::cell::Cell::new(false);
    let cancelled = std::cell::Cell::new(false);
    let result = forest.change(
        &control,
        root,
        &bytes(b"new"),
        Some(1),
        |id| Ok(bytes(if id == 0 { b"old" } else { b"new" })),
        Edit {
            registry: &registry,
            admit: &mut |usage| {
                if usage.bytes() != 0 {
                    allocated.set(true);
                }
                Ok(())
            },
            step: &mut || {
                if allocated.get() {
                    cancelled.set(true);
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
        },
    );
    assert!(
        allocated.get() && cancelled.get(),
        "cancel after an actual retained allocation"
    );
    assert!(matches!(result, Err(DomError::Cancelled)));
    registry.reconcile_releases();
    inspect(&forest, root);
    assert_eq!(
        forest
            .lookup(root, &bytes(b"old"), |_| Ok(bytes(b"old")), &mut || Ok(()))
            .unwrap(),
        Some(0)
    );
    assert_eq!(
        forest
            .lookup(root, &bytes(b"new"), |_| Ok(bytes(b"old")), &mut || Ok(()))
            .unwrap(),
        None
    );
    let retry = forest
        .change(
            &control,
            root,
            &bytes(b"new"),
            Some(1),
            |id| Ok(bytes(if id == 0 { b"old" } else { b"new" })),
            Edit {
                registry: &registry,
                admit: &mut |_| Ok(()),
                step: &mut || Ok(()),
            },
        )
        .unwrap();
    inspect(&retry.forest, retry.root);
    assert_eq!(
        retry
            .forest
            .lookup(
                retry.root,
                &bytes(b"new"),
                |id| Ok(bytes(if id == 0 { b"old" } else { b"new" })),
                &mut || Ok(())
            )
            .unwrap(),
        Some(1)
    );
}
