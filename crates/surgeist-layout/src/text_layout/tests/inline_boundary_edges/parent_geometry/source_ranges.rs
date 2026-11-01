//! Source ranges retain authored ownership after physical edge placement.
//! Uses that module's genuine capture/prepare/Layout constructors.
//! Independently derived Ahem10 packet positions: LTR4..14, RTL6..16.

use super::*;

fn source_rects<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    fixture: &Fixture,
) -> Vec<text::Rect> {
    let [placed] = batch.placed_text() else {
        panic!("one real placed paragraph")
    };
    let layout = placed.layout();
    assert_eq!(layout.source().text().code_units(), [65]);
    let origin = fixture.binding.origins[0].origin.clone();
    let query = layout
        .source_range_fragments(&origin, text::Range::new(0, 1))
        .unwrap();
    assert_eq!(query.origin(), &origin);
    assert_eq!(query.local_range(), text::Range::new(0, 1));
    assert_eq!(
        query.units().len(),
        1,
        "edges never become authored characters"
    );
    let unit = query.units().next().unwrap();
    assert_eq!(unit.line_index(), 0);
    assert!(unit.cell_unit().is_none());
    assert_eq!(
        unit.finalized_line().prepared_text(),
        fixture.binding.prepared
    );
    unit.pieces().map(|piece| piece.rect()).collect()
}

fn lane<S: LayoutScalar>() {
    for rtl in [false, true] {
        for captured_child in [false, true] {
            let fixture = Fixture::nested_parent("A", rtl, false, captured_child);
            source_premises(&fixture, "A", rtl, captured_child);
            let ordinary = compute::<S>(&fixture, rtl, 10.0, captured_child, false);
            let shifted = compute::<S>(&fixture, rtl, 20.0, captured_child, true);
            let expected = |x| {
                vec![text::Rect {
                    origin: text::Point::new(x, 0.0),
                    size: text::Size::new(10.0, 10.0),
                }]
            };
            assert_eq!(source_rects(&ordinary, &fixture), expected(0.0));
            assert_eq!(
                source_rects(&shifted, &fixture),
                expected(if rtl { 6.0 } else { 4.0 })
            );
            // New placement and query allocation cannot rewrite a prior result.
            assert_eq!(source_rects(&ordinary, &fixture), expected(0.0));
        }
    }
}

#[test]
fn original_source_ranges_follow_actual_parent_edge_packet_geometry_f32() {
    lane::<f32>();
}

#[test]
fn original_source_ranges_follow_actual_parent_edge_packet_geometry_f64() {
    lane::<f64>();
}
