//! Numeric native groups: literal alignment oracles, no Text preparation proxy.
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::supplied_line::BidiLevel;

#[derive(Clone)]
struct Tree<S: l::scalar::LayoutScalar> {
    inputs: Vec<l::node::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::node::NodeInputOf<S>,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for Tree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for Tree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}
fn n<S: l::scalar::LayoutScalar>(x: f64) -> S {
    S::from_f64(x)
}
fn px<S: l::scalar::LayoutScalar>(x: f64) -> l::sizing::PreferredSizeOf<S> {
    l::sizing::PreferredSizeOf::value(l::length::LengthPercentageOf::px(n(x)).unwrap())
}
fn size<S: l::scalar::LayoutScalar>(
    mode: l::geometry::WritingMode,
    inline: f64,
    block: f64,
) -> l::geometry::Size<S> {
    match mode {
        l::geometry::WritingMode::HorizontalTb => l::geometry::Size::new(n(inline), n(block)),
        _ => l::geometry::Size::new(n(block), n(inline)),
    }
}
fn block<S: l::scalar::LayoutScalar>(
    mode: l::geometry::WritingMode,
    direction: l::geometry::Direction,
    width: f64,
) -> l::node::NodeInputOf<S> {
    l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        writing_mode: mode,
        direction,
        text_align: l::text_flow::TextAlign::Start,
        text_align_last: l::text_flow::TextAlignLast::Start,
        size: if mode == l::geometry::WritingMode::HorizontalTb {
            l::geometry::Size::new(px(width), l::sizing::PreferredSizeOf::AUTO)
        } else {
            l::geometry::Size::new(l::sizing::PreferredSizeOf::AUTO, px(width))
        },
        ..l::node::NodeInputOf::default()
    }
}
fn atomic<S: l::scalar::LayoutScalar>(
    mode: l::geometry::WritingMode,
    direction: l::geometry::Direction,
    inline: f64,
) -> l::node::LayoutInputOf<S> {
    let physical = size::<S>(mode, inline, 10.0);
    l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        display: l::box_model::Display::InlineBlock,
        writing_mode: mode,
        direction,
        vertical_align: l::inline::VerticalAlignOf::Baseline,
        atomic_inline_participation: Some(l::inline::AtomicInlineParticipation::new(
            BidiLevel::try_new(u8::from(direction == l::geometry::Direction::Rtl)).unwrap(),
            l::text_flow::InlineBreakOpportunity::Allowed,
        )),
        size: l::geometry::Size::new(
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(physical.width).unwrap(),
            ),
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(physical.height).unwrap(),
            ),
        ),
        ..l::node::NodeInputOf::default()
    })
}
fn forced<S: l::scalar::LayoutScalar>(
    mode: l::geometry::WritingMode,
    direction: l::geometry::Direction,
) -> l::node::LayoutInputOf<S> {
    l::node::LayoutInputOf::line_break(
        l::text_flow::LineBreakInputOf::new()
            .with_writing_mode(mode)
            .with_direction(direction)
            .with_metrics(
                l::inline::InlineMetricsOf::try_new(
                    surgeist_layout::baseline::InlineBaselineContext::Physical,
                    n(10.0),
                    n(10.0),
                )
                .unwrap(),
            ),
    )
}
impl<S: l::scalar::LayoutScalar> Tree<S> {
    fn new(inputs: Vec<l::node::LayoutInputOf<S>>, edges: &[(u32, &[u32])]) -> Self {
        let mut children = vec![vec![]; inputs.len()];
        for &(node, list) in edges {
            children[node as usize] = list.to_vec();
        }
        Self {
            inputs,
            children,
            non_box: l::node::NodeInputOf::non_box(),
        }
    }
    fn update(&mut self, node: u32, f: impl FnOnce(&mut l::node::NodeInputOf<S>)) {
        let mut input = self.inputs[node as usize].as_box().unwrap().clone();
        f(&mut input);
        self.inputs[node as usize] = l::node::LayoutInputOf::box_input(input);
    }
    fn compute(
        &self,
        mode: l::geometry::WritingMode,
        changed: &[u32],
    ) -> l::batch::CompletedLayoutBatchOf<u32, S> {
        let available = if mode == l::geometry::WritingMode::HorizontalTb {
            crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(Some(n(100.0)), None),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::Normal,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            )
        } else {
            crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(None, Some(n(100.0))),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::Normal,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            )
        };
        let request = {
            let sizing_request = available;
            l::request::LayoutRootRequestOf::standalone(
                sizing_request,
                surgeist_layout::geometry::FlowAxes::new(
                    surgeist_layout::tree::LayoutTree::node_input(self, 0).writing_mode,
                    surgeist_layout::tree::LayoutTree::node_input(self, 0).direction,
                ),
                sizing_request
                    .orthogonal_fallback()
                    .initial_containing_block(),
            )
        };
        let batch = l::compute_layout_invalidated(self, 0, request, changed).unwrap();
        numeric(&batch);
        batch
    }
}
fn numeric<S: l::scalar::LayoutScalar>(batch: &l::batch::CompletedLayoutBatchOf<u32, S>) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}
fn node<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    id: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, id, crate::layout::observation::Kind::Box)
}
fn node_of_kind<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    id: u32,
    expected_kind: crate::layout::observation::Kind,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, id, expected_kind)
}
// Independent physical rectangles from the five writing-mode definitions.
// The argument u is measured from LINE-LEFT, not logical inline-start. This
// deliberately does not call FlowAxes or any production alignment helper.
fn point<S: l::scalar::LayoutScalar>(
    mode: l::geometry::WritingMode,
    u: f64,
    b: f64,
    inline: f64,
    block: f64,
    total_block: f64,
) -> l::geometry::Point<S> {
    match mode {
        l::geometry::WritingMode::HorizontalTb => l::geometry::Point::new(n(u), n(b)),
        l::geometry::WritingMode::VerticalRl | l::geometry::WritingMode::SidewaysRl => {
            l::geometry::Point::new(n(total_block - b - block), n(u))
        }
        l::geometry::WritingMode::VerticalLr => l::geometry::Point::new(n(b), n(u)),
        l::geometry::WritingMode::SidewaysLr => {
            l::geometry::Point::new(n(b), n(100.0 - u - inline))
        }
    }
}
fn two_rows<S: l::scalar::LayoutScalar>(
    mode: l::geometry::WritingMode,
    direction: l::geometry::Direction,
    group: l::text_flow::TextGroupAlign,
    align: l::text_flow::TextAlign,
) -> Tree<S> {
    let mut root = block(mode, direction, 100.0);
    root.text_group_align = group;
    root.text_align = align;
    root.text_align_last = match align {
        l::text_flow::TextAlign::Center => l::text_flow::TextAlignLast::Center,
        l::text_flow::TextAlign::End => l::text_flow::TextAlignLast::End,
        l::text_flow::TextAlign::Justify => l::text_flow::TextAlignLast::Justify,
        _ => l::text_flow::TextAlignLast::Start,
    };
    Tree::new(
        vec![
            l::node::LayoutInputOf::box_input(root),
            atomic(mode, direction, 60.0),
            forced(mode, direction),
            atomic(mode, direction, 40.0),
        ],
        &[(0, &[1, 2, 3])],
    )
}
fn assert_two_rows<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    mode: l::geometry::WritingMode,
    direction: l::geometry::Direction,
    starts: [f64; 2],
) {
    numeric(batch);
    for entries in [
        crate::layout::observation::phase(batch, false),
        crate::layout::observation::phase(batch, true),
    ] {
        assert_eq!(node(entries, 0).size, size(mode, 100.0, 20.0));
        for (id, width, top, start, source) in
            [(1, 60.0, 0.0, starts[0], 0), (3, 40.0, 10.0, starts[1], 2)]
        {
            let item = node(entries, id);
            assert_eq!(item.source_index, l::node::SourceIndex::new(source));
            assert_eq!(item.size, size(mode, width, 10.0));
            assert_eq!(item.location, point(mode, start, top, width, 10.0, 20.0));
            assert_eq!(item.margin, l::geometry::Edges::all(S::ZERO));
            assert_eq!(item.border, l::geometry::Edges::all(S::ZERO));
            assert_eq!(item.padding, l::geometry::Edges::all(S::ZERO));
        }
        let br = node_of_kind(
            entries,
            2,
            crate::layout::observation::Kind::NonBox(l::fragmentation::NonBoxKind::LineBreak),
        );
        assert_eq!(br.source_index, l::node::SourceIndex::new(1));
        assert_eq!(br.size, l::geometry::Size::ZERO);
        let end = starts[0]
            + if direction == l::geometry::Direction::Ltr {
                60.0
            } else {
                0.0
            };
        assert_eq!(br.location, point(mode, end, 10.0, 0.0, 0.0, 20.0));
    }
}
fn matrix<S: l::scalar::LayoutScalar>() {
    assert_eq!(
        l::text_flow::TextGroupAlign::default(),
        l::text_flow::TextGroupAlign::None
    );
    assert_eq!(
        l::node::NodeInputOf::<S>::default().text_group_align,
        l::text_flow::TextGroupAlign::None
    );
    for mode in [
        l::geometry::WritingMode::HorizontalTb,
        l::geometry::WritingMode::VerticalRl,
        l::geometry::WritingMode::VerticalLr,
        l::geometry::WritingMode::SidewaysRl,
        l::geometry::WritingMode::SidewaysLr,
    ] {
        for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
            let rtl = direction == l::geometry::Direction::Rtl;
            for (group, align, starts) in [
                (
                    l::text_flow::TextGroupAlign::None,
                    l::text_flow::TextAlign::Center,
                    [20.0, 30.0],
                ),
                (
                    l::text_flow::TextGroupAlign::Left,
                    l::text_flow::TextAlign::Center,
                    [0.0, 10.0],
                ),
                (
                    l::text_flow::TextGroupAlign::Right,
                    l::text_flow::TextAlign::Center,
                    [40.0, 50.0],
                ),
                // Numeric Justify has no spacing slots. Its actual
                // unexpandable Center fallback uses the reduced frame and
                // leaves the 60/40 atomics unexpanded.
                (
                    l::text_flow::TextGroupAlign::Left,
                    l::text_flow::TextAlign::Justify,
                    [0.0, 10.0],
                ),
                (
                    l::text_flow::TextGroupAlign::Right,
                    l::text_flow::TextAlign::Justify,
                    [40.0, 50.0],
                ),
                (
                    l::text_flow::TextGroupAlign::Center,
                    l::text_flow::TextAlign::Start,
                    if rtl { [20.0, 40.0] } else { [20.0, 20.0] },
                ),
                (
                    l::text_flow::TextGroupAlign::Center,
                    l::text_flow::TextAlign::End,
                    if rtl { [20.0, 20.0] } else { [20.0, 40.0] },
                ),
                (
                    l::text_flow::TextGroupAlign::Start,
                    l::text_flow::TextAlign::Start,
                    if rtl { [40.0, 60.0] } else { [0.0, 0.0] },
                ),
                (
                    l::text_flow::TextGroupAlign::End,
                    l::text_flow::TextAlign::Start,
                    if rtl { [0.0, 20.0] } else { [40.0, 40.0] },
                ),
            ] {
                let tree = two_rows::<S>(mode, direction, group, align);
                let first = tree.compute(mode, &[]);
                assert_two_rows(&first, mode, direction, starts);
                let replay = tree.compute(mode, &[]);
                assert_two_rows(&replay, mode, direction, starts);
                assert_eq!(
                    first
                        .node_layouts()
                        .map(|(node, view)| (
                            node,
                            match view {
                                surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(
                                    view,
                                ) => view.unrounded_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                                    view.unrounded_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                                    panic!("continuous fixture unexpectedly fragmented"),
                            }
                        ))
                        .collect::<Vec<_>>(),
                    replay
                        .node_layouts()
                        .map(|(node, view)| (
                            node,
                            match view {
                                surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(
                                    view,
                                ) => view.unrounded_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                                    view.unrounded_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                                    panic!("continuous fixture unexpectedly fragmented"),
                            }
                        ))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    first
                        .node_layouts()
                        .map(|(node, view)| (
                            node,
                            match view {
                                surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(
                                    view,
                                ) => view.final_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                                    view.final_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                                    panic!("continuous fixture unexpectedly fragmented"),
                            }
                        ))
                        .collect::<Vec<_>>(),
                    replay
                        .node_layouts()
                        .map(|(node, view)| (
                            node,
                            match view {
                                surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(
                                    view,
                                ) => view.final_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                                    view.final_entry(),
                                surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                                    panic!("continuous fixture unexpectedly fragmented"),
                            }
                        ))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}
#[test]
fn numeric_group_matrix_preserves_rows_controls_and_axes() {
    matrix::<f32>();
    matrix::<f64>();
}

fn ordered_slots<S: l::scalar::LayoutScalar>() {
    let mode = l::geometry::WritingMode::HorizontalTb;
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        let mut root = block(mode, direction, 100.0);
        root.text_group_align = l::text_flow::TextGroupAlign::Center;
        let tree = Tree::<S>::new(
            vec![
                l::node::LayoutInputOf::box_input(root),
                atomic(mode, direction, 30.0),
                atomic(mode, direction, 20.0),
                atomic(mode, direction, 10.0),
                forced(mode, direction),
                atomic(mode, direction, 25.0),
                atomic(mode, direction, 15.0),
            ],
            &[(0, &[1, 2, 3, 4, 5, 6])],
        );
        let batch = tree.compute(mode, &[]);
        // Real resolved0/1 levels preserve each atomic's public native identity
        // and physical visual order. Group padding shifts the existing slots;
        // it cannot reorder boxes or change which real Forced row owns them.
        let xs = if direction == l::geometry::Direction::Ltr {
            [20.0, 50.0, 70.0, 20.0, 45.0]
        } else {
            [50.0, 30.0, 20.0, 55.0, 40.0]
        };
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            for ((id, width, y), x) in [
                (1, 30.0, 0.0),
                (2, 20.0, 0.0),
                (3, 10.0, 0.0),
                (5, 25.0, 10.0),
                (6, 15.0, 10.0),
            ]
            .into_iter()
            .zip(xs)
            {
                let item = node_of_kind(
                    entries,
                    id,
                    crate::layout::observation::expected_kind(
                        l::tree::LayoutTree::layout_input(&tree, id),
                        false,
                    ),
                );
                assert_eq!(
                    item.source_index,
                    l::node::SourceIndex::new((id - 1) as usize)
                );
                assert_eq!(item.location, l::geometry::Point::new(n(x), n(y)));
                assert_eq!(item.size, l::geometry::Size::new(n(width), n(10.0)));
            }
            assert_eq!(
                node_of_kind(
                    entries,
                    4,
                    crate::layout::observation::Kind::NonBox(
                        l::fragmentation::NonBoxKind::LineBreak
                    )
                )
                .source_index,
                l::node::SourceIndex::new(3)
            );
            assert_eq!(
                node_of_kind(
                    entries,
                    4,
                    crate::layout::observation::Kind::NonBox(
                        l::fragmentation::NonBoxKind::LineBreak
                    )
                )
                .size,
                l::geometry::Size::ZERO
            );
            assert_eq!(
                node(entries, 0).size,
                l::geometry::Size::new(n(100.0), n(20.0))
            );
        }
    }
}
#[test]
fn source_resolved_atomic_slots_keep_source_identity_and_visual_order() {
    ordered_slots::<f32>();
    ordered_slots::<f64>();
}

mod cache;
mod edges;
mod membership;
