//! A visible ordinary block shares its parent's float context without enclosing
//! its floated child's height. The root context still encloses that descendant.
use std::{convert::Infallible, slice};
use surgeist_layout as l;

struct Tree<S: l::scalar::LayoutScalar> {
    inputs: [l::node::NodeInputOf<S>; 3],
    children: [Vec<u32>; 3],
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

fn px<S: l::scalar::LayoutScalar>(value: f64) -> l::sizing::PreferredSizeOf<S> {
    l::sizing::PreferredSizeOf::value(
        l::length::LengthPercentageOf::px(S::from_f64(value)).unwrap(),
    )
}

fn block<S: l::scalar::LayoutScalar>(width: f64) -> l::node::NodeInputOf<S> {
    l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        writing_mode: l::geometry::WritingMode::HorizontalTb,
        direction: l::geometry::Direction::Ltr,
        size: l::geometry::Size::new(px(width), l::sizing::PreferredSizeOf::AUTO),
        ..l::node::NodeInputOf::default()
    }
}

fn output<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    node: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, node, crate::layout::observation::Kind::Box)
}

fn ordinary_block_excludes_its_float_but_root_contains_it<S: l::scalar::LayoutScalar>() {
    let mut float = block::<S>(20.0);
    float.float = l::box_model::Float::Left;
    float.size.height = px(30.0);
    let tree = Tree {
        inputs: [block(100.0), block(100.0), float],
        children: [vec![1], vec![2], vec![]],
    };
    let request = {
        let sizing_request = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(S::from_f64(100.0)), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::geometry::WritingMode::HorizontalTb,
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
        assert_eq!(entries.0.node_layouts().len(), 3);
        for node in 0..3 {
            assert_eq!(
                output(entries, node).source_index,
                l::node::SourceIndex::ZERO
            );
        }
    }

    let entries = crate::layout::observation::phase(&batch, false);
    let ordinary = output(entries, 1);
    // CSS2 10.6.3: this visible block has no normal-flow children or lines.
    assert_eq!(ordinary.size.height, S::ZERO);
    assert_eq!(ordinary.size.width, S::from_f64(100.0));
    assert_eq!(ordinary.location, l::geometry::Point::ZERO);

    let float = output(entries, 2);
    assert_eq!(float.location, l::geometry::Point::ZERO);
    assert_eq!(
        float.size,
        l::geometry::Size::new(S::from_f64(20.0), S::from_f64(30.0))
    );
    // CSS2 10.6.7: the root encloses floats participating in its context,
    // including this descendant reached through the ordinary zero-height block.
    assert_eq!(output(entries, 0).size.height, S::from_f64(30.0));
}

#[test]
fn visible_shared_block_excludes_float_height_while_root_contains_it_f32() {
    ordinary_block_excludes_its_float_but_root_contains_it::<f32>();
}

#[test]
fn visible_shared_block_excludes_float_height_while_root_contains_it_f64() {
    ordinary_block_excludes_its_float_but_root_contains_it::<f64>();
}

mod propagation;

mod placement;

mod auto_owner_axes;
mod bfc_auto_sizing;
