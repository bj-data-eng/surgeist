#![forbid(unsafe_code)]

//! A real earlier float excludes an indivisible numeric inline box until y20.
use surgeist_layout as l;

fn px(value: f32) -> l::PreferredSize {
    l::PreferredSize::value(l::LengthPercentageOf::px(value).unwrap())
}

struct Tree {
    inputs: [l::NodeInput; 3],
}
impl Tree {
    fn new() -> Self {
        Self {
            inputs: [
                l::NodeInput {
                    display: l::Display::Block,
                    size: l::Size::new(px(40.0), l::PreferredSize::AUTO),
                    ..l::NodeInput::default()
                },
                l::NodeInput {
                    display: l::Display::Block,
                    float: l::Float::Left,
                    size: l::Size::new(px(25.0), px(20.0)),
                    ..l::NodeInput::default()
                },
                l::NodeInput {
                    display: l::Display::InlineBlock,
                    atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
                        surgeist_text::BidiLevel::try_new(0).unwrap(),
                        l::InlineBreakOpportunity::Prohibited,
                    )),
                    size: l::Size::new(px(28.0), px(10.0)),
                    ..l::NodeInput::default()
                },
            ],
        }
    }
}
impl l::Traverse for Tree {
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
impl l::LayoutTree for Tree {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: usize) -> &l::NodeInput {
        &self.inputs[node]
    }
    fn layout_input(&self, node: usize) -> l::LayoutInputView<'_> {
        l::LayoutInputView::Box(&self.inputs[node])
    }
}

#[test]
fn indivisible_numeric_inline_box_waits_for_the_real_wider_band_below_an_earlier_float() {
    let tree = Tree::new();
    let request = l::LayoutRootRequest::viewport(l::Size::new(
        l::Available::definite(40.0),
        l::Available::MAX_CONTENT,
    ))
    .unwrap();
    let batch = l::compute_layout(&tree, 0, request).unwrap();
    let actual = |node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
    };
    assert_eq!(actual(1).location, l::Point::new(0.0, 0.0));
    assert_eq!(actual(1).size, l::Size::new(25.0, 20.0));
    assert_eq!(actual(2).size, l::Size::new(28.0, 10.0));
    // The preceding float leaves15px at y0; the indivisible28px box fits
    // the genuine40px band beginning at y20 without overlapping the float.
    assert_eq!(actual(2).location, l::Point::new(0.0, 20.0));
}
