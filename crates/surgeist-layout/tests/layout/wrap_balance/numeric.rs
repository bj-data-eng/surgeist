//! Real numeric participants exercise the same normalized selection policy.
mod stable;

use super::*;

pub(super) fn numeric_atomic<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    width: f64,
    height: f64,
    following: l::text_flow::InlineBreakOpportunity,
) -> l::node::LayoutInputOf<S> {
    l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        display: l::box_model::Display::InlineBlock,
        direction,
        vertical_align: l::inline::VerticalAlignOf::Baseline,
        atomic_inline_participation: Some(l::inline::AtomicInlineParticipation::new(
            t::supplied_line::BidiLevel::try_new(if direction == l::geometry::Direction::Ltr {
                0
            } else {
                1
            })
            .unwrap(),
            following,
        )),
        size: l::geometry::Size::new(
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
            ),
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(height)).unwrap(),
            ),
        ),
        ..l::node::NodeInputOf::default()
    })
}
pub(super) fn numeric_forced<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    baseline: f64,
    height: f64,
) -> l::node::LayoutInputOf<S> {
    l::node::LayoutInputOf::line_break(
        l::text_flow::LineBreakInputOf::new()
            .with_direction(direction)
            .with_metrics(
                l::inline::InlineMetricsOf::try_new(
                    surgeist_layout::baseline::InlineBaselineContext::Physical,
                    S::from_f64(baseline),
                    S::from_f64(height),
                )
                .unwrap(),
            ),
    )
}
fn numeric_boundary<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    kind: l::inline::InlineBoundaryKind,
) -> l::node::LayoutInputOf<S> {
    l::node::LayoutInputOf::inline_boundary(
        l::inline::InlineBoundaryInputOf::contribution(
            kind,
            l::inline::InlineMetricsOf::try_new(
                surgeist_layout::baseline::InlineBaselineContext::Physical,
                S::from_f64(8.0),
                S::from_f64(10.0),
            )
            .unwrap(),
            l::inline::InlineBoundaryEdgeOf::ZERO,
        )
        .unwrap()
        .with_direction(direction),
    )
}
pub(super) fn numeric_tree<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    participants: Vec<l::node::LayoutInputOf<S>>,
) -> Tree<S> {
    let count = participants.len();
    let mut inputs = vec![l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        direction,
        text_align: l::text_flow::TextAlign::Start,
        text_align_last: l::text_flow::TextAlignLast::Start,
        size: l::geometry::Size::new(
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
            ),
            l::sizing::PreferredSizeOf::AUTO,
        ),
        ..l::node::NodeInputOf::default()
    })];
    inputs.extend(participants);
    let mut children = vec![vec![]; count + 1];
    children[0] = (1..=count).map(|id| u32::try_from(id).unwrap()).collect();
    Tree {
        inputs,
        non_box: l::node::NodeInputOf::non_box(),
        children,
    }
}
fn partition_tree<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    groups: usize,
    permission: l::text_flow::InlineBreakOpportunity,
) -> Tree<S> {
    let mut participants = vec![];
    for group in 0..groups {
        for width in [30.0, 30.0, 10.0] {
            participants.push(numeric_atomic(direction, width, 10.0, permission));
        }
        if group + 1 < groups {
            // Numerical control, matching the atoms' actual bottom10
            // fallback baseline; no font or fabricated Text strut participates.
            participants.push(numeric_forced(direction, 10.0, 10.0));
        }
    }
    numeric_tree(direction, participants)
}
fn numeric_compute<S: l::scalar::LayoutScalar>(
    tree: &Tree<S>,
    available: surgeist_layout::constraint::SizingRequestOf<S>,
    changed: &[u32],
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let request = {
        let sizing_request = available;
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(tree, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(tree, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    };
    let batch = l::compute_layout_invalidated(tree, 0, request, changed).unwrap();
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    if !changed.is_empty() {
        assert!(batch.invalidated_nodes().contains(&0));
    }
    batch
}
fn numeric_style<S: l::scalar::LayoutScalar>(
    tree: &mut Tree<S>,
    style: l::text_flow::TextWrapStyle,
    available: surgeist_layout::constraint::SizingRequestOf<S>,
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let old = tree.inputs[0].as_box().cloned().unwrap();
    tree.inputs[0] = l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        text_wrap_style: style,
        ..old
    });
    assert_eq!(tree.inputs[0].as_box().unwrap().text_wrap_style, style);
    numeric_compute(tree, available, &[0])
}
pub(super) fn numeric_node<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    id: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, id, crate::layout::observation::Kind::Box)
}
fn numeric_node_of_kind<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    id: u32,
    expected_kind: crate::layout::observation::Kind,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, id, expected_kind)
}
fn numeric_equal<S: l::scalar::LayoutScalar>(
    a: &l::batch::CompletedLayoutBatchOf<u32, S>,
    b: &l::batch::CompletedLayoutBatchOf<u32, S>,
    fixture: &Tree<S>,
) {
    for (left, right) in [
        (
            crate::layout::observation::phase(a, false),
            crate::layout::observation::phase(b, false),
        ),
        (
            crate::layout::observation::phase(a, true),
            crate::layout::observation::phase(b, true),
        ),
    ] {
        for id in 0..fixture.inputs.len() {
            let kind =
                crate::layout::observation::expected_kind(fixture.inputs[id].as_view(), false);
            assert_eq!(
                numeric_node_of_kind(left, id as u32, kind),
                numeric_node_of_kind(right, id as u32, kind)
            );
        }
    }
}
fn assert_row<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    ids: &[u32],
    direction: l::geometry::Direction,
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
    let start = if direction == l::geometry::Direction::Ltr {
        0.0
    } else {
        60.0 - width
    };
    assert_eq!(left, S::from_f64(start));
    assert_eq!(right, S::from_f64(start + width));
}
fn assert_partition<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    direction: l::geometry::Direction,
    groups: usize,
    balanced: bool,
) {
    for entries in [
        crate::layout::observation::phase(batch, false),
        crate::layout::observation::phase(batch, true),
    ] {
        assert_eq!(
            numeric_node(entries, 0).size,
            l::geometry::Size::new(S::from_f64(60.0), S::from_f64(20.0 * groups as f64))
        );
        for group in 0..groups {
            let first = u32::try_from(1 + 4 * group).unwrap();
            let top = 20.0 * group as f64;
            for (id, width) in [(first, 30.0), (first + 1, 30.0), (first + 2, 10.0)] {
                let actual = numeric_node(entries, id);
                assert_eq!(
                    actual.source_index,
                    l::node::SourceIndex::new((id - 1) as usize)
                );
                assert_eq!(
                    actual.size,
                    l::geometry::Size::new(S::from_f64(width), S::from_f64(10.0))
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
                let control = numeric_node_of_kind(
                    entries,
                    first + 3,
                    crate::layout::observation::Kind::NonBox(
                        l::fragmentation::NonBoxKind::LineBreak,
                    ),
                );
                assert_eq!(
                    control.source_index,
                    l::node::SourceIndex::new((first + 2) as usize)
                );
                assert_eq!(control.size, l::geometry::Size::ZERO);
                assert_eq!(control.location.y, S::from_f64(top + 20.0));
            }
        }
    }
}
fn numeric_partition_transition<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    groups: usize,
) {
    let mut tree = partition_tree::<S>(
        direction,
        groups,
        l::text_flow::InlineBreakOpportunity::Allowed,
    );
    assert_eq!(
        tree.inputs[0].as_box().unwrap().text_wrap_style,
        l::text_flow::TextWrapStyle::Auto
    );
    let capacity = crate::layout::native_request::sizing(
        surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
        surgeist_layout::geometry::Size::new(
            surgeist_layout::constraint::SizingOperation::Normal,
            surgeist_layout::constraint::SizingOperation::Normal,
        ),
    );
    let auto = numeric_compute(&tree, capacity, &[]);
    assert_partition(&auto, direction, groups, false);
    let balanced = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Balance, capacity);
    assert_partition(&balanced, direction, groups, true);
    let replay = numeric_compute(&tree, capacity, &[]);
    assert_partition(&replay, direction, groups, true);
    numeric_equal(&balanced, &replay, &tree);
    let restored = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Auto, capacity);
    assert_partition(&restored, direction, groups, false);
    numeric_equal(&auto, &restored, &tree);
    assert_partition(&balanced, direction, groups, true);
    assert_partition(&auto, direction, groups, false);
}
#[test]
fn balance_numeric_atoms_use_thirty_then_forty_without_text_proxy() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        numeric_partition_transition::<f32>(direction, 1);
        numeric_partition_transition::<f64>(direction, 1);
    }
}
#[test]
fn balance_numeric_groups_keep_their_real_forced_control_and_count() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        numeric_partition_transition::<f32>(direction, 2);
        numeric_partition_transition::<f64>(direction, 2);
    }
}

fn prohibited_transition<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    let mut tree = partition_tree::<S>(
        direction,
        1,
        l::text_flow::InlineBreakOpportunity::Prohibited,
    );
    let capacity = crate::layout::native_request::sizing(
        surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
        surgeist_layout::geometry::Size::new(
            surgeist_layout::constraint::SizingOperation::Normal,
            surgeist_layout::constraint::SizingOperation::Normal,
        ),
    );
    let auto = numeric_compute(&tree, capacity, &[]);
    let balanced = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Balance, capacity);
    let restored = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Auto, capacity);
    for batch in [&auto, &balanced, &restored] {
        for entries in [
            crate::layout::observation::phase(batch, false),
            crate::layout::observation::phase(batch, true),
        ] {
            assert_eq!(
                numeric_node(entries, 0).size,
                l::geometry::Size::new(S::from_f64(60.0), S::from_f64(10.0))
            );
            for (id, width) in [(1, 30.0), (2, 30.0), (3, 10.0)] {
                let actual = numeric_node_of_kind(
                    entries,
                    id,
                    crate::layout::observation::expected_kind(
                        l::tree::LayoutTree::layout_input(&tree, id),
                        false,
                    ),
                );
                assert_eq!(
                    actual.source_index,
                    l::node::SourceIndex::new((id - 1) as usize)
                );
                assert_eq!(
                    actual.size,
                    l::geometry::Size::new(S::from_f64(width), S::from_f64(10.0))
                );
                assert_eq!(actual.location.y, S::ZERO);
            }
        }
    }
    numeric_equal(&auto, &balanced, &tree);
    numeric_equal(&auto, &restored, &tree);
}
#[test]
fn balance_does_not_create_breaks_between_prohibited_numeric_edges() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        prohibited_transition::<f32>(direction);
        prohibited_transition::<f64>(direction);
    }
}

fn numeric_custody<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    let cases = [
        (vec![], 0.0),
        (
            vec![
                numeric_boundary(direction, l::inline::InlineBoundaryKind::Start),
                numeric_boundary(direction, l::inline::InlineBoundaryKind::End),
            ],
            0.0,
        ),
        (
            vec![
                numeric_boundary(direction, l::inline::InlineBoundaryKind::Start),
                numeric_atomic(
                    direction,
                    0.0,
                    0.0,
                    l::text_flow::InlineBreakOpportunity::Allowed,
                ),
                numeric_boundary(direction, l::inline::InlineBoundaryKind::End),
            ],
            10.0,
        ),
        (vec![numeric_forced(direction, 16.0, 20.0)], 20.0),
    ];
    for (case, (participants, height)) in cases.into_iter().enumerate() {
        let mut tree = numeric_tree::<S>(direction, participants);
        let capacity = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        let auto = numeric_compute(&tree, capacity, &[]);
        let balanced = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Balance, capacity);
        let restored = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Auto, capacity);
        for batch in [&auto, &balanced, &restored] {
            for entries in [
                crate::layout::observation::phase(batch, false),
                crate::layout::observation::phase(batch, true),
            ] {
                assert_eq!(
                    numeric_node(entries, 0).size,
                    l::geometry::Size::new(S::from_f64(60.0), S::from_f64(height))
                );
                for id in 1..tree.inputs.len() {
                    let actual = numeric_node_of_kind(
                        entries,
                        id as u32,
                        crate::layout::observation::expected_kind(
                            l::tree::LayoutTree::layout_input(&tree, id as u32),
                            false,
                        ),
                    );
                    assert_eq!(actual.source_index, l::node::SourceIndex::new(id - 1));
                    assert_eq!(actual.size, l::geometry::Size::ZERO);
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
        numeric_equal(&auto, &balanced, &tree);
        numeric_equal(&auto, &restored, &tree);
    }
}
#[test]
fn balance_keeps_real_zero_atomic_and_forced_content_but_not_phantom_metric_lines() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        numeric_custody::<f32>(direction);
        numeric_custody::<f64>(direction);
    }
}

fn numeric_intrinsic<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    for minimum in [true, false] {
        let mut tree =
            partition_tree::<S>(direction, 1, l::text_flow::InlineBreakOpportunity::Allowed);
        let old = tree.inputs[0].as_box().cloned().unwrap();
        tree.inputs[0] = l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
            size: l::geometry::Size::new(l::sizing::PreferredSizeOf::AUTO, old.size.height),
            ..old
        });
        let available = if minimum {
            crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(None, None),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::MinContent,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            )
        } else {
            crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(None, None),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::MaxContent,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            )
        };
        let auto = numeric_compute(&tree, available, &[]);
        let balanced = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Balance, available);
        let restored = numeric_style(&mut tree, l::text_flow::TextWrapStyle::Auto, available);
        for batch in [&auto, &balanced, &restored] {
            for entries in [
                crate::layout::observation::phase(batch, false),
                crate::layout::observation::phase(batch, true),
            ] {
                assert_eq!(
                    numeric_node(entries, 0).size,
                    l::geometry::Size::new(
                        S::from_f64(if minimum { 30.0 } else { 70.0 }),
                        S::from_f64(if minimum { 30.0 } else { 10.0 })
                    )
                );
                for (i, width) in [30.0, 30.0, 10.0].into_iter().enumerate() {
                    let actual = numeric_node_of_kind(
                        entries,
                        (i + 1) as u32,
                        crate::layout::observation::expected_kind(
                            l::tree::LayoutTree::layout_input(&tree, (i + 1) as u32),
                            false,
                        ),
                    );
                    assert_eq!(actual.source_index, l::node::SourceIndex::new(i));
                    assert_eq!(
                        actual.size,
                        l::geometry::Size::new(S::from_f64(width), S::from_f64(10.0))
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
        numeric_equal(&auto, &balanced, &tree);
        numeric_equal(&auto, &restored, &tree);
    }
}
#[test]
fn balance_leaves_true_auto_width_numeric_min_and_max_content_unchanged() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        numeric_intrinsic::<f32>(direction);
        numeric_intrinsic::<f64>(direction);
    }
}

mod avoid_short_last_line;

mod pretty;
