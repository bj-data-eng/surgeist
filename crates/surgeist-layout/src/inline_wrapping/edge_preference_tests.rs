//! Independent constraints are sets, not weighted declaration counts.
use super::*;

#[test]
fn strict_subset_winner_survives_every_discovery_permutation() {
    let preferences = EdgePreferences::new(3);
    let violations = [vec![0, 1], vec![1, 2], vec![1]];
    for permutation in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut frontier = preferences.frontier();
        for candidate in permutation {
            frontier
                .consider(
                    preferences.class(&violations[candidate]).unwrap(),
                    candidate,
                )
                .unwrap();
        }
        // Candidate2 violates only constraint1: a strict subset of both others.
        assert_eq!(frontier.into_preferred(), Some(2));
    }
}

#[test]
fn incomparable_constraints_keep_the_consumers_normal_receipt_preference() {
    let preferences = EdgePreferences::new(4);
    for order in [[0, 1], [1, 0]] {
        let mut frontier = preferences.frontier();
        for candidate in order {
            // Independent box-edge occurrences. A smaller declaration count
            // does not outrank a candidate with a different violated edge.
            let edges: &[usize] = if candidate == 0 { &[0] } else { &[1, 2, 3] };
            frontier
                .consider(preferences.class(edges).unwrap(), candidate)
                .unwrap();
        }
        assert_eq!(frontier.into_preferred(), Some(order[1]));
    }
}

#[test]
fn equivalent_sets_keep_normal_ties_and_foreign_constraint_custody_is_rejected() {
    let preferences = EdgePreferences::new(2);
    let mut frontier = preferences.frontier();
    frontier
        .consider(preferences.class(&[1, 0, 1]).unwrap(), "earlier")
        .unwrap();
    frontier
        .consider(preferences.class(&[0, 1]).unwrap(), "normal-preferred")
        .unwrap();
    assert_eq!(frontier.into_preferred(), Some("normal-preferred"));
    let mut frontier = preferences.frontier();
    let foreign = EdgePreferences::new(2);
    assert_eq!(
        frontier.consider(foreign.class(&[0]).unwrap(), 0),
        Err(WrappingScopeError::InconsistentIncidence)
    );
    assert!(matches!(
        preferences.class(&[2]),
        Err(WrappingScopeError::InconsistentIncidence)
    ));
}

#[test]
fn disjoint_then_overlapping_constraints_keep_dominance_and_normal_order() {
    let preferences = EdgePreferences::new(3);
    let mut frontier = preferences.frontier();
    for (edges, receipt) in [
        (vec![0], "first singleton"),
        (vec![1], "later singleton"),
        (vec![0, 1], "dominated union"),
    ] {
        frontier
            .consider(preferences.class(&edges).unwrap(), receipt)
            .unwrap();
    }
    // Both original singletons strictly dominate the later union; neither
    // dominates the other, so their normal latest receipt still wins.
    assert_eq!(frontier.into_preferred(), Some("later singleton"));

    let mut frontier = preferences.frontier();
    for (edges, receipt) in [
        (vec![0, 1], "original superset"),
        (vec![2], "disjoint singleton"),
        (vec![1], "removes original superset"),
        (vec![0], "later independent singleton"),
    ] {
        frontier
            .consider(preferences.class(&edges).unwrap(), receipt)
            .unwrap();
    }
    // The removed superset's former edge0 is no longer a constraint on its
    // own. The three retained singleton classes are incomparable.
    assert_eq!(
        frontier.into_preferred(),
        Some("later independent singleton")
    );
}

#[test]
fn empty_constraint_set_dominates_disjoint_receipts_and_later_fresh_edges() {
    let preferences = EdgePreferences::new(3);
    let mut frontier = preferences.frontier();
    for (edges, receipt) in [
        (vec![0], "first"),
        (vec![1], "second"),
        (vec![], "outside all constraints"),
        (vec![2], "later fresh edge"),
    ] {
        frontier
            .consider(preferences.class(&edges).unwrap(), receipt)
            .unwrap();
    }
    // Empty is a strict subset of each nonempty set, including the unseen
    // later edge. Ordinary recency cannot override strict dominance.
    assert_eq!(frontier.into_preferred(), Some("outside all constraints"));
}

#[test]
fn ranked_equivalents_after_disjoint_receipts_preserve_better_worse_and_latest_ties() {
    let preferences = EdgePreferences::new(2);
    let offers = [
        (0, 40, "initial equivalent"),
        (1, 20, "disjoint equal finalist"),
        (0, 20, "better equivalent"),
        (0, 50, "worse equivalent"),
        (0, 20, "latest exact tie"),
    ];
    for (count, expected) in [
        (3, "better equivalent"),
        (4, "better equivalent"),
        (5, "latest exact tie"),
    ] {
        let mut frontier = preferences.frontier();
        for &(edge, rank, receipt) in &offers[..count] {
            frontier
                .consider_ranked(
                    preferences.class(&[edge]).unwrap(),
                    (rank, receipt),
                    |a, b| a.0.cmp(&b.0),
                )
                .unwrap();
        }
        // Observe each prefix independently: a later good offer must not
        // conceal an incorrectly accepted worse equivalent in the middle.
        assert_eq!(
            frontier.into_preferred_ranked(|a, b| a.0.cmp(&b.0)),
            Some((20, expected))
        );
    }
}

#[test]
fn mixed_disjoint_scopes_keep_foreign_rejection_and_real_empty_ancestor_dominance() {
    let ancestry = Arc::new(Ancestry {
        // Scope0 encloses sibling1 and2; scope3 is a descendant of2.
        intervals: vec![(1, 5), (2, 3), (3, 5), (4, 5)],
    });
    let class = |scope, edges: &[usize]| AvoidClass {
        ancestry: Arc::clone(&ancestry),
        scope,
        edges: edges.to_vec(),
    };
    for add_ancestor in [false, true] {
        let mut frontier = class(None, &[]).frontier();
        frontier
            .consider(class(Some(1), &[0]), "first sibling")
            .unwrap();
        frontier
            .consider(class(Some(2), &[1]), "later sibling")
            .unwrap();
        let foreign = AvoidClass {
            ancestry: Arc::new(Ancestry {
                intervals: vec![(1, 5), (2, 3), (3, 5), (4, 5)],
            }),
            scope: None,
            edges: vec![],
        };
        assert_eq!(
            frontier.consider(foreign, "foreign outside"),
            Err(WrappingScopeError::InconsistentIncidence)
        );
        if add_ancestor {
            frontier
                .consider(class(Some(0), &[]), "real empty ancestor")
                .unwrap();
            frontier
                .consider(class(Some(3), &[2]), "later descendant edge")
                .unwrap();
            // Actual ancestor containment plus the empty edge subset defeats
            // both earlier siblings and a later fresh descendant constraint.
            assert_eq!(frontier.into_preferred(), Some("real empty ancestor"));
        } else {
            // Nonempty disjoint sets remain incomparable despite differing
            // scopes. Rejected foreign input cannot disturb the latest result.
            assert_eq!(frontier.into_preferred(), Some("later sibling"));
        }
    }
}
