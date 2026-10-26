//! Shared contexts transfer descendant floats without giving ordinary blocks
//! float-containing auto heights or using relative visual offsets as flow input.
use super::{block, l, output, px};
use std::{convert::Infallible, slice};
use surgeist_text::BidiLevel;

struct FlowTree<S: l::LayoutScalar> {
    inputs: Vec<l::NodeInputOf<S>>,
    children: Vec<Vec<u32>>,
}

impl<S: l::LayoutScalar> l::Traverse for FlowTree<S> {
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

impl<S: l::LayoutScalar> l::LayoutTree for FlowTree<S> {
    type MeasureError = Infallible;

    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        &self.inputs[node as usize]
    }

    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutInputViewOf::Box(&self.inputs[node as usize])
    }
}

impl<S: l::LayoutScalar> FlowTree<S> {
    fn new(inputs: Vec<l::NodeInputOf<S>>, edges: &[(u32, &[u32])]) -> Self {
        let mut children = vec![vec![]; inputs.len()];
        for &(parent, list) in edges {
            children[parent as usize] = list.to_vec();
        }
        Self { inputs, children }
    }

    fn compute(&self, source_indices: &[usize]) -> l::CompletedLayoutBatchOf<u32, S> {
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(100.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let batch = l::compute_layout(self, 0, request).unwrap();
        assert!(batch.placed_text().is_empty());
        assert!(batch.unrounded_inline_fragments().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(entries.len(), self.inputs.len());
            for (node, &index) in source_indices.iter().enumerate() {
                assert_eq!(
                    output(entries, u32::try_from(node).unwrap()).source_index,
                    l::SourceIndex::new(index)
                );
            }
        }
        batch
    }
}

fn float<S: l::LayoutScalar>(side: l::Float, width: f64, height: f64) -> l::NodeInputOf<S> {
    let mut input = block(width);
    input.float = side;
    input.size.height = px(height);
    input
}

fn atomic<S: l::LayoutScalar>() -> l::NodeInputOf<S> {
    l::NodeInputOf {
        display: l::Display::InlineBlock,
        writing_mode: l::WritingMode::HorizontalTb,
        direction: l::Direction::Ltr,
        text_align: l::TextAlign::Start,
        vertical_align: l::VerticalAlign::Baseline,
        atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            l::InlineBreakOpportunity::Allowed,
        )),
        size: l::Size::new(px(10.0), px(10.0)),
        ..l::NodeInputOf::default()
    }
}

fn later_line_and_clearance<S: l::LayoutScalar>() {
    for clear in [false, true] {
        let mut later = block(100.0);
        later.clear = if clear {
            l::Clear::Both
        } else {
            l::Clear::None
        };
        let tree = FlowTree::<S>::new(
            vec![
                block(100.0),
                block(100.0),
                float(l::Float::Left, 20.0, 30.0),
                later,
                atomic(),
            ],
            &[(0, &[1, 3]), (1, &[2]), (3, &[4])],
        );
        let batch = tree.compute(&[0, 0, 0, 1, 0]);
        let entries = batch.unrounded_entries();
        assert_eq!(output(entries, 1).size.height, S::ZERO);
        assert_eq!(output(entries, 1).location, l::Point::ZERO);
        assert_eq!(output(entries, 2).location, l::Point::ZERO);
        assert_eq!(
            output(entries, 2).size,
            l::Size::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        // Without clear, only the line is shortened by the float; the later
        // ordinary block remains at the nominal block origin. Clear moves it
        // below the 30-unit float, so its local line starts at inline zero.
        assert_eq!(
            output(entries, 3).location,
            l::Point::new(S::ZERO, S::from_f64(if clear { 30.0 } else { 0.0 }))
        );
        assert_eq!(output(entries, 3).size.height, S::from_f64(10.0));
        assert_eq!(
            output(entries, 4).location,
            l::Point::new(S::from_f64(if clear { 0.0 } else { 20.0 }), S::ZERO)
        );
        assert_eq!(
            output(entries, 4).size,
            l::Size::new(S::from_f64(10.0), S::from_f64(10.0))
        );
        assert_eq!(
            output(entries, 0).size.height,
            S::from_f64(if clear { 40.0 } else { 30.0 })
        );
    }
}

#[test]
fn descendant_float_shortens_later_real_line_and_clear_moves_the_block_below_it() {
    later_line_and_clearance::<f32>();
    later_line_and_clearance::<f64>();
}

fn nested_shared_blocks<S: l::LayoutScalar>() {
    let tree = FlowTree::<S>::new(
        vec![
            block(100.0),
            block(100.0),
            block(100.0),
            float(l::Float::Left, 20.0, 30.0),
            block(100.0),
            atomic(),
        ],
        &[(0, &[1, 4]), (1, &[2]), (2, &[3]), (4, &[5])],
    );
    let batch = tree.compute(&[0, 0, 0, 0, 1, 0]);
    let entries = batch.unrounded_entries();
    for node in [1, 2] {
        assert_eq!(output(entries, node).size.height, S::ZERO);
        assert_eq!(output(entries, node).location, l::Point::ZERO);
    }
    assert_eq!(output(entries, 3).location, l::Point::ZERO);
    assert_eq!(output(entries, 4).location, l::Point::ZERO);
    assert_eq!(output(entries, 4).size.height, S::from_f64(10.0));
    assert_eq!(
        output(entries, 5).location,
        l::Point::new(S::from_f64(20.0), S::ZERO)
    );
    assert_eq!(output(entries, 0).size.height, S::from_f64(30.0));
}

#[test]
fn descendant_float_crosses_two_ordinary_ancestors_without_enlarging_either() {
    nested_shared_blocks::<f32>();
    nested_shared_blocks::<f64>();
}

fn relative_visual_offset<S: l::LayoutScalar>() {
    let mut shared = block(100.0);
    shared.inset.left =
        l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(7.0)).unwrap());
    shared.inset.top =
        l::LengthAutoOf::value(l::LengthPercentageOf::px(S::from_f64(11.0)).unwrap());
    let tree = FlowTree::<S>::new(
        vec![
            block(100.0),
            shared,
            float(l::Float::Left, 20.0, 30.0),
            block(100.0),
            atomic(),
        ],
        &[(0, &[1, 3]), (1, &[2]), (3, &[4])],
    );
    let batch = tree.compute(&[0, 0, 0, 1, 0]);
    let entries = batch.unrounded_entries();
    assert_eq!(output(entries, 1).size.height, S::ZERO);
    assert_eq!(
        output(entries, 1).location,
        l::Point::new(S::from_f64(7.0), S::from_f64(11.0))
    );
    assert_eq!(output(entries, 2).location, l::Point::ZERO);
    assert_eq!(output(entries, 3).location, l::Point::ZERO);
    assert_eq!(
        output(entries, 4).location,
        l::Point::new(S::from_f64(20.0), S::ZERO)
    );
    // Relative visual displacement changes neither normal flow nor used height.
    // Scroll/overflow bounds may still include the displaced descendant.
    assert_eq!(output(entries, 0).size.height, S::from_f64(30.0));
}

#[test]
fn relative_visual_offset_does_not_move_exported_nominal_float_exclusion() {
    relative_visual_offset::<f32>();
    relative_visual_offset::<f64>();
}

fn incoming_and_outgoing_floats<S: l::LayoutScalar>() {
    let tree = FlowTree::<S>::new(
        vec![
            block(100.0),
            float(l::Float::Left, 20.0, 30.0),
            block(100.0),
            float(l::Float::Right, 10.0, 20.0),
            block(100.0),
            atomic(),
        ],
        &[(0, &[1, 2, 4]), (2, &[3]), (4, &[5])],
    );
    let batch = tree.compute(&[0, 0, 1, 0, 2, 0]);
    let entries = batch.unrounded_entries();
    assert_eq!(output(entries, 1).location, l::Point::ZERO);
    assert_eq!(output(entries, 2).location, l::Point::ZERO);
    assert_eq!(output(entries, 2).size.height, S::ZERO);
    assert_eq!(
        output(entries, 3).location,
        l::Point::new(S::from_f64(90.0), S::ZERO)
    );
    assert_eq!(output(entries, 4).location, l::Point::ZERO);
    assert_eq!(
        output(entries, 5).location,
        l::Point::new(S::from_f64(20.0), S::ZERO)
    );
    assert_eq!(output(entries, 0).size.height, S::from_f64(30.0));
}

#[test]
fn existing_left_float_and_new_descendant_right_float_share_one_parent_context() {
    incoming_and_outgoing_floats::<f32>();
    incoming_and_outgoing_floats::<f64>();
}
