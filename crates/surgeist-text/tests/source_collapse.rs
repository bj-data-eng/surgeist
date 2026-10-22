//! Source-local lexical Phase I, before line trimming, shaping or bidi placement.
//! Current CSS Text3 §4.1.1/4.1.3 and Text4 §4.3.1/4.3.3 supply ordering.
//! Contextual removal uses the adopted UA policy: ZWSP, otherwise two F/W/H
//! scalars excluding Hangul, otherwise SPACE. The 2012 draft is its precedent,
//! not a claim that current standards mandate that particular UA choice.
use surgeist_text::{
    Affinity, DocumentSegmentBreakError, DocumentSegmentBreaks, ErrorCode, ErrorDetail, Id,
    InlineBox, InlineBoxKind, PlaintextItem, ProcessedSource, Range, Size, Source, SourcePosition,
    Style, Text, WhiteSpace, process_source_whitespace,
};

fn style(mode: WhiteSpace) -> Style {
    Style {
        white_space: mode,
        ..Style::default()
    }
}
fn source(units: &[u16]) -> Source {
    Source::identified(
        Text::from_code_units(units.to_vec()),
        Id::from_u64(6472),
        19,
    )
}
fn box_at(id: u64, kind: InlineBoxKind, at: usize) -> InlineBox {
    InlineBox::new(Id::from_u64(id), kind, at, Size::new(3.0, 4.0))
}

// Every input reaches the real operation even when an earlier row is refused.
// Errors are recorded, without naming a mode-refusal variant that will disappear.
fn attempt(
    label: &str,
    original: Source,
    mode: WhiteSpace,
    breaks: DocumentSegmentBreaks,
    expected: &[u16],
    refusals: &mut Vec<String>,
) -> Option<ProcessedSource> {
    let retained = original.clone();
    let original_units = retained.text().code_units().to_vec();
    let original_spans = retained.spans().to_vec();
    let original_boxes = retained.boxes().to_vec();
    let base = style(mode);
    let outcome = process_source_whitespace(original, base.clone(), breaks.clone());
    assert_eq!(retained.id(), Some(Id::from_u64(6472)), "{label}");
    assert_eq!(retained.revision(), 19, "{label}");
    assert_eq!(retained.text().code_units(), original_units, "{label}");
    assert_eq!(retained.spans(), original_spans, "{label}");
    assert_eq!(retained.boxes(), original_boxes, "{label}");
    match outcome {
        Err(error) => {
            eprintln!("{label}: {error:?}");
            refusals.push(format!("{label}: {error:?}"));
            None
        }
        Ok(result) => {
            assert_eq!(
                result.source(),
                &retained,
                "{label}: original source custody"
            );
            assert_eq!(result.base_style(), &base, "{label}");
            assert_eq!(result.segment_breaks(), &breaks, "{label}");
            assert_eq!(result.processed_text().code_units(), expected, "{label}");
            for view in result.spans() {
                assert!(std::ptr::eq(
                    view.original(),
                    &result.source().spans()[view.ordinal()]
                ));
                assert_eq!(view.original(), &retained.spans()[view.ordinal()]);
            }
            for view in result.inline_boxes() {
                assert!(std::ptr::eq(
                    view.original(),
                    &result.source().boxes()[view.ordinal()]
                ));
                assert_eq!(view.original(), &retained.boxes()[view.ordinal()]);
            }
            for affinity in [Affinity::Before, Affinity::After] {
                let mut units = Vec::new();
                let mut boxes = Vec::new();
                for item in result.plaintext_items(affinity) {
                    match item {
                        PlaintextItem::Text(text) => {
                            assert_eq!(text.range().start().get(), units.len(), "{label}");
                            units.extend_from_slice(text.code_units());
                            assert_eq!(text.range().end().get(), units.len(), "{label}");
                        }
                        PlaintextItem::InlineBox(view) => {
                            assert_eq!(view.position(affinity).get(), units.len(), "{label}");
                            boxes.push(view.ordinal());
                        }
                    }
                }
                assert_eq!(units, expected, "{label}: same backing under {affinity:?}");
                let mut ordered: Vec<_> = (0..retained.boxes().len()).collect();
                ordered.sort_by_key(|&ordinal| (retained.boxes()[ordinal].index(), ordinal));
                assert_eq!(boxes, ordered, "{label}: each real occurrence once");
            }
            Some(result)
        }
    }
}
fn finish(refusals: Vec<String>) {
    assert!(
        refusals.is_empty(),
        "lexical processing must execute:\n{}",
        refusals.join("\n")
    );
}

#[test]
fn collapse_orders_space_tab_break_and_cr_rules_without_phase_two_trimming() {
    let rows: [(&str, &[u16], DocumentSegmentBreaks, &[u16]); 10] = [
        (
            "first space survives",
            &[65, 32, 9, 32, 66],
            DocumentSegmentBreaks::LineFeedDefault,
            &[65, 32, 66],
        ),
        (
            "tab representative becomes space",
            &[65, 9, 32, 9, 66],
            DocumentSegmentBreaks::LineFeedDefault,
            &[65, 32, 66],
        ),
        (
            "surrounding spaces then duplicate breaks",
            &[65, 10, 32, 9, 10, 66],
            DocumentSegmentBreaks::LineFeedDefault,
            &[65, 32, 66],
        ),
        (
            "leading and trailing representatives remain",
            &[32, 9, 65, 32],
            DocumentSegmentBreaks::LineFeedDefault,
            &[32, 65, 32],
        ),
        (
            "defined CRLF is one occurrence",
            &[65, 32, 9, 13, 10, 32, 32, 66],
            DocumentSegmentBreaks::Defined(vec![Range::new(3, 5)]),
            &[65, 32, 66],
        ),
        (
            "LF default sees CR as surrounding space",
            &[65, 13, 10, 66],
            DocumentSegmentBreaks::LineFeedDefault,
            &[65, 32, 66],
        ),
        (
            "defined CRLF consumes both units",
            &[65, 13, 10, 66],
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            &[65, 32, 66],
        ),
        (
            "explicit empty never falls back to LF",
            &[65, 13, 10, 66],
            DocumentSegmentBreaks::Defined(vec![]),
            &[65, 32, 10, 66],
        ),
        (
            "unclassified line separator stays ordinary",
            &[65, 0x2028, 66],
            DocumentSegmentBreaks::LineFeedDefault,
            &[65, 0x2028, 66],
        ),
        (
            "document line separator transforms",
            &[65, 0x2028, 66],
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 2)]),
            &[65, 32, 66],
        ),
    ];
    let mut refusals = Vec::new();
    for (label, units, breaks, expected) in rows {
        attempt(
            label,
            source(units),
            WhiteSpace::Collapse,
            breaks,
            expected,
            &mut refusals,
        );
    }
    finish(refusals);
}

fn maps(
    result: &ProcessedSource,
    before: &[usize],
    after: &[usize],
    inverse_before: &[usize],
    inverse_after: &[usize],
) {
    for (affinity, forward, inverse) in [
        (Affinity::Before, before, inverse_before),
        (Affinity::After, after, inverse_after),
    ] {
        let actual: Vec<_> = (0..=result.source().text().len())
            .map(|at| {
                result
                    .source_to_processed(
                        SourcePosition::try_new(result.source().text(), at).unwrap(),
                        affinity,
                    )
                    .unwrap()
                    .get()
            })
            .collect();
        assert_eq!(actual, forward);
        assert!(actual.windows(2).all(|pair| pair[0] <= pair[1]));
        let actual: Vec<_> = (0..=result.processed_text().len())
            .map(|at| {
                result
                    .processed_to_source(result.position(at).unwrap(), affinity)
                    .unwrap()
                    .get()
            })
            .collect();
        assert_eq!(actual, inverse);
    }
    assert!(matches!(
        result
            .position(result.processed_text().len() + 1)
            .unwrap_err()
            .detail(),
        Some(ErrorDetail::InvalidProcessedRange { .. })
    ));
    assert!(result.range(1, 0).is_err());
}

#[test]
fn collapsed_followers_and_crlf_keep_literal_endpoint_maps_and_original_records() {
    let mut refusals = Vec::new();
    let mut original = source(&[65, 32, 9, 32, 66]);
    original.span(
        Range::new(1, 2),
        Style {
            size: 17.0,
            ..style(WhiteSpace::Collapse)
        },
    );
    original.span(
        Range::new(2, 4),
        Style {
            size: 23.0,
            ..style(WhiteSpace::Collapse)
        },
    );
    original.span(Range::new(3, 3), style(WhiteSpace::Preserve));
    if let Some(result) = attempt(
        "space map",
        original,
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::LineFeedDefault,
        &[65, 32, 66],
        &mut refusals,
    ) {
        maps(
            &result,
            &[0, 1, 2, 2, 2, 3],
            &[0, 1, 2, 2, 2, 3],
            &[0, 1, 2, 5],
            &[0, 1, 4, 5],
        );
        let spans: Vec<_> = result.spans().collect();
        for affinity in [Affinity::Before, Affinity::After] {
            assert_eq!(
                (
                    spans[0].range(affinity).start().get(),
                    spans[0].range(affinity).end().get()
                ),
                (1, 2)
            );
            assert_eq!(
                (
                    spans[1].range(affinity).start().get(),
                    spans[1].range(affinity).end().get()
                ),
                (2, 2)
            );
            assert!(spans[2].range(affinity).is_empty());
        }
    }
    let mut original = source(&[65, 32, 9, 13, 10, 32, 32, 66]);
    original.inline_box(box_at(1, InlineBoxKind::OutOfFlow, 4));
    original.span(Range::new(4, 4), style(WhiteSpace::Preserve));
    if let Some(result) = attempt(
        "CRLF map and interior anchor",
        original,
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::Defined(vec![Range::new(3, 5)]),
        &[65, 32, 66],
        &mut refusals,
    ) {
        maps(
            &result,
            &[0, 1, 1, 1, 1, 2, 2, 2, 3],
            &[0, 1, 1, 1, 2, 2, 2, 2, 3],
            &[0, 1, 5, 8],
            &[0, 3, 7, 8],
        );
        let view = result.inline_boxes().next().unwrap();
        assert_eq!(view.original().index(), 4);
        assert_eq!(view.position(Affinity::Before).get(), 1);
        assert_eq!(view.position(Affinity::After).get(), 2);
        let empty = result.spans().next().unwrap();
        assert_eq!(empty.range(Affinity::Before).start().get(), 1);
        assert_eq!(empty.range(Affinity::After).start().get(), 2);
        assert!(empty.range(Affinity::Before).is_empty());
        assert!(empty.range(Affinity::After).is_empty());
    }
    finish(refusals);
}

#[test]
fn effective_modes_and_last_span_precedence_do_not_invent_context_boundaries() {
    let mut rows = Vec::new();
    let mut preserved = source(&[65, 32, 32, 66]);
    preserved.span(Range::new(1, 2), style(WhiteSpace::Preserve));
    rows.push((
        "preserved space stops duplicate collapse",
        preserved,
        vec![65, 32, 32, 66],
    ));
    let mut tab = source(&[65, 9, 32, 66]);
    tab.span(Range::new(1, 2), style(WhiteSpace::PreserveSpaces));
    rows.push((
        "preserved converted tab is not collapsible",
        tab,
        vec![65, 32, 32, 66],
    ));
    for (mode, label, unit) in [
        (WhiteSpace::Preserve, "spaces around preserved LF", 10),
        (
            WhiteSpace::PreserveSpaces,
            "spaces around preserved converted break",
            32,
        ),
    ] {
        let mut s = source(&[65, 32, 10, 32, 66]);
        s.span(Range::new(2, 3), style(mode));
        rows.push((label, s, vec![65, unit, 66]));
    }
    let mut duplicate = source(&[65, 10, 32, 9, 10, 66]);
    duplicate.span(Range::new(4, 5), style(WhiteSpace::Preserve));
    rows.push((
        "preserved second break is distinct",
        duplicate,
        vec![65, 32, 10, 66],
    ));
    let mut deleted = source(&[65, 32, 9, 32, 66]);
    deleted.span(Range::new(2, 3), style(WhiteSpace::Discard));
    rows.push((
        "discard deletion exposes collapse adjacency",
        deleted,
        vec![65, 32, 66],
    ));
    let mut precedence = source(&[65, 32, 9, 32, 66]);
    precedence.span(Range::new(1, 4), style(WhiteSpace::Preserve));
    precedence.span(Range::new(1, 4), style(WhiteSpace::Collapse));
    precedence.span(Range::new(2, 2), style(WhiteSpace::Preserve));
    rows.push(("last nonempty span wins", precedence, vec![65, 32, 66]));
    let mut letters = source(&[0x4e2d, 10, 0x6587]);
    letters.span(Range::new(0, 1), style(WhiteSpace::Preserve));
    letters.span(Range::new(2, 3), style(WhiteSpace::Discard));
    rows.push((
        "nonwhitespace style boundaries do not bar context",
        letters,
        vec![0x4e2d, 0x6587],
    ));
    let mut refusals = Vec::new();
    for (label, s, expected) in rows {
        attempt(
            label,
            s,
            WhiteSpace::Collapse,
            DocumentSegmentBreaks::LineFeedDefault,
            &expected,
            &mut refusals,
        );
    }
    finish(refusals);
}

#[test]
fn actual_inflow_boxes_bar_adjacency_while_outof_flow_occurrences_remain_transparent() {
    let mut refusals = Vec::new();
    for (kind, expected_spaces, expected_han) in [
        (
            InlineBoxKind::InFlow,
            &[65, 32, 32, 66][..],
            &[0x4e2d, 32, 0x6587][..],
        ),
        (
            InlineBoxKind::OutOfFlow,
            &[65, 32, 66][..],
            &[0x4e2d, 0x6587][..],
        ),
    ] {
        let mut s = source(&[65, 32, 32, 66]);
        // Deliberately reverse source-anchor order and retain coincident ordinals.
        s.inline_box(box_at(1, kind, 2));
        s.inline_box(box_at(2, InlineBoxKind::OutOfFlow, 0));
        s.inline_box(box_at(3, InlineBoxKind::OutOfFlow, 2));
        attempt(
            &format!("space barrier {kind:?}"),
            s,
            WhiteSpace::Collapse,
            DocumentSegmentBreaks::LineFeedDefault,
            expected_spaces,
            &mut refusals,
        );
        let mut s = source(&[0x4e2d, 10, 0x6587]);
        s.inline_box(box_at(4, kind, 1));
        attempt(
            &format!("context barrier {kind:?}"),
            s,
            WhiteSpace::Collapse,
            DocumentSegmentBreaks::LineFeedDefault,
            expected_han,
            &mut refusals,
        );
    }
    let mut s = source(&[32, 9, 10]);
    s.inline_box(box_at(5, InlineBoxKind::OutOfFlow, 1));
    attempt(
        "all deleted text retains occurrence",
        s,
        WhiteSpace::Discard,
        DocumentSegmentBreaks::LineFeedDefault,
        &[],
        &mut refusals,
    );
    finish(refusals);
}

#[test]
fn contextual_break_policy_uses_real_scalar_properties_and_preserves_other_units() {
    let rows: [(&str, &[u16], &[u16]); 20] = [
        ("Han", &[0x4e2d, 10, 0x6587], &[0x4e2d, 0x6587]),
        ("Kana", &[0x3042, 10, 0x30a2], &[0x3042, 0x30a2]),
        (
            "fullwidth punctuation",
            &[0x4e2d, 0xff0c, 10, 0x6587],
            &[0x4e2d, 0xff0c, 0x6587],
        ),
        ("halfwidth Kana", &[0xff71, 10, 0xff72], &[0xff71, 0xff72]),
        (
            "supplementary Han",
            &[0xd840, 0xdc00, 10, 0xd840, 0xdc01],
            &[0xd840, 0xdc00, 0xd840, 0xdc01],
        ),
        ("ZWSP before", &[65, 0x200b, 10, 66], &[65, 0x200b, 66]),
        ("ZWSP after", &[65, 10, 0x200b, 66], &[65, 0x200b, 66]),
        ("Latin", &[65, 10, 66], &[65, 32, 66]),
        (
            "Hangul excluded despite wide",
            &[0xd55c, 10, 0xae00],
            &[0xd55c, 32, 0xae00],
        ),
        (
            "ambiguous Greek excluded",
            &[0x03b1, 10, 0x03b2],
            &[0x03b1, 32, 0x03b2],
        ),
        (
            "NBSP is real context",
            &[0x4e2d, 0x00a0, 10, 0x6587],
            &[0x4e2d, 0x00a0, 32, 0x6587],
        ),
        (
            "other Zs is real context",
            &[0x4e2d, 0x2003, 10, 0x6587],
            &[0x4e2d, 0x2003, 32, 0x6587],
        ),
        (
            "WJ not skipped",
            &[0x4e2d, 0x2060, 10, 0x6587],
            &[0x4e2d, 0x2060, 32, 0x6587],
        ),
        (
            "ZWJ not skipped",
            &[0x4e2d, 0x200d, 10, 0x6587],
            &[0x4e2d, 0x200d, 32, 0x6587],
        ),
        (
            "isolated high is opaque",
            &[0xd840, 10, 0x6587],
            &[0xd840, 32, 0x6587],
        ),
        (
            "isolated low is opaque",
            &[0x4e2d, 10, 0xdc00],
            &[0x4e2d, 32, 0xdc00],
        ),
        ("absent context", &[10, 0x6587], &[32, 0x6587]),
        (
            "bidi controls retained but ignored for context",
            &[0x4e2d, 0x200f, 10, 0x2066, 0x6587],
            &[0x4e2d, 0x200f, 0x2066, 0x6587],
        ),
        (
            "bidi controls retained through space collapse",
            &[65, 32, 0x200f, 9, 32, 66],
            &[65, 32, 0x200f, 66],
        ),
        (
            "NBSP Zs pair and isolated units remain exact",
            &[0x00a0, 0x2002, 0x202f, 0x3000, 0xd83d, 0xde00, 0xdc00],
            &[0x00a0, 0x2002, 0x202f, 0x3000, 0xd83d, 0xde00, 0xdc00],
        ),
    ];
    let mut refusals = Vec::new();
    for (label, units, expected) in rows {
        attempt(
            label,
            source(units),
            WhiteSpace::Collapse,
            DocumentSegmentBreaks::LineFeedDefault,
            expected,
            &mut refusals,
        );
    }
    finish(refusals);
}

#[test]
fn context_never_fuses_original_surrogates_across_deleted_or_ignored_units() {
    let mut refusals = Vec::new();
    let mut s = source(&[0xd840, 32, 0xdc00, 10, 0xd840, 0xdc01]);
    s.span(Range::new(1, 2), style(WhiteSpace::Discard));
    attempt(
        "discard cannot manufacture U+20000 context",
        s,
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::LineFeedDefault,
        &[0xd840, 0xdc00, 32, 0xd840, 0xdc01],
        &mut refusals,
    );
    attempt(
        "ignored bidi cannot manufacture a scalar",
        source(&[0xd840, 0x200f, 0xdc00, 10, 0xd840, 0xdc01]),
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::LineFeedDefault,
        &[0xd840, 0x200f, 0xdc00, 32, 0xd840, 0xdc01],
        &mut refusals,
    );
    finish(refusals);
}

#[test]
fn noncollapse_modes_and_unused_collapse_base_remain_executable_controls() {
    let mut refusals = Vec::new();
    for (mode, expected) in [
        (WhiteSpace::Discard, &[65, 66][..]),
        (WhiteSpace::Preserve, &[65, 32, 9, 10, 66][..]),
        (WhiteSpace::PreserveSpaces, &[65, 32, 32, 32, 66][..]),
    ] {
        attempt(
            &format!("supported {mode:?}"),
            source(&[65, 32, 9, 13, 10, 66]),
            mode,
            DocumentSegmentBreaks::Defined(vec![Range::new(3, 5)]),
            expected,
            &mut refusals,
        );
    }
    let mut s = source(&[65, 32, 32, 66]);
    s.span(Range::new(0, 4), style(WhiteSpace::Preserve));
    attempt(
        "unused base Collapse",
        s,
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::LineFeedDefault,
        &[65, 32, 32, 66],
        &mut refusals,
    );
    attempt(
        "empty Collapse",
        source(&[]),
        WhiteSpace::Collapse,
        DocumentSegmentBreaks::Defined(vec![]),
        &[],
        &mut refusals,
    );
    finish(refusals);
}

#[test]
fn intrinsic_delimiter_and_inflow_validation_keep_precedence_without_partial_mutation() {
    let malformed = DocumentSegmentBreaks::Defined(vec![Range::new(1, 1)]);
    let mut invalid = source(&[65, 13, 10, 66]);
    invalid.span(Range::new(0, 5), style(WhiteSpace::Preserve));
    let mut anchor = source(&[65, 13, 10, 66]);
    anchor.inline_box(box_at(1, InlineBoxKind::InFlow, 5));
    let mut duplicate = source(&[65, 13, 10, 66]);
    duplicate.inline_box(box_at(1, InlineBoxKind::InFlow, 0));
    duplicate.inline_box(box_at(1, InlineBoxKind::OutOfFlow, 4));
    for (label, s, code) in [
        ("span before delimiters", invalid, ErrorCode::InvalidRange),
        ("anchor before delimiters", anchor, ErrorCode::InvalidRange),
        (
            "duplicate ID before delimiters",
            duplicate,
            ErrorCode::InvalidStyle,
        ),
    ] {
        let retained = s.clone();
        let error =
            process_source_whitespace(s.clone(), style(WhiteSpace::Collapse), malformed.clone())
                .unwrap_err();
        assert_eq!(error.code, code, "{label}");
        assert_eq!(s, retained);
    }
    let mut crossing = source(&[65, 13, 10, 66]);
    crossing.inline_box(box_at(2, InlineBoxKind::InFlow, 2));
    let error = process_source_whitespace(crossing.clone(), style(WhiteSpace::Collapse), malformed)
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidDocumentSegmentBreak {
            occurrence: 0,
            range: Range::new(1, 1),
            reason: DocumentSegmentBreakError::Empty
        })
    );
    let error = process_source_whitespace(
        crossing.clone(),
        style(WhiteSpace::Collapse),
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
    )
    .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::SegmentBreakCrossesInlineBox {
            occurrence: 0,
            box_ordinal: 0,
            id: Id::from_u64(2),
            anchor: 2
        })
    );
    assert_eq!(crossing.boxes()[0], box_at(2, InlineBoxKind::InFlow, 2));
    let mut mixed = source(&[65, 13, 10, 66]);
    mixed.span(Range::new(2, 3), style(WhiteSpace::PreserveSpaces));
    let error = process_source_whitespace(
        mixed.clone(),
        style(WhiteSpace::Preserve),
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
            after: WhiteSpace::PreserveSpaces
        })
    );
}

#[test]
fn collapse_does_not_split_a_defined_delimiter_at_an_effective_mode_change() {
    let mut mismatches = Vec::new();
    for (before, after) in [
        (WhiteSpace::Collapse, WhiteSpace::Preserve),
        (WhiteSpace::Preserve, WhiteSpace::Collapse),
    ] {
        let mut s = source(&[65, 13, 10, 66]);
        s.span(Range::new(2, 3), style(after));
        let retained = s.clone();
        let outcome = process_source_whitespace(
            s,
            style(before),
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
        );
        match outcome {
            Err(error) => {
                let expected = ErrorDetail::SegmentBreakCrossesProcessingMode {
                    occurrence: 0,
                    range: Range::new(1, 3),
                    at: 2,
                    before,
                    after,
                };
                if error.detail() != Some(&expected) {
                    mismatches.push(format!("{before:?} -> {after:?}: {error:?}"));
                }
            }
            Ok(_) => mismatches.push(format!("{before:?} -> {after:?}: delimiter was admitted")),
        }
        assert_eq!(retained.text().code_units(), &[65, 13, 10, 66]);
        assert_eq!(retained.spans()[0].range(), Range::new(2, 3));
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
