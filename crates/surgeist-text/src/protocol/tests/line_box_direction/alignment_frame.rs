//! Resolved line-relative padding changes alignment, not the selected text frame.
use super::*;

fn framed(
    p: &PreparedText,
    box_direction: LineBoxDirection,
    alignment: Alignment,
    insets: (f64, f64),
) -> LineSelectionRequest {
    let mut r = context(p, box_direction, Some(100.0), 0.0, alignment);
    r.context.alignment_insets = LineAlignmentInsets::try_new(insets.0, insets.1).unwrap();
    r
}

fn natural_measures(line: &FinalizedLine) -> [InlineMeasure; 4] {
    let m = line.measurements();
    [
        m.fit(),
        m.alignment(),
        m.min_content_contribution(),
        m.max_content_contribution(),
    ]
}

fn assert_outer_frame(p: &PreparedText, line: FinalizedLine, text: &str) -> Layout {
    assert_eq!(line.prepared_text(), p.clone());
    let layout = place(p.clone(), line, 0.0, 8.0).unwrap();
    assert_eq!(
        layout.source().text().code_units(),
        text.encode_utf16().collect::<Vec<_>>()
    );
    let lines = layout.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].range(), Range::new(0, text.encode_utf16().count()));
    assert_eq!(lines[0].bounds(), Rect::new(0.0, 0.0, 100.0, 10.0));
    assert_eq!(lines[0].baseline(), 8.0);
    layout
}

#[test]
fn line_relative_frames_align_real_sixty_and_forty_unit_bodies_without_changing_the_outer_frame() {
    for body in BODIES {
        for (text, natural, left_center, right_center, rtl_start) in [
            ("AAAAAA", 60.0, 0.0, 40.0, 20.0),
            ("AAAA", 40.0, 10.0, 50.0, 40.0),
        ] {
            let p = capture(text, body, WhiteSpace::Preserve, false);
            for (direction, alignment, sides, start) in [
                (BOXES[0], Alignment::Start, (20.0, 20.0), 20.0),
                (BOXES[1], Alignment::Start, (20.0, 20.0), rtl_start),
                (BOXES[0], Alignment::Center, (0.0, 40.0), left_center),
                (BOXES[1], Alignment::Center, (0.0, 40.0), left_center),
                (BOXES[0], Alignment::Center, (40.0, 0.0), right_center),
                (BOXES[1], Alignment::Center, (40.0, 0.0), right_center),
            ] {
                let before = finish(&p, context(&p, direction, Some(100.0), 0.0, alignment));
                let r = framed(&p, direction, alignment, sides);
                assert_eq!(r.start, before.selected_line().request().start);
                assert_eq!(r.end, before.selected_line().request().end);
                assert_eq!(r.termination, before.selected_line().request().termination);
                let after = finish(&p, r);
                assert_eq!(after.feedback().advance, natural);
                assert_eq!(natural_measures(&after), natural_measures(&before));
                assert_eq!(after.feedback().line_height, 10.0);
                assert_eq!(after.feedback().baseline, 8.0);
                assert_eq!(after.selected_line().request().context.base_direction, body);
                assert_eq!(
                    after.selected_line().request().context.available_inline,
                    Some(100.0)
                );
                assert_eq!(
                    glyphs(&after),
                    (0..text.len())
                        .map(|i| (Range::new(i, i + 1), 35, 10.0, start + 10.0 * i as f64))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    after
                        .mapping_views()
                        .map(|m| (m.source_range(), m.presentation_range()))
                        .collect::<Vec<_>>(),
                    before
                        .mapping_views()
                        .map(|m| (m.source_range(), m.presentation_range()))
                        .collect::<Vec<_>>()
                );
                let delta = xs(&after)[0] - xs(&before)[0];
                let old = assert_outer_frame(&p, before, text);
                let new = assert_outer_frame(&p, after, text);
                for at in 0..=text.len() {
                    for affinity in [Affinity::Before, Affinity::After] {
                        let a = old.cursor(Cursor::new(at, affinity)).unwrap().rect();
                        let b = new.cursor(Cursor::new(at, affinity)).unwrap().rect();
                        assert_eq!(b.origin.x - a.origin.x, delta);
                        assert_eq!(b.origin.y, a.origin.y);
                        assert_eq!(b.size, a.size);
                    }
                }
            }
        }
    }
}

#[test]
fn justification_spends_only_the_real_reduced_capacity_on_authenticated_space_slots() {
    for body in BODIES {
        for direction in BOXES {
            for (text, natural, amount, space_range, expected) in [
                (
                    "AA AAA",
                    60.0,
                    0.0,
                    Range::new(2, 3),
                    vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0],
                ),
                (
                    "A AA",
                    40.0,
                    20.0,
                    Range::new(1, 2),
                    vec![20.0, 30.0, 60.0, 70.0],
                ),
            ] {
                let p = capture(text, body, WhiteSpace::Preserve, false);
                let mut r = framed(&p, direction, Alignment::Justify, (20.0, 20.0));
                r.context.spacing = SpacingPolicy::Spaces;
                let selected = p.select(r).unwrap();
                let unexpanded = selected
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::None,
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();
                assert_eq!(unexpanded.measurements().alignment().advance(), natural);
                let slots: Vec<_> = selected.spacing_slots().collect();
                assert_eq!(slots.len(), 1);
                assert_eq!(
                    slots[0]
                        .pieces()
                        .map(|piece| piece.owner())
                        .collect::<Vec<_>>(),
                    [crate::SpacingSlotOwner::Source {
                        origin: p.origin_views().next().unwrap().handle(),
                        local: space_range,
                        aggregate: space_range,
                    }]
                );
                let expanded = selected
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::Complete(vec![SpacingAllocation {
                            slot: slots[0].clone(),
                            amount,
                        }]),
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();
                assert_eq!(expanded.selected_line(), selected);
                assert_eq!(expanded.feedback().advance, 60.0);
                assert_eq!(expanded.measurements().alignment().advance(), 60.0);
                assert_eq!(xs(&expanded), expected);
                // Allocation changes spacing after the real SPACE, not font advances.
                assert_eq!(
                    glyphs(&expanded).iter().map(|g| g.2).collect::<Vec<_>>(),
                    vec![10.0; text.len()]
                );
                assert_outer_frame(&p, expanded, text);
                assert_eq!(unexpanded.feedback().advance, natural);
            }
        }
    }
}

#[test]
fn signed_indentation_and_overconstrained_frames_preserve_the_resolved_overflow_start() {
    for body in BODIES {
        for (text, indent, sides, starts) in [
            ("AA", -5.0, (20.0, 20.0), [15.0, 65.0]),
            ("AA", 5.0, (20.0, 20.0), [25.0, 55.0]),
            ("AAAAAAAAAAAA", 5.0, (20.0, 20.0), [25.0, -45.0]),
            // Positive sides may exceed W: this is an overconstrained frame,
            // not an invalid declaration and not a clamp on signed geometry.
            ("AA", 0.0, (60.0, 60.0), [60.0, 20.0]),
        ] {
            let p = capture(text, body, WhiteSpace::Preserve, false);
            for (i, direction) in BOXES.into_iter().enumerate() {
                let mut r = framed(&p, direction, Alignment::Start, sides);
                r.context.indent = indent;
                let line = finish(&p, r);
                assert_eq!(xs(&line)[0], starts[i]);
                assert_eq!(line.feedback().advance, text.len() as f64 * 10.0);
                assert_outer_frame(&p, line, text);
            }
        }
    }
}

#[test]
fn alignment_insets_leave_css_tab_resolution_in_the_original_reference_frame() {
    let p = capture("A\tX", BODIES[0], WhiteSpace::Preserve, true);
    for (direction, expected) in [(BOXES[0], [20.0, 60.0]), (BOXES[1], [30.0, 70.0])] {
        let mut r = framed(&p, direction, Alignment::Start, (20.0, 20.0));
        r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
        let mut original = r.clone();
        original.context.alignment_insets = LineAlignmentInsets::ZERO;
        let before = finish(&p, original);
        let line = finish(&p, r);
        assert_eq!(line.feedback().advance, 50.0);
        assert_eq!(xs(&line), expected);
        assert_eq!(
            glyphs(&line)
                .iter()
                .map(|g| (g.0, g.1, g.2))
                .collect::<Vec<_>>(),
            [(Range::new(0, 1), 35, 10.0), (Range::new(2, 3), 58, 10.0)]
        );
        assert_eq!(natural_measures(&line), natural_measures(&before));
        assert_eq!(
            line.selected_line().request().context.tabs,
            before.selected_line().request().context.tabs
        );
        assert_outer_frame(&p, line, "A\tX");
    }
}

#[test]
fn a_small_alignment_frame_does_not_reclassify_conditional_preserved_space_as_overflowing() {
    let p = capture("A ", BODIES[1], WhiteSpace::Preserve, false);
    let mut r = context(&p, BOXES[0], Some(40.0), 5.0, Alignment::Start);
    let old = finish(&p, r.clone());
    r.context.alignment_insets = LineAlignmentInsets::try_new(15.0, 15.0).unwrap();
    let new = finish(&p, r);
    for line in [&old, &new] {
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(line.measurements().alignment().advance(), 20.0);
        assert_eq!(
            (line.feedback().hanging_start, line.feedback().hanging_end),
            (0.0, 0.0)
        );
        let edge = line
            .edge_units()
            .find(|edge| edge.source_pieces().any(|r| r == Range::new(1, 2)))
            .unwrap();
        assert_eq!(
            edge.disposition(),
            LineEdgeDisposition::Conditional {
                side: LineEdgeSide::Start,
                eligible: false
            }
        );
        assert_eq!(edge.excluded_contribution(), 0.0);
    }
    assert_eq!(natural_measures(&new), natural_measures(&old));
    let a = |line: &FinalizedLine| {
        glyphs(line)
            .into_iter()
            .find(|g| g.0 == Range::new(0, 1))
            .unwrap()
            .3
    };
    assert_eq!((a(&old), a(&new)), (15.0, 30.0));
}

#[test]
fn frame_identity_is_semantic_but_conditional_candidates_may_use_their_actual_fresh_frame() {
    let p = capture("AAAA", BODIES[0], WhiteSpace::Preserve, false);
    let failed_frame = |sides| {
        let mut r = framed(&p, BOXES[0], Alignment::Start, sides);
        r.context.available_inline = Some(25.0);
        finish(&p, r)
    };
    let first = failed_frame((2.0, 2.0));
    let repeat = failed_frame((2.0, 2.0));
    let other = failed_frame((1.0, 3.0));
    let a = place(p.clone(), first.clone(), 0.0, 8.0).unwrap();
    let b = place(p.clone(), repeat, 0.0, 8.0).unwrap();
    let c = place(p.clone(), other.clone(), 0.0, 8.0).unwrap();
    assert_eq!(a.key(), b.key());
    assert_ne!(a.key(), c.key());
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = p
        .opportunity_views()
        .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let search = |frame: &FinalizedLine| {
        p.conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(frame),
        )
        .unwrap()
    };
    assert_eq!(search(&first), search(&first));
    assert_ne!(search(&first), search(&other));
    let batch = search(&first).begin(&[]).unwrap();
    let cut = batch
        .opportunities()
        .iter()
        .find(|c| c.boundary().view().source_after().get() == 2)
        .unwrap();
    let mut r = framed(&p, BOXES[0], Alignment::Start, (1.0, 3.0));
    r.context.available_inline = Some(25.0);
    r.start = cut.start();
    r.end = cut.boundary();
    r.participants.truncate(r.end.view().participant_offset());
    r.termination = PreparedTermination::Conditional(cut.clone());
    r.context.end = LineEnd::Soft;
    let prefix = finish(&p, r);
    assert_eq!(xs(&prefix), [1.0, 11.0]);
    assert_eq!(prefix.feedback().advance, 20.0);
    assert_eq!(xs(&first), [2.0, 12.0, 22.0, 32.0]);
    assert_eq!(a.source(), c.source());
}

#[test]
fn insets_validate_intrinsically_and_nonzero_insets_require_a_finite_outer_frame() {
    assert_eq!(LineAlignmentInsets::ZERO.line_left(), 0.0);
    assert_eq!(LineAlignmentInsets::ZERO.line_right(), 0.0);
    let zero = LineAlignmentInsets::try_new(-0.0, -0.0).unwrap();
    assert_eq!(zero, LineAlignmentInsets::ZERO);
    assert_eq!(zero.line_left().to_bits(), 0.0f64.to_bits());
    assert_eq!(zero.line_right().to_bits(), 0.0f64.to_bits());
    for (left, right, category) in [
        (-1.0, 0.0, TextNumericError::OutOfRange),
        (0.0, -1.0, TextNumericError::OutOfRange),
        (f64::NAN, 0.0, TextNumericError::NonFinite),
        (0.0, f64::INFINITY, TextNumericError::NonFinite),
        (f64::NEG_INFINITY, 0.0, TextNumericError::NonFinite),
    ] {
        let error = LineAlignmentInsets::try_new(left, right).unwrap_err();
        assert!(
            matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
            phase: TextNumericPhase::Selection, category: actual, ..
        }) if *actual == category)
        );
    }
    let p = capture("AA", BODIES[0], WhiteSpace::Preserve, false);
    let mut r = context(&p, BOXES[0], None, 0.0, Alignment::Start);
    r.context.alignment_insets = LineAlignmentInsets::ZERO;
    assert_eq!(finish(&p, r.clone()).feedback().advance, 20.0);
    r.context.alignment_insets = LineAlignmentInsets::try_new(1.0, 0.0).unwrap();
    assert_eq!(
        reason(&p.select(r).unwrap_err()),
        &ProtocolError::UnsupportedContext {
            field: ProtocolContextField::AlignmentInsets
        }
    );
}

#[test]
fn real_atomic_and_positive_endpoint_allocations_follow_the_same_alignment_translation() {
    let mut source = Source::identified("AA", Id::from_u64(147801), 1);
    source.inline_box(InlineBox::new(
        Id::from_u64(147802),
        InlineBoxKind::InFlow,
        1,
        Size::new(20.0, 10.0),
    ));
    let base = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    };
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: base,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: BODIES[0],
            origins: vec![ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, 2),
                aggregate: Range::new(0, 2),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            replacements: vec![],
            controls: vec![
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphControl {
                    at: 2,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            anchor_order: vec![
                ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(0),
                    affinity: Affinity::After,
                },
                ParagraphAnchor {
                    item: ParagraphAnchorItem::Atomic(0),
                    affinity: Affinity::After,
                },
                ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(1),
                    affinity: Affinity::After,
                },
            ],
            boundary_policies: [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .collect(),
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = system();
    let p = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let atomic = p.atomic_views().next().unwrap().handle();
    let controls = p.control_views().map(|c| c.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    let allocation = AtomicLineAllocation::InFlow {
        flow: AtomicFlowAllocation {
            advance: 20.0,
            before_baseline: 8.0,
            after_baseline: 2.0,
            occupied_start: 0.0,
            occupied_end: 20.0,
        },
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -8.0, 20.0, 10.0),
            content: Rect::new(0.0, -8.0, 20.0, 10.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Baseline,
    };
    for (direction, glyph_positions, atomic_x, endpoint_xs) in [
        (BOXES[0], [22.0, 52.0], 32.0, [20.0, 62.0]),
        (BOXES[1], [37.0, 67.0], 47.0, [35.0, 77.0]),
    ] {
        let mut r = framed(&p, direction, Alignment::Start, (20.0, 20.0));
        r.atomics = vec![AtomicAllocation {
            occurrence: atomic.clone(),
            allocation,
        }];
        r.controls = controls
            .iter()
            .zip([2.0, 3.0])
            .map(|(occurrence, advance)| ControlAllocation {
                occurrence: occurrence.clone(),
                strut: Some(ControlStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                    alignment: LineBoxAlignment::Baseline,
                }),
                boundary: Some(InlineBoundaryAllocation::try_new(advance).unwrap()),
            })
            .collect();
        let line = finish(&p, r);
        assert_eq!(line.feedback().advance, 45.0);
        assert_eq!(line.measurements().alignment().advance(), 45.0);
        assert_eq!(
            glyphs(&line),
            [
                (Range::new(0, 1), 35, 10.0, glyph_positions[0]),
                (Range::new(1, 2), 35, 10.0, glyph_positions[1])
            ]
        );
        let slots = line.atomic_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].occurrence, atomic);
        assert_eq!(slots[0].allocation, allocation);
        assert_eq!(slots[0].origin.x, atomic_x);
        let actual_controls = line.control_slots().collect::<Vec<_>>();
        assert_eq!(
            actual_controls
                .iter()
                .map(|c| c.control())
                .collect::<Vec<_>>(),
            controls
        );
        assert_eq!(
            actual_controls
                .iter()
                .map(|c| c.slot().x)
                .collect::<Vec<_>>(),
            endpoint_xs
        );
        // The exact real allocated box is also supplied at final placement.
        let layout = Layout::from_finalized_lines(PlacementRequest {
            prepared: p.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![AtomicPlacement {
                    occurrence: atomic.clone(),
                    border: Rect::new(atomic_x, 0.0, 20.0, 10.0),
                    content: Rect::new(atomic_x, 0.0, 20.0, 10.0),
                    baseline: 8.0,
                }],
            })],
        })
        .unwrap();
        assert_eq!(layout.source().text().code_units(), [65, 65]);
        assert_eq!(layout.lines()[0].bounds(), Rect::new(0.0, 0.0, 100.0, 10.0));
    }
}
