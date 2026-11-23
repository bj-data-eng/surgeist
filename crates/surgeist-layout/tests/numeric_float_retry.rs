#![forbid(unsafe_code)]

//! A real earlier float excludes an indivisible numeric inline box until y20.
#[path = "support/native_request.rs"]
mod native_request;

use surgeist_layout as l;

fn px(value: f32) -> l::sizing::PreferredSize {
    l::sizing::PreferredSize::value(l::length::LengthPercentageOf::px(value).unwrap())
}

struct Tree {
    inputs: [l::node::NodeInput; 3],
}
impl Tree {
    fn new() -> Self {
        Self {
            inputs: [
                l::node::NodeInput {
                    display: l::box_model::Display::Block,
                    size: l::geometry::Size::new(px(40.0), l::sizing::PreferredSize::AUTO),
                    ..l::node::NodeInput::default()
                },
                l::node::NodeInput {
                    display: l::box_model::Display::Block,
                    float: l::box_model::Float::Left,
                    size: l::geometry::Size::new(px(25.0), px(20.0)),
                    ..l::node::NodeInput::default()
                },
                l::node::NodeInput {
                    display: l::box_model::Display::InlineBlock,
                    atomic_inline_participation: Some(l::inline::AtomicInlineParticipation::new(
                        surgeist_text::supplied_line::BidiLevel::try_new(0).unwrap(),
                        l::text_flow::InlineBreakOpportunity::Prohibited,
                    )),
                    size: l::geometry::Size::new(px(28.0), px(10.0)),
                    ..l::node::NodeInput::default()
                },
            ],
        }
    }
}
impl l::tree::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 {
            [1, 2].as_slice()
        } else {
            [].as_slice()
        }
        .iter()
        .copied()
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 2 } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!(node, 0);
        [1, 2][index]
    }
}
impl l::tree::LayoutTree for Tree {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: usize) -> &l::node::NodeInput {
        &self.inputs[node]
    }
    fn layout_input(&self, node: usize) -> l::node::LayoutInputView<'_> {
        l::node::LayoutInputView::Box(&self.inputs[node])
    }
}

#[test]
fn indivisible_numeric_inline_box_waits_for_the_real_wider_band_below_an_earlier_float() {
    let tree = Tree::new();
    let request = {
        let sizing_request = crate::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(40.0), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequest::standalone(
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
    let actual = |node| {
        batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .expect("fixture node produces one box")
            .unrounded_output()
    };
    assert_eq!(actual(1).location, l::geometry::Point::new(0.0, 0.0));
    assert_eq!(actual(1).size, l::geometry::Size::new(25.0, 20.0));
    assert_eq!(actual(2).size, l::geometry::Size::new(28.0, 10.0));
    // The preceding float leaves15px at y0; the indivisible28px box fits
    // the 40px band beginning at y20 without overlapping the float.
    assert_eq!(actual(2).location, l::geometry::Point::new(0.0, 20.0));
}
