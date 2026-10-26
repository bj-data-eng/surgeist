//! A root's auto block extent includes its real float before final physical
//! projection, including block-decreasing and bottom-to-top inline axes.
use std::{convert::Infallible, slice};
use surgeist_layout as l;

struct Tree<S: l::LayoutScalar> {
    inputs: [l::NodeInputOf<S>; 2],
    children: [Vec<u32>; 2],
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
        &self.inputs[node as usize]
    }

    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutInputViewOf::Box(&self.inputs[node as usize])
    }
}

fn root_auto_block_contains_float_in_each_writing_mode<S: l::LayoutScalar>() {
    let n = S::from_f64;
    let px = |value| l::PreferredSizeOf::value(l::LengthPercentageOf::px(n(value)).unwrap());
    // Literal physical sizes/positions from Writing Modes 4's direction table;
    // no production FlowAxes or float-placement helper derives these oracles.
    for (mode, root_size, float_size, float_location) in [
        (
            l::WritingMode::HorizontalTb,
            (100.0, 30.0),
            (20.0, 30.0),
            (0.0, 0.0),
        ),
        (
            l::WritingMode::VerticalRl,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 0.0),
        ),
        (
            l::WritingMode::VerticalLr,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 0.0),
        ),
        (
            l::WritingMode::SidewaysRl,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 0.0),
        ),
        (
            l::WritingMode::SidewaysLr,
            (30.0, 100.0),
            (30.0, 20.0),
            (0.0, 80.0),
        ),
    ] {
        let horizontal = mode == l::WritingMode::HorizontalTb;
        let root = l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: mode,
            direction: l::Direction::Ltr,
            size: if horizontal {
                l::Size::new(px(100.0), l::PreferredSizeOf::AUTO)
            } else {
                l::Size::new(l::PreferredSizeOf::AUTO, px(100.0))
            },
            ..l::NodeInputOf::default()
        };
        let float = l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: mode,
            direction: l::Direction::Ltr,
            float: l::Float::Left,
            size: if horizontal {
                l::Size::new(px(20.0), px(30.0))
            } else {
                l::Size::new(px(30.0), px(20.0))
            },
            ..l::NodeInputOf::default()
        };
        let tree = Tree {
            inputs: [root, float],
            children: [vec![1], vec![]],
        };
        // Available sizes are physical: logical inline is finite100, while
        // logical block is intrinsic MaxContent and the authored block is AUTO.
        let available = if horizontal {
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
        let batch = l::compute_layout(&tree, 0, request).unwrap();
        assert!(batch.placed_text().is_empty());
        assert!(batch.unrounded_inline_fragments().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(entries.len(), 2, "mode={mode:?}");
            for (id, size) in [(0, root_size), (1, float_size)] {
                let mut matching = entries.iter().filter(|entry| entry.node() == id);
                let output = matching.next().unwrap().output();
                assert!(
                    matching.next().is_none(),
                    "one actual output; mode={mode:?}"
                );
                assert_eq!(output.source_index, l::SourceIndex::ZERO, "mode={mode:?}");
                assert_eq!(
                    output.size,
                    l::Size::new(n(size.0), n(size.1)),
                    "mode={mode:?}"
                );
                if id == 1 {
                    assert_eq!(
                        output.location,
                        l::Point::new(n(float_location.0), n(float_location.1)),
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
