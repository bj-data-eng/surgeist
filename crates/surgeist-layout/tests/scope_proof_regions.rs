#![forbid(unsafe_code)]

//! Emitted scope extents demand their flow, without widening through dormant gaps.

use std::{collections::HashMap, convert::Infallible, sync::Arc};
use surgeist_layout as layout;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";
const AHEM: &[u8] = include_bytes!("../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct Tree {
    children: Vec<usize>,
    nested: HashMap<usize, Vec<usize>>,
    inputs: HashMap<usize, layout::LayoutInput>,
    non_box: layout::NodeInput,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children = if node == 0 {
            self.children.as_slice()
        } else {
            self.nested.get(&node).map_or(&[][..], Vec::as_slice)
        };
        children.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.children(node).count()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl layout::LayoutTree for Tree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[&node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[&node].as_view()
    }
}

#[derive(Clone, Copy)]
enum Owners {
    LocalBeforeBlock,
    LocalInsideWideAncestor,
    DisjointAroundBlock,
    SpanningBlock,
    WholeDomain,
    DormantSpanning,
}

struct Fixture {
    tree: Tree,
    parents: Vec<layout::TextParagraphBinding<usize>>,
    empty_child: layout::TextParagraphBinding<usize>,
    additional_empty_children: Vec<layout::TextParagraphBinding<usize>>,
}

struct Pair {
    scope: usize,
    start: usize,
    end: usize,
}

fn inline(
    scopes: &mut Vec<text::SourceProcessingScope>,
    controls: &mut Vec<text::ParagraphControl>,
    style: text::Style,
    start: usize,
    end: usize,
) -> Pair {
    let pair = Pair {
        scope: scopes.len(),
        start: controls.len(),
        end: controls.len() + 1,
    };
    controls.extend([
        text::ParagraphControl {
            at: start,
            kind: text::ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: end,
            kind: text::ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    scopes.push(text::SourceProcessingScope::Inline {
        trim: text::WhiteSpaceTrim::default(),
        style: Some(style),
        boundaries: Some(text::SourceProcessingScopeBoundaries {
            start: pair.start,
            end: pair.end,
        }),
    });
    pair
}

fn fixture(owners: Owners) -> Fixture {
    fixture_with_empty_paragraphs(owners, 1)
}

fn fixture_with_empty_paragraphs(owners: Owners, child_paragraphs: usize) -> Fixture {
    fixture_with_child_topology(
        owners,
        child_paragraphs,
        text::SourceProcessingTopology::Complete,
    )
}

fn fixture_with_child_topology(
    owners: Owners,
    child_paragraphs: usize,
    child_topology: text::SourceProcessingTopology,
) -> Fixture {
    assert!(child_paragraphs > 0);
    let resumed = child_paragraphs + 1;
    let plain = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        locale: Some("ja".into()),
        word_space_transform: text::WordSpaceTransform::None,
        ..text::Style::default()
    };
    let authored_auto = text::Style {
        word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
        ..plain.clone()
    };
    let emitter = text::Style {
        word_space_transform: text::WordSpaceTransform::Space,
        ..plain.clone()
    };
    let block = |base_style, topology| text::SourceProcessingScope::Block {
        topology,
        trim: text::WhiteSpaceTrim::default(),
        base_style,
        used_direction: text::ParagraphBaseDirection::LeftToRight,
    };
    let mut source = text::Source::new(format!("{JAPANESE}{JAPANESE}"));
    let n = JAPANESE.encode_utf16().count();
    let end = 2 * n;
    source.span(
        text::Range::new(0, n),
        if matches!(owners, Owners::DormantSpanning) {
            plain.clone()
        } else {
            authored_auto.clone()
        },
    );
    source.span(
        text::Range::new(n, end),
        if matches!(
            owners,
            Owners::DisjointAroundBlock | Owners::SpanningBlock | Owners::WholeDomain
        ) {
            authored_auto
        } else {
            plain.clone()
        },
    );
    let origin = |start, end| text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(start, end),
        aggregate: text::Range::new(start, end),
        kind: text::ParagraphOriginKind::Authored,
    };
    // Scope0 is the actual root domain; scope1 is the real empty Block child.
    let mut scopes = vec![
        block(
            if matches!(owners, Owners::WholeDomain) {
                emitter.clone()
            } else {
                plain.clone()
            },
            text::SourceProcessingTopology::Complete,
        ),
        block(plain.clone(), child_topology),
    ];
    let mut controls = Vec::new();
    use text::SourceProcessingEvent as E;
    let control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let occurrence = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let (before, after, first_controls, last_controls, first_nodes, last_nodes) = match owners {
        Owners::LocalBeforeBlock => {
            let local = inline(&mut scopes, &mut controls, emitter, 0, n);
            (
                vec![
                    E::EnterScope(local.scope),
                    control(local.start),
                    occurrence(0),
                    control(local.end),
                    E::ExitScope(local.scope),
                ],
                vec![occurrence(1)],
                vec![local.start, local.end],
                vec![],
                vec![10 + local.start, 2, 10 + local.end],
                vec![4],
            )
        }
        Owners::LocalInsideWideAncestor => {
            let wide = inline(&mut scopes, &mut controls, plain, 0, end);
            let local = inline(&mut scopes, &mut controls, emitter, 0, n);
            (
                vec![
                    E::EnterScope(wide.scope),
                    control(wide.start),
                    E::EnterScope(local.scope),
                    control(local.start),
                    occurrence(0),
                    control(local.end),
                    E::ExitScope(local.scope),
                ],
                vec![occurrence(1), control(wide.end), E::ExitScope(wide.scope)],
                vec![wide.start, local.start, local.end],
                vec![wide.end],
                vec![10 + wide.start, 10 + local.start, 2, 10 + local.end],
                vec![4, 10 + wide.end],
            )
        }
        Owners::DisjointAroundBlock => {
            let left = inline(&mut scopes, &mut controls, emitter.clone(), 0, n);
            let right = inline(&mut scopes, &mut controls, emitter, n, end);
            (
                vec![
                    E::EnterScope(left.scope),
                    control(left.start),
                    occurrence(0),
                    control(left.end),
                    E::ExitScope(left.scope),
                ],
                vec![
                    E::EnterScope(right.scope),
                    control(right.start),
                    occurrence(1),
                    control(right.end),
                    E::ExitScope(right.scope),
                ],
                vec![left.start, left.end],
                vec![right.start, right.end],
                vec![10 + left.start, 2, 10 + left.end],
                vec![10 + right.start, 4, 10 + right.end],
            )
        }
        Owners::SpanningBlock | Owners::DormantSpanning => {
            let style = if matches!(owners, Owners::SpanningBlock) {
                emitter
            } else {
                plain
            };
            let spanning = inline(&mut scopes, &mut controls, style, 0, end);
            (
                vec![
                    E::EnterScope(spanning.scope),
                    control(spanning.start),
                    occurrence(0),
                ],
                vec![
                    occurrence(1),
                    control(spanning.end),
                    E::ExitScope(spanning.scope),
                ],
                vec![spanning.start],
                vec![spanning.end],
                vec![10 + spanning.start, 2],
                vec![4, 10 + spanning.end],
            )
        }
        Owners::WholeDomain => (
            vec![occurrence(0)],
            vec![occurrence(1)],
            vec![],
            vec![],
            vec![2],
            vec![4],
        ),
    };
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    events.extend(before);
    events.extend([E::ExitParagraph(0), E::EnterScope(1)]);
    for paragraph in 1..=child_paragraphs {
        events.extend([E::EnterParagraph(paragraph), E::ExitParagraph(paragraph)]);
    }
    events.extend([E::ExitScope(1), E::EnterParagraph(resumed)]);
    events.extend(after);
    events.extend([E::ExitParagraph(resumed), E::ExitScope(0)]);
    let control_facts = controls.clone();
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![origin(0, n), origin(n, end)],
            connections: vec![],
            controls,
            boundary_policies: vec![],
            replacements: vec![],
            scopes,
            paragraphs: (0..resumed + 1)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: text::ParagraphBaseDirection::LeftToRight,
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
    let prepared: Vec<_> = processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::AnalyzedSource::from_processed_paragraph(paragraph).unwrap();
            let policies = analyzed
                .wrapping_relations()
                .filter(|row| row.ancestor_wrap().is_none())
                .map(|row| text::WrappingPolicy {
                    relation: row.handle(),
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect();
            let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect();
    assert_eq!(prepared.len(), resumed + 1);
    for paragraph in &prepared[1..resumed] {
        assert_eq!(paragraph.origin_views().count(), 0);
        assert_eq!(paragraph.control_views().count(), 0);
        assert_eq!(paragraph.atomic_views().count(), 0);
        assert_eq!(paragraph.participants().count(), 0);
        assert_eq!(paragraph.scopes().count(), 0);
        assert!(paragraph.captured_paragraph().is_some());
    }
    for (index, expected) in [
        (0, !matches!(owners, Owners::DormantSpanning)),
        (
            resumed,
            matches!(
                owners,
                Owners::DisjointAroundBlock | Owners::SpanningBlock | Owners::WholeDomain
            ),
        ),
    ] {
        assert_eq!(
            prepared[index]
                .virtual_separators()
                .map(|value| value.view().source_position().get())
                .collect::<Vec<_>>(),
            if expected {
                vec![if index == 0 { 3 } else { n + 3 }]
            } else {
                vec![]
            }
        );
        assert_eq!(prepared[index].scopes().count(), usize::from(expected));
    }
    if matches!(owners, Owners::WholeDomain) {
        assert_eq!(
            prepared[0].scopes().next().unwrap().captured_scope().role(),
            text::ScopeRole::Domain
        );
    }
    if matches!(owners, Owners::LocalInsideWideAncestor) {
        let local = prepared[0].scopes().next().unwrap().captured_scope();
        let wide = local.parent().unwrap();
        assert_eq!(wide.role(), text::ScopeRole::Inline);
        assert_eq!(
            wide.style().unwrap().word_space_transform,
            text::WordSpaceTransform::None
        );
        assert!(wide.entry_control().is_some());
        assert!(wide.exit_control().is_some());
        assert!(
            local.exit_position()
                < prepared[1]
                    .captured_paragraph()
                    .unwrap()
                    .domain()
                    .enter_position()
        );
    }

    let mut inputs = HashMap::from([
        (
            0,
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
        ),
        (
            3,
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
        ),
        (
            2,
            layout::LayoutInput::inline_text(layout::TextParticipation::new(
                prepared[0].origin_views().next().unwrap().handle(),
            )),
        ),
        (
            4,
            layout::LayoutInput::inline_text(layout::TextParticipation::new(
                prepared[resumed].origin_views().next().unwrap().handle(),
            )),
        ),
    ]);
    for (ordinal, control) in control_facts.into_iter().enumerate() {
        let kind = match control.kind {
            text::ParagraphControlKind::InlineBoundaryStart => layout::InlineBoundaryKind::Start,
            text::ParagraphControlKind::InlineBoundaryEnd => layout::InlineBoundaryKind::End,
            _ => unreachable!(),
        };
        inputs.insert(
            10 + ordinal,
            layout::LayoutInput::inline_boundary(layout::InlineBoundaryInput::new(
                kind,
                layout::InlineMetrics::try_new(8.0, 10.0).unwrap(),
                layout::InlineBoundaryEdgeOf::ZERO,
            )),
        );
    }
    let mut children = first_nodes.clone();
    children.push(3);
    children.extend_from_slice(&last_nodes);
    let tree = Tree {
        children,
        nested: HashMap::new(),
        inputs,
        non_box: layout::NodeInput::non_box(),
    };
    let parents = [
        (0, first_nodes, first_controls, 2),
        (resumed, last_nodes, last_controls, 4),
    ]
    .into_iter()
    .map(|(index, nodes, controls, origin_node)| {
        let prepared = prepared[index].clone();
        let origin = prepared.origin_views().next().unwrap().handle();
        let bound_controls = prepared
            .control_views()
            .zip(controls)
            .map(|(view, ordinal)| layout::TextControlBinding {
                occurrence: view.handle(),
                node: 10 + ordinal,
            })
            .collect();
        layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes,
            origins: vec![layout::TextOriginBinding {
                origin,
                node: origin_node,
            }],
            controls: bound_controls,
            atomics: vec![],
            line_strut: None,
        }
    })
    .collect();
    let empty_child = layout::TextParagraphBinding {
        prepared: prepared[1].clone(),
        container: 3,
        nodes: vec![],
        origins: vec![],
        controls: vec![],
        atomics: vec![],
        line_strut: None,
    };
    let additional_empty_children = prepared[2..resumed]
        .iter()
        .map(|prepared| layout::TextParagraphBinding {
            prepared: prepared.clone(),
            container: 3,
            nodes: vec![],
            origins: vec![],
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        })
        .collect();
    Fixture {
        tree,
        parents,
        empty_child,
        additional_empty_children,
    }
}

fn request() -> layout::LayoutRootRequest {
    layout::LayoutRootRequest::viewport(layout::Size::new(
        layout::Available::definite(200.0),
        layout::Available::MAX_CONTENT,
    ))
    .unwrap()
}

fn compute(fixture: &Fixture, with_child: bool) -> layout::CompletedLayoutBatch<usize> {
    let mut paragraphs = fixture.parents.clone();
    if with_child {
        paragraphs.insert(1, fixture.empty_child.clone());
    }
    let capture = layout::TextLayoutInput::try_new(&fixture.tree, 0, paragraphs).unwrap();
    layout::compute_layout_with_text(&capture, 0, request()).unwrap()
}

fn rejects(
    fixture: &Fixture,
    paragraphs: Vec<layout::TextParagraphBinding<usize>>,
    reason: layout::TextBindingError,
) {
    let Err(error) = layout::TextLayoutInput::try_new(&fixture.tree, 0, paragraphs) else {
        panic!("invalid scope association must refuse {reason:?}")
    };
    assert!(
        matches!(error.kind(), layout::LayoutErrorKind::InvalidInput(layout::LayoutInvalidInput::TextBinding(actual)) if *actual == reason),
        "{error:?}"
    );
}

fn assert_geometry(batch: &layout::CompletedLayoutBatch<usize>, expected: &[(usize, f32)]) {
    assert_eq!(batch.placed_text().len(), 2);
    assert!(
        batch
            .placed_text()
            .iter()
            .all(|entry| entry.container() == 0 && entry.layout().lines().len() == 1)
    );
    let child = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 3)
        .unwrap()
        .output();
    assert_eq!(child.location, layout::Point::new(0.0, 10.0));
    assert_eq!(child.size, layout::Size::new(200.0, 0.0));
    assert!(
        !batch
            .final_inline_fragments()
            .iter()
            .any(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(3))
    );
    let mut generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter_map(|entry| {
            let layout::InlineFragmentOwner::Scope(_) = entry.owner() else {
                return None;
            };
            let fragment = entry.fragment();
            let finalized = fragment.finalized_fragment();
            let view = finalized.view();
            let text::FinalizedFragmentOwner::VirtualSeparator(occurrence) = view.owner() else {
                panic!("actual virtual contributor")
            };
            assert_eq!(view.origin(), None);
            assert_eq!(view.aggregate_range(), None);
            let rect = fragment.rect().unwrap();
            // Ahem has a one-em advance. Three original10px cells precede each
            // actual10px separator; each parent line is10px and the child is empty.
            assert_eq!(rect.origin().x, 30.0);
            assert_eq!(rect.size(), layout::Size::new(10.0, 10.0));
            Some((occurrence.view().source_position().get(), rect.origin().y))
        })
        .collect();
    generated.sort_by_key(|entry| entry.0);
    assert_eq!(generated, expected);
}

#[test]
fn local_emitter_before_dormant_child_does_not_require_child_association() {
    let fixture = fixture(Owners::LocalBeforeBlock);
    assert_geometry(&compute(&fixture, false), &[(3, 0.0)]);
}

#[test]
fn known_wide_ancestor_endpoints_do_not_expand_the_local_emitter_into_its_child_gap() {
    let fixture = fixture(Owners::LocalInsideWideAncestor);
    assert_geometry(&compute(&fixture, false), &[(3, 0.0)]);
}

#[test]
fn disjoint_emitters_around_a_dormant_child_do_not_demand_their_gap() {
    let fixture = fixture(Owners::DisjointAroundBlock);
    assert_geometry(&compute(&fixture, false), &[(3, 0.0), (11, 10.0)]);
}

#[test]
fn spanning_emitter_requires_the_real_empty_child_association_without_adding_text() {
    let fixture = fixture(Owners::SpanningBlock);
    rejects(
        &fixture,
        fixture.parents.clone(),
        layout::TextBindingError::ScopeAssociation,
    );
    assert_geometry(&compute(&fixture, true), &[(3, 0.0), (11, 10.0)]);
}

#[test]
fn domain_emitter_requires_its_full_domain_block_markers() {
    let fixture = fixture(Owners::WholeDomain);
    rejects(
        &fixture,
        fixture.parents.clone(),
        layout::TextBindingError::ScopeAssociation,
    );
    assert_geometry(&compute(&fixture, true), &[(3, 0.0), (11, 10.0)]);
}

#[test]
fn dormant_supplied_empty_metadata_cannot_hide_a_foreign_original_binding() {
    let fixture = fixture(Owners::DormantSpanning);
    assert!(
        fixture
            .parents
            .iter()
            .all(|paragraph| paragraph.prepared.scopes().len() == 0)
    );
    let mut malformed = fixture.empty_child.clone();
    malformed
        .origins
        .push(fixture.parents[0].origins[0].clone());
    let mut paragraphs = fixture.parents.clone();
    paragraphs.insert(1, malformed);
    rejects(&fixture, paragraphs, layout::TextBindingError::EmptyRun);
}

#[test]
fn dormant_duplicate_empty_paragraph_identity_still_refuses() {
    let fixture = fixture(Owners::DormantSpanning);
    let mut paragraphs = fixture.parents.clone();
    paragraphs.extend([fixture.empty_child.clone(), fixture.empty_child.clone()]);
    rejects(
        &fixture,
        paragraphs,
        layout::TextBindingError::OverlappingParagraphs,
    );
}

#[test]
fn dormant_empty_paragraph_cannot_request_a_line_strut() {
    let fixture = fixture(Owners::DormantSpanning);
    let mut malformed = fixture.empty_child.clone();
    malformed.line_strut = Some(text::LineStrut {
        baseline: 8.0,
        line_extent: 10.0,
    });
    let mut paragraphs = fixture.parents.clone();
    paragraphs.insert(1, malformed);
    rejects(&fixture, paragraphs, layout::TextBindingError::EmptyRun);
}

#[test]
fn distinct_empty_paragraphs_share_their_real_empty_domain_slot_in_any_inventory_order() {
    let fixture = fixture_with_empty_paragraphs(Owners::SpanningBlock, 2);
    let first = fixture.empty_child.prepared.captured_paragraph().unwrap();
    let second = fixture.additional_empty_children[0]
        .prepared
        .captured_paragraph()
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(first.domain(), second.domain());
    assert!(first.exit_position() < second.enter_position());
    // Only the domain marker is used by the parent's spanning region. A second
    // genuine empty paragraph does not invent another physical content slot.
    let baseline = compute(&fixture, true);
    let paragraphs = vec![
        fixture.parents[1].clone(),
        fixture.additional_empty_children[0].clone(),
        fixture.parents[0].clone(),
        fixture.empty_child.clone(),
    ];
    let capture = layout::TextLayoutInput::try_new(&fixture.tree, 0, paragraphs).unwrap();
    let associated = layout::compute_layout_with_text(&capture, 0, request()).unwrap();
    assert_geometry(&associated, &[(3, 0.0), (11, 10.0)]);
    assert_eq!(associated.final_entries(), baseline.final_entries());
    assert_eq!(
        associated.final_inline_fragments().len(),
        baseline.final_inline_fragments().len()
    );
}

#[test]
fn a_foreign_empty_paragraph_cannot_supply_the_spanning_owners_child_marker() {
    let actual = fixture(Owners::SpanningBlock);
    let foreign = fixture(Owners::SpanningBlock);
    assert_ne!(
        actual.empty_child.prepared.captured_paragraph(),
        foreign.empty_child.prepared.captured_paragraph()
    );
    let mut paragraphs = actual.parents.clone();
    // Equal source text, empty inventories and the same actual node number do
    // not transfer the independent source capture's paragraph/domain custody.
    paragraphs.push(foreign.empty_child);
    rejects(
        &actual,
        paragraphs,
        layout::TextBindingError::ScopeAssociation,
    );
}

#[test]
fn an_empty_child_association_cannot_name_its_parent_as_its_actual_container() {
    let fixture = fixture(Owners::SpanningBlock);
    let mut wrong = fixture.empty_child.clone();
    wrong.container = 0;
    let mut paragraphs = fixture.parents.clone();
    paragraphs.push(wrong);
    rejects(
        &fixture,
        paragraphs,
        layout::TextBindingError::ScopeAssociation,
    );
}

struct IncidenceFixture {
    tree: Tree,
    flow: Vec<layout::TextParagraphBinding<usize>>,
    empty: layout::TextParagraphBinding<usize>,
}

fn nonempty_domain_with_terminal_empty_paragraph() -> IncidenceFixture {
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        word_space_transform: text::WordSpaceTransform::None,
        ..text::Style::default()
    };
    let mut source = text::Source::new("AXB");
    source.span(text::Range::new(0, 3), style.clone());
    let origins = (0..3)
        .map(|index| text::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: text::Range::new(index, index + 1),
            aggregate: text::Range::new(index, index + 1),
            kind: text::ParagraphOriginKind::Authored,
        })
        .collect();
    let block = || text::SourceProcessingScope::Block {
        topology: text::SourceProcessingTopology::Complete,
        trim: text::WhiteSpaceTrim::default(),
        base_style: style.clone(),
        used_direction: text::ParagraphBaseDirection::LeftToRight,
    };
    use text::SourceProcessingEvent as E;
    let occurrence = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins,
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![block(), block()],
            paragraphs: (0..4)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: text::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // Parent paragraphs own A/B. The child domain first owns genuine X,
            // then its distinct empty paragraph at the terminal content slot.
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                occurrence(0),
                E::ExitParagraph(0),
                E::EnterScope(1),
                E::EnterParagraph(1),
                occurrence(1),
                E::ExitParagraph(1),
                E::EnterParagraph(2),
                E::ExitParagraph(2),
                E::ExitScope(1),
                E::EnterParagraph(3),
                occurrence(2),
                E::ExitParagraph(3),
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
            let policies = analyzed
                .wrapping_relations()
                .filter(|row| row.ancestor_wrap().is_none())
                .map(|row| text::WrappingPolicy {
                    relation: row.handle(),
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect();
            let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect();
    assert_eq!(prepared.len(), 4);
    assert!(
        prepared
            .iter()
            .all(|paragraph| paragraph.scopes().len() == 0)
    );
    assert_eq!(prepared[1].origin_views().count(), 1);
    assert_eq!(prepared[2].origin_views().count(), 0);
    assert_eq!(prepared[2].control_views().count(), 0);
    assert_eq!(prepared[2].atomic_views().count(), 0);
    assert_eq!(prepared[2].participants().count(), 0);
    assert_eq!(
        prepared[1].captured_paragraph().unwrap().domain(),
        prepared[2].captured_paragraph().unwrap().domain()
    );
    let mut inputs = HashMap::from([
        (
            0,
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
        ),
        (
            3,
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
        ),
    ]);
    let flow = [(0, 0, 2), (1, 3, 6), (3, 0, 4)]
        .into_iter()
        .map(|(index, container, node)| {
            let origin = prepared[index].origin_views().next().unwrap().handle();
            inputs.insert(
                node,
                layout::LayoutInput::inline_text(layout::TextParticipation::new(origin.clone())),
            );
            layout::TextParagraphBinding {
                prepared: prepared[index].clone(),
                container,
                nodes: vec![node],
                origins: vec![layout::TextOriginBinding { origin, node }],
                controls: vec![],
                atomics: vec![],
                line_strut: None,
            }
        })
        .collect();
    let tree = Tree {
        children: vec![2, 3, 4],
        nested: HashMap::from([(3, vec![6])]),
        inputs,
        non_box: layout::NodeInput::non_box(),
    };
    let empty = layout::TextParagraphBinding {
        prepared: prepared[2].clone(),
        container: 3,
        nodes: vec![],
        origins: vec![],
        controls: vec![],
        atomics: vec![],
        line_strut: None,
    };
    IncidenceFixture { tree, flow, empty }
}

#[test]
fn an_empty_paragraph_in_a_nonempty_domain_uses_its_authentic_terminal_run_slot() {
    let fixture = nonempty_domain_with_terminal_empty_paragraph();
    let baseline_capture =
        layout::TextLayoutInput::try_new(&fixture.tree, 0, fixture.flow.clone()).unwrap();
    let baseline = layout::compute_layout_with_text(&baseline_capture, 0, request()).unwrap();
    let mut paragraphs = fixture.flow.clone();
    paragraphs.insert(0, fixture.empty);
    let capture = layout::TextLayoutInput::try_new(&fixture.tree, 0, paragraphs).unwrap();
    let associated = layout::compute_layout_with_text(&capture, 0, request()).unwrap();
    assert_eq!(associated.final_entries(), baseline.final_entries());
    assert_eq!(associated.placed_text().len(), 3);
    assert_eq!(
        associated
            .placed_text()
            .iter()
            .map(|entry| entry.layout().lines().len())
            .sum::<usize>(),
        3
    );
    assert_eq!(
        associated.final_inline_fragments().len(),
        baseline.final_inline_fragments().len()
    );
    let child = associated
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 3)
        .unwrap()
        .output();
    assert_eq!(child.location, layout::Point::new(0.0, 10.0));
    assert_eq!(child.size, layout::Size::new(200.0, 10.0));
}

#[test]
fn one_empty_paragraph_cannot_certify_a_nonempty_domain_with_its_real_neighbor_omitted() {
    let mut fixture = nonempty_domain_with_terminal_empty_paragraph();
    // The actual tree now has an empty Box3 and the caller omits the genuine X
    // run. The supplied empty paragraph still belongs AFTER X in its authentic
    // source domain; it cannot turn that nonempty domain into an empty one.
    fixture.tree.nested.remove(&3);
    fixture.tree.inputs.remove(&6);
    fixture.flow.remove(1);
    let mut paragraphs = fixture.flow;
    paragraphs.push(fixture.empty);
    let Err(error) = layout::TextLayoutInput::try_new(&fixture.tree, 0, paragraphs) else {
        panic!("missing genuine neighbor flow cannot establish empty-paragraph incidence")
    };
    assert!(
        matches!(
            error.kind(),
            layout::LayoutErrorKind::InvalidInput(layout::LayoutInvalidInput::TextBinding(
                layout::TextBindingError::ScopeAssociation
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn unknown_child_inline_topology_preserves_genuine_empty_domain_authority() {
    let fixture = fixture_with_child_topology(
        Owners::SpanningBlock,
        1,
        text::SourceProcessingTopology::Unknown,
    );
    assert_eq!(
        fixture.parents[0]
            .prepared
            .captured_paragraph()
            .unwrap()
            .domain()
            .domain_topology(),
        text::SourceProcessingTopology::Complete
    );
    assert_eq!(
        fixture
            .empty_child
            .prepared
            .captured_paragraph()
            .unwrap()
            .domain()
            .domain_topology(),
        text::SourceProcessingTopology::Unknown
    );
    // Unknown describes inline inventory, not the canonical paragraph/domain
    // identity or its actual empty Block slot between the parent's two lines.
    assert_geometry(&compute(&fixture, true), &[(3, 0.0), (11, 10.0)]);
}

#[test]
fn unknown_child_inline_topology_does_not_waive_a_demanded_block_association() {
    let fixture = fixture_with_child_topology(
        Owners::SpanningBlock,
        1,
        text::SourceProcessingTopology::Unknown,
    );
    rejects(
        &fixture,
        fixture.parents.clone(),
        layout::TextBindingError::ScopeAssociation,
    );
}

#[test]
fn flat_empty_preparation_without_canonical_paragraph_identity_cannot_associate_a_domain() {
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: text::Source::new(""),
        base_style: text::Style {
            font: text::Font::new().family("Ahem"),
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            word_space_transform: text::WordSpaceTransform::None,
            ..text::Style::default()
        },
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert!(prepared.captured_paragraph().is_none());
    assert_eq!(prepared.origin_views().count(), 0);
    assert_eq!(prepared.control_views().count(), 0);
    assert_eq!(prepared.atomic_views().count(), 0);
    assert_eq!(prepared.participants().count(), 0);
    assert_eq!(prepared.scopes().count(), 0);
    assert_eq!(prepared.virtual_separators().count(), 0);
    let tree = Tree {
        children: vec![],
        nested: HashMap::new(),
        inputs: HashMap::from([(
            0,
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
        )]),
        non_box: layout::NodeInput::non_box(),
    };
    let paragraph = layout::TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![],
        origins: vec![],
        controls: vec![],
        atomics: vec![],
        line_strut: None,
    };
    let Err(error) = layout::TextLayoutInput::try_new(&tree, 0, vec![paragraph]) else {
        panic!("empty output alone cannot provide canonical paragraph/domain authority")
    };
    assert!(
        matches!(
            error.kind(),
            layout::LayoutErrorKind::InvalidInput(layout::LayoutInvalidInput::TextBinding(
                layout::TextBindingError::EmptyRun
            ))
        ),
        "{error:?}"
    );
}
