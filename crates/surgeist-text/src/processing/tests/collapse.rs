use super::*;

fn collapse(source: Source) -> ProcessedSource {
    process_source_whitespace(
        source,
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap()
}

fn deletions(result: &ProcessedSource) -> Vec<(Range, DeletionCause)> {
    result
        .map
        .iter()
        .filter_map(|piece| match piece.kind {
            MappingKind::Delete(cause) => Some((piece.source, cause)),
            _ => None,
        })
        .collect()
}

#[test]
fn executed_deletion_reasons_remain_distinct_at_the_same_processed_anchor() {
    let mut source = Source::new("A \t B");
    source.span(Range::new(2, 3), discard());
    let result = collapse(source);
    assert_eq!(result.processed_text().code_units(), [65, 32, 66]);
    assert_eq!(
        deletions(&result),
        [
            (Range::new(2, 3), DeletionCause::Discard),
            (Range::new(3, 4), DeletionCause::CollapsedSpace),
        ]
    );
    for (affinity, expected) in [(Affinity::Before, 2), (Affinity::After, 4)] {
        assert_eq!(
            result
                .processed_to_source(result.position(2).unwrap(), affinity)
                .unwrap()
                .get(),
            expected
        );
    }

    let result = collapse(Source::new("A \n \nB"));
    assert_eq!(result.processed_text().code_units(), [65, 32, 66]);
    assert_eq!(
        deletions(&result),
        [
            (Range::new(1, 2), DeletionCause::SpaceAroundSegmentBreak),
            (Range::new(3, 4), DeletionCause::SpaceAroundSegmentBreak),
            (Range::new(4, 5), DeletionCause::DuplicateSegmentBreak),
        ]
    );
    let result = collapse(Source::new("中\n文"));
    assert_eq!(result.processed_text().code_units(), [0x4e2d, 0x6587]);
    assert_eq!(
        deletions(&result),
        [(Range::new(1, 2), DeletionCause::ContextualSegmentBreak)]
    );
}

#[test]
fn actual_original_pair_context_is_split_only_by_real_inflow_or_document_occurrences() {
    for (kind, expected) in [
        (
            InlineBoxKind::InFlow,
            vec![0xd840, 0xdc00, 32, 0xd840, 0xdc01],
        ),
        (
            InlineBoxKind::OutOfFlow,
            vec![0xd840, 0xdc00, 0xd840, 0xdc01],
        ),
    ] {
        let mut source = Source::new(Text::from_code_units(vec![
            0xd840, 0xdc00, 10, 0xd840, 0xdc01,
        ]));
        source.inline_box(box_(81, kind, 1));
        let result = collapse(source.clone());
        assert_eq!(result.processed_text().code_units(), expected);
        assert_eq!(result.source(), &source);
        assert_eq!(
            result.inline_boxes().next().unwrap().original(),
            &source.boxes()[0]
        );
    }
    // An original document occurrence inside a pair is real, not a decode error
    // to repair or a permission to combine neighboring retained isolated units.
    let source = Source::new(Text::from_code_units(vec![
        0xd840, 0xdc00, 10, 0xd840, 0xdc01,
    ]));
    let result = process_source_whitespace(
        source,
        Style {
            white_space: WhiteSpace::Collapse,
            ..Style::default()
        },
        DocumentSegmentBreaks::Defined(vec![Range::new(1, 2), Range::new(2, 3)]),
    )
    .unwrap();
    assert_eq!(
        result.processed_text().code_units(),
        [0xd840, 32, 0xd840, 0xdc01]
    );
}

#[test]
fn deleted_text_never_erases_actual_barriers_or_relabels_preserved_context() {
    for (kind, expected) in [
        (InlineBoxKind::InFlow, vec![65, 32, 32, 66]),
        (InlineBoxKind::OutOfFlow, vec![65, 32, 66]),
    ] {
        let mut source = Source::new("A \t B");
        source.span(Range::new(2, 3), discard());
        source.inline_box(box_(92, kind, 2));
        let result = collapse(source);
        assert_eq!(result.processed_text().code_units(), expected);
        assert_eq!(plaintext(&result), expected);
        assert_eq!(result.inline_boxes().count(), 1);
    }
    let mut source = Source::new("中 \n文");
    source.span(Range::new(1, 2), Style::default());
    let result = collapse(source);
    assert_eq!(
        result.processed_text().code_units(),
        [0x4e2d, 32, 32, 0x6587]
    );
}
