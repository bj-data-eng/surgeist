use super::*;
use crate::*;

fn input(left: Text, right: Text) -> ContextualSourceInput {
    let split = left.len();
    let mut units = left.code_units().to_vec();
    units.extend(right.code_units());
    let source = Source::new(Text::from_code_units(units));
    let end = source.text().len();
    ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        computed_line_boundaries: Vec::new(),
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: [Range::new(0, split), Range::new(split, end)]
            .into_iter()
            .map(|range| ParagraphOrigin {
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            })
            .collect(),
        connections: vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: ParagraphConnectionKind::Continuous,
        }],
        controls: Vec::new(),
        boundary_policies: vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }],
        replacements: Vec::new(),
        scopes: vec![
            SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: SourceProcessingTopology::Complete,
                trim: WhiteSpaceTrim::default(),
                base_style: Style::default(),
                used_direction: ParagraphBaseDirection::LeftToRight,
            },
            SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: WhiteSpaceTrim::default(),
                style: None,
                boundaries: None,
            },
        ],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::EnterScope(1),
            SourceProcessingEvent::Connection(0),
            SourceProcessingEvent::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitScope(1),
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    }
}
fn before() -> ComputedLineBoundaryInput {
    ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: SourceBoxSide::Before,
    }
}
fn analyze(input: ContextualSourceInput) -> AnalyzedSource {
    let processed = Arc::new(process_contextual_source(input).unwrap());
    AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap()
}

#[test]
fn computed_force_prevents_contextual_removal_without_fabricating_source_control() {
    let mut raw = input(Text::from("i"), Text::from("\u{307}"));
    let SourceProcessingScope::Block { base_style, .. } = &mut raw.scopes[0] else {
        unreachable!()
    };
    base_style.locale = Some("lt".into());
    base_style.text_transform =
        TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Uppercase));
    let continuous = analyze(raw.clone());
    assert_eq!(continuous.inner.projection.text, "I");
    raw.computed_line_boundaries.push(before());
    let forced = analyze(raw);
    assert_eq!(forced.inner.projection.text, "I\u{307}");
    assert_eq!(forced.source().text(), &Text::from("i\u{307}"));
    assert_eq!(
        forced.processed_source().processed_text(),
        forced.source().text()
    );
    assert_eq!(forced.inner.paragraph.controls.len(), 0);
    assert_eq!(forced.inner.analysis.graphemes, [0..1, 1..2]);
    let cause = forced
        .processed_source()
        .captured_computed_line_boundary(0)
        .unwrap();
    assert_eq!(cause.source_position().get(), 1);
    assert!(matches!(
        cause.target(),
        CapturedComputedLineBoundaryTarget::Scope {
            side: SourceBoxSide::Before,
            ..
        }
    ));
}

#[test]
fn same_target_is_one_cause_but_empty_scope_edges_remain_distinct() {
    let mut raw = input(Text::from("A"), Text::from(""));
    raw.computed_line_boundaries = vec![
        before(),
        before(),
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::After,
        },
    ];
    let processed = process_contextual_source(raw).unwrap();
    let a = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        Some(a.clone()),
        processed.captured_computed_line_boundary(1)
    );
    assert_ne!(Some(a), processed.captured_computed_line_boundary(2));
    assert_eq!(processed.paragraphs[0].data.computed_boundaries.len(), 2);
    assert!(processed.captured_control(0).is_none());
}

#[test]
fn associated_scope_and_endpoint_retain_two_causes_at_one_group() {
    let mut raw = input(Text::from("A"), Text::from("B"));
    raw.controls = vec![
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let SourceProcessingScope::Inline { boundaries, .. } = &mut raw.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(SourceProcessingScopeBoundaries { start: 0, end: 1 });
    raw.events.insert(
        4,
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::After,
        },
    );
    let exit = raw
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::ExitScope(1)))
        .unwrap();
    raw.events.insert(
        exit,
        SourceProcessingEvent::Control {
            control: 1,
            affinity: Affinity::Before,
        },
    );
    raw.computed_line_boundaries =
        vec![before(), ComputedLineBoundaryInput::Endpoint { control: 0 }];
    let processed = process_contextual_source(raw).unwrap();
    assert_ne!(
        processed.captured_computed_line_boundary(0),
        processed.captured_computed_line_boundary(1)
    );
    assert_eq!(processed.paragraphs[0].data.computed_boundaries.len(), 1);
    assert_eq!(
        processed.paragraphs[0].data.computed_boundaries[0]
            .causes
            .len(),
        2
    );
    assert_eq!(processed.paragraphs[0].data.controls.len(), 2);
}

#[test]
fn hard_unit_split_is_early_and_invalid_scalar_or_delimiter_is_precise() {
    let mut raw = input(Text::from("A"), Text::from("\u{301}B"));
    let continuous = analyze(raw.clone());
    assert_eq!(continuous.inner.projection.graphemes, [0, 2, 3]);
    raw.computed_line_boundaries.push(before());
    assert_eq!(analyze(raw).inner.projection.graphemes, [0, 1, 2, 3]);
    let mut scalar = input(
        Text::from_code_units(vec![0xd83d]),
        Text::from_code_units(vec![0xde00]),
    );
    scalar.computed_line_boundaries.push(before());
    let mut delimiter = input(Text::from("\r"), Text::from("\n"));
    delimiter.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(0, 2)]);
    delimiter.computed_line_boundaries.push(before());
    for (raw, reason) in [
        (scalar, ComputedLineBoundaryError::InsideScalar),
        (delimiter, ComputedLineBoundaryError::InsideDelimiter),
    ] {
        assert!(
            matches!(process_contextual_source(raw).unwrap_err().detail(),
            Some(ErrorDetail::SourceProcessing { reason: SourceProcessingError::ComputedLineBoundary { request: 0, reason: actual } }) if *actual == reason)
        );
    }
}

#[test]
fn real_native_finalization_consumes_computed_forces_without_host_control_or_fragment() {
    for (left, right, requests, advances) in [
        ("A", "B", vec![before()], vec![10.0, 10.0]),
        ("", "A", vec![before()], vec![0.0, 10.0]),
        (
            "A",
            "",
            vec![
                before(),
                ComputedLineBoundaryInput::Scope {
                    scope: 1,
                    side: SourceBoxSide::After,
                },
            ],
            vec![10.0, 0.0, 0.0],
        ),
    ] {
        let mut raw = input(Text::from(left), Text::from(right));
        raw.computed_line_boundaries = requests;
        let SourceProcessingScope::Block { base_style, .. } = &mut raw.scopes[0] else {
            unreachable!()
        };
        base_style.font = Font::new().family("Ahem");
        base_style.size = 10.0;
        base_style.line_height = LineHeight::Absolute(10.0);
        let analyzed = crate::test_wrapping::bind_missing(&analyze(raw), Wrap::Word);
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(
                include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            )
            .unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        assert_eq!(prepared.control_views().count(), 0);
        let mut start = prepared.boundary_views().next().unwrap().handle();
        let mut actual = Vec::new();
        for offer in prepared.opportunity_views().filter(|offer| {
            matches!(
                offer.kind(),
                PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
            )
        }) {
            let end = offer.boundary();
            let from = start.view().participant_offset();
            let to = end.view().participant_offset();
            if offer.kind() == PreparedBreakKind::Forced {
                assert!(offer.computed_line_boundary().is_some());
            }
            let selected = prepared
                .select(LineSelectionRequest {
                    inline_boxes: None,
                    segment: crate::test_wrapping::only_source_segment(&prepared),
                    start,
                    end: end.clone(),
                    participants: prepared.participants().skip(from).take(to - from).collect(),
                    termination: PreparedTermination::Ordinary(offer.handle()),
                    alternative: None,
                    incoming: None,
                    atomics: Vec::new(),
                    controls: Vec::new(),
                    boundary_context: Vec::new(),
                    line_box_struts: Vec::new(),
                    line_strut: None,
                    context: LineContext {
                        alignment_insets: crate::LineAlignmentInsets::ZERO,
                        formatted_line: FormattedLineContext::default(),
                        base_direction: ParagraphBaseDirection::LeftToRight,
                        line_box_direction: LineBoxDirection::LeftToRight,
                        orientation: LineOrientation::Horizontal,
                        start: if actual.is_empty() {
                            LineStart::Paragraph
                        } else {
                            LineStart::Continuation
                        },
                        end: if offer.kind() == PreparedBreakKind::Forced {
                            LineEnd::Forced
                        } else {
                            LineEnd::Paragraph
                        },
                        available_inline: Some(100.0),
                        indent: 0.0,
                        alignment: Alignment::Left,
                        tabs: TabContext::NoTabs,
                        spacing: SpacingPolicy::None,
                    },
                })
                .unwrap();
            let finalized = selected
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            assert_eq!(finalized.control_slots().count(), 0);
            actual.push(finalized.measurements().fit().advance());
            start = end;
        }
        assert_eq!(actual, advances);
    }
}
