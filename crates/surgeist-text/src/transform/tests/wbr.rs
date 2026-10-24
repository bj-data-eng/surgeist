//! Literal semantic expectations for genuine, source-less wbr occurrences.
use crate::*;

fn style(operation: WordSpaceTransform, mode: WhiteSpace) -> Style {
    Style {
        word_space_transform: operation,
        white_space: mode,
        ..Style::default()
    }
}
fn wbr(at: usize, operation: WordSpaceTransform) -> ParagraphControl {
    ParagraphControl {
        at,
        kind: ParagraphControlKind::Wbr {
            style: Box::new(style(operation, WhiteSpace::Preserve)),
        },
    }
}
fn input(text: &str, mode: WhiteSpace, controls: Vec<ParagraphControl>) -> ParagraphInput {
    let source = Source::identified(text, Id::from_u64(6307), 11);
    let len = source.text().len();
    let mut anchor_order: Vec<_> = (0..controls.len())
        .map(|i| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(i),
            affinity: Affinity::After,
        })
        .collect();
    if len == 0 {
        anchor_order.insert(
            0,
            ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::After,
            },
        );
    }
    ParagraphInput {
        source: source.clone(),
        base_style: style(WordSpaceTransform::None, mode),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls,
        replacements: vec![],
        boundary_policies: vec![],
        anchor_order,
    }
}
fn plaintext(a: &AnalyzedSource) -> Vec<u16> {
    a.processed_source()
        .plaintext_items(Affinity::After)
        .flat_map(|item| match item {
            PlaintextItem::Text(t) => t.code_units().to_vec(),
            PlaintextItem::InlineBox(_) => vec![],
        })
        .collect()
}

#[test]
fn genuine_wbr_capability_requires_the_processed_source_executor() {
    assert!(TextStyleFeature::ALL.contains(&TextStyleFeature::NativeWbr));
    assert_eq!(
        TextStyleFeature::NativeWbr.support(TextStylePhase::ProcessedParagraph),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::NativeWbr.support(TextStylePhase::Standalone),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresProcessedSourceLayout)
    );
}

#[test]
fn wbr_inserts_at_the_scalar_cut_inside_an_original_grapheme() {
    for (operation, expected, graphemes) in [
        (WordSpaceTransform::None, "A\u{3099}", vec![0, 2]),
        (WordSpaceTransform::Space, "A \u{3099}", vec![0, 1, 3]),
        (
            WordSpaceTransform::IdeographicSpace,
            "A\u{3000}\u{3099}",
            vec![0, 1, 3],
        ),
    ] {
        let value = input("A\u{3099}", WhiteSpace::Preserve, vec![wbr(1, operation)]);
        let source = value.source.clone();
        let a = AnalyzedSource::from_paragraph(value).unwrap();
        assert_eq!(a.source(), &source);
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            [0x41, 0x3099]
        );
        assert_eq!(plaintext(&a), [0x41, 0x3099]);
        assert_eq!(a.inner.projection.text, expected);
        assert_eq!(a.inner.projection.graphemes, graphemes);
    }
    let a = AnalyzedSource::from_paragraph(input(
        "A\u{3099}",
        WhiteSpace::Preserve,
        vec![
            wbr(1, WordSpaceTransform::Space),
            wbr(1, WordSpaceTransform::IdeographicSpace),
        ],
    ))
    .unwrap();
    assert_eq!(a.inner.projection.text, "A \u{3000}\u{3099}");
    assert_eq!(a.inner.projection.graphemes, [0, 1, 2, 4]);
}

#[test]
fn wbr_generation_uses_its_own_style_and_does_not_replay_phase_i() {
    for mode in [
        WhiteSpace::Collapse,
        WhiteSpace::Discard,
        WhiteSpace::Preserve,
        WhiteSpace::PreserveSpaces,
    ] {
        let mut value = input("AA", mode, vec![wbr(1, WordSpaceTransform::Space)]);
        if let ParagraphControlKind::Wbr { style } = &mut value.controls[0].kind {
            style.white_space = mode;
            style.text_transform =
                TextTransform::Set(TextTransformSet::none().with_full_width(true));
        }
        let a = AnalyzedSource::from_paragraph(value).unwrap();
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            [0x41, 0x41]
        );
        assert_eq!(a.inner.projection.text, "A\u{3000}A", "{mode:?}");
    }
    let mut value = input(
        "AA",
        WhiteSpace::Preserve,
        vec![wbr(1, WordSpaceTransform::None)],
    );
    value.base_style.word_space_transform = WordSpaceTransform::IdeographicSpace;
    assert_eq!(
        AnalyzedSource::from_paragraph(value)
            .unwrap()
            .inner
            .projection
            .text,
        "AA"
    );
}

#[test]
fn wbr_inside_deleted_source_inserts_without_resurrecting_authored_spaces() {
    for (operation, expected) in [
        (WordSpaceTransform::None, "AA"),
        (WordSpaceTransform::Space, "A A"),
        (WordSpaceTransform::IdeographicSpace, "A\u{3000}A"),
    ] {
        let a = AnalyzedSource::from_paragraph(input(
            "A  A",
            WhiteSpace::Discard,
            vec![wbr(2, operation)],
        ))
        .unwrap();
        assert_eq!(a.source().text().code_units(), [0x41, 0x20, 0x20, 0x41]);
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            [0x41, 0x41]
        );
        assert_eq!(plaintext(&a), [0x41, 0x41]);
        assert_eq!(a.inner.projection.text, expected);
    }
}

#[test]
fn positive_wbr_refuses_interior_supplementary_and_actual_phase_i_replacement_seams() {
    for (text, at, retained, breaks) in [
        (
            "A\u{1f600}A",
            2,
            Range::new(1, 3),
            DocumentSegmentBreaks::LineFeedDefault,
        ),
        (
            "A\r\nA",
            2,
            Range::new(1, 3),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
        ),
    ] {
        let mut none = input(
            text,
            WhiteSpace::PreserveSpaces,
            vec![wbr(at, WordSpaceTransform::None)],
        );
        none.segment_breaks = breaks.clone();
        let original = none.source.clone();
        let baseline = AnalyzedSource::from_paragraph(none).unwrap();
        assert_eq!(baseline.source(), &original);
        assert_eq!(
            baseline
                .inner
                .projection
                .atoms
                .iter()
                .filter(|a| a.source == retained)
                .count(),
            1
        );
        for operation in [
            WordSpaceTransform::Space,
            WordSpaceTransform::IdeographicSpace,
        ] {
            let mut positive = input(text, WhiteSpace::PreserveSpaces, vec![wbr(at, operation)]);
            positive.segment_breaks = breaks.clone();
            let error = AnalyzedSource::from_paragraph(positive).unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::TextTransform {
                    reason: TextTransformError::UnrepresentableSeparatorSeam {
                        control: 0,
                        at,
                        retained
                    }
                })
            );
        }
        assert_eq!(
            original.text().code_units(),
            text.encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn retained_hard_replacement_suppresses_an_otherwise_unrepresentable_positive_wbr() {
    for operation in [
        WordSpaceTransform::Space,
        WordSpaceTransform::IdeographicSpace,
    ] {
        let mut value = input("A\r\nA", WhiteSpace::Preserve, vec![wbr(2, operation)]);
        value.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]);
        let a = AnalyzedSource::from_paragraph(value).unwrap();
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            [0x41, 0xa, 0x41]
        );
        assert_eq!(a.inner.projection.text, "A\nA");
        assert_eq!(
            a.inner
                .projection
                .atoms
                .iter()
                .filter(|a| a.source == Range::new(1, 3) && a.code_point == 0xa)
                .count(),
            1
        );
    }
}

#[test]
fn actual_forced_control_suppression_crosses_neutral_controls_but_not_an_optional_break() {
    for (between, expected) in [
        (vec![], "AA"),
        (
            vec![
                ParagraphControlKind::InlineBoundaryStart,
                ParagraphControlKind::InlineBoundaryEnd,
            ],
            "AA",
        ),
        (
            vec![ParagraphControlKind::OptionalBreak { wrap: Wrap::Word }],
            "A A",
        ),
    ] {
        let mut controls = vec![ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::ForcedBreak,
        }];
        controls.extend(
            between
                .into_iter()
                .map(|kind| ParagraphControl { at: 1, kind }),
        );
        controls.push(wbr(1, WordSpaceTransform::Space));
        let a =
            AnalyzedSource::from_paragraph(input("AA", WhiteSpace::Preserve, controls)).unwrap();
        assert_eq!(a.inner.projection.text, expected);
        assert_eq!(a.source().text().code_units(), [0x41, 0x41]);
    }
    // An LF transformed to SPACE is not a surviving hard separator.
    for (mode, expected) in [
        (WhiteSpace::Preserve, "A\nA"),
        (WhiteSpace::PreserveSpaces, "A  A"),
    ] {
        let a = AnalyzedSource::from_paragraph(input(
            "A\nA",
            mode,
            vec![wbr(1, WordSpaceTransform::Space)],
        ))
        .unwrap();
        assert_eq!(a.inner.projection.text, expected);
    }
}

#[test]
fn contextual_event_order_retains_the_same_genuine_wbr_cut() {
    let source = Source::identified("AA", Id::from_u64(6308), 12);
    let processed = std::sync::Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: [Range::new(0, 1), Range::new(1, 2)]
                .into_iter()
                .map(|r| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: r,
                    aggregate: r,
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: ParagraphConnectionKind::Continuous,
            }],
            controls: vec![wbr(1, WordSpaceTransform::Space)],
            boundary_policies: vec![ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }],
            replacements: vec![],
            scopes: vec![SourceProcessingScope::Block {
                trim: WhiteSpaceTrim::default(),
                base_style: style(WordSpaceTransform::None, WhiteSpace::Preserve),
                used_direction: ParagraphBaseDirection::LeftToRight,
            }],
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
                SourceProcessingEvent::Control {
                    control: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.processed_text().code_units(), [0x41, 0x41]);
    let a =
        AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap()).unwrap();
    assert_eq!(a.source(), &source);
    assert_eq!(a.inner.projection.text, "A A");
    assert_eq!(plaintext(&a), [0x41, 0x41]);
}
