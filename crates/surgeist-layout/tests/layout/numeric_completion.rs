//! Existing numeric-only native wrapping; no Text source or future selector API.
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::supplied_line::BidiLevel;

struct NumericTree<S: l::scalar::LayoutScalar> {
    inputs: Vec<l::node::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::node::NodeInputOf<S>,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for NumericTree<S> {
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
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for NumericTree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}
impl<S: l::scalar::LayoutScalar> NumericTree<S> {
    fn new(
        direction: l::geometry::Direction,
        participants: Vec<l::node::LayoutInputOf<S>>,
    ) -> Self {
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
        children[0] = (1..=count).map(|n| u32::try_from(n).unwrap()).collect();
        Self {
            inputs,
            children,
            non_box: l::node::NodeInputOf::non_box(),
        }
    }
    fn compute(&self) -> l::batch::CompletedLayoutBatchOf<u32, S> {
        let request = {
            let sizing_request = crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::Normal,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            );
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
        l::compute_layout(self, 0, request).unwrap()
    }
}
fn atomic<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    width: f64,
    height: f64,
) -> l::node::LayoutInputOf<S> {
    l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        display: l::box_model::Display::InlineBlock,
        direction,
        vertical_align: l::inline::VerticalAlignOf::Baseline,
        atomic_inline_participation: Some(l::inline::AtomicInlineParticipation::new(
            BidiLevel::try_new(if direction == l::geometry::Direction::Ltr {
                0
            } else {
                1
            })
            .unwrap(),
            l::text_flow::InlineBreakOpportunity::Allowed,
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
fn boundary<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    kind: l::inline::InlineBoundaryKind,
) -> l::node::LayoutInputOf<S> {
    // Both endpoints are the SAME actual native box's8/10 baseline metrics.
    // Its individual inline margin, border and padding are zero.
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
fn forced<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
) -> l::node::LayoutInputOf<S> {
    l::node::LayoutInputOf::line_break(
        l::text_flow::LineBreakInputOf::new()
            .with_direction(direction)
            .with_metrics(
                l::inline::InlineMetricsOf::try_new(
                    surgeist_layout::baseline::InlineBaselineContext::Physical,
                    S::from_f64(16.0),
                    S::from_f64(20.0),
                )
                .unwrap(),
            ),
    )
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
fn assert_numeric_only<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
) {
    // Actual numeric child geometry is the oracle below, not a Text surrogate.
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}
fn ordinary_atoms<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    let tree = NumericTree::<S>::new(
        direction,
        vec![
            atomic(direction, 30.0, 10.0),
            atomic(direction, 30.0, 10.0),
            atomic(direction, 10.0, 10.0),
        ],
    );
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [
        crate::layout::observation::phase(&batch, false),
        crate::layout::observation::phase(&batch, true),
    ] {
        let root = node(entries, 0);
        assert_eq!(
            root.size,
            l::geometry::Size::new(S::from_f64(60.0), S::from_f64(20.0))
        );
        let [a, b, c] = [1, 2, 3].map(|id| {
            node_of_kind(
                entries,
                id,
                crate::layout::observation::expected_kind(
                    l::tree::LayoutTree::layout_input(&tree, id),
                    false,
                ),
            )
        });
        assert_eq!(
            [a.source_index, b.source_index, c.source_index],
            [
                l::node::SourceIndex::new(0),
                l::node::SourceIndex::new(1),
                l::node::SourceIndex::new(2)
            ]
        );
        assert_eq!(
            [a.size, b.size, c.size],
            [
                l::geometry::Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                l::geometry::Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                l::geometry::Size::splat(S::from_f64(10.0))
            ]
        );
        // The last-fitting permitted first line owns both30px boxes. The
        // remaining actual10px box is a second10px-high line, not a text node.
        assert_eq!(
            [a.location.y, b.location.y, c.location.y],
            [S::ZERO, S::ZERO, S::from_f64(10.0)]
        );
        // Equal30px boxes occupy the full first row; relative bidi ordering of
        // those equal boxes is outside this partition characterization.
        assert_eq!(
            [
                a.location.x.min(b.location.x),
                a.location.x.max(b.location.x)
            ],
            [S::ZERO, S::from_f64(30.0)]
        );
        assert_eq!(
            c.location.x,
            S::from_f64(if direction == l::geometry::Direction::Ltr {
                0.0
            } else {
                50.0
            })
        );
        assert_eq!(
            [a.size.width + b.size.width, c.size.width],
            [S::from_f64(60.0), S::from_f64(10.0)]
        );
    }
}
#[test]
fn current_numeric_atomic_wrapping_keeps_full_fitting_prefix_then_remainder() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        ordinary_atoms::<f32>(direction);
        ordinary_atoms::<f64>(direction);
    }
}
fn pure_endpoints<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    for participants in [
        vec![],
        vec![
            boundary(direction, l::inline::InlineBoundaryKind::Start),
            boundary(direction, l::inline::InlineBoundaryKind::End),
        ],
    ] {
        let tree = NumericTree::<S>::new(direction, participants);
        let batch = tree.compute();
        assert_numeric_only(&batch);
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            assert_eq!(
                node(entries, 0).size,
                l::geometry::Size::new(S::from_f64(60.0), S::ZERO)
            );
        }
        // A nonempty actual host inventory must not vanish to obtain zero
        // extent; both real endpoint nodes retain source-ordered outputs.
        for (index, id) in tree.children[0].iter().enumerate() {
            assert_eq!(
                node_of_kind(
                    crate::layout::observation::phase(&batch, true),
                    *id,
                    crate::layout::observation::expected_kind(
                        tree.inputs[*id as usize].as_view(),
                        false
                    )
                )
                .source_index,
                l::node::SourceIndex::new(index)
            );
        }
    }
}
#[test]
fn pure_zero_decoration_numeric_endpoints_do_not_create_css_line_extent() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        pure_endpoints::<f32>(direction);
        pure_endpoints::<f64>(direction);
    }
}
fn zero_atomic_is_real<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    let tree = NumericTree::<S>::new(
        direction,
        vec![
            boundary(direction, l::inline::InlineBoundaryKind::Start),
            atomic(direction, 0.0, 0.0),
            boundary(direction, l::inline::InlineBoundaryKind::End),
        ],
    );
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [
        crate::layout::observation::phase(&batch, false),
        crate::layout::observation::phase(&batch, true),
    ] {
        assert_eq!(
            node(entries, 0).size,
            l::geometry::Size::new(S::from_f64(60.0), S::from_f64(10.0))
        );
        let actual = node(entries, 2);
        assert_eq!(actual.source_index, l::node::SourceIndex::new(1));
        assert_eq!(actual.size, l::geometry::Size::ZERO);
        // A zero-height atomic has fallback baseline0. Its actual y8 therefore
        // observes the real8/10 line baseline from the supplied endpoint box.
        assert_eq!(
            actual.location,
            l::geometry::Point::new(
                S::from_f64(if direction == l::geometry::Direction::Ltr {
                    0.0
                } else {
                    60.0
                }),
                S::from_f64(8.0)
            )
        );
    }
}
#[test]
fn actual_zero_sized_in_flow_atomic_keeps_positive_endpoint_line_real() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        zero_atomic_is_real::<f32>(direction);
        zero_atomic_is_real::<f64>(direction);
    }
}
fn forced_is_real<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    let tree = NumericTree::<S>::new(
        direction,
        vec![
            boundary(direction, l::inline::InlineBoundaryKind::Start),
            forced(direction),
            boundary(direction, l::inline::InlineBoundaryKind::End),
        ],
    );
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [
        crate::layout::observation::phase(&batch, false),
        crate::layout::observation::phase(&batch, true),
    ] {
        // Start8/10 plus actual Forced16/20 gives before16/after4:20.
        // The strut-only Paragraph receiver adds no used line extent.
        assert_eq!(
            node(entries, 0).size,
            l::geometry::Size::new(S::from_f64(60.0), S::from_f64(20.0))
        );
        let actual = node_of_kind(
            entries,
            2,
            crate::layout::observation::Kind::NonBox(l::fragmentation::NonBoxKind::LineBreak),
        );
        assert_eq!(actual.source_index, l::node::SourceIndex::new(1));
        // This real Forced control is Baseline-aligned at the actual16px
        // baseline of its20px line, independent of the phantom receiver.
        assert_eq!(actual.location.y, S::from_f64(16.0));
    }
}
#[test]
fn actual_forced_numeric_control_keeps_its_line_without_phantom_receiver_extent() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        forced_is_real::<f32>(direction);
        forced_is_real::<f64>(direction);
    }
}

fn phantom_gap_preserves_block_margins<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
) {
    let a = l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        direction,
        size: l::geometry::Size::new(
            l::sizing::PreferredSizeOf::AUTO,
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(5.0)).unwrap(),
            ),
        ),
        margin: l::geometry::Edges {
            bottom: l::length::LengthAutoOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(10.0)).unwrap(),
            ),
            ..l::geometry::Edges::all(l::length::LengthAutoOf::ZERO)
        },
        ..l::node::NodeInputOf::default()
    });
    let b = l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        direction,
        size: l::geometry::Size::new(
            l::sizing::PreferredSizeOf::AUTO,
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(7.0)).unwrap(),
            ),
        ),
        margin: l::geometry::Edges {
            top: l::length::LengthAutoOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(20.0)).unwrap(),
            ),
            ..l::geometry::Edges::all(l::length::LengthAutoOf::ZERO)
        },
        ..l::node::NodeInputOf::default()
    });
    let mut control = None;
    for endpoints in [false, true] {
        let mut tree = NumericTree::<S>::new(
            direction,
            vec![
                a.clone(),
                boundary(direction, l::inline::InlineBoundaryKind::Start),
                boundary(direction, l::inline::InlineBoundaryKind::End),
                b.clone(),
            ],
        );
        if !endpoints {
            // The same real A/B node identities remain1/4. Endpoint input
            // records are unreachable storage, not children, hidden boxes or
            // fabricated source participants in this control traversal.
            tree.children[0] = vec![1, 4];
        }
        let batch = tree.compute();
        assert_numeric_only(&batch);
        let observed = [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ]
        .map(|entries| {
            let root = node(entries, 0);
            let a = node(entries, 1);
            let b = node(entries, 4);
            assert_eq!(
                root.size,
                l::geometry::Size::new(S::from_f64(60.0), S::from_f64(32.0))
            );
            assert_eq!(a.location, l::geometry::Point::ZERO);
            assert_eq!(
                a.size,
                l::geometry::Size::new(S::from_f64(60.0), S::from_f64(5.0))
            );
            assert_eq!(
                b.location,
                l::geometry::Point::new(S::ZERO, S::from_f64(25.0))
            );
            assert_eq!(
                b.size,
                l::geometry::Size::new(S::from_f64(60.0), S::from_f64(7.0))
            );
            assert_eq!(a.source_index, l::node::SourceIndex::new(0));
            assert_eq!(
                b.source_index,
                l::node::SourceIndex::new(if endpoints { 3 } else { 1 })
            );
            (root.size, a.location, a.size, b.location, b.size)
        });
        if let Some(without_endpoints) = control {
            assert_eq!(observed, without_endpoints);
        } else {
            control = Some(observed);
        }
    }
}
#[test]
fn phantom_numeric_endpoint_run_preserves_adjoining_real_block_margins() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        phantom_gap_preserves_block_margins::<f32>(direction);
        phantom_gap_preserves_block_margins::<f64>(direction);
    }
}

mod decoration;

mod warm_cache;
