//! Stable retains ordinary native cuts without a later-line quality objective.
use super::*;

fn assert_custody<S: l::LayoutScalar>(batch: &l::CompletedLayoutBatchOf<u32, S>, count: usize) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), count);
        for id in 0..count {
            assert_eq!(
                entries
                    .iter()
                    .filter(|entry| entry.node() == id as u32)
                    .count(),
                1
            );
            assert_eq!(
                numeric_node(entries, id as u32).source_index,
                l::SourceIndex::new(id.saturating_sub(1))
            );
        }
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.unrounded_inline_fragments().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

fn assert_plain_cuts<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    direction: l::Direction,
    balanced: bool,
) {
    assert_partition(batch, direction, 1, balanced);
    let x = match (direction, balanced) {
        (l::Direction::Ltr, false) => [0.0, 30.0, 0.0],
        (l::Direction::Rtl, false) => [30.0, 0.0, 50.0],
        (l::Direction::Ltr, true) => [0.0, 0.0, 30.0],
        (l::Direction::Rtl, true) => [30.0, 30.0, 20.0],
    };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for (index, expected) in x.into_iter().enumerate() {
            assert_eq!(
                numeric_node(entries, (index + 1) as u32).location.x,
                S::from_f64(expected)
            );
        }
    }
}

fn ordinary_transition_and_suffix<S: l::LayoutScalar>(direction: l::Direction) {
    let capacity = l::AvailableOf::definite(S::from_f64(60.0));
    let mut tree = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
    let auto = numeric_compute(&tree, capacity, &[]);
    assert_plain_cuts(&auto, direction, false);
    let balanced = numeric_style(&mut tree, l::TextWrapStyle::Balance, capacity);
    assert_plain_cuts(&balanced, direction, true);
    let stable = numeric_style(&mut tree, l::TextWrapStyle::Stable, capacity);
    assert_plain_cuts(&stable, direction, false);
    numeric_equal(&auto, &stable, tree.inputs.len());

    let mut fresh = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
    let fresh_stable = numeric_style(&mut fresh, l::TextWrapStyle::Stable, capacity);
    numeric_equal(&stable, &fresh_stable, tree.inputs.len());
    let replay = numeric_compute(&tree, capacity, &[]);
    numeric_equal(&stable, &replay, tree.inputs.len());
    // Tree has no cache implementation: this is reexecution, not a cache hit.
    let restored = numeric_style(&mut tree, l::TextWrapStyle::Auto, capacity);
    assert_plain_cuts(&restored, direction, false);
    numeric_equal(&auto, &restored, tree.inputs.len());

    let stable_again = numeric_style(&mut tree, l::TextWrapStyle::Stable, capacity);
    numeric_equal(&stable, &stable_again, tree.inputs.len());
    let old_tail = tree.inputs[3].as_box().cloned().unwrap();
    tree.inputs[3] = l::LayoutInputOf::box_input(l::NodeInputOf {
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(40.0)).unwrap()),
            old_tail.size.height,
        ),
        ..old_tail
    });
    // The real child mutation names that child; ancestor invalidation is owned
    // by compute_layout_invalidated, not an invented cache revision.
    let suffix = numeric_compute(&tree, capacity, &[3]);
    assert_eq!(
        tree.inputs[0].as_box().unwrap().text_wrap_style,
        l::TextWrapStyle::Stable
    );
    for entries in [suffix.unrounded_entries(), suffix.final_entries()] {
        assert_eq!(
            numeric_node(entries, 0).size,
            l::Size::new(S::from_f64(60.0), S::from_f64(20.0))
        );
        assert_row(entries, &[1, 2], direction, 0.0, 60.0);
        assert_row(entries, &[3], direction, 10.0, 40.0);
        assert_eq!(
            numeric_node(entries, 3).size,
            l::Size::new(S::from_f64(40.0), S::from_f64(10.0))
        );
    }
    for (before, after) in [
        (stable.unrounded_entries(), suffix.unrounded_entries()),
        (stable.final_entries(), suffix.final_entries()),
    ] {
        for id in [1, 2] {
            assert_eq!(numeric_node(before, id), numeric_node(after, id));
        }
    }
    let mut fresh_suffix = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
    fresh_suffix.inputs[3] = tree.inputs[3].clone();
    let fresh_suffix = numeric_style(&mut fresh_suffix, l::TextWrapStyle::Stable, capacity);
    numeric_equal(&suffix, &fresh_suffix, tree.inputs.len());

    for batch in [&auto, &balanced, &stable, &restored, &suffix] {
        assert_custody(batch, tree.inputs.len());
    }
    // Prior completed values retain their original cuts, dimensions and source.
    assert_plain_cuts(&auto, direction, false);
    assert_plain_cuts(&balanced, direction, true);
    assert_plain_cuts(&stable, direction, false);
}

#[test]
fn stable_native_prefix_survives_style_switches_and_a_coherent_suffix_change() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        ordinary_transition_and_suffix::<f32>(direction);
        ordinary_transition_and_suffix::<f64>(direction);
    }
}

fn forced_and_prohibited<S: l::LayoutScalar>(direction: l::Direction) {
    let capacity = l::AvailableOf::definite(S::from_f64(60.0));
    let mut forced = partition_tree::<S>(direction, 2, l::InlineBreakOpportunity::Allowed);
    let ordinary = numeric_compute(&forced, capacity, &[]);
    let stable = numeric_style(&mut forced, l::TextWrapStyle::Stable, capacity);
    assert_partition(&ordinary, direction, 2, false);
    assert_partition(&stable, direction, 2, false);
    numeric_equal(&ordinary, &stable, forced.inputs.len());
    assert_custody(&stable, forced.inputs.len());

    // Actual Prohibited participation is this numeric owner's no-soft-wrap
    // permission. Stable cannot invent cuts, even at the real Forced groups.
    for groups in [1, 2] {
        let mut tree =
            partition_tree::<S>(direction, groups, l::InlineBreakOpportunity::Prohibited);
        let auto = numeric_compute(&tree, capacity, &[]);
        let stable = numeric_style(&mut tree, l::TextWrapStyle::Stable, capacity);
        numeric_equal(&auto, &stable, tree.inputs.len());
        assert_custody(&stable, tree.inputs.len());
        for entries in [stable.unrounded_entries(), stable.final_entries()] {
            assert_eq!(
                numeric_node(entries, 0).size,
                l::Size::new(S::from_f64(60.0), S::from_f64(10.0 * groups as f64))
            );
            for group in 0..groups {
                let first = u32::try_from(1 + group * 4).unwrap();
                assert_row(
                    entries,
                    &[first, first + 1, first + 2],
                    direction,
                    10.0 * group as f64,
                    70.0,
                );
                if group + 1 < groups {
                    let control = numeric_node(entries, first + 3);
                    assert_eq!(control.size, l::Size::ZERO);
                    assert_eq!(control.location.y, S::from_f64(10.0 * (group + 1) as f64));
                }
            }
        }
    }
}

#[test]
fn stable_preserves_real_forced_groups_and_prohibited_soft_edges() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        forced_and_prohibited::<f32>(direction);
        forced_and_prohibited::<f64>(direction);
    }
}

fn ordinary_intrinsics<S: l::LayoutScalar>(direction: l::Direction) {
    for minimum in [true, false] {
        let mut tree = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
        let old = tree.inputs[0].as_box().cloned().unwrap();
        tree.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
            size: l::Size::new(l::PreferredSizeOf::AUTO, old.size.height),
            ..old
        });
        let available = if minimum {
            l::AvailableOf::MIN_CONTENT
        } else {
            l::AvailableOf::MAX_CONTENT
        };
        let auto = numeric_compute(&tree, available, &[]);
        let stable = numeric_style(&mut tree, l::TextWrapStyle::Stable, available);
        numeric_equal(&auto, &stable, tree.inputs.len());
        assert_custody(&stable, tree.inputs.len());
        for batch in [&auto, &stable] {
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(
                    numeric_node(entries, 0).size,
                    l::Size::new(
                        S::from_f64(if minimum { 30.0 } else { 70.0 }),
                        S::from_f64(if minimum { 30.0 } else { 10.0 })
                    )
                );
                for (i, width) in [30.0, 30.0, 10.0].into_iter().enumerate() {
                    let actual = numeric_node(entries, (i + 1) as u32);
                    assert_eq!(
                        actual.size,
                        l::Size::new(S::from_f64(width), S::from_f64(10.0))
                    );
                    assert_eq!(
                        actual.location.y,
                        S::from_f64(if minimum { 10.0 * i as f64 } else { 0.0 })
                    );
                }
            }
        }
    }
}

#[test]
fn stable_intrinsic_numeric_contributions_remain_ordinary_min_and_max_content() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        ordinary_intrinsics::<f32>(direction);
        ordinary_intrinsics::<f64>(direction);
    }
}
