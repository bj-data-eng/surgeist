//! Public numeric qualification of actual horizontal word halves in Vertical
//! Cells. This child inherits the signed fixture's real NBSP153 font premises.

use super::*;

fn prepare_nbsp(
    font: &[u8],
    word: f64,
    options: PreparationOptions,
) -> crate::Result<(Arc<ProcessedSource>, PreparedText)> {
    let (processed, analyzed) = captured("\u{a0}", WhiteSpace::Preserve, word);
    source_custody(&processed, &analyzed, &[0], word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.to_vec()).unwrap();
    let prepared = system.prepare(&analyzed, options)?;
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    assert!(prepared.cell_views().next().unwrap().has_provider_input());
    Ok((processed, prepared))
}

fn vertical() -> PreparationOptions {
    PreparationOptions::default().with_orientation(LineOrientation::Vertical)
}

fn place_at_x(
    prepared: &PreparedText,
    line: &FinalizedLine,
    baseline_x: f64,
) -> crate::Result<Layout> {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: baseline_x,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
}

fn numeric(
    error: Error,
    field: &'static str,
    phase: TextNumericPhase,
    category: TextNumericError,
    value: f64,
) {
    assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
    match error.detail() {
        Some(ErrorDetail::InvalidTextNumeric {
            field: actual_field,
            phase: actual_phase,
            category: actual_category,
            value: actual_value,
        }) => {
            assert_eq!(*actual_field, field);
            assert_eq!(*actual_phase, phase);
            assert_eq!(*actual_category, category);
            assert_eq!(*actual_value, value);
        }
        detail => panic!("expected the actual numeric owner, got {detail:?}"),
    }
}

#[test]
fn a_locally_retained_cell_word_half_must_survive_its_physical_horizontal_axis() {
    let font = qualified_nbsp_font(false);
    let (processed, prepared) = prepare_nbsp(&font, 2.0, vertical()).unwrap();
    let line = complete_line(&prepared);
    // Intrinsic10 + WS2 gives logical0..12 and raw1..11. W12 compresses
    // by5/6, with zero centering. This avoids an earlier centering-loss check
    // hiding the actual word-half qualification at the large physical anchor.
    output(
        &prepared,
        &line,
        &processed,
        &font,
        &Expected {
            word: 2.0,
            raw: 10.0,
            pen: 1.0,
            terminal: 12.0,
            occupied: (0.0, 12.0),
            scale: 5.0 / 6.0,
            offset: 0.0,
            physical: 95.0 + 5.0 / 6.0,
        },
    );
    let retained = place_at_x(&prepared, &line, 100.0).unwrap();
    let packets = retained.glyph_runs();
    let bounds = retained.bounds();
    // The qualified symmetric BASE metric gives the E10 square x=B-5..B+5.
    // f64 spacing at this finite x is2; the scaled nonzero half5/6 is lost.
    let large = 10_000_000_000_000_000.0_f64;
    let square_x = large - 5.0;
    assert!(large.is_finite());
    assert_eq!(square_x + 5.0 / 6.0, square_x);
    let error = place_at_x(&prepared, &line, large).unwrap_err();
    numeric(
        error,
        "placed spacing displacement",
        TextNumericPhase::Placement,
        TextNumericError::NonZeroLost,
        1.0,
    );
    assert_eq!(retained.glyph_runs(), packets);
    assert_eq!(retained.bounds(), bounds);
    one_em(&line);
}

#[test]
fn a_genuine_intrinsic_zero_nbsp_requests_no_word_half_at_the_large_x() {
    let font = qualified_nbsp_font(true);
    let tiny = f64::from_bits(1);
    for word in [2.0, tiny] {
        let (processed, prepared) = prepare_nbsp(&font, word, vertical()).unwrap();
        let line = complete_line(&prepared);
        output(
            &prepared,
            &line,
            &processed,
            &font,
            &Expected {
                word,
                raw: 0.0,
                pen: 0.0,
                terminal: 0.0,
                occupied: (0.0, 0.0),
                scale: 1.0,
                offset: 5.0,
                physical: 100.0,
            },
        );
        // Source and provider input remain nonempty. The intrinsic-zero gate,
        // rather than blank ink or a guessed zero half, makes this a no-op.
        let placed = place_at_x(&prepared, &line, 10_000_000_000_000_000.0).unwrap();
        let packets = placed.glyph_runs();
        let packets = packets
            .iter()
            .filter(|run| !run.glyphs().is_empty())
            .collect::<Vec<_>>();
        let [packet] = packets.as_slice() else {
            panic!("one actual NBSP provider packet");
        };
        assert_eq!(packet.glyph_frame().outline_x(), Point::new(1.0, 0.0));
        assert!(packet.glyph_frame().origin().x.is_finite());
        assert_eq!(
            packet
                .glyphs()
                .iter()
                .map(|g| (g.id(), g.pen(), g.advance()))
                .collect::<Vec<_>>(),
            [(153, 0.0, 0.0)]
        );
        one_em(&line);
    }
}

#[test]
fn a_tiny_real_authored_word_term_refuses_before_a_half_can_be_manufactured() {
    let font = qualified_nbsp_font(false);
    let tiny = f64::from_bits(1);
    assert!(tiny != 0.0 && tiny.is_finite());
    assert_eq!(tiny * 0.5, 0.0);
    for word in [tiny, -tiny] {
        assert_eq!(10.0 + word, 10.0);
        let error = prepare_nbsp(&font, word, vertical()).unwrap_err();
        // The public font has nonzero intrinsic10. Canonical logical addition
        // therefore detects loss before the later half-underflow kernel. This
        // is not a claim that public inputs reached that later kernel.
        numeric(
            error,
            "word spacing",
            TextNumericPhase::Shaping,
            TextNumericError::NonZeroLost,
            word,
        );
    }
    let (_, control) = prepare_nbsp(&font, 0.0, vertical()).unwrap();
    one_em(&complete_line(&control));
}

#[test]
fn a_finite_authored_word_term_that_overflows_public_scaling_is_refused() {
    let font = qualified_nbsp_font(false);
    let word = f64::MAX;
    assert!(word.is_finite());
    assert!((word * 2.0).is_infinite());
    let options = PreparationOptions::try_new(2.0, TabPreparation::FontAdvance)
        .unwrap()
        .with_orientation(LineOrientation::Vertical);
    let error = prepare_nbsp(&font, word, options).unwrap_err();
    numeric(
        error,
        "word spacing",
        TextNumericPhase::Preparation,
        TextNumericError::NonFinite,
        f64::INFINITY,
    );
    // This qualifies the actual public scale boundary, not a hypothetical
    // overflow in a later occupied-interval or width-trial computation.
}
