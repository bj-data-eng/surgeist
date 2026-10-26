//! Genuine inline edges retain capture order around a scalar-interior wbr.
use crate::*;
fn capture(operation: WordSpaceTransform) -> ParagraphInput {
    let source = Source::new("A\u{3099}");
    ParagraphInput {
        source: source.clone(),
        base_style: Style::default(),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, 2),
            aggregate: Range::new(0, 2),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        replacements: vec![],
        boundary_policies: vec![],
        controls: vec![
            ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::Wbr {
                    style: Box::new(Style {
                        word_space_transform: operation,
                        ..Style::default()
                    }),
                },
            },
            ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        anchor_order: (0..3)
            .map(|i| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(i),
                affinity: Affinity::After,
            })
            .collect(),
    }
}
#[test]
fn none_wbr_with_real_neutral_inline_edges_keeps_valid_capture_order() {
    let a = AnalyzedSource::from_paragraph(capture(WordSpaceTransform::None)).unwrap();
    assert_eq!(a.inner.projection.text, "A\u{3099}");
    assert_eq!(a.source().text().code_units(), [0x41, 0x3099]);
}
#[test]
fn positive_wbr_with_real_neutral_inline_edges_keeps_valid_capture_order() {
    let a = AnalyzedSource::from_paragraph(capture(WordSpaceTransform::Space)).unwrap();
    assert_eq!(a.inner.projection.text, "A \u{3099}");
    assert_eq!(a.inner.projection.graphemes, [0, 1, 3]);
    assert_eq!(a.source().text().code_units(), [0x41, 0x3099]);
}

fn ordered(
    text: &str,
    start: usize,
    wbr: usize,
    reverse: bool,
    operation: WordSpaceTransform,
) -> ParagraphInput {
    let mut p = capture(operation);
    let source = Source::new(text);
    let end = source.text().len();
    p.source = source.clone();
    p.origins[0].source = source;
    p.origins[0].local = Range::new(0, end);
    p.origins[0].aggregate = Range::new(0, end);
    p.controls[0].at = start;
    p.controls[1].at = wbr;
    p.controls[2].at = end;
    if reverse {
        p.controls.swap(0, 1);
        p.anchor_order[0].affinity = Affinity::Before;
        p.anchor_order[1].affinity = Affinity::Before;
    }
    p
}
#[test]
fn different_source_cuts_in_one_old_egc_use_rebuilt_typography_without_reordering_events() {
    for reverse in [false, true] {
        for operation in [WordSpaceTransform::None, WordSpaceTransform::Space] {
            let mut p = ordered(
                "A\u{3099}\u{3099}",
                if reverse { 2 } else { 1 },
                if reverse { 1 } else { 2 },
                reverse,
                operation,
            );
            // Actual later pipeline changes must preserve the projected cuts too.
            p.base_style.text_transform =
                TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Lowercase));
            for control in &mut p.controls {
                if let ParagraphControlKind::Wbr { style } = &mut control.kind {
                    style.text_transform =
                        TextTransform::Set(TextTransformSet::none().with_full_width(true));
                }
            }
            let a = AnalyzedSource::from_paragraph(p).unwrap();
            let expected = if operation == WordSpaceTransform::None {
                "a\u{3099}\u{3099}"
            } else if reverse {
                "a\u{3000}\u{3099}\u{3099}"
            } else {
                "a\u{3099}\u{3000}\u{3099}"
            };
            assert_eq!(a.inner.projection.text, expected);
            let cuts: Vec<_> = a
                .inner
                .projection
                .anchor_atoms
                .iter()
                .copied()
                .flatten()
                .collect();
            assert_eq!(
                cuts,
                if operation == WordSpaceTransform::None {
                    if reverse {
                        vec![0, 0, 3]
                    } else {
                        vec![3, 3, 3]
                    }
                } else if reverse {
                    vec![1, 1, 4]
                } else {
                    vec![2, 2, 4]
                }
            );
            let actual: Vec<_> = a
                .inner
                .transform
                .events
                .iter()
                .filter_map(|e| match e {
                    crate::transform::relation::RelationEvent::Anchor { order } => Some(*order),
                    _ => None,
                })
                .collect();
            assert_eq!(
                actual,
                if operation == WordSpaceTransform::None {
                    vec![0, 1, 2]
                } else if reverse {
                    vec![1, 2]
                } else {
                    vec![0, 2]
                }
            );
        }
    }
}
#[test]
fn no_emission_retains_phase_i_replacement_projection_but_positive_interior_seams_refuse() {
    for (text, mode, breaks, expected, retained) in [
        (
            "A\r\nA",
            WhiteSpace::PreserveSpaces,
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            "A A",
            Range::new(1, 3),
        ),
        (
            "A\r\nA",
            WhiteSpace::Preserve,
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            "A\nA",
            Range::new(1, 3),
        ),
    ] {
        for operation in [WordSpaceTransform::None, WordSpaceTransform::Space] {
            let mut p = ordered(text, 2, 2, false, operation);
            p.base_style.white_space = mode;
            p.segment_breaks = breaks.clone();
            let a = AnalyzedSource::from_paragraph(p);
            if operation == WordSpaceTransform::Space && mode == WhiteSpace::PreserveSpaces {
                let error = a.unwrap_err();
                assert_eq!(error.code, ErrorCode::UnsupportedFeature);
                assert_eq!(
                    error.detail(),
                    Some(&ErrorDetail::TextTransform {
                        reason: TextTransformError::UnrepresentableSeparatorSeam {
                            control: 1,
                            at: 2,
                            retained
                        }
                    })
                );
            } else {
                let a = a.unwrap();
                assert_eq!(a.inner.projection.text, expected);
                assert_eq!(a.inner.projection.anchor_atoms, [Some(2), Some(2), Some(3)]);
                assert_eq!(
                    a.source().text().code_units(),
                    text.encode_utf16().collect::<Vec<_>>()
                );
                assert_eq!(
                    a.inner
                        .projection
                        .atoms
                        .iter()
                        .filter(|atom| atom.source == retained)
                        .count(),
                    1
                );
            }
        }
    }
}

#[test]
fn crossing_requested_typographic_affinities_refuse_with_typed_order_error() {
    let mut p = ordered("A\u{3099}\u{3099}", 2, 1, true, WordSpaceTransform::None);
    // This is intentionally different from the valid Before/Before dual case:
    // original Wbr After projects to3, then the later Start Before projects0.
    p.anchor_order[0].affinity = Affinity::After;
    let error = AnalyzedSource::from_paragraph(p).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: crate::ProtocolError::InvalidAnchorOrder {
                position: 1,
                kind: crate::ProtocolHandleKind::Control,
                occurrence: 1
            }
        })
    ));
}
