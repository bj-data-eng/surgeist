use super::*;

fn nested<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for parent_direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for child_direction in [l::Direction::Ltr, l::Direction::Rtl] {
            for (child_group, first_width) in [
                (l::TextGroupAlign::None, 60.0),
                (l::TextGroupAlign::End, 60.0),
                (l::TextGroupAlign::End, 80.0),
            ] {
                for reversed in [false, true] {
                    let mut root = block(mode, parent_direction, 100.0);
                    root.text_group_align = l::TextGroupAlign::Center;
                    let mut child = block(mode, child_direction, 100.0);
                    child.text_group_align = child_group;
                    let tree = Tree::<S>::new(
                        vec![
                            l::LayoutInputOf::box_input(root),
                            atomic(mode, parent_direction, 80.0),
                            forced(mode, parent_direction),
                            l::LayoutInputOf::box_input(child),
                            atomic(mode, child_direction, first_width),
                            forced(mode, child_direction),
                            atomic(mode, child_direction, 40.0),
                        ],
                        &[
                            (0, if reversed { &[3, 1, 2] } else { &[1, 2, 3] }),
                            (3, &[4, 5, 6]),
                        ],
                    );
                    let batch = tree.compute(mode, &[]);
                    // Parent's actual outside80 is the limiter: ancestor10/10.
                    // Child End spends only the remaining20 for a60/40 child;
                    // an80/40 child shares the original limiter and spends0.
                    let child_starts = match (child_group, first_width == 80.0, child_direction) {
                        (_, true, l::Direction::Ltr) => [10.0, 10.0],
                        (_, true, l::Direction::Rtl) => [10.0, 50.0],
                        (l::TextGroupAlign::None, false, l::Direction::Ltr) => [10.0, 10.0],
                        (l::TextGroupAlign::None, false, l::Direction::Rtl) => [30.0, 50.0],
                        (_, false, l::Direction::Ltr) => [30.0, 30.0],
                        (_, false, l::Direction::Rtl) => [10.0, 30.0],
                    };
                    for entries in [batch.unrounded_entries(), batch.final_entries()] {
                        assert_eq!(node(entries, 1).location.x, n(10.0));
                        assert_eq!(
                            node(entries, 3).location.x,
                            S::ZERO,
                            "group insets belong to lines, not the child block box"
                        );
                        assert_eq!(node(entries, 4).location.x, n(child_starts[0]));
                        assert_eq!(node(entries, 6).location.x, n(child_starts[1]));
                        assert_eq!(node(entries, 4).source_index, l::SourceIndex::new(0));
                        assert_eq!(node(entries, 5).source_index, l::SourceIndex::new(1));
                        assert_eq!(node(entries, 6).source_index, l::SourceIndex::new(2));
                        assert_eq!(
                            node(entries, 1).source_index,
                            l::SourceIndex::new(usize::from(reversed))
                        );
                        assert_eq!(node(entries, 4).size, l::Size::new(n(first_width), n(10.0)));
                        assert_eq!(node(entries, 6).size, l::Size::new(n(40.0), n(10.0)));
                        assert_eq!(node(entries, 6).location.y, n(10.0));
                    }
                }
            }
        }
    }
}
#[test]
fn nested_and_none_descendants_share_natural_basis_without_double_spending() {
    nested::<f32>();
    nested::<f64>();
}

fn boundaries<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    for kind in 0..5 {
        let mut root = block(mode, direction, 100.0);
        root.text_group_align = l::TextGroupAlign::Center;
        let mut boundary = block(mode, direction, 100.0);
        match kind {
            0 => {} // Real ordinary same-BFC descendant: its100 line limits P0.
            1 => boundary.block_flow_context = l::BlockFlowContext::Independent,
            2 => {
                boundary.overflow =
                    l::ComputedOverflow::try_new(l::Overflow::Hidden, l::Overflow::Hidden).unwrap()
            }
            3 => boundary.display = l::Display::Flex,
            4 => boundary.display = l::Display::Grid,
            _ => unreachable!(),
        }
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(root),
                atomic(mode, direction, 60.0),
                forced(mode, direction),
                l::LayoutInputOf::box_input(boundary),
                l::LayoutInputOf::box_input(block(mode, direction, 100.0)),
                atomic(mode, direction, 100.0),
            ],
            &[(0, &[1, 2, 3]), (3, &[4]), (4, &[5])],
        );
        let batch = tree.compute(mode, &[]);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                node(entries, 1).location.x,
                if kind == 0 { S::ZERO } else { n(20.0) }
            );
            assert_eq!(node(entries, 1).size, l::Size::new(n(60.0), n(10.0)));
            assert_eq!(node(entries, 5).location.x, S::ZERO);
            assert_eq!(node(entries, 5).size, l::Size::new(n(100.0), n(10.0)));
            assert_eq!(node(entries, 5).source_index, l::SourceIndex::new(0));
        }
    }
    // An independent atomic interior is excluded, but the actual80px exterior
    // remains the parent's limiter. The interior's100px overflow cannot erase P20.
    let mut tree = two_rows::<S>(
        mode,
        direction,
        l::TextGroupAlign::Center,
        l::TextAlign::Start,
    );
    tree.inputs[1] = atomic(mode, direction, 80.0);
    tree.inputs
        .push(l::LayoutInputOf::box_input(block(mode, direction, 100.0)));
    tree.inputs.push(atomic(mode, direction, 100.0));
    tree.children.extend([vec![5], vec![]]);
    tree.children[1] = vec![4];
    let batch = tree.compute(mode, &[]);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 1).location.x, n(10.0));
        assert_eq!(node(entries, 3).location.x, n(10.0));
        assert_eq!(node(entries, 5).location.x, S::ZERO);
        assert_eq!(node(entries, 1).size, l::Size::new(n(80.0), n(10.0)));
    }
}
#[test]
fn actual_context_dispatch_excludes_interiors_but_keeps_atomic_exteriors() {
    boundaries::<f32>();
    boundaries::<f64>();
}

fn independent_owner<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let mut root = block(mode, direction, 100.0);
    root.text_group_align = l::TextGroupAlign::Center;
    let mut child = block(mode, direction, 100.0);
    child.block_flow_context = l::BlockFlowContext::Independent;
    child.text_group_align = l::TextGroupAlign::End;
    let tree = Tree::<S>::new(
        vec![
            l::LayoutInputOf::box_input(root),
            atomic(mode, direction, 60.0),
            forced(mode, direction),
            l::LayoutInputOf::box_input(child),
            atomic(mode, direction, 40.0),
        ],
        &[(0, &[1, 2, 3]), (3, &[4])],
    );
    let batch = tree.compute(mode, &[]);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 1).location.x, n(20.0));
        // Own independent40px line spends its full60px residual once. It
        // cannot inherit ancestor20/20 or become a member of the outer group.
        assert_eq!(node(entries, 4).location.x, n(60.0));
        assert_eq!(node(entries, 3).location.x, S::ZERO);
        assert_eq!(node(entries, 4).source_index, l::SourceIndex::new(0));
    }
}
#[test]
fn independent_context_can_run_its_own_group_without_ancestor_insets() {
    independent_owner::<f32>();
    independent_owner::<f64>();
}

fn float_and_absolute<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let mut root = block(mode, direction, 100.0);
    root.text_group_align = l::TextGroupAlign::Center;
    let mut float = block(mode, direction, 20.0);
    float.float = l::Float::Right;
    float.size.height = px(10.0);
    let tree = Tree::<S>::new(
        vec![
            l::LayoutInputOf::box_input(root),
            atomic(mode, direction, 60.0),
            forced(mode, direction),
            l::LayoutInputOf::box_input(float),
            atomic(mode, direction, 50.0),
            l::LayoutInputOf::box_input(block(mode, direction, 100.0)),
            atomic(mode, direction, 100.0),
        ],
        &[(0, &[1, 2, 3, 4]), (3, &[5]), (5, &[6])],
    );
    let grouped = tree.compute(mode, &[]);
    let mut ordinary_tree = tree.clone();
    ordinary_tree.update(0, |root| root.text_group_align = l::TextGroupAlign::None);
    let ordinary = ordinary_tree.compute(mode, &[]);
    // Actual first line W100/N60 and second W80/N50 give min(40,30)=30.
    // The real right float is dispatched between the two inline runs at y10.
    for entries in [grouped.unrounded_entries(), grouped.final_entries()] {
        assert_eq!(node(entries, 1).location, l::Point::new(n(15.0), S::ZERO));
        assert_eq!(node(entries, 4).location, l::Point::new(n(15.0), n(10.0)));
        assert_eq!(node(entries, 3).location, l::Point::new(n(80.0), n(10.0)));
        assert_eq!(node(entries, 3).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(
            node(entries, 6).location.x,
            S::ZERO,
            "the float interior is its own collection"
        );
        assert_eq!(node(entries, 4).source_index, l::SourceIndex::new(3));
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
    }
    assert_eq!(
        node(ordinary.final_entries(), 3),
        node(grouped.final_entries(), 3)
    );
    let mut absolute = block(mode, direction, 100.0);
    absolute.position = l::Position::Absolute;
    absolute.size.height = px(10.0);
    absolute.inset.left = l::LengthAutoOf::ZERO;
    absolute.inset.top = l::LengthAutoOf::ZERO;
    let mut abs_tree = two_rows::<S>(
        mode,
        direction,
        l::TextGroupAlign::Center,
        l::TextAlign::Start,
    );
    abs_tree.inputs.extend([
        l::LayoutInputOf::box_input(absolute),
        atomic(mode, direction, 100.0),
    ]);
    abs_tree.children.extend([vec![5], vec![]]);
    abs_tree.children[0].insert(0, 4);
    let batch = abs_tree.compute(mode, &[]);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 1).location.x, n(20.0));
        assert_eq!(node(entries, 3).location.x, n(20.0));
        assert_eq!(node(entries, 4).location, l::Point::ZERO);
        assert_eq!(node(entries, 5).location.x, S::ZERO);
        assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(1));
    }
}
#[test]
fn real_narrowed_bands_and_outflow_interiors_keep_their_original_geometry() {
    float_and_absolute::<f32>();
    float_and_absolute::<f64>();
}

fn independent_semantics<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    assert_eq!(l::BlockFlowContext::default(), l::BlockFlowContext::Normal);
    for independent in [false, true] {
        let mut child = block(mode, direction, 100.0);
        child.block_flow_context = if independent {
            l::BlockFlowContext::Independent
        } else {
            l::BlockFlowContext::Normal
        };
        let mut float = block(mode, direction, 20.0);
        float.float = l::Float::Left;
        float.size.height = px(30.0);
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(block(mode, direction, 100.0)),
                l::LayoutInputOf::box_input(child),
                l::LayoutInputOf::box_input(float),
            ],
            &[(0, &[1]), (1, &[2])],
        );
        let batch = tree.compute(mode, &[]);
        assert_eq!(
            node(batch.final_entries(), 1).size.height,
            if independent { n(30.0) } else { S::ZERO }
        );
        assert_eq!(node(batch.final_entries(), 2).location, l::Point::ZERO);
        assert_eq!(
            node(batch.final_entries(), 2).size,
            l::Size::new(n(20.0), n(30.0))
        );
        assert_eq!(node(batch.final_entries(), 0).size.height, n(30.0));
        let mut root = block(mode, direction, 100.0);
        root.padding.top = l::LengthOf::value(l::LengthPercentageOf::px(n(1.0)).unwrap());
        let mut child = block(mode, direction, 100.0);
        child.block_flow_context = if independent {
            l::BlockFlowContext::Independent
        } else {
            l::BlockFlowContext::Normal
        };
        let mut grandchild = block(mode, direction, 100.0);
        grandchild.size.height = px(5.0);
        grandchild.margin.top = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(10.0)).unwrap());
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(root),
                l::LayoutInputOf::box_input(child),
                l::LayoutInputOf::box_input(grandchild),
            ],
            &[(0, &[1]), (1, &[2])],
        );
        let batch = tree.compute(mode, &[]);
        // Root padding1 prevents collapse with the root. A Normal child's
        // grandchild margin10 escapes to y11; an Independent child's margin
        // remains inside its actual15px height, starting at root content y1.
        assert_eq!(
            node(batch.final_entries(), 1).location.y,
            if independent { n(1.0) } else { n(11.0) }
        );
        assert_eq!(
            node(batch.final_entries(), 1).size.height,
            if independent { n(15.0) } else { n(5.0) }
        );
        assert_eq!(
            node(batch.final_entries(), 2).location.y,
            if independent { n(10.0) } else { S::ZERO }
        );
        assert_eq!(node(batch.final_entries(), 0).size.height, n(16.0));
    }
}
#[test]
fn independent_block_context_is_real_float_containment_not_only_group_filtering() {
    independent_semantics::<f32>();
    independent_semantics::<f64>();
}
