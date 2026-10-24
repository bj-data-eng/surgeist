//! Public capture identity and paragraph-incidence contracts.

use std::{collections::HashSet, sync::Arc};
use surgeist_text::*;

#[path = "captured_scope/flow.rs"]
mod flow;

const JAPANESE: &str = "今日は天気です。";
const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn style(operation: WordSpaceTransform) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        locale: Some("ja".into()),
        wrap: Wrap::Word,
        word_space_transform: operation,
        ..Style::default()
    }
}

fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
}

fn block(topology: SourceProcessingTopology) -> SourceProcessingScope {
    SourceProcessingScope::Block {
        topology,
        trim: WhiteSpaceTrim::default(),
        base_style: style(WordSpaceTransform::None),
        used_direction: ParagraphBaseDirection::LeftToRight,
    }
}

fn origin(source: &Source, start: usize, end: usize) -> ParagraphOrigin {
    ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: Range::new(start, end),
        aggregate: Range::new(start, end),
        kind: ParagraphOriginKind::Authored,
    }
}

fn paragraph() -> SourceProcessingParagraph {
    SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    }
}

fn prepare(processed: &Arc<ProcessedSource>) -> Vec<PreparedText> {
    let mut system = system();
    processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = AnalyzedSource::from_processed_paragraph(paragraph).unwrap();
            let policies = analyzed
                .wrapping_relations()
                .filter(|relation| relation.ancestor_wrap().is_none())
                .map(|relation| WrappingPolicy {
                    relation: relation.handle(),
                    ancestor_wrap: Wrap::Word,
                })
                .collect();
            let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
            system
                .prepare(&analyzed, PreparationOptions::default())
                .unwrap()
        })
        .collect()
}

fn continuation(enabled: bool, known: bool) -> Arc<ProcessedSource> {
    use SourceProcessingEvent as E;
    let n = JAPANESE.encode_utf16().count();
    let end = n * 2 + 2;
    let mut source = Source::new(format!("{JAPANESE}X{JAPANESE}Y"));
    source.span(
        Range::new(0, end),
        style(if enabled {
            WordSpaceTransform::SpaceAutoPhrase
        } else {
            WordSpaceTransform::None
        }),
    );
    let occurrence = |origin| E::Origin {
        origin,
        affinity: Affinity::After,
    };
    let control = |control| E::Control {
        control,
        affinity: Affinity::After,
    };
    Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: source.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![
                origin(&source, 0, n),
                origin(&source, n, n + 1),
                origin(&source, n + 1, n * 2 + 1),
                origin(&source, n * 2 + 1, end),
            ],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            controls: vec![
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphControl {
                    at: end,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            scopes: vec![
                block(if known {
                    SourceProcessingTopology::Complete
                } else {
                    SourceProcessingTopology::Unknown
                }),
                SourceProcessingScope::Inline {
                    trim: WhiteSpaceTrim::default(),
                    style: known.then(|| {
                        style(if enabled {
                            WordSpaceTransform::Space
                        } else {
                            WordSpaceTransform::None
                        })
                    }),
                    boundaries: known
                        .then_some(SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
                block(SourceProcessingTopology::Complete),
                block(SourceProcessingTopology::Complete),
            ],
            paragraphs: vec![paragraph(); 5],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                occurrence(0),
                E::ExitParagraph(0),
                E::EnterScope(2),
                E::EnterParagraph(1),
                occurrence(1),
                E::ExitParagraph(1),
                E::ExitScope(2),
                E::EnterParagraph(2),
                occurrence(2),
                E::ExitParagraph(2),
                E::EnterScope(3),
                E::EnterParagraph(3),
                occurrence(3),
                E::ExitParagraph(3),
                E::ExitScope(3),
                E::EnterParagraph(4),
                control(1),
                E::ExitScope(1),
                E::ExitParagraph(4),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    )
}

fn inline(prepared: &PreparedText) -> CapturedScope {
    prepared
        .captured_paragraph()
        .unwrap()
        .scope_intersections()
        .map(|intersection| intersection.scope())
        .find(|scope| scope.role() == ScopeRole::Inline)
        .unwrap()
}

#[test]
fn independent_preparations_share_capture_but_keep_prepared_identity() {
    let processed = continuation(true, true);
    let first = prepare(&processed);
    let second = prepare(&processed);
    assert_ne!(first[0], second[0]);
    let owner = first[0].scopes().next().unwrap();
    let other = second[0].scopes().next().unwrap();
    assert_ne!(owner, other);
    assert_eq!(owner.captured_scope(), other.captured_scope());
    assert_eq!(
        first[0].captured_paragraph(),
        second[0].captured_paragraph()
    );
    let first_control = first[0].control_views().next().unwrap().handle();
    let other_control = second[0].control_views().next().unwrap().handle();
    assert_ne!(first_control, other_control);
    assert_eq!(
        first_control.captured_control(),
        other_control.captured_control()
    );
    let third = prepare(&continuation(true, true));
    let foreign = third[0].scopes().next().unwrap().captured_scope();
    assert_ne!(owner.captured_scope(), foreign);
    assert_eq!(
        HashSet::from([
            owner.captured_scope(),
            other.captured_scope(),
            foreign.clone()
        ])
        .len(),
        2
    );
    assert_eq!(
        owner
            .captured_scope()
            .enter_position()
            .partial_cmp(&foreign.enter_position()),
        None
    );
    assert!(
        !owner
            .captured_scope()
            .enter_position()
            .same_capture(&foreign.enter_position())
    );
}

#[test]
fn middle_and_endpoint_only_paragraphs_retain_the_complete_global_owner() {
    let prepared = prepare(&continuation(true, true));
    let owner = prepared[0].scopes().next().unwrap();
    let canonical = owner.captured_scope();
    assert_eq!(
        canonical,
        prepared[2].scopes().next().unwrap().captured_scope()
    );
    assert!(owner.view().entry_control().is_some());
    assert!(owner.view().exit_control().is_none());
    let middle = prepared[2].scopes().next().unwrap();
    assert!(middle.view().entry_control().is_none());
    assert!(middle.view().exit_control().is_none());
    let start = canonical.entry_control().unwrap();
    let end = canonical.exit_control().unwrap();
    assert_eq!(start.paragraph(), prepared[0].captured_paragraph().unwrap());
    assert_eq!(end.paragraph(), prepared[4].captured_paragraph().unwrap());
    assert_eq!(start.kind(), &ParagraphControlKind::InlineBoundaryStart);
    assert_eq!(end.kind(), &ParagraphControlKind::InlineBoundaryEnd);
    assert_eq!(
        end,
        prepared[4]
            .control_views()
            .next()
            .unwrap()
            .handle()
            .captured_control()
            .unwrap()
    );
    assert_eq!(prepared[4].origin_views().count(), 0);
    assert_eq!(prepared[4].virtual_separators().count(), 0);
    assert_eq!(
        prepared[0]
            .virtual_separators()
            .map(|v| v.view().source_position().get())
            .collect::<Vec<_>>(),
        vec![3]
    );
    assert_eq!(
        prepared[2]
            .virtual_separators()
            .map(|v| v.view().source_position().get())
            .collect::<Vec<_>>(),
        vec![12]
    );
    let paragraph = prepared[2].captured_paragraph().unwrap();
    let intersection = paragraph
        .scope_intersections()
        .find(|item| item.scope() == canonical)
        .unwrap();
    assert_eq!(intersection.paragraph(), paragraph);
    assert_eq!(intersection.enter_position(), paragraph.enter_position());
    assert_eq!(intersection.exit_position(), paragraph.exit_position());
    assert!(canonical.enter_position() < intersection.enter_position());
    assert!(intersection.exit_position() < canonical.exit_position());
    let child = prepared[1].captured_paragraph().unwrap();
    assert_ne!(child.domain(), canonical.domain());
    assert_eq!(child.domain().parent(), Some(canonical.clone()));
    assert!(
        child
            .scope_intersections()
            .any(|item| item.scope() == canonical)
    );
}

#[test]
fn none_and_dormant_unknown_facts_remain_observable_without_new_demands() {
    let prepared = prepare(&continuation(false, true));
    assert!(
        prepared
            .iter()
            .all(|paragraph| paragraph.scopes().len() == 0)
    );
    let owner = inline(&prepared[2]);
    assert!(owner.entry_control().is_some());
    assert!(owner.exit_control().is_some());
    assert_eq!(
        owner.style().unwrap().word_space_transform,
        WordSpaceTransform::None
    );
    let unknown = prepare(&continuation(false, false));
    let dormant = inline(&unknown[2]);
    assert!(dormant.style().is_none());
    assert!(dormant.entry_control().is_none());
    assert!(dormant.exit_control().is_none());
    assert_eq!(dormant.domain_topology(), SourceProcessingTopology::Unknown);
    assert!(dormant.domain_definition().is_none());
    assert_eq!(
        dormant.domain().domain_definition(),
        Some(CapturedDomainDefinition::Block)
    );
    assert!(
        unknown
            .iter()
            .all(|paragraph| paragraph.scopes().len() == 0)
    );
}

#[test]
fn equal_source_anchors_preserve_distinct_control_and_original_event_order() {
    let prepared = prepare(&continuation(false, true));
    let control = prepared[0].control_views().next().unwrap();
    let original = prepared[0].origin_views().next().unwrap();
    assert_eq!(
        control.source_position().get(),
        original.aggregate_range().start
    );
    assert!(control.capture_position().unwrap() < original.capture_position().unwrap());
    assert_eq!(
        control.capture_position(),
        control
            .handle()
            .captured_control()
            .map(|control| control.event_position())
    );
    let last = prepared[4].control_views().next().unwrap();
    assert!(original.capture_position().unwrap() < last.capture_position().unwrap());
    assert_eq!(
        last.handle().captured_control().unwrap().source_position(),
        last.source_position()
    );
}

#[test]
fn same_anchor_atomic_domains_retain_the_exact_nominated_occurrence() {
    use SourceProcessingEvent as E;
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let mut source = Source::new("X");
        for id in [10, 20] {
            source.inline_box(InlineBox::new(
                Id::from_u64(id),
                kind,
                0,
                Size::new(10.0, 10.0),
            ));
        }
        let child = match kind {
            InlineBoxKind::InFlow => SourceProcessingScope::AtomicBlock {
                topology: SourceProcessingTopology::Unknown,
                trim: WhiteSpaceTrim::default(),
                base_style: style(WordSpaceTransform::None),
                used_direction: ParagraphBaseDirection::LeftToRight,
                atomic: 0,
            },
            InlineBoxKind::OutOfFlow => SourceProcessingScope::OutOfFlowBlock {
                topology: SourceProcessingTopology::Unknown,
                trim: WhiteSpaceTrim::default(),
                base_style: style(WordSpaceTransform::None),
                used_direction: ParagraphBaseDirection::LeftToRight,
                atomic: 0,
            },
        };
        let processed = Arc::new(
            process_contextual_source(ContextualSourceInput {
                source: source.clone(),
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                envelope: SourceProcessingEnvelope::Complete,
                origins: vec![origin(&source, 0, 1)],
                connections: vec![],
                controls: vec![],
                boundary_policies: if kind == InlineBoxKind::InFlow {
                    (0..2)
                        .flat_map(|atomic| {
                            [
                                ParagraphBoundaryDefinition::AtomicBefore(atomic),
                                ParagraphBoundaryDefinition::AtomicAfter(atomic),
                            ]
                        })
                        .map(|definition| ParagraphBoundaryPolicy {
                            definition,
                            preceding_wrap: Wrap::Word,
                            following_wrap: Wrap::Word,
                            ancestor_wrap: Wrap::Word,
                        })
                        .collect()
                } else {
                    vec![]
                },
                replacements: vec![],
                scopes: vec![block(SourceProcessingTopology::Unknown), child],
                paragraphs: vec![paragraph(); 2],
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Atomic {
                        atomic: 0,
                        affinity: Affinity::After,
                    },
                    E::Atomic {
                        atomic: 1,
                        affinity: Affinity::After,
                    },
                    E::EnterScope(1),
                    E::EnterParagraph(1),
                    E::Origin {
                        origin: 0,
                        affinity: Affinity::After,
                    },
                    E::ExitParagraph(1),
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let prepared = prepare(&processed);
        let atomics: Vec<_> = prepared[0]
            .atomic_views()
            .map(|view| view.capture_position().unwrap())
            .collect();
        assert_ne!(atomics[0], atomics[1]);
        assert!(atomics[0] < atomics[1]);
        let domain = prepared[1].captured_paragraph().unwrap().domain();
        let expected = match kind {
            InlineBoxKind::InFlow => CapturedDomainDefinition::AtomicBlock {
                atomic: atomics[0].clone(),
            },
            InlineBoxKind::OutOfFlow => CapturedDomainDefinition::OutOfFlowBlock {
                atomic: atomics[0].clone(),
            },
        };
        assert_eq!(domain.domain_definition(), Some(expected));
        assert!(atomics[1] < domain.enter_position());
    }
}

#[test]
fn empty_nested_owners_keep_distinct_same_anchor_endpoints_without_emitting() {
    use SourceProcessingEvent as E;
    let control = |control| E::Control {
        control,
        affinity: Affinity::After,
    };
    let inline = |start, end| SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: Some(SourceProcessingScopeBoundaries { start, end }),
    };
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source: Source::new(""),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins: vec![],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            controls: [
                ParagraphControlKind::InlineBoundaryStart,
                ParagraphControlKind::InlineBoundaryStart,
                ParagraphControlKind::InlineBoundaryEnd,
                ParagraphControlKind::InlineBoundaryEnd,
            ]
            .into_iter()
            .map(|kind| ParagraphControl { at: 0, kind })
            .collect(),
            scopes: vec![
                block(SourceProcessingTopology::Unknown),
                inline(0, 3),
                inline(1, 2),
            ],
            paragraphs: vec![paragraph()],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                E::EnterScope(2),
                control(1),
                control(2),
                E::ExitScope(2),
                control(3),
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let prepared = prepare(&processed);
    assert_eq!(prepared[0].origin_views().count(), 0);
    assert_eq!(prepared[0].scopes().len(), 0);
    let controls: Vec<_> = prepared[0]
        .control_views()
        .map(|view| view.handle().captured_control().unwrap())
        .collect();
    assert_eq!(controls.iter().cloned().collect::<HashSet<_>>().len(), 4);
    assert!(
        controls
            .iter()
            .all(|control| control.source_position().get() == 0)
    );
    assert!(
        controls
            .windows(2)
            .all(|pair| pair[0].event_position() < pair[1].event_position())
    );
    let paragraph = prepared[0].captured_paragraph().unwrap();
    let scopes: Vec<_> = paragraph
        .scope_intersections()
        .map(|item| item.scope())
        .filter(|scope| scope.role() == ScopeRole::Inline)
        .collect();
    let outer = scopes
        .iter()
        .find(|scope| scope.entry_control() == Some(controls[0].clone()))
        .unwrap();
    let inner = scopes
        .iter()
        .find(|scope| scope.entry_control() == Some(controls[1].clone()))
        .unwrap();
    assert_eq!(inner.parent().as_ref(), Some(outer));
    assert_eq!(inner.exit_control(), Some(controls[2].clone()));
    assert_eq!(outer.exit_control(), Some(controls[3].clone()));
    assert!(outer.style().is_none());
    assert!(inner.style().is_none());
}

#[test]
fn standalone_preparation_does_not_acquire_contextual_authority() {
    let analyzed = AnalyzedSource::new(Source::new("X"), style(WordSpaceTransform::None)).unwrap();
    let prepared = system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert!(prepared.captured_paragraph().is_none());
    assert!(
        prepared
            .origin_views()
            .all(|view| view.capture_position().is_none())
    );
}
