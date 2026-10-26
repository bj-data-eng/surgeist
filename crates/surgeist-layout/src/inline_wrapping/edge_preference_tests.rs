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
