use super::*;
use crate::*;

fn fixture() -> PreparedText {
    fixture_controls(vec![
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ])
}

fn fixture_controls(controls: Vec<ParagraphControl>) -> PreparedText {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
        .unwrap();
    let source = Source::new("AB");
    let analyzed = AnalyzedSource::from_paragraph(ParagraphInput {
        source: source.clone(),
        base_style: Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            word_break: WordBreak::BreakAll,
            ..Style::default()
        },
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            source,
            local: crate::Range::new(0, 2),
            aggregate: crate::Range::new(0, 2),
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        }],
        connections: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: (0..controls.len())
            .map(|control| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(control),
                affinity: Affinity::Before,
            })
            .collect(),
        controls,
    })
    .unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn first_request(prepared: &PreparedText) -> LineSelectionRequest {
    let end = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().participant_offset() == 1
        })
        .unwrap()
        .handle();
    LineSelectionRequest {
        start: prepared.boundary_views().next().unwrap().handle(),
        end: end.view().boundary(),
        termination: PreparedTermination::Ordinary(end),
        participants: prepared.participants().take(1).collect(),
        incoming: None,
        alternative: None,
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        context: LineContext {
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: crate::LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Horizontal,
            start: LineStart::Paragraph,
            end: LineEnd::Soft,
            available_inline: Some(20.0),
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing: SpacingPolicy::None,
        },
    }
}
fn reason(error: &Error) -> &ProtocolError {
    let Some(ErrorDetail::InvalidTextProtocol { reason }) = error.detail() else {
        panic!("wrong error {error:?}")
    };
    reason
}

#[test]
fn forced_event_stops_context_before_same_atom_endpoints() {
    let prepared = fixture_controls(vec![
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::ForcedBreak,
        },
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let mut request = first_request(&fixture());
    request.start = prepared.boundary_views().next().unwrap().handle();
    request.end = end.view().boundary();
    request.participants = prepared
        .participants()
        .take(request.end.view().participant_offset())
        .collect();
    request.controls = request
        .participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
                boundary: None,
            }),
            _ => None,
        })
        .collect();
    request.termination = PreparedTermination::Ordinary(end);
    request.context.end = LineEnd::Forced;
    assert!(
        prepared
            .boundary_context_demand(&request)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        prepared
            .select(request.clone())
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        10.0
    );
    let unreachable = prepared
        .control_views()
        .find(|control| *control.kind() == ParagraphControlKind::InlineBoundaryStart)
        .unwrap()
        .handle();
    request
        .boundary_context
        .push(InlineBoundaryContextAllocation {
            occurrence: unreachable,
            boundary: InlineBoundaryAllocation::ZERO,
        });
    assert!(matches!(
        reason(&prepared.boundary_context_demand(&request).unwrap_err()),
        ProtocolError::UnexpectedBoundaryContext { .. }
    ));
}
#[test]
fn endpoint_allocation_rejects_invalid_numbers_and_normalizes_zero() {
    assert_eq!(
        InlineBoundaryAllocation::try_new(-0.0)
            .unwrap()
            .advance()
            .to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        InlineBoundaryAllocation::try_new(8.0).unwrap().advance(),
        8.0
    );
    for (value, expected) in [
        (-1.0, TextNumericError::OutOfRange),
        (f64::NAN, TextNumericError::NonFinite),
        (f64::INFINITY, TextNumericError::NonFinite),
    ] {
        assert!(
            matches!(InlineBoundaryAllocation::try_new(value).unwrap_err().detail(), Some(ErrorDetail::InvalidTextNumeric { field: "inline boundary advance", phase: TextNumericPhase::Selection, category, .. }) if *category == expected)
        );
    }
}
#[test]
fn demand_stops_at_positive_context_and_zero_reveals_the_next_endpoint() {
    let prepared = fixture();
    let mut request = first_request(&prepared);
    let first = prepared.boundary_context_demand(&request).unwrap().unwrap();
    assert!(
        matches!(reason(&prepared.select(request.clone()).unwrap_err()), ProtocolError::MissingBoundaryContext { occurrence } if *occurrence == first.index)
    );
    request
        .boundary_context
        .push(InlineBoundaryContextAllocation {
            occurrence: first.clone(),
            boundary: InlineBoundaryAllocation::try_new(5.0).unwrap(),
        });
    assert!(
        prepared
            .boundary_context_demand(&request)
            .unwrap()
            .is_none()
    );
    let positive = prepared
        .select(request.clone())
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(positive.feedback().advance, 10.0);
    request.boundary_context[0].boundary = InlineBoundaryAllocation::ZERO;
    let farther = prepared.boundary_context_demand(&request).unwrap().unwrap();
    assert_ne!(farther, first);
    request
        .boundary_context
        .push(InlineBoundaryContextAllocation {
            occurrence: farther.clone(),
            boundary: InlineBoundaryAllocation::ZERO,
        });
    assert!(
        prepared
            .boundary_context_demand(&request)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        prepared
            .select(request.clone())
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        10.0
    );
    request.boundary_context[0].boundary = InlineBoundaryAllocation::try_new(5.0).unwrap();
    assert!(
        matches!(reason(&prepared.select(request.clone()).unwrap_err()), ProtocolError::UnexpectedBoundaryContext { occurrence } if *occurrence == farther.index)
    );
    request.boundary_context.swap(0, 1);
    assert!(
        matches!(reason(&prepared.boundary_context_demand(&request).unwrap_err()), ProtocolError::UnexpectedBoundaryContext { occurrence } if *occurrence == farther.index)
    );
    assert_eq!(positive.feedback().advance, 10.0);
}
#[test]
fn selected_endpoints_contribute_once_to_flow_prefix_and_exact_event_selection() {
    let prepared = fixture();
    let mut request = first_request(&prepared);
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    request.end = end.view().boundary();
    request.termination = PreparedTermination::Ordinary(end);
    request.context.end = LineEnd::Paragraph;
    request.context.available_inline = Some(40.0);
    request.participants = prepared.participants().collect();
    request.controls = prepared
        .control_views()
        .map(|control| ControlAllocation {
            occurrence: control.handle(),
            strut: None,
            boundary: Some(
                InlineBoundaryAllocation::try_new(
                    if *control.kind() == ParagraphControlKind::InlineBoundaryStart {
                        5.0
                    } else {
                        8.0
                    },
                )
                .unwrap(),
            ),
        })
        .collect();
    let mut missing = request.clone();
    missing.controls[0].boundary = None;
    assert!(matches!(
        reason(&prepared.select(missing).unwrap_err()),
        ProtocolError::MissingControlBoundaryGeometry { occurrence: 0 }
    ));
    let mut overlap = request.clone();
    overlap
        .boundary_context
        .push(InlineBoundaryContextAllocation {
            occurrence: request.controls[0].occurrence.clone(),
            boundary: InlineBoundaryAllocation::ZERO,
        });
    assert!(matches!(
        reason(&prepared.boundary_context_demand(&overlap).unwrap_err()),
        ProtocolError::DuplicateBoundaryContext { occurrence: 0 }
    ));
    let mut overflow = request.clone();
    for control in &mut overflow.controls {
        control.boundary = Some(InlineBoundaryAllocation::try_new(f64::MAX).unwrap());
    }
    let line = prepared
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(matches!(
        prepared
            .select(overflow)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::InvalidTextNumeric {
            phase: TextNumericPhase::Finalization,
            category: TextNumericError::NonZeroLost,
            ..
        })
    ));
    assert_eq!(line.feedback().advance, 33.0);
    for measure in [
        line.measurements().fit(),
        line.measurements().alignment(),
        line.measurements().min_content_contribution(),
        line.measurements().max_content_contribution(),
    ] {
        assert_eq!(measure.advance(), 33.0);
        assert_eq!(measure.occupied_end(), 33.0);
    }
    assert_eq!(
        line.control_slots()
            .map(|control| control.inline_bounds())
            .collect::<Vec<_>>(),
        [(10.0, 15.0), (25.0, 33.0)]
    );
    let start = prepared.control_views().next().unwrap().handle();
    assert_eq!(
        line.prefix_at(&TextSeam::AfterParticipant(PreparedParticipant::Control(
            start
        )))
        .unwrap()
        .advance(),
        15.0
    );
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: vec![LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 10.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let fragments = layout
        .source_range_fragments(&origin, crate::Range::new(1, 2))
        .unwrap();
    let pieces: Vec<_> = fragments
        .units()
        .flat_map(|unit| unit.pieces().collect::<Vec<_>>())
        .collect();
    assert_eq!(
        (pieces[0].rect().origin.x, pieces[0].rect().size.width),
        (15.0, 10.0)
    );
    let positions: Vec<_> = layout.positions().collect();
    let before = positions
        .iter()
        .find(|position| {
            position.source_cursor() == Cursor::new(1, Affinity::After)
                && layout.position_geometry(position).unwrap().rect().origin.x == 10.0
        })
        .unwrap();
    let after = positions
        .iter()
        .find(|position| {
            position.source_cursor() == Cursor::new(2, Affinity::Before)
                && layout.position_geometry(position).unwrap().rect().origin.x == 33.0
        })
        .unwrap();
    let selected = layout.position_selection(before, after).unwrap();
    assert_eq!(selected.source_text().code_units(), [66]);
    assert_eq!(selected.plaintext().code_units(), [66]);
    assert_eq!(
        selected
            .geometry()
            .rects()
            .iter()
            .map(|rect| (rect.rect().origin.x, rect.rect().size.width))
            .collect::<Vec<_>>(),
        [(10.0, 23.0)]
    );
}
