//! Atomic Top/Bottom aligns actual margin edges in physical line directions.
use super::*;

fn single_signed_top<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    let direction = l::Direction::Ltr;
    let mut host = Host::<S>::new(root(mode, direction, l::InlineBaselineContext::Physical));
    let mut item = atomic(mode, direction, 10.0, (-2.0, 6.0));
    item.vertical_align = l::VerticalAlign::Top;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(item)), 1);
    let batch = host.compute(mode, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), 2);
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(14.0)));
        assert_eq!(node(entries, 1).size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(node(entries, 1).margin.top, n(-2.0));
        assert_eq!(node(entries, 1).margin.bottom, n(6.0));
        assert_eq!(node(entries, 0).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(0));
        absent(node(entries, 1).named_baselines);
        // Atomic bounds are its real margin box. Its14-unit span starts0;
        // border top−2 makes margin-over −2−(−2)=0 and margin-under14.
        assert_eq!(node(entries, 1).location, l::Point::new(S::ZERO, n(-2.0)));
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

fn vertical_lr_taller_peer<S: l::LayoutScalar>(alignment: l::VerticalAlign, expected_x: f64) {
    let mode = l::WritingMode::VerticalLr;
    let direction = l::Direction::Ltr;
    let mut host = Host::<S>::new(root(mode, direction, l::InlineBaselineContext::Physical));
    let mut item = atomic(mode, direction, 10.0, (0.0, 0.0));
    item.vertical_align = alignment;
    assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(item)), 1);
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::box_input(atomic(mode, direction, 30.0, (0.0, 0.0)))
        ),
        2
    );
    let batch = host.compute(mode, &[]).unwrap();
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), 3);
        assert_eq!(node(entries, 0).size, l::Size::new(n(30.0), n(100.0)));
        assert_eq!(node(entries, 2).location, l::Point::new(S::ZERO, n(20.0)));
        assert_eq!(node(entries, 2).size, l::Size::new(n(30.0), n(20.0)));
        assert_eq!(node(entries, 1).size, l::Size::new(n(10.0), n(20.0)));
        assert_eq!(node(entries, 1).margin, l::Edges::ZERO);
        assert_eq!(node(entries, 2).margin, l::Edges::ZERO);
        for (id, source) in [(0, 0), (1, 0), (2, 1)] {
            assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
        }
        absent(node(entries, 1).named_baselines);
        absent(node(entries, 2).named_baselines);
        // Real level0 Ltr source order gives A y0, peer y20. The peer's30
        // dominates the10-high group. WM4 line-over is RIGHT (A x20 for
        // Top), line-under LEFT (A x0 for Bottom), regardless of block-start.
        assert_eq!(
            node(entries, 1).location,
            l::Point::new(n(expected_x), S::ZERO)
        );
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

#[test]
fn atomic_top_signed_margin_aligns_actual_over_edge_f32() {
    single_signed_top::<f32>();
}
#[test]
fn atomic_top_signed_margin_aligns_actual_over_edge_f64() {
    single_signed_top::<f64>();
}
#[test]
fn vertical_lr_atomic_top_aligns_line_over_with_taller_peer_f32() {
    vertical_lr_taller_peer::<f32>(l::VerticalAlign::Top, 20.0);
}
#[test]
fn vertical_lr_atomic_top_aligns_line_over_with_taller_peer_f64() {
    vertical_lr_taller_peer::<f64>(l::VerticalAlign::Top, 20.0);
}
#[test]
fn vertical_lr_atomic_bottom_aligns_line_under_with_taller_peer_f32() {
    vertical_lr_taller_peer::<f32>(l::VerticalAlign::Bottom, 0.0);
}
#[test]
fn vertical_lr_atomic_bottom_aligns_line_under_with_taller_peer_f64() {
    vertical_lr_taller_peer::<f64>(l::VerticalAlign::Bottom, 0.0);
}
