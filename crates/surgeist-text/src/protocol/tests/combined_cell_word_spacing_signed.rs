//! Signed inner word terms use the union of logical endpoints and raw advance
//! coverage, before the real fixed square's centering/compression. NBSP survives
//! Preserve: blank ink and zero advance never imply absent source/provider input.

#[path = "combined_cell_word_spacing_resources.rs"]
mod resources;

#[path = "combined_cell_word_spacing_precision.rs"]
mod precision;

use super::*;
use skrifa::{
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

fn qualified_nbsp_font(zero: bool) -> Vec<u8> {
    let original = parent_font();
    let bytes = if zero {
        let mut hmtx = sfnt::table(&original, *b"hmtx").to_vec();
        assert_eq!(sfnt::u16_at(&hmtx, 153 * 4), 1000);
        hmtx[153 * 4..153 * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
        let edited = sfnt::replace_tables(&original, vec![(*b"hmtx", hmtx.clone())]);
        assert_eq!(sfnt::table(&edited, *b"hmtx"), hmtx);
        assert_eq!(
            &hmtx[..153 * 4],
            &sfnt::table(&original, *b"hmtx")[..153 * 4]
        );
        assert_eq!(
            &hmtx[153 * 4 + 2..],
            &sfnt::table(&original, *b"hmtx")[153 * 4 + 2..]
        );
        // The runtime-only metric derivative retains every other actual table.
        // SFNT assembly alone also updates the head checksum adjustment.
        for ordinal in 0..usize::from(sfnt::u16_at(&original, 4)) {
            let record = 12 + ordinal * 16;
            let tag: [u8; 4] = original[record..record + 4].try_into().unwrap();
            if tag == *b"hmtx" {
                continue;
            }
            let before = sfnt::table(&original, tag);
            let after = sfnt::table(&edited, tag);
            if tag == *b"head" {
                assert_eq!(&after[..8], &before[..8]);
                assert_eq!(&after[12..], &before[12..]);
            } else {
                assert_eq!(after, before);
            }
        }
        edited
    } else {
        original
    };
    glyph_frame_font_premises(&bytes);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.charmap().map(' '), Some(GlyphId::new(153)));
    let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    let advance = if zero { 0 } else { 1000 };
    assert_eq!(
        metrics.advance_width(GlyphId::new(153)),
        Some(if zero { 0.0 } else { 1000.0 })
    );
    // Static glyf metrics return a valid zero box for an empty glyph. None
    // denotes unavailable/invalid metrics, not the absence of contour bytes.
    assert_eq!(sfnt::u16_at(sfnt::table(&bytes, *b"head"), 50), 0);
    let loca = sfnt::table(&bytes, *b"loca");
    assert_eq!(
        [
            u32::from(sfnt::u16_at(loca, 153 * 2)) * 2,
            u32::from(sfnt::u16_at(loca, 154 * 2)) * 2
        ],
        [3912, 3912]
    );
    let bounds = metrics
        .bounds(GlyphId::new(153))
        .expect("valid empty NBSP metric bounds");
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0; 4]
    );
    let font = ShaperFontRef::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    for request in [
        None,
        Some((*b"hwid", WidthFeature::HalfWidth)),
        Some((*b"twid", WidthFeature::ThirdWidth)),
        Some((*b"qwid", WidthFeature::QuarterWidth)),
    ] {
        let mut input = UnicodeBuffer::new();
        input.add(' ', 0);
        input.set_direction(Direction::LeftToRight);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        let features = if let Some((tag, feature)) = request {
            input.observe_width_feature(feature);
            vec![Feature::new(Tag::new(&tag), 1, ..)]
        } else {
            vec![]
        };
        let output = shaper.shape(input, &features);
        assert_eq!(output.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            output
                .glyph_infos()
                .iter()
                .map(|info| (info.glyph_id, info.cluster))
                .collect::<Vec<_>>(),
            [(153, 0)]
        );
        assert_eq!(
            output
                .glyph_positions()
                .iter()
                .map(|position| (
                    position.x_advance,
                    position.y_advance,
                    position.x_offset,
                    position.y_offset
                ))
                .collect::<Vec<_>>(),
            [(advance, 0, 0, 0)]
        );
        if let Some((_, feature)) = request {
            let receipt = output.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.feature(), feature);
            assert_eq!(receipt.input_len(), 1);
            assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
            assert_eq!(
                receipt.input(0).unwrap().outcome(),
                WidthInputOutcome::NoAction
            );
            assert!(receipt.input(1).is_none());
        } else {
            assert!(output.width_feature_receipt().unwrap().is_none());
        }
    }
    bytes
}

struct Expected {
    word: f64,
    raw: f32,
    pen: f64,
    terminal: f64,
    occupied: (f64, f64),
    scale: f64,
    offset: f64,
    physical: f64,
}

fn output(
    prepared: &PreparedText,
    line: &FinalizedLine,
    processed: &ProcessedSource,
    font: &[u8],
    expected: &Expected,
) {
    one_em(line);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual NBSP Cell");
    };
    assert_eq!(cell.square().origin.x, 0.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.scope_displacement().get(), 0.0);
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    let views = line
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let [view] = views.as_slice() else {
        panic!("one real provider NBSP run, even with blank ink/zero advance");
    };
    assert!(view.cell().is_some());
    assert_eq!(view.face().bytes(), font);
    assert_eq!(view.font_size(), 10.0);
    assert_eq!(view.glyph_basis(), GlyphBasis::Unrotated);
    let local_origin = view.origin().x;
    assert_eq!(
        view.glyphs()
            .map(|glyph| (
                glyph.range(),
                glyph.id(),
                local_origin + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 153, expected.pen, 0.0, 0.0, expected.raw)]
    );
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), processed.source());
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let packets = layout
        .glyph_runs()
        .into_iter()
        .filter(|run| !run.glyphs().is_empty())
        .collect::<Vec<_>>();
    let [packet] = packets.as_slice() else {
        panic!("actual owned NBSP glyph packet");
    };
    assert_eq!(packet.font().bytes(), font);
    assert_eq!(packet.font_size(), 10.0);
    let frame = packet.glyph_frame();
    close(frame.outline_x().x, expected.scale);
    assert_eq!(frame.outline_x().y, 0.0);
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    close(
        frame.origin().x,
        95.0 + expected.offset + expected.scale * local_origin,
    );
    close(frame.origin().y, 28.0);
    let [glyph] = packet.glyphs() else {
        panic!("one retained canonical provider glyph");
    };
    assert_eq!(
        (
            glyph.range(),
            glyph.id(),
            glyph.x(),
            glyph.y(),
            glyph.advance()
        ),
        (Range::new(0, 1), 153, 0.0, 0.0, expected.raw)
    );
    close(
        frame.origin().x + expected.scale * glyph.pen(),
        expected.physical,
    );
}

fn case(font: &[u8], expected: Expected) {
    // These are independent model operands, not nonexistent public getters.
    // The signed terminal is not the occupied width: include both logical
    // endpoints and the actual glyph advance interval before centering.
    let raw_end = expected.pen + f64::from(expected.raw);
    assert_eq!(
        expected.occupied,
        (
            0.0_f64.min(expected.terminal).min(expected.pen),
            0.0_f64.max(expected.terminal).max(raw_end)
        )
    );
    let width = expected.occupied.1 - expected.occupied.0;
    close(
        expected.scale,
        if width > 10.0 { 10.0 / width } else { 1.0 },
    );
    close(
        expected.offset,
        (10.0 - width * expected.scale) / 2.0 - expected.occupied.0 * expected.scale,
    );
    let (processed, analyzed) = captured(" ", WhiteSpace::Preserve, expected.word);
    assert_eq!(processed.source().text(), &Text::from(" "));
    assert_eq!(processed.source().text().len(), 1); // UTF-16, not its two UTF-8 bytes.
    source_custody(&processed, &analyzed, &[0], expected.word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.to_vec()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("source-nonempty NBSP preparation");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    assert_eq!(cell.source_cell().selection_unit_count(), 1);
    assert_eq!(
        cell.source_cell().used_code_points().collect::<Vec<_>>(),
        [0xa0]
    );
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(metrics.effective_instance().unwrap().face().bytes(), font);
    let line = complete_line(&prepared);
    output(&prepared, &line, &processed, font, &expected);
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    source_custody(
        &processed,
        &replay.prepared_text().analyzed_source(),
        &[0],
        expected.word,
    );
    output(&prepared, &replay, &processed, font, &expected);
}

#[test]
fn surviving_nbsp_positive_word_term_is_measured_before_compression() {
    let font = qualified_nbsp_font(false);
    // Logical0..14 plus raw2..12 gives W14, scale10/14, offset0.
    case(
        &font,
        Expected {
            word: 4.0,
            raw: 10.0,
            pen: 2.0,
            terminal: 14.0,
            occupied: (0.0, 14.0),
            scale: 5.0 / 7.0,
            offset: 0.0,
            physical: 95.0 + 10.0 / 7.0,
        },
    );
}

#[test]
fn negative_nbsp_word_terms_center_actual_signed_coverage_at_fixed_one_em() {
    let font = qualified_nbsp_font(false);
    for expected in [
        // Terminal8 does not compress raw-1..9: occupied width10, offset1.
        Expected {
            word: -2.0,
            raw: 10.0,
            pen: -1.0,
            terminal: 8.0,
            occupied: (-1.0, 9.0),
            scale: 1.0,
            offset: 1.0,
            physical: 95.0,
        },
        // Zero terminal is still real ten-unit provider advance coverage.
        Expected {
            word: -10.0,
            raw: 10.0,
            pen: -5.0,
            terminal: 0.0,
            occupied: (-5.0, 5.0),
            scale: 1.0,
            offset: 5.0,
            physical: 95.0,
        },
        // Logical-20..0 exceeds raw-15..-5 and therefore sets occupied width20.
        Expected {
            word: -30.0,
            raw: 10.0,
            pen: -15.0,
            terminal: -20.0,
            occupied: (-20.0, 0.0),
            scale: 0.5,
            offset: 10.0,
            physical: 97.5,
        },
    ] {
        case(&font, expected);
    }
}

#[test]
fn intrinsic_zero_nbsp_with_positive_word_spacing_keeps_real_provider_input_without_a_term() {
    let font = qualified_nbsp_font(true);
    // Actual intrinsic0 suppresses the WS4 term. The nonempty blank glyph stays
    // a real provider payload, centered at the middle of E10 with scale1.
    case(
        &font,
        Expected {
            word: 4.0,
            raw: 0.0,
            pen: 0.0,
            terminal: 0.0,
            occupied: (0.0, 0.0),
            scale: 1.0,
            offset: 5.0,
            physical: 100.0,
        },
    );
}
