//! Signed intervals retain metric authority and use the existing forward grid.
use super::*;

fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32, f64)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| (g.id(), g.advance(), run.origin().x + g.pen()))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn multiple_negative_tabs_recompute_spacing_and_publish_the_same_source_rows() {
    let mut system = system();
    let own = Style {
        white_space: WhiteSpace::Preserve,
        tab_size: crate::TabSize::spaces(4.0).unwrap(),
        ..style()
    };
    let p = css_capture(
        &mut system,
        captured(
            "A \tB\tC",
            own,
            Some(Style {
                word_spacing: -21.0,
                ..style()
            }),
            ParagraphBaseDirection::LeftToRight,
        ),
        1.0,
    )
    .unwrap();
    let metric = p.inner.tab_metrics.iter().flatten().next().unwrap();
    assert_eq!((metric.period, metric.threshold), (-4.0, 10.0));
    let mut r = request(&p);
    r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    r.context.spacing = SpacingPolicy::Spaces;
    let selected = p.select(r).unwrap();
    let slots: Vec<_> = selected.spacing_slots().collect();
    assert_eq!(slots.len(), 1);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 96.0);
    // Real Ahem20: A+SPACE reaches40. Each D4 tab shifts8. +1 SPACE
    // is absorbed by the first tab; -1 moves both following grid phases by4.
    for (amount, total, b, c, first_width) in [
        (0.0, 96.0, 48.0, 76.0, 8.0),
        (1.0, 96.0, 48.0, 76.0, 7.0),
        (-1.0, 92.0, 44.0, 72.0, 5.0),
    ] {
        let line = selected
            .finalize(SpacingAllocations::Complete(vec![SpacingAllocation {
                slot: slots[0].clone(),
                amount,
            }]))
            .unwrap();
        assert_eq!(line.feedback().advance, total);
        assert_eq!(
            glyphs(&line),
            [
                (35, 20.0, 0.0),
                (3, 20.0, 20.0),
                (36, 20.0, b),
                (37, 20.0, c)
            ]
        );
        for (range, advance) in [(crate::Range::new(2, 3), b), (crate::Range::new(4, 5), c)] {
            let tab = p.participants().find(|part| matches!(part, PreparedParticipant::Text(u) if u.view().source_range()==range)).unwrap();
            assert_eq!(
                line.prefix_at(&crate::TextSeam::AfterParticipant(tab))
                    .unwrap()
                    .advance(),
                advance
            );
        }
        let origin = p.origin_views().next().unwrap().handle();
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            lines: vec![LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 30.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }],
        })
        .unwrap();
        let fragments = layout
            .source_range_fragments(&origin, crate::Range::new(2, 3))
            .unwrap();
        let units: Vec<_> = fragments.units().collect();
        assert_eq!(units.len(), 1);
        let pieces: Vec<_> = units[0].pieces().collect();
        assert_eq!(pieces.len(), 1);
        assert_eq!(
            (pieces[0].rect().origin.x, pieces[0].rect().size.width),
            (40.0 + amount, first_width)
        );
        assert_eq!(
            layout
                .cursor(crate::Cursor::new(3, crate::Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            b
        );
        assert_eq!(layout.source().text().code_units(), [65, 32, 9, 66, 9, 67]);
    }
    assert_eq!(natural.feedback().advance, 96.0);
}

#[test]
fn selected_negative_replacement_tabs_keep_each_real_before_after_emission() {
    for placement in [
        crate::HyphenationPlacement::LogicalBefore,
        crate::HyphenationPlacement::LogicalAfter,
        crate::HyphenationPlacement::BothSides,
    ] {
        let mut system = system();
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(4.0).unwrap(),
            hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("\t")),
            locale: Some("en-Latn".into()),
            native_hyphenation_convention: Some(
                crate::NativeHyphenationConvention::language_script(
                    "en-Latn",
                    "Latn",
                    "\t".into(),
                    placement,
                )
                .unwrap(),
            ),
            ..style()
        };
        let p = css_capture(
            &mut system,
            captured(
                "AX\u{ad}CCC",
                own,
                Some(Style {
                    word_spacing: -21.0,
                    ..style()
                }),
                ParagraphBaseDirection::LeftToRight,
            ),
            1.0,
        )
        .unwrap();
        let metric = p.inner.tab_metrics.iter().flatten().next().unwrap();
        assert_eq!((metric.period, metric.threshold), (-4.0, 10.0));
        let offer = p
            .opportunity_views()
            .find(|o| o.kind() == PreparedBreakKind::Discretionary)
            .unwrap();
        let alternative = offer.alternatives().next().unwrap().handle();
        let boundary = offer.boundary();
        let offset = boundary.view().participant_offset();
        let mut r = request(&p);
        r.end = boundary.clone();
        r.participants = p.participants().take(offset).collect();
        r.termination = PreparedTermination::Ordinary(offer.handle());
        r.alternative = Some(alternative.clone());
        r.context.end = LineEnd::Soft;
        r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
        let first = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        let mut r = request(&p);
        r.start = boundary;
        r.participants = p.participants().skip(offset).collect();
        r.incoming = first.hyphenation_continuation();
        r.context.start = LineStart::Continuation;
        r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
        let second = p
            .select(r)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        let before = placement != crate::HyphenationPlacement::LogicalAfter;
        let after = placement != crate::HyphenationPlacement::LogicalBefore;
        assert_eq!(first.feedback().advance, if before { 48.0 } else { 40.0 });
        assert_eq!(second.feedback().advance, if after { 68.0 } else { 60.0 });
        assert_eq!(first.hyphenation_continuation().is_some(), after);
        assert!(second.hyphenation_continuation().is_none());
        assert_eq!(
            glyphs(&first).iter().map(|g| g.0).collect::<Vec<_>>(),
            [35, 58]
        );
        assert_eq!(
            glyphs(&second).iter().map(|g| g.0).collect::<Vec<_>>(),
            [37, 37, 37]
        );
        let origin = p.origin_views().next().unwrap().handle();
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            lines: vec![
                LinePlacement {
                    line: first,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 20.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                },
                LinePlacement {
                    line: second,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 40.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                },
            ],
        })
        .unwrap();
        let fragments = layout
            .source_range_fragments(&origin, crate::Range::new(2, 3))
            .unwrap();
        let units: Vec<_> = fragments.units().collect();
        assert_eq!(units.len(), usize::from(before) + usize::from(after));
        for unit in units {
            assert_eq!(
                unit.alternatives().collect::<Vec<_>>().as_slice(),
                std::slice::from_ref(&alternative)
            );
            let pieces: Vec<_> = unit.pieces().collect();
            assert_eq!(pieces.len(), 1);
            assert_eq!(
                (
                    unit.line_index(),
                    pieces[0].rect().origin.x,
                    pieces[0].rect().size.width
                ),
                if unit.line_index() == 0 {
                    (0, 40.0, 8.0)
                } else {
                    (1, 0.0, 8.0)
                }
            );
        }
        assert_eq!(
            layout.source().text().code_units(),
            [65, 88, 173, 67, 67, 67]
        );
    }
}

#[test]
fn negative_tab_consumes_an_atomic_pen_only_when_a_later_tab_needs_it() {
    let mut system = system();
    let mut prepare = |text: &str| {
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(4.0).unwrap(),
            ..style()
        };
        let mut input = captured(
            text,
            own,
            Some(Style {
                word_spacing: -21.0,
                ..style()
            }),
            ParagraphBaseDirection::LeftToRight,
        );
        input.source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(72),
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
        css_capture(&mut system, input, 1.0).unwrap()
    };
    let sole = prepare("\t");
    let tabs = TabContext::CssStops { origin: Some(0.0) };
    let retained = finish_with_huge_atomic(&sole, tabs.clone()).unwrap();
    assert_eq!(retained.feedback().advance, 2.0_f64.powi(60));
    let atomic = sole
        .participants()
        .find(|p| matches!(p, PreparedParticipant::Atomic(_)))
        .unwrap();
    assert_eq!(
        retained
            .prefix_at(&crate::TextSeam::BeforeParticipant(atomic))
            .unwrap()
            .advance(),
        8.0
    );
    let later = prepare("\t\t");
    let error = finish_with_huge_atomic(&later, tabs.clone()).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextNumeric {
            field: "tab traversal pen",
            category: crate::TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_eq!(retained.feedback().advance, 2.0_f64.powi(60));
    assert_eq!(
        finish_with_huge_atomic(&sole, tabs)
            .unwrap()
            .feedback()
            .advance,
        2.0_f64.powi(60)
    );
    assert_eq!(sole.analyzed_source().source().text().code_units(), [9]);
    assert_eq!(later.analyzed_source().source().text().code_units(), [9, 9]);
}

#[test]
fn negative_period_uses_block_direction_but_progresses_in_paragraph_direction() {
    let fixture = crate::shaping_test_fonts::directional_space_ahem();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture.clone()).unwrap();
    // Existing fixture tables: at20px LTR SPACE3=10, RTL rtla SPACE0=20;
    // isolated ZERO stays LTR20. With spacing-23 and multiplier4, T=-52/-12.
    for (block_direction, paragraph_direction, period, advance) in [
        (
            ParagraphBaseDirection::LeftToRight,
            ParagraphBaseDirection::LeftToRight,
            -52.0,
            55.0,
        ),
        (
            ParagraphBaseDirection::LeftToRight,
            ParagraphBaseDirection::RightToLeft,
            -52.0,
            49.0,
        ),
        (
            ParagraphBaseDirection::RightToLeft,
            ParagraphBaseDirection::LeftToRight,
            -12.0,
            15.0,
        ),
        (
            ParagraphBaseDirection::RightToLeft,
            ParagraphBaseDirection::RightToLeft,
            -12.0,
            21.0,
        ),
    ] {
        let own = Style {
            white_space: WhiteSpace::Preserve,
            tab_size: crate::TabSize::spaces(4.0).unwrap(),
            ..style()
        };
        let mut input = captured("\t", own, None, paragraph_direction);
        input.tab_reference = Some(
            crate::ParagraphTabReference::new(
                Style {
                    word_spacing: -23.0,
                    ..style()
                },
                block_direction,
            )
            .unwrap(),
        );
        let p = css_capture(&mut system, input, 1.0).unwrap();
        let metric = p.inner.tab_metrics.iter().flatten().next().unwrap();
        assert_eq!((metric.period, metric.threshold), (period, 10.0));
        assert_eq!(
            metric._block_font.as_ref().unwrap().font.face().bytes(),
            fixture
        );
        assert_eq!(metric.font.as_ref().unwrap().font.face().bytes(), fixture);
        assert_eq!(metric.font.as_ref().unwrap().font_size, 20.0);
        let line = finish(&p, Some(3.0), 0.0).unwrap();
        assert_eq!(line.feedback().advance, advance);
        // A real retained metric/strut row may be present, but no TAB glyph
        // (including .notdef) is emitted by the CSS grid operation.
        assert!(glyphs(&line).is_empty());
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), fixture);
            assert_eq!(run.effective_instance().unwrap().font_size(), 20.0);
        }
        assert_eq!(p.analyzed_source().source().text().code_units(), [9]);
    }
}
