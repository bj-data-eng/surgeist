//! An earlier outside cut dominates Stable's later fitting inside-scope cut.
use super::*;

fn assert_hull<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
    ids: &[u32],
    direction: l::Direction,
    width: f64,
) {
    let actual = ids.iter().map(|&id| node(entries, id)).collect::<Vec<_>>();
    let left = actual
        .iter()
        .map(|item| item.location.x)
        .reduce(S::min)
        .unwrap();
    let right = actual
        .iter()
        .map(|item| item.location.x + item.size.width)
        .reduce(S::max)
        .unwrap();
    let used = actual
        .iter()
        .fold(S::ZERO, |sum, item| sum + item.size.width);
    let start = if direction == l::Direction::Ltr {
        0.0
    } else {
        60.0 - width
    };
    assert_eq!(used, S::from_f64(width));
    assert_eq!(left, S::from_f64(start));
    assert_eq!(right, S::from_f64(start + width));
}

fn earlier_outside_cut_dominates<S: l::LayoutScalar>(direction: l::Direction) {
    for style in [l::TextWrapStyle::Auto, l::TextWrapStyle::Stable] {
        let mut tree = fixture::<S>(direction, style);
        // Real source order A30, Start, B20, C10, End, D10 puts the late
        // ABC60 cut INSIDE NativePair(1,4). The earlier A30 cut is outside.
        // Avoid therefore requires A / BCD = 30/40 before local quality; with
        // ordinary scope permission, Stable takes ABC / D = 60/10.
        tree.children[0] = vec![2, 1, 3, 5, 4, 6];
        assert_eq!(tree.inputs[0].as_box().unwrap().text_wrap_style, style);
        for inside in [l::WrapInside::Auto, l::WrapInside::Avoid] {
            let avoided = inside == l::WrapInside::Avoid;
            let batch = compute(&tree, inside);
            let prior = batch.clone();
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(entries.len(), 7);
                let root = node(entries, 0);
                assert_eq!(root.size.width, S::from_f64(60.0));
                assert_eq!(root.source_index, l::SourceIndex::new(0));
                for (id, source) in [(2, 0), (1, 1), (3, 2), (5, 3), (4, 4), (6, 5)] {
                    assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
                }
                let receiving_y = node(entries, 6).location.y;
                assert!(receiving_y > S::ZERO);
                let expected_x = match (direction, avoided) {
                    (l::Direction::Ltr, false) => [0.0, 30.0, 50.0, 0.0],
                    (l::Direction::Ltr, true) => [0.0, 0.0, 20.0, 30.0],
                    (l::Direction::Rtl, false) => [30.0, 10.0, 0.0, 50.0],
                    (l::Direction::Rtl, true) => [30.0, 40.0, 30.0, 20.0],
                };
                for (index, (id, width)) in [(2, 30.0), (3, 20.0), (5, 10.0), (6, 10.0)]
                    .into_iter()
                    .enumerate()
                {
                    let actual = node(entries, id);
                    assert_eq!(
                        actual.size,
                        l::Size::new(S::from_f64(width), S::from_f64(10.0))
                    );
                    assert_eq!(actual.location.x, S::from_f64(expected_x[index]));
                    let receiving = id == 6 || (avoided && id != 2);
                    assert_eq!(
                        actual.location.y,
                        if receiving { receiving_y } else { S::ZERO }
                    );
                }
                let first: &[u32] = if avoided { &[2] } else { &[2, 3, 5] };
                let second: &[u32] = if avoided { &[3, 5, 6] } else { &[6] };
                assert_hull(entries, first, direction, if avoided { 30.0 } else { 60.0 });
                assert_hull(
                    entries,
                    second,
                    direction,
                    if avoided { 40.0 } else { 10.0 },
                );

                let start = node(entries, 1);
                let end = node(entries, 4);
                assert_eq!(start.size, l::Size::ZERO);
                assert_eq!(end.size, l::Size::ZERO);
                // Preserve the selected half-open source prefix. Without
                // Avoid the cut after C precedes End, so End starts D's row;
                // with Avoid, Start/B/C/End/D all follow the cut after A.
                // Both controls attach to their selected row's baseline.
                // The actual empty10-high baseline atoms supply offset10;
                // this does not assert total line heights with8/2 controls.
                assert!(start.location.y > S::ZERO);
                assert_eq!(
                    start.location.y,
                    node(entries, 3).location.y + S::from_f64(10.0)
                );
                assert!(end.location.y >= start.location.y);
                assert!(end.location.y > node(entries, 5).location.y);
                assert!(end.location.y <= root.size.height);
                assert_eq!(end.location.y, receiving_y + S::from_f64(10.0));
                assert_eq!(
                    end.location.x,
                    S::from_f64(match (direction, avoided) {
                        (l::Direction::Ltr, false) => 0.0,
                        (l::Direction::Rtl, false) => 60.0,
                        (_, true) => 30.0,
                    })
                );
            }
            assert!(batch.placed_text().is_empty());
            assert!(batch.unrounded_inline_fragments().is_empty());
            assert!(batch.final_inline_fragments().is_empty());
            let replay = compute(&tree, inside);
            assert_eq!(batch.unrounded_entries(), replay.unrounded_entries());
            assert_eq!(batch.final_entries(), replay.final_entries());
            assert_eq!(batch.unrounded_entries(), prior.unrounded_entries());
            assert_eq!(batch.final_entries(), prior.final_entries());
        }
    }
}

#[test]
fn stable_scope_avoidance_prefers_an_earlier_outside_cut_to_the_late_inside_cut() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        earlier_outside_cut_dominates::<f32>(direction);
        earlier_outside_cut_dominates::<f64>(direction);
    }
}
