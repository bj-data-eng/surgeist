#![forbid(unsafe_code)]

use surgeist_fuzz::{FuzzErrorKind, InputLimits, generation::*};

fn limits(bytes: u64, nodes: u64, depth: u64, text: u64, operations: u64) -> InputLimits {
    InputLimits::new(bytes, nodes, depth, text, operations).unwrap()
}

#[test]
fn byte_text_and_vector_budgets_reject_before_construction() {
    let mut budget = Budget::new(limits(4, 0, 0, 2, 0));
    let mut entropy = Entropy::new(&[0, 0xd8, 0xff, 0xdf, 1]);
    assert_eq!(entropy.utf16(2, &mut budget).unwrap(), [0xd800, 0xdfff]);
    assert_eq!(budget.used().bytes, 4);
    assert_eq!(budget.used().text_units, 2);
    assert!(entropy.bytes(1, &mut budget).is_err());
    assert_eq!(entropy.remaining(), 1);

    let mut calls = 0;
    let mut budget = Budget::new(limits(7, 0, 0, 0, 0));
    assert!(
        bounded_vector(2, BudgetCharge::default(), &mut budget, || {
            calls += 1;
            Ok(4_u32)
        })
        .is_err()
    );
    assert_eq!(calls, 0);
    assert_eq!(budget.used(), BudgetCharge::default());
    let result = bounded_vector(
        1,
        BudgetCharge {
            bytes: 3,
            ..BudgetCharge::default()
        },
        &mut budget,
        || Ok(4_u32),
    )
    .unwrap();
    assert_eq!(result, [4]);
    assert_eq!(budget.used().bytes, 7);
}

#[test]
fn strict_utf8_preserves_valid_scalars_and_rejects_lossy_conversion() {
    let mut budget = Budget::new(limits(5, 0, 0, 2, 0));
    assert_eq!(
        Entropy::new("😀a".as_bytes()).utf8(5, &mut budget).unwrap(),
        "😀a"
    );
    assert_eq!(budget.used().text_units, 2);
    let mut budget = Budget::new(limits(1, 0, 0, 1, 0));
    assert_eq!(
        Entropy::new(&[0xff])
            .utf8(1, &mut budget)
            .unwrap_err()
            .kind(),
        FuzzErrorKind::InvalidCase
    );
    assert_eq!(budget.used(), BudgetCharge::default());
}

#[test]
fn empty_wide_and_deep_forests_have_exact_checked_edges() {
    let empty = OrderedForest::from_parents(&[], &mut Budget::new(limits(1, 0, 0, 0, 0))).unwrap();
    assert_eq!(empty.preorder().count(), 0);
    let count = 10_000_u64;
    let mut budget = Budget::new(limits(
        count * OrderedForest::node_bytes(),
        count,
        count,
        0,
        0,
    ));
    let mut deep = OrderedForest::new();
    let mut parent = None;
    for _ in 0..count {
        parent = Some(deep.push(parent, &mut budget).unwrap());
    }
    assert_eq!(deep.depth(parent.unwrap()).unwrap(), count);
    assert_eq!(deep.preorder().count() as u64, count);
    assert_eq!(budget.used().edges, count - 1);
    assert_eq!(budget.used().bytes, count * OrderedForest::node_bytes());
    assert!(deep.push(parent, &mut budget).is_err());
    assert_eq!(deep.len() as u64, count);
    // Flat storage can be dropped at a depth exceeding ordinary stack budgets.
    drop(deep);

    let mut budget = Budget::new(limits(5 * OrderedForest::node_bytes(), 5, 2, 0, 0));
    let mut wide = OrderedForest::new();
    let root = wide.push(None, &mut budget).unwrap();
    for _ in 0..4 {
        wide.push(Some(root), &mut budget).unwrap();
    }
    assert_eq!(
        wide.children(root)
            .unwrap()
            .map(FixtureId::index)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        wide.preorder().map(FixtureId::index).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );
    assert_eq!(budget.used().depth, 2);
}

#[test]
fn disconnected_roots_and_subtree_walk_preserve_parent_child_order() {
    let parents = [
        None,
        Some(FixtureId::new(0)),
        None,
        Some(FixtureId::new(1)),
        Some(FixtureId::new(0)),
        Some(FixtureId::new(2)),
    ];
    let forest = OrderedForest::from_parents(
        &parents,
        &mut Budget::new(limits(6 * OrderedForest::node_bytes(), 6, 3, 0, 0)),
    )
    .unwrap();
    assert_eq!(
        forest.roots().map(FixtureId::index).collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(
        forest.preorder().map(FixtureId::index).collect::<Vec<_>>(),
        [0, 1, 3, 4, 2, 5]
    );
    assert_eq!(
        forest
            .subtree(FixtureId::new(1))
            .unwrap()
            .map(FixtureId::index)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    for (index, parent) in parents.into_iter().enumerate() {
        assert_eq!(forest.parent(FixtureId::new(index)).unwrap(), parent);
        if let Some(parent) = parent {
            assert!(
                forest
                    .children(parent)
                    .unwrap()
                    .any(|id| id.index() == index)
            );
        }
    }
}

#[test]
fn invalid_references_and_excess_depth_are_rejected_without_repair() {
    let mut budget = Budget::new(limits(3 * OrderedForest::node_bytes(), 3, 2, 0, 0));
    assert!(OrderedForest::from_parents(&[Some(FixtureId::new(0))], &mut budget).is_err());
    assert!(OrderedForest::from_parents(&[Some(FixtureId::new(1)), None], &mut budget).is_err());
    assert_eq!(budget.used(), BudgetCharge::default());
    let mut forest = OrderedForest::new();
    let root = forest.push(None, &mut budget).unwrap();
    let child = forest.push(Some(root), &mut budget).unwrap();
    let before = budget.used();
    assert!(forest.push(Some(child), &mut budget).is_err());
    assert!(forest.push(Some(FixtureId::new(99)), &mut budget).is_err());
    assert_eq!(budget.used(), before);
    assert_eq!(forest.len(), 2);
}

#[test]
fn shared_credit_rejects_overflow_and_edges_above_the_node_ceiling() {
    let mut budget = Budget::new(limits(u64::MAX, 2, 2, u64::MAX, u64::MAX));
    budget
        .credit(BudgetCharge {
            bytes: u64::MAX,
            nodes: 2,
            depth: 2,
            edges: 1,
            ..BudgetCharge::default()
        })
        .unwrap();
    let before = budget.used();
    assert!(
        budget
            .credit(BudgetCharge {
                bytes: 1,
                ..BudgetCharge::default()
            })
            .is_err()
    );
    assert!(
        budget
            .credit(BudgetCharge {
                edges: 1,
                ..BudgetCharge::default()
            })
            .is_err()
    );
    assert_eq!(budget.used(), before);
}

#[test]
fn numeric_choices_exhaust_without_retry_and_include_exact_float_edges() {
    let mut entropy = Entropy::new(&[]);
    assert_eq!(entropy.range(7, 7).unwrap(), 7);
    assert!(entropy.range(0, 1).is_err());
    assert_eq!(entropy.remaining(), 0);
    assert!(entropy.choice::<u8>(&[]).is_err());
    assert!(entropy.weighted_choice(&[(0, 3)]).is_err());
    assert!(entropy.weighted_choice(&[(u64::MAX, 1), (1, 2)]).is_err());
    assert_eq!(*entropy.weighted_choice(&[(0, 1), (1, 2)]).unwrap(), 2);
    assert_eq!(
        Entropy::new(&u64::MAX.to_le_bytes())
            .range(0, u64::MAX)
            .unwrap(),
        u64::MAX
    );
    assert!(FLOAT_EDGES.contains(&(-0.0_f64).to_bits()));
    assert!(
        FLOAT_EDGES
            .iter()
            .any(|bits| f64::from_bits(*bits).is_nan())
    );
    assert!(
        FLOAT_EDGES
            .iter()
            .any(|bits| f64::from_bits(*bits).is_infinite())
    );
}

#[test]
fn deterministic_recipe_materializes_identical_bytes_for_the_same_basis() {
    let config = DeterministicConfig {
        seed: 42,
        entropy_bytes: 32,
    };
    let ceilings = limits(32, 0, 0, 0, 0);
    assert_eq!(
        config.entropy(ceilings).unwrap(),
        config.entropy(ceilings).unwrap()
    );
    assert_ne!(
        config.entropy(ceilings).unwrap(),
        DeterministicConfig { seed: 43, ..config }
            .entropy(ceilings)
            .unwrap()
    );
    assert!(config.entropy(limits(31, 0, 0, 0, 0)).is_err());
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Operation {
    Create(u32),
    Use(u32),
}

#[test]
fn sequences_check_dependent_references_count_discards_and_preserve_invalid_requests() {
    let mut budget = Budget::new(limits(1024, 0, 0, 0, 3));
    let mut sequence = SequenceBuilder::new();
    let valid = |steps: &[SequenceStep<Operation>], operation: &Operation| match operation {
        Operation::Create(_) => true,
        Operation::Use(id) => steps.iter().any(|step| {
            step.lane() == SequenceLane::Valid && step.operation() == &Operation::Create(*id)
        }),
    };
    assert!(
        !sequence
            .append_valid(
                &mut budget,
                BudgetCharge::default(),
                || Ok(Operation::Use(7)),
                valid
            )
            .unwrap()
    );
    assert_eq!(sequence.rejected_preconditions(), 1);
    assert_eq!(budget.used().operations, 0);
    assert!(
        sequence
            .append_valid(
                &mut budget,
                BudgetCharge::default(),
                || Ok(Operation::Create(7)),
                valid
            )
            .unwrap()
    );
    assert!(
        sequence
            .append_valid(
                &mut budget,
                BudgetCharge::default(),
                || Ok(Operation::Use(7)),
                valid
            )
            .unwrap()
    );
    sequence
        .append_invalid(&mut budget, BudgetCharge::default(), || {
            Ok(Operation::Use(99))
        })
        .unwrap();
    assert_eq!(sequence.steps()[2].lane(), SequenceLane::ExpectedRejection);
    assert_eq!(sequence.steps()[2].operation(), &Operation::Use(99));
    let mut constructed = false;
    assert!(
        sequence
            .append_invalid(&mut budget, BudgetCharge::default(), || {
                constructed = true;
                Ok(Operation::Create(8))
            })
            .is_err()
    );
    assert!(!constructed);
    assert_eq!(budget.used().operations, 3);
}

#[cfg(feature = "property")]
#[test]
fn property_strategies_admit_every_materialized_case_under_hard_caps() {
    use proptest::{
        strategy::{Strategy, ValueTree},
        test_runner::{Config, TestRunner},
    };
    let ceilings = limits(3 * OrderedForest::node_bytes(), 3, 2, 3, 0);
    let mut runner = TestRunner::new(Config {
        cases: 128,
        failure_persistence: None,
        ..Config::default()
    });
    for _ in 0..128 {
        let forest = property::forest(10_000, ceilings)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        assert!(forest.len() <= 3);
        assert!(forest.preorder().all(|id| forest.depth(id).unwrap() <= 2));
        let units = property::utf16(10_000, ceilings)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        assert!(units.len() <= 3);
        let text = property::utf8(10_000, ceilings)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        assert!(text.chars().count() <= 3);
        assert!(text.len() as u64 <= ceilings.bytes());
        let bytes = property::bytes(10_000, ceilings)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        assert!(bytes.len() as u64 <= ceilings.bytes());
        let values = property::vector(
            proptest::prelude::any::<u32>(),
            10_000,
            BudgetCharge {
                text_units: 1,
                ..BudgetCharge::default()
            },
            ceilings,
        )
        .unwrap()
        .new_tree(&mut runner)
        .unwrap()
        .current();
        assert!(values.len() <= 3);
    }
    let zero = limits(1, 0, 0, 0, 0);
    assert!(
        property::forest(usize::MAX, zero)
            .new_tree(&mut runner)
            .unwrap()
            .current()
            .is_empty()
    );
}

#[cfg(feature = "coverage")]
#[test]
fn arbitrary_adapters_bound_wide_recipes_and_terminate_on_empty_entropy() {
    use arbitrary::Unstructured;
    let ceilings = limits(3 * OrderedForest::node_bytes(), 3, 2, 3, 0);
    for fill in [0, 1, 0x7f, 0xff] {
        let bytes = [fill; 1024];
        let forest = coverage::forest(&mut Unstructured::new(&bytes), 10_000, ceilings).unwrap();
        assert!(forest.len() <= 3);
        assert!(forest.preorder().all(|id| forest.depth(id).unwrap() <= 2));
        let units = coverage::utf16(&mut Unstructured::new(&bytes), 10_000, ceilings).unwrap();
        assert!(units.len() <= 3);
        let output = coverage::bytes(&mut Unstructured::new(&bytes), 10_000, ceilings).unwrap();
        assert!(output.len() as u64 <= ceilings.bytes());
    }
    let _finite_result = coverage::forest(&mut Unstructured::new(&[]), usize::MAX, ceilings);
}
