#![forbid(unsafe_code)]

//! Genuine atomic content may enter after its containing paragraph has closed.

use std::sync::Arc;
use surgeist_text as text;

fn child_scope(kind: text::InlineBoxKind) -> text::SourceProcessingScope {
    match kind {
        text::InlineBoxKind::InFlow => text::SourceProcessingScope::AtomicBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: text::SourceProcessingTopology::Complete,
            trim: text::WhiteSpaceTrim::default(),
            base_style: text::Style::default(),
            used_direction: text::ParagraphBaseDirection::LeftToRight,
            atomic: 0,
        },
        text::InlineBoxKind::OutOfFlow => text::SourceProcessingScope::OutOfFlowBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: text::SourceProcessingTopology::Complete,
            trim: text::WhiteSpaceTrim::default(),
            base_style: text::Style::default(),
            used_direction: text::ParagraphBaseDirection::LeftToRight,
            atomic: 0,
        },
    }
}

fn capture(
    kind: text::InlineBoxKind,
    closed_parent: bool,
    parent_text_before_child: bool,
) -> text::ContextualSourceInput {
    let mut source = text::Source::new(if parent_text_before_child { "PX" } else { "X" });
    source.inline_box(text::InlineBox::new(
        text::Id::from_u64(6312),
        kind,
        0,
        text::Size::new(10.0, 10.0),
    ));
    let origin = |start, end| text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(start, end),
        aggregate: text::Range::new(start, end),
        kind: text::ParagraphOriginKind::Authored,
    };
    let origins = if parent_text_before_child {
        vec![origin(0, 1), origin(1, 2)]
    } else {
        vec![origin(0, 1)]
    };
    use text::SourceProcessingEvent as E;
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Atomic {
            atomic: 0,
            affinity: text::Affinity::After,
        },
    ];
    if parent_text_before_child {
        events.push(E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        });
    }
    if closed_parent {
        events.push(E::ExitParagraph(0));
    }
    // The containing scope is still genuinely open. The nominee occurrence is
    // unchanged and, in the positive cases, remains at the current source cursor.
    events.extend([
        E::EnterScope(1),
        E::EnterParagraph(1),
        E::Origin {
            origin: usize::from(parent_text_before_child),
            affinity: text::Affinity::After,
        },
        E::ExitParagraph(1),
        E::ExitScope(1),
    ]);
    if !closed_parent {
        events.push(E::ExitParagraph(0));
    }
    events.push(E::ExitScope(0));
    text::ContextualSourceInput {
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins,
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        boundary_policies: if kind == text::InlineBoxKind::InFlow {
            [
                text::ParagraphBoundaryDefinition::AtomicBefore(0),
                text::ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| text::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: text::Wrap::Word,
                following_wrap: text::Wrap::Word,
                ancestor_wrap: text::Wrap::Word,
            })
            .collect()
        } else {
            vec![]
        },
        scopes: vec![
            text::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: text::WhiteSpaceTrim::default(),
                base_style: text::Style::default(),
                used_direction: text::ParagraphBaseDirection::LeftToRight,
            },
            child_scope(kind),
        ],
        paragraphs: (0..2)
            .map(|_| text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            })
            .collect(),
        events,
    }
}

fn admits_exact_nominee(kind: text::InlineBoxKind, closed_parent: bool) {
    let processed = Arc::new(text::process_contextual_source(capture(kind, closed_parent, false))
        .unwrap_or_else(|error| panic!("{kind:?} child with closed_parent={closed_parent} retains its genuine nominee and containing scope and must admit: {error:?}")));
    assert_eq!(processed.source().text(), &text::Text::from("X"));
    assert_eq!(processed.source().boxes().len(), 1);
    assert_eq!(processed.source().boxes()[0].id(), text::Id::from_u64(6312));
    assert_eq!(processed.source().boxes()[0].kind(), kind);
    let paragraphs: Vec<_> = processed.paragraphs().collect();
    assert_eq!(paragraphs.len(), 2);
    assert!(paragraphs[0].source_pieces().next().is_none());
    assert_eq!(
        paragraphs[1].source_pieces().collect::<Vec<_>>(),
        [text::Range::new(0, 1)]
    );
    let original: Vec<_> = paragraphs[1]
        .plaintext_items(text::Affinity::After)
        .flat_map(|item| match item {
            text::PlaintextItem::Text(value) => value.code_units().to_vec(),
            text::PlaintextItem::InlineBox(_) => panic!("the parent nominee is not child text"),
        })
        .collect();
    assert_eq!(original, [u16::from(b'X')]);
}

fn rejects(input: text::ContextualSourceInput, reason: text::SourceProcessingError) {
    let Err(error) = text::process_contextual_source(input) else {
        panic!("an invalid atomic content association must refuse {reason:?}")
    };
    assert_eq!(error.code, text::ErrorCode::InvalidSourceProcessing);
    assert!(
        matches!(error.detail(), Some(text::ErrorDetail::SourceProcessing { reason: actual }) if *actual == reason),
        "{error:?}"
    );
}

#[test]
fn in_flow_atomic_child_admits_after_its_nominee_parent_paragraph_closes() {
    admits_exact_nominee(text::InlineBoxKind::InFlow, true);
}

#[test]
fn out_of_flow_atomic_child_admits_after_its_nominee_parent_paragraph_closes() {
    admits_exact_nominee(text::InlineBoxKind::OutOfFlow, true);
}

#[test]
fn active_parent_counterparts_keep_the_exact_child_origin_and_nominee() {
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        admits_exact_nominee(kind, false);
    }
}

#[test]
fn a_content_domain_cannot_nominate_an_atomic_of_the_other_kind() {
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        let mut input = capture(kind, false, false);
        input.scopes[1] = child_scope(if kind == text::InlineBoxKind::InFlow {
            text::InlineBoxKind::OutOfFlow
        } else {
            text::InlineBoxKind::InFlow
        });
        rejects(input, text::SourceProcessingError::Domain);
    }
}

#[test]
fn a_content_domain_requires_its_nominee_occurrence_to_have_happened() {
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        let mut input = capture(kind, false, false);
        input
            .events
            .retain(|event| !matches!(event, text::SourceProcessingEvent::Atomic { .. }));
        rejects(input, text::SourceProcessingError::Domain);
    }
}

#[test]
fn an_actual_nominee_at_a_different_source_cursor_cannot_donate_its_content_domain() {
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        // Consuming the genuine parent origin P moves the cursor to1; the
        // nominee remains anchored at0 and the child owns the genuine X at1.
        rejects(
            capture(kind, false, true),
            text::SourceProcessingError::Domain,
        );
    }
}

#[test]
fn one_actual_nominee_cannot_supply_two_distinct_content_domains() {
    use text::SourceProcessingEvent as E;
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        let mut input = capture(kind, false, false);
        input.scopes.push(child_scope(kind));
        input.paragraphs.push(input.paragraphs[1]);
        // The first child is genuinely empty. Both child entries stay at the
        // nominee's anchor0, so duplicate custody is the sole association flaw.
        input
            .events
            .retain(|event| !matches!(event, E::Origin { .. }));
        let before_parent_exit = input
            .events
            .iter()
            .position(|event| matches!(event, E::ExitParagraph(0)))
            .unwrap();
        input.events.splice(
            before_parent_exit..before_parent_exit,
            [
                E::EnterScope(2),
                E::EnterParagraph(2),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::ExitParagraph(2),
                E::ExitScope(2),
            ],
        );
        rejects(input, text::SourceProcessingError::Occurrence);
    }
}

#[test]
fn a_later_paragraph_in_the_same_domain_does_not_replace_nominee_parentage() {
    use text::SourceProcessingEvent as E;
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        let mut input = capture(kind, false, false);
        input.paragraphs.push(input.paragraphs[0]);
        let child = input
            .events
            .iter()
            .position(|event| matches!(event, E::EnterScope(1)))
            .unwrap();
        input
            .events
            .splice(child..child, [E::ExitParagraph(0), E::EnterParagraph(2)]);
        let end = input
            .events
            .iter()
            .rposition(|event| matches!(event, E::ExitParagraph(0)))
            .unwrap();
        input.events[end] = E::ExitParagraph(2);
        let processed = Arc::new(text::process_contextual_source(input).unwrap());
        let paragraphs: Vec<_> = processed.paragraphs().collect();
        assert_eq!(paragraphs.len(), 3);
        assert!(paragraphs[0].source_pieces().next().is_none());
        assert!(paragraphs[1].source_pieces().next().is_none());
        assert_eq!(
            paragraphs[2].source_pieces().collect::<Vec<_>>(),
            [text::Range::new(0, 1)]
        );
        assert_eq!(processed.source().boxes()[0].id(), text::Id::from_u64(6312));
    }
}

#[test]
fn a_same_anchor_nominee_from_another_processing_domain_cannot_be_borrowed() {
    use text::SourceProcessingEvent as E;
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        let mut input = capture(kind, false, false);
        input.scopes.push(input.scopes[0].clone());
        input.paragraphs.push(input.paragraphs[0]);
        let child = input
            .events
            .iter()
            .position(|event| matches!(event, E::EnterScope(1)))
            .unwrap();
        input.events.splice(
            child..child,
            [E::ExitParagraph(0), E::EnterScope(2), E::EnterParagraph(2)],
        );
        let end = input
            .events
            .iter()
            .rposition(|event| matches!(event, E::ExitParagraph(0)))
            .unwrap();
        input
            .events
            .splice(end..=end, [E::ExitParagraph(2), E::ExitScope(2)]);
        rejects(input, text::SourceProcessingError::Domain);
    }
}
