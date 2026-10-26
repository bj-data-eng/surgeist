//! Genuine connection event ranks distinguish generated text at one numeric seam.
use super::*;

fn wbr(at: usize, operation: WordSpaceTransform) -> ParagraphControl {
    ParagraphControl {
        at,
        kind: ParagraphControlKind::Wbr {
            style: Box::new(Style {
                white_space: WhiteSpace::Preserve,
                word_space_transform: operation,
                ..Style::default()
            }),
        },
    }
}
fn split(text: &str, at: usize, before: bool) -> ContextualSourceInput {
    let mut p = input(text, WhiteSpace::Preserve);
    p.origins = [
        Range::new(0, at),
        Range::new(at, text.encode_utf16().count()),
    ]
    .into_iter()
    .map(|r| origin(&p.source, r))
    .collect();
    p.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Disconnected,
    }];
    p.boundary_policies = vec![policy(ParagraphBoundaryDefinition::Connection(0))];
    p.controls = vec![wbr(at, WordSpaceTransform::Space)];
    let connection = SourceProcessingEvent::Connection(0);
    let control = SourceProcessingEvent::Control {
        control: 0,
        affinity: Affinity::After,
    };
    p.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    ];
    p.events.extend(if before {
        [control, connection]
    } else {
        [connection, control]
    });
    p.events.extend([
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    p
}
fn analyze(p: ContextualSourceInput) -> AnalyzedSource {
    let p = Arc::new(process_contextual_source(p).unwrap());
    AnalyzedSource::from_processed_paragraph(p.paragraphs().next().unwrap()).unwrap()
}
#[test]
fn actual_connection_order_determines_generated_space_and_mark_grapheme() {
    for (before, cut, graphemes) in [(true, 2, vec![0, 1, 2, 3]), (false, 1, vec![0, 1, 3])] {
        let a = analyze(split("A\u{3099}", 1, before));
        assert_eq!(
            a.inner.paragraph.connection_anchor_cuts.as_deref(),
            Some(&[usize::from(before)][..])
        );
        assert_eq!(a.inner.projection.text, "A \u{3099}");
        assert_eq!(a.inner.projection.barriers, [1]);
        assert_eq!(a.inner.projection.barrier_atoms, [cut]);
        assert_eq!(a.inner.projection.graphemes, graphemes);
        assert_eq!(
            a.processed_source().processed_text().code_units(),
            [0x41, 0x3099]
        );
    }
}
#[test]
fn only_same_connected_side_proves_forced_suppression() {
    for (before, expected) in [(true, "A\nB"), (false, "A\n B")] {
        let a = analyze(split("A\nB", 2, before));
        assert_eq!(a.inner.projection.text, expected);
        assert_eq!(a.source().text().code_units(), [0x41, 0xa, 0x42]);
    }
}
fn direct(operation: WordSpaceTransform, text: &str, at: usize) -> ParagraphInput {
    let p = split(text, at, false);
    ParagraphInput {
        source: p.source,
        base_style: style(WhiteSpace::Preserve, 10.0),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: p.origins,
        connections: vec![ParagraphConnection {
            at,
            kind: ParagraphConnectionKind::Disconnected,
        }],
        controls: vec![wbr(at, operation)],
        replacements: vec![],
        boundary_policies: p.boundary_policies,
        anchor_order: vec![ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        }],
    }
}
#[test]
fn direct_positive_missing_side_refuses_even_with_a_nearest_lf_but_none_does_not() {
    for (text, at) in [("A\u{3099}", 1), ("A\nB", 2)] {
        let none =
            AnalyzedSource::from_paragraph(direct(WordSpaceTransform::None, text, at)).unwrap();
        assert_eq!(
            none.source().text().code_units(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        for operation in [
            WordSpaceTransform::Space,
            WordSpaceTransform::IdeographicSpace,
        ] {
            let error = AnalyzedSource::from_paragraph(direct(operation, text, at)).unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::TextTransform {
                    reason: TextTransformError::MissingSeparatorBoundaryContext {
                        control: 0,
                        at,
                        connection: 0
                    }
                })
            );
        }
    }
    let mut p = direct(WordSpaceTransform::Space, "ABCD", 2);
    p.controls[0].at = 3;
    assert_eq!(
        AnalyzedSource::from_paragraph(p)
            .unwrap()
            .inner
            .projection
            .text,
        "ABC D"
    );
}
#[test]
fn parent_holes_and_empty_occurrences_retain_each_actual_connection_receipt() {
    let mut p = consecutive_child_holes(true, false);
    for c in &mut p.connections {
        c.kind = ParagraphConnectionKind::Disconnected;
    }
    p.controls.push(wbr(2, WordSpaceTransform::Space));
    let at = p
        .events
        .iter()
        .position(|event| matches!(event, SourceProcessingEvent::Atomic { atomic: 1, .. }))
        .unwrap();
    p.events.insert(
        at,
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::After,
        },
    );
    let a = analyze(p);
    assert_eq!(
        a.inner.paragraph.connection_anchor_cuts.as_deref(),
        Some(&[1, 4][..])
    );
    assert_eq!(
        a.inner.paragraph.connection_ends.as_deref(),
        Some(&[(1, 2), (2, 3)][..])
    );
    assert_eq!(a.inner.projection.text, "A B");
    assert_eq!(a.inner.projection.barrier_atoms, [1, 2]);
    let pieces: Vec<_> = (0..a.inner.transform.rows.len())
        .flat_map(|r| {
            a.inner
                .transform
                .original_pieces(r)
                .iter()
                .map(|p| p.aggregate)
        })
        .collect();
    assert_eq!(pieces, [Range::new(0, 1), Range::new(3, 4)]);
}
