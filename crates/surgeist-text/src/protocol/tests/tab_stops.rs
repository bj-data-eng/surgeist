//! CSS tab positioning uses actual stops, with source text preserved.
use super::*;

#[test]
fn preserved_tab_positions_the_next_glyph_at_its_block_stop() {
    let mut system = system();
    let prepared = prepared(&mut system, "A\tB", WhiteSpace::Preserve);
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        [65, 9, 66]
    );

    // The independent FontAdvance policy remains a real retained-font control:
    // Ahem at 20px emits its 20px .notdef glyph for U+0009 under that policy.
    let mut control = request(&prepared);
    control.context.tabs = TabContext::FontAdvance;
    let control = prepared
        .select(control)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(control.feedback().advance, 60.0);
    let glyphs = |line: &FinalizedLine| {
        line.run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance(), glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        glyphs(&control),
        [(35, 20.0, 0.0), (0, 20.0, 20.0), (36, 20.0, 40.0)]
    );

    // Four actual 20px spaces give an 80px tab interval. From A's end at
    // 20px, the next stop is 80px: a 60px shift, above the 10px half-ch floor.
    // CSS renders the tab as that shift, without its FontAdvance glyph.
    let prepared = css(
        &mut system,
        "A\tB",
        crate::TabSize::spaces(4.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let mut selection = request(&prepared);
    selection.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    let line = prepared
        .select(selection)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 100.0);
    assert_eq!(glyphs(&line), [(35, 20.0, 0.0), (36, 20.0, 80.0)]);
    assert_eq!(
        line.prepared_text()
            .analyzed_source()
            .source()
            .text()
            .code_units(),
        [65, 9, 66]
    );
}

fn css(
    system: &mut System,
    text: &str,
    size: crate::TabSize,
    direction: ParagraphBaseDirection,
) -> PreparedText {
    let source = Source::new(text);
    let style = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: size,
        ..style()
    };
    let input = ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        source: source.clone(),
        base_style: style.clone(),
        tab_reference: Some(crate::ParagraphTabReference::new(style, direction).unwrap()),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            source,
            local: crate::Range::new(0, text.encode_utf16().count()),
            aggregate: crate::Range::new(0, text.encode_utf16().count()),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops).unwrap(),
        )
        .unwrap()
}

fn finish(p: &PreparedText, origin: Option<f64>, indent: f64) -> crate::Result<FinalizedLine> {
    let mut r = request(p);
    r.context.tabs = TabContext::CssStops { origin };
    r.context.base_direction = match p.analyzed_source().direction() {
        crate::Direction::RightToLeft => ParagraphBaseDirection::RightToLeft,
        _ => ParagraphBaseDirection::LeftToRight,
    };
    r.context.indent = indent;
    p.select(r)?.finalize(SpacingAllocations::None)
}

#[test]
fn css_tab_uses_one_threshold_bump_and_strictly_next_stop() {
    let mut system = system();
    // ZERO is 20px: half-ch is10. Exact10 does not bump;9 does.
    for (period, expected) in [
        (30.0, 50.0),
        (29.0, 78.0),
        (31.0, 51.0),
        (20.0, 60.0),
        (1.0, 42.0),
    ] {
        let p = css(
            &mut system,
            "A\tB",
            crate::TabSize::length(period).unwrap(),
            ParagraphBaseDirection::LeftToRight,
        );
        assert_eq!(
            finish(&p, Some(0.0), 0.0).unwrap().feedback().advance,
            expected
        );
    }
}

#[test]
fn signed_grid_extrapolates_in_paragraph_progression() {
    let mut system = system();
    for (direction, origin) in [
        (ParagraphBaseDirection::LeftToRight, 100.0),
        (ParagraphBaseDirection::RightToLeft, -100.0),
    ] {
        let p = css(
            &mut system,
            "\t\tA",
            crate::TabSize::length(40.0).unwrap(),
            direction,
        );
        let line = finish(&p, Some(origin), 0.0).unwrap();
        assert_eq!(line.feedback().advance, 80.0);
        let prefix = line
            .prefix_at(&crate::TextSeam::AfterParticipant(
                p.participants().next().unwrap(),
            ))
            .unwrap();
        assert_eq!(prefix.advance(), 20.0);
        assert!(prefix.has_retained_flow_participation());
    }
}

#[test]
fn indent_changes_pen_while_alignment_does_not_restart_grid() {
    let mut system = system();
    let p = css(
        &mut system,
        "A\tB",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let mut r = request(&p);
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    r.context.indent = 20.0;
    r.context.available_inline = Some(200.0);
    r.context.alignment = crate::Alignment::Right;
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 80.0);
    let glyphs: Vec<_> = line
        .run_views()
        .flat_map(|run| run.glyphs().map(|g| (g.id(), g.pen())).collect::<Vec<_>>())
        .collect();
    assert_eq!(glyphs, [(35, 0.0), (36, 60.0)]);
}

#[test]
fn spacing_before_tab_is_absorbed_and_recomputed_from_selected_rows() {
    let mut system = system();
    let p = css(
        &mut system,
        "A \tB",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let mut r = request(&p);
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    r.context.spacing = SpacingPolicy::Spaces;
    let selected = p.select(r).unwrap();
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(slots.len(), 1);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 100.0);
    for (amount, expected) in [(20.0, 100.0), (40.0, 180.0)] {
        let line = selected
            .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: slots[0].clone(),
                amount,
            }]))
            .unwrap();
        assert_eq!(line.feedback().advance, expected);
    }
    assert_eq!(natural.feedback().advance, 100.0);
}

#[test]
fn unknown_phase_is_consumed_only_by_positive_retained_tabs() {
    let mut system = system();
    let zero = css(
        &mut system,
        "A\tB",
        crate::TabSize::spaces(0.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(finish(&zero, None, 0.0).unwrap().feedback().advance, 40.0);
    let positive = css(
        &mut system,
        "A\tB",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    assert!(matches!(
        reason(&finish(&positive, None, 0.0).unwrap_err()),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::IntrinsicTabPhase
        }
    ));
    assert_eq!(
        finish(&positive, Some(0.0), 0.0)
            .unwrap()
            .feedback()
            .advance,
        100.0
    );
}

#[test]
fn prepared_tab_demand_is_distinct_from_capture_and_selected_policy() {
    let mut system = system();
    let p = css(
        &mut system,
        "A\tB",
        crate::TabSize::default(),
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(
        p.preparation_options().tabs(),
        crate::TabPreparation::CssStops
    );
    assert_eq!(
        p.tab_reference().unwrap().direction(),
        ParagraphBaseDirection::LeftToRight
    );
    let mut r = request(&p);
    r.context.tabs = TabContext::FontAdvance;
    assert!(matches!(
        reason(&p.select(r).unwrap_err()),
        ProtocolError::MismatchedContext {
            field: ProtocolContextField::TabPreparation
        }
    ));
    assert!(crate::TabSize::spaces(-1.0).is_err());
    assert!(crate::TabSize::length(f64::INFINITY).is_err());
}

fn captured(
    text: &str,
    style: Style,
    block: Option<Style>,
    direction: ParagraphBaseDirection,
) -> ParagraphInput {
    let source = Source::new(text);
    let end = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        source: source.clone(),
        base_style: style,
        tab_reference: block
            .map(|style| crate::ParagraphTabReference::new(style, direction).unwrap()),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            source,
            local: crate::Range::new(0, end),
            aggregate: crate::Range::new(0, end),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn css_capture(
    system: &mut System,
    input: ParagraphInput,
    scale: f32,
) -> crate::Result<PreparedText> {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input)?,
        crate::Wrap::Word,
    );
    system.prepare(
        &analyzed,
        PreparationOptions::try_new(scale, crate::TabPreparation::CssStops)?,
    )
}
#[test]
fn block_space_and_tab_zero_have_distinct_owners_and_scale_once() {
    let mut system = system();
    let body = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::spaces(4.0).unwrap(),
        ..style()
    };
    let block = Style {
        size: 10.0,
        letter_spacing: 2.0,
        word_spacing: 3.0,
        ..style()
    };
    // (10+2+3)*4=60, independently of body font20; scale2 =>period120.
    for (scale, expected) in [(1.0, 80.0), (2.0, 160.0)] {
        let p = css_capture(
            &mut system,
            captured(
                "A\tB",
                body.clone(),
                Some(block.clone()),
                ParagraphBaseDirection::LeftToRight,
            ),
            scale,
        )
        .unwrap();
        assert_eq!(
            finish(&p, Some(0.0), 0.0).unwrap().feedback().advance,
            expected
        );
    }
    let mut input = captured(
        "A\tB",
        Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::length(30.0).unwrap(),
            ..style()
        },
        Some(style()),
        ParagraphBaseDirection::LeftToRight,
    );
    // Only TAB's font is40: half-ch20 forces the second stop60 after A20.
    input.source.span(
        crate::Range::new(1, 2),
        Style {
            size: 40.0,
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::length(30.0).unwrap(),
            ..style()
        },
    );
    let p = css_capture(&mut system, input, 1.0).unwrap();
    assert_eq!(finish(&p, Some(0.0), 0.0).unwrap().feedback().advance, 80.0);
    let length = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::length(30.0).unwrap(),
        ..style()
    };
    let p = css_capture(
        &mut system,
        captured(
            "A\tB",
            length,
            Some(style()),
            ParagraphBaseDirection::LeftToRight,
        ),
        2.0,
    )
    .unwrap();
    assert_eq!(
        finish(&p, Some(0.0), 0.0).unwrap().feedback().advance,
        100.0
    );
}
#[test]
fn zero_tab_is_fontless_but_still_needs_actual_line_metrics() {
    let mut empty = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let zero = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::spaces(0.0).unwrap(),
        ..style()
    };
    let mut input = captured("\t", zero, None, ParagraphBaseDirection::LeftToRight);
    input.paragraph_strut = ParagraphStrut::AllocatedContributions;
    let p = css_capture(&mut empty, input, 1.0).unwrap();
    let mut r = request(&p);
    r.context.tabs = TabContext::CssStops { origin: None };
    assert!(matches!(
        reason(
            &p.select(r.clone())
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap_err()
        ),
        ProtocolError::MissingLineMetrics
    ));
    r.line_strut = Some(crate::LineStrut {
        baseline: 7.0,
        line_extent: 10.0,
    });
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 0.0);
    assert_eq!(line.feedback().baseline, 7.0);
    assert_eq!(line.run_views().count(), 0);
    assert!(
        line.prefix_at(&crate::TextSeam::AfterParticipant(
            p.participants().next().unwrap()
        ))
        .unwrap()
        .has_retained_flow_participation()
    );
}
#[test]
fn spacing_derived_zero_skips_threshold_and_negative_interval_is_precise() {
    let mut system = system();
    let body = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::spaces(4.0).unwrap(),
        ..style()
    };
    let block = Style {
        word_spacing: -20.0,
        ..style()
    };
    let p = css_capture(
        &mut system,
        captured(
            "A\tB",
            body.clone(),
            Some(block),
            ParagraphBaseDirection::LeftToRight,
        ),
        1.0,
    )
    .unwrap();
    assert_eq!(finish(&p, None, 0.0).unwrap().feedback().advance, 40.0);
    let block = Style {
        word_spacing: -21.0,
        ..style()
    };
    assert!(matches!(
        reason(
            &css_capture(
                &mut system,
                captured(
                    "A\tB",
                    body,
                    Some(block),
                    ParagraphBaseDirection::LeftToRight
                ),
                1.0
            )
            .unwrap_err()
        ),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::NegativeTabInterval
        }
    ));
    assert_eq!(finish(&p, None, 0.0).unwrap().feedback().advance, 40.0);
}
#[test]
fn selected_replacement_tabs_are_ready_and_keep_definition_custody() {
    let mut system = system();
    let owning = Style {
        white_space: WhiteSpace::Preserve,
        hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("\tB")),
        tab_size: crate::TabSize::length(80.0).unwrap(),
        ..style()
    };
    let p = css_capture(
        &mut system,
        captured(
            "A\u{ad}C",
            owning.clone(),
            Some(owning),
            ParagraphBaseDirection::LeftToRight,
        ),
        1.0,
    )
    .unwrap();
    let offer = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    let mut r = request(&p);
    r.end = offer.boundary();
    r.termination = crate::PreparedTermination::Ordinary(offer.handle());
    r.alternative = offer.alternatives().next().map(|a| a.handle());
    r.context.end = LineEnd::Soft;
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    r.participants = p
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 100.0);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [35, 36]
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 173, 67]
    );
    let mut missing = captured(
        "A\u{ad}C",
        Style {
            white_space: WhiteSpace::Preserve,
            hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("\t")),
            ..style()
        },
        None,
        ParagraphBaseDirection::LeftToRight,
    );
    missing.paragraph_strut = ParagraphStrut::AllocatedContributions;
    assert!(matches!(
        reason(&css_capture(&mut system, missing, 1.0).unwrap_err()),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::TabReference
        }
    ));
}
#[test]
fn css_tabs_survive_system_drop_and_numeric_failure_preserves_old_line() {
    let p = {
        let mut system = system();
        css(
            &mut system,
            "A\tB",
            crate::TabSize::length(80.0).unwrap(),
            ParagraphBaseDirection::LeftToRight,
        )
    };
    let good = finish(&p, Some(0.0), 0.0).unwrap();
    assert_eq!(good.feedback().advance, 100.0);
    assert!(finish(&p, Some(f64::NAN), 0.0).is_err());
    assert!(finish(&p, Some(-f64::MAX), f64::MAX).is_err());
    assert_eq!(good.feedback().advance, 100.0);
}

#[test]
fn source_less_zero_tab_replacement_retains_alternative_bounds() {
    let mut system = system();
    let mut input = captured("A", style(), None, ParagraphBaseDirection::LeftToRight);
    input.controls.push(paragraph::ParagraphControl {
        at: 1,
        kind: paragraph::ParagraphControlKind::DiscretionaryBreak {
            style: Box::new(Style {
                font: Font::new().family("Unavailable"),
                hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("\t")),
                tab_size: crate::TabSize::spaces(0.0).unwrap(),
                ..style()
            }),
        },
    });
    input.anchor_order.push(paragraph::ParagraphAnchor {
        item: paragraph::ParagraphAnchorItem::Control(0),
        affinity: crate::Affinity::After,
    });
    let p = css_capture(&mut system, input, 1.0).unwrap();
    let offer = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    let alternative = offer.alternatives().next().unwrap().handle();
    let mut r = request(&p);
    r.end = offer.boundary();
    r.participants = p
        .participants()
        .take(r.end.view().participant_offset())
        .collect();
    r.termination = crate::PreparedTermination::Ordinary(offer.handle());
    r.alternative = Some(alternative.clone());
    r.context.end = LineEnd::Soft;
    r.context.tabs = TabContext::CssStops { origin: None };
    r.controls = p
        .control_views()
        .map(|c| ControlAllocation {
            occurrence: c.handle(),
            strut: None,
        })
        .collect();
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let fragment=line.fragment_views().find(|f|matches!(f.owner(),crate::FinalizedFragmentOwner::Alternative(ref a) if *a==alternative)).unwrap();
    assert!(fragment.origin().is_none());
    assert!(fragment.aggregate_range().is_none());
    let bounds = fragment
        .bounds()
        .expect("actual fontless alternative has its zero extent");
    assert_eq!(
        bounds,
        crate::Rect::new(20.0, line.feedback().baseline, 0.0, 0.0)
    );
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(
        line.run_views().map(|r| r.glyphs().count()).sum::<usize>(),
        1
    );
    assert_eq!(p.analyzed_source().source().text().code_units(), [65]);
}
#[test]
fn tab_range_and_cursor_use_the_same_resolved_allocation() {
    let mut system = system();
    let p = css(
        &mut system,
        "A\tB",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let line = finish(&p, Some(0.0), 0.0).unwrap();
    let origin = p.origin_views().next().unwrap().handle();
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 7.0,
                baseline: 30.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    let fragments = layout
        .source_range_fragments(&origin, crate::Range::new(1, 2))
        .unwrap();
    let fragments: Vec<_> = fragments.units().collect();
    assert_eq!(fragments.len(), 1);
    let pieces: Vec<_> = fragments[0].pieces().collect();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].rect().origin.x, 27.0);
    assert_eq!(pieces[0].rect().size.width, 60.0);
    assert!(
        layout
            .cursor(crate::Cursor::new(1, crate::Affinity::After))
            .is_ok()
    );
    assert!(
        layout
            .cursor(crate::Cursor::new(2, crate::Affinity::Before))
            .is_ok()
    );
}
#[test]
fn collapsed_away_tabs_and_absence_do_not_consume_block_capture() {
    let mut system = system();
    for text in ["AB", "A\tB"] {
        let p = css_capture(
            &mut system,
            captured(
                text,
                Style {
                    white_space: WhiteSpace::Collapse,
                    ..style()
                },
                None,
                ParagraphBaseDirection::LeftToRight,
            ),
            1.0,
        )
        .unwrap();
        assert_eq!(
            finish(&p, None, 0.0).unwrap().feedback().advance,
            if text == "AB" { 40.0 } else { 60.0 }
        );
    }
}

#[test]
fn finite_huge_grid_origin_preserves_its_exact_phase() {
    let mut system = system();
    let p = css(
        &mut system,
        "A\tB",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let good = finish(&p, Some(0.0), 0.0).unwrap();
    let origin = 1e308_f64;
    assert_eq!(origin.to_bits(), 0x7fe1_ccf3_85eb_c8a0);
    // This exact binary64 integer is5010420900022432 *2^971. After
    // dividing by16, its remainder mod5 is2*3=1: origin mod80 is16.
    // From A's pen20, the next stop96 gives TAB76 and total116.
    let large = finish(&p, Some(origin), 0.0).unwrap();
    assert_eq!(large.feedback().advance, 116.0);
    assert_eq!(
        large
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [(35, 0.0), (36, 96.0)]
    );
    assert_eq!(good.feedback().advance, 100.0);
}

fn check_consumed_grid_origin(direction: ParagraphBaseDirection, origin: f64) {
    let mut system = system();
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(64.0).unwrap(),
        direction,
    );
    // The mirrored grids have a real 48px shift from physical pen +/-32
    // to the next stop +/-80; half-ch10 does not require another stop.
    assert_eq!(
        finish(&p, Some(origin), 32.0).unwrap().feedback().advance,
        48.0
    );

    let period = 2.0_f64.powi(60);
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(period).unwrap(),
        direction,
    );
    let control = finish(&p, Some(0.0), period).unwrap();
    assert_eq!(control.feedback().advance, period);
    // Exact distance is16, above half-ch10, but f64 subtraction absorbs
    // the nonzero origin in the huge pen. A whole-period guess is invalid.
    let error = finish(&p, Some(origin), period)
        .expect_err("consumed nonzero grid origin must not vanish in displacement");
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            phase: crate::TextNumericPhase::Finalization,
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_eq!(control.feedback().advance, period);
    assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
}

#[test]
fn huge_positive_pen_cannot_absorb_consumed_grid_origin() {
    check_consumed_grid_origin(ParagraphBaseDirection::LeftToRight, 16.0);
}

#[test]
fn huge_negative_pen_cannot_absorb_consumed_grid_origin() {
    check_consumed_grid_origin(ParagraphBaseDirection::RightToLeft, -16.0);
}

fn check_grid_period_translation(direction: ParagraphBaseDirection, sign: f64) {
    let mut system = system();
    let period = 2.0_f64.powi(60);
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(period).unwrap(),
        direction,
    );
    let control = finish(&p, Some(sign * 256.0), 240.0).unwrap();
    assert_eq!(control.feedback().advance, 16.0);
    // Translating the origin by one exact period leaves the grid unchanged.
    // Subtracting240 from period+256 first rounds away the remaining16,
    // although the actual next stop256 and shift16 are representable.
    let translated = finish(&p, Some(sign * (period + 256.0)), 240.0).unwrap();
    assert_eq!(translated.feedback().advance, 16.0);
    assert_eq!(control.feedback().advance, 16.0);
    assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
}

#[test]
fn grid_period_translation_preserves_small_distance_ltr() {
    check_grid_period_translation(ParagraphBaseDirection::LeftToRight, 1.0);
}

#[test]
fn grid_period_translation_preserves_small_distance_rtl() {
    check_grid_period_translation(ParagraphBaseDirection::RightToLeft, -1.0);
}

fn check_opposite_signed_remainders(direction: ParagraphBaseDirection, sign: f64) {
    let mut system = system();
    let period = 2.0_f64.powi(60);
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(period).unwrap(),
        direction,
    );
    // Origin +(T-128), pen -144: next stop -128. Origin -(T-128),
    // pen112: next stop128. Both shifts16 remain representable even
    // when the opposite cyclic distance T-16 does not. Mirror in RTL.
    for (origin, indent) in [(period - 128.0, -144.0), (-(period - 128.0), 112.0)] {
        assert_eq!(
            finish(&p, Some(sign * origin), indent)
                .unwrap()
                .feedback()
                .advance,
            16.0
        );
    }
}

#[test]
fn opposite_signed_grid_remainders_keep_small_distance_ltr() {
    check_opposite_signed_remainders(ParagraphBaseDirection::LeftToRight, 1.0);
}

#[test]
fn opposite_signed_grid_remainders_keep_small_distance_rtl() {
    check_opposite_signed_remainders(ParagraphBaseDirection::RightToLeft, -1.0);
}

#[test]
fn threshold_bump_cannot_absorb_its_nonzero_distance() {
    let mut system = system();
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(2.0_f64.powi(60)).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    // Distance8 is below half-ch10, so exactly one period must be added.
    // The resulting period+8 is unrepresentable; dropping8 is not a bump.
    let error = finish(&p, Some(8.0), 0.0).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "tab threshold bump",
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
}

#[test]
fn last_tab_geometry_does_not_consume_an_unused_temporary_destination() {
    let mut system = system();
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(2.0_f64.powi(60)).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    // Pen16 is on the grid: a whole period ahead is required. Real geometry
    // retains its start16 and width separately; no later tab needs the
    // unrepresentable temporary sum period+16.
    let line = finish(&p, Some(16.0), 16.0).unwrap();
    let period = 2.0_f64.powi(60);
    assert_eq!(line.feedback().advance, period);
    let bounds = line.fragment_views().next().unwrap().bounds().unwrap();
    assert_eq!((bounds.origin.x, bounds.size.width), (16.0, period));
    assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
    assert_eq!(
        line.run_views().map(|r| r.glyphs().count()).sum::<usize>(),
        0
    );
}

#[test]
fn fractional_and_signed_remainder_phases_keep_their_actual_distances() {
    let mut system = system();
    for (direction, sign) in [
        (ParagraphBaseDirection::LeftToRight, 1.0),
        (ParagraphBaseDirection::RightToLeft, -1.0),
    ] {
        let p = css(
            &mut system,
            "\t",
            crate::TabSize::length(80.0).unwrap(),
            direction,
        );
        for (origin, indent, expected) in [
            (60.0, -30.0, 10.0),
            (-60.0, 30.0, 70.0),
            (20.0, -10.0, 30.0),
            (-20.0, 10.0, 50.0),
            (15.0, 30.0, 65.0),
            (15.0, 60.0, 35.0),
            (-30.0, -15.0, 65.0),
            (-60.0, -15.0, 35.0),
            (0.3, 0.1, 80.2),
        ] {
            let line = finish(&p, Some(sign * origin), indent).unwrap();
            assert!((line.feedback().advance - expected).abs() < 1e-12);
        }
    }
}

#[test]
fn grid_phase_does_not_require_an_overflowing_coordinate_difference() {
    let mut system = system();
    let period = 2.0_f64.powi(1023) * 1.5;
    for (direction, sign) in [
        (ParagraphBaseDirection::LeftToRight, 1.0),
        (ParagraphBaseDirection::RightToLeft, -1.0),
    ] {
        let p = css(
            &mut system,
            "\t",
            crate::TabSize::length(period).unwrap(),
            direction,
        );
        for side in [-1.0, 1.0] {
            // These are exact binary fractions of the period. The unreduced
            // difference is +/-1.5T and overflows, but both next-stop shifts
            // are T/2 and their actual destination pens remain finite.
            let line =
                finish(&p, Some(sign * side * period * 0.75), -side * period * 0.75).unwrap();
            assert_eq!(line.feedback().advance, period * 0.5);
        }
    }
}

#[test]
fn absent_zero_metric_uses_quarter_em_half_ch_threshold() {
    let fixture =
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(32, 3), (65, 35), (66, 36)]);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture).unwrap();
    let p = css(
        &mut system,
        "A\tB",
        crate::TabSize::length(26.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    // Missing ZERO =>ch10, half-ch5. Distance6 survives, rather than bumping.
    assert_eq!(finish(&p, Some(0.0), 0.0).unwrap().feedback().advance, 46.0);
}
#[test]
fn actual_atomic_advance_participates_in_the_same_tab_traversal() {
    let mut system = system();
    for kind in [
        crate::InlineBoxKind::InFlow,
        crate::InlineBoxKind::OutOfFlow,
    ] {
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::length(80.0).unwrap(),
            ..style()
        };
        let mut input = captured(
            "A\tB",
            own.clone(),
            Some(own),
            ParagraphBaseDirection::LeftToRight,
        );
        input.source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(70),
            kind,
            1,
            crate::Size::new(15.0, 10.0),
        ));
        input.anchor_order.push(paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Atomic(0),
            affinity: crate::Affinity::Before,
        });
        if kind == crate::InlineBoxKind::InFlow {
            for definition in [
                paragraph::ParagraphBoundaryDefinition::AtomicBefore(0),
                paragraph::ParagraphBoundaryDefinition::AtomicAfter(0),
            ] {
                input
                    .boundary_policies
                    .push(paragraph::ParagraphBoundaryPolicy {
                        definition,
                        preceding_wrap: crate::Wrap::Word,
                        following_wrap: crate::Wrap::Word,
                        ancestor_wrap: crate::Wrap::Word,
                    });
            }
        }
        let p = css_capture(&mut system, input, 1.0).unwrap();
        let mut r = request(&p);
        r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
        r.atomics = p
            .atomic_views()
            .map(|a| AtomicAllocation {
                occurrence: a.handle(),
                allocation: if kind == crate::InlineBoxKind::OutOfFlow {
                    AtomicLineAllocation::OutOfFlow
                } else {
                    AtomicLineAllocation::InFlow {
                        flow: AtomicFlowAllocation {
                            advance: 15.0,
                            before_baseline: 8.0,
                            after_baseline: 2.0,
                            occupied_start: 0.0,
                            occupied_end: 15.0,
                        },
                        box_geometry: AtomicBoxAllocation {
                            border: crate::Rect::new(0.0, -8.0, 15.0, 10.0),
                            content: crate::Rect::new(0.0, -8.0, 15.0, 10.0),
                            baseline: 0.0,
                        },
                        alignment: LineBoxAlignment::Baseline,
                    }
                },
            })
            .collect();
        let line = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(line.feedback().advance, 100.0);
        let tab=p.participants().find(|p|matches!(p,PreparedParticipant::Text(u) if u.view().source_range()==crate::Range::new(1,2))).unwrap();
        assert_eq!(
            line.prefix_at(&crate::TextSeam::BeforeParticipant(tab))
                .unwrap()
                .advance(),
            if kind == crate::InlineBoxKind::InFlow {
                35.0
            } else {
                20.0
            }
        );
    }
}

fn huge_atomic_after_first_unit(
    system: &mut System,
    text: &str,
    demand: crate::TabPreparation,
) -> PreparedText {
    let own = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::length(20.0).unwrap(),
        ..style()
    };
    let mut input = captured(
        text,
        own.clone(),
        Some(own),
        ParagraphBaseDirection::LeftToRight,
    );
    input.source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(71),
        crate::InlineBoxKind::InFlow,
        1,
        crate::Size::new(2.0_f64.powi(60), 10.0),
    ));
    input.anchor_order.push(paragraph::ParagraphAnchor {
        item: paragraph::ParagraphAnchorItem::Atomic(0),
        affinity: crate::Affinity::Before,
    });
    for definition in [
        paragraph::ParagraphBoundaryDefinition::AtomicBefore(0),
        paragraph::ParagraphBoundaryDefinition::AtomicAfter(0),
    ] {
        input
            .boundary_policies
            .push(paragraph::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            });
    }
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(input).unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::try_new(1.0, demand).unwrap(),
        )
        .unwrap()
}

fn finish_with_huge_atomic(p: &PreparedText, tabs: TabContext) -> crate::Result<FinalizedLine> {
    let mut r = request(p);
    r.context.tabs = tabs;
    let advance = 2.0_f64.powi(60);
    r.atomics = p
        .atomic_views()
        .map(|a| AtomicAllocation {
            occurrence: a.handle(),
            allocation: AtomicLineAllocation::InFlow {
                flow: AtomicFlowAllocation {
                    advance,
                    before_baseline: 8.0,
                    after_baseline: 2.0,
                    occupied_start: 0.0,
                    occupied_end: advance,
                },
                box_geometry: AtomicBoxAllocation {
                    border: crate::Rect::new(0.0, -8.0, advance, 10.0),
                    content: crate::Rect::new(0.0, -8.0, advance, 10.0),
                    baseline: 0.0,
                },
                alignment: LineBoxAlignment::Baseline,
            },
        })
        .collect();
    p.select(r)?.finalize(SpacingAllocations::None)
}

#[test]
fn trailing_atomic_does_not_consume_the_completed_tab_frame() {
    let mut system = system();
    for (text, demand, tabs) in [
        ("A", crate::TabPreparation::FontAdvance, TabContext::NoTabs),
        (
            "\t",
            crate::TabPreparation::FontAdvance,
            TabContext::FontAdvance,
        ),
        (
            "\t",
            crate::TabPreparation::CssStops,
            TabContext::CssStops { origin: Some(0.0) },
        ),
    ] {
        let p = huge_atomic_after_first_unit(&mut system, text, demand);
        let line = finish_with_huge_atomic(&p, tabs).unwrap();
        assert_eq!(line.feedback().advance, 2.0_f64.powi(60));
        let atomic = p
            .participants()
            .find(|part| matches!(part, PreparedParticipant::Atomic(_)))
            .unwrap();
        assert_eq!(
            line.prefix_at(&crate::TextSeam::BeforeParticipant(atomic))
                .unwrap()
                .advance(),
            20.0
        );
        assert_eq!(
            p.analyzed_source().source().text().code_units(),
            if text == "A" { [65] } else { [9] }
        );
    }
}

#[test]
fn a_later_positive_tab_still_consumes_the_atomic_pen() {
    let mut system = system();
    let p = huge_atomic_after_first_unit(&mut system, "\t\t", crate::TabPreparation::CssStops);
    let error =
        finish_with_huge_atomic(&p, TabContext::CssStops { origin: Some(0.0) }).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "tab traversal pen",
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
}

#[test]
fn rtl_last_tab_needs_no_unrepresentable_temporary_destination() {
    let mut system = system();
    let huge = 2.0_f64.powi(60);
    let finish = |p: &PreparedText, tabs| {
        let mut r = request(p);
        r.context.base_direction = ParagraphBaseDirection::RightToLeft;
        r.context.indent = huge;
        r.context.available_inline = Some(huge);
        r.context.tabs = tabs;
        p.select(r).unwrap().finalize(SpacingAllocations::None)
    };
    let control = css(
        &mut system,
        "A",
        crate::TabSize::length(huge).unwrap(),
        ParagraphBaseDirection::RightToLeft,
    );
    let ordinary = finish(&control, TabContext::NoTabs).unwrap();
    assert_eq!(ordinary.feedback().advance, 20.0);
    let ordinary_bounds = ordinary.fragment_views().next().unwrap().bounds().unwrap();
    assert_eq!(
        (ordinary_bounds.origin.x, ordinary_bounds.size.width),
        (-20.0, 20.0)
    );
    let p = css(
        &mut system,
        "\t",
        crate::TabSize::length(huge).unwrap(),
        ParagraphBaseDirection::RightToLeft,
    );
    let line = finish(
        &p,
        TabContext::CssStops {
            origin: Some(-16.0),
        },
    )
    .unwrap();
    assert_eq!(line.feedback().advance, 16.0);
    let bounds = line.fragment_views().next().unwrap().bounds().unwrap();
    assert_eq!((bounds.origin.x, bounds.size.width), (-16.0, 16.0));
    assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
    assert_eq!(
        line.run_views().map(|r| r.glyphs().count()).sum::<usize>(),
        0
    );
    assert_eq!(ordinary.feedback().advance, 20.0);
}

#[test]
fn mixed_isolate_order_keeps_tab_at_its_post_l1_l2_slot() {
    let mut system = system();
    let p = css(
        &mut system,
        "A\u{2067}B\u{2069}\tC",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let line = finish(&p, Some(0.0), 0.0).unwrap();
    assert_eq!(line.feedback().advance, 100.0);
    let tab=p.participants().find(|part|matches!(part,PreparedParticipant::Text(unit) if unit.view().source_range()==crate::Range::new(4,5))).unwrap();
    assert_eq!(
        line.prefix_at(&crate::TextSeam::BeforeParticipant(tab.clone()))
            .unwrap()
            .advance(),
        40.0
    );
    assert_eq!(
        line.prefix_at(&crate::TextSeam::AfterParticipant(tab))
            .unwrap()
            .advance(),
        80.0
    );
    // The maintained provider zeroes default-ignorable positions, then maps
    // their glyphs to nominal SPACE. Ahem SPACE is glyph3 with proven empty
    // outline; zero advance alone would not prove that paint fact.
    let glyphs: Vec<_> = line
        .run_views()
        .flat_map(|run| {
            let instance = run.effective_instance().unwrap();
            run.glyphs()
                .map(|glyph| {
                    if glyph.id() == 3 {
                        assert_eq!(
                            instance
                                .glyph_outline(3, run.face().catalogue_identity())
                                .unwrap()
                                .bounds(),
                            None
                        );
                    }
                    (glyph.id(), glyph.advance())
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        glyphs,
        [(35, 20.0), (3, 0.0), (36, 20.0), (3, 0.0), (37, 20.0)]
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 8295, 66, 8297, 9, 67]
    );
}
#[test]
fn zero_tab_layout_keeps_source_and_refuses_fabricated_range_font_metrics() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let mut input = captured(
        "\t",
        Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(0.0).unwrap(),
            ..style()
        },
        None,
        ParagraphBaseDirection::LeftToRight,
    );
    input.paragraph_strut = ParagraphStrut::AllocatedContributions;
    let p = css_capture(&mut system, input, 1.0).unwrap();
    let origin = p.origin_views().next().unwrap().handle();
    let mut r = request(&p);
    r.context.tabs = TabContext::CssStops { origin: None };
    r.line_strut = Some(LineStrut {
        baseline: 7.0,
        line_extent: 10.0,
    });
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 7.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    assert!(matches!(
        layout
            .source_range_fragments(&origin, crate::Range::new(0, 1))
            .unwrap_err()
            .detail(),
        Some(crate::ErrorDetail::SourceRangeFragments {
            reason: crate::SourceRangeFragmentError::MissingFontMetrics { .. }
        })
    ));
    assert_eq!(layout.source().text().code_units(), [9]);
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
    #[cfg(feature = "text-accessibility")]
    {
        let mut state = crate::Accessibility::default();
        let update = layout
            .accessibility_update(
                &mut state,
                accesskit::NodeId(900),
                accesskit::NodeId(900),
                crate::Point::new(0.0, 0.0),
            )
            .unwrap();
        assert!(
            update
                .nodes
                .iter()
                .any(|(_, node)| node.value() == Some("\t"))
        );
    }
}
#[test]
fn tab_size_value_and_dimension_change_style_cache_identity() {
    let source = crate::ValidatedSource::try_from(Source::new("A\tB")).unwrap();
    let system = system();
    let key = |tab_size| {
        crate::Key::from_validated(
            &source,
            &crate::ValidatedStyle::try_from(Style {
                tab_size,
                ..style()
            })
            .unwrap(),
            crate::ValidatedOptions::try_from(crate::Options::default()).unwrap(),
            system.font_catalogue_identity(),
        )
    };
    assert_ne!(
        key(crate::TabSize::spaces(8.0).unwrap()),
        key(crate::TabSize::spaces(4.0).unwrap())
    );
    assert_ne!(
        key(crate::TabSize::spaces(8.0).unwrap()),
        key(crate::TabSize::length(8.0).unwrap())
    );
}

#[test]
fn block_space_metric_uses_actual_used_direction_and_zero_keeps_even_level() {
    let fixture = crate::shaping_test_fonts::directional_space_ahem();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture).unwrap();
    // Literal authored tables: SPACE glyph3=500units, rtla replaces it with
    // glyph0=1000; ordinary LTR ZERO19=1000, but rtla would replace it
    // with SPACE500. UPEM1000 at20px gives10/20/20 for the requested profiles.
    for (direction, advance, glyph) in [
        (ParagraphBaseDirection::LeftToRight, 10.0, 3),
        (ParagraphBaseDirection::RightToLeft, 20.0, 0),
    ] {
        let input = captured(
            " ",
            Style {
                white_space: WhiteSpace::Preserve,
                ..style()
            },
            None,
            direction,
        );
        let analyzed = crate::test_wrapping::bind_missing(
            &AnalyzedSource::from_paragraph(input).unwrap(),
            crate::Wrap::Word,
        );
        let p = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let mut r = request(&p);
        r.context.base_direction = direction;
        let line = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(
            line.run_views()
                .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [glyph]
        );
    }
    for (block_direction, expected) in [
        (ParagraphBaseDirection::LeftToRight, 40.0),
        (ParagraphBaseDirection::RightToLeft, 60.0),
    ] {
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(2.0).unwrap(),
            ..style()
        };
        let mut input = captured(
            "\tB",
            own.clone(),
            None,
            ParagraphBaseDirection::LeftToRight,
        );
        input.tab_reference =
            Some(crate::ParagraphTabReference::new(own, block_direction).unwrap());
        let p = css_capture(&mut system, input, 1.0).unwrap();
        assert_eq!(
            finish(&p, Some(0.0), 0.0).unwrap().feedback().advance,
            expected
        );
    }
}

#[test]
fn zero_en_metric_stays_ltr_even_when_block_and_paragraph_are_rtl() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::directional_space_ahem())
        .unwrap();
    for direction in [
        ParagraphBaseDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft,
    ] {
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(1.3).unwrap(),
            ..style()
        };
        let mut input = captured("A\tB", own.clone(), None, direction);
        input.tab_reference = Some(
            crate::ParagraphTabReference::new(own, ParagraphBaseDirection::RightToLeft).unwrap(),
        );
        let p = css_capture(&mut system, input, 1.0).unwrap();
        // RTL SPACE20 *1.3=26. ZERO stays LTR20 =>half-ch10, so distance6
        // bumps once to32, total20+32+20=72. Wrong RTL ZERO would yield46.
        assert_eq!(finish(&p, Some(0.0), 0.0).unwrap().feedback().advance, 72.0);
    }
}

#[test]
fn absence_of_positive_tabs_does_not_consume_a_temporary_signed_pen() {
    let mut system = system();
    let input = captured("A", style(), None, ParagraphBaseDirection::RightToLeft);
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input).unwrap(),
        crate::Wrap::Word,
    );
    // Genuine ordinary control: huge indent cancels the same available width in
    // the existing later RTL alignment operation. Physical glyph pen is-20..0.
    let ordinary = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let mut request_ordinary = request(&ordinary);
    request_ordinary.context.base_direction = ParagraphBaseDirection::RightToLeft;
    request_ordinary.context.indent = 1e308;
    request_ordinary.context.available_inline = Some(1e308);
    assert_eq!(
        ordinary
            .select(request_ordinary)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
    let p = system
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops).unwrap(),
        )
        .unwrap();
    let mut r = request(&p);
    r.context.base_direction = ParagraphBaseDirection::RightToLeft;
    r.context.indent = 1e308;
    r.context.available_inline = Some(1e308);
    r.context.tabs = TabContext::CssStops { origin: None };
    assert_eq!(
        p.select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        20.0
    );
}

#[test]
fn zero_tabs_do_not_consume_a_temporary_signed_pen() {
    let mut system = system();
    let ordinary_input = captured("AB", style(), None, ParagraphBaseDirection::RightToLeft);
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(ordinary_input).unwrap(),
        crate::Wrap::Word,
    );
    let ordinary = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let mut control = request(&ordinary);
    control.context.base_direction = ParagraphBaseDirection::RightToLeft;
    control.context.indent = 1e308;
    control.context.available_inline = Some(1e308);
    assert_eq!(
        ordinary
            .select(control)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        40.0
    );
    let input = captured(
        "A\tB",
        Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(0.0).unwrap(),
            ..style()
        },
        None,
        ParagraphBaseDirection::RightToLeft,
    );
    let p = css_capture(&mut system, input, 1.0).unwrap();
    let mut r = request(&p);
    r.context.base_direction = ParagraphBaseDirection::RightToLeft;
    r.context.indent = 1e308;
    r.context.available_inline = Some(1e308);
    r.context.tabs = TabContext::CssStops { origin: None };
    assert_eq!(
        p.select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        40.0
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 9, 66]
    );
}

#[test]
fn tab_interval_numeric_failures_preserve_retained_results() {
    let mut system = system();
    let p = css(
        &mut system,
        "A\tB",
        crate::TabSize::length(80.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    let old = finish(&p, Some(0.0), 0.0).unwrap();
    for (size, scale, expected) in [
        (
            crate::TabSize::length(f64::MAX).unwrap(),
            2.0,
            crate::TextNumericError::NonFinite,
        ),
        (
            crate::TabSize::spaces(f64::MAX).unwrap(),
            1.0,
            crate::TextNumericError::NonFinite,
        ),
        (
            crate::TabSize::length(f64::MIN_POSITIVE).unwrap(),
            1e-30,
            crate::TextNumericError::PositiveLost,
        ),
    ] {
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: size,
            ..style()
        };
        let error = css_capture(
            &mut system,
            captured(
                "\t",
                own.clone(),
                Some(own),
                ParagraphBaseDirection::LeftToRight,
            ),
            scale,
        )
        .unwrap_err();
        assert!(
            matches!(error.detail(),Some(crate::ErrorDetail::InvalidTextNumeric{phase:crate::TextNumericPhase::Preparation,category,..}) if *category==expected)
        );
    }
    assert_eq!(old.feedback().advance, 100.0);
}
#[test]
fn missing_space_metric_is_distinct_from_missing_capture_and_zero_bypasses_it() {
    let fixture =
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(48, 19), (65, 35), (66, 36)]);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture).unwrap();
    let own = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::spaces(2.0).unwrap(),
        ..style()
    };
    let error = css_capture(
        &mut system,
        captured(
            "\t",
            own.clone(),
            Some(own),
            ParagraphBaseDirection::LeftToRight,
        ),
        1.0,
    )
    .unwrap_err();
    assert!(matches!(
        reason(&error),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::TabMetrics
        }
    ));
    let zero = css(
        &mut system,
        "A\tB",
        crate::TabSize::spaces(0.0).unwrap(),
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(finish(&zero, None, 0.0).unwrap().feedback().advance, 40.0);
}
