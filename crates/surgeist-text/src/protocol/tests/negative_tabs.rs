//! Existing-callable positive qualification for spacing-derived negative CSS tabs.
use super::*;
use crate::{LineHeight, PlaintextItem, Range, TabPreparation, TabSize};
use skrifa::MetadataProvider;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));

fn actual_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 20.0,
        line_height: LineHeight::Absolute(20.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn input(
    text: &str,
    own: Style,
    block: Option<Style>,
    direction: ParagraphBaseDirection,
) -> ParagraphInput {
    let source = Source::identified(text, crate::Id::from_u64(64091), 6);
    let length = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: own,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: block
            .map(|style| crate::ParagraphTabReference::new(style, direction).unwrap()),
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn prepare(value: ParagraphInput, demand: TabPreparation) -> crate::Result<PreparedText> {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value)?,
        crate::Wrap::Word,
    );
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(AHEM.to_vec()).unwrap();
    s.prepare(&analyzed, PreparationOptions::try_new(1.0, demand)?)
}

fn finish(p: &PreparedText, origin: Option<f64>, indent: f64) -> crate::Result<FinalizedLine> {
    let mut r = request(p);
    r.context.tabs = TabContext::CssStops { origin };
    r.context.base_direction = if p.analyzed_source().direction() == crate::Direction::RightToLeft {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    r.context.indent = indent;
    r.context.available_inline = None;
    r.context.alignment = crate::Alignment::Left;
    r.context.spacing = SpacingPolicy::None;
    p.select(r)?.finalize(SpacingAllocations::None)
}

fn qualify_font_and_signed_space() {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.charmap().map(0x20_u32).unwrap().to_u32(), 3);
    assert_eq!(font.charmap().map(0x30_u32).unwrap().to_u32(), 18);
    // A separate real FontAdvance source qualifies resource coverage and the
    // actual negative SPACE contribution. It is not a fake CSS-tab measurement.
    let spaced = Style {
        word_spacing: -21.0,
        ..actual_style()
    };
    let p = prepare(
        input(" 0", spaced, None, ParagraphBaseDirection::LeftToRight),
        TabPreparation::FontAdvance,
    )
    .unwrap();
    let mut r = request(&p);
    r.context.tabs = TabContext::FontAdvance;
    r.context.spacing = SpacingPolicy::None;
    r.context.available_inline = None;
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 19.0); // (20 - 21) + 20
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.effective_instance().unwrap().font_size(), 20.0);
        glyphs.extend(run.glyphs().map(|g| {
            (
                g.id(),
                g.advance(),
                run.origin().x + g.pen() + f64::from(g.x()),
            )
        }));
    }
    assert_eq!(glyphs, [(3, 20.0, 0.0), (18, 20.0, -1.0)]);
    assert_eq!(p.analyzed_source().source().text().code_units(), [32, 48]);
}

fn css_tab(word_spacing: f64, direction: ParagraphBaseDirection) -> crate::Result<PreparedText> {
    let own = Style {
        tab_size: TabSize::spaces(4.0).unwrap(),
        ..actual_style()
    };
    let block = Style {
        word_spacing,
        ..actual_style()
    };
    prepare(
        input("\t", own, Some(block), direction),
        TabPreparation::CssStops,
    )
}

fn assert_metric(p: &PreparedText, signed_period: f64, threshold: f64) {
    let metrics: Vec<_> = p.inner.tab_metrics.iter().flatten().collect();
    assert_eq!(metrics.len(), 1);
    let metric = metrics[0];
    assert_eq!(
        metric.period, signed_period,
        "retain the resolved signed period, not its grid magnitude"
    );
    assert_eq!(metric.threshold, threshold);
    let space = metric
        ._block_font
        .as_ref()
        .expect("number-sized tab retains its actual block SPACE resource");
    assert_eq!(space.font.face().bytes(), AHEM);
    assert_eq!(space.font_size, 20.0);
    if signed_period != 0.0 {
        let zero = metric
            .font
            .as_ref()
            .expect("nonzero tab retains its actual ZERO metric");
        assert_eq!(zero.font.face().bytes(), AHEM);
        assert_eq!(zero.font_size, 20.0);
    } else {
        assert!(
            metric.font.is_none(),
            "spacing-derived zero does not consume ZERO"
        );
    }
}

fn assert_source_and_geometry(p: &PreparedText, line: FinalizedLine, advance: f64) {
    assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0,
        "a CSS tab has no provider TAB glyph"
    );
    let plaintext: Vec<_> = p
        .analyzed_source()
        .processed_source()
        .plaintext_items(crate::Affinity::Before)
        .flat_map(|item| match item {
            PlaintextItem::Text(text) => text.code_units().to_vec(),
            PlaintextItem::InlineBox(_) => panic!("no boxes"),
        })
        .collect();
    assert_eq!(plaintext, [9]);
    let origin = p.origin_views().next().unwrap().handle();
    assert_eq!(
        origin.view().source().id(),
        Some(crate::Id::from_u64(64091))
    );
    assert_eq!(origin.view().source().revision(), 6);
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 11.0,
                baseline: 30.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    let observed = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(observed.units().count(), 1);
    assert_eq!(
        observed
            .units()
            .flat_map(|u| u.pieces().collect::<Vec<_>>())
            .map(|piece| piece.rect().size.width)
            .sum::<f64>(),
        advance
    );
    assert!(
        layout
            .cursor(crate::Cursor::new(0, crate::Affinity::After))
            .is_ok()
    );
    assert!(
        layout
            .cursor(crate::Cursor::new(1, crate::Affinity::Before))
            .is_ok()
    );
}

fn literal_case(direction: ParagraphBaseDirection, origin: f64, pen: f64, destination: f64) {
    qualify_font_and_signed_space(); // reaches actual resource facts before the intended capability failure
    let p = css_tab(-21.0, direction).expect(
        "valid spacing-derived negative tab period must execute its forward magnitude grid",
    );
    assert_metric(&p, -4.0, 10.0);
    let sign = if direction == ParagraphBaseDirection::LeftToRight {
        1.0
    } else {
        -1.0
    };
    // Temporary signed tab pen = sign*indent; use an actual negative indent to
    // express physical pen+1 in RTL rather than silently replacing it by -1.
    let line = finish(&p, Some(origin), sign * pen).unwrap();
    let expected_advance = sign * (destination - pen);
    assert!(expected_advance > 0.0);
    assert_eq!(pen + sign * line.feedback().advance, destination);
    assert_source_and_geometry(&p, line, expected_advance);
}

#[test]
fn negative_period_ltr_pen_zero_uses_exactly_one_threshold_bump() {
    literal_case(ParagraphBaseDirection::LeftToRight, 0.0, 0.0, 8.0);
}
#[test]
fn negative_period_ltr_pen_one_keeps_its_real_phase() {
    literal_case(ParagraphBaseDirection::LeftToRight, 0.0, 1.0, 8.0);
}
#[test]
fn negative_period_rtl_pen_zero_progresses_toward_negative_x() {
    literal_case(ParagraphBaseDirection::RightToLeft, 0.0, 0.0, -8.0);
}
#[test]
fn negative_period_rtl_positive_physical_pen_is_not_a_negated_phase() {
    literal_case(ParagraphBaseDirection::RightToLeft, 0.0, 1.0, -4.0);
}
#[test]
fn negative_period_displaced_ltr_block_origin_is_consumed() {
    literal_case(ParagraphBaseDirection::LeftToRight, 3.0, 0.0, 7.0);
}
#[test]
fn negative_period_displaced_rtl_block_origin_is_consumed() {
    literal_case(ParagraphBaseDirection::RightToLeft, 3.0, 0.0, -5.0);
}

#[test]
fn negative_period_threshold_equality_and_larger_gap_do_not_bump() {
    qualify_font_and_signed_space();
    for (spacing, period, destination) in [(-22.5, -10.0, 10.0), (-23.0, -12.0, 12.0)] {
        let p = css_tab(spacing, ParagraphBaseDirection::LeftToRight).unwrap();
        assert_metric(&p, period, 10.0);
        assert_eq!(
            finish(&p, Some(0.0), 0.0).unwrap().feedback().advance,
            destination
        );
    }
}

#[test]
fn positive_and_spacing_derived_zero_controls_keep_distinct_resource_demands() {
    qualify_font_and_signed_space();
    let positive = css_tab(-19.0, ParagraphBaseDirection::LeftToRight).unwrap();
    assert_metric(&positive, 4.0, 10.0);
    assert_source_and_geometry(&positive, finish(&positive, Some(0.0), 0.0).unwrap(), 8.0);
    let zero = css_tab(-20.0, ParagraphBaseDirection::LeftToRight).unwrap();
    assert_metric(&zero, 0.0, 0.0);
    let line = finish(&zero, None, 0.0).unwrap();
    assert_eq!(line.feedback().advance, 0.0);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    assert_eq!(zero.analyzed_source().source().text().code_units(), [9]);
    assert!(
        line.prefix_at(&crate::TextSeam::AfterParticipant(
            zero.participants().next().unwrap()
        ))
        .unwrap()
        .has_retained_flow_participation()
    );
}

#[test]
fn authored_zero_and_negative_specified_syntax_remain_different_from_negative_used_period() {
    assert!(matches!(
        TabSize::spaces(-1.0).unwrap_err().detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "tab size",
            category: crate::TextNumericError::OutOfRange,
            ..
        })
    ));
    assert!(matches!(
        TabSize::length(-1.0).unwrap_err().detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "tab size",
            category: crate::TextNumericError::OutOfRange,
            ..
        })
    ));
    let own = Style {
        tab_size: TabSize::spaces(0.0).unwrap(),
        ..actual_style()
    };
    let p = prepare(
        input("\t", own, None, ParagraphBaseDirection::LeftToRight),
        TabPreparation::CssStops,
    )
    .unwrap();
    let metric = p.inner.tab_metrics.iter().flatten().next().unwrap();
    assert_eq!(metric.period, 0.0);
    assert!(metric.font.is_none() && metric._block_font.is_none());
    assert_eq!(finish(&p, None, 0.0).unwrap().feedback().advance, 0.0);
}

#[test]
fn negative_nonzero_period_needs_real_phase_and_refusal_leaves_retained_output_unchanged() {
    qualify_font_and_signed_space();
    let p = css_tab(-21.0, ParagraphBaseDirection::LeftToRight).unwrap();
    let retained = finish(&p, Some(0.0), 0.0).unwrap();
    assert_eq!(retained.feedback().advance, 8.0);
    assert!(matches!(
        reason(&finish(&p, None, 0.0).unwrap_err()),
        ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TabOrigin)
    ));
    let error = finish(&p, Some(f64::NAN), 0.0).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "tab origin",
            phase: crate::TextNumericPhase::Selection,
            category: crate::TextNumericError::NonFinite,
            ..
        })
    ));
    assert_eq!(retained.feedback().advance, 8.0);
    assert_eq!(finish(&p, Some(0.0), 0.0).unwrap().feedback().advance, 8.0);
    assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
}

#[test]
fn negative_grid_cannot_round_away_a_real_origin_in_the_single_threshold_bump() {
    qualify_font_and_signed_space();
    let p = css_tab(-21.0, ParagraphBaseDirection::LeftToRight).unwrap();
    let retained = finish(&p, Some(0.0), 0.0).unwrap();
    // First stop is this real positive origin. One required bump would produce
    // 4+2^-1022, which is not representable in binary64. Consumed phase cannot
    // silently become4, zero, or an ordinary fit failure.
    let error = finish(&p, Some(f64::MIN_POSITIVE), 0.0).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            phase: crate::TextNumericPhase::Finalization,
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_eq!(retained.feedback().advance, 8.0);
    assert_eq!(finish(&p, Some(0.0), 0.0).unwrap().feedback().advance, 8.0);
}
