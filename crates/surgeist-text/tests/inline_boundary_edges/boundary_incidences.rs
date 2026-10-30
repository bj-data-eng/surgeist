//! Original-event placements of existing boundaries, without granting new cuts.
use super::*;
use std::hash::{Hash, Hasher};
use t::CapturedScopeIncidence as R;

fn block(base: t::Style) -> t::SourceProcessingScope {
    t::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: t::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: base,
        used_direction: t::ParagraphBaseDirection::LeftToRight,
    }
}
fn inline() -> t::SourceProcessingScope {
    t::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: None,
        boundaries: None,
    }
}
fn input(text: &str, ranges: &[(usize, usize)]) -> t::ContextualSourceInput {
    let source = t::Source::identified(text, t::Id::from_u64(75081), 3);
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    for i in 0..ranges.len() {
        if i > 0 {
            events.push(E::Connection(i - 1));
        }
        events.push(origin(i));
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    t::ContextualSourceInput {
        scope_alignment_limits: None,
        source: source.clone(),
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        envelope: t::SourceProcessingEnvelope::Complete,
        origins: ranges
            .iter()
            .map(|&(a, b)| t::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: t::Range::new(a, b),
                aggregate: t::Range::new(a, b),
                kind: t::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: (1..ranges.len())
            .map(|i| t::SourceProcessingConnection {
                before_origin: i - 1,
                after_origin: i,
                kind: t::ParagraphConnectionKind::Continuous,
            })
            .collect(),
        boundary_policies: (1..ranges.len())
            .map(|i| t::ParagraphBoundaryPolicy {
                definition: t::ParagraphBoundaryDefinition::Connection(i - 1),
                preceding_wrap: t::Wrap::Word,
                following_wrap: t::Wrap::Word,
                ancestor_wrap: t::Wrap::Word,
            })
            .collect(),
        scopes: vec![block(style(t::WhiteSpace::Preserve))],
        paragraphs: vec![t::SourceProcessingParagraph {
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: t::ParagraphStrut::DefaultFont,
        }],
        controls: vec![],
        computed_line_boundaries: vec![],
        replacements: vec![],
        events,
    }
}
fn prepare_processed(processed: &Arc<t::ProcessedSource>, bytes: &[u8]) -> Vec<t::PreparedText> {
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = t::AnalyzedSource::from_processed_paragraph(
                paragraph,
                t::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let policies = analyzed
                .wrapping_relations()
                .filter(|r| r.ancestor_wrap().is_none())
                .map(|r| t::WrappingPolicy {
                    relation: r.handle(),
                    ancestor_wrap: t::Wrap::Word,
                })
                .collect();
            system
                .prepare(
                    &analyzed.with_wrapping_policies(policies).unwrap(),
                    t::PreparationOptions::default(),
                )
                .unwrap()
        })
        .collect()
}
fn prepare_capture_input(
    value: t::ContextualSourceInput,
    bytes: &[u8],
) -> (Arc<t::ProcessedSource>, t::PreparedText) {
    let processed = Arc::new(t::process_contextual_source(value).unwrap());
    let mut paragraphs = prepare_processed(&processed, bytes);
    assert_eq!(paragraphs.len(), 1);
    (processed, paragraphs.pop().unwrap())
}
fn basic_inventory(p: &t::PreparedText) -> Vec<(t::PreparedBreakKind, usize, usize)> {
    p.opportunity_views()
        .map(|o| {
            (
                o.kind(),
                o.boundary().view().source_before().get(),
                o.boundary().view().source_after().get(),
            )
        })
        .collect()
}
fn space_offer(p: &t::PreparedText) -> t::PreparedBoundary {
    assert_eq!(
        basic_inventory(p),
        [
            (t::PreparedBreakKind::Allowed, 2, 2),
            (t::PreparedBreakKind::ParagraphEnd, 3, 3)
        ]
    );
    soft_at(p, 2).into_iter().next().unwrap().view().boundary()
}
fn hash(value: &impl Hash) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}
fn rows(p: &t::PreparedText, b: &t::PreparedBoundary, scopes: &[t::CapturedScope]) -> Vec<Vec<R>> {
    let before = basic_inventory(p);
    let choices = p.boundary_incidences(b).unwrap();
    let again = p.boundary_incidences(b).unwrap();
    assert_eq!(choices, again);
    assert_eq!(hash(&choices), hash(&again));
    let result = choices
        .iter()
        .map(|choice| {
            assert_eq!(choice.boundary(), *b);
            let forward = scopes
                .iter()
                .map(|s| p.incidence_scope_relation(choice, s).unwrap())
                .collect::<Vec<_>>();
            let mut reverse = scopes
                .iter()
                .rev()
                .map(|s| p.incidence_scope_relation(choice, s).unwrap())
                .collect::<Vec<_>>();
            reverse.reverse();
            assert_eq!(forward, reverse);
            forward
        })
        .collect();
    assert_eq!(basic_inventory(p), before);
    result
}
fn empty_input() -> t::ContextualSourceInput {
    let mut value = input("A B", &[(0, 2), (2, 3)]);
    value.scopes.push(inline());
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::Connection(0),
        E::EnterScope(1),
        E::ExitScope(1),
        origin(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    value
}

#[test]
fn one_empty_scope_has_two_incidences_of_the_same_soft_boundary() {
    for direction in [
        t::ParagraphBaseDirection::LeftToRight,
        t::ParagraphBaseDirection::RightToLeft,
    ] {
        let mut value = empty_input();
        value.paragraphs[0].base_direction = direction;
        let t::SourceProcessingScope::Block { used_direction, .. } = &mut value.scopes[0] else {
            unreachable!()
        };
        *used_direction = direction;
        let (capture, p) = prepare_capture_input(value, AHEM);
        let b = space_offer(&p);
        assert_eq!(
            rows(&p, &b, &[capture.captured_scope(1).unwrap()]),
            [vec![R::AtBefore], vec![R::AtAfter]]
        );
    }
}

#[test]
fn adjacent_nonempty_edges_across_connection_are_one_shared_incidence() {
    let mut value = input("A B", &[(0, 2), (2, 3)]);
    value.scopes.extend([inline(), inline()]);
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::ExitScope(1),
        E::Connection(0),
        E::EnterScope(2),
        origin(1),
        E::ExitScope(2),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, AHEM);
    let b = space_offer(&p);
    assert_eq!(
        rows(
            &p,
            &b,
            &[
                capture.captured_scope(1).unwrap(),
                capture.captured_scope(2).unwrap()
            ]
        ),
        [vec![R::AtAfter, R::AtBefore]]
    );
}

#[test]
fn ordered_sibling_empties_have_three_consistent_global_choices() {
    let mut value = input("A B", &[(0, 2), (2, 3)]);
    value.scopes.extend([inline(), inline(), inline()]);
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::EnterScope(2),
        E::ExitScope(2),
        E::Connection(0),
        E::EnterScope(3),
        E::ExitScope(3),
        origin(1),
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, AHEM);
    let b = space_offer(&p);
    assert_eq!(
        rows(
            &p,
            &b,
            &[
                capture.captured_scope(1).unwrap(),
                capture.captured_scope(2).unwrap(),
                capture.captured_scope(3).unwrap()
            ]
        ),
        [
            vec![R::Inside, R::AtBefore, R::Before],
            vec![R::Inside, R::AtAfter, R::AtBefore],
            vec![R::Inside, R::After, R::AtAfter],
        ]
    );
}

#[test]
fn an_empty_original_occurrence_constrains_which_side_of_its_scope_can_move() {
    let mut value = input("A B", &[(0, 2), (2, 2), (2, 3)]);
    value.scopes.push(inline());
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::Connection(0),
        E::EnterScope(1),
        origin(1),
        E::ExitScope(1),
        E::Connection(1),
        origin(2),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, AHEM);
    let empty = p.origin_views().nth(1).unwrap().handle();
    let unit = p
        .unit_views()
        .find(|u| u.empty_origin().as_ref() == Some(&empty))
        .unwrap()
        .handle();
    let at = p
        .participants()
        .position(|part| part == t::PreparedParticipant::Text(unit.clone()))
        .unwrap();
    let before = p
        .boundary_views()
        .find(|b| b.participant_offset() == at)
        .unwrap()
        .handle();
    let after = p
        .boundary_views()
        .find(|b| b.participant_offset() == at + 1)
        .unwrap()
        .handle();
    assert_eq!(
        before.view().processed_position(),
        after.view().processed_position()
    );
    let scope = capture.captured_scope(1).unwrap();
    assert_eq!(
        rows(&p, &before, std::slice::from_ref(&scope)),
        [vec![R::AtBefore]]
    );
    assert_eq!(rows(&p, &after, &[scope]), [vec![R::AtAfter]]);
}

#[test]
fn phase_i_deleted_space_retains_its_original_before_and_after_custody() {
    let mut value = input("X  Y", &[(0, 2), (2, 3), (3, 4)]);
    let t::SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
        unreachable!()
    };
    base_style.white_space = t::WhiteSpace::Collapse;
    base_style.word_break = t::WordBreak::BreakAll;
    value.scopes.push(inline());
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::Connection(0),
        origin(1),
        E::ExitScope(1),
        E::Connection(1),
        origin(2),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, AHEM);
    assert_eq!(
        basic_inventory(&p),
        [
            (t::PreparedBreakKind::Allowed, 2, 2),
            (t::PreparedBreakKind::Allowed, 3, 3),
            (t::PreparedBreakKind::ParagraphEnd, 4, 4)
        ]
    );
    let early = soft_at(&p, 2).remove(0).view().boundary();
    let late = soft_at(&p, 3).remove(0).view().boundary();
    assert_eq!(
        early.view().processed_position(),
        late.view().processed_position()
    );
    let removed = p
        .unit_views()
        .find(|u| u.source_range() == t::Range::new(2, 3))
        .unwrap();
    assert!(removed.processed_range().is_empty());
    let scope = capture.captured_scope(1).unwrap();
    assert_eq!(
        rows(&p, &early, std::slice::from_ref(&scope)),
        [vec![R::Inside]]
    );
    assert_eq!(rows(&p, &late, &[scope]), [vec![R::AtAfter]]);
}

#[test]
fn exact_endpoint_exterior_sides_keep_the_queried_mixed_prefix() {
    let mut value = empty_input();
    let t::SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(t::SourceProcessingScopeBoundaries { start: 0, end: 1 });
    value.controls = vec![
        t::ParagraphControl {
            at: 2,
            kind: K::InlineBoundaryStart,
        },
        t::ParagraphControl {
            at: 2,
            kind: K::InlineBoundaryEnd,
        },
    ];
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::Connection(0),
        E::EnterScope(1),
        control(0),
        control(1),
        E::ExitScope(1),
        origin(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, AHEM);
    let scope = capture.captured_scope(1).unwrap();
    let start = p
        .control_views()
        .find(|v| matches!(v.kind(), K::InlineBoundaryStart))
        .unwrap()
        .handle();
    let end = p
        .control_views()
        .find(|v| matches!(v.kind(), K::InlineBoundaryEnd))
        .unwrap()
        .handle();
    assert!(p.control_is_boundary_exact(&start).unwrap());
    assert!(p.control_is_boundary_exact(&end).unwrap());
    let s = p
        .participants()
        .position(|part| part == t::PreparedParticipant::Control(start.clone()))
        .unwrap();
    let e = p
        .participants()
        .position(|part| part == t::PreparedParticipant::Control(end.clone()))
        .unwrap()
        + 1;
    let before = p
        .boundary_views()
        .find(|b| b.participant_offset() == s)
        .unwrap()
        .handle();
    let after = p
        .boundary_views()
        .find(|b| b.participant_offset() == e)
        .unwrap()
        .handle();
    assert!(!p.boundary_consumes_control(&before, &start).unwrap());
    assert!(p.boundary_consumes_control(&after, &end).unwrap());
    // These are issued facts boundaries. No following opportunity is invented
    // for End, and neither queried boundary is moved to the other one's prefix.
    assert_eq!(
        rows(&p, &before, std::slice::from_ref(&scope)),
        [vec![R::AtBefore]]
    );
    assert_eq!(rows(&p, &after, &[scope]), [vec![R::AtAfter]]);
}

#[test]
fn computed_pin_cannot_slide_across_an_empty_box_at_the_same_source_position() {
    for (side, associated_endpoint, expected) in [
        (t::SourceBoxSide::Before, false, R::AtBefore),
        (t::SourceBoxSide::Before, true, R::AtBefore),
        (t::SourceBoxSide::After, false, R::AtAfter),
    ] {
        let mut value = input("AB", &[(0, 1), (1, 2)]);
        value.scopes.push(inline());
        value.events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            E::ExitScope(1),
            origin(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ];
        value.computed_line_boundaries =
            vec![t::ComputedLineBoundaryInput::Scope { scope: 1, side }];
        if associated_endpoint {
            let t::SourceProcessingScope::Inline { boundaries, .. } = &mut value.scopes[1] else {
                unreachable!()
            };
            *boundaries = Some(t::SourceProcessingScopeBoundaries { start: 0, end: 1 });
            value.controls = vec![
                t::ParagraphControl {
                    at: 1,
                    kind: K::InlineBoundaryStart,
                },
                t::ParagraphControl {
                    at: 1,
                    kind: K::InlineBoundaryEnd,
                },
            ];
            let enter = value
                .events
                .iter()
                .position(|e| matches!(e, E::EnterScope(1)))
                .unwrap();
            value.events.insert(enter + 1, control(0));
            let exit = value
                .events
                .iter()
                .position(|e| matches!(e, E::ExitScope(1)))
                .unwrap();
            value.events.insert(exit, control(1));
            value
                .computed_line_boundaries
                .push(t::ComputedLineBoundaryInput::Endpoint { control: 0 });
        }
        let (capture, p) = prepare_capture_input(value, AHEM);
        assert_eq!(
            basic_inventory(&p),
            [
                (t::PreparedBreakKind::Forced, 1, 1),
                (t::PreparedBreakKind::ParagraphEnd, 2, 2)
            ]
        );
        let offer = p
            .opportunity_views()
            .find(|o| o.kind() == t::PreparedBreakKind::Forced)
            .unwrap();
        assert_eq!(
            offer
                .computed_line_boundary()
                .unwrap()
                .view()
                .causes()
                .count(),
            if associated_endpoint { 2 } else { 1 }
        );
        let b = offer.boundary();
        assert_eq!(
            rows(&p, &b, &[capture.captured_scope(1).unwrap()]),
            [vec![expected]]
        );
    }
}

#[test]
fn a_scope_edge_hidden_inside_a_cross_origin_tcu_is_not_an_extra_incidence() {
    let mut value = input("A\u{301}B", &[(0, 1), (1, 3)]);
    let t::SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
        unreachable!()
    };
    base_style.word_break = t::WordBreak::BreakAll;
    value.scopes.extend([inline(), inline()]);
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::ExitScope(1),
        E::Connection(0),
        E::EnterScope(2),
        origin(1),
        E::ExitScope(2),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, &font::ligature());
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [t::Range::new(0, 2), t::Range::new(2, 3)]
    );
    assert_eq!(
        basic_inventory(&p),
        [
            (t::PreparedBreakKind::Allowed, 2, 2),
            (t::PreparedBreakKind::ParagraphEnd, 3, 3)
        ]
    );
    let b = soft_at(&p, 2).remove(0).view().boundary();
    assert_eq!(
        rows(
            &p,
            &b,
            &[
                capture.captured_scope(1).unwrap(),
                capture.captured_scope(2).unwrap()
            ]
        ),
        [vec![R::After, R::Inside]]
    );
}

#[test]
fn two_expanded_s_units_share_a_valid_original_body_class() {
    let mut value = input("ßB", &[(0, 1), (1, 2)]);
    let t::SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
        unreachable!()
    };
    base_style.word_break = t::WordBreak::BreakAll;
    base_style.text_transform =
        t::TextTransform::Set(t::TextTransformSet::none().with_case(t::CaseTransform::Uppercase));
    value.scopes.push(inline());
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::ExitScope(1),
        E::Connection(0),
        origin(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, p) = prepare_capture_input(value, AHEM);
    let offers = p
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().participant_offset()))
        .collect::<Vec<_>>();
    assert_eq!(
        offers,
        [
            (t::PreparedBreakKind::Allowed, 1),
            (t::PreparedBreakKind::Allowed, 2),
            (t::PreparedBreakKind::ParagraphEnd, 3)
        ]
    );
    let a = p
        .opportunity_views()
        .find(|o| o.boundary().view().participant_offset() == 1)
        .unwrap()
        .boundary();
    let b = p
        .opportunity_views()
        .find(|o| {
            o.kind() == t::PreparedBreakKind::Allowed
                && o.boundary().view().participant_offset() == 2
        })
        .unwrap()
        .boundary();
    let scope = capture.captured_scope(1).unwrap();
    assert_eq!(
        rows(&p, &a, std::slice::from_ref(&scope)),
        [vec![R::Inside]]
    );
    assert_eq!(rows(&p, &b, &[scope]), [vec![R::AtAfter]]);
    let whole = finish(
        &p,
        p.boundary_views().next().unwrap().handle(),
        paragraph_end(&p),
    );
    assert_eq!(
        whole
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [53, 53, 36]
    );
    assert_eq!(p.analyzed_source().source().text(), &t::Text::from("ßB"));
}

fn reason(error: &t::Error) -> &t::ProtocolError {
    match error.detail() {
        Some(t::ErrorDetail::InvalidTextProtocol { reason }) => reason,
        _ => panic!("{error:?}"),
    }
}
fn scope_error(error: &t::Error, expected: t::WrappingScopeQueryError) {
    assert!(
        matches!(reason(error),t::ProtocolError::WrappingScopeQuery {reason} if *reason==expected),
        "{error:?}"
    )
}

#[test]
fn incidence_capabilities_keep_preparation_capture_role_and_domain_errors_distinct() {
    let (capture, p) = prepare_capture_input(empty_input(), AHEM);
    let b = space_offer(&p);
    let scope = capture.captured_scope(1).unwrap();
    let local = p.boundary_incidences(&b).unwrap().remove(0);
    let same_capture = prepare_processed(&capture, AHEM).remove(0);
    let (foreign_capture, foreign) = prepare_capture_input(empty_input(), AHEM);
    for other in [&same_capture, &foreign] {
        let ob = space_offer(other);
        let incidence = other.boundary_incidences(&ob).unwrap().remove(0);
        assert!(matches!(
            reason(&p.boundary_incidences(&ob).unwrap_err()),
            t::ProtocolError::ForeignPreparation {
                kind: t::ProtocolHandleKind::Boundary,
                ..
            }
        ));
        assert!(matches!(
            reason(&p.incidence_scope_relation(&incidence, &scope).unwrap_err()),
            t::ProtocolError::ForeignPreparation {
                kind: t::ProtocolHandleKind::BoundaryIncidence,
                ..
            }
        ));
    }
    scope_error(
        &p.incidence_scope_relation(&local, &foreign_capture.captured_scope(1).unwrap())
            .unwrap_err(),
        t::WrappingScopeQueryError::ForeignCapture,
    );
    scope_error(
        &p.incidence_scope_relation(&local, &capture.captured_scope(0).unwrap())
            .unwrap_err(),
        t::WrappingScopeQueryError::NonInlineScope,
    );
    let mut domain = input("XYZ", &[(0, 1), (1, 2), (2, 3)]);
    domain.source.inline_box(t::InlineBox::new(
        t::Id::from_u64(75082),
        t::InlineBoxKind::OutOfFlow,
        1,
        t::Size::new(5.0, 10.0),
    ));
    domain.scopes.extend([
        inline(),
        t::SourceProcessingScope::OutOfFlowBlock {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: t::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style(t::WhiteSpace::Preserve),
            used_direction: t::ParagraphBaseDirection::LeftToRight,
            atomic: 0,
        },
        inline(),
    ]);
    domain.paragraphs.push(domain.paragraphs[0]);
    domain.connections = vec![t::SourceProcessingConnection {
        before_origin: 0,
        after_origin: 2,
        kind: t::ParagraphConnectionKind::Continuous,
    }];
    domain.boundary_policies.truncate(1);
    domain.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::Atomic {
            atomic: 0,
            affinity: t::Affinity::After,
        },
        E::EnterScope(2),
        E::EnterParagraph(1),
        E::EnterScope(3),
        origin(1),
        E::ExitScope(3),
        E::ExitParagraph(1),
        E::ExitScope(2),
        E::Connection(0),
        origin(2),
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let domain = Arc::new(t::process_contextual_source(domain).unwrap());
    let prepared = prepare_processed(&domain, AHEM);
    let parent = &prepared[0];
    let end = paragraph_end(parent).view().boundary();
    let incidence = parent.boundary_incidences(&end).unwrap().remove(0);
    scope_error(
        &parent
            .incidence_scope_relation(&incidence, &domain.captured_scope(3).unwrap())
            .unwrap_err(),
        t::WrappingScopeQueryError::WrongDomain,
    );
    let flat = t::AnalyzedSource::new(
        t::Source::new("AB"),
        style(t::WhiteSpace::Preserve),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let flat = system
        .prepare(&flat, t::PreparationOptions::default())
        .unwrap();
    scope_error(
        &flat
            .boundary_incidences(&paragraph_end(&flat).view().boundary())
            .unwrap_err(),
        t::WrappingScopeQueryError::MissingCapture,
    );
}
