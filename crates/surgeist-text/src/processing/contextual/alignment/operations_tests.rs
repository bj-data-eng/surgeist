//! Bounded helper qualification using actual immutable captured scope keys.
//! The fresh helper ledger excludes the prior capture and caller scratch. These
//! tests assert semantic results/admission, not heap steps, allocation or a trace.
use super::{ScopeAlignmentBudget, operations};
use crate::*;
use std::cmp::Ordering;
use std::sync::Arc;

fn capture(count: usize) -> Arc<ProcessedSource> {
    use SourceProcessingEvent as E;
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    };
    let reference = ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    };
    assert!(count >= 2);
    let units = count - 1;
    let raw = "X".repeat(units);
    let mut source = Source::identified(raw.as_str(), Id::from_u64(1578078), 67);
    source.span(Range::new(0, units), style.clone());
    let mut scopes = vec![SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: style.clone(),
        used_direction: ParagraphBaseDirection::LeftToRight,
        font_reference: Some(reference),
    }];
    scopes.extend((0..count).map(|_| SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        trim: WhiteSpaceTrim::default(),
        style: Some(style.clone()),
        boundaries: None,
        alignment_boundary: Some(ScopeAlignmentBoundary::Initial),
        font_reference: Some(reference),
    }));
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    // Every target owns original content or a genuine descendant. Entry order
    // descends despite the ascending inventory. One nested pair retains a real
    // ancestor relationship without turning the threshold case into deep nesting.
    events.extend([
        E::EnterScope(count),
        E::EnterScope(count - 1),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::ExitScope(count - 1),
        E::ExitScope(count),
    ]);
    for (origin, ordinal) in (1..=count - 2).rev().enumerate() {
        events.extend([
            E::Connection(origin),
            E::EnterScope(ordinal),
            E::Origin {
                origin: origin + 1,
                affinity: Affinity::After,
            },
            E::ExitScope(ordinal),
        ]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    Arc::new(
        process_contextual_source(ContextualSourceInput {
            joint_flows: Vec::new(),
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            scope_alignment_limits: Some(ScopeAlignmentLimits::new(
                100_000, 10_000, 20_000_000, 67_108_864,
            )),
            origins: (0..units)
                .map(|index| ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(index, index + 1),
                    aggregate: Range::new(index, index + 1),
                    kind: ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..units - 1)
                .map(|index| SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            boundary_policies: (0..units - 1)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            scopes,
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    )
}
fn targets(capture: &ProcessedSource, count: usize) -> Vec<CapturedScope> {
    (1..=count)
        .map(|ordinal| {
            let scope = capture.captured_scope(ordinal).unwrap();
            assert_eq!(
                scope.alignment_boundary(),
                Some(ScopeAlignmentBoundary::Initial)
            );
            scope
        })
        .collect()
}
fn compare_entry(left: &CapturedScope, right: &CapturedScope) -> Ordering {
    left.enter_position()
        .partial_cmp(&right.enter_position())
        .expect("same actual capture")
}
fn budget(work: usize) -> ScopeAlignmentBudget {
    ScopeAlignmentBudget::new(ScopeAlignmentLimits::new(0, 0, work, 0), 0, 0).unwrap()
}
fn usage(ledger: &ScopeAlignmentBudget) -> usize {
    let usage = ledger.usage();
    assert_eq!(usage.records(), 0);
    assert_eq!(usage.font_queries(), 0);
    assert_eq!(usage.payload_bytes(), 0);
    assert_eq!(usage.peak_payload_bytes(), 0);
    usage.structural_work()
}
fn reason(error: Error) -> ScopeAlignmentError {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected precise helper budget refusal, got {error:?}")
    };
    *reason
}
fn search_refusal(error: Error, ledger: &ScopeAlignmentBudget, limit: usize) {
    assert_eq!(usage(ledger), limit);
    assert_eq!(
        reason(error),
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::StructuralWork,
            required: limit + 1,
            limit,
        }
    );
}
fn unchanged(capture: &ProcessedSource, before: Option<ScopeAlignmentUsage>, units: usize) {
    assert_eq!(capture.source().id(), Some(Id::from_u64(1578078)));
    assert_eq!(capture.source().revision(), 67);
    assert_eq!(capture.source().text().code_units(), vec![88; units]);
    assert_eq!(
        capture.captured_scope(0).unwrap().scope_alignment_usage(),
        before
    );
}

#[test]
fn more_than_512_authentic_targets_with_nesting_order_by_source_entry_not_input_ordinal() {
    const COUNT: usize = 600;
    let capture = capture(COUNT);
    let original = targets(&capture, COUNT);
    let before = capture.captured_scope(0).unwrap().scope_alignment_usage();
    assert_eq!(
        original[COUNT - 2].parent(),
        Some(original[COUNT - 1].clone())
    );
    assert_eq!(original[COUNT - 1].parent(), capture.captured_scope(0));
    let expected = (1..=COUNT).rev().collect::<Vec<_>>();
    for limit in [0, 1] {
        let mut scratch = original.clone();
        let mut ledger = budget(limit);
        let error =
            operations::order_by(&mut scratch, Some(&mut ledger), compare_entry).unwrap_err();
        let admitted = usage(&ledger);
        let ScopeAlignmentError::Limit {
            kind,
            required,
            limit: actual,
        } = reason(error)
        else {
            panic!("ordering must refuse the actual structural-work limit")
        };
        assert_eq!(kind, ScopeAlignmentLimitKind::StructuralWork);
        assert_eq!(actual, limit);
        assert!(admitted <= limit);
        assert!(required > limit);
        // Comparisons cost one, an admitted two-element swap costs two. Do not
        // prescribe the heap schedule or whether refused scratch was permuted.
        assert!((1..=2).contains(&(required - admitted)));
        let identities = scratch
            .iter()
            .map(CapturedScope::capture_ordinal)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            identities,
            (1..=COUNT).collect::<std::collections::BTreeSet<_>>()
        );
        unchanged(&capture, before, COUNT - 1);
    }
    let mut scratch = original.clone();
    let mut ledger = budget(200_000);
    operations::order_by(&mut scratch, Some(&mut ledger), compare_entry).unwrap();
    assert_eq!(
        scratch
            .iter()
            .map(CapturedScope::capture_ordinal)
            .collect::<Vec<_>>(),
        expected
    );
    assert!(usage(&ledger) > 0 && usage(&ledger) <= 200_000);
    let mut unmetered = original.clone();
    operations::order_by(&mut unmetered, None, compare_entry).unwrap();
    assert_eq!(unmetered, scratch);
    assert_eq!(
        original
            .iter()
            .map(CapturedScope::capture_ordinal)
            .collect::<Vec<_>>(),
        (1..=COUNT).collect::<Vec<_>>()
    );
    unchanged(&capture, before, COUNT - 1);
}

#[test]
fn sparse_authentic_scope_searches_preserve_found_and_insertion_semantics_under_bounds() {
    let capture = capture(12);
    let before = capture.captured_scope(0).unwrap().scope_alignment_usage();
    let sparse = [2, 5, 9].map(|ordinal| capture.captured_scope(ordinal).unwrap());
    let original = sparse.clone();
    let interior = capture.captured_scope(4).unwrap();
    for limit in [0, 1] {
        let mut ledger = budget(limit);
        let error = operations::binary_search_by(&sparse, Some(&mut ledger), |scope| {
            scope.capture_ordinal().cmp(&interior.capture_ordinal())
        })
        .unwrap_err();
        search_refusal(error, &ledger, limit);
    }
    for (ordinal, expected) in [(5, Ok(1)), (4, Err(1)), (0, Err(0)), (12, Err(3))] {
        let key = capture.captured_scope(ordinal).unwrap();
        let mut ledger = budget(16);
        let found = operations::binary_search_by(&sparse, Some(&mut ledger), |scope| {
            scope.capture_ordinal().cmp(&key.capture_ordinal())
        })
        .unwrap();
        assert_eq!(found, expected);
        assert!(usage(&ledger) > 0 && usage(&ledger) <= 16);
        assert_eq!(
            operations::binary_search_by(&sparse, None, |scope| {
                scope.capture_ordinal().cmp(&key.capture_ordinal())
            })
            .unwrap(),
            expected
        );
    }
    assert_eq!(sparse, original);
    unchanged(&capture, before, 11);
}

#[test]
fn authentic_scope_partition_boundaries_are_fallible_and_keep_borrowed_metadata_unchanged() {
    let capture = capture(12);
    let before = capture.captured_scope(0).unwrap().scope_alignment_usage();
    let sparse = [2, 5, 9].map(|ordinal| capture.captured_scope(ordinal).unwrap());
    let original = sparse.clone();
    let interior = capture.captured_scope(4).unwrap();
    for limit in [0, 1] {
        let mut ledger = budget(limit);
        let error = operations::partition_point(&sparse, Some(&mut ledger), |scope| {
            scope.capture_ordinal() < interior.capture_ordinal()
        })
        .unwrap_err();
        search_refusal(error, &ledger, limit);
    }
    for (ordinal, expected) in [(0, 0), (2, 0), (5, 1), (12, 3)] {
        let key = capture.captured_scope(ordinal).unwrap();
        let mut ledger = budget(16);
        assert_eq!(
            operations::partition_point(&sparse, Some(&mut ledger), |scope| {
                scope.capture_ordinal() < key.capture_ordinal()
            })
            .unwrap(),
            expected
        );
        assert!(usage(&ledger) > 0 && usage(&ledger) <= 16);
        assert_eq!(
            operations::partition_point(&sparse, None, |scope| {
                scope.capture_ordinal() < key.capture_ordinal()
            })
            .unwrap(),
            expected
        );
    }
    let key = capture.captured_scope(5).unwrap();
    let mut ledger = budget(16);
    assert_eq!(
        operations::partition_point(&sparse, Some(&mut ledger), |scope| {
            scope.capture_ordinal() <= key.capture_ordinal()
        })
        .unwrap(),
        2
    );
    assert_eq!(sparse, original);
    unchanged(&capture, before, 11);
}
