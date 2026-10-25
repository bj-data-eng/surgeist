//! A genuine inline scope continues around a nested in-flow block.

use super::*;
use std::sync::Arc;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn continuation(
    enabled: bool,
    captured_boundaries: bool,
) -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    let mut base = ahem_style(10.0, 10.0);
    base.locale = Some("ja".into());
    let authored = text::Style {
        word_space_transform: if enabled {
            text::WordSpaceTransform::SpaceAutoPhrase
        } else {
            text::WordSpaceTransform::None
        },
        ..base.clone()
    };
    let owner = text::Style {
        word_space_transform: if enabled {
            text::WordSpaceTransform::Space
        } else {
            text::WordSpaceTransform::None
        },
        ..base.clone()
    };
    let mut source = Source::new(format!("{JAPANESE}X{JAPANESE}"));
    let n = JAPANESE.encode_utf16().count();
    let end = 2 * n + 1;
    source.span(text::Range::new(0, end), authored);
    let origin = |start, end| text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(start, end),
        aggregate: text::Range::new(start, end),
        kind: text::ParagraphOriginKind::Authored,
    };
    let block = || text::SourceProcessingScope::Block {
        topology: text::SourceProcessingTopology::Complete,
        trim: text::WhiteSpaceTrim::default(),
        base_style: base.clone(),
        used_direction: ParagraphBaseDirection::LeftToRight,
    };
    use text::SourceProcessingEvent as E;
    let occurrence = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![origin(0, n), origin(n, n + 1), origin(n + 1, end)],
            connections: vec![],
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: end,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![
                block(),
                text::SourceProcessingScope::Inline {
                    trim: text::WhiteSpaceTrim::default(),
                    style: Some(owner),
                    boundaries: captured_boundaries
                        .then_some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
                block(),
            ],
            paragraphs: (0..3)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // The sole genuine Start is in p0 and the sole genuine End in p2.
            // The child block creates its own p1 without closing the inline.
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
                control(1),
                E::ExitScope(1),
                E::ExitParagraph(2),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared: Vec<_> = processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::AnalyzedSource::from_processed_paragraph(paragraph).unwrap();
            let analyzed =
                crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect();
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3, 4, 5])
        .children(3, [6])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    for (node, kind) in [(1, InlineBoundaryKind::Start), (5, InlineBoundaryKind::End)] {
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                kind,
                InlineMetricsOf::try_new(8.0, 10.0).unwrap(),
                crate::InlineBoundaryEdgeOf::ZERO,
            )),
        );
    }
    let bindings = prepared
        .into_iter()
        .enumerate()
        .map(|(index, prepared)| {
            let (container, nodes, origin_node, control_node) = match index {
                0 => (0, vec![1, 2], 2, 1),
                1 => (3, vec![6], 6, 0),
                2 => (0, vec![4, 5], 4, 5),
                _ => unreachable!(),
            };
            let origin = prepared.origin_views().next().unwrap().handle();
            tree.insert_children(origin_node, []);
            tree.insert_input(
                origin_node,
                LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
            );
            // Published Japanese literal: 今日は | 天気です。; X has no proposal.
            let expected = if enabled && index != 1 {
                vec![if index == 0 { 3 } else { n + 4 }]
            } else {
                vec![]
            };
            assert_eq!(
                prepared
                    .virtual_separators()
                    .map(|v| v.view().source_position().get())
                    .collect::<Vec<_>>(),
                expected
            );
            let controls = prepared
                .control_views()
                .map(|view| TextControlBinding {
                    occurrence: view.handle(),
                    node: control_node,
                })
                .collect();
            TextParagraphBinding {
                prepared,
                container,
                nodes,
                origins: vec![TextOriginBinding {
                    origin,
                    node: origin_node,
                }],
                atomics: vec![],
                controls,
                line_strut: None,
            }
        })
        .collect();
    (tree, bindings)
}

#[test]
fn complete_inline_scope_around_real_block_without_generated_output_is_accepted() {
    let (tree, bindings) = continuation(false, true);
    let capture = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 3);
    assert!(
        batch
            .final_inline_fragments()
            .iter()
            .all(|entry| matches!(entry.owner(), InlineFragmentOwner::Node(_)))
    );
    assert_eq!(
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 3)
            .unwrap()
            .output()
            .size
            .height,
        10.0
    );
}

#[test]
fn complete_inline_scope_keeps_generated_output_before_and_after_real_block() {
    let (tree, bindings) = continuation(true, true);
    let capture = LayoutFlowInput::try_new(&tree, 0, bindings.clone(), Vec::new()).unwrap_or_else(|error| {
        panic!("the complete global inline pair [1,5] must support its actual paragraph projections: {error:?}")
    });
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 3);
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 2);
    let mut positions = Vec::new();
    for entry in generated {
        let InlineFragmentOwner::Scope(owner) = entry.owner() else {
            unreachable!()
        };
        assert_eq!(
            owner.association,
            TextScopeAssociation::Inline { start: 1, end: 5 }
        );
        let fragment = entry.fragment();
        let view = fragment.finalized_fragment().view();
        let text::FinalizedFragmentOwner::VirtualSeparator(occurrence) = view.owner() else {
            panic!("real virtual separator authority")
        };
        let at = occurrence.view().source_position().get();
        let paragraph = if at == 3 {
            &bindings[0]
        } else {
            assert_eq!(at, 12);
            &bindings[2]
        };
        assert_eq!(occurrence.prepared_text(), paragraph.prepared);
        assert_eq!(view.origin(), None);
        assert_eq!(view.local_range(), None);
        assert_eq!(view.aggregate_range(), None);
        positions.push((at, fragment.rect().unwrap()));
    }
    positions.sort_by_key(|(at, _)| *at);
    // Each source glyph and genuine generated SPACE advances10px. The child
    // block contributes its real10px line between the two parent10px lines.
    assert_eq!(
        positions,
        vec![
            (
                3,
                ScrollRectOf::try_new(Point::new(30.0, 0.0), Size::new(10.0, 10.0)).unwrap()
            ),
            (
                12,
                ScrollRectOf::try_new(Point::new(30.0, 20.0), Size::new(10.0, 10.0)).unwrap()
            ),
        ]
    );
    assert!(
        batch
            .final_entries()
            .iter()
            .filter(|entry| [1, 5].contains(&entry.node()))
            .all(|entry| entry.output().size.width == 0.0)
    );
}

#[test]
fn missing_global_inline_boundary_capture_remains_a_refusal() {
    let (tree, bindings) = continuation(true, false);
    let Err(error) = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()) else {
        panic!("an uncaptured global pair must not be inferred from nearby controls")
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeBoundaryCapture
        ))
    ));
}

/// Real paragraph cuts, with optional endpoint-only paragraphs and an outer
/// nonemitting inline ancestor. Every control remains one authored occurrence.
fn several_segments(
    count: usize,
    endpoint_only: bool,
    nested: bool,
) -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    segmented_fixture(count, endpoint_only, nested, false)
}

pub(super) fn empty_block_capture() -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    segmented_fixture(2, false, false, true)
}

fn segmented_fixture(
    count: usize,
    endpoint_only: bool,
    nested: bool,
    empty_blocks: bool,
) -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    let mut base = ahem_style(10.0, 10.0);
    base.locale = Some("ja".into());
    let authored = text::Style {
        word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
        ..base.clone()
    };
    let owner = text::Style {
        word_space_transform: text::WordSpaceTransform::Space,
        ..base.clone()
    };
    let mut original = (0..count)
        .map(|_| JAPANESE)
        .collect::<Vec<_>>()
        .join(if empty_blocks { "" } else { "X" });
    if endpoint_only {
        original = format!("X{original}X");
    }
    let mut source = Source::new(original);
    let end = source.text().len();
    source.span(text::Range::new(0, end), authored);
    let block = || text::SourceProcessingScope::Block {
        topology: text::SourceProcessingTopology::Complete,
        trim: text::WhiteSpaceTrim::default(),
        base_style: base.clone(),
        used_direction: ParagraphBaseDirection::LeftToRight,
    };
    let mut scopes = vec![block()];
    let mut controls = Vec::new();
    let mut boundary_nodes = Vec::new();
    if nested {
        controls.push(text::ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        });
        boundary_nodes.push(100);
        scopes.push(text::SourceProcessingScope::Inline {
            trim: text::WhiteSpaceTrim::default(),
            style: None,
            boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 3 }),
        });
    }
    let owner_scope = scopes.len();
    let start = controls.len();
    controls.push(text::ParagraphControl {
        at: 0,
        kind: ParagraphControlKind::InlineBoundaryStart,
    });
    boundary_nodes.push(101);
    controls.push(text::ParagraphControl {
        at: end,
        kind: ParagraphControlKind::InlineBoundaryEnd,
    });
    boundary_nodes.push(102);
    if nested {
        controls.push(text::ParagraphControl {
            at: end,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        });
        boundary_nodes.push(103);
    }
    scopes.push(text::SourceProcessingScope::Inline {
        trim: text::WhiteSpaceTrim::default(),
        style: Some(owner),
        boundaries: Some(text::SourceProcessingScopeBoundaries {
            start,
            end: start + 1,
        }),
    });
    use text::SourceProcessingEvent as E;
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if nested {
        events.extend([
            E::EnterScope(1),
            E::Control {
                control: 0,
                affinity: text::Affinity::After,
            },
        ]);
    }
    events.extend([
        E::EnterScope(owner_scope),
        E::Control {
            control: start,
            affinity: text::Affinity::After,
        },
    ]);
    let mut runs: Vec<(u32, Vec<u32>)> = vec![(0, boundary_nodes[..=start].to_vec())];
    let mut origins = Vec::new();
    let mut origin_nodes = Vec::new();
    let mut root_children = boundary_nodes[..=start].to_vec();
    let mut child_runs = Vec::new();
    let mut paragraph = 0;
    let mut cursor = 0;
    let mut next_block = 0u32;
    macro_rules! push_block {
        () => {{
            events.push(E::ExitParagraph(paragraph));
            let scope = scopes.len();
            scopes.push(block());
            paragraph += 1;
            events.extend([E::EnterScope(scope), E::EnterParagraph(paragraph)]);
            let container = 200 + next_block;
            let child = 400 + next_block;
            next_block += 1;
            if !empty_blocks {
                origins.push(text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(cursor, cursor + 1),
                    aggregate: text::Range::new(cursor, cursor + 1),
                    kind: text::ParagraphOriginKind::Authored,
                });
                events.push(E::Origin {
                    origin: origins.len() - 1,
                    affinity: text::Affinity::After,
                });
                origin_nodes.push(child);
            }
            events.extend([E::ExitParagraph(paragraph), E::ExitScope(scope)]);
            runs.push((
                container,
                if empty_blocks {
                    Vec::new()
                } else {
                    vec![child]
                },
            ));
            root_children.push(container);
            child_runs.push((container, (!empty_blocks).then_some(child)));
            cursor += usize::from(!empty_blocks);
            paragraph += 1;
            events.push(E::EnterParagraph(paragraph));
            runs.push((0, Vec::new()));
        }};
    }
    if endpoint_only {
        push_block!();
    }
    for segment in 0..count {
        let node = 300 + segment as u32;
        let next = cursor + JAPANESE.encode_utf16().count();
        origins.push(text::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: text::Range::new(cursor, next),
            aggregate: text::Range::new(cursor, next),
            kind: text::ParagraphOriginKind::Authored,
        });
        events.push(E::Origin {
            origin: origins.len() - 1,
            affinity: text::Affinity::After,
        });
        origin_nodes.push(node);
        runs[paragraph].1.push(node);
        root_children.push(node);
        cursor = next;
        if segment + 1 < count {
            push_block!();
        }
    }
    if endpoint_only {
        push_block!();
    }
    let _ = (next_block, cursor);
    events.extend([
        E::Control {
            control: start + 1,
            affinity: text::Affinity::After,
        },
        E::ExitScope(owner_scope),
    ]);
    if nested {
        events.extend([
            E::Control {
                control: 3,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
        ]);
    }
    events.extend([E::ExitParagraph(paragraph), E::ExitScope(0)]);
    runs[paragraph].1.extend(&boundary_nodes[start + 1..]);
    root_children.extend(&boundary_nodes[start + 1..]);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins,
            connections: vec![],
            controls,
            boundary_policies: vec![],
            replacements: vec![],
            scopes,
            paragraphs: runs
                .iter()
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
    let mut tree = PublicLayoutTreeOf::new().children(0, root_children).style(
        0,
        NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        },
    );
    for (container, child) in child_runs {
        tree = tree.children(container, child).style(
            container,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    }
    for &node in &boundary_nodes {
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                if node <= 101 {
                    InlineBoundaryKind::Start
                } else {
                    InlineBoundaryKind::End
                },
                InlineMetricsOf::try_new(8.0, 10.0).unwrap(),
                crate::InlineBoundaryEdgeOf::ZERO,
            )),
        );
    }
    let mut source_node = origin_nodes.into_iter();
    let mut bindings = Vec::new();
    for (processed, (container, nodes)) in processed.paragraphs().zip(runs) {
        let analyzed = text::AnalyzedSource::from_processed_paragraph(processed).unwrap();
        let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
        let prepared = system
            .prepare(&analyzed, text::PreparationOptions::default())
            .unwrap();
        let origins = prepared
            .origin_views()
            .map(|view| {
                let node = source_node.next().unwrap();
                let origin = view.handle();
                tree.insert_children(node, []);
                tree.insert_input(
                    node,
                    LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
                );
                TextOriginBinding { origin, node }
            })
            .collect();
        let controls = prepared
            .control_views()
            .zip(
                nodes
                    .iter()
                    .copied()
                    .filter(|node| boundary_nodes.contains(node)),
            )
            .map(|(view, node)| TextControlBinding {
                occurrence: view.handle(),
                node,
            })
            .collect();
        bindings.push(TextParagraphBinding {
            prepared,
            container,
            nodes,
            origins,
            atomics: vec![],
            controls,
            line_strut: None,
        });
    }
    (tree, bindings)
}

#[test]
fn middle_continuation_with_neither_endpoint_keeps_one_global_owner_and_local_geometry() {
    let (tree, mut bindings) = several_segments(3, false, true);
    let middle = bindings[2].prepared.clone();
    assert!(middle.control_views().next().is_none());
    let owners: Vec<_> = [0, 2, 4]
        .into_iter()
        .map(|i| bindings[i].prepared.scopes().next().unwrap())
        .collect();
    assert_ne!(owners[0], owners[1]);
    assert_eq!(owners[0].captured_scope(), owners[1].captured_scope());
    bindings.reverse(); // Caller vector order supplies no traversal authority.
    let capture = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    let mut facts: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter_map(|entry| {
            let InlineFragmentOwner::Scope(owner) = entry.owner() else {
                return None;
            };
            assert_eq!(
                owner.association,
                TextScopeAssociation::Inline {
                    start: 101,
                    end: 102
                }
            );
            assert_eq!(owner.scope.captured_scope(), owners[0].captured_scope());
            let text::FinalizedFragmentOwner::VirtualSeparator(v) =
                entry.fragment().finalized_fragment().view().owner()
            else {
                panic!("virtual authority");
            };
            Some((
                v.view().source_position().get(),
                entry.fragment().rect().unwrap().origin(),
            ))
        })
        .collect();
    facts.sort_by_key(|fact| fact.0);
    assert_eq!(
        facts,
        [
            (3, Point::new(30.0, 0.0)),
            (12, Point::new(30.0, 20.0)),
            (21, Point::new(30.0, 40.0))
        ]
    );
    assert_eq!(batch.placed_text().len(), 5);
}

#[test]
fn genuine_nonemitting_endpoint_paragraphs_supply_remote_controls_without_generated_duplicates() {
    let (tree, bindings) = several_segments(2, true, false);
    assert_eq!(bindings[0].prepared.origin_views().len(), 0);
    assert_eq!(bindings[6].prepared.origin_views().len(), 0);
    assert_eq!(bindings[0].prepared.virtual_separators().len(), 0);
    assert_eq!(bindings[6].prepared.virtual_separators().len(), 0);
    let capture = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()).unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 7);
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 2);
    for entry in generated {
        assert_eq!(entry.fragment().rect().unwrap().origin().x, 30.0);
        assert_eq!(entry.fragment().rect().unwrap().size().width, 10.0);
    }
}

#[test]
fn foreign_same_ordinal_capture_cannot_supply_a_remote_endpoint() {
    let (tree, mut bindings) = continuation(true, true);
    let (_, foreign) = continuation(true, true);
    assert_ne!(
        bindings[0]
            .prepared
            .scopes()
            .next()
            .unwrap()
            .captured_scope(),
        foreign[0]
            .prepared
            .scopes()
            .next()
            .unwrap()
            .captured_scope()
    );
    bindings[2] = foreign[2].clone();
    let tree = tree.input(
        4,
        LayoutInputOf::inline_text(TextParticipation::new(
            bindings[2].origins[0].origin.clone(),
        )),
    );
    let Err(error) = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()) else {
        panic!("foreign capture endpoint");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeAssociation
        ))
    ));
}

#[test]
fn multiple_actual_paths_to_a_needed_scope_node_refuse_ambiguity() {
    let (tree, bindings) = continuation(true, true);
    let tree = tree
        .children(90, [0])
        .children(91, [0])
        .children(99, [90, 91])
        .style(90, NodeInputOf::default())
        .style(91, NodeInputOf::default())
        .style(99, NodeInputOf::default());
    let Err(error) = LayoutFlowInput::try_new(&tree, 99, bindings, Vec::new()) else {
        panic!("scope node has two real occurrences");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeTreeAmbiguity
        ))
    ));
}

#[test]
fn absent_nonemitting_endpoint_paragraph_is_a_missing_real_association() {
    let (tree, mut bindings) = several_segments(2, true, false);
    bindings.remove(0);
    let Err(error) = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()) else {
        panic!("the actual remote Start binding must be supplied");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeAssociation
        ))
    ));
}

#[test]
fn real_nested_block_order_must_agree_with_the_global_capture() {
    let (tree, bindings) = several_segments(3, false, false);
    let tree = tree.children(0, [101, 300, 201, 301, 200, 302, 102]);
    let Err(error) = LayoutFlowInput::try_new(&tree, 0, bindings, Vec::new()) else {
        panic!("the child domains cannot exchange actual tree order");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ParticipantOrder
        ))
    ));
}

#[test]
fn positive_global_endpoints_remain_local_to_their_actual_paragraphs() {
    let (mut tree, bindings) = continuation(true, true);
    let compute = |tree: &PublicLayoutTreeOf<f64>| {
        let capture = LayoutFlowInput::try_new(tree, 0, bindings.clone(), Vec::new()).unwrap();
        compute_layout_with_flow(&capture, 0, request(500.0)).unwrap()
    };
    let zero = compute(&tree);
    for (node, kind, border, padding) in [
        (1, InlineBoundaryKind::Start, 3.0, 5.0),
        (5, InlineBoundaryKind::End, 2.0, 3.0),
    ] {
        tree.insert_input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                kind,
                InlineMetricsOf::try_new(8.0, 10.0).unwrap(),
                InlineBoundaryEdgeOf::try_new(border, padding).unwrap(),
            )),
        );
    }
    let positive = compute(&tree);
    assert_eq!(positive.placed_text().len(), 3);
    let node = |batch: &CompletedLayoutBatchOf<u32, f64>, node| {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
    };
    assert_eq!(node(&positive, 1).size, Size::new(8.0, 0.0));
    assert_eq!(node(&positive, 5).size, Size::new(5.0, 0.0));
    assert_eq!(node(&positive, 3), node(&zero, 3));
    assert_eq!(node(&positive, 6), node(&zero, 6));
    let separators = |batch: &CompletedLayoutBatchOf<u32, f64>| {
        batch
            .unrounded_inline_fragments()
            .iter()
            .filter_map(|entry| {
                let InlineFragmentOwner::Scope(binding) = entry.owner() else {
                    return None;
                };
                assert_eq!(
                    binding.association,
                    TextScopeAssociation::Inline { start: 1, end: 5 }
                );
                let text::FinalizedFragmentOwner::VirtualSeparator(separator) =
                    entry.fragment().finalized_fragment().view().owner()
                else {
                    panic!("Scope geometry must remain a real separator")
                };
                Some((separator, entry.fragment().rect().unwrap()))
            })
            .collect::<Vec<_>>()
    };
    let before = separators(&zero);
    let after = separators(&positive);
    assert_eq!(before.len(), 2);
    assert_eq!(after.len(), 2);
    for ((first, old), (second, new)) in before.iter().zip(&after) {
        assert_eq!(first, second);
        assert_eq!(new.size(), Size::new(10.0, 10.0));
        assert_eq!(
            new.origin(),
            Point::new(
                old.origin().x
                    + if first.view().source_position().get() == 3 {
                        8.0
                    } else {
                        0.0
                    },
                old.origin().y
            )
        );
    }
    let mut actual = Vec::new();
    for entry in positive.unrounded_inline_fragments() {
        if let text::FinalizedFragmentOwner::Control(control) =
            entry.fragment().finalized_fragment().view().owner()
        {
            let InlineFragmentOwner::Node(node) = entry.owner() else {
                panic!("an endpoint must remain node-owned")
            };
            actual.push((*node, control));
        }
    }
    assert_eq!(actual.len(), 2);
    assert!(
        actual
            .iter()
            .any(|(node, control)| *node == 1 && *control == bindings[0].controls[0].occurrence)
    );
    assert!(
        actual
            .iter()
            .any(|(node, control)| *node == 5 && *control == bindings[2].controls[0].occurrence)
    );
    assert!(bindings[1].controls.is_empty());
}
