//! Genuine Wbr separates original UTF-16 decoding without disconnecting context.
use super::*;

fn capture(units: Vec<u16>, mode: WhiteSpace) -> ContextualSourceInput {
    let mut p = input(Text::from_code_units(units), mode);
    p.origins = vec![
        origin(&p.source, Range::new(0, 1)),
        origin(&p.source, Range::new(1, p.source.text().len())),
    ];
    p.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    p.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    p.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    p
}
fn with_wbr(mut p: ContextualSourceInput, operation: WordSpaceTransform) -> ContextualSourceInput {
    p.controls.push(ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::Wbr {
            style: Box::new(Style {
                word_space_transform: operation,
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            }),
        },
    });
    p.events.insert(
        4,
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::After,
        },
    );
    p
}

#[test]
fn none_wbr_splits_original_decoding_before_phase_one_wide_context() {
    // U+20000 is Wide/Han. Its isolated low unit has no scalar width context.
    // CSS Phase I therefore retains the LF as SPACE beside that isolated unit.
    let p = with_wbr(
        capture(vec![0xd840, 0xdc00, 0xa, 0x6f22], WhiteSpace::Collapse),
        WordSpaceTransform::None,
    );
    let processed = process_contextual_source(p).unwrap();
    assert_eq!(
        processed.source().text().code_units(),
        [0xd840, 0xdc00, 0xa, 0x6f22]
    );
    assert_eq!(
        processed.processed_text().code_units(),
        [0xd840, 0xdc00, 0x20, 0x6f22]
    );
    assert_eq!(
        text(processed.plaintext_items(Affinity::After)),
        [0xd840, 0xdc00, 0x20, 0x6f22]
    );

    // Removing only the genuine control restores the intact scalar's Wide context.
    let intact = process_contextual_source(capture(
        vec![0xd840, 0xdc00, 0xa, 0x6f22],
        WhiteSpace::Collapse,
    ))
    .unwrap();
    assert_eq!(
        intact.processed_text().code_units(),
        [0xd840, 0xdc00, 0x6f22]
    );
}

#[test]
fn none_wbr_keeps_two_original_units_through_public_preparation_and_placement() {
    let p = with_wbr(
        capture(vec![0xd840, 0xdc00], WhiteSpace::Preserve),
        WordSpaceTransform::None,
    );
    let processed = Arc::new(process_contextual_source(p).unwrap());
    assert_eq!(processed.processed_text().code_units(), [0xd840, 0xdc00]);
    let view = processed.paragraphs().next().unwrap();
    let analyzed = AnalyzedSource::from_processed_paragraph(
        view.clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let prepared = prepare(&mut system(), view);
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.source_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.processed_range().len())
            .collect::<Vec<_>>(),
        [1, 1]
    );
    // Semantic values remain original opaque code units; glyph repair does not
    // install U+FFFD properties or re-form the supplementary scalar.
    assert_eq!(
        analyzed
            .inner
            .host_projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd840, 0xdc00]
    );
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd840, 0xdc00]
    );
    let finalized = line(&prepared);
    assert_eq!(
        finalized
            .inner
            .selected
            .presentation
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd840, 0xdc00]
    );
    let control = prepared.control_views().next().unwrap().handle();
    let participants: Vec<_> = prepared.participants().collect();
    assert_eq!(participants.len(), 3);
    assert!(matches!(&participants[0], PreparedParticipant::Text(_)));
    assert!(matches!(&participants[1], PreparedParticipant::Control(actual) if actual == &control));
    assert!(matches!(&participants[2], PreparedParticipant::Text(_)));
    let origins: Vec<_> = prepared
        .origin_views()
        .map(|origin| origin.handle())
        .collect();
    let layout = placed(prepared);
    let left = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let right = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    let left_unit = left.units().next().unwrap();
    let right_unit = right.units().next().unwrap();
    assert!(!left_unit.same_unit(&right_unit));
    let positions: Vec<_> = layout.positions().collect();
    let selected = layout
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selected.source_text().code_units(), [0xd840, 0xdc00]);
    assert_eq!(selected.plaintext().code_units(), [0xd840, 0xdc00]);
    assert_eq!(layout.source().text().code_units(), [0xd840, 0xdc00]);
}

#[test]
fn ordinary_origin_inline_and_out_of_flow_seams_keep_an_intact_scalar() {
    for seam in 0..3 {
        let mut p = capture(vec![0xd840, 0xdc00], WhiteSpace::Preserve);
        match seam {
            0 => {}
            1 => {
                p.scopes.push(SourceProcessingScope::Inline {
                    alignment_boundary: None,
                    font_reference: None,
                    trim: WhiteSpaceTrim::default(),
                    style: None,
                    boundaries: None,
                });
                p.events.insert(4, SourceProcessingEvent::EnterScope(1));
                p.events.insert(6, SourceProcessingEvent::ExitScope(1));
            }
            2 => {
                p.source.inline_box(InlineBox::new(
                    Id::from_u64(630011),
                    InlineBoxKind::OutOfFlow,
                    1,
                    Size::new(5.0, 10.0),
                ));
                p.origins = vec![
                    origin(&p.source, Range::new(0, 1)),
                    origin(&p.source, Range::new(1, 2)),
                ];
                p.events.insert(
                    4,
                    SourceProcessingEvent::Atomic {
                        atomic: 0,
                        affinity: Affinity::After,
                    },
                );
            }
            _ => unreachable!(),
        }
        let processed = Arc::new(process_contextual_source(p).unwrap());
        let view = processed.paragraphs().next().unwrap();
        let analyzed = AnalyzedSource::from_processed_paragraph(
            view.clone(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(analyzed.graphemes().collect::<Vec<_>>(), [Range::new(0, 2)]);
        let prepared = prepare(&mut system(), view);
        assert_eq!(
            prepared
                .unit_views()
                .map(|unit| unit.source_range())
                .collect::<Vec<_>>(),
            [Range::new(0, 2)]
        );
        placed(prepared);
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn none_wbr_preserves_exact_ax_refusal_for_isolated_original_units() {
    let p = with_wbr(
        capture(vec![0xd840, 0xdc00], WhiteSpace::Preserve),
        WordSpaceTransform::None,
    );
    let processed = Arc::new(process_contextual_source(p).unwrap());
    let view = processed.paragraphs().next().unwrap();
    let analyzed = AnalyzedSource::from_processed_paragraph(
        view.clone(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut state = Accessibility::new(10);
    let parent = accesskit::NodeId(630012);
    let expected = ErrorDetail::AccessibilityIllFormedSource {
        index: 0,
        unit: 0xd840,
    };
    assert_eq!(
        analyzed
            .accessibility_update(&mut state, parent, parent)
            .unwrap_err()
            .detail(),
        Some(&expected)
    );
    let layout = placed(prepare(&mut system(), view));
    assert_eq!(
        layout
            .accessibility_update(&mut state, parent, parent, Point::new(0.0, 0.0))
            .unwrap_err()
            .detail(),
        Some(&expected)
    );
}
