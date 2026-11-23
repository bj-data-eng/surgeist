//! Physical placement consumes supplied L2 order once and retains source order.
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::supplied_line::BidiLevel;

struct AtomicTree<S: l::scalar::LayoutScalar> {
    inputs: Vec<l::node::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::node::NodeInputOf<S>,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for AtomicTree<S> {
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
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for AtomicTree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}
impl<S: l::scalar::LayoutScalar> AtomicTree<S> {
    fn new(
        direction: l::geometry::Direction,
        capacity: f64,
        widths: &[f64],
        levels: &[u8],
    ) -> Self {
        assert_eq!(widths.len(), levels.len());
        let mut inputs = vec![l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
            display: l::box_model::Display::Block,
            direction,
            writing_mode: l::geometry::WritingMode::HorizontalTb,
            text_wrap_style: l::text_flow::TextWrapStyle::Auto,
            text_align: l::text_flow::TextAlign::Start,
            text_align_last: l::text_flow::TextAlignLast::Start,
            size: l::geometry::Size::new(
                l::sizing::PreferredSizeOf::value(
                    l::length::LengthPercentageOf::px(S::from_f64(capacity)).unwrap(),
                ),
                l::sizing::PreferredSizeOf::AUTO,
            ),
            ..l::node::NodeInputOf::default()
        })];
        for (&width, &level) in widths.iter().zip(levels) {
            assert!(width > 0.0);
            inputs.push(l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
                display: l::box_model::Display::InlineBlock,
                direction,
                vertical_align: l::inline::VerticalAlignOf::Baseline,
                atomic_inline_participation: Some(l::inline::AtomicInlineParticipation::new(
                    BidiLevel::try_new(level).unwrap(),
                    l::text_flow::InlineBreakOpportunity::Allowed,
                )),
                size: l::geometry::Size::new(
                    l::sizing::PreferredSizeOf::value(
                        l::length::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
                    ),
                    l::sizing::PreferredSizeOf::value(
                        l::length::LengthPercentageOf::px(S::from_f64(10.0)).unwrap(),
                    ),
                ),
                ..l::node::NodeInputOf::default()
            }));
        }
        let mut children = vec![vec![]; inputs.len()];
        children[0] = (1..inputs.len())
            .map(|id| u32::try_from(id).unwrap())
            .collect();
        Self {
            inputs,
            children,
            non_box: l::node::NodeInputOf::non_box(),
        }
    }
}
fn output<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    node: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, node, crate::layout::observation::Kind::Box)
}
fn assert_placement<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    capacity: f64,
    widths: &[f64],
    levels: &[u8],
    source_x: &[f64],
    physical_order: &[u32],
    hull: (f64, f64),
) {
    assert_eq!(widths.len(), source_x.len());
    let tree = AtomicTree::<S>::new(direction, capacity, widths, levels);
    let input_before = tree.inputs.clone();
    let source_order = tree.children[0].clone();
    for (index, &level) in levels.iter().enumerate() {
        let actual = tree.inputs[index + 1]
            .as_box()
            .unwrap()
            .atomic_inline_participation
            .unwrap();
        assert_eq!(actual.bidi_level().get(), level);
        assert_eq!(
            actual.following_break(),
            l::text_flow::InlineBreakOpportunity::Allowed
        );
    }
    let request = {
        let sizing_request = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(S::from_f64(capacity)), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(&tree, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(&tree, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    };
    let batch = l::compute_layout(&tree, 0, request).unwrap();
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    assert_eq!(tree.inputs, input_before);
    assert_eq!(tree.children[0], source_order);
    for entries in [
        crate::layout::observation::phase(&batch, false),
        crate::layout::observation::phase(&batch, true),
    ] {
        assert_eq!(
            output(entries, 0).size,
            l::geometry::Size::new(S::from_f64(capacity), S::from_f64(10.0))
        );
        let mut physical = vec![];
        for (i, (&width, &x)) in widths.iter().zip(source_x).enumerate() {
            let id = u32::try_from(i + 1).unwrap();
            let actual = output(entries, id);
            assert_eq!(actual.source_index, l::node::SourceIndex::new(i));
            assert_eq!(
                actual.size,
                l::geometry::Size::new(S::from_f64(width), S::from_f64(10.0))
            );
            assert_eq!(actual.location.y, S::ZERO);
            assert_eq!(
                actual.location.x,
                S::from_f64(x),
                "actual source node{id}, supplied level{}",
                levels[i]
            );
            physical.push((
                id,
                actual.location.x.to_f64(),
                (actual.location.x + actual.size.width).to_f64(),
            ));
        }
        physical.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap());
        assert_eq!(
            physical.iter().map(|item| item.0).collect::<Vec<_>>(),
            physical_order
        );
        assert_eq!(physical.first().unwrap().1, hull.0);
        assert_eq!(physical.last().unwrap().2, hull.1);
        for pair in physical.windows(2) {
            assert_eq!(pair[0].2, pair[1].1);
        }
        assert_eq!(hull.1 - hull.0, widths.iter().sum::<f64>());
    }
}

macro_rules! scalar_cases {
    ($module:ident,$scalar:ty) => {
        mod $module {
            use super::*;
            #[test]
            fn ltr_even_levels_keep_source_order() {
                assert_placement::<$scalar>(
                    l::geometry::Direction::Ltr,
                    60.0,
                    &[30.0, 20.0, 10.0],
                    &[0, 0, 0],
                    &[0.0, 30.0, 50.0],
                    &[1, 2, 3],
                    (0.0, 60.0),
                );
            }
            #[test]
            fn rtl_odd_levels_reverse_physical_order_once() {
                // L2 reverses the complete level1 run: C,B,A. Physical widths
                // then accumulate from the occupied interval's left edge.
                assert_placement::<$scalar>(
                    l::geometry::Direction::Rtl,
                    60.0,
                    &[30.0, 20.0, 10.0],
                    &[1, 1, 1],
                    &[30.0, 10.0, 0.0],
                    &[3, 2, 1],
                    (0.0, 60.0),
                );
            }
            #[test]
            fn rtl_partial_line_keeps_reversed_order_in_right_aligned_hull() {
                assert_placement::<$scalar>(
                    l::geometry::Direction::Rtl,
                    100.0,
                    &[30.0, 20.0, 10.0],
                    &[1, 1, 1],
                    &[70.0, 50.0, 40.0],
                    &[3, 2, 1],
                    (40.0, 100.0),
                );
            }
            #[test]
            fn ltr_mixed_levels_reverse_only_the_odd_subsequence() {
                // Level1 B,C reverses to C,B; the level0 exterior stays A,D.
                assert_placement::<$scalar>(
                    l::geometry::Direction::Ltr,
                    100.0,
                    &[30.0, 20.0, 10.0, 10.0],
                    &[0, 1, 1, 0],
                    &[0.0, 40.0, 30.0, 60.0],
                    &[1, 3, 2, 4],
                    (0.0, 70.0),
                );
            }
            #[test]
            fn rtl_mixed_levels_preserve_the_even_subsequence_inside_outer_reversal() {
                // L2 level2 first reverses B,C, then level1 reverses all slots:
                // A,C,B,D becomes D,B,C,A. The70-unit hull begins at30.
                assert_placement::<$scalar>(
                    l::geometry::Direction::Rtl,
                    100.0,
                    &[30.0, 20.0, 10.0, 10.0],
                    &[1, 2, 2, 1],
                    &[70.0, 40.0, 60.0, 30.0],
                    &[4, 2, 3, 1],
                    (30.0, 100.0),
                );
            }
        }
    };
}
scalar_cases!(f32_cases, f32);
scalar_cases!(f64_cases, f64);
