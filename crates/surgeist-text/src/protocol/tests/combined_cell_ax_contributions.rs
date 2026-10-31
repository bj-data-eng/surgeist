//! Actual per-original Cell contributions, expansion, inner bidi and native TABs.

#![cfg(feature = "text-accessibility")]

use super::*;
use crate::engine::layout::data::{ClusterContent, FinalizedRunFrame};
use crate::protocol::tests::combined_cells::{
    inner_tabs as tabs, widths::tracking_payload as tracking,
};
use skrifa::MetadataProvider;

const PARENT: NodeId = NodeId(1);

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "actual{actual:?}, expected{expected:?}"
    );
}

fn source_controls(analyzed: &AnalyzedSource, expected: &[&str]) -> Accessibility {
    let mut state = Accessibility::new(90_000);
    let update = analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let units = original_units(&update, PARENT);
    assert_eq!(
        units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        expected
    );
    for (_, node) in carriers(&update, PARENT) {
        assert_eq!(node.bounds(), None);
        assert_eq!(node.character_positions(), None);
        assert_eq!(node.character_widths(), None);
        assert_eq!(node.font_family(), None);
        assert_eq!(node.font_size(), None);
    }
    // Every ORIGINAL host unit in these deliberately small fixtures occupies
    // one UTF16 unit, including sharp-s (whose AX UTF8 character length is2).
    for (index, unit) in units.iter().enumerate() {
        let a = Cursor::new(index, Affinity::After);
        let b = Cursor::new(index + 1, Affinity::Before);
        assert_eq!(analyzed.access_position(&state, a).unwrap(), Some(unit.1));
        assert_eq!(analyzed.access_position(&state, b).unwrap(), Some(unit.2));
        assert_eq!(
            analyzed
                .access_selection(&state, Selection::new(a, b))
                .unwrap(),
            Some(TextSelection {
                anchor: unit.1,
                focus: unit.2
            })
        );
    }
    state
}

// Bounds are independent per-ORIGINAL contribution oracles. Valid carriers can
// group original units, so compare each actual carrier with the union of ONLY
// those units. Missing contribution stays None; it is not a zero rectangle.
fn placed_controls(
    layout: &Layout,
    state: &mut Accessibility,
    expected: &[&str],
    bounds: &[Option<Rect>],
) -> TreeUpdate {
    assert_eq!(expected.len(), bounds.len());
    let update = layout
        .accessibility_update(state, PARENT, PARENT, Point::new(0.0, 0.0))
        .expect("placed AX must project actual original contribution ownership");
    let units = original_units(&update, PARENT);
    assert_eq!(
        units.iter().map(|unit| unit.0).collect::<Vec<_>>(),
        expected
    );
    let mut first = 0;
    for (_, node) in carriers(&update, PARENT) {
        let end = first + node.character_lengths().len();
        assert!(end <= bounds.len());
        let mut rects = bounds[first..end].iter().flatten();
        if let Some(rect) = rects.next() {
            let (mut x0, mut y0) = (rect.origin.x, rect.origin.y);
            let (mut x1, mut y1) = (x0 + rect.size.width, y0 + rect.size.height);
            for rect in rects {
                x0 = x0.min(rect.origin.x);
                y0 = y0.min(rect.origin.y);
                x1 = x1.max(rect.origin.x + rect.size.width);
                y1 = y1.max(rect.origin.y + rect.size.height);
            }
            let actual = node
                .bounds()
                .expect("this original has actual retained contributions");
            near(actual.x0, x0);
            near(actual.y0, y0);
            near(actual.x1, x1);
            near(actual.y1, y1);
        } else {
            assert_eq!(
                node.bounds(),
                None,
                "no Cell-square/strut/neighbor donation"
            );
        }
        assert_eq!(node.character_positions(), None);
        assert_eq!(node.character_widths(), None);
        first = end;
    }
    assert_eq!(first, expected.len());
    for (index, unit) in units.iter().enumerate() {
        let a = Cursor::new(index, Affinity::After);
        let b = Cursor::new(index + 1, Affinity::Before);
        assert_eq!(layout.access_position(state, a).unwrap(), Some(unit.1));
        assert_eq!(layout.access_position(state, b).unwrap(), Some(unit.2));
        assert_eq!(
            layout
                .access_selection(state, Selection::new(a, b))
                .unwrap(),
            Some(TextSelection {
                anchor: unit.1,
                focus: unit.2
            })
        );
    }
    update
}

#[test]
fn uppercase_sharp_s_ax_deduplicates_actual_original_ancestry_and_unions_both_s_contributions() {
    // Existing factory REALLY authors Uppercase on the captured All Inline.
    // It qualifies S53/1000 normal and Complete/NoAction on all three width
    // features using this exact maintained native-table font, before Text use.
    let (mut system, fixture) =
        tracking::fixture_with_processing("ß", 0.0, false, false, 10.0, WhiteSpace::Preserve, true);
    assert_eq!(fixture.processed.source().text(), &Text::from("ß"));
    assert_eq!(fixture.processed.processed_text(), &Text::from("ß"));
    assert_eq!(fixture.analyzed.combined_cells().len(), 1);
    let source = fixture.analyzed.combined_cells().next().unwrap();
    assert_eq!(
        (source.selection_unit_count(), source.used_unit_count()),
        (2, 2)
    );
    assert_eq!(
        source.presentation_code_points().collect::<Vec<_>>(),
        [0x53, 0x53]
    );
    let ancestry = source.source_pieces().collect::<Vec<_>>();
    assert_eq!(ancestry.len(), 2);
    for (subposition, piece) in ancestry.iter().enumerate() {
        assert_eq!(piece.source(), fixture.processed.source());
        assert_eq!(piece.local(), Range::new(0, 1));
        assert_eq!(piece.aggregate(), Range::new(0, 1));
        assert_eq!(
            piece.origin_position(),
            fixture.processed.captured_origin(0).unwrap()
        );
        assert_eq!(piece.emission_subposition(), Some(subposition));
        assert!(piece.is_used() && piece.has_presentation_emission());
    }
    let mut state = source_controls(&fixture.analyzed, &["ß"]);
    // The decoder above proves original valueß and UTF8 length[2], neither
    // transformed SS nor two duplicate originalß selectable characters.
    let prepared = system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = super::super::super::complete_line(&prepared);
    one_em(&line);
    let raw = line
        .run_views()
        .flat_map(|run| {
            assert!(run.face().is_same_face(&fixture.body.face()));
            assert_eq!(run.font_size(), 10.0);
            let instance = run.effective_instance().unwrap();
            let metric = instance
                .metrics(instance.face().catalogue_identity())
                .unwrap();
            assert_eq!((metric.ascent(), metric.descent()), (8.0, -2.0));
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        g.id(),
                        run.origin().x + g.pen(),
                        g.advance(),
                        g.x(),
                        g.y(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        [
            (Range::new(0, 1), 53, 0.0, 10.0, 0.0, 0.0),
            (Range::new(0, 1), 53, 10.0, 10.0, 0.0, 0.0),
        ]
    );
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), fixture.processed.source());
    let packets = layout.glyph_runs();
    let [packet] = packets.as_slice() else {
        panic!("one actual SS provider packet")
    };
    let frame = packet.glyph_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.outline_x(), Point::new(0.5, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    // Real metric rectangles are[95,100]x[20,30] and[100,105]x[20,30].
    // Their SAME originalß gets their union, not duplicated semantics.
    placed_controls(
        &layout,
        &mut state,
        &["ß"],
        &[Some(Rect::new(95.0, 20.0, 10.0, 10.0))],
    );
}

#[test]
fn rtl_inner_tab_groups_reverse_bounds_without_reversing_original_ax_units() {
    // Maintained genuine RTL native fixture: UAX9 W7/I2 gives AB and12 level2;
    // TAB(S) resets to paragraph1. Real L2 is physical12,TAB,AB. This is not
    // a caller paint rotation or a guessed reversal of the LTR glyphs in AB.
    let bytes = tabs::qualified_font();
    let capture = tabs::captured(
        "AB\t12",
        TabSize::length(20.0).unwrap(),
        20.0,
        ParagraphBaseDirection::RightToLeft,
        TextCombineUpright::All,
        LineOrientation::Vertical,
        "Ahem",
    );
    let source = capture.analyzed.combined_cells().next().unwrap();
    assert_eq!(capture.analyzed.combined_cells().len(), 1);
    assert_eq!(
        source.inner_direction(),
        ParagraphBaseDirection::RightToLeft
    );
    assert_eq!(
        source
            .inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        [2, 2, 1, 2, 2]
    );
    assert_eq!(
        (source.selection_unit_count(), source.used_unit_count()),
        (5, 5)
    );
    assert_eq!(
        source.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x42, 9, 0x31, 0x32]
    );
    let mut state = source_controls(&capture.analyzed, &["A", "B", "\t", "1", "2"]);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let options = PreparationOptions::try_new(1.0, TabPreparation::CssStops)
        .unwrap()
        .with_orientation(LineOrientation::Vertical);
    let prepared = system.prepare(&capture.analyzed, options).unwrap();
    let line = super::super::super::complete_line(&prepared);
    one_em(&line);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(
        layout.analyzed_source().identity(),
        capture.analyzed.identity()
    );
    let face = skrifa::FontRef::new(&bytes).unwrap();
    let mut physical = Vec::new();
    for run in layout.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert_eq!(run.face().bytes(), bytes.as_slice());
        assert_eq!(run.font_size(), 10.0);
        let frame = run.glyph_frame();
        near(frame.outline_x().x, 1.0 / 6.0);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        near(frame.origin().y, 28.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.advance(), glyph.x(), glyph.y()), (10.0, 0.0, 0.0));
            physical.push((
                glyph.id(),
                frame.origin().x + frame.outline_x().x * glyph.pen(),
            ));
        }
    }
    physical.sort_by(|a, b| a.1.total_cmp(&b.1));
    assert_eq!(physical.len(), 4);
    for ((id, x), (ch, pen)) in
        physical
            .iter()
            .zip([('1', 0.0), ('2', 10.0), ('A', 40.0), ('B', 50.0)])
    {
        assert_eq!(*id, face.charmap().map(ch).unwrap().to_u32());
        near(*x, 95.0 + pen / 6.0);
    }
    // This concrete ordinal belongs to an executed native operation, not a
    // fabricated font/glyph or provider-atoms/source-hull membership guess.
    let data = &layout.inner.lines[0].finalized.data;
    let [cell] = data.cells.as_slice() else {
        panic!("one actual RTL Cell")
    };
    let [tab] = cell.tabs.as_slice() else {
        panic!("one executed RTL TAB")
    };
    assert_eq!(tab.atom, 2);
    // RTL retains signed displacement; its physical interval still spans20.
    assert_eq!(tab.amount, -20.0);
    assert_eq!(tab.after - tab.before, tab.amount);
    assert!(matches!(
        data.clusters[tab.cluster].content,
        ClusterContent::CssTab(_)
    ));
    assert_eq!(data.clusters[tab.cluster].content.font_run(), None);
    // Source-order rectangles, independently derived from real physical L2:
    // 1@0,2@10,TAB20..40,A@40,B@50, W60 compressed into E10.
    placed_controls(
        &layout,
        &mut state,
        &["A", "B", "\t", "1", "2"],
        &[
            Some(Rect::new(95.0 + 40.0 / 6.0, 20.0, 10.0 / 6.0, 10.0)),
            Some(Rect::new(95.0 + 50.0 / 6.0, 20.0, 10.0 / 6.0, 10.0)),
            Some(Rect::new(95.0 + 20.0 / 6.0, 20.0, 20.0 / 6.0, 10.0)),
            Some(Rect::new(95.0, 20.0, 10.0 / 6.0, 10.0)),
            Some(Rect::new(95.0 + 10.0 / 6.0, 20.0, 10.0 / 6.0, 10.0)),
        ],
    );
}

#[test]
fn removed_only_space_cell_keeps_original_ax_semantics_without_donating_any_geometry_or_font() {
    // Existing actual Source inner-edge removal, NOT a Phase-I deleted origin
    // or an invented semantic-zero provider scalar.
    let (processed, analyzed) = super::super::super::captured(" ", WhiteSpace::Collapse, 0.0);
    let source = analyzed.combined_cells().next().unwrap();
    assert_eq!(analyzed.combined_cells().len(), 1);
    assert_eq!(source.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        (source.selection_unit_count(), source.used_unit_count()),
        (1, 0)
    );
    assert_eq!(source.presentation_code_points().len(), 0);
    let ancestry = source.source_pieces().collect::<Vec<_>>();
    let [piece] = ancestry.as_slice() else {
        panic!("one real removed original SPACE")
    };
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.local(), Range::new(0, 1));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(0).unwrap()
    );
    assert!(!piece.is_used() && !piece.has_presentation_emission());
    let mut state = source_controls(&analyzed, &[" "]);
    let bytes = super::super::super::qualified_font();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert!(!prepared.cell_views().next().unwrap().has_provider_input());
    let line = super::super::super::complete_line(&prepared);
    one_em(&line);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
    assert!(layout.glyph_runs().is_empty());
    assert!(layout.clusters().is_empty());
    let update = placed_controls(&layout, &mut state, &[" "], &[None]);
    for (_, node) in carriers(&update, PARENT) {
        assert_eq!(node.font_family(), None);
        assert_eq!(node.font_size(), None);
    }
}

#[test]
fn native_css_tab_ax_uses_its_actual_nonprovider_band_and_retains_genuine_zero_width() {
    // ch10 gives half-ch5. A period6 gap meets that floor without another
    // period: actual intrinsic0..6 fits E10 with scale1/centering2. Zero period
    // is the separately qualified native0..0 operation, centered by5; unlike
    // removed SPACE it contributes a real zero-width rectangle.
    let bytes = tabs::qualified_font();
    for (period, offset, expected) in [
        (6.0, 2.0, Rect::new(97.0, 20.0, 6.0, 10.0)),
        (0.0, 5.0, Rect::new(100.0, 20.0, 0.0, 10.0)),
    ] {
        let capture = tabs::captured(
            "\t",
            TabSize::length(period).unwrap(),
            20.0,
            ParagraphBaseDirection::LeftToRight,
            TextCombineUpright::All,
            LineOrientation::Vertical,
            "Ahem",
        );
        let source = capture.analyzed.combined_cells().next().unwrap();
        assert_eq!(source.scope(), capture.processed.captured_scope(1).unwrap());
        assert_eq!(
            (source.selection_unit_count(), source.used_unit_count()),
            (1, 1)
        );
        assert_eq!(source.presentation_code_points().collect::<Vec<_>>(), [9]);
        let pieces = source.source_pieces().collect::<Vec<_>>();
        let [piece] = pieces.as_slice() else {
            panic!("one actual native TAB original")
        };
        assert_eq!(
            piece.origin_position(),
            capture.processed.captured_origin(0).unwrap()
        );
        assert!(piece.is_used() && piece.has_presentation_emission());
        let mut state = source_controls(&capture.analyzed, &["\t"]);
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(bytes.clone()).unwrap();
        let prepared = system
            .prepare(
                &capture.analyzed,
                PreparationOptions::try_new(1.0, TabPreparation::CssStops)
                    .unwrap()
                    .with_orientation(LineOrientation::Vertical),
            )
            .unwrap();
        let cell = prepared.cell_views().next().unwrap();
        assert_eq!(cell.extent(), 10.0);
        assert!(!cell.has_provider_input());
        let line = super::super::super::complete_line(&prepared);
        one_em(&line);
        let layout = placed_vertical_cell(&prepared, line);
        assert!(layout.glyph_runs().is_empty());
        assert_eq!(layout.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
        let data = &layout.inner.lines[0].finalized.data;
        assert!(data.runs.is_empty() && data.font_runs.is_empty());
        let [cell] = data.cells.as_slice() else {
            panic!("one genuine provider-free native Cell")
        };
        assert!(cell.frame.is_none(), "no glyph/body baseline exists");
        let transform = cell.transform.expect("real independent native layout");
        assert_eq!((transform.x_scale, transform.x_offset), (1.0, offset));
        let [tab] = cell.tabs.as_slice() else {
            panic!("one actual native operation")
        };
        assert_eq!(
            (tab.atom, tab.before, tab.after, tab.amount),
            (0, 0.0, period, period)
        );
        let cluster = &data.clusters[tab.cluster];
        assert_eq!(cluster.source, Range::new(0, 1));
        assert_eq!(cluster.frame, FinalizedRunFrame::Cell(0));
        assert!(matches!(cluster.content, ClusterContent::CssTab(_)));
        assert_eq!(cluster.content.font_run(), None);
        let update = placed_controls(&layout, &mut state, &["\t"], &[Some(expected)]);
        for (_, node) in carriers(&update, PARENT) {
            assert_eq!(
                node.font_family(),
                None,
                "native reference is not a dummy font association"
            );
            assert_eq!(node.font_size(), None);
        }
    }
}
