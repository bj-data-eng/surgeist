//! Genuine positioned custody, phantom completion and provider transparency.
use super::*;
use crate::{Error, ErrorCode, ErrorDetail, TextNumericError, TextNumericPhase};

fn typography() -> AnalysisTypography {
    AnalysisTypography::Upright {
        computed_direction: ParagraphBaseDirection::RightToLeft,
    }
}

fn prepare(system: &mut System, input: ParagraphInput) -> PreparedText {
    system
        .prepare(
            &analyze(input, typography()),
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap()
}

fn physical(occurrence: PreparedAtomic) -> AtomicPlacement {
    AtomicPlacement {
        occurrence,
        border: Rect::new(300.0, 500.0, 60.0, 140.0),
        content: Rect::new(305.0, 507.0, 48.0, 126.0),
        baseline: 321.0,
    }
}

fn placement(
    prepared: &PreparedText,
    line: &FinalizedLine,
    atomics: Vec<AtomicPlacement>,
) -> PlacementRequest {
    PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics,
        })],
    }
}

fn protocol(error: Error, expected: ProtocolError) {
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol { reason: expected })
    );
}

#[test]
fn positioned_placement_rejects_missing_duplicate_foreign_and_nonfinite_actual_geometry() {
    let mut system = system_with(AHEM);
    let prepared = prepare(&mut system, input(true));
    let line = finish(&prepared);
    let actual = physical(prepared.atomic_views().next().unwrap().handle());
    let valid =
        Layout::from_finalized_lines(placement(&prepared, &line, vec![actual.clone()])).unwrap();
    assert_eq!(valid.inline_boxes()[0].rect(), actual.border);
    protocol(
        Layout::from_finalized_lines(placement(&prepared, &line, vec![])).unwrap_err(),
        ProtocolError::MissingAtomic { occurrence: 0 },
    );
    protocol(
        Layout::from_finalized_lines(placement(
            &prepared,
            &line,
            vec![actual.clone(), actual.clone()],
        ))
        .unwrap_err(),
        ProtocolError::DuplicateAtomic { occurrence: 0 },
    );

    // This is a genuinely separately issued preparation of equal authored
    // characters, not a fabricated occurrence index or catalogue token.
    let mut foreign_input = input(true);
    let mut foreign_source = Source::identified("XX", Id::from_u64(1_554_102), 1);
    foreign_source.inline_box(foreign_input.source.boxes()[0]);
    foreign_input.origins[0].source = foreign_source.clone();
    foreign_input.source = foreign_source;
    let foreign = prepare(&mut system, foreign_input);
    assert_ne!(
        foreign.analyzed_source().identity(),
        prepared.analyzed_source().identity()
    );
    let other = foreign.atomic_views().next().unwrap().handle();
    assert_ne!(other, actual.occurrence);
    protocol(
        Layout::from_finalized_lines(placement(&prepared, &line, vec![physical(other)]))
            .unwrap_err(),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::Atomic,
            occurrence: 0,
        },
    );

    for (bad, field) in [
        (
            AtomicPlacement {
                border: Rect::new(f64::NAN, 500.0, 60.0, 140.0),
                ..actual.clone()
            },
            "placed x",
        ),
        (
            AtomicPlacement {
                content: Rect::new(305.0, f64::INFINITY, 48.0, 126.0),
                ..actual.clone()
            },
            "placed y",
        ),
        (
            AtomicPlacement {
                baseline: f64::NEG_INFINITY,
                ..actual.clone()
            },
            "placed atomic baseline",
        ),
    ] {
        let error =
            Layout::from_finalized_lines(placement(&prepared, &line, vec![bad])).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
            field: actual, phase: TextNumericPhase::Placement, category: TextNumericError::NonFinite, ..
        }) if *actual == field),
            "{error:?}"
        );
    }
    // Finite independent geometry is intentionally accepted even after a
    // previous failed attempt: an OutOfFlow box is not matched to a flow slot.
    let moved = AtomicPlacement {
        border: Rect::new(-40.0, 90.0, 6.0, 14.0),
        content: Rect::new(-39.0, 92.0, 4.0, 10.0),
        baseline: -37.0,
        ..actual
    };
    let retry =
        Layout::from_finalized_lines(placement(&prepared, &line, vec![moved.clone()])).unwrap();
    assert_eq!(retry.inline_boxes()[0].rect(), moved.border);
    assert_eq!(line.feedback().advance, 8.0);
    assert_eq!(
        valid.inline_boxes()[0].rect(),
        Rect::new(300.0, 500.0, 60.0, 140.0)
    );
}

#[test]
fn source_free_positioned_occurrence_retains_genuine_nonformatted_terminal_completion() {
    let mut source = Source::identified("", Id::from_u64(1_554_101), 1);
    source.inline_box(InlineBox::new(
        BOX,
        InlineBoxKind::OutOfFlow,
        0,
        PhysicalSize::new(60.0, 140.0),
    ));
    let mut value = input(true);
    value.source = source.clone();
    value.origins.clear();
    let mut system = system_with(AHEM);
    let prepared = prepare(&mut system, value);
    assert_eq!(prepared.origin_views().count(), 0);
    assert_eq!(prepared.atomic_views().count(), 1);
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [PreparedParticipant::Atomic(
            prepared.atomic_views().next().unwrap().handle()
        )]
    );
    let line = finish(&prepared);
    assert!(!line.has_in_flow_content());
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (0.0, 8.0, 10.0)
    );
    assert_eq!(line.fragment_views().count(), 0);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    let selected = line.selected_line();
    assert_eq!(selected.request().context.end, LineEnd::Paragraph);
    let PreparedTermination::Ordinary(offer) = &selected.request().termination else {
        panic!("actual ParagraphEnd occurrence required")
    };
    assert_eq!(offer.view().kind(), PreparedBreakKind::ParagraphEnd);
    let actual = physical(prepared.atomic_views().next().unwrap().handle());
    let completion = line
        .nonformatted()
        .expect("an OOF-only terminal is genuinely nonformatted despite its nominal strut");
    let request = |atomics| PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::NonFormatted(NonFormattedLinePlacement {
            line: completion.clone(),
            origin: Point::new(7.0, 11.0),
            orientation: LineOrientation::Vertical,
            atomics,
        })],
    };
    let layout = Layout::from_finalized_lines(request(vec![actual.clone()])).unwrap();
    assert_eq!(layout.source(), &source);
    assert!(layout.lines().is_empty());
    assert_eq!(layout.metrics().line_count(), 0);
    assert_eq!(layout.metrics().size(), PhysicalSize::new(0.0, 0.0));
    assert_eq!(layout.positions().count(), 0);
    let retained = layout.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].line.finalized_line(), &line);
    assert_eq!(retained[0].origin, Point::new(7.0, 11.0));
    assert_eq!(retained[0].orientation, LineOrientation::Vertical);
    assert_eq!(retained[0].atomics, [actual]);
    protocol(
        Layout::from_finalized_lines(request(vec![])).unwrap_err(),
        ProtocolError::MissingAtomic { occurrence: 0 },
    );
}

fn arabic_input(affinity: Option<Affinity>) -> ParagraphInput {
    let mut source =
        Source::identified("\u{628}\u{64e}\u{200d}\u{628}", Id::from_u64(1_554_200), 1);
    if affinity.is_some() {
        source.inline_box(InlineBox::new(
            BOX,
            InlineBoxKind::OutOfFlow,
            1,
            PhysicalSize::new(60.0, 140.0),
        ));
    }
    let mut value = input(false);
    value.source = source.clone();
    value.base_style.size = 10.0;
    value.base_style.line_height = LineHeight::Absolute(12.0);
    value.origins = vec![ParagraphOrigin {
        text_node: None,
        source,
        local: Range::new(0, 4),
        aggregate: Range::new(0, 4),
        kind: ParagraphOriginKind::Authored,
    }];
    if let Some(affinity) = affinity {
        value.anchor_order = vec![ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(0),
            affinity,
        }];
        value.boundary_policies = [
            ParagraphBoundaryDefinition::AtomicBefore(0),
            ParagraphBoundaryDefinition::AtomicAfter(0),
        ]
        .into_iter()
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    }
    value
}

fn arabic_finish(prepared: &PreparedText) -> FinalizedLine {
    let mut value = request(prepared);
    value.context.orientation = LineOrientation::Vertical;
    value.context.available_inline = Some(24.0);
    value.context.alignment = crate::Alignment::Start;
    value.context.spacing = SpacingPolicy::None;
    value.atomics = prepared
        .atomic_views()
        .map(|view| AtomicAllocation {
            occurrence: view.handle(),
            allocation: AtomicLineAllocation::OutOfFlow,
        })
        .collect();
    prepared
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn source_levels(prepared: &PreparedText) -> Vec<std::collections::BTreeSet<u8>> {
    // An actual occurrence may partition a prepared source view. Compare the
    // original code units' resolved levels, not incidental view partitioning.
    (0..4)
        .map(|index| {
            prepared
                .unit_views()
                .filter(|unit| unit.source_range().contains(index))
                .map(|unit| unit.level().get())
                .collect()
        })
        .collect()
}

fn glyph_facts(line: &FinalizedLine) -> Vec<(Range, u32, f32, f32, f32, f64)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                        run.origin().x + glyph.pen(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn positioned_inside_real_arabic_cell_preserves_isol_mark_joiner_context_and_bidi_for_both_affinities()
 {
    let bytes = arabic_font();
    qualify(&bytes, &[(0x628, 35), (0x64e, 100), (0x200d, 253)]);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(
        parsed
            .glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(100)),
        Some(0.0)
    );
    assert!(parsed.gpos().is_ok());
    let mut system = system_with(&bytes);
    let control = prepare(&mut system, arabic_input(None));
    let plain = arabic_finish(&control);
    let expected = [
        (Range::new(0, 3), 36),
        (Range::new(0, 3), 100),
        (Range::new(3, 4), 274),
    ];
    assert_eq!(
        glyph_facts(&plain)
            .iter()
            .map(|g| (g.0, g.1))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        (
            plain.feedback().advance,
            plain.feedback().baseline,
            plain.feedback().line_height
        ),
        (24.0, 6.0, 12.0)
    );
    let levels = source_levels(&control);
    assert!(levels.iter().all(|level| !level.is_empty()));
    for affinity in [Affinity::Before, Affinity::After] {
        let prepared = prepare(&mut system, arabic_input(Some(affinity)));
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(3, 4)]
        );
        assert_eq!(source_levels(&prepared), levels);
        assert_eq!(
            prepared
                .opportunity_views()
                .filter(|offer| offer.kind() == PreparedBreakKind::Allowed)
                .count(),
            control
                .opportunity_views()
                .filter(|offer| offer.kind() == PreparedBreakKind::Allowed)
                .count()
        );
        let line = arabic_finish(&prepared);
        assert_eq!(glyph_facts(&line), glyph_facts(&plain));
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (24.0, 6.0, 12.0)
        );
        let atomic = prepared.atomic_views().next().unwrap().handle();
        assert_eq!(atomic.view().source_position().get(), 1);
        let slot = line.atomic_slots().next().unwrap();
        assert_eq!(slot.occurrence, atomic);
        assert_eq!(slot.allocation, AtomicLineAllocation::OutOfFlow);
        let layout =
            Layout::from_finalized_lines(placement(&prepared, &line, vec![physical(atomic)]))
                .unwrap();
        let actual = layout
            .glyph_runs()
            .into_iter()
            .flat_map(|run| {
                run.glyphs()
                    .iter()
                    .map(|glyph| {
                        (
                            glyph.id(),
                            Point::new(
                                run.origin().x + f64::from(glyph.x()),
                                run.origin().y + glyph.pen() + f64::from(glyph.y()),
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        // Real GSUB context before isol chooses36 through the actual ZWJ;
        // authored GPOS anchors(400,600)-(100,200) at size10 give(+3,-4).
        assert_eq!(
            actual,
            [
                (36, Point::new(94.0, 29.0)),
                (100, Point::new(97.0, 25.0)),
                (274, Point::new(94.0, 39.0))
            ]
        );
        assert_eq!(layout.source(), prepared.analyzed_source().source());
        assert_eq!(
            layout.hit(Point::new(310.0, 510.0)),
            crate::Hit::InlineBox(BOX)
        );
    }
}
