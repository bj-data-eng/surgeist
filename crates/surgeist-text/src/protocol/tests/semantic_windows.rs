//! Public semantic-window requirements, independent of fitting geometry.
use super::*;
use crate::{Range, SemanticAtomicAllocation, SemanticControlAllocation, SemanticWindowRequest};

fn input(text: &str, white_space: WhiteSpace) -> ParagraphInput {
    let source = Source::new(text);
    let length = text.encode_utf16().count();
    ParagraphInput {
        source: source.clone(),
        base_style: Style {
            white_space,
            ..style()
        },
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        paragraph_strut: ParagraphStrut::DefaultFont,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if length == 0 {
            vec![paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::After,
            }]
        } else {
            vec![]
        },
    }
}
fn capture(system: &mut System, input: ParagraphInput) -> PreparedText {
    system
        .prepare(
            &crate::test_wrapping::bind_missing(
                &AnalyzedSource::from_paragraph(
                    input,
                    crate::AnalysisTypography::HorizontalComposition,
                )
                .unwrap(),
                crate::Wrap::Word,
            ),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn window(p: &PreparedText) -> SemanticWindowRequest {
    let mut result = SemanticWindowRequest::new(p.source_segments().next().unwrap());
    for participant in p.participants() {
        match participant {
            PreparedParticipant::Atomic(occurrence) => {
                result.atomics.push(SemanticAtomicAllocation {
                    kind: occurrence.view().original_box().kind(),
                    occurrence,
                })
            }
            PreparedParticipant::Control(occurrence) => {
                let endpoint = matches!(
                    occurrence.view().kind(),
                    crate::ParagraphControlKind::InlineBoundaryStart
                        | crate::ParagraphControlKind::InlineBoundaryEnd
                );
                result.controls.push(SemanticControlAllocation {
                    occurrence,
                    boundary: endpoint
                        .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                });
            }
            _ => {}
        }
    }
    result
}
fn window_reason(failure: &crate::SemanticWindowFailure) -> &ProtocolError {
    reason(failure.error())
}

fn bounded(text: &str) -> PreparedText {
    use crate::SourceProcessingEvent as E;
    let mut value = input(text, WhiteSpace::Preserve);
    let length = text.encode_utf16().count();
    value.controls = vec![
        paragraph::ParagraphControl {
            at: 0,
            kind: crate::ParagraphControlKind::InlineBoundaryStart,
        },
        paragraph::ParagraphControl {
            at: length,
            kind: crate::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let captured = std::sync::Arc::new(
        crate::process_contextual_source(crate::ContextualSourceInput {
            source: value.source,
            segment_breaks: value.segment_breaks,
            envelope: crate::SourceProcessingEnvelope::Complete,
            origins: value.origins,
            controls: value.controls,
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(crate::ScopeAlignmentLimits::new(
                100_000, 4096, 10_000_000, 67_108_864,
            )),
            scopes: vec![crate::SourceProcessingScope::Block {
                topology: crate::SourceProcessingTopology::Complete,
                trim: crate::WhiteSpaceTrim::default(),
                base_style: value.base_style,
                used_direction: ParagraphBaseDirection::LeftToRight,
                writing_axis: Some(crate::SourceWritingAxis::Horizontal),
                horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
                font_reference: None,
            }],
            paragraphs: vec![crate::SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Control {
                    control: 0,
                    affinity: crate::Affinity::After,
                },
                E::Origin {
                    origin: 0,
                    affinity: crate::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: crate::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    system()
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

#[test]
fn transaction_limits_count_discarded_branches_and_preserve_input_on_failure() {
    // A long discarded overlay is larger than the fixed three-part proof.
    let text = format!("{}\nY\nZ", "X".repeat(256));
    let p = bounded(&text);
    let r = window(&p);
    let proof = p.semantic_window(&r).unwrap();
    assert_eq!(proof.partitions().len(), 3);
    let usage = proof.scope_alignment_usage().unwrap();
    let input = proof.input_retention();
    let live = proof.scope_alignment_retention();
    assert_eq!(usage.font_queries(), 0);
    assert!(
        usage.records() > input.records() + live.records(),
        "discarded overlay/census rows stay charged"
    );
    assert_eq!(
        usage.payload_bytes(),
        input.payload_bytes() + live.payload_bytes()
    );
    assert!(usage.peak_payload_bytes() > usage.payload_bytes());
    // All earlier partitions/materializations fit; the final charged record
    // cannot be funded by pretending discarded scratch never existed.
    let limit = usage.records() - 1;
    let failed = p
        .semantic_window_with_scope_allowance(&r, ScopeAlignmentAllowance::new(limit, 67_108_864))
        .unwrap_err();
    assert!(
        matches!(failed.error().detail(), Some(crate::ErrorDetail::ScopeAlignment { reason: crate::ScopeAlignmentError::Limit { kind: crate::ScopeAlignmentLimitKind::Records, required, limit: actual } }) if *required == usage.records() && *actual == limit)
    );
    let failed_usage = failed.scope_alignment_usage().unwrap();
    assert_eq!(failed_usage.records(), limit);
    assert_eq!(failed_usage.font_queries(), 0);
    assert_eq!(
        failed_usage.payload_bytes(),
        failed.input_retention().payload_bytes()
    );
    assert!(failed_usage.peak_payload_bytes() > failed_usage.payload_bytes());
    assert_eq!(
        proof
            .partitions()
            .iter()
            .map(|p| p.has_in_flow_content())
            .collect::<Vec<_>>(),
        [true, true, true]
    );
    // Fail inside the resource census after its allocation-free traversal and
    // retained-input admission, before semantic observation has started.
    assert!(input.records() > 0);
    let failed = p
        .semantic_window_with_scope_allowance(
            &r,
            ScopeAlignmentAllowance::new(input.records(), 67_108_864),
        )
        .unwrap_err();
    let census_usage = failed.scope_alignment_usage().unwrap();
    assert!(census_usage.structural_work() > 0);
    assert_eq!(census_usage.records(), input.records());
    assert_eq!(failed.input_retention(), input);
    assert_eq!(census_usage.payload_bytes(), input.payload_bytes());
    let (error, saved_usage, saved_input) = failed.into_parts();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::ScopeAlignment { .. })
    ));
    assert_eq!(saved_usage, Some(census_usage));
    assert_eq!(saved_input, input);
}

#[test]
fn live_proof_census_deduplicates_clones_and_rejects_foreign_family() {
    let p = bounded("X");
    let r = window(&p);
    let proof = p.semantic_window(&r).unwrap();
    let clone = proof.clone();
    let products = [
        crate::ScopeAlignmentProduct::SemanticWindow(&proof),
        crate::ScopeAlignmentProduct::SemanticWindow(&clone),
    ];
    let census = crate::scope_alignment_census(
        crate::ScopeAlignmentCensusInput::Product(crate::ScopeAlignmentProduct::SemanticWindow(
            &proof,
        )),
        crate::ScopeAlignmentProducts::Mixed(&products),
        ScopeAlignmentAllowance::new(100_000, 67_108_864),
    )
    .unwrap();
    assert_eq!(
        census.external_retention(),
        ScopeAlignmentRetention::default()
    );
    assert!(
        census.input_retention().payload_bytes()
            >= proof.scope_alignment_retention().payload_bytes()
    );
    assert_eq!(census.usage().font_queries(), 0);
    let foreign = bounded("X");
    let controls = foreign
        .control_views()
        .map(|c| c.handle().captured_control().unwrap())
        .collect::<Vec<_>>();
    let native = foreign
        .native_inline_box(controls[0].clone(), controls[1].clone())
        .unwrap();
    let family = foreign.inline_box_family(vec![native], None).unwrap();
    let mut bad = r;
    bad.inline_boxes = Some(family);
    assert_eq!(
        window_reason(&p.semantic_window(&bad).unwrap_err()),
        &ProtocolError::InlineBoxFragmentation(crate::InlineBoxFragmentationError::ForeignFamily)
    );
    assert!(proof.matches(&window(&p)));
}
fn content(p: &PreparedText) -> bool {
    let evidence = p.semantic_window(&window(p)).unwrap();
    assert_eq!(evidence.partitions().len(), 1);
    assert_eq!(
        evidence.partitions()[0].terminal(),
        p.source_segments().next().unwrap().end_opportunity()
    );
    assert_eq!(
        evidence.partitions()[0]
            .terminal()
            .view()
            .alternatives()
            .count(),
        0
    );
    evidence.partitions()[0].has_in_flow_content()
}

#[test]
fn original_content_and_edge_whitespace_are_semantic_not_advance_tests() {
    let mut system = system();
    for (text, mode, spacing, expected) in [
        ("", WhiteSpace::Collapse, 0.0, false),
        (" ", WhiteSpace::Collapse, 0.0, false),
        ("\u{1680}", WhiteSpace::Collapse, 0.0, false),
        (" ", WhiteSpace::Preserve, -20.0, true),
        ("X", WhiteSpace::Collapse, -20.0, true),
        ("X", WhiteSpace::Collapse, -25.0, true),
        (" X ", WhiteSpace::Collapse, 0.0, true),
    ] {
        let mut value = input(text, mode);
        value.base_style.letter_spacing = spacing;
        let p = capture(&mut system, value);
        let stats = system.stats();
        assert_eq!(content(&p), expected, "{text:?}/{mode:?}/{spacing}");
        assert_eq!(system.stats(), stats);
        let actual = p
            .select(request(&p))
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(actual.has_in_flow_content(), expected);
    }
}

#[test]
fn evidence_keeps_exact_source_and_fixed_true_false_missing_context() {
    let mut system = system();
    let p = capture(&mut system, input("X", WhiteSpace::Collapse));
    let mut r = window(&p);
    let missing = p.semantic_window(&r).unwrap();
    assert!(missing.matches(&r));
    assert_eq!(missing.request().formatted_line.first(), None);
    r.formatted_line = FormattedLineContext::new(Some(true), Some(false));
    let known = p.semantic_window(&r).unwrap();
    assert!(known.matches(&r));
    assert!(!missing.matches(&r));
    assert_eq!(known.request().formatted_line, r.formatted_line);
    for first in [Some(true), Some(false), None] {
        for last in [Some(true), Some(false), None] {
            let mut context = r.clone();
            context.formatted_line = FormattedLineContext::new(first, last);
            let evidence = p.semantic_window(&context).unwrap();
            assert!(evidence.partitions()[0].has_in_flow_content());
            assert_eq!(evidence.request().formatted_line, context.formatted_line);
        }
    }
    assert_eq!(
        known.participants().collect::<Vec<_>>(),
        p.participants().collect::<Vec<_>>()
    );
    let foreign = capture(&mut system, input("X", WhiteSpace::Collapse));
    r.start = foreign.source_segments().next().unwrap().start();
    assert!(matches!(
        window_reason(&p.semantic_window(&r).unwrap_err()),
        ProtocolError::ForeignPreparation {
            kind: ProtocolHandleKind::Boundary,
            ..
        }
    ));
    assert!(missing.matches(&window(&p)));
}

#[test]
fn mandatory_equal_source_ends_are_distinct_and_resumption_does_not_repeat_forced() {
    let mut value = input("X", WhiteSpace::Collapse);
    value.controls = vec![
        paragraph::ParagraphControl {
            at: 0,
            kind: crate::ParagraphControlKind::ForcedBreak,
        },
        paragraph::ParagraphControl {
            at: 0,
            kind: crate::ParagraphControlKind::ForcedBreak,
        },
    ];
    value.anchor_order = (0..2)
        .map(|i| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(i),
            affinity: crate::Affinity::After,
        })
        .collect();
    let p = capture(&mut system(), value);
    let mut r = window(&p);
    let evidence = p.semantic_window(&r).unwrap();
    assert_eq!(
        evidence
            .partitions()
            .iter()
            .map(|p| (p.terminal().view().kind(), p.has_in_flow_content()))
            .collect::<Vec<_>>(),
        [
            (PreparedBreakKind::Forced, false),
            (PreparedBreakKind::Forced, false),
            (PreparedBreakKind::ParagraphEnd, true)
        ]
    );
    assert_ne!(
        evidence.partitions()[0].terminal(),
        evidence.partitions()[1].terminal()
    );
    assert_eq!(
        evidence.partitions()[0]
            .terminal()
            .view()
            .boundary()
            .view()
            .source_after(),
        evidence.partitions()[1]
            .terminal()
            .view()
            .boundary()
            .view()
            .source_after()
    );
    r.start = evidence.partitions()[0].terminal().view().boundary();
    r.controls.remove(0);
    let resumed = p.semantic_window(&r).unwrap();
    assert_eq!(resumed.partitions().len(), 2);
    assert_eq!(
        resumed.partitions()[0].terminal(),
        evidence.partitions()[1].terminal()
    );
    r.completion = evidence.partitions()[0].terminal();
    assert_eq!(
        window_reason(&p.semantic_window(&r).unwrap_err()),
        &ProtocolError::InvalidSemanticWindow
    );
}

#[test]
fn zero_size_atomic_roles_and_missing_or_foreign_occurrences_are_preserved() {
    for (kind, expected) in [
        (crate::InlineBoxKind::InFlow, true),
        (crate::InlineBoxKind::OutOfFlow, false),
    ] {
        let mut value = input("", WhiteSpace::Collapse);
        let mut source = value.source.clone();
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(21630),
            kind,
            0,
            crate::Size::new(0.0, 0.0),
        ));
        if kind == crate::InlineBoxKind::InFlow {
            value.boundary_policies = [
                paragraph::ParagraphBoundaryDefinition::AtomicBefore(0),
                paragraph::ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| paragraph::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect();
        }
        value.source = source.clone();
        value.origins[0].source = source;
        value.anchor_order.insert(
            0,
            paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Atomic(0),
                affinity: crate::Affinity::After,
            },
        );
        let twin = capture(&mut system(), value.clone());
        let p = capture(&mut system(), value);
        let mut r = window(&p);
        assert_eq!(content(&p), expected);
        let proof = p.semantic_window(&r).unwrap();
        assert_eq!(proof.request().atomics[0].kind, kind);
        r.atomics[0].kind = if expected {
            crate::InlineBoxKind::OutOfFlow
        } else {
            crate::InlineBoxKind::InFlow
        };
        assert!(matches!(
            window_reason(&p.semantic_window(&r).unwrap_err()),
            ProtocolError::InvalidSemanticAtomic { .. }
        ));
        r.atomics.clear();
        assert!(matches!(
            window_reason(&p.semantic_window(&r).unwrap_err()),
            ProtocolError::MissingAtomic { .. }
        ));
        let mut foreign = window(&p);
        foreign.atomics[0].occurrence = window(&twin).atomics[0].occurrence.clone();
        assert!(matches!(
            window_reason(&p.semantic_window(&foreign).unwrap_err()),
            ProtocolError::ForeignPreparation {
                kind: ProtocolHandleKind::Atomic,
                ..
            }
        ));
    }
}

#[test]
fn ordinary_and_latent_automatic_alternatives_keep_their_owned_source_cuts() {
    for auto_phrase in [false, true] {
        let mut value = input("associate", WhiteSpace::Collapse);
        value.base_style.locale = Some("en-US".into());
        value.base_style.hyphens = crate::Hyphens::Auto;
        value.base_style.word_break = if auto_phrase {
            crate::WordBreak::AutoPhrase
        } else {
            crate::WordBreak::Normal
        };
        let p = capture(&mut system(), value);
        let proof = p.semantic_window(&window(&p)).unwrap();
        assert!(proof.partitions()[0].has_in_flow_content());
        // The pinned en-US exception is as-so-ciate, independent of bands.
        assert_eq!(
            proof
                .conditionals()
                .iter()
                .map(|e| e
                    .alternative()
                    .view()
                    .boundary()
                    .view()
                    .source_after()
                    .get())
                .collect::<Vec<_>>(),
            [2, 4]
        );
        for alternative in proof.conditionals() {
            assert!(matches!(
                alternative.alternative().view().definition(),
                crate::PreparedAlternativeDefinition::Automatic { .. }
            ));
            assert!(alternative.before_has_in_flow_content());
            assert_eq!(alternative.partition(), 0);
        }
    }
}

#[test]
fn actual_boundary_participation_is_bound_without_metric_struts() {
    let mut value = input("", WhiteSpace::Collapse);
    value.controls = vec![
        paragraph::ParagraphControl {
            at: 0,
            kind: crate::ParagraphControlKind::InlineBoundaryStart,
        },
        paragraph::ParagraphControl {
            at: 0,
            kind: crate::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    value
        .anchor_order
        .extend((0..2).map(|i| paragraph::ParagraphAnchor {
            item: paragraph::ParagraphAnchorItem::Control(i),
            affinity: crate::Affinity::After,
        }));
    let p = capture(&mut system(), value);
    let mut r = window(&p);
    let neutral = p.semantic_window(&r).unwrap();
    assert!(!neutral.partitions()[0].has_in_flow_content());
    r.controls[0].boundary = Some(crate::InlineBoundaryAllocation::contribution(1.0).unwrap());
    let participating = p.semantic_window(&r).unwrap();
    assert!(participating.partitions()[0].has_in_flow_content());
    assert!(!neutral.matches(&r));
    r.controls[0].boundary = None;
    assert!(matches!(
        window_reason(&p.semantic_window(&r).unwrap_err()),
        ProtocolError::MissingControlBoundaryGeometry { .. }
    ));
}

#[test]
fn semantic_observation_defers_the_actual_terminal_metric_refusal() {
    let mut value = input("X ", WhiteSpace::Preserve);
    value.terminal_edge = ParagraphTerminalEdge::Unknown;
    let p = capture(&mut system(), value);
    let r = window(&p);
    assert!(p.semantic_window(&r).unwrap().partitions()[0].has_in_flow_content());
    let error = p
        .select(request(&p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap_err();
    assert_eq!(
        reason(&error),
        &ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
    );
}

#[test]
fn issued_shy_and_resumed_after_spelling_keep_source_provenance() {
    let mut value = input("ab\u{ad}cde", WhiteSpace::Collapse);
    value.base_style.hyphens = crate::Hyphens::Manual;
    value.base_style.locale = Some("und-Latn".into());
    value.base_style.hyphenate_limit_chars = crate::HyphenateLimitChars {
        total: crate::HyphenationMinimum::Count(0),
        before: crate::HyphenationMinimum::Count(0),
        after: crate::HyphenationMinimum::Count(0),
    };
    value.base_style.native_hyphenation_convention = Some(
        crate::NativeHyphenationConvention::language_script(
            "und-Latn",
            "Latn",
            crate::Text::from("-"),
            crate::HyphenationPlacement::BothSides,
        )
        .unwrap(),
    );
    let p = capture(&mut system(), value);
    let r = window(&p);
    let whole = p.semantic_window(&r).unwrap();
    let alternative = whole
        .conditionals()
        .iter()
        .find(|e| e.alternative().view().source_range() == Some(Range::new(2, 3)))
        .expect("word-internal SHY issues its actual native alternative")
        .alternative();
    assert_eq!(alternative.view().text().code_units(), [45]);
    assert!(
        whole
            .conditionals()
            .iter()
            .any(|e| e.after_has_in_flow_content() == Some(true))
    );
    let mut selected = request(&p);
    selected.end = alternative.view().opportunity().unwrap().view().boundary();
    selected.termination = PreparedTermination::Ordinary(alternative.view().opportunity().unwrap());
    selected.context.end = LineEnd::Soft;
    selected
        .participants
        .truncate(selected.end.view().participant_offset());
    selected.alternative = Some(alternative.clone());
    let line = p
        .select(selected)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let mut resumed = r;
    resumed.start = line.selected_line().request().end.clone();
    resumed.incoming = line.hyphenation_continuation();
    assert!(resumed.incoming.is_some());
    let proof = p.semantic_window(&resumed).unwrap();
    assert!(
        proof.partitions()[0].has_in_flow_content(),
        "receiving window retains computed and original spelling"
    );
    assert_eq!(
        proof.request().incoming.as_ref().unwrap().alternative(),
        alternative
    );
    resumed.incoming = None;
    assert!(p.semantic_window(&resumed).unwrap().partitions()[0].has_in_flow_content());
}
