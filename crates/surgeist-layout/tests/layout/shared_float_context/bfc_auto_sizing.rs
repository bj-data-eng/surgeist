//! An auto-width overflow BFC must fit beside a preceding float or clear it.
//! CSS permits either choice; these tests assert the shared constraints.
use std::{convert::Infallible, slice};
use surgeist_layout as l;

struct Tree<S: l::scalar::LayoutScalar> {
    inputs: [l::node::NodeInputOf<S>; 4],
    children: [Vec<u32>; 4],
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

fn block<S: l::scalar::LayoutScalar>() -> l::node::NodeInputOf<S> {
    l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        writing_mode: l::geometry::WritingMode::HorizontalTb,
        direction: l::geometry::Direction::Ltr,
        size: l::geometry::Size::new(px(100.0), l::sizing::PreferredSizeOf::AUTO),
        ..l::node::NodeInputOf::default()
    }
}

fn output<S: l::scalar::LayoutScalar>(
    entries: (&l::batch::CompletedLayoutBatchOf<u32, S>, bool),
    node: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe(entries, node, crate::layout::observation::Kind::Box)
}

fn auto_height_bfc_respects_actual_float_band<S: l::scalar::LayoutScalar>() {
    // The authored-height neighbour qualifies the same real float/BFC setup
    // before the auto-height profile exercises float-containing size discovery.
    for authored_height in [true, false] {
        let mut incoming = block::<S>();
        incoming.float = l::box_model::Float::Left;
        incoming.size = l::geometry::Size::new(px(20.0), px(30.0));
        let mut bfc = block::<S>();
        bfc.size.width = l::sizing::PreferredSizeOf::AUTO;
        if authored_height {
            bfc.size.height = px(20.0);
        }
        bfc.overflow = l::overflow::ComputedOverflow::try_new(
            l::overflow::Overflow::Hidden,
            l::overflow::Overflow::Hidden,
        )
        .unwrap();
        let mut internal = block::<S>();
        internal.float = l::box_model::Float::Left;
        internal.size = l::geometry::Size::new(px(10.0), px(20.0));
        let tree = Tree {
            inputs: [block(), incoming, bfc, internal],
            children: [vec![1, 2], vec![], vec![3], vec![]],
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
        let entries = crate::layout::observation::phase(&batch, false);
        let root = output(entries, 0);
        let incoming = output(entries, 1);
        let bfc = output(entries, 2);
        let internal = output(entries, 3);
        assert_eq!(root.size.width, S::from_f64(100.0));
        assert_eq!(incoming.location, l::geometry::Point::ZERO);
        assert_eq!(
            incoming.size,
            l::geometry::Size::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        assert_eq!(internal.location, l::geometry::Point::ZERO);
        assert_eq!(
            internal.size,
            l::geometry::Size::new(S::from_f64(10.0), S::from_f64(20.0))
        );
        assert_eq!(bfc.size.height, S::from_f64(20.0));
        assert!(bfc.size.width >= S::from_f64(10.0));
        assert!(bfc.location.x >= S::ZERO && bfc.location.y >= S::ZERO);
        // With no margins, padding, borders, min-size or fixed width, an auto
        // width must fit the containing span. Adjacent placement may narrow
        // it; otherwise clearing provides the full span below the float.
        assert!(
            bfc.location.x + bfc.size.width <= root.size.width,
            "auto-width BFC exceeds its containing span; authored_height={authored_height}, bfc.location={:?}, bfc.size={:?}, root.size={:?}",
            bfc.location,
            bfc.size,
            root.size
        );
        assert!(
            bfc.location.x >= S::from_f64(20.0) || bfc.location.y >= S::from_f64(30.0),
            "positive-height BFC overlaps the preceding float; authored_height={authored_height}, bfc.location={:?}, bfc.size={:?}, root.size={:?}",
            bfc.location,
            bfc.size,
            root.size
        );
        assert_eq!(
            root.size.height,
            S::from_f64(30.0).max(bfc.location.y + bfc.size.height)
        );
        assert!(batch.placed_text().is_empty());
        assert!(batch.unrounded_inline_fragments().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            assert_eq!(entries.0.node_layouts().len(), 4);
            for (node, index) in [(0, 0), (1, 0), (2, 1), (3, 0)] {
                assert_eq!(
                    output(entries, node).source_index,
                    l::node::SourceIndex::new(index)
                );
            }
        }
    }
}

#[test]
fn auto_height_overflow_bfc_fits_or_clears_the_real_parent_float_f32() {
    auto_height_bfc_respects_actual_float_band::<f32>();
}

#[test]
fn auto_height_overflow_bfc_fits_or_clears_the_real_parent_float_f64() {
    auto_height_bfc_respects_actual_float_band::<f64>();
}
