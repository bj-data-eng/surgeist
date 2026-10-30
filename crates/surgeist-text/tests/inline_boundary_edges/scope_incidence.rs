//! Public prefix custody and logical scope facts from genuine captured events.
//! Scope membership is independent of physical placement and source hulls.
use super::*;

fn base() -> t::Style {
    t::Style {
        word_break: t::WordBreak::BreakAll,
        ..style(t::WhiteSpace::Preserve)
    }
}
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
fn inline(p: &t::PreparedText) -> Vec<t::CapturedScope> {
    p.captured_paragraph()
        .unwrap()
        .scope_intersections()
        .map(|i| i.scope())
        .filter(|s| s.role() == t::ScopeRole::Inline)
        .collect()
}
fn local(p: &t::PreparedText, c: &t::CapturedControl) -> t::PreparedControl {
    p.control_views()
        .find(|v| v.handle().captured_control().as_ref() == Some(c))
        .unwrap()
        .handle()
}
fn end(p: &t::PreparedText) -> t::PreparedOpportunity {
    p.opportunity_views()
        .find(|o| o.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle()
}
fn cut(p: &t::PreparedText, at: usize) -> t::PreparedOpportunity {
    soft_at(p, at).into_iter().next().unwrap()
}
fn unit_consumed(p: &t::PreparedText, b: &t::PreparedBoundary, unit: &t::PreparedUnit) -> bool {
    p.participants()
        .take(b.view().participant_offset())
        .any(|part| part == t::PreparedParticipant::Text(unit.clone()))
}

fn prefix(p: &t::PreparedText, b: &t::PreparedBoundary, c: &t::PreparedControl) -> bool {
    // This is the public participant inventory's declared prefix, including a
    // real control represented by generated text rather than a control slot.
    p.participants()
        .take(b.view().participant_offset())
        .any(|part| match part {
            t::PreparedParticipant::Control(actual) => actual == *c,
            t::PreparedParticipant::Text(unit) => unit.view().controls().any(|actual| actual == *c),
            _ => false,
        })
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
    );
}
fn input(value: &str, base: t::Style, ranges: &[(usize, usize)]) -> t::ContextualSourceInput {
    let source = t::Source::identified(value, t::Id::from_u64(75072), 11);
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    for i in 0..ranges.len() {
        if i > 0 {
            events.push(E::Connection(i - 1));
        }
        events.push(origin(i));
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    t::ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
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
        boundary_policies: (0..ranges.len().saturating_sub(1))
            .map(|i| t::ParagraphBoundaryPolicy {
                definition: t::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: t::Wrap::Word,
                following_wrap: t::Wrap::Word,
                ancestor_wrap: t::Wrap::Word,
            })
            .collect(),
        scopes: vec![block(base)],
        paragraphs: vec![t::SourceProcessingParagraph {
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: t::ParagraphStrut::DefaultFont,
        }],
        controls: vec![],
        replacements: vec![],
        events,
    }
}
fn prepare_input(value: t::ContextualSourceInput, font: &[u8]) -> Vec<t::PreparedText> {
    let processed = Arc::new(t::process_contextual_source(value).unwrap());
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.to_vec()).unwrap();
    processed
        .paragraphs()
        .map(|paragraph| {
            let a = t::AnalyzedSource::from_processed_paragraph(
                paragraph,
                t::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let policies = a
                .wrapping_relations()
                .filter(|r| r.ancestor_wrap().is_none())
                .map(|r| t::WrappingPolicy {
                    relation: r.handle(),
                    ancestor_wrap: t::Wrap::Word,
                })
                .collect();
            system
                .prepare(
                    &a.with_wrapping_policies(policies).unwrap(),
                    t::PreparationOptions::default(),
                )
                .unwrap()
        })
        .collect()
}

#[test]
fn ordinary_cuts_distinguish_nested_sibling_and_absolute_prefix_membership() {
    let p = prepare_capture(
        t::Source::new("ABCDE"),
        block(base()),
        &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryEnd, 3),
            (K::InlineBoundaryStart, 1),
            (K::InlineBoundaryEnd, 2),
            (K::InlineBoundaryStart, 3),
            (K::InlineBoundaryEnd, 5),
        ],
        &[(0, 1), (2, 3), (4, 5)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            E::Connection(0),
            E::EnterScope(2),
            control(2),
            origin(1),
            control(3),
            E::ExitScope(2),
            E::Connection(1),
            origin(2),
            control(1),
            E::ExitScope(1),
            E::Connection(2),
            E::EnterScope(3),
            control(4),
            origin(3),
            E::Connection(3),
            origin(4),
            control(5),
            E::ExitScope(3),
        ],
        AHEM,
    );
    let scopes = inline(&p);
    let inner = scopes
        .iter()
        .find(|s| s.parent().is_some_and(|p| p.role() == t::ScopeRole::Inline))
        .unwrap()
        .clone();
    let outer = inner.parent().unwrap();
    let sibling = scopes
        .into_iter()
        .find(|s| *s != outer && s.parent() == outer.parent())
        .unwrap();
    let os = local(&p, &outer.entry_control().unwrap());
    let oe = local(&p, &outer.exit_control().unwrap());
    let ins = local(&p, &inner.entry_control().unwrap());
    let ine = local(&p, &inner.exit_control().unwrap());
    let ss = local(&p, &sibling.entry_control().unwrap());
    let se = local(&p, &sibling.exit_control().unwrap());
    let a = cut(&p, 1);
    let b = cut(&p, 2);
    let c = cut(&p, 3);
    let d = cut(&p, 4);
    let start = p.boundary_views().next().unwrap().handle();
    for (boundary, membership, consumed) in [
        (
            start,
            [false, false, false],
            [false, false, false, false, false, false],
        ),
        (
            a.view().boundary(),
            [true, false, false],
            [true, false, false, false, false, false],
        ),
        (
            b.view().boundary(),
            [true, false, false],
            [true, false, true, true, false, false],
        ),
        (
            c.view().boundary(),
            [false, false, false],
            [true, true, true, true, false, false],
        ),
        (
            d.view().boundary(),
            [false, false, true],
            [true, true, true, true, true, false],
        ),
        (
            end(&p).view().boundary(),
            [false, false, false],
            [true, true, true, true, true, true],
        ),
    ] {
        for (scope, expected) in [&outer, &inner, &sibling].into_iter().zip(membership) {
            assert_eq!(
                p.boundary_inside_captured_scope(&boundary, scope).unwrap(),
                expected
            );
        }
        for (control, expected) in [&os, &oe, &ins, &ine, &ss, &se].into_iter().zip(consumed) {
            assert_eq!(
                p.boundary_consumes_control(&boundary, control).unwrap(),
                expected
            );
        }
    }
    // The current B-only selection does not allocate the earlier outer Start.
    // Its end nevertheless remains inside the outer scope in the absolute flow.
    let line = finish(&p, a.view().boundary(), b);
    let selected = line.selected_line();
    assert!(
        !selected
            .request()
            .controls
            .iter()
            .any(|c| c.occurrence == os)
    );
    assert!(
        p.boundary_consumes_control(&selected.request().end, &os)
            .unwrap()
    );
    assert!(
        p.boundary_inside_captured_scope(&selected.request().end, &outer)
            .unwrap()
    );
}

#[test]
fn every_control_kind_uses_real_prefix_incidence_including_materialized_wbr() {
    let wbr_style = style(t::WhiteSpace::Preserve);
    let kinds = vec![
        K::ForcedBreak,
        K::OptionalBreak {
            wrap: t::Wrap::Word,
        },
        K::Wbr {
            style: Box::new(wbr_style.clone()),
        },
        K::Wbr {
            style: Box::new(t::Style {
                word_space_transform: t::WordSpaceTransform::Space,
                ..wbr_style.clone()
            }),
        },
        K::DiscretionaryBreak {
            style: Box::new(wbr_style),
        },
        K::LeftToRightEmbedding,
        K::RightToLeftEmbedding,
        K::LeftToRightOverride,
        K::RightToLeftOverride,
        K::PopDirectionalFormat,
        K::LeftToRightIsolate,
        K::RightToLeftIsolate,
        K::FirstStrongIsolate,
        K::PopDirectionalIsolate,
    ];
    for kind in kinds {
        let p = prepare_capture(
            t::Source::new("AB"),
            block(base()),
            &[(0, 1), (1, 2)],
            &[
                (K::InlineBoundaryStart, 0),
                (kind.clone(), 1),
                (K::InlineBoundaryEnd, 2),
            ],
            &[(0, 2)],
            vec![
                E::EnterScope(1),
                control(0),
                origin(0),
                control(1),
                E::Connection(0),
                origin(1),
                control(2),
                E::ExitScope(1),
            ],
            AHEM,
        );
        let target = p
            .control_views()
            .find(|c| *c.kind() == kind)
            .unwrap()
            .handle();
        let first = p.boundary_views().next().unwrap().handle();
        assert!(!p.boundary_consumes_control(&first, &target).unwrap());
        assert!(
            p.boundary_consumes_control(&end(&p).view().boundary(), &target)
                .unwrap()
        );
        for b in p.boundary_views().map(|v| v.handle()) {
            for c in p.control_views().map(|v| v.handle()) {
                assert_eq!(
                    p.boundary_consumes_control(&b, &c).unwrap(),
                    prefix(&p, &b, &c),
                    "{kind:?}"
                );
            }
        }
    }
}

#[test]
fn deleted_positive_space_keeps_distinct_logical_cuts_at_one_processed_position() {
    let p = capture(
        "X  Y",
        t::WhiteSpace::Collapse,
        &[(0, 2), (2, 3), (3, 4)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 3)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            E::Connection(0),
            origin(1),
            control(1),
            E::ExitScope(1),
            E::Connection(1),
            origin(2),
        ],
    );
    let scope = inline(&p).pop().unwrap();
    let close = local(&p, &scope.exit_control().unwrap());
    let deleted = p
        .unit_views()
        .find(|u| u.source_range() == t::Range::new(2, 3))
        .unwrap();
    assert!(deleted.processed_range().is_empty());
    let early = cut(&p, 2).view().boundary();
    let late = cut(&p, 3).view().boundary();
    assert_eq!(
        early.view().processed_position(),
        late.view().processed_position()
    );
    assert!(!p.boundary_consumes_control(&early, &close).unwrap());
    assert!(p.boundary_inside_captured_scope(&early, &scope).unwrap());
    assert!(p.boundary_consumes_control(&late, &close).unwrap());
    assert!(!p.boundary_inside_captured_scope(&late, &scope).unwrap());
}

#[test]
fn removed_and_empty_origins_do_not_leave_a_closed_zero_output_scope_active() {
    for (value, ranges, mode) in [
        (" X", vec![(0, 1), (1, 2)], t::WhiteSpace::Discard),
        ("X", vec![(0, 0), (0, 1)], t::WhiteSpace::Preserve),
    ] {
        let at = ranges[0].1;
        let p = prepare_capture(
            t::Source::new(value),
            block(t::Style {
                white_space: mode,
                ..base()
            }),
            &ranges,
            &[
                (K::InlineBoundaryStart, 0),
                (K::InlineBoundaryEnd, at),
                (
                    K::OptionalBreak {
                        wrap: t::Wrap::Word,
                    },
                    at,
                ),
            ],
            &[(0, 1)],
            vec![
                E::EnterScope(1),
                control(0),
                origin(0),
                control(1),
                E::ExitScope(1),
                control(2),
                E::Connection(0),
                origin(1),
            ],
            AHEM,
        );
        let scope = inline(&p).pop().unwrap();
        let close = local(&p, &scope.exit_control().unwrap());
        let explicit = p
            .control_views()
            .find(|c| matches!(c.kind(), K::OptionalBreak { .. }))
            .unwrap()
            .handle();
        let boundary = p
            .boundary_views()
            .map(|v| v.handle())
            .find(|b| prefix(&p, b, &explicit))
            .unwrap();
        assert!(p.boundary_consumes_control(&boundary, &close).unwrap());
        assert!(p.boundary_consumes_control(&boundary, &explicit).unwrap());
        assert!(!p.boundary_inside_captured_scope(&boundary, &scope).unwrap());
        if mode == t::WhiteSpace::Discard {
            assert_eq!(
                p.analyzed_source().processed_source().processed_text(),
                &t::Text::from("X")
            );
            assert!(
                p.unit_views()
                    .find(|u| u.source_range() == t::Range::new(0, 1))
                    .unwrap()
                    .processed_range()
                    .is_empty()
            );
        } else {
            assert!(p.origin_views().any(|o| o.local_range().is_empty()));
        }
        assert_eq!(p.analyzed_source().source().text(), &t::Text::from(value));
    }
}

#[test]
fn cross_tcu_logical_scope_membership_does_not_inherit_a_layout_pairing_limit() {
    let font = font::ligature();
    for paired in [false, true] {
        let mut value = input("A\u{301}B", base(), &[(0, 1), (1, 3)]);
        value.scopes.extend([
            t::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: Some(base()),
                boundaries: paired
                    .then_some(t::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
            },
            t::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: Some(base()),
                boundaries: paired
                    .then_some(t::SourceProcessingScopeBoundaries { start: 2, end: 3 }),
            },
        ]);
        if paired {
            value.controls = vec![
                t::ParagraphControl {
                    at: 0,
                    kind: K::InlineBoundaryStart,
                },
                t::ParagraphControl {
                    at: 1,
                    kind: K::InlineBoundaryEnd,
                },
                t::ParagraphControl {
                    at: 1,
                    kind: K::InlineBoundaryStart,
                },
                t::ParagraphControl {
                    at: 3,
                    kind: K::InlineBoundaryEnd,
                },
            ];
        }
        value.events = vec![E::EnterScope(0), E::EnterParagraph(0), E::EnterScope(1)];
        if paired {
            value.events.push(control(0));
        }
        value.events.push(origin(0));
        if paired {
            value.events.push(control(1));
        }
        value
            .events
            .extend([E::ExitScope(1), E::Connection(0), E::EnterScope(2)]);
        if paired {
            value.events.push(control(2));
        }
        value.events.push(origin(1));
        if paired {
            value.events.push(control(3));
        }
        value
            .events
            .extend([E::ExitScope(2), E::ExitParagraph(0), E::ExitScope(0)]);
        let p = prepare_input(value, &font).pop().unwrap();
        assert_eq!(
            p.analyzed_source().graphemes().collect::<Vec<_>>(),
            [t::Range::new(0, 2), t::Range::new(2, 3)]
        );
        let mut scopes = inline(&p);
        scopes.sort_by(|a, b| a.enter_position().partial_cmp(&b.enter_position()).unwrap());
        let acute = p
            .unit_views()
            .find(|u| u.source_range() == t::Range::new(1, 2))
            .unwrap()
            .handle();
        let b = p
            .unit_views()
            .find(|u| u.source_range() == t::Range::new(2, 3))
            .unwrap()
            .handle();
        // A snapped incoming marker can retain an earlier original anchor;
        // the actual whole-TCU opportunity is identified by source receipts.
        let boundary = p
            .opportunity_views()
            .find(|o| {
                o.kind() == t::PreparedBreakKind::Allowed
                    && unit_consumed(&p, &o.boundary(), &acute)
                    && !unit_consumed(&p, &o.boundary(), &b)
            })
            .unwrap()
            .boundary();
        // The trailing whole-TCU cut has consumed A's exit and the acute's
        // actual scope entry, while B in that second scope remains ahead.
        assert!(
            !p.boundary_inside_captured_scope(&boundary, &scopes[0])
                .unwrap()
        );
        assert!(
            p.boundary_inside_captured_scope(&boundary, &scopes[1])
                .unwrap()
        );
        assert!(
            !p.boundary_inside_captured_scope(&end(&p).view().boundary(), &scopes[1])
                .unwrap()
        );
        if paired {
            // Both middle markers originate inside A+acute and must move to a
            // TCU edge; that does not change the original logical scope facts.
            for marker in [
                scopes[0].exit_control().unwrap(),
                scopes[1].entry_control().unwrap(),
            ] {
                assert!(!p.control_is_boundary_exact(&local(&p, &marker)).unwrap());
            }
            for marker in [
                scopes[0].entry_control().unwrap(),
                scopes[1].exit_control().unwrap(),
            ] {
                assert!(p.control_is_boundary_exact(&local(&p, &marker)).unwrap());
            }
        }
        for c in p.control_views().map(|c| c.handle()) {
            assert_eq!(
                p.boundary_consumes_control(&boundary, &c).unwrap(),
                prefix(&p, &boundary, &c)
            );
        }
    }
}

#[test]
fn discarded_prefix_translates_exact_markers_without_turning_them_into_snapping() {
    let p = prepare_capture(
        t::Source::new(" A"),
        block(t::Style {
            white_space: t::WhiteSpace::Discard,
            ..base()
        }),
        &[(0, 1), (1, 2)],
        &[(K::InlineBoundaryStart, 1), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            control(1),
            E::ExitScope(1),
        ],
        AHEM,
    );
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &t::Text::from("A")
    );
    let a = p
        .unit_views()
        .find(|u| u.source_range() == t::Range::new(1, 2))
        .unwrap();
    assert_eq!(a.processed_range().start().get(), 0);
    let scope = inline(&p).pop().unwrap();
    let start = local(&p, &scope.entry_control().unwrap());
    assert_eq!(start.view().source_position().get(), 1);
    assert!(p.control_is_boundary_exact(&start).unwrap());
    assert!(
        p.control_is_boundary_exact(&local(&p, &scope.exit_control().unwrap()))
            .unwrap()
    );
}

#[test]
fn executed_case_transform_preserves_original_scope_and_source_custody() {
    let transformed = t::Style {
        text_transform: t::TextTransform::Set(
            t::TextTransformSet::none().with_case(t::CaseTransform::Uppercase),
        ),
        ..base()
    };
    let p = prepare_capture(
        t::Source::identified("aB", t::Id::from_u64(75073), 4),
        block(transformed),
        &[(0, 1), (1, 2)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 1)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
        AHEM,
    );
    let scope = inline(&p).pop().unwrap();
    let boundary = cut(&p, 1).view().boundary();
    assert!(!p.boundary_inside_captured_scope(&boundary, &scope).unwrap());
    let whole = finish(&p, p.boundary_views().next().unwrap().handle(), end(&p));
    assert_eq!(
        whole
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [35, 36]
    );
    assert_eq!(p.analyzed_source().source().text(), &t::Text::from("aB"));
    assert_eq!(
        p.analyzed_source().source().id(),
        Some(t::Id::from_u64(75073))
    );
}

#[test]
fn control_owned_generated_space_does_not_create_a_scope_at_its_source_anchor() {
    let owner = t::Style {
        word_space_transform: t::WordSpaceTransform::Space,
        ..base()
    };
    let p = prepare_capture(
        t::Source::new("AB"),
        block(owner.clone()),
        &[(0, 1), (1, 2)],
        &[
            (K::InlineBoundaryStart, 0),
            (K::InlineBoundaryEnd, 1),
            (
                K::Wbr {
                    style: Box::new(owner),
                },
                1,
            ),
        ],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            control(2),
            E::Connection(0),
            origin(1),
        ],
        AHEM,
    );
    let scope = inline(&p).pop().unwrap();
    let wbr = p
        .control_views()
        .find(|c| matches!(c.kind(), K::Wbr { .. }))
        .unwrap()
        .handle();
    let emitted = p
        .unit_views()
        .find(|u| u.controls().any(|c| c == wbr))
        .unwrap()
        .handle();
    assert!(emitted.view().origins().next().is_none());
    let b = p
        .unit_views()
        .find(|u| u.source_range() == t::Range::new(1, 2))
        .unwrap()
        .handle();
    // This is an issued mixed boundary, not a claim of soft-wrap permission.
    let boundary = p
        .boundary_views()
        .map(|v| v.handle())
        .find(|at| unit_consumed(&p, at, &emitted) && !unit_consumed(&p, at, &b))
        .unwrap();
    assert!(p.boundary_consumes_control(&boundary, &wbr).unwrap());
    assert!(!p.boundary_inside_captured_scope(&boundary, &scope).unwrap());
    let line = finish(&p, p.boundary_views().next().unwrap().handle(), end(&p));
    let generated = line
        .mapping_views()
        .find(|m| m.kind() == t::PresentationMappingKind::GeneratedSeparator)
        .unwrap();
    assert_eq!(generated.separator_control(), Some(wbr));
    assert!(generated.source_range().is_empty());
    assert_eq!(generated.source_range(), t::Range::new(1, 1));
}

#[test]
fn computed_before_indication_keeps_the_actual_original_scope_exit() {
    let owner = t::Style {
        word_break: t::WordBreak::Normal,
        locale: Some("en-US".into()),
        hyphens: t::Hyphens::Manual,
        hyphenate_character: t::HyphenateCharacter::String(t::Text::from("X")),
        hyphenate_limit_chars: t::HyphenateLimitChars {
            total: t::HyphenationMinimum::Count(0),
            before: t::HyphenationMinimum::Count(0),
            after: t::HyphenationMinimum::Count(0),
        },
        ..base()
    };
    let p = prepare_capture(
        t::Source::new("AB\u{ad}CDE"),
        block(owner),
        &[(0, 3), (3, 6)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 3)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
        AHEM,
    );
    let scope = inline(&p).pop().unwrap();
    let close = local(&p, &scope.exit_control().unwrap());
    let alternative = p
        .alternative_views()
        .find(|a| a.source_range() == Some(t::Range::new(2, 3)))
        .unwrap()
        .handle();
    let offer = p
        .opportunity_views()
        .find(|o| o.alternatives().any(|a| a.handle() == alternative))
        .unwrap()
        .handle();
    let whole = finish(&p, p.boundary_views().next().unwrap().handle(), end(&p));
    let mut r = whole.selected_line().request().clone();
    r.end = offer.view().boundary();
    r.termination = t::PreparedTermination::Ordinary(offer);
    r.participants.truncate(r.end.view().participant_offset());
    r.controls.retain(|c| {
        r.participants
            .contains(&t::PreparedParticipant::Control(c.occurrence.clone()))
    });
    r.alternative = Some(alternative.clone());
    r.context.end = t::LineEnd::Soft;
    r.boundary_context.clear();
    while let Some(occurrence) = p.boundary_context_demand(&r).unwrap() {
        r.boundary_context.push(t::InlineBoundaryContextAllocation {
            occurrence,
            boundary: t::InlineBoundaryAllocation::ZERO,
        });
    }
    let line = p
        .select(r)
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    let selected = line.selected_line();
    let boundary = &selected.request().end;
    assert!(p.boundary_consumes_control(boundary, &close).unwrap());
    assert!(!p.boundary_inside_captured_scope(boundary, &scope).unwrap());
    assert!(
        line.mapping_views()
            .any(|m| m.alternative() == Some(alternative.clone()))
    );
    assert_eq!(alternative.view().text(), &t::Text::from("X"));
}

#[test]
fn context_custody_errors_are_precise_and_real_domain_holes_stay_distinct() {
    let mut value = input("XYZ", base(), &[(0, 1), (1, 2), (2, 3)]);
    value.connections.clear();
    value.boundary_policies.clear();
    value.scopes = vec![
        block(base()),
        t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: Some(base()),
            boundaries: None,
        },
        block(base()),
        t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: Some(base()),
            boundaries: None,
        },
    ];
    value.paragraphs = vec![
        t::SourceProcessingParagraph {
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: t::ParagraphStrut::DefaultFont
        };
        3
    ];
    value.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        origin(0),
        E::ExitParagraph(0),
        E::EnterScope(2),
        E::EnterParagraph(1),
        E::EnterScope(3),
        origin(1),
        E::ExitScope(3),
        E::ExitParagraph(1),
        E::ExitScope(2),
        E::EnterParagraph(2),
        origin(2),
        E::ExitScope(1),
        E::ExitParagraph(2),
        E::ExitScope(0),
    ];
    let paragraphs = prepare_input(value, AHEM);
    let first = &paragraphs[0];
    let child = &paragraphs[1];
    let last = &paragraphs[2];
    let outer = inline(first)
        .into_iter()
        .find(|s| s.domain() == first.captured_paragraph().unwrap().domain())
        .unwrap();
    let child_scope = inline(child)
        .into_iter()
        .find(|s| s.domain() == child.captured_paragraph().unwrap().domain())
        .unwrap();
    assert!(
        first
            .boundary_inside_captured_scope(&end(first).view().boundary(), &outer)
            .unwrap()
    );
    assert!(
        last.boundary_inside_captured_scope(
            &last.boundary_views().next().unwrap().handle(),
            &outer
        )
        .unwrap()
    );
    assert!(
        !last
            .boundary_inside_captured_scope(&end(last).view().boundary(), &outer)
            .unwrap()
    );
    scope_error(
        &first
            .boundary_inside_captured_scope(&end(first).view().boundary(), &child_scope)
            .unwrap_err(),
        t::WrappingScopeQueryError::WrongDomain,
    );
    scope_error(
        &first
            .boundary_inside_captured_scope(
                &end(first).view().boundary(),
                &first.captured_paragraph().unwrap().domain(),
            )
            .unwrap_err(),
        t::WrappingScopeQueryError::NonInlineScope,
    );

    let paired = capture(
        "A B",
        t::WhiteSpace::Preserve,
        &[(0, 2), (2, 3)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            E::EnterScope(1),
            control(0),
            origin(0),
            control(1),
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ],
    );
    let scope = inline(&paired).pop().unwrap();
    let control = local(&paired, &scope.entry_control().unwrap());
    let boundary = end(&paired).view().boundary();
    assert!(matches!(
        reason(
            &paired
                .boundary_consumes_control(&end(first).view().boundary(), &control)
                .unwrap_err()
        ),
        t::ProtocolError::ForeignPreparation {
            kind: t::ProtocolHandleKind::Boundary,
            ..
        }
    ));
    assert!(matches!(
        reason(
            &first
                .boundary_consumes_control(&end(first).view().boundary(), &control)
                .unwrap_err()
        ),
        t::ProtocolError::ForeignPreparation {
            kind: t::ProtocolHandleKind::Control,
            ..
        }
    ));
    assert!(matches!(
        reason(&first.control_is_boundary_exact(&control).unwrap_err()),
        t::ProtocolError::ForeignPreparation {
            kind: t::ProtocolHandleKind::Control,
            ..
        }
    ));
    assert!(matches!(
        reason(
            &paired
                .boundary_inside_captured_scope(&end(first).view().boundary(), &scope)
                .unwrap_err()
        ),
        t::ProtocolError::ForeignPreparation {
            kind: t::ProtocolHandleKind::Boundary,
            ..
        }
    ));
    scope_error(
        &paired
            .boundary_inside_captured_scope(&boundary, &outer)
            .unwrap_err(),
        t::WrappingScopeQueryError::ForeignCapture,
    );

    let flat = t::AnalyzedSource::new(
        t::Source::new("AB"),
        base(),
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
            .boundary_inside_captured_scope(&end(&flat).view().boundary(), &scope)
            .unwrap_err(),
        t::WrappingScopeQueryError::MissingCapture,
    );
}
