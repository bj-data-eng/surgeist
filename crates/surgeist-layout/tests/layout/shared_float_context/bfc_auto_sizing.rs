//! An auto-width overflow BFC must fit beside a preceding float or clear it.
//! CSS permits either choice; these tests assert the shared constraints.
use std::{convert::Infallible, slice};
use surgeist_layout as l;

struct Tree<S: l::LayoutScalar> {
    inputs: [l::NodeInputOf<S>; 4],
    children: [Vec<u32>; 4],
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

fn px<S: l::LayoutScalar>(value: f64) -> l::PreferredSizeOf<S> {
    l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn block<S: l::LayoutScalar>() -> l::NodeInputOf<S> {
    l::NodeInputOf {
        display: l::Display::Block,
        writing_mode: l::WritingMode::HorizontalTb,
        direction: l::Direction::Ltr,
        size: l::Size::new(px(100.0), l::PreferredSizeOf::AUTO),
        ..l::NodeInputOf::default()
    }
}

fn output<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> l::NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|entry| entry.node() == node);
    let output = matching.next().unwrap().output();
    assert!(matching.next().is_none(), "one output per actual node");
    output
}

fn auto_height_bfc_respects_actual_float_band<S: l::LayoutScalar>() {
    // The authored-height neighbour qualifies the same real float/BFC setup
    // before the auto-height profile exercises float-containing size discovery.
    for authored_height in [true, false] {
        let mut incoming = block::<S>();
        incoming.float = l::Float::Left;
        incoming.size = l::Size::new(px(20.0), px(30.0));
        let mut bfc = block::<S>();
        bfc.size.width = l::PreferredSizeOf::AUTO;
        if authored_height {
            bfc.size.height = px(20.0);
        }
        bfc.overflow =
            l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap();
        let mut internal = block::<S>();
        internal.float = l::Float::Left;
        internal.size = l::Size::new(px(10.0), px(20.0));
        let tree = Tree {
            inputs: [block(), incoming, bfc, internal],
            children: [vec![1, 2], vec![], vec![3], vec![]],
        };
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(100.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let batch = l::compute_layout(&tree, 0, request).unwrap();
        let entries = batch.unrounded_entries();
        let root = output(entries, 0);
        let incoming = output(entries, 1);
        let bfc = output(entries, 2);
        let internal = output(entries, 3);
        assert_eq!(root.size.width, S::from_f64(100.0));
        assert_eq!(incoming.location, l::Point::ZERO);
        assert_eq!(
            incoming.size,
            l::Size::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        assert_eq!(internal.location, l::Point::ZERO);
        assert_eq!(
            internal.size,
            l::Size::new(S::from_f64(10.0), S::from_f64(20.0))
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
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(entries.len(), 4);
            for (node, index) in [(0, 0), (1, 0), (2, 1), (3, 0)] {
                assert_eq!(
                    output(entries, node).source_index,
                    l::SourceIndex::new(index)
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
