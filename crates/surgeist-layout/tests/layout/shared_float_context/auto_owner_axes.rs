//! A root's auto block extent includes its real float before final physical
//! projection, including block-decreasing and bottom-to-top inline axes.
use std::{convert::Infallible, slice};
use surgeist_layout as l;

struct Tree<S: l::scalar::LayoutScalar> {
    inputs: [l::node::NodeInputOf<S>; 2],
    children: [Vec<u32>; 2],
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
        &self.inputs[node as usize]
    }

    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        l::node::LayoutInputViewOf::Box(&self.inputs[node as usize])
    }
}

fn root_auto_block_contains_float_in_each_writing_mode<S: l::scalar::LayoutScalar>() {
    let n = S::from_f64;
    let px = |value| {
        l::sizing::PreferredSizeOf::value(l::length::LengthPercentageOf::px(n(value)).unwrap())
    };
    // Literal physical sizes/positions from Writing Modes 4's direction table;
    // no production FlowAxes or float-placement helper derives these oracles.
    for (mode, root_size, float_size, float_location) in [
        (
            l::geometry::WritingMode::HorizontalTb,
            (100.0, 30.0),
            (20.0, 30.0),
            (0.0, 0.0),
        ),
        (
            l::geometry::WritingMode::VerticalRl,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 0.0),
        ),
        (
            l::geometry::WritingMode::VerticalLr,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 0.0),
        ),
        (
            l::geometry::WritingMode::SidewaysRl,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 0.0),
        ),
        (
            l::geometry::WritingMode::SidewaysLr,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 80.0),
        ),
    ] {
        let horizontal = mode == l::geometry::WritingMode::HorizontalTb;
        let root = l::node::NodeInputOf {
            display: l::box_model::Display::Block,
            writing_mode: mode,
            direction: l::geometry::Direction::Ltr,
            size: if horizontal {
                l::geometry::Size::new(px(100.0), l::sizing::PreferredSizeOf::AUTO)
            } else {
                l::geometry::Size::new(l::sizing::PreferredSizeOf::AUTO, px(100.0))
            },
            ..l::node::NodeInputOf::default()
        };
        let float = l::node::NodeInputOf {
            display: l::box_model::Display::Block,
            writing_mode: mode,
            direction: l::geometry::Direction::Ltr,
            float: l::box_model::Float::Left,
            size: if horizontal {
                l::geometry::Size::new(px(20.0), px(30.0))
            } else {
                l::geometry::Size::new(px(30.0), px(20.0))
            },
            ..l::node::NodeInputOf::default()
        };
        let tree = Tree {
            inputs: [root, float],
            children: [vec![1], vec![]],
        };
        // The native viewport has physical finite100 on logical inline. Logical
        // block is indefinite Normal space, and the authored block size is AUTO.
        let available = if horizontal {
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
                    mode,
                    surgeist_layout::geometry::Direction::Ltr,
                ),
                sizing_request
                    .orthogonal_fallback()
                    .initial_containing_block(),
            )
        };
        let batch = l::compute_layout(&tree, 0, request).unwrap();
        assert!(batch.placed_text().is_empty());
        assert!(batch.unrounded_inline_fragments().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            assert_eq!(entries.0.node_layouts().len(), 2, "mode={mode:?}");
            for (id, size) in [(0, root_size), (1, float_size)] {
                let output = crate::layout::observation::observe(
                    entries,
                    id,
                    crate::layout::observation::Kind::Box,
                );
                assert_eq!(
                    entries
                        .0
                        .node_layouts()
                        .filter(|(node, _)| *node == id)
                        .count(),
                    1,
                    "one actual output; mode={mode:?}"
                );
                assert_eq!(
                    output.source_index,
                    l::node::SourceIndex::ZERO,
                    "mode={mode:?}"
                );
                assert_eq!(
                    output.size,
                    l::geometry::Size::new(n(size.0), n(size.1)),
                    "mode={mode:?}"
                );
                if id == 1 {
                    assert_eq!(
                        output.location,
                        l::geometry::Point::new(n(float_location.0), n(float_location.1)),
                        "mode={mode:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn auto_block_root_projects_its_real_float_in_all_five_writing_modes_f32() {
    root_auto_block_contains_float_in_each_writing_mode::<f32>();
}

#[test]
fn auto_block_root_projects_its_real_float_in_all_five_writing_modes_f64() {
    root_auto_block_contains_float_in_each_writing_mode::<f64>();
}
