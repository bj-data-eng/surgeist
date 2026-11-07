//! Stable retains ordinary native cuts without a later-line quality objective.
use super::*;

fn assert_custody<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    fixture: &Tree<S>,
) {
    let count = fixture.inputs.len();
    for entries in [
        crate::layout::observation::phase(batch, false),
        crate::layout::observation::phase(batch, true),
    ] {
        assert_eq!(entries.0.node_layouts().len(), count);
        for id in 0..count {
            assert_eq!(
                entries
                    .0
                    .node_layouts()
                    .filter(|(node, _)| *node == id as u32)
                    .count(),
                1
            );
            assert_eq!(
                numeric_node_of_kind(
                    entries,
                    id as u32,
                    crate::layout::observation::expected_kind(fixture.inputs[id].as_view(), false)
                )
                .source_index,
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
    for entries in [
        crate::layout::observation::phase(batch, false),
        crate::layout::observation::phase(batch, true),
    ] {
        for (index, expected) in x.into_iter().enumerate() {
            assert_eq!(
                numeric_node(entries, (index + 1) as u32).location.x,
                S::from_f64(expected)
            );
        }
    }
}

fn ordinary_transition_and_suffix<S: l::LayoutScalar>(direction: l::Direction) {
    let capacity = crate::layout::native_request::sizing(
        surgeist_layout::Size::new(Some(S::from_f64(60.0)), None),
        surgeist_layout::Size::new(
            surgeist_layout::SizingOperation::Normal,
            surgeist_layout::SizingOperation::Normal,
        ),
    );
    let mut tree = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
    let auto = numeric_compute(&tree, capacity, &[]);
    assert_plain_cuts(&auto, direction, false);
    let balanced = numeric_style(&mut tree, l::TextWrapStyle::Balance, capacity);
    assert_plain_cuts(&balanced, direction, true);
    let stable = numeric_style(&mut tree, l::TextWrapStyle::Stable, capacity);
    assert_plain_cuts(&stable, direction, false);
    numeric_equal(&auto, &stable, &tree);

    let mut fresh = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
    let fresh_stable = numeric_style(&mut fresh, l::TextWrapStyle::Stable, capacity);
    numeric_equal(&stable, &fresh_stable, &tree);
    let replay = numeric_compute(&tree, capacity, &[]);
    numeric_equal(&stable, &replay, &tree);
    // Tree has no cache implementation: this is reexecution, not a cache hit.
    let restored = numeric_style(&mut tree, l::TextWrapStyle::Auto, capacity);
    assert_plain_cuts(&restored, direction, false);
    numeric_equal(&auto, &restored, &tree);

    let stable_again = numeric_style(&mut tree, l::TextWrapStyle::Stable, capacity);
    numeric_equal(&stable, &stable_again, &tree);
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
    for entries in [
        crate::layout::observation::phase(&suffix, false),
        crate::layout::observation::phase(&suffix, true),
    ] {
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
        (
            crate::layout::observation::phase(&stable, false),
            crate::layout::observation::phase(&suffix, false),
        ),
        (
            crate::layout::observation::phase(&stable, true),
            crate::layout::observation::phase(&suffix, true),
        ),
    ] {
        for id in [1, 2] {
            assert_eq!(
                numeric_node_of_kind(
                    before,
                    id,
                    crate::layout::observation::expected_kind(
                        l::LayoutTree::layout_input(&tree, id),
                        false
                    )
                ),
                numeric_node_of_kind(
                    after,
                    id,
                    crate::layout::observation::expected_kind(
                        l::LayoutTree::layout_input(&tree, id),
                        false
                    )
                )
            );
        }
    }
    let mut fresh_suffix = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
    fresh_suffix.inputs[3] = tree.inputs[3].clone();
    let fresh_suffix = numeric_style(&mut fresh_suffix, l::TextWrapStyle::Stable, capacity);
    numeric_equal(&suffix, &fresh_suffix, &tree);

    for batch in [&auto, &balanced, &stable, &restored, &suffix] {
        assert_custody(batch, &tree);
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
    let capacity = crate::layout::native_request::sizing(
        surgeist_layout::Size::new(Some(S::from_f64(60.0)), None),
        surgeist_layout::Size::new(
            surgeist_layout::SizingOperation::Normal,
            surgeist_layout::SizingOperation::Normal,
        ),
    );
    let mut forced = partition_tree::<S>(direction, 2, l::InlineBreakOpportunity::Allowed);
    let ordinary = numeric_compute(&forced, capacity, &[]);
    let stable = numeric_style(&mut forced, l::TextWrapStyle::Stable, capacity);
    assert_partition(&ordinary, direction, 2, false);
    assert_partition(&stable, direction, 2, false);
    numeric_equal(&ordinary, &stable, &forced);
    assert_custody(&stable, &forced);

    // Actual Prohibited participation is this numeric owner's no-soft-wrap
    // permission. Stable cannot invent cuts, even at the real Forced groups.
    for groups in [1, 2] {
        let mut tree =
            partition_tree::<S>(direction, groups, l::InlineBreakOpportunity::Prohibited);
        let auto = numeric_compute(&tree, capacity, &[]);
        let stable = numeric_style(&mut tree, l::TextWrapStyle::Stable, capacity);
        numeric_equal(&auto, &stable, &tree);
        assert_custody(&stable, &tree);
        for entries in [
            crate::layout::observation::phase(&stable, false),
            crate::layout::observation::phase(&stable, true),
        ] {
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
                    let control = numeric_node_of_kind(
                        entries,
                        first + 3,
                        crate::layout::observation::expected_kind(
                            l::LayoutTree::layout_input(&tree, first + 3),
                            false,
                        ),
                    );
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
            crate::layout::native_request::sizing(
                surgeist_layout::Size::new(None, None),
                surgeist_layout::Size::new(
                    surgeist_layout::SizingOperation::MinContent,
                    surgeist_layout::SizingOperation::Normal,
                ),
            )
        } else {
            crate::layout::native_request::sizing(
                surgeist_layout::Size::new(None, None),
                surgeist_layout::Size::new(
                    surgeist_layout::SizingOperation::MaxContent,
                    surgeist_layout::SizingOperation::Normal,
                ),
            )
        };
        let auto = numeric_compute(&tree, available, &[]);
        let stable = numeric_style(&mut tree, l::TextWrapStyle::Stable, available);
        numeric_equal(&auto, &stable, &tree);
        assert_custody(&stable, &tree);
        for batch in [&auto, &stable] {
            for entries in [
                crate::layout::observation::phase(batch, false),
                crate::layout::observation::phase(batch, true),
            ] {
                assert_eq!(
                    numeric_node(entries, 0).size,
                    l::Size::new(
                        S::from_f64(if minimum { 30.0 } else { 70.0 }),
                        S::from_f64(if minimum { 30.0 } else { 10.0 })
                    )
                );
                for (i, width) in [30.0, 30.0, 10.0].into_iter().enumerate() {
                    let actual = numeric_node_of_kind(
                        entries,
                        (i + 1) as u32,
                        crate::layout::observation::expected_kind(
                            l::LayoutTree::layout_input(&tree, (i + 1) as u32),
                            false,
                        ),
                    );
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
