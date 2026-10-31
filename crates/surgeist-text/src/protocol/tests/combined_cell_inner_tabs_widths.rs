//! Fixed specified TAB metrics and candidate-local execution over a real width
//! font. Private child of combined_cell_inner_tabs; no TAB goes to Shaper in
//! these CssStops operations and no TAB action is inferred from a glyph.

use super::*;

#[path = "combined_cell_inner_tabs_resources.rs"]
mod resources;
use crate::shaping_test_fonts::{
    COMBINED_TAB_WIDTH_FAMILY, combined_tab_width_ahem, qualify_combined_tab_width,
};

type Raw = (usize, u32, f64, f32);

fn resources() -> (System, Vec<u8>) {
    let bytes = native_metric_font(&combined_tab_width_ahem());
    qualify_combined_tab_width(&bytes);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(parent_font()).unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 2);
    (system, bytes)
}

fn raw(line: &FinalizedLine, bytes: &[u8], cell: bool) -> Vec<Raw> {
    let mut result = Vec::new();
    for run in line.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert_eq!(run.cell().is_some(), cell);
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), 10.0);
        if cell {
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        }
        for glyph in run.glyphs() {
            assert_eq!(
                glyph.range(),
                Range::new(glyph.range().start, glyph.range().start + 1)
            );
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
            result.push((
                glyph.range().start,
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.advance(),
            ));
        }
    }
    result.sort_by_key(|row| row.0);
    result
}

fn ordinary_control(
    system: &mut System,
    bytes: &[u8],
    text: &str,
    size: TabSize,
    orientation: LineOrientation,
    expected: &[Raw],
    width: f64,
) {
    let ordinary = captured(
        text,
        size,
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::None,
        orientation,
        COMBINED_TAB_WIDTH_FAMILY,
    );
    assert_eq!(ordinary.analyzed.combined_cells().len(), 0);
    let prepared = system
        .prepare(&ordinary.analyzed, options(orientation))
        .unwrap();
    let line = complete(&prepared, TabContext::CssStops { origin: Some(0.0) });
    assert_eq!(line.feedback().advance, width);
    assert_eq!(raw(&line, bytes, false), expected);
}

fn observe(
    prepared: &PreparedText,
    capture: &Captured,
    line: FinalizedLine,
    bytes: &[u8],
    expected: &[Raw],
    width: f64,
) {
    assert_eq!(line.prepared_text(), *prepared);
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.measurements().fit().advance(), 10.0);
    assert_eq!(raw(&line, bytes, true), expected);
    assert!(
        line.run_views()
            .all(|run| run.glyphs().all(|glyph| glyph.range() != Range::new(1, 2))),
        "the genuine CSS TAB original has no provider glyph"
    );
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine outer Cell");
    };
    assert_eq!(
        cell.source_cell().scope(),
        capture.processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().origin.x, 0.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let orientation = prepared.orientation();
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation,
            atomics: vec![],
        })],
    })
    .unwrap();
    let square = square(orientation);
    assert_eq!(layout.bounds(), square);
    assert_eq!(layout.source(), capture.processed.source());
    assert_eq!(
        layout.analyzed_source().identity(),
        capture.analyzed.identity()
    );
    let scale = 10.0 / width;
    let mut physical = Vec::new();
    for run in layout.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        let frame = run.glyph_frame();
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        close(frame.outline_x().x, scale);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        close(frame.origin().y, square.origin.y + 8.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
            physical.push((
                glyph.id(),
                frame.origin().x + scale * glyph.pen(),
                glyph.advance(),
            ));
        }
    }
    physical.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    assert_eq!(physical.len(), expected.len());
    for (actual, &(_, glyph, pen, advance)) in physical.iter().zip(expected) {
        assert_eq!((actual.0, actual.2), (glyph, advance));
        close(actual.1, square.origin.x + scale * pen);
    }
    // Borrowed placements are source-free. The owned packet retains source
    // ranges alongside the same resource, advances and actual Cell frame.
    let borrowed = layout
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let owned = layout
        .glyph_runs()
        .into_iter()
        .filter(|run| !run.glyphs().is_empty())
        .collect::<Vec<_>>();
    assert_eq!(borrowed.len(), owned.len());
    for (view, packet) in borrowed.iter().zip(&owned) {
        assert_eq!(packet.font().bytes(), bytes);
        assert_eq!(packet.range(), view.range());
        assert_eq!(packet.glyph_frame(), view.glyph_frame());
        assert_eq!(
            packet
                .glyphs()
                .iter()
                .map(|g| (g.id(), g.pen(), g.advance(), g.x(), g.y()))
                .collect::<Vec<_>>(),
            view.glyphs()
                .map(|g| (g.id(), g.pen(), g.advance(), g.x(), g.y()))
                .collect::<Vec<_>>()
        );
    }
    let mut owned_source = owned
        .iter()
        .flat_map(|packet| packet.glyphs().iter())
        .map(|glyph| (glyph.range().start, glyph.id(), glyph.advance()))
        .collect::<Vec<_>>();
    owned_source.sort_by_key(|row| row.0);
    assert_eq!(
        owned_source,
        expected
            .iter()
            .map(|&(source, glyph, _, advance)| (source, glyph, advance))
            .collect::<Vec<_>>()
    );
    let start = Cursor::new(0, Affinity::After);
    let end = Cursor::new(3, Affinity::Before);
    let positions = layout.positions().collect::<Vec<_>>();
    assert_eq!(
        positions
            .iter()
            .map(PlacedPosition::source_cursor)
            .collect::<Vec<_>>(),
        [start, end]
    );
    let ys = if orientation == LineOrientation::SidewaysCounterclockwise {
        [19.0, 9.0]
    } else {
        [20.0, 30.0]
    };
    for (cursor, y) in [start, end].into_iter().zip(ys) {
        assert_eq!(
            layout.cursor(cursor).unwrap().rect(),
            Rect::new(square.origin.x, y, 10.0, 1.0)
        );
    }
    let selected = layout
        .position_selection(&positions[0], &positions[1])
        .unwrap();
    assert_eq!(selected.source_text(), capture.processed.source().text());
    assert_eq!(selected.source_pieces().len(), 3);
    for (piece, source) in selected.source_pieces().iter().zip(0..3) {
        assert_eq!(piece.local_range(), Range::new(source, source + 1));
        assert_eq!(piece.aggregate_range(), Range::new(source, source + 1));
        assert_eq!(
            piece.origin().view().capture_position(),
            Some(capture.processed.captured_origin(source).unwrap())
        );
    }
}

fn fixed_reference(orientation: LineOrientation) {
    let (mut system, bytes) = resources();
    // No Cell: the genuine nearest Root Ahem20 supplies SPACE20/T40; A0/A40
    // give50. The Cell below must use Inline10/T20, not that outer reference.
    ordinary_control(
        &mut system,
        &bytes,
        "A\tA",
        TabSize::spaces(2.0).unwrap(),
        orientation,
        &[(0, 35, 0.0, 10.0), (2, 35, 40.0, 10.0)],
        50.0,
    );
    let capture = captured(
        "A\tA",
        TabSize::spaces(2.0).unwrap(),
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        orientation,
        COMBINED_TAB_WIDTH_FAMILY,
    );
    source_premises(
        &capture,
        "A\tA",
        ParagraphBaseDirection::LeftToRight,
        &[0, 0, 0],
    );
    let prepared = system.prepare(&capture.analyzed,options(orientation))
        .expect("mixed provider/TAB width candidates must execute against the fixed specified Inline reference");
    assert!(prepared.cell_views().next().unwrap().has_provider_input());
    assert_eq!(prepared.cell_views().next().unwrap().extent(), 10.0);
    let line = complete(&prepared, TabContext::NoTabs);
    // Specified normal SPACE10 and ZERO10 stay fixed. Normal30/hwid25/twid24/
    // qwid23 are all overE10. The selected qwid glyphs retain advance3 at0/20;
    // scale10/23, rather than the wrongly re-queried qwid grid6/width9/scale1.
    let expected = [(0, 16, 0.0, 3.0), (2, 16, 20.0, 3.0)];
    observe(&prepared, &capture, line.clone(), &bytes, &expected, 23.0);
    for origin in [None, Some(700.0)] {
        observe(
            &prepared,
            &capture,
            complete(&prepared, TabContext::CssStops { origin }),
            &bytes,
            &expected,
            23.0,
        );
    }
    drop(system);
    observe(
        &prepared,
        &capture,
        line.with_spacing(SpacingAllocations::None).unwrap(),
        &bytes,
        &expected,
        23.0,
    );
}

fn partial_provider(orientation: LineOrientation) {
    let (mut system, bytes) = resources();
    ordinary_control(
        &mut system,
        &bytes,
        "A\tB",
        TabSize::spaces(2.0).unwrap(),
        orientation,
        &[(0, 35, 0.0, 10.0), (2, 36, 40.0, 5.0)],
        45.0,
    );
    let capture = captured(
        "A\tB",
        TabSize::spaces(2.0).unwrap(),
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        orientation,
        COMBINED_TAB_WIDTH_FAMILY,
    );
    source_premises(
        &capture,
        "A\tB",
        ParagraphBaseDirection::LeftToRight,
        &[0, 0, 0],
    );
    let prepared = system
        .prepare(&capture.analyzed, options(orientation))
        .expect("a valid TAB proof cannot certify the unacted B provider root");
    // The font qualifier proved actual A Covered/B NoAction for all three
    // plans, even when hwid's A output equals B36. TAB owns no provider outcome.
    // All trials are ineligible: NORMAL A35/B36 retains10/5 at0/20, W25.
    let expected = [(0, 35, 0.0, 10.0), (2, 36, 20.0, 5.0)];
    let line = complete(&prepared, TabContext::NoTabs);
    observe(&prepared, &capture, line.clone(), &bytes, &expected, 25.0);
    drop(system);
    observe(
        &prepared,
        &capture,
        line.with_spacing(SpacingAllocations::None).unwrap(),
        &bytes,
        &expected,
        25.0,
    );
}

#[test]
fn vertical_width_trials_keep_specified_space_and_ch_grid_fixed() {
    fixed_reference(LineOrientation::Vertical);
}
#[test]
fn clockwise_width_trials_keep_specified_space_and_ch_grid_fixed() {
    fixed_reference(LineOrientation::SidewaysClockwise);
}
#[test]
fn counterclockwise_width_trials_keep_specified_space_and_ch_grid_fixed() {
    fixed_reference(LineOrientation::SidewaysCounterclockwise);
}
#[test]
fn vertical_tab_layout_proof_cannot_cover_the_unacted_b_root() {
    partial_provider(LineOrientation::Vertical);
}
#[test]
fn clockwise_tab_layout_proof_cannot_cover_the_unacted_b_root() {
    partial_provider(LineOrientation::SidewaysClockwise);
}
#[test]
fn counterclockwise_tab_layout_proof_cannot_cover_the_unacted_b_root() {
    partial_provider(LineOrientation::SidewaysCounterclockwise);
}

#[test]
fn a_partial_provider_candidate_must_still_report_its_real_tab_precision_failure() {
    let (mut system, bytes) = resources();
    let period = 2.0_f64.powi(57);
    // At this binade the lower spacing is16. Normal prefix B5+A10=15 is
    // retained in the displacement (rounded T-16 !=T). qwid B5+A3=8 is exactly
    // a ties-to-even loss: T-8 ==T, so the common checked subtraction refuses.
    let previous = f64::from_bits(period.to_bits() - 1);
    assert_eq!(period - previous, 16.0);
    assert_eq!(period - 15.0, previous);
    assert_eq!(period - 8.0, period);
    // A real ordinary CSS-stop control qualifies the consumed normal prefix
    // and last-TAB geometry before the Cell's trial obligation is demanded.
    ordinary_control(
        &mut system,
        &bytes,
        "BA\t",
        TabSize::length(period).unwrap(),
        LineOrientation::Vertical,
        &[(0, 36, 0.0, 5.0), (1, 35, 5.0, 10.0)],
        period,
    );
    let capture = captured(
        "BA\t",
        TabSize::length(period).unwrap(),
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        LineOrientation::Vertical,
        COMBINED_TAB_WIDTH_FAMILY,
    );
    // Preserved trailing TAB must be present in the actual completed used
    // source, not inferred from raw text or substituted with a provider glyph.
    source_premises(
        &capture,
        "BA\t",
        ParagraphBaseDirection::LeftToRight,
        &[0, 0, 0],
    );
    let error = system
        .prepare(&capture.analyzed, options(LineOrientation::Vertical))
        .expect_err(
            "insufficient B coverage must not short-circuit a constructed qwid TAB execution",
        );
    assert!(
        matches!(error.detail(),Some(ErrorDetail::InvalidTextNumeric {
        field:"tab grid displacement",phase:crate::TextNumericPhase::Preparation,
        category:crate::TextNumericError::NonZeroLost,value,
    }) if *value == -8.0),
        "unexpected preparation error: {error:?}"
    );
}
