//! An atomic content domain belongs to its nominated occurrence, even at a shared anchor.

use super::*;
use std::sync::Arc;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const A0: u32 = 10;
const A1: u32 = 20;
const INLINE_NODES: [u32; 3] = [30, 31, 32];

fn atomic_domain() -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    atomic_domain_kind(text::InlineBoxKind::InFlow)
}

fn atomic_domain_kind(
    kind: text::InlineBoxKind,
) -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    atomic_domain_shape(kind, DomainShape::Atomic)
}

enum DomainShape {
    Atomic,
    SeveralDeferred,
    LaterInlineParent,
    ClosedParent,
    PlainBlock,
    PlainBlockWrapper,
    PlainAtomicConflict,
}

fn atomic_domain_shape(
    kind: text::InlineBoxKind,
    shape: DomainShape,
) -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    let two_domains = matches!(shape, DomainShape::SeveralDeferred);
    let foreign_parent = matches!(shape, DomainShape::LaterInlineParent);
    let closed_parent = matches!(
        shape,
        DomainShape::ClosedParent
            | DomainShape::PlainBlock
            | DomainShape::PlainBlockWrapper
            | DomainShape::PlainAtomicConflict
    );
    let plain_child = matches!(
        shape,
        DomainShape::PlainBlock | DomainShape::PlainBlockWrapper | DomainShape::PlainAtomicConflict
    );
    let wrapper = matches!(shape, DomainShape::PlainBlockWrapper);
    let atomic_conflict = matches!(shape, DomainShape::PlainAtomicConflict);
    let mut base = ahem_style(10.0, 10.0);
    base.locale = Some("ja".into());
    let owner = text::Style {
        word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
        ..base.clone()
    };
    let mut source = Source::new(JAPANESE);
    let whole = text::Range::new(0, JAPANESE.encode_utf16().count());
    source.span(whole, owner.clone());
    // Distinct genuine occurrences have the same anchor and real box metrics.
    for node in [A0, A1] {
        source.inline_box(text::InlineBox::new(
            text::Id::from_u64(u64::from(node)),
            kind,
            0,
            text::Size::new(100.0, 10.0),
        ));
    }
    use text::SourceProcessingEvent as E;
    let domain = |atomic| {
        if plain_child {
            text::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: text::WhiteSpaceTrim::default(),
                base_style: base.clone(),
                used_direction: ParagraphBaseDirection::LeftToRight,
            }
        } else if kind == text::InlineBoxKind::InFlow {
            text::SourceProcessingScope::AtomicBlock {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: text::WhiteSpaceTrim::default(),
                base_style: base.clone(),
                used_direction: ParagraphBaseDirection::LeftToRight,
                atomic,
            }
        } else {
            text::SourceProcessingScope::OutOfFlowBlock {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: text::WhiteSpaceTrim::default(),
                base_style: base.clone(),
                used_direction: ParagraphBaseDirection::LeftToRight,
                atomic,
            }
        }
    };
    let mut scopes = vec![
        text::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: text::SourceProcessingTopology::Complete,
            trim: text::WhiteSpaceTrim::default(),
            base_style: base.clone(),
            used_direction: ParagraphBaseDirection::LeftToRight,
        },
        domain(0),
    ];
    if two_domains {
        scopes.push(domain(1));
    }
    let owner_scope = scopes.len();
    let start = if two_domains { 2 } else { 0 };
    scopes.push(text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: text::WhiteSpaceTrim::default(),
        style: Some(owner),
        boundaries: Some(text::SourceProcessingScopeBoundaries {
            start,
            end: start + 1,
        }),
    });
    let mut controls = Vec::new();
    if two_domains {
        controls.extend([
            text::ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ]);
    }
    controls.extend([
        text::ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: whole.end,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Atomic {
            atomic: 0,
            affinity: text::Affinity::After,
        },
        E::Atomic {
            atomic: 1,
            affinity: text::Affinity::After,
        },
        E::EnterScope(1),
        E::EnterParagraph(1),
    ];
    if two_domains {
        let empty_inline = scopes.len();
        scopes.push(text::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: text::WhiteSpaceTrim::default(),
            style: None,
            boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
        });
        events.extend([
            E::EnterScope(empty_inline),
            E::Control {
                control: 0,
                affinity: text::Affinity::After,
            },
            E::Control {
                control: 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(empty_inline),
            E::ExitParagraph(1),
            E::ExitScope(1),
            E::EnterScope(2),
            E::EnterParagraph(2),
        ]);
    }
    let child_paragraph = if two_domains { 2 } else { 1 };
    let child_domain = if two_domains { 2 } else { 1 };
    events.extend([
        E::EnterScope(owner_scope),
        E::Control {
            control: start,
            affinity: text::Affinity::After,
        },
        E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        },
        E::Control {
            control: start + 1,
            affinity: text::Affinity::After,
        },
        E::ExitScope(owner_scope),
        E::ExitParagraph(child_paragraph),
        E::ExitScope(child_domain),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ]);
    if closed_parent {
        let end = events
            .iter()
            .position(|event| matches!(event, E::ExitParagraph(0)))
            .unwrap();
        let exit = events.remove(end);
        let child = events
            .iter()
            .position(|event| matches!(event, E::EnterScope(1)))
            .unwrap();
        events.insert(child, exit);
    }
    if foreign_parent {
        assert!(!two_domains);
        let inline = scopes.len();
        let boundary = controls.len();
        controls.extend([
            text::ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: whole.end,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ]);
        scopes.push(text::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: text::WhiteSpaceTrim::default(),
            style: None,
            boundaries: Some(text::SourceProcessingScopeBoundaries {
                start: boundary,
                end: boundary + 1,
            }),
        });
        let enter = events
            .iter()
            .position(|e| matches!(e, E::EnterScope(1)))
            .unwrap();
        events.splice(
            enter..enter,
            [
                E::EnterScope(inline),
                E::Control {
                    control: boundary,
                    affinity: text::Affinity::After,
                },
            ],
        );
        let exit = events
            .iter()
            .position(|e| matches!(e, E::ExitScope(1)))
            .unwrap()
            + 1;
        events.splice(
            exit..exit,
            [
                E::Control {
                    control: boundary + 1,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(inline),
            ],
        );
    }
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: whole,
                aggregate: whole,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls,
            boundary_policies: (0..2)
                .flat_map(|atomic| {
                    [
                        text::ParagraphBoundaryDefinition::AtomicBefore(atomic),
                        text::ParagraphBoundaryDefinition::AtomicAfter(atomic),
                    ]
                })
                .map(|definition| text::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            scopes,
            paragraphs: (0..=child_paragraph)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            events,
        })
        .unwrap(),
    );
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let mut prepared: Vec<_> = processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::AnalyzedSource::from_processed_paragraph(
                paragraph,
                text::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let analyzed =
                crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect();
    assert_eq!(prepared.len(), if two_domains { 3 } else { 2 });
    let child = prepared.pop().unwrap();
    let parent = prepared.remove(0);
    let empty = prepared.pop();
    assert_eq!(parent.scopes().len(), 0);
    assert_eq!(child.scopes().len(), 1);
    assert_eq!(
        child
            .virtual_separators()
            .map(|v| v.view().source_position().get())
            .collect::<Vec<_>>(),
        [3]
    );
    let atomics: Vec<_> = parent.atomic_views().map(|v| v.handle()).collect();
    assert_eq!(atomics.len(), 2);
    for (atomic, node) in atomics.iter().zip([A0, A1]) {
        assert_eq!(
            atomic.view().original_box().id(),
            text::Id::from_u64(u64::from(node))
        );
        assert_eq!(atomic.view().original_box().index(), 0);
    }
    let origin = child.origin_views().next().unwrap().handle();
    let controls = child
        .control_views()
        .zip([30, 32])
        .map(|(v, node)| TextControlBinding {
            occurrence: v.handle(),
            node,
        })
        .collect();
    let parent_controls = parent
        .control_views()
        .zip([50, 51])
        .map(|(view, node)| TextControlBinding {
            occurrence: view.handle(),
            node,
        })
        .collect();
    let mut bindings = vec![
        TextParagraphBinding {
            prepared: parent,
            container: 0,
            nodes: if foreign_parent {
                vec![A0, A1, 50, 51]
            } else {
                vec![A0, A1]
            },
            origins: vec![],
            atomics: atomics
                .into_iter()
                .zip([A0, A1])
                .map(|(occurrence, node)| TextAtomicBinding { occurrence, node })
                .collect(),
            controls: parent_controls,
            line_strut: None,
        },
        TextParagraphBinding {
            prepared: child,
            container: if two_domains { A1 } else { A0 },
            nodes: INLINE_NODES.to_vec(),
            origins: vec![TextOriginBinding {
                origin: origin.clone(),
                node: 31,
            }],
            atomics: vec![],
            controls,
            line_strut: None,
        },
    ];
    let mut tree = PublicLayoutTreeOf::new()
        .children(
            0,
            if foreign_parent {
                vec![A0, A1, 50, 51]
            } else {
                vec![A0, A1]
            },
        )
        .children(
            A0,
            if two_domains {
                vec![40, 41]
            } else {
                INLINE_NODES.to_vec()
            },
        )
        .children(
            A1,
            if two_domains {
                INLINE_NODES.to_vec()
            } else {
                vec![]
            },
        )
        .children(31, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(
            31,
            LayoutInputOf::inline_text(TextParticipation::new(origin)),
        );
    for node in [A0, A1] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::InlineBlock,
                position: if kind == text::InlineBoxKind::OutOfFlow {
                    Position::Absolute
                } else {
                    Position::Relative
                },
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::default()
            },
        );
    }
    for (node, kind) in [
        (30, InlineBoundaryKind::Start),
        (32, InlineBoundaryKind::End),
    ] {
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                kind,
                InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, 8.0, 10.0)
                    .unwrap(),
                crate::InlineBoundaryEdgeOf::ZERO,
            )),
        );
    }
    if foreign_parent {
        for (node, kind) in [
            (50, InlineBoundaryKind::Start),
            (51, InlineBoundaryKind::End),
        ] {
            tree = tree.children(node, []).input(
                node,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                    kind,
                    InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, 8.0, 10.0)
                        .unwrap(),
                    crate::InlineBoundaryEdgeOf::ZERO,
                )),
            );
        }
    }
    if let Some(prepared) = empty {
        let controls = prepared
            .control_views()
            .zip([40, 41])
            .map(|(view, node)| TextControlBinding {
                occurrence: view.handle(),
                node,
            })
            .collect();
        bindings.insert(
            1,
            TextParagraphBinding {
                prepared,
                container: A0,
                nodes: vec![40, 41],
                origins: vec![],
                atomics: vec![],
                controls,
                line_strut: None,
            },
        );
        for (node, kind) in [
            (40, InlineBoundaryKind::Start),
            (41, InlineBoundaryKind::End),
        ] {
            tree = tree.children(node, []).input(
                node,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                    kind,
                    InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, 8.0, 10.0)
                        .unwrap(),
                    crate::InlineBoundaryEdgeOf::ZERO,
                )),
            );
        }
    }
    if plain_child && !atomic_conflict {
        assert!(!two_domains && closed_parent);
        bindings[1].container = 50;
        tree = tree
            .children(A0, [])
            .children(0, [A0, A1, 50])
            .children(50, INLINE_NODES)
            .style(
                50,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            );
    }
    if wrapper {
        tree = tree.children(0, [A0, A1, 90]).children(90, [50]).style(
            90,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    }
    (tree, bindings)
}

#[test]
fn earlier_same_anchor_atomic_accepts_its_own_content_domain_and_computes_phrase_geometry() {
    let (tree, bindings) = atomic_domain();
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings.clone(),
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 1);
    let InlineFragmentOwner::Scope(owner) = generated[0].owner() else {
        unreachable!()
    };
    assert_eq!(owner.scope, bindings[1].prepared.scopes().next().unwrap());
    assert_eq!(
        owner.association,
        TextScopeAssociation::Inline { start: 30, end: 32 }
    );
    let rect = generated[0].fragment().rect().unwrap();
    // Real Ahem advances: three 10px original units precede the generated space.
    assert_eq!(rect.origin().x, 30.0);
    assert_eq!(rect.size().width, 10.0);
    assert!(batch.placed_text().iter().any(|placed| {
        placed.layout().source().text().code_units() == JAPANESE.encode_utf16().collect::<Vec<_>>()
    }));
}

#[test]
fn later_same_anchor_atomic_rejects_the_earlier_atomics_content_domain() {
    let (tree, mut bindings) = atomic_domain();
    // Preserve the exact source capture, preparations, occurrence bindings,
    // endpoint controls and parent run. Move only the actual child container.
    let wrong_tree = tree.children(A0, []).children(A1, INLINE_NODES);
    bindings[1].container = A1;
    let Err(error) = LayoutFlowInput::try_new(
        &wrong_tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("A0's captured atomic content domain must not be accepted under A1")
    };
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
                TextBindingError::ScopeAssociation
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn out_of_flow_content_domain_keeps_its_exact_same_anchor_atomic_custody() {
    let (tree, bindings) = atomic_domain_kind(text::InlineBoxKind::OutOfFlow);
    LayoutFlowInput::try_new(
        &tree,
        0,
        bindings.clone(),
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let wrong_tree = tree.children(A0, []).children(A1, INLINE_NODES);
    let mut wrong = bindings;
    wrong[1].container = A1;
    let Err(error) = LayoutFlowInput::try_new(
        &wrong_tree,
        0,
        wrong,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("out-of-flow atomic identity cannot be donated");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeAssociation
        ))
    ));
}

#[test]
fn parent_atomic_order_is_not_replaced_by_content_domain_schedule() {
    let (tree, mut bindings) = atomic_domain();
    let tree = tree.children(0, [A1, A0]);
    bindings[0].nodes = vec![A1, A0];
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("the genuine parent atomic participants retain their own order");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantOrder
        ))
    ));
}

#[test]
fn deferred_child_content_cannot_reverse_its_actual_boundary_controls() {
    let (tree, mut bindings) = atomic_domain();
    let tree = tree.children(A0, [32, 31, 30]);
    bindings[1].nodes = vec![32, 31, 30];
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("deferred scheduling does not change child control custody or order");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantOrder
        ))
    ));
}

#[test]
fn several_deferred_atomic_content_domains_keep_independent_actual_flows() {
    for kind in [text::InlineBoxKind::InFlow, text::InlineBoxKind::OutOfFlow] {
        let (tree, bindings) = atomic_domain_shape(kind, DomainShape::SeveralDeferred);
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[1].prepared.origin_views().len(), 0);
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            bindings.clone(),
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
        let generated: Vec<_> = batch
            .final_inline_fragments()
            .iter()
            .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
            .collect();
        assert_eq!(generated.len(), 1);
        let InlineFragmentOwner::Scope(owner) = generated[0].owner() else {
            unreachable!()
        };
        assert_eq!(owner.scope, bindings[2].prepared.scopes().next().unwrap());
        assert_eq!(
            owner.association,
            TextScopeAssociation::Inline { start: 30, end: 32 }
        );
        assert_eq!(generated[0].fragment().rect().unwrap().size().width, 10.0);
    }
}

#[test]
fn deferred_entry_cannot_donate_a_later_inline_parent_to_an_earlier_atomic() {
    let (tree, bindings) =
        atomic_domain_shape(text::InlineBoxKind::InFlow, DomainShape::LaterInlineParent);
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("the nominated atomic must belong to its retained parent inline ancestry");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeAssociation
        ))
    ));
}

#[test]
fn deferred_content_can_follow_its_nominated_atomics_parent_paragraph_exit() {
    let (tree, bindings) =
        atomic_domain_shape(text::InlineBoxKind::InFlow, DomainShape::ClosedParent);
    let parent = bindings[0].prepared.captured_paragraph().unwrap();
    let child = bindings[1].prepared.captured_paragraph().unwrap().domain();
    assert!(parent.exit_position() < child.enter_position());
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    assert_eq!(
        batch
            .final_inline_fragments()
            .iter()
            .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
            .count(),
        1
    );
}

#[test]
fn plain_block_subtree_root_keeps_its_own_flow_without_an_outside_parent_association() {
    let (tree, mut bindings) =
        atomic_domain_shape(text::InlineBoxKind::InFlow, DomainShape::PlainBlock);
    let child = bindings.remove(1);
    let capture = LayoutFlowInput::try_new(
        &tree,
        50,
        vec![child],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 50, request(500.0)).unwrap();
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].fragment().rect().unwrap().origin().x, 30.0);
}

#[test]
fn atomic_content_root_cannot_guess_its_missing_nominated_atomic_binding() {
    let (tree, mut bindings) = atomic_domain();
    let child = bindings.remove(1);
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        A0,
        vec![child],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("an atomic content root still requires authentic nominated custody");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeAssociation
        ))
    ));
}

#[test]
fn plain_block_custody_allows_unmapped_layout_only_wrappers() {
    let (tree, bindings) =
        atomic_domain_shape(text::InlineBoxKind::InFlow, DomainShape::PlainBlockWrapper);
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 1);
    let location = |node| {
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
            .location
    };
    // Baseline-less10px atomics contribute10px before the baseline; the
    // genuine Ahem default-font strut adds its2px descent:10+2=12px.
    let parent = batch
        .placed_text()
        .iter()
        .find(|p| p.container() == 0)
        .unwrap();
    assert_eq!(parent.layout().lines().len(), 1);
    assert_eq!(parent.layout().lines()[0].baseline(), 10.0);
    assert_eq!(parent.layout().lines()[0].bounds().size.height, 12.0);
    assert_eq!(location(90), Point::new(0.0, 12.0));
    assert_eq!(location(50), Point::new(0.0, 0.0));
    // Fragment coordinates are local to the actual paragraph container50.

    assert_eq!(
        generated[0].fragment().rect().unwrap().origin(),
        Point::new(30.0, 0.0)
    );
}

#[test]
fn plain_block_definition_cannot_reuse_an_actual_prepared_atomic_container() {
    let (tree, bindings) = atomic_domain_shape(
        text::InlineBoxKind::InFlow,
        DomainShape::PlainAtomicConflict,
    );
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("a plain Block cannot claim an explicitly bound atomic role");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeAssociation
        ))
    ));
}
