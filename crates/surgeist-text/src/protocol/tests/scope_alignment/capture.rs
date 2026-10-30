//! Admission starts at the real immutable capture, before fonts or preparation.
use crate::*;
use std::sync::Arc;

pub(super) fn reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    }
}
pub(super) fn generous() -> ScopeAlignmentLimits {
    ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216)
}
pub(super) fn style(size: f32, height: f64) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size,
        line_height: LineHeight::Absolute(height),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

/// Genuine nested scopes and one original contribution. The scope inventory
/// ordinals are producer input, not substitute CapturedScope identities.
pub(super) fn captured_input(inlines: usize) -> ContextualSourceInput {
    let source = Source::identified("A", Id::from_u64(1578001), 23);
    let mut scopes = vec![SourceProcessingScope::Block {
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: style(20.0, 24.0),
        used_direction: ParagraphBaseDirection::LeftToRight,
        font_reference: None,
    }];
    scopes.extend((0..inlines).map(|_| SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: Some(style(10.0, 12.0)),
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
    }));
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    events.extend((1..=inlines).map(SourceProcessingEvent::EnterScope));
    events.push(SourceProcessingEvent::Origin {
        origin: 0,
        affinity: Affinity::After,
    });
    events.extend((1..=inlines).rev().map(SourceProcessingEvent::ExitScope));
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    ContextualSourceInput {
        scope_alignment_limits: None,
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: vec![ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, 1),
            aggregate: Range::new(0, 1),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        computed_line_boundaries: vec![],
        scopes,
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events,
    }
}
fn inline_facts(
    value: &mut ContextualSourceInput,
    index: usize,
    boundary: Option<ScopeAlignmentBoundary>,
    reference: Option<ScopeFontReference>,
) {
    let SourceProcessingScope::Inline {
        alignment_boundary,
        font_reference,
        ..
    } = &mut value.scopes[index]
    else {
        panic!("fixture index is a genuine inline scope")
    };
    *alignment_boundary = boundary;
    *font_reference = reference;
}
fn error_reason(error: &Error) -> ScopeAlignmentError {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("missing exact feature refusal: {error:?}")
    };
    *reason
}
fn rejected_limit(value: ContextualSourceInput, expected: ScopeAlignmentLimitKind) {
    let error = process_contextual_source(value).unwrap_err();
    match error_reason(&error) {
        ScopeAlignmentError::Limit {
            kind,
            required,
            limit,
        } => {
            assert_eq!(kind, expected);
            assert_eq!(limit, 0);
            assert!(
                required > 0,
                "zero must refuse a genuine positive requirement"
            );
        }
        other => panic!("expected resource-specific limit, got {other:?}"),
    }
}

#[test]
fn ordinary_capture_with_no_alignment_facts_needs_no_new_limits_or_default_initial_class() {
    let captured = process_contextual_source(captured_input(3)).unwrap();
    assert_eq!(captured.source().text().code_units(), [65]);
    assert_eq!(captured.processed_text().code_units(), [65]);
    for index in 0..4 {
        let scope = captured.captured_scope(index).unwrap();
        assert_eq!(scope.alignment_boundary(), None);
        assert_eq!(scope.font_reference(), None);
        assert_eq!(scope.scope_alignment_limits(), None);
        assert_eq!(scope.scope_alignment_usage(), None);
    }
    assert!(captured.captured_scope(4).is_none());
}

#[test]
fn each_supplied_early_fact_requires_limits_even_if_it_requests_no_displacement() {
    for (boundary, font) in [
        (Some(ScopeAlignmentBoundary::Initial), None),
        (Some(ScopeAlignmentBoundary::NonInitial), None),
        (None, Some(reference())),
        (Some(ScopeAlignmentBoundary::Initial), Some(reference())),
    ] {
        let mut value = captured_input(1);
        inline_facts(&mut value, 1, boundary, font);
        assert_eq!(
            error_reason(&process_contextual_source(value).unwrap_err()),
            ScopeAlignmentError::LimitsRequired
        );
    }
    let mut domain = captured_input(0);
    let SourceProcessingScope::Block { font_reference, .. } = &mut domain.scopes[0] else {
        unreachable!()
    };
    *font_reference = Some(reference());
    assert_eq!(
        error_reason(&process_contextual_source(domain).unwrap_err()),
        ScopeAlignmentError::LimitsRequired
    );
}

#[test]
fn sparse_fact_rows_do_not_charge_every_ordinary_scope_or_count_two_fields_as_two_rows() {
    let mut value = captured_input(31);
    // Exactly one scope supplies both facts;31 ordinary inline scopes plus the
    // domain are genuine topology, not31 additional feature records.
    inline_facts(
        &mut value,
        17,
        Some(ScopeAlignmentBoundary::NonInitial),
        Some(reference()),
    );
    let limits = ScopeAlignmentLimits::new(1, 0, 100_000, 1_048_576);
    value.scope_alignment_limits = Some(limits);
    let captured = process_contextual_source(value).unwrap();
    let target = captured.captured_scope(17).unwrap();
    assert_eq!(
        target.alignment_boundary(),
        Some(ScopeAlignmentBoundary::NonInitial)
    );
    assert_eq!(target.font_reference(), Some(reference()));
    let usage = target.scope_alignment_usage().unwrap();
    assert_eq!(usage.records(), 1);
    assert_eq!(usage.font_queries(), 0);
    assert!(usage.structural_work() >= 32);
    assert!(usage.structural_work() <= limits.structural_work());
    assert!(usage.payload_bytes() > 0);
    assert!(usage.peak_payload_bytes() >= usage.payload_bytes());
    assert!(usage.peak_payload_bytes() <= limits.payload_bytes());
    for index in 0..32 {
        let scope = captured.captured_scope(index).unwrap();
        assert_eq!(scope.scope_alignment_limits(), Some(limits));
        assert_eq!(scope.scope_alignment_usage(), Some(usage));
        if index != 17 {
            assert_eq!(scope.alignment_boundary(), None);
            assert_eq!(scope.font_reference(), None);
        }
    }
}

#[test]
fn zero_records_and_font_queries_admit_an_explicit_but_fact_free_capture() {
    let mut value = captured_input(1);
    let limits = ScopeAlignmentLimits::new(0, 0, 100, 1_048_576);
    value.scope_alignment_limits = Some(limits);
    let captured = process_contextual_source(value).unwrap();
    let scope = captured.captured_scope(1).unwrap();
    assert_eq!(scope.scope_alignment_limits(), Some(limits));
    let usage = scope.scope_alignment_usage().unwrap();
    assert_eq!((usage.records(), usage.font_queries()), (0, 0));
    // Explicit feature custody still has a retained payload header. No size_of
    // or allocator capacity is copied into this behavioral expectation.
    assert!(usage.payload_bytes() > 0);
    assert!(usage.peak_payload_bytes() >= usage.payload_bytes());
    assert_eq!(scope.alignment_boundary(), None);
}

#[test]
fn meaningful_zero_limits_refuse_only_the_actual_early_resource_being_consumed() {
    let mut records = captured_input(1);
    inline_facts(&mut records, 1, Some(ScopeAlignmentBoundary::Initial), None);
    records.scope_alignment_limits = Some(ScopeAlignmentLimits::new(0, 0, 100, 1_048_576));
    rejected_limit(records, ScopeAlignmentLimitKind::Records);
    let mut work = captured_input(1);
    work.scope_alignment_limits = Some(ScopeAlignmentLimits::new(0, 0, 0, 1_048_576));
    rejected_limit(work, ScopeAlignmentLimitKind::StructuralWork);
    let mut bytes = captured_input(1);
    bytes.scope_alignment_limits = Some(ScopeAlignmentLimits::new(0, 0, 100, 0));
    rejected_limit(bytes, ScopeAlignmentLimitKind::PayloadBytes);
}

#[test]
fn capture_identity_parentage_and_immutable_accounting_survive_readback_and_analysis() {
    let mut value = captured_input(1);
    inline_facts(
        &mut value,
        1,
        Some(ScopeAlignmentBoundary::Initial),
        Some(reference()),
    );
    value.scope_alignment_limits = Some(generous());
    let captured = Arc::new(process_contextual_source(value.clone()).unwrap());
    let parent = captured.captured_scope(0).unwrap();
    let scope = captured.captured_scope(1).unwrap();
    assert_eq!(parent.role(), ScopeRole::Domain);
    assert_eq!(scope.role(), ScopeRole::Inline);
    assert_eq!(scope.parent(), Some(parent.clone()));
    assert_eq!(scope.domain(), parent);
    assert_eq!(scope.style().unwrap().size, 10.0);
    assert_eq!(
        scope.style().unwrap().line_height,
        LineHeight::Absolute(12.0)
    );
    assert_eq!(parent.style().unwrap().size, 20.0);
    assert_eq!(
        parent.style().unwrap().line_height,
        LineHeight::Absolute(24.0)
    );
    let limits = scope.scope_alignment_limits();
    let usage = scope.scope_alignment_usage();
    let cloned = Arc::clone(&captured);
    assert_eq!(cloned.captured_scope(1), Some(scope.clone()));
    let independently_captured = process_contextual_source(value).unwrap();
    let foreign = independently_captured.captured_scope(1).unwrap();
    assert_ne!(
        scope, foreign,
        "equal source ordinals/facts cannot identify a capture"
    );
    assert!(
        !scope
            .enter_position()
            .same_capture(&foreign.enter_position())
    );
    let analysis = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let through_analysis = analysis.processed_source().captured_scope(1).unwrap();
    assert_eq!(through_analysis, scope);
    assert_eq!(through_analysis.scope_alignment_limits(), limits);
    assert_eq!(through_analysis.scope_alignment_usage(), usage);
    assert_eq!(scope.scope_alignment_usage(), usage);
    assert_eq!(analysis.source().text().code_units(), [65]);
}

#[test]
fn domain_font_context_alone_remains_context_without_an_inline_target_or_font_query() {
    let mut value = captured_input(1);
    let reference = reference();
    let SourceProcessingScope::Block {
        font_reference,
        base_style,
        ..
    } = &mut value.scopes[0]
    else {
        unreachable!()
    };
    *font_reference = Some(reference);
    base_style.font = Font::new().family("No registered scope family");
    value.scope_alignment_limits = Some(ScopeAlignmentLimits::new(1, 0, 100, 1_048_576));
    let captured = process_contextual_source(value).unwrap();
    let domain = captured.captured_scope(0).unwrap();
    let inline = captured.captured_scope(1).unwrap();
    assert_eq!(domain.role(), ScopeRole::Domain);
    assert_eq!(domain.font_reference(), Some(reference));
    assert_eq!(domain.alignment_boundary(), None);
    assert_eq!(inline.alignment_boundary(), None);
    assert_eq!(inline.font_reference(), None);
    assert_eq!(domain.scope_alignment_usage().unwrap().font_queries(), 0);
    assert_eq!(domain.scope_alignment_usage().unwrap().records(), 1);
    assert_eq!(captured.processed_text().code_units(), [65]);
}

#[test]
fn failed_new_capture_keeps_the_previously_issued_scope_and_usage_unchanged() {
    let mut value = captured_input(1);
    inline_facts(
        &mut value,
        1,
        Some(ScopeAlignmentBoundary::NonInitial),
        Some(reference()),
    );
    value.scope_alignment_limits = Some(generous());
    let old = process_contextual_source(value.clone()).unwrap();
    let scope = old.captured_scope(1).unwrap();
    let usage = scope.scope_alignment_usage();
    value.scope_alignment_limits = Some(ScopeAlignmentLimits::new(0, 0, 100, 1_048_576));
    rejected_limit(value, ScopeAlignmentLimitKind::Records);
    assert_eq!(old.captured_scope(1), Some(scope.clone()));
    assert_eq!(scope.scope_alignment_usage(), usage);
    assert_eq!(
        scope.alignment_boundary(),
        Some(ScopeAlignmentBoundary::NonInitial)
    );
    assert_eq!(scope.font_reference(), Some(reference()));
    assert_eq!(old.source().text().code_units(), [65]);
}
