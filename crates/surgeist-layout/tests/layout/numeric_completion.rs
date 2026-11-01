//! Existing numeric-only native wrapping; no Text source or future selector API.
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::BidiLevel;

struct NumericTree<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::NodeInputOf<S>,
}
impl<S: l::LayoutScalar> l::Traverse for NumericTree<S> {
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
impl<S: l::LayoutScalar> l::LayoutTree for NumericTree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}
impl<S: l::LayoutScalar> NumericTree<S> {
    fn new(direction: l::Direction, participants: Vec<l::LayoutInputOf<S>>) -> Self {
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
        children[0] = (1..=count).map(|n| u32::try_from(n).unwrap()).collect();
        Self {
            inputs,
            children,
            non_box: l::NodeInputOf::non_box(),
        }
    }
    fn compute(&self) -> l::CompletedLayoutBatchOf<u32, S> {
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(60.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout(self, 0, request).unwrap()
    }
}
fn atomic<S: l::LayoutScalar>(
    direction: l::Direction,
    width: f64,
    height: f64,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::InlineBlock,
        direction,
        vertical_align: l::VerticalAlignOf::Baseline,
        atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
            BidiLevel::try_new(if direction == l::Direction::Ltr { 0 } else { 1 }).unwrap(),
            l::InlineBreakOpportunity::Allowed,
        )),
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(width)).unwrap()),
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(height)).unwrap()),
        ),
        ..l::NodeInputOf::default()
    })
}
fn boundary<S: l::LayoutScalar>(
    direction: l::Direction,
    kind: l::InlineBoundaryKind,
) -> l::LayoutInputOf<S> {
    // Both endpoints are the SAME actual native box's8/10 baseline metrics.
    // Its individual inline margin, border and padding are genuinely zero.
    l::LayoutInputOf::inline_boundary(
        l::InlineBoundaryInputOf::contribution(
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
fn forced<S: l::LayoutScalar>(direction: l::Direction) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::line_break(
        l::LineBreakInputOf::new()
            .with_direction(direction)
            .with_metrics(
                l::InlineMetricsOf::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    S::from_f64(16.0),
                    S::from_f64(20.0),
                )
                .unwrap(),
            ),
    )
}
fn node<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    id: u32,
) -> l::NodeOutputOf<S> {
    entries
        .iter()
        .find(|row| row.node() == id)
        .unwrap()
        .output()
}
fn assert_numeric_only<S: l::LayoutScalar>(batch: &l::CompletedLayoutBatchOf<u32, S>) {
    // Actual numeric child geometry is the oracle below, not a Text surrogate.
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}
fn ordinary_atoms<S: l::LayoutScalar>(direction: l::Direction) {
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
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = node(entries, 0);
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(60.0), S::from_f64(20.0))
        );
        let [a, b, c] = [1, 2, 3].map(|id| node(entries, id));
        assert_eq!(
            [a.source_index, b.source_index, c.source_index],
            [
                l::SourceIndex::new(0),
                l::SourceIndex::new(1),
                l::SourceIndex::new(2)
            ]
        );
        assert_eq!(
            [a.size, b.size, c.size],
            [
                l::Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                l::Size::new(S::from_f64(30.0), S::from_f64(10.0)),
                l::Size::splat(S::from_f64(10.0))
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
            S::from_f64(if direction == l::Direction::Ltr {
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
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        ordinary_atoms::<f32>(direction);
        ordinary_atoms::<f64>(direction);
    }
}
fn pure_endpoints<S: l::LayoutScalar>(direction: l::Direction) {
    for participants in [
        vec![],
        vec![
            boundary(direction, l::InlineBoundaryKind::Start),
            boundary(direction, l::InlineBoundaryKind::End),
        ],
    ] {
        let tree = NumericTree::<S>::new(direction, participants);
        let batch = tree.compute();
        assert_numeric_only(&batch);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                node(entries, 0).size,
                l::Size::new(S::from_f64(60.0), S::ZERO)
            );
        }
        // A nonempty actual host inventory must not vanish to obtain zero
        // extent; both real endpoint nodes retain source-ordered outputs.
        for (index, id) in tree.children[0].iter().enumerate() {
            assert_eq!(
                node(batch.final_entries(), *id).source_index,
                l::SourceIndex::new(index)
            );
        }
    }
}
#[test]
fn pure_zero_decoration_numeric_endpoints_do_not_create_css_line_extent() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        pure_endpoints::<f32>(direction);
        pure_endpoints::<f64>(direction);
    }
}
fn zero_atomic_is_real<S: l::LayoutScalar>(direction: l::Direction) {
    let tree = NumericTree::<S>::new(
        direction,
        vec![
            boundary(direction, l::InlineBoundaryKind::Start),
            atomic(direction, 0.0, 0.0),
            boundary(direction, l::InlineBoundaryKind::End),
        ],
    );
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(
            node(entries, 0).size,
            l::Size::new(S::from_f64(60.0), S::from_f64(10.0))
        );
        let actual = node(entries, 2);
        assert_eq!(actual.source_index, l::SourceIndex::new(1));
        assert_eq!(actual.size, l::Size::ZERO);
        // A zero-height atomic has fallback baseline0. Its actual y8 therefore
        // observes the real8/10 line baseline from the supplied endpoint box.
        assert_eq!(
            actual.location,
            l::Point::new(
                S::from_f64(if direction == l::Direction::Ltr {
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
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        zero_atomic_is_real::<f32>(direction);
        zero_atomic_is_real::<f64>(direction);
    }
}
fn forced_is_real<S: l::LayoutScalar>(direction: l::Direction) {
    let tree = NumericTree::<S>::new(
        direction,
        vec![
            boundary(direction, l::InlineBoundaryKind::Start),
            forced(direction),
            boundary(direction, l::InlineBoundaryKind::End),
        ],
    );
    let batch = tree.compute();
    assert_numeric_only(&batch);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        // Start8/10 plus actual Forced16/20 gives before16/after4:20.
        // The strut-only Paragraph receiver adds no used line extent.
        assert_eq!(
            node(entries, 0).size,
            l::Size::new(S::from_f64(60.0), S::from_f64(20.0))
        );
        let actual = node(entries, 2);
        assert_eq!(actual.source_index, l::SourceIndex::new(1));
        // This real Forced control is Baseline-aligned at the actual16px
        // baseline of its20px line, independent of the phantom receiver.
        assert_eq!(actual.location.y, S::from_f64(16.0));
    }
}
#[test]
fn actual_forced_numeric_control_keeps_its_line_without_phantom_receiver_extent() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        forced_is_real::<f32>(direction);
        forced_is_real::<f64>(direction);
    }
}

fn phantom_gap_preserves_block_margins<S: l::LayoutScalar>(direction: l::Direction) {
    let a = l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::Block,
        direction,
        size: l::Size::new(
            l::PreferredSizeOf::AUTO,
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(5.0)).unwrap()),
        ),
        margin: l::Edges {
            bottom: l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(10.0)).unwrap()),
            ..l::Edges::all(l::LengthAutoOf::ZERO)
        },
        ..l::NodeInputOf::default()
    });
    let b = l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::Block,
        direction,
        size: l::Size::new(
            l::PreferredSizeOf::AUTO,
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(7.0)).unwrap()),
        ),
        margin: l::Edges {
            top: l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(20.0)).unwrap()),
            ..l::Edges::all(l::LengthAutoOf::ZERO)
        },
        ..l::NodeInputOf::default()
    });
    let mut control = None;
    for endpoints in [false, true] {
        let mut tree = NumericTree::<S>::new(
            direction,
            vec![
                a.clone(),
                boundary(direction, l::InlineBoundaryKind::Start),
                boundary(direction, l::InlineBoundaryKind::End),
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
        let observed = [batch.unrounded_entries(), batch.final_entries()].map(|entries| {
            let root = node(entries, 0);
            let a = node(entries, 1);
            let b = node(entries, 4);
            assert_eq!(
                root.size,
                l::Size::new(S::from_f64(60.0), S::from_f64(32.0))
            );
            assert_eq!(a.location, l::Point::ZERO);
            assert_eq!(a.size, l::Size::new(S::from_f64(60.0), S::from_f64(5.0)));
            assert_eq!(b.location, l::Point::new(S::ZERO, S::from_f64(25.0)));
            assert_eq!(b.size, l::Size::new(S::from_f64(60.0), S::from_f64(7.0)));
            assert_eq!(a.source_index, l::SourceIndex::new(0));
            assert_eq!(
                b.source_index,
                l::SourceIndex::new(if endpoints { 3 } else { 1 })
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
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        phantom_gap_preserves_block_margins::<f32>(direction);
        phantom_gap_preserves_block_margins::<f64>(direction);
    }
}

mod decoration;

mod warm_cache;
