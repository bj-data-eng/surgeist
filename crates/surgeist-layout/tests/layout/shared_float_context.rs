//! A visible ordinary block shares its parent's float context without enclosing
//! its floated child's height. The root context still encloses that descendant.
use std::{convert::Infallible, slice};
use surgeist_layout as l;

struct Tree<S: l::LayoutScalar> {
    inputs: [l::NodeInputOf<S>; 3],
    children: [Vec<u32>; 3],
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

fn ordinary_block_excludes_its_float_but_root_contains_it<S: l::LayoutScalar>() {
    let mut float = block::<S>(20.0);
    float.float = l::Float::Left;
    float.size.height = px(30.0);
    let tree = Tree {
        inputs: [block(100.0), block(100.0), float],
        children: [vec![1], vec![2], vec![]],
    };
    let request = l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(100.0)),
        l::AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = l::compute_layout(&tree, 0, request).unwrap();

    assert!(batch.placed_text().is_empty());
    assert!(batch.unrounded_inline_fragments().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), 3);
        for node in 0..3 {
            assert_eq!(output(entries, node).source_index, l::SourceIndex::ZERO);
        }
    }

    let entries = batch.unrounded_entries();
    let ordinary = output(entries, 1);
    // CSS2 10.6.3: this visible block has no normal-flow children or lines.
    assert_eq!(ordinary.size.height, S::ZERO);
    assert_eq!(ordinary.size.width, S::from_f64(100.0));
    assert_eq!(ordinary.location, l::Point::ZERO);

    let float = output(entries, 2);
    assert_eq!(float.location, l::Point::ZERO);
    assert_eq!(
        float.size,
        l::Size::new(S::from_f64(20.0), S::from_f64(30.0))
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
