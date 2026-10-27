//! A genuinely outside ordinary cut dominates the attractive inside tail repair.
use super::*;

fn scope_before_quality<S: l::LayoutScalar>() {
    with_style::<S>(l::TextWrapStyle::AvoidShortLastLine);
}
pub(super) fn with_style<S: l::LayoutScalar>(style: l::TextWrapStyle) {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        // Existing authentic source: Start,A30,B20,End,C10,D10. Desired
        // repair A/BCD crosses inside the real pair; ordinary ABC/D is outside.
        let tree = fixture::<S>(direction, style);
        for inside in [l::WrapInside::Auto, l::WrapInside::Avoid] {
            let repaired = inside == l::WrapInside::Auto;
            let batch = compute(&tree, inside);
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(entries.len(), 7);
                let receiving = node(entries, 6).location.y;
                assert!(receiving > S::ZERO);
                let x = match (direction, repaired) {
                    (l::Direction::Ltr, false) => [0.0, 30.0, 50.0, 0.0],
                    (l::Direction::Ltr, true) => [0.0, 0.0, 20.0, 30.0],
                    (l::Direction::Rtl, false) => [30.0, 10.0, 0.0, 50.0],
                    (l::Direction::Rtl, true) => [30.0, 40.0, 30.0, 20.0],
                };
                for (i, (id, width)) in [(2, 30.0), (3, 20.0), (5, 10.0), (6, 10.0)]
                    .into_iter()
                    .enumerate()
                {
                    let output = node(entries, id);
                    assert_eq!(output.source_index, l::SourceIndex::new(id as usize - 1));
                    assert_eq!(
                        output.size,
                        l::Size::new(S::from_f64(width), S::from_f64(10.0))
                    );
                    assert_eq!(output.location.x, S::from_f64(x[i]));
                    assert_eq!(
                        output.location.y,
                        if id == 6 || (repaired && id != 2) {
                            receiving
                        } else {
                            S::ZERO
                        }
                    );
                }
                let start = node(entries, 1);
                let end = node(entries, 4);
                for (id, output) in [(1, start), (4, end)] {
                    assert_eq!(output.source_index, l::SourceIndex::new(id as usize - 1));
                    assert_eq!(output.size, l::Size::ZERO);
                }
                // End really precedes C in this source. It stays with A/B
                // under Avoid, and with B/C/D after the legal Auto repair.
                if repaired {
                    assert!(end.location.y > start.location.y);
                    assert!(end.location.y >= receiving);
                } else {
                    assert_eq!(end.location.y, start.location.y);
                    assert!(end.location.y < receiving);
                }
                assert_eq!(
                    end.location.x,
                    S::from_f64(match (direction, repaired) {
                        (l::Direction::Ltr, false) => 50.0,
                        (l::Direction::Ltr, true) => 20.0,
                        (l::Direction::Rtl, false) => 10.0,
                        (l::Direction::Rtl, true) => 40.0,
                    })
                );
            }
            let ordinary = fixture::<S>(direction, l::TextWrapStyle::Auto);
            let ordinary = compute(&ordinary, inside);
            if !repaired {
                assert_eq!(ordinary.unrounded_entries(), batch.unrounded_entries());
                assert_eq!(ordinary.final_entries(), batch.final_entries());
            }
            let replay = compute(&tree, inside);
            assert_eq!(batch.unrounded_entries(), replay.unrounded_entries());
            assert_eq!(batch.final_entries(), replay.final_entries());
        }
    }
}
#[test]
fn avoid_short_last_line_scope_frontier_precedes_the_longer_tail_f32() {
    scope_before_quality::<f32>();
}
#[test]
fn avoid_short_last_line_scope_frontier_precedes_the_longer_tail_f64() {
    scope_before_quality::<f64>();
}
