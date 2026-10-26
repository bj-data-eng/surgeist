//! A float in an ordinary shared block must retain placement beside an earlier
//! same-context float when its actual node output is published.
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

fn block<S: l::LayoutScalar>(width: f64) -> l::NodeInputOf<S> {
    l::NodeInputOf {
        display: l::Display::Block,
        writing_mode: l::WritingMode::HorizontalTb,
        direction: l::Direction::Ltr,
        size: l::Size::new(px(width), l::PreferredSizeOf::AUTO),
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

fn incoming_left_float_positions_descendant_left_float<S: l::LayoutScalar>() {
    let mut incoming = block::<S>(20.0);
    incoming.float = l::Float::Left;
    incoming.size.height = px(30.0);
    let mut descendant = block::<S>(10.0);
    descendant.float = l::Float::Left;
    descendant.size.height = px(20.0);
    let tree = Tree {
        inputs: [block(100.0), incoming, block(100.0), descendant],
        children: [vec![1, 2], vec![], vec![3], vec![]],
    };
    let request = l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(100.0)),
        l::AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = l::compute_layout(&tree, 0, request).unwrap();
    let entries = batch.unrounded_entries();

    // CSS2 9.5.1 rules 2 and 8: the later left float fits at the highest
    // position beside the earlier 20-unit float. The ordinary parent is at
    // nominal (0,0), so this is also the descendant's local position.
    assert_eq!(
        output(entries, 3).location,
        l::Point::new(S::from_f64(20.0), S::ZERO)
    );
    assert_eq!(
        output(entries, 3).size,
        l::Size::new(S::from_f64(10.0), S::from_f64(20.0))
    );
    assert_eq!(output(entries, 1).location, l::Point::ZERO);
    assert_eq!(
        output(entries, 1).size,
        l::Size::new(S::from_f64(20.0), S::from_f64(30.0))
    );
    assert_eq!(output(entries, 2).location, l::Point::ZERO);
    // Keep the placement assertion ahead of the separately known ordinary
    // block auto-height defect, so its failure cannot hide this regression.
    assert_eq!(output(entries, 2).size.height, S::ZERO);
    assert_eq!(output(entries, 0).size.height, S::from_f64(30.0));

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

#[test]
fn incoming_left_float_keeps_descendant_left_float_at_inline_twenty_f32() {
    incoming_left_float_positions_descendant_left_float::<f32>();
}

#[test]
fn incoming_left_float_keeps_descendant_left_float_at_inline_twenty_f64() {
    incoming_left_float_positions_descendant_left_float::<f64>();
}
