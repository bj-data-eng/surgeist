//! Real numeric participants exercise the same normalized selection policy.
mod stable;

use super::*;

fn numeric_atomic<S: l::LayoutScalar>(
    direction: l::Direction,
    width: f64,
    height: f64,
    following: l::InlineBreakOpportunity,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::InlineBlock,
        direction,
        vertical_align: l::VerticalAlign::Baseline,
        atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
            t::BidiLevel::try_new(if direction == l::Direction::Ltr { 0 } else { 1 }).unwrap(),
            following,
        )),
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(width)).unwrap()),
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(height)).unwrap()),
        ),
        ..l::NodeInputOf::default()
    })
}
fn numeric_forced<S: l::LayoutScalar>(
    direction: l::Direction,
    baseline: f64,
    height: f64,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::line_break(
        l::LineBreakInputOf::new()
            .with_direction(direction)
            .with_metrics(
                l::InlineMetricsOf::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    S::from_f64(baseline),
                    S::from_f64(height),
                )
                .unwrap(),
            ),
    )
}
fn numeric_boundary<S: l::LayoutScalar>(
    direction: l::Direction,
    kind: l::InlineBoundaryKind,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::inline_boundary(
        l::InlineBoundaryInputOf::new(
            kind,
            l::InlineMetricsOf::try_new(
                surgeist_layout::InlineBaselineContext::Physical,
                S::from_f64(8.0),
                S::from_f64(10.0),
            )
            .unwrap(),
            l::InlineBoundaryEdgeOf::ZERO,
        )
        .with_direction(direction),
    )
}
fn numeric_tree<S: l::LayoutScalar>(
    direction: l::Direction,
    participants: Vec<l::LayoutInputOf<S>>,
) -> Tree<S> {
    let count = participants.len();
    let mut inputs = vec![l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::Block,
        direction,
        text_align: l::TextAlign::Start,
        text_align_last: l::TextAlignLast::Start,
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(60.0)).unwrap()),
            l::PreferredSizeOf::AUTO,
        ),
        ..l::NodeInputOf::default()
    })];
    inputs.extend(participants);
    let mut children = vec![vec![]; count + 1];
    children[0] = (1..=count).map(|id| u32::try_from(id).unwrap()).collect();
    Tree {
        inputs,
        non_box: l::NodeInputOf::non_box(),
        children,
    }
}
fn partition_tree<S: l::LayoutScalar>(
    direction: l::Direction,
    groups: usize,
    permission: l::InlineBreakOpportunity,
) -> Tree<S> {
    let mut participants = vec![];
    for group in 0..groups {
        for width in [30.0, 30.0, 10.0] {
            participants.push(numeric_atomic(direction, width, 10.0, permission));
        }
        if group + 1 < groups {
            // Genuine numerical control, matching the atoms' actual bottom10
            // fallback baseline; no font or fabricated Text strut participates.
            participants.push(numeric_forced(direction, 10.0, 10.0));
        }
    }
    numeric_tree(direction, participants)
}
fn numeric_compute<S: l::LayoutScalar>(
    tree: &Tree<S>,
    available: l::AvailableOf<S>,
    changed: &[u32],
) -> l::CompletedLayoutBatchOf<u32, S> {
    let request =
        l::LayoutRootRequestOf::viewport(l::Size::new(available, l::AvailableOf::MAX_CONTENT))
            .unwrap();
    let batch = l::compute_layout_invalidated(tree, 0, request, changed).unwrap();
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    if !changed.is_empty() {
        assert!(batch.invalidated_nodes().contains(&0));
    }
    batch
}
fn numeric_style<S: l::LayoutScalar>(
    tree: &mut Tree<S>,
    style: l::TextWrapStyle,
    available: l::AvailableOf<S>,
) -> l::CompletedLayoutBatchOf<u32, S> {
    let old = tree.inputs[0].as_box().cloned().unwrap();
    tree.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
        text_wrap_style: style,
        ..old
    });
    assert_eq!(tree.inputs[0].as_box().unwrap().text_wrap_style, style);
    numeric_compute(tree, available, &[0])
}
fn numeric_node<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    id: u32,
) -> l::NodeOutputOf<S> {
    entries
        .iter()
        .find(|entry| entry.node() == id)
        .unwrap()
        .output()
}
fn numeric_equal<S: l::LayoutScalar>(
    a: &l::CompletedLayoutBatchOf<u32, S>,
    b: &l::CompletedLayoutBatchOf<u32, S>,
    count: usize,
) {
    for (left, right) in [
        (a.unrounded_entries(), b.unrounded_entries()),
        (a.final_entries(), b.final_entries()),
    ] {
        for id in 0..count {
            assert_eq!(
                numeric_node(left, id as u32),
                numeric_node(right, id as u32)
            );
        }
    }
}
fn assert_row<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    ids: &[u32],
    direction: l::Direction,
    y: f64,
    width: f64,
) {
    let actual = ids
        .iter()
        .map(|&id| numeric_node(entries, id))
        .collect::<Vec<_>>();
    assert!(actual.iter().all(|item| item.location.y == S::from_f64(y)));
    assert_eq!(
        actual
            .iter()
            .fold(S::ZERO, |sum, item| sum + item.size.width),
        S::from_f64(width)
    );
    let left = actual
        .iter()
        .map(|item| item.location.x)
        .reduce(S::min)
        .unwrap();
    let right = actual
        .iter()
        .map(|item| item.location.x + item.size.width)
        .reduce(S::max)
        .unwrap();
    let start = if direction == l::Direction::Ltr {
        0.0
    } else {
        60.0 - width
    };
    assert_eq!(left, S::from_f64(start));
    assert_eq!(right, S::from_f64(start + width));
}
fn assert_partition<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    direction: l::Direction,
    groups: usize,
    balanced: bool,
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(
            numeric_node(entries, 0).size,
            l::Size::new(S::from_f64(60.0), S::from_f64(20.0 * groups as f64))
        );
        for group in 0..groups {
            let first = u32::try_from(1 + 4 * group).unwrap();
            let top = 20.0 * group as f64;
            for (id, width) in [(first, 30.0), (first + 1, 30.0), (first + 2, 10.0)] {
                let actual = numeric_node(entries, id);
                assert_eq!(actual.source_index, l::SourceIndex::new((id - 1) as usize));
                assert_eq!(
                    actual.size,
                    l::Size::new(S::from_f64(width), S::from_f64(10.0))
                );
            }
            if balanced {
                assert_row(entries, &[first], direction, top, 30.0);
                assert_row(
                    entries,
                    &[first + 1, first + 2],
                    direction,
                    top + 10.0,
                    40.0,
                );
            } else {
                assert_row(entries, &[first, first + 1], direction, top, 60.0);
                assert_row(entries, &[first + 2], direction, top + 10.0, 10.0);
            }
            if group + 1 < groups {
                let control = numeric_node(entries, first + 3);
                assert_eq!(
                    control.source_index,
                    l::SourceIndex::new((first + 2) as usize)
                );
                assert_eq!(control.size, l::Size::ZERO);
                assert_eq!(control.location.y, S::from_f64(top + 20.0));
            }
        }
    }
}
fn numeric_partition_transition<S: l::LayoutScalar>(direction: l::Direction, groups: usize) {
    let mut tree = partition_tree::<S>(direction, groups, l::InlineBreakOpportunity::Allowed);
    assert_eq!(
        tree.inputs[0].as_box().unwrap().text_wrap_style,
        l::TextWrapStyle::Auto
    );
    let capacity = l::AvailableOf::definite(S::from_f64(60.0));
    let auto = numeric_compute(&tree, capacity, &[]);
    assert_partition(&auto, direction, groups, false);
    let balanced = numeric_style(&mut tree, l::TextWrapStyle::Balance, capacity);
    assert_partition(&balanced, direction, groups, true);
    let replay = numeric_compute(&tree, capacity, &[]);
    assert_partition(&replay, direction, groups, true);
    numeric_equal(&balanced, &replay, tree.inputs.len());
    let restored = numeric_style(&mut tree, l::TextWrapStyle::Auto, capacity);
    assert_partition(&restored, direction, groups, false);
    numeric_equal(&auto, &restored, tree.inputs.len());
    assert_partition(&balanced, direction, groups, true);
    assert_partition(&auto, direction, groups, false);
}
#[test]
fn balance_numeric_atoms_use_thirty_then_forty_without_text_proxy() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        numeric_partition_transition::<f32>(direction, 1);
        numeric_partition_transition::<f64>(direction, 1);
    }
}
#[test]
fn balance_numeric_groups_keep_their_real_forced_control_and_count() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        numeric_partition_transition::<f32>(direction, 2);
        numeric_partition_transition::<f64>(direction, 2);
    }
}

fn prohibited_transition<S: l::LayoutScalar>(direction: l::Direction) {
    let mut tree = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Prohibited);
    let capacity = l::AvailableOf::definite(S::from_f64(60.0));
    let auto = numeric_compute(&tree, capacity, &[]);
    let balanced = numeric_style(&mut tree, l::TextWrapStyle::Balance, capacity);
    let restored = numeric_style(&mut tree, l::TextWrapStyle::Auto, capacity);
    for batch in [&auto, &balanced, &restored] {
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                numeric_node(entries, 0).size,
                l::Size::new(S::from_f64(60.0), S::from_f64(10.0))
            );
            for (id, width) in [(1, 30.0), (2, 30.0), (3, 10.0)] {
                let actual = numeric_node(entries, id);
                assert_eq!(actual.source_index, l::SourceIndex::new((id - 1) as usize));
                assert_eq!(
                    actual.size,
                    l::Size::new(S::from_f64(width), S::from_f64(10.0))
                );
                assert_eq!(actual.location.y, S::ZERO);
            }
        }
    }
    numeric_equal(&auto, &balanced, tree.inputs.len());
    numeric_equal(&auto, &restored, tree.inputs.len());
}
#[test]
fn balance_does_not_create_breaks_between_prohibited_numeric_edges() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        prohibited_transition::<f32>(direction);
        prohibited_transition::<f64>(direction);
    }
}

fn numeric_custody<S: l::LayoutScalar>(direction: l::Direction) {
    let cases = [
        (vec![], 0.0),
        (
            vec![
                numeric_boundary(direction, l::InlineBoundaryKind::Start),
                numeric_boundary(direction, l::InlineBoundaryKind::End),
            ],
            0.0,
        ),
        (
            vec![
                numeric_boundary(direction, l::InlineBoundaryKind::Start),
                numeric_atomic(direction, 0.0, 0.0, l::InlineBreakOpportunity::Allowed),
                numeric_boundary(direction, l::InlineBoundaryKind::End),
            ],
            10.0,
        ),
        (vec![numeric_forced(direction, 16.0, 20.0)], 20.0),
    ];
    for (case, (participants, height)) in cases.into_iter().enumerate() {
        let mut tree = numeric_tree::<S>(direction, participants);
        let capacity = l::AvailableOf::definite(S::from_f64(60.0));
        let auto = numeric_compute(&tree, capacity, &[]);
        let balanced = numeric_style(&mut tree, l::TextWrapStyle::Balance, capacity);
        let restored = numeric_style(&mut tree, l::TextWrapStyle::Auto, capacity);
        for batch in [&auto, &balanced, &restored] {
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(
                    numeric_node(entries, 0).size,
                    l::Size::new(S::from_f64(60.0), S::from_f64(height))
                );
                for id in 1..tree.inputs.len() {
                    let actual = numeric_node(entries, id as u32);
                    assert_eq!(actual.source_index, l::SourceIndex::new(id - 1));
                    assert_eq!(actual.size, l::Size::ZERO);
                    let y = match case {
                        1 => 0.0,
                        2 => 8.0,
                        3 => 16.0,
                        _ => unreachable!(),
                    };
                    assert_eq!(actual.location.y, S::from_f64(y));
                }
            }
        }
        numeric_equal(&auto, &balanced, tree.inputs.len());
        numeric_equal(&auto, &restored, tree.inputs.len());
    }
}
#[test]
fn balance_keeps_real_zero_atomic_and_forced_content_but_not_phantom_metric_lines() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        numeric_custody::<f32>(direction);
        numeric_custody::<f64>(direction);
    }
}

fn numeric_intrinsic<S: l::LayoutScalar>(direction: l::Direction) {
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
        let balanced = numeric_style(&mut tree, l::TextWrapStyle::Balance, available);
        let restored = numeric_style(&mut tree, l::TextWrapStyle::Auto, available);
        for batch in [&auto, &balanced, &restored] {
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
                    assert_eq!(actual.source_index, l::SourceIndex::new(i));
                    assert_eq!(
                        actual.size,
                        l::Size::new(S::from_f64(width), S::from_f64(10.0))
                    );
                    assert_eq!(
                        actual.location.y,
                        if minimum {
                            S::from_f64(10.0 * i as f64)
                        } else {
                            S::ZERO
                        }
                    );
                }
            }
        }
        numeric_equal(&auto, &balanced, tree.inputs.len());
        numeric_equal(&auto, &restored, tree.inputs.len());
    }
}
#[test]
fn balance_leaves_true_auto_width_numeric_min_and_max_content_unchanged() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        numeric_intrinsic::<f32>(direction);
        numeric_intrinsic::<f64>(direction);
    }
}

mod avoid_short_last_line;
