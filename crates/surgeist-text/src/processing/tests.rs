use super::*;
use crate::{Id, Size};

fn discard() -> Style {
    Style {
        white_space: WhiteSpace::Discard,
        ..Style::default()
    }
}

#[test]
fn exact_coverage_plaintext_reuses_executed_mapping_without_filling_holes() {
    let processed = process_source_whitespace(
        Source::new("A  B C"),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    assert_eq!(
        processed.processed_text().code_units(),
        "A B C".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        processed
            .plaintext_for_coverage(&[Range::new(0, 1), Range::new(3, 4)])
            .unwrap()
            .code_units(),
        [0x41, 0x42]
    );
    // Two selected original pieces from the same executed collapse contribute
    // its one real processed SPACE once, rather than inventing another pass.
    assert_eq!(
        processed
            .plaintext_for_coverage(&[Range::new(1, 2), Range::new(2, 3)])
            .unwrap()
            .code_units(),
        [0x20]
    );
    assert!(processed.plaintext_for_coverage(&[]).unwrap().is_empty());
    let processed = process_source_whitespace(
        Source::new(Text::from_code_units(vec![0x41, 0xd800, 0x42])),
        Style {
            white_space: WhiteSpace::Preserve,
            ..Style::default()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    assert_eq!(
        processed
            .plaintext_for_coverage(&[Range::new(1, 2)])
            .unwrap()
            .code_units(),
        [0xd800]
    );
}
fn process(source: Source, breaks: DocumentSegmentBreaks) -> ProcessedSource {
    process_source_whitespace(source, discard(), breaks).unwrap()
}
fn position(source: &Source, at: usize) -> SourcePosition {
    SourcePosition::try_new(source.text(), at).unwrap()
}
fn forward(result: &ProcessedSource) -> Vec<usize> {
    (0..=result.source().text().len())
        .map(|at| {
            let at = position(result.source(), at);
            let before = result
                .source_to_processed(at, Affinity::Before)
                .unwrap()
                .get();
            assert_eq!(
                before,
                result
                    .source_to_processed(at, Affinity::After)
                    .unwrap()
                    .get()
            );
            before
        })
        .collect()
}
fn plaintext(result: &ProcessedSource) -> Vec<u16> {
    result
        .plaintext_items(Affinity::Before)
        .flat_map(|item| match item {
            PlaintextItem::Text(text) => text.code_units().iter().copied(),
            PlaintextItem::InlineBox(_) => [].iter().copied(),
        })
        .collect()
}
fn box_(id: u64, kind: InlineBoxKind, at: usize) -> InlineBox {
    InlineBox::new(Id::from_u64(id), kind, at, Size::new(3.0, 4.0))
}

#[test]
fn exact_units_mapping_and_original_custody_survive_real_partial_plaintext_consumption() {
    let units = vec![
        0x20, 0x41, 0x09, 0xd800, 0x0d, 0x0a, 0xa0, 0x2003, 0x20, 0x42, 0xdc00,
    ];
    let original = Source::identified(Text::from_code_units(units.clone()), Id::from_u64(19), 37);
    let retained = original.clone();
    let result = process(original, DocumentSegmentBreaks::LineFeedDefault);
    assert_eq!(
        result.processed_text().code_units(),
        [0x41, 0xd800, 0xa0, 0x2003, 0x42, 0xdc00]
    );
    assert_eq!(forward(&result), [0, 0, 1, 1, 2, 2, 2, 3, 4, 4, 5, 6]);
    assert_eq!(result.source(), &retained);
    assert_eq!(result.source().id(), Some(Id::from_u64(19)));
    assert_eq!(result.source().revision(), 37);
    assert_eq!(retained.text().code_units(), units);
    drop(retained);
    assert_eq!(
        plaintext(&result),
        [0x41, 0xd800, 0xa0, 0x2003, 0x42, 0xdc00]
    );
    assert_eq!(&result.source().text().code_units()[4..6], &[0x0d, 0x0a]);
}

#[test]
fn affinity_uses_complete_preimages_including_empty_deleted_and_pair_interiors() {
    let result = process(Source::new("A \tB"), DocumentSegmentBreaks::LineFeedDefault);
    assert_eq!(forward(&result), [0, 1, 1, 1, 2]);
    assert_eq!(
        result
            .processed_to_source(result.position(1).unwrap(), Affinity::Before)
            .unwrap()
            .get(),
        1
    );
    assert_eq!(
        result
            .processed_to_source(result.position(1).unwrap(), Affinity::After)
            .unwrap()
            .get(),
        3
    );
    let all = process(
        Source::new(" \t\r\n"),
        DocumentSegmentBreaks::LineFeedDefault,
    );
    assert_eq!(forward(&all), [0, 0, 0, 0, 0]);
    assert_eq!(
        all.processed_to_source(all.position(0).unwrap(), Affinity::Before)
            .unwrap()
            .get(),
        0
    );
    assert_eq!(
        all.processed_to_source(all.position(0).unwrap(), Affinity::After)
            .unwrap()
            .get(),
        4
    );
    let empty = process_source_whitespace(
        Source::new(""),
        Style::default(),
        DocumentSegmentBreaks::Defined(vec![]),
    )
    .unwrap();
    assert_eq!(forward(&empty), [0]);
    for affinity in [Affinity::Before, Affinity::After] {
        assert_eq!(
            empty
                .processed_to_source(empty.position(0).unwrap(), affinity)
                .unwrap()
                .get(),
            0
        );
    }
    let pair = process(
        Source::new(Text::from_code_units(vec![
            0x20, 0xd83d, 0xde00, 0x20, 0xfffd, 0xd800,
        ])),
        DocumentSegmentBreaks::LineFeedDefault,
    );
    assert_eq!(forward(&pair), [0, 0, 1, 2, 2, 3, 4]);
    assert_eq!(
        pair.processed_text().code_units(),
        [0xd83d, 0xde00, 0xfffd, 0xd800]
    );
    assert_eq!(
        pair.processed_to_source(pair.position(1).unwrap(), Affinity::Before)
            .unwrap()
            .get(),
        2
    );
    assert_eq!(pair.range(1, 1).unwrap().len(), 0);
}

#[test]
fn last_span_policy_and_original_borrowed_records_are_exact_without_font_admission() {
    let mut source = Source::new("A \tB");
    source.span(Range::new(0, 4), discard());
    source.span(Range::new(1, 3), discard());
    source.span(Range::new(2, 2), Style::default());
    let base = Style {
        font: crate::Font::new().family("[invalid-unused"),
        size: f32::NAN,
        ..Style::default()
    };
    let result =
        process_source_whitespace(source.clone(), base, DocumentSegmentBreaks::LineFeedDefault)
            .unwrap();
    assert!(result.base_style().size.is_nan());
    assert_eq!(result.base_style().font.family, ["[invalid-unused"]);
    let spans: Vec<_> = result.spans().collect();
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[1].range(Affinity::Before).start().get(), 1);
    assert!(spans[1].range(Affinity::Before).is_empty());
    assert_eq!(spans[1].original().range(), Range::new(1, 3));
    assert!(std::ptr::eq(
        spans[1].original(),
        &result.source().spans()[1]
    ));
    assert_eq!(
        spans[2].original().style().white_space,
        WhiteSpace::Preserve
    );
    assert_eq!(result.source(), &source);
    let mut collapsed = source.clone();
    collapsed.span(
        Range::new(2, 4),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
    );
    let retained = collapsed.clone();
    let admitted =
        process_source_whitespace(collapsed, discard(), DocumentSegmentBreaks::LineFeedDefault)
            .unwrap();
    assert_eq!(admitted.source(), &retained);
    assert_eq!(admitted.processed_text().code_units(), [0x41, 0x20, 0x42]);
    let admitted = process_source_whitespace(
        Source::new(" \t"),
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    assert_eq!(admitted.processed_text().code_units(), [0x20]);
    let mut reversed = Source::new("AB");
    reversed.span(Range::new(0, 2), Style::default());
    reversed.span(Range::new(0, 2), discard());
    assert_eq!(
        process_source_whitespace(
            reversed,
            Style::default(),
            DocumentSegmentBreaks::LineFeedDefault
        )
        .unwrap()
        .processed_text()
        .code_units(),
        [65, 66]
    );
}

#[test]
fn document_classification_distinguishes_defined_empty_lf_crlf_and_adjacent_occurrences() {
    for (text, breaks, expected) in [
        (
            "A\u{2028}B",
            DocumentSegmentBreaks::LineFeedDefault,
            vec![65, 0x2028, 66],
        ),
        (
            "A\u{2028}B",
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 2)]),
            vec![65, 66],
        ),
        (
            "A\nB",
            DocumentSegmentBreaks::Defined(vec![]),
            vec![65, 10, 66],
        ),
        ("A\nB", DocumentSegmentBreaks::LineFeedDefault, vec![65, 66]),
        (
            "A\r\nB",
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            vec![65, 66],
        ),
        (
            "A\r\nB",
            DocumentSegmentBreaks::Defined(vec![]),
            vec![65, 10, 66],
        ),
    ] {
        assert_eq!(
            process(Source::new(text), breaks)
                .processed_text()
                .code_units(),
            expected
        );
    }
    let adjacent = DocumentSegmentBreaks::Defined(vec![Range::new(0, 1), Range::new(1, 2)]);
    let result = process(Source::new("AB"), adjacent.clone());
    assert_eq!(result.segment_breaks(), &adjacent);
    assert!(result.processed_text().is_empty());
    for (ranges, occurrence, reason) in [
        (vec![Range::new(1, 1)], 0, DocumentSegmentBreakError::Empty),
        (
            vec![Range::new(2, 1)],
            0,
            DocumentSegmentBreakError::Reversed,
        ),
        (
            vec![Range::new(0, 5)],
            0,
            DocumentSegmentBreakError::OutOfBounds,
        ),
        (
            vec![Range::new(0, 3), Range::new(2, 4)],
            1,
            DocumentSegmentBreakError::Overlap,
        ),
        (
            vec![Range::new(2, 3), Range::new(0, 1)],
            1,
            DocumentSegmentBreakError::Unordered,
        ),
    ] {
        let range = ranges[occurrence];
        assert_eq!(
            process_source_whitespace(
                Source::new("ABCD"),
                discard(),
                DocumentSegmentBreaks::Defined(ranges)
            )
            .unwrap_err()
            .detail(),
            Some(&ErrorDetail::InvalidDocumentSegmentBreak {
                occurrence,
                range,
                reason
            })
        );
    }
}

#[test]
fn actual_atomic_box_crossing_refuses_but_out_of_flow_is_transparent() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let mut source = Source::new("A\r\nB");
        source.inline_box(box_(7, kind, 2));
        let result = process_source_whitespace(
            source,
            discard(),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
        );
        if kind == InlineBoxKind::InFlow {
            assert_eq!(
                result.unwrap_err().detail(),
                Some(&ErrorDetail::SegmentBreakCrossesInlineBox {
                    occurrence: 0,
                    box_ordinal: 0,
                    id: Id::from_u64(7),
                    anchor: 2
                })
            );
        } else {
            let result = result.unwrap();
            let view = result.inline_boxes().next().unwrap();
            assert_eq!(view.position(Affinity::Before).get(), 1);
            assert_eq!(view.original().index(), 2);
            assert_eq!(view.original().kind(), InlineBoxKind::OutOfFlow);
            assert_eq!(plaintext(&result), [65, 66]);
        }
    }
    for anchor in [1, 3] {
        let mut source = Source::new("A\r\nB");
        source.inline_box(box_(1, InlineBoxKind::InFlow, anchor));
        assert_eq!(
            process(
                source,
                DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)])
            )
            .processed_text()
            .code_units(),
            [65, 66]
        );
    }
    let mut source = Source::new("ABCD");
    source.inline_box(box_(9, InlineBoxKind::InFlow, 2));
    source.inline_box(box_(3, InlineBoxKind::InFlow, 2));
    assert_eq!(
        process_source_whitespace(
            source,
            discard(),
            DocumentSegmentBreaks::Defined(vec![Range::new(0, 4)])
        )
        .unwrap_err()
        .detail(),
        Some(&ErrorDetail::SegmentBreakCrossesInlineBox {
            occurrence: 0,
            box_ordinal: 0,
            id: Id::from_u64(9),
            anchor: 2
        })
    );
}

#[test]
fn plaintext_orders_real_boxes_by_original_anchor_then_vector_order_without_invented_text() {
    let mut source = Source::new("A \tB");
    source.inline_box(box_(8, InlineBoxKind::InFlow, 3));
    source.inline_box(box_(9, InlineBoxKind::OutOfFlow, 2));
    source.inline_box(box_(3, InlineBoxKind::InFlow, 2));
    let result = process(source, DocumentSegmentBreaks::LineFeedDefault);
    assert_eq!(
        result
            .inline_boxes()
            .map(|view| (
                view.ordinal(),
                view.original().id(),
                view.original().index(),
                view.position(Affinity::Before).get()
            ))
            .collect::<Vec<_>>(),
        [
            (0, Id::from_u64(8), 3, 1),
            (1, Id::from_u64(9), 2, 1),
            (2, Id::from_u64(3), 2, 1)
        ]
    );
    let mut order = vec![];
    for item in result.plaintext_items(Affinity::Before) {
        match item {
            PlaintextItem::Text(text) => order.push((text.code_units().to_vec(), None)),
            PlaintextItem::InlineBox(view) => {
                assert!(std::ptr::eq(
                    view.original(),
                    &result.source().boxes()[view.ordinal()]
                ));
                assert_eq!(view.original().size(), Size::new(3.0, 4.0));
                order.push((vec![], Some(view.original().id())));
            }
        }
    }
    assert_eq!(
        order,
        [
            (vec![65], None),
            (vec![], Some(Id::from_u64(9))),
            (vec![], Some(Id::from_u64(3))),
            (vec![], Some(Id::from_u64(8))),
            (vec![66], None)
        ]
    );
    assert_eq!(plaintext(&result), [65, 66]);
    for text in ["", " \t"] {
        let mut source = Source::new(text);
        source.inline_box(box_(1, InlineBoxKind::InFlow, 0));
        let result = process(source, DocumentSegmentBreaks::LineFeedDefault);
        assert_eq!(result.plaintext_items(Affinity::Before).count(), 1);
        assert!(matches!(
            result.plaintext_items(Affinity::Before).next(),
            Some(PlaintextItem::InlineBox(_))
        ));
    }
    assert_eq!(
        process(
            Source::new("\u{200b}\u{a0}\u{2028}"),
            DocumentSegmentBreaks::LineFeedDefault
        )
        .processed_text()
        .code_units(),
        [0x200b, 0xa0, 0x2028]
    );
}

#[test]
fn intrinsic_validation_and_result_relative_bounds_remain_precise_before_modes() {
    let mut invalid = Source::new("A");
    invalid.span(Range::new(0, 2), discard());
    assert!(matches!(
        process_source_whitespace(
            invalid,
            Style::default(),
            DocumentSegmentBreaks::LineFeedDefault
        )
        .unwrap_err()
        .detail(),
        Some(ErrorDetail::InvalidSourceRange { .. })
    ));
    for source in [
        {
            let mut s = Source::new("A");
            s.inline_box(box_(1, InlineBoxKind::InFlow, 2));
            s
        },
        {
            let mut s = Source::new("A");
            s.inline_box(box_(1, InlineBoxKind::InFlow, 0));
            s.inline_box(box_(1, InlineBoxKind::OutOfFlow, 1));
            s
        },
        {
            let mut s = Source::new("A");
            s.inline_box(InlineBox::new(
                Id::from_u64(1),
                InlineBoxKind::InFlow,
                0,
                Size::new(f64::NAN, 2.0),
            ));
            s
        },
    ] {
        assert!(
            process_source_whitespace(
                source,
                Style::default(),
                DocumentSegmentBreaks::LineFeedDefault
            )
            .is_err()
        );
    }
    let result = process(Source::new("A B"), DocumentSegmentBreaks::LineFeedDefault);
    assert_eq!(
        result.range(2, 1).unwrap_err().detail(),
        Some(&ErrorDetail::InvalidProcessedRange {
            start: 2,
            end: 1,
            text_len: 2
        })
    );
    let foreign = process(Source::new("ABCDE"), DocumentSegmentBreaks::LineFeedDefault);
    assert!(
        result
            .processed_to_source(foreign.position(5).unwrap(), Affinity::Before)
            .is_err()
    );
    assert!(
        result
            .source_to_processed(position(foreign.source(), 5), Affinity::Before)
            .is_err()
    );
    let mut system = crate::System::default();
    let error = system
        .layout(Source::new("A B"), discard(), crate::Options::default())
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: crate::TextStyleFeature::WhiteSpaceDiscard,
            reason: crate::UnsupportedTextStyleReason::RequiresProcessedSourceLayout
        })
    );
}

mod preservation;

mod collapse;
