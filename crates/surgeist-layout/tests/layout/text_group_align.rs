//! Numeric native groups: literal alignment oracles, no Text preparation proxy.
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::BidiLevel;

#[derive(Clone)]
struct Tree<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::NodeInputOf<S>,
}
impl<S: l::LayoutScalar> l::Traverse for Tree<S> {
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
impl<S: l::LayoutScalar> l::LayoutTree for Tree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}
fn n<S: l::LayoutScalar>(x: f64) -> S {
    S::from_f64(x)
}
fn px<S: l::LayoutScalar>(x: f64) -> l::PreferredSizeOf<S> {
    l::PreferredSizeOf::value(l::LengthPercentageOf::px(n(x)).unwrap())
}
fn size<S: l::LayoutScalar>(mode: l::WritingMode, inline: f64, block: f64) -> l::Size<S> {
    match mode {
        l::WritingMode::HorizontalTb => l::Size::new(n(inline), n(block)),
        _ => l::Size::new(n(block), n(inline)),
    }
}
fn block<S: l::LayoutScalar>(
    mode: l::WritingMode,
    direction: l::Direction,
    width: f64,
) -> l::NodeInputOf<S> {
    l::NodeInputOf {
        display: l::Display::Block,
        writing_mode: mode,
        direction,
        text_align: l::TextAlign::Start,
        text_align_last: l::TextAlignLast::Start,
        size: if mode == l::WritingMode::HorizontalTb {
            l::Size::new(px(width), l::PreferredSizeOf::AUTO)
        } else {
            l::Size::new(l::PreferredSizeOf::AUTO, px(width))
        },
        ..l::NodeInputOf::default()
    }
}
fn atomic<S: l::LayoutScalar>(
    mode: l::WritingMode,
    direction: l::Direction,
    inline: f64,
) -> l::LayoutInputOf<S> {
    let physical = size::<S>(mode, inline, 10.0);
    l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::InlineBlock,
        writing_mode: mode,
        direction,
        vertical_align: l::VerticalAlign::Baseline,
        atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
            BidiLevel::try_new(u8::from(direction == l::Direction::Rtl)).unwrap(),
            l::InlineBreakOpportunity::Allowed,
        )),
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(physical.width).unwrap()),
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(physical.height).unwrap()),
        ),
        ..l::NodeInputOf::default()
    })
}
fn forced<S: l::LayoutScalar>(
    mode: l::WritingMode,
    direction: l::Direction,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::line_break(
        l::LineBreakInputOf::new()
            .with_writing_mode(mode)
            .with_direction(direction)
            .with_metrics(
                l::InlineMetricsOf::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    n(10.0),
                    n(10.0),
                )
                .unwrap(),
            ),
    )
}
impl<S: l::LayoutScalar> Tree<S> {
    fn new(inputs: Vec<l::LayoutInputOf<S>>, edges: &[(u32, &[u32])]) -> Self {
        let mut children = vec![vec![]; inputs.len()];
        for &(node, list) in edges {
            children[node as usize] = list.to_vec();
        }
        Self {
            inputs,
            children,
            non_box: l::NodeInputOf::non_box(),
        }
    }
    fn update(&mut self, node: u32, f: impl FnOnce(&mut l::NodeInputOf<S>)) {
        let mut input = self.inputs[node as usize].as_box().unwrap().clone();
        f(&mut input);
        self.inputs[node as usize] = l::LayoutInputOf::box_input(input);
    }
    fn compute(&self, mode: l::WritingMode, changed: &[u32]) -> l::CompletedLayoutBatchOf<u32, S> {
        let available = if mode == l::WritingMode::HorizontalTb {
            l::Size::new(
                l::AvailableOf::definite(n(100.0)),
                l::AvailableOf::MAX_CONTENT,
            )
        } else {
            l::Size::new(
                l::AvailableOf::MAX_CONTENT,
                l::AvailableOf::definite(n(100.0)),
            )
        };
        let request = l::LayoutRootRequestOf::viewport(available).unwrap();
        let batch = l::compute_layout_invalidated(self, 0, request, changed).unwrap();
        numeric(&batch);
        batch
    }
}
fn numeric<S: l::LayoutScalar>(batch: &l::CompletedLayoutBatchOf<u32, S>) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}
fn node<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    id: u32,
) -> l::NodeOutputOf<S> {
    let mut rows = entries.iter().filter(|entry| entry.node() == id);
    let output = rows.next().unwrap().output();
    assert!(rows.next().is_none(), "one actual output per native node");
    output
}
// Independent physical rectangles from the five writing-mode definitions.
// The argument u is measured from LINE-LEFT, not logical inline-start. This
// deliberately does not call FlowAxes or any production alignment helper.
fn point<S: l::LayoutScalar>(
    mode: l::WritingMode,
    u: f64,
    b: f64,
    inline: f64,
    block: f64,
    total_block: f64,
) -> l::Point<S> {
    match mode {
        l::WritingMode::HorizontalTb => l::Point::new(n(u), n(b)),
        l::WritingMode::VerticalRl | l::WritingMode::SidewaysRl => {
            l::Point::new(n(total_block - b - block), n(u))
        }
        l::WritingMode::VerticalLr => l::Point::new(n(b), n(u)),
        l::WritingMode::SidewaysLr => l::Point::new(n(b), n(100.0 - u - inline)),
    }
}
fn two_rows<S: l::LayoutScalar>(
    mode: l::WritingMode,
    direction: l::Direction,
    group: l::TextGroupAlign,
    align: l::TextAlign,
) -> Tree<S> {
    let mut root = block(mode, direction, 100.0);
    root.text_group_align = group;
    root.text_align = align;
    root.text_align_last = match align {
        l::TextAlign::Center => l::TextAlignLast::Center,
        l::TextAlign::End => l::TextAlignLast::End,
        l::TextAlign::Justify => l::TextAlignLast::Justify,
        _ => l::TextAlignLast::Start,
    };
    Tree::new(
        vec![
            l::LayoutInputOf::box_input(root),
            atomic(mode, direction, 60.0),
            forced(mode, direction),
            atomic(mode, direction, 40.0),
        ],
        &[(0, &[1, 2, 3])],
    )
}
fn assert_two_rows<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    mode: l::WritingMode,
    direction: l::Direction,
    starts: [f64; 2],
) {
    numeric(batch);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 0).size, size(mode, 100.0, 20.0));
        for (id, width, top, start, source) in
            [(1, 60.0, 0.0, starts[0], 0), (3, 40.0, 10.0, starts[1], 2)]
        {
            let item = node(entries, id);
            assert_eq!(item.source_index, l::SourceIndex::new(source));
            assert_eq!(item.size, size(mode, width, 10.0));
            assert_eq!(item.location, point(mode, start, top, width, 10.0, 20.0));
            assert_eq!(item.margin, l::Edges::all(S::ZERO));
            assert_eq!(item.border, l::Edges::all(S::ZERO));
            assert_eq!(item.padding, l::Edges::all(S::ZERO));
        }
        let br = node(entries, 2);
        assert_eq!(br.source_index, l::SourceIndex::new(1));
        assert_eq!(br.size, l::Size::ZERO);
        let end = starts[0]
            + if direction == l::Direction::Ltr {
                60.0
            } else {
                0.0
            };
        assert_eq!(br.location, point(mode, end, 10.0, 0.0, 0.0, 20.0));
    }
}
fn matrix<S: l::LayoutScalar>() {
    assert_eq!(l::TextGroupAlign::default(), l::TextGroupAlign::None);
    assert_eq!(
        l::NodeInputOf::<S>::default().text_group_align,
        l::TextGroupAlign::None
    );
    for mode in [
        l::WritingMode::HorizontalTb,
        l::WritingMode::VerticalRl,
        l::WritingMode::VerticalLr,
        l::WritingMode::SidewaysRl,
        l::WritingMode::SidewaysLr,
    ] {
        for direction in [l::Direction::Ltr, l::Direction::Rtl] {
            let rtl = direction == l::Direction::Rtl;
            for (group, align, starts) in [
                (l::TextGroupAlign::None, l::TextAlign::Center, [20.0, 30.0]),
                (l::TextGroupAlign::Left, l::TextAlign::Center, [0.0, 10.0]),
                (l::TextGroupAlign::Right, l::TextAlign::Center, [40.0, 50.0]),
                // Numeric Justify has no spacing slots. Its actual
                // unexpandable Center fallback uses the reduced frame and
                // leaves the genuine60/40 atomics unexpanded.
                (l::TextGroupAlign::Left, l::TextAlign::Justify, [0.0, 10.0]),
                (
                    l::TextGroupAlign::Right,
                    l::TextAlign::Justify,
                    [40.0, 50.0],
                ),
                (
                    l::TextGroupAlign::Center,
                    l::TextAlign::Start,
                    if rtl { [20.0, 40.0] } else { [20.0, 20.0] },
                ),
                (
                    l::TextGroupAlign::Center,
                    l::TextAlign::End,
                    if rtl { [20.0, 20.0] } else { [20.0, 40.0] },
                ),
                (
                    l::TextGroupAlign::Start,
                    l::TextAlign::Start,
                    if rtl { [40.0, 60.0] } else { [0.0, 0.0] },
                ),
                (
                    l::TextGroupAlign::End,
                    l::TextAlign::Start,
                    if rtl { [0.0, 20.0] } else { [40.0, 40.0] },
                ),
            ] {
                let tree = two_rows::<S>(mode, direction, group, align);
                let first = tree.compute(mode, &[]);
                assert_two_rows(&first, mode, direction, starts);
                let replay = tree.compute(mode, &[]);
                assert_two_rows(&replay, mode, direction, starts);
                assert_eq!(first.unrounded_entries(), replay.unrounded_entries());
                assert_eq!(first.final_entries(), replay.final_entries());
            }
        }
    }
}
#[test]
fn numeric_group_matrix_preserves_rows_controls_and_axes() {
    matrix::<f32>();
    matrix::<f64>();
}

fn ordered_slots<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let mut root = block(mode, direction, 100.0);
        root.text_group_align = l::TextGroupAlign::Center;
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(root),
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
        let xs = if direction == l::Direction::Ltr {
            [20.0, 50.0, 70.0, 20.0, 45.0]
        } else {
            [50.0, 30.0, 20.0, 55.0, 40.0]
        };
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
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
                let item = node(entries, id);
                assert_eq!(item.source_index, l::SourceIndex::new((id - 1) as usize));
                assert_eq!(item.location, l::Point::new(n(x), n(y)));
                assert_eq!(item.size, l::Size::new(n(width), n(10.0)));
            }
            assert_eq!(node(entries, 4).source_index, l::SourceIndex::new(3));
            assert_eq!(node(entries, 4).size, l::Size::ZERO);
            assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
        }
    }
}
#[test]
fn genuine_resolved_atomic_slots_keep_source_identity_and_visual_order() {
    ordered_slots::<f32>();
    ordered_slots::<f64>();
}

mod cache;
mod edges;
mod membership;
