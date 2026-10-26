use super::*;

fn preserved(source: Source, mode: WhiteSpace, breaks: DocumentSegmentBreaks) -> ProcessedSource {
    process_source_whitespace(
        source,
        Style {
            white_space: mode,
            ..Style::default()
        },
        breaks,
    )
    .unwrap()
}
fn projected(result: &ProcessedSource, affinity: Affinity) -> Vec<usize> {
    (0..=result.source().text().len())
        .map(|at| {
            result
                .source_to_processed(position(result.source(), at), affinity)
                .unwrap()
                .get()
        })
        .collect()
}
fn inverse(result: &ProcessedSource, affinity: Affinity) -> Vec<usize> {
    (0..=result.processed_text().len())
        .map(|at| {
            result
                .processed_to_source(result.position(at).unwrap(), affinity)
                .unwrap()
                .get()
        })
        .collect()
}

#[test]
fn preserve_breaks_collapses_spaces_but_keeps_each_original_delimiter() {
    let source = Source::identified("中 \t\n \n\t 文\t X\rY", Id::from_u64(6401), 2);
    let result = preserved(
        source.clone(),
        WhiteSpace::PreserveBreaks,
        DocumentSegmentBreaks::LineFeedDefault,
    );
    assert_eq!(result.processed_text(), &Text::from("中\n\n文 X Y"));
    assert_eq!(
        plaintext(&result),
        "中\n\n文 X Y".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(result.source(), &source);
    assert!(!result.operations().iter().any(|piece| matches!(
        piece.kind,
        MappingKind::Delete(
            DeletionCause::DuplicateSegmentBreak | DeletionCause::ContextualSegmentBreak
        )
    )));
    let contracted = preserved(
        Source::new("A\r\n\r\nB"),
        WhiteSpace::PreserveBreaks,
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 3), Range::new(3, 5)]),
    );
    assert_eq!(contracted.processed_text().code_units(), [65, 10, 10, 66]);
    assert_eq!(
        projected(&contracted, Affinity::Before),
        [0, 1, 1, 2, 2, 3, 4]
    );
    assert_eq!(
        projected(&contracted, Affinity::After),
        [0, 1, 2, 2, 3, 3, 4]
    );
}

#[test]
fn preserve_breaks_and_collapse_share_space_collapsing_across_mode_spans() {
    let mut source = Source::new("A \t  B");
    source.span(
        Range::new(2, 4),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
    );
    let result = preserved(
        source.clone(),
        WhiteSpace::PreserveBreaks,
        DocumentSegmentBreaks::LineFeedDefault,
    );
    assert_eq!(result.processed_text().code_units(), [65, 32, 66]);
    assert_eq!(result.source(), &source);
    assert_eq!(
        result
            .operations()
            .iter()
            .filter_map(|piece| match piece.kind {
                MappingKind::Delete(DeletionCause::CollapsedSpace) => Some(piece.source),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [Range::new(2, 5)]
    );
}

#[test]
fn new_modes_keep_raw_standalone_admission_explicit() {
    for (mode, feature) in [
        (
            WhiteSpace::PreserveBreaks,
            crate::TextStyleFeature::WhiteSpacePreserveBreaks,
        ),
        (
            WhiteSpace::BreakSpaces,
            crate::TextStyleFeature::WhiteSpaceBreakSpaces,
        ),
    ] {
        assert_eq!(
            feature.support(crate::TextStylePhase::ProcessedParagraph),
            crate::TextStyleSupport::Supported
        );
        assert_eq!(
            feature.support(crate::TextStylePhase::Standalone),
            crate::TextStyleSupport::Unsupported(
                crate::UnsupportedTextStyleReason::RequiresProcessedSourceLayout
            )
        );
        let mut system = crate::System::new(crate::SystemOptions {
            font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        let error = system
            .layout(
                Source::new("A B"),
                Style {
                    white_space: mode,
                    ..Style::default()
                },
                crate::Options::default(),
            )
            .unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::UnsupportedTextStyle {
                feature,
                reason: crate::UnsupportedTextStyleReason::RequiresProcessedSourceLayout
            })
        );
    }
}

#[test]
fn contraction_affinity_and_incident_inverse_do_not_invent_interior_roundtrips() {
    let mut source = Source::identified("A\r\nB", Id::from_u64(72), 51);
    source.span(Range::new(1, 2), Style::default());
    source.span(Range::new(2, 3), Style::default());
    source.span(Range::new(2, 2), Style::default());
    let original = source.clone();
    let result = preserved(
        source,
        WhiteSpace::Preserve,
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
    );
    assert_eq!(result.processed_text().code_units(), [65, 10, 66]);
    assert_eq!(projected(&result, Affinity::Before), [0, 1, 1, 2, 3]);
    assert_eq!(projected(&result, Affinity::After), [0, 1, 2, 2, 3]);
    assert_eq!(inverse(&result, Affinity::Before), [0, 1, 3, 4]);
    assert_eq!(inverse(&result, Affinity::After), [0, 1, 3, 4]);
    let views: Vec<_> = result.spans().collect();
    for (affinity, expected) in [
        (Affinity::Before, [(1, 1), (1, 2), (1, 1)]),
        (Affinity::After, [(1, 2), (2, 2), (2, 2)]),
    ] {
        assert_eq!(
            views
                .iter()
                .map(|view| {
                    let r = view.range(affinity);
                    (r.start().get(), r.end().get())
                })
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert!(std::ptr::eq(
        views[0].original(),
        &result.source().spans()[0]
    ));
    assert_eq!(result.source(), &original);
    assert_eq!(&result.source().text().code_units()[1..3], &[13, 10]);
    drop(original);
    assert_eq!(result.source().id(), Some(Id::from_u64(72)));
    assert_eq!(result.source().revision(), 51);
}

#[test]
fn preserve_spaces_and_document_occurrences_have_literal_distinct_outputs() {
    let units = vec![65, 13, 10, 9, 0xa0, 0xd800, 66];
    let source = Source::identified(Text::from_code_units(units.clone()), Id::from_u64(8), 3);
    let result = preserved(
        source.clone(),
        WhiteSpace::PreserveSpaces,
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
    );
    assert_eq!(
        result.processed_text().code_units(),
        [65, 32, 32, 0xa0, 0xd800, 66]
    );
    assert_eq!(source.text().code_units(), units);
    assert_eq!(result.source(), &source);
    for (mode, breaks, expected) in [
        (
            WhiteSpace::Preserve,
            DocumentSegmentBreaks::LineFeedDefault,
            vec![65, 32, 10, 66],
        ),
        (
            WhiteSpace::Preserve,
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            vec![65, 10, 66],
        ),
        (
            WhiteSpace::Preserve,
            DocumentSegmentBreaks::Defined(vec![]),
            vec![65, 32, 10, 66],
        ),
        (
            WhiteSpace::PreserveSpaces,
            DocumentSegmentBreaks::LineFeedDefault,
            vec![65, 32, 32, 66],
        ),
        (
            WhiteSpace::PreserveSpaces,
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            vec![65, 32, 66],
        ),
        (
            WhiteSpace::PreserveSpaces,
            DocumentSegmentBreaks::Defined(vec![]),
            vec![65, 32, 10, 66],
        ),
    ] {
        let result = preserved(Source::new("A\r\nB"), mode, breaks.clone());
        assert_eq!(result.processed_text().code_units(), expected);
        assert_eq!(result.segment_breaks(), &breaks);
    }
    for (mode, output) in [(WhiteSpace::Preserve, 10), (WhiteSpace::PreserveSpaces, 32)] {
        assert_eq!(
            preserved(
                Source::new("A\u{2028}B"),
                mode,
                DocumentSegmentBreaks::Defined(vec![Range::new(1, 2)])
            )
            .processed_text()
            .code_units(),
            [65, output, 66]
        );
        assert_eq!(
            preserved(
                Source::new("A\u{2028}B"),
                mode,
                DocumentSegmentBreaks::Defined(vec![])
            )
            .processed_text()
            .code_units(),
            [65, 0x2028, 66]
        );
        assert_eq!(
            preserved(
                Source::new("  \t"),
                mode,
                DocumentSegmentBreaks::Defined(vec![])
            )
            .processed_text()
            .code_units(),
            if mode == WhiteSpace::Preserve {
                vec![32, 32, 9]
            } else {
                vec![32, 32, 32]
            }
        );
    }
}

#[test]
fn effective_mode_changes_refuse_one_delimiter_in_source_order_with_precise_precedence() {
    for after in [WhiteSpace::Discard, WhiteSpace::PreserveSpaces] {
        let mut source = Source::new("A\r\nB");
        source.span(
            Range::new(2, 4),
            Style {
                white_space: after,
                ..Style::default()
            },
        );
        let original = source.clone();
        let error = process_source_whitespace(
            source,
            Style::default(),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
        )
        .unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::SegmentBreakCrossesProcessingMode {
                occurrence: 0,
                range: Range::new(1, 3),
                at: 2,
                before: WhiteSpace::Preserve,
                after
            })
        );
        assert_eq!(original.text().code_units(), [65, 13, 10, 66]);
    }
    let mut same = Source::new("A\r\nB");
    same.span(Range::new(2, 3), Style::default());
    same.span(Range::new(2, 2), discard());
    let admitted = preserved(
        same,
        WhiteSpace::Preserve,
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
    );
    assert_eq!(admitted.processed_text().code_units(), [65, 10, 66]);
    assert_eq!(admitted.spans().count(), 2);
    let mut tie = Source::new("A\r\nB");
    tie.span(
        Range::new(2, 4),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
    );
    assert_eq!(
        process_source_whitespace(
            tie.clone(),
            Style::default(),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)])
        )
        .unwrap_err()
        .detail(),
        Some(&ErrorDetail::SegmentBreakCrossesProcessingMode {
            occurrence: 0,
            range: Range::new(1, 3),
            at: 2,
            before: WhiteSpace::Preserve,
            after: WhiteSpace::Collapse
        })
    );
    tie.inline_box(box_(9, InlineBoxKind::InFlow, 2));
    assert!(matches!(
        process_source_whitespace(
            tie,
            Style {
                white_space: WhiteSpace::Collapse,
                ..Style::default()
            },
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)])
        )
        .unwrap_err()
        .detail(),
        Some(ErrorDetail::SegmentBreakCrossesInlineBox { .. })
    ));
    let mut earlier = Source::new("ABCDEF");
    earlier.span(
        Range::new(2, 4),
        Style {
            white_space: WhiteSpace::PreserveSpaces,
            ..Style::default()
        },
    );
    earlier.span(
        Range::new(4, 6),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
    );
    assert!(matches!(
        process_source_whitespace(
            earlier,
            Style::default(),
            DocumentSegmentBreaks::Defined(vec![Range::new(0, 4)])
        )
        .unwrap_err()
        .detail(),
        Some(ErrorDetail::SegmentBreakCrossesProcessingMode { at: 2, .. })
    ));
    for reverse in [false, true] {
        let mut source = Source::new("A \tB");
        for mode in if reverse {
            [WhiteSpace::Discard, WhiteSpace::Preserve]
        } else {
            [WhiteSpace::Preserve, WhiteSpace::Discard]
        } {
            source.span(
                Range::new(1, 3),
                Style {
                    white_space: mode,
                    ..Style::default()
                },
            );
        }
        assert_eq!(
            preserved(
                source,
                WhiteSpace::Preserve,
                DocumentSegmentBreaks::LineFeedDefault
            )
            .processed_text()
            .code_units(),
            if reverse {
                vec![65, 32, 9, 66]
            } else {
                vec![65, 66]
            }
        );
    }
}

#[test]
fn transparent_interior_boxes_expose_both_presentation_choices_without_losing_occurrences() {
    let mut source = Source::new("A\r\nB");
    source.inline_box(box_(9, InlineBoxKind::OutOfFlow, 2));
    source.inline_box(box_(3, InlineBoxKind::OutOfFlow, 2));
    let result = preserved(
        source,
        WhiteSpace::Preserve,
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
    );
    for (affinity, anchor, texts) in [
        (Affinity::Before, 1, vec![vec![65], vec![10, 66]]),
        (Affinity::After, 2, vec![vec![65, 10], vec![66]]),
    ] {
        let mut actual_texts = vec![];
        let mut boxes = vec![];
        let mut at = 0;
        for item in result.plaintext_items(affinity) {
            match item {
                PlaintextItem::Text(text) => {
                    assert_eq!(text.range().start().get(), at);
                    at = text.range().end().get();
                    assert_eq!(
                        text.code_units(),
                        &result.processed_text().code_units()[text.range().start().get()..at]
                    );
                    actual_texts.push(text.code_units().to_vec());
                }
                PlaintextItem::InlineBox(view) => {
                    assert_eq!(at, anchor);
                    assert_eq!(view.position(affinity).get(), anchor);
                    assert_eq!(view.original().index(), 2);
                    assert_eq!(view.original().kind(), InlineBoxKind::OutOfFlow);
                    assert_eq!(view.original().size(), Size::new(3.0, 4.0));
                    assert!(std::ptr::eq(
                        view.original(),
                        &result.source().boxes()[view.ordinal()]
                    ));
                    boxes.push(view.original().id());
                }
            }
        }
        assert_eq!(at, 3);
        assert_eq!(actual_texts, texts);
        assert_eq!(actual_texts.concat(), [65, 10, 66]);
        assert_eq!(boxes, [Id::from_u64(9), Id::from_u64(3)]);
    }
    for anchor in [1, 3] {
        let mut source = Source::new("A\r\nB");
        source.inline_box(box_(1, InlineBoxKind::InFlow, anchor));
        let result = preserved(
            source,
            WhiteSpace::Preserve,
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
        );
        let view = result.inline_boxes().next().unwrap();
        assert_eq!(
            view.position(Affinity::Before),
            view.position(Affinity::After)
        );
    }
    let mut source = Source::new("A\r\nB");
    source.inline_box(box_(1, InlineBoxKind::InFlow, 2));
    assert!(matches!(
        process_source_whitespace(
            source,
            Style::default(),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)])
        )
        .unwrap_err()
        .detail(),
        Some(ErrorDetail::SegmentBreakCrossesInlineBox { .. })
    ));
}

#[test]
fn deletion_extrema_coexist_with_contraction_endpoints_and_adjacent_occurrences() {
    let mut source = Source::new(" A\r\n B");
    source.span(Range::new(0, 1), discard());
    source.span(Range::new(4, 5), discard());
    let result = preserved(
        source,
        WhiteSpace::Preserve,
        DocumentSegmentBreaks::Defined(vec![Range::new(2, 4)]),
    );
    assert_eq!(result.processed_text().code_units(), [65, 10, 66]);
    assert_eq!(projected(&result, Affinity::Before), [0, 0, 1, 1, 2, 2, 3]);
    assert_eq!(projected(&result, Affinity::After), [0, 0, 1, 2, 2, 2, 3]);
    assert_eq!(inverse(&result, Affinity::Before), [0, 2, 4, 6]);
    assert_eq!(inverse(&result, Affinity::After), [1, 2, 5, 6]);
    let breaks = DocumentSegmentBreaks::Defined(vec![Range::new(0, 2), Range::new(2, 4)]);
    let adjacent = preserved(
        Source::new("\r\n\r\n"),
        WhiteSpace::Preserve,
        breaks.clone(),
    );
    assert_eq!(adjacent.processed_text().code_units(), [10, 10]);
    assert_eq!(adjacent.segment_breaks(), &breaks);
    assert_eq!(inverse(&adjacent, Affinity::Before), [0, 2, 4]);
    assert_eq!(inverse(&adjacent, Affinity::After), [0, 2, 4]);
}

#[test]
fn exact_units_and_discard_regressions_hold_under_both_explicit_affinities() {
    let original =
        Text::from_code_units(vec![0xd83d, 0xde00, 0xd800, 0xfffd, 32, 9, 13, 10, 0xdc00]);
    let source = Source::new(original.clone());
    let result = preserved(
        source,
        WhiteSpace::PreserveSpaces,
        DocumentSegmentBreaks::Defined(vec![Range::new(6, 8)]),
    );
    assert_eq!(
        result.processed_text().code_units(),
        [0xd83d, 0xde00, 0xd800, 0xfffd, 32, 32, 32, 0xdc00]
    );
    for affinity in [Affinity::Before, Affinity::After] {
        assert_eq!(
            result
                .source_to_processed(position(result.source(), 1), affinity)
                .unwrap()
                .get(),
            1
        );
        assert_eq!(
            result
                .processed_to_source(result.position(1).unwrap(), affinity)
                .unwrap()
                .get(),
            1
        );
        let discarded = process(
            Source::new(" \tA \r\nB "),
            DocumentSegmentBreaks::LineFeedDefault,
        );
        assert_eq!(projected(&discarded, affinity), [0, 0, 0, 1, 1, 1, 1, 2, 2]);
        assert_eq!(
            discarded
                .plaintext_items(affinity)
                .flat_map(|item| match item {
                    PlaintextItem::Text(text) => text.code_units().iter().copied(),
                    PlaintextItem::InlineBox(_) => [].iter().copied(),
                })
                .collect::<Vec<_>>(),
            [65, 66]
        );
    }
    assert_eq!(result.source().text(), &original);
    let mut system = crate::System::new(crate::SystemOptions {
        font_catalogue: crate::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    let error = system
        .layout(
            Source::new("A B"),
            Style {
                white_space: WhiteSpace::PreserveSpaces,
                ..Style::default()
            },
            crate::Options::default(),
        )
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: crate::TextStyleFeature::WhiteSpacePreserveSpaces,
            reason: crate::UnsupportedTextStyleReason::RequiresProcessedSourceLayout
        })
    );
}
