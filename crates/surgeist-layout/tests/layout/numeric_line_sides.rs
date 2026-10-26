//! Numeric Left/Right alignment uses the line sides of the writing mode,
//! independently of the actual paragraph's LTR/RTL base direction.
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::BidiLevel;

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

fn node<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    id: u32,
) -> l::NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|entry| entry.node() == id);
    let output = matching.next().unwrap().output();
    assert!(matching.next().is_none(), "one output per actual node");
    output
}

fn assert_geometry<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    root_size: l::Size<S>,
    child_size: l::Size<S>,
    location: l::Point<S>,
    context: &str,
) {
    // Keep semantic physical geometry first, ahead of auxiliary custody checks.
    assert_eq!(
        node(batch.unrounded_entries(), 1).location,
        location,
        "{context}"
    );
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 1).location, location, "{context}");
        assert_eq!(node(entries, 1).size, child_size, "{context}");
        assert_eq!(node(entries, 0).size, root_size, "{context}");
        assert_eq!(node(entries, 0).location, l::Point::ZERO, "{context}");
        assert_eq!(entries.len(), 2, "{context}");
        for id in [0, 1] {
            assert_eq!(
                node(entries, id).source_index,
                l::SourceIndex::ZERO,
                "{context}"
            );
        }
    }
    assert!(batch.placed_text().is_empty(), "{context}");
    assert!(batch.unrounded_inline_fragments().is_empty(), "{context}");
    assert!(batch.final_inline_fragments().is_empty(), "{context}");
}

fn line_sides<S: l::LayoutScalar>(explicit_last: bool) {
    let n = S::from_f64;
    let px = |value| l::PreferredSizeOf::value(l::LengthPercentageOf::px(n(value)).unwrap());
    // WM4 Table 11: line-left/right are left/right in HorizontalTb,
    // top/bottom in VerticalRl/Lr and SidewaysRl, bottom/top in SidewaysLr.
    // Direction changes inline-start, but never changes these line sides.
    for (mode, root_size, child_size, left, right) in [
        (
            l::WritingMode::HorizontalTb,
            (100.0, 10.0),
            (30.0, 10.0),
            (0.0, 0.0),
            (70.0, 0.0),
        ),
        (
            l::WritingMode::VerticalRl,
            (10.0, 100.0),
            (10.0, 30.0),
            (0.0, 0.0),
            (0.0, 70.0),
        ),
        (
            l::WritingMode::VerticalLr,
            (10.0, 100.0),
            (10.0, 30.0),
            (0.0, 0.0),
            (0.0, 70.0),
        ),
        (
            l::WritingMode::SidewaysRl,
            (10.0, 100.0),
            (10.0, 30.0),
            (0.0, 0.0),
            (0.0, 70.0),
        ),
        (
            l::WritingMode::SidewaysLr,
            (10.0, 100.0),
            (10.0, 30.0),
            (0.0, 70.0),
            (0.0, 0.0),
        ),
    ] {
        let horizontal = mode == l::WritingMode::HorizontalTb;
        for direction in [l::Direction::Ltr, l::Direction::Rtl] {
            let cases = if explicit_last {
                [
                    (l::TextAlign::Left, l::TextAlignLast::Right, right, left),
                    (l::TextAlign::Right, l::TextAlignLast::Left, left, right),
                ]
            } else {
                [
                    (l::TextAlign::Left, l::TextAlignLast::Auto, left, right),
                    (l::TextAlign::Right, l::TextAlignLast::Auto, right, left),
                ]
            };
            for (align, last, expected, changed_expected) in cases {
                let root = l::NodeInputOf {
                    display: l::Display::Block,
                    writing_mode: mode,
                    direction,
                    text_align: align,
                    text_align_last: last,
                    size: if horizontal {
                        l::Size::new(px(100.0), l::PreferredSizeOf::AUTO)
                    } else {
                        l::Size::new(l::PreferredSizeOf::AUTO, px(100.0))
                    },
                    ..l::NodeInputOf::default()
                };
                let child = l::NodeInputOf {
                    display: l::Display::InlineBlock,
                    writing_mode: mode,
                    direction,
                    vertical_align: l::VerticalAlign::Baseline,
                    atomic_inline_participation: Some(l::AtomicInlineParticipation::new(
                        BidiLevel::try_new(u8::from(direction == l::Direction::Rtl)).unwrap(),
                        l::InlineBreakOpportunity::Allowed,
                    )),
                    size: if horizontal {
                        l::Size::new(px(30.0), px(10.0))
                    } else {
                        l::Size::new(px(10.0), px(30.0))
                    },
                    ..l::NodeInputOf::default()
                };
                let mut tree = Tree {
                    inputs: [root, child],
                    children: [vec![1], vec![]],
                };
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
                let context = format!(
                    "mode={mode:?}, direction={direction:?}, align={align:?}, last={last:?}"
                );
                let root_size = l::Size::new(n(root_size.0), n(root_size.1));
                let child_size = l::Size::new(n(child_size.0), n(child_size.1));
                let location = l::Point::new(n(expected.0), n(expected.1));
                let original = l::compute_layout(&tree, 0, request).unwrap();
                assert_geometry(&original, root_size, child_size, location, &context);

                // A genuine host input edit recomputes through the public owner;
                // the first owned output batch must remain immutable. This host
                // supplies no cache, so no warm-cache result is claimed.
                if explicit_last {
                    tree.inputs[0].text_align_last = if last == l::TextAlignLast::Left {
                        l::TextAlignLast::Right
                    } else {
                        l::TextAlignLast::Left
                    };
                } else {
                    tree.inputs[0].text_align = if align == l::TextAlign::Left {
                        l::TextAlign::Right
                    } else {
                        l::TextAlign::Left
                    };
                }
                let changed = l::compute_layout(&tree, 0, request).unwrap();
                assert_geometry(
                    &changed,
                    root_size,
                    child_size,
                    l::Point::new(n(changed_expected.0), n(changed_expected.1)),
                    &format!("changed host input from {context}"),
                );
                assert_geometry(&original, root_size, child_size, location, &context);
            }
        }
    }
}

#[test]
fn numeric_left_and_right_follow_line_sides_with_last_auto_f32() {
    line_sides::<f32>(false);
}

#[test]
fn numeric_left_and_right_follow_line_sides_with_last_auto_f64() {
    line_sides::<f64>(false);
}

#[test]
fn numeric_explicit_opposite_last_side_overrides_paragraph_alignment_f32() {
    line_sides::<f32>(true);
}

#[test]
fn numeric_explicit_opposite_last_side_overrides_paragraph_alignment_f64() {
    line_sides::<f64>(true);
}
