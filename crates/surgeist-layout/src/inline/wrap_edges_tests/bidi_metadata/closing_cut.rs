//! A real closing BoxEdge belongs with its preceding selected box content.
use super::*;

fn positions<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> [(f64, f64); 6] {
    assert!(batch.placed_text().is_empty());
    std::array::from_fn(|index| {
        let actual = output(batch, 10 + index as u32);
        assert_eq!(actual.size, Size::new(n(10.0), n(10.0)));
        (actual.location.x.to_f64(), actual.location.y.to_f64())
    })
}

fn closing_cut<S: LayoutScalar>() {
    let mut f = split_box_fixture::<S>();
    let mut root = crate::LayoutTree::node_input(&f.tree, 0).clone();
    root.size.width = PreferredSizeOf::AUTO;
    f.tree.insert_input(0, LayoutInputOf::box_input(root));
    for index in 0..6u32 {
        let node = 10 + index;
        let mut actual = crate::LayoutTree::node_input(&f.tree, node).clone();
        let level = actual.atomic_inline_participation.unwrap().bidi_level();
        assert_eq!(level.get(), u8::from(index >= 3));
        let following = if matches!(index, 1 | 3) {
            InlineBreakOpportunity::Allowed
        } else {
            InlineBreakOpportunity::Prohibited
        };
        let participation = AtomicInlineParticipation::new(level, following);
        actual.atomic_inline_participation = Some(participation);
        f.tree.insert_input(node, LayoutInputOf::box_input(actual));
        let stored = f
            .participants
            .iter_mut()
            .find_map(|row| match row {
                MixedInlineParticipantOf::Atomic {
                    item,
                    participation,
                } if item.source_index == node as usize => Some(participation),
                _ => None,
            })
            .unwrap();
        *stored = participation;
    }
    assert_eq!(f.nodes, [10, 11, 20, 12, 13, 21, 14, 15]);
    let flow = f.flow();
    assert!(flow.paragraphs().is_empty());
    let wide = compute_layout_with_flow(&flow, 0, request(n(100.0))).unwrap();
    assert_eq!(
        positions(&wide),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (23.0, 0.0),
            (53.0, 0.0),
            (43.0, 0.0),
            (33.0, 0.0),
        ]
    );
    assert_eq!(output(&wide, 20).location.x, n(20.0));
    assert_eq!(output(&wide, 21).location.x, n(63.0));
    assert_eq!(output(&wide, 20).size.width, n(3.0));
    assert_eq!(output(&wide, 21).size.width, n(5.0));

    // Actual soft opportunities follow atomics1 and3. The second line must
    // retain End21: 3 + 10 + 10 + 5 = 28. Leaving End21 for the third line
    // gives it no selected owner and cannot authenticate an empty gap there.
    let narrow = compute_layout_with_flow(&flow, 0, request(n(30.0))).unwrap();
    assert_eq!(
        positions(&narrow),
        [
            (0.0, 0.0),
            (10.0, 0.0),
            (3.0, 10.0),
            (13.0, 10.0),
            (10.0, 20.0),
            (0.0, 20.0),
        ]
    );
    for (node, x, width) in [(20, 0.0, 3.0), (21, 23.0, 5.0)] {
        let actual = output(&narrow, node);
        assert_eq!(actual.location, Point::new(n(x), n(20.0)));
        assert_eq!(actual.size, Size::new(n(width), S::ZERO));
    }
    assert_eq!(positions(&wide)[3], (53.0, 0.0));
}

#[test]
fn native_closing_cut_keeps_end_edge_with_its_selected_content_f32() {
    closing_cut::<f32>();
}

#[test]
fn native_closing_cut_keeps_end_edge_with_its_selected_content_f64() {
    closing_cut::<f64>();
}
