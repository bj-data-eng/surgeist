//! Independent public consumer qualification of completed domain outputs.
//!
//! Raw paragraph markers author storage, not native run seams. The original
//! capture/tree below owns every source and block boundary. New domain/segment
//! APIs are functionally qualified; no historical compiler RED is claimed.
use super::*;
use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const ROOT: u32 = 0;
const A: u32 = 2;
const BLOCK: u32 = 3;
const C: u32 = 4;
const B: u32 = 6;

#[derive(Clone, Copy)]
enum Shape {
    Single,
    Flat { joined: bool, empty_middle: bool },
    Child { joined: bool },
}

struct Fixture {
    source: Source,
    processed: Arc<text::ProcessedSource>,
    raws: Vec<text::CapturedParagraph>,
    prepared: Vec<text::PreparedText>,
    child: bool,
}

fn block_style(size: f32) -> text::Style {
    text::Style {
        white_space: text::WhiteSpace::Preserve,
        ..ahem_style(size, f64::from(size))
    }
}

fn domain(size: f32) -> text::SourceProcessingScope {
    text::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: text::SourceProcessingTopology::Complete,
        trim: text::WhiteSpaceTrim::default(),
        base_style: block_style(size),
        used_direction: ParagraphBaseDirection::LeftToRight,
    }
}

fn fixture(shape: Shape) -> Fixture {
    use text::SourceProcessingEvent as E;
    let child = matches!(shape, Shape::Child { .. });
    let source = Source::identified(
        if child { "ABC" } else { "AC" },
        text::Id::from_u64(784_901),
        71,
    );
    let origin = |start, end| text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(start, end),
        aggregate: text::Range::new(start, end),
        kind: text::ParagraphOriginKind::Authored,
    };
    let authored = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let (paragraph_count, scopes, events, joint_flows) = match shape {
        Shape::Single => (
            1,
            vec![domain(10.0)],
            vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                authored(0),
                E::Connection(0),
                authored(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
            vec![],
        ),
        Shape::Flat {
            joined,
            empty_middle,
        } => {
            let last = if empty_middle { 2 } else { 1 };
            let mut events = vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                authored(0),
                E::ExitParagraph(0),
            ];
            if empty_middle {
                events.extend([E::EnterParagraph(1), E::ExitParagraph(1)]);
            }
            events.extend([
                E::EnterParagraph(last),
                E::Connection(0),
                authored(1),
                E::ExitParagraph(last),
                E::ExitScope(0),
            ]);
            let flows = if joined {
                vec![text::SourceProcessingJointFlow {
                    scope: 0,
                    paragraphs: (0..=last).collect(),
                    facts_owner: if empty_middle { 1 } else { 0 },
                }]
            } else {
                vec![]
            };
            (last + 1, vec![domain(10.0)], events, flows)
        }
        Shape::Child { joined } => (
            3,
            vec![domain(10.0), domain(20.0)],
            vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                authored(0),
                E::ExitParagraph(0),
                E::EnterScope(1),
                E::EnterParagraph(1),
                authored(1),
                E::ExitParagraph(1),
                E::ExitScope(1),
                E::EnterParagraph(2),
                E::Connection(0),
                authored(2),
                E::ExitParagraph(2),
                E::ExitScope(0),
            ],
            if joined {
                vec![text::SourceProcessingJointFlow {
                    scope: 0,
                    paragraphs: vec![0, 2],
                    facts_owner: 0,
                }]
            } else {
                vec![]
            },
        ),
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: if child {
                vec![origin(0, 1), origin(1, 2), origin(2, 3)]
            } else {
                vec![origin(0, 1), origin(1, 2)]
            },
            // This is the actual parent-domain connection, not a child-domain
            // shortcut. The real Block separates A/C; a raw seam alone does not.
            connections: vec![text::SourceProcessingConnection {
                before_origin: 0,
                after_origin: if child { 2 } else { 1 },
                kind: if child {
                    text::ParagraphConnectionKind::Disconnected
                } else {
                    text::ParagraphConnectionKind::Continuous
                },
            }],
            boundary_policies: vec![text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }],
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows,
            scope_alignment_limits: None,
            scopes,
            paragraphs: (0..paragraph_count)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            events,
        })
        .unwrap(),
    );
    let raws: Vec<_> = processed
        .paragraphs()
        .map(|raw| raw.captured_paragraph())
        .collect();
    assert_eq!(raws.len(), paragraph_count);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let analyze = |member| {
        let batch = text::AnalyzedDomain::from_processed_paragraph(
            processed.paragraphs().nth(member).unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(Arc::ptr_eq(batch.processed_source(), &processed));
        assert_eq!(batch.domain(), raws[member].domain());
        batch
    };
    let parent = analyze(0);
    let mut prepared = parent
        .outputs()
        .map(|output| {
            let analyzed =
                crate::test_support::text::in_flat_wrapping_context(output.clone(), Wrap::Word);
            let value = system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap();
            let analyzed_owner = output.captured_owner().unwrap();
            let prepared_owner = value.captured_owner().unwrap();
            assert_eq!(
                prepared_owner.members().collect::<Vec<_>>(),
                analyzed_owner.members().collect::<Vec<_>>()
            );
            assert_eq!(prepared_owner.facts_owner(), analyzed_owner.facts_owner());
            assert_eq!(prepared_owner.joint_flow(), analyzed_owner.joint_flow());
            value
        })
        .collect::<Vec<_>>();
    if child {
        let child_domain = analyze(1);
        assert_eq!(child_domain.outputs().count(), 1);
        let child_prepared = system
            .prepare(
                child_domain.outputs().next().unwrap(),
                text::PreparationOptions::default(),
            )
            .unwrap();
        // Ordered by the first actual member: joined-parent/child, or A/B/C.
        prepared.insert(1, child_prepared);
    }
    Fixture {
        source,
        processed,
        raws,
        prepared,
        child,
    }
}

fn bind<S: LayoutScalar>(
    fixture: &Fixture,
) -> (PublicLayoutTreeOf<S>, Vec<TextParagraphBinding<u32>>) {
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(
            ROOT,
            if fixture.child {
                vec![A, BLOCK, C]
            } else {
                vec![A, C]
            },
        )
        .style(
            ROOT,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    if fixture.child {
        tree = tree.children(BLOCK, [B]).style(
            BLOCK,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    }
    let mut bindings = Vec::new();
    for prepared in &fixture.prepared {
        let owner = prepared.captured_owner().unwrap();
        let child = fixture.child && owner.domain() == fixture.raws[1].domain();
        let origins: Vec<_> = prepared
            .origin_views()
            .map(|origin| {
                assert_eq!(origin.source(), &fixture.source);
                let range = origin.aggregate_range();
                assert_eq!(range.end, range.start + 1);
                let node = match (fixture.child, range.start) {
                    (_, 0) => A,
                    (true, 1) => B,
                    (true, 2) | (false, 1) => C,
                    _ => panic!("unexpected authored origin {range:?}"),
                };
                let handle = origin.handle();
                tree.insert_children(node, []);
                tree.insert_input(
                    node,
                    LayoutInputOf::inline_text(TextParticipation::new(handle.clone())),
                );
                assert!(
                    origin
                        .capture_position()
                        .unwrap()
                        .same_capture(&owner.facts_owner().enter_position())
                );
                TextOriginBinding {
                    origin: handle,
                    node,
                }
            })
            .collect();
        let nodes = origins.iter().map(|origin| origin.node).collect();
        bindings.push(TextParagraphBinding {
            inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
            prepared: prepared.clone(),
            container: if child { BLOCK } else { ROOT },
            nodes,
            origins,
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        });
    }
    (tree, bindings)
}

fn capture<'a, S: LayoutScalar>(
    tree: &'a PublicLayoutTreeOf<S>,
    bindings: Vec<TextParagraphBinding<u32>>,
) -> LayoutFlowInput<'a, PublicLayoutTreeOf<S>> {
    LayoutFlowInput::try_new(
        tree,
        ROOT,
        bindings,
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: vec![],
            child_order: vec![],
        },
        vec![],
    )
    .unwrap()
}

fn packets(layout: &text::Layout) -> Vec<(text::Range, u32, f64, f64, f32)> {
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.glyph_basis(), text::GlyphBasis::Unrotated);
    }
    let mut values = layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            assert_eq!(run.orientation(), text::LineOrientation::Horizontal);
            let origin = run.origin();
            run.glyphs()
                .iter()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y + f64::from(glyph.y()),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    values.sort_unstable_by_key(|value| value.0.start);
    values
}

fn provenance(layout: &text::Layout, prepared: &text::PreparedText) {
    let analyzed = layout.analyzed_source();
    let actual = analyzed.captured_owner().unwrap();
    let expected = prepared.captured_owner().unwrap();
    assert_eq!(
        actual.members().collect::<Vec<_>>(),
        expected.members().collect::<Vec<_>>()
    );
    assert_eq!(actual.facts_owner(), expected.facts_owner());
    assert_eq!(actual.joint_flow(), expected.joint_flow());
    assert_eq!(actual.domain(), expected.domain());
}

fn assert_one_run<S: LayoutScalar>(shape: Shape) {
    let fixture = fixture(shape);
    assert_eq!(fixture.prepared.len(), 1);
    assert_eq!(fixture.prepared[0].source_segments().len(), 1);
    let (tree, bindings) = bind::<S>(&fixture);
    assert_eq!(bindings[0].nodes, [A, C]);
    let flow = capture(&tree, bindings);
    let batch = compute_layout_with_flow(&flow, ROOT, request(100.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let output = &batch.placed_text()[0];
    assert_eq!(output.container(), ROOT);
    assert_eq!(output.nodes(), [A, C]);
    assert_eq!(output.layout().source(), &fixture.source);
    provenance(output.layout(), &fixture.prepared[0]);
    assert!(
        output
            .layout()
            .run_views()
            .all(|run| run.font_size() == 10.0)
    );
    // Retained Ahem is 1000 UPEM, 1000 advance and 800/-200 hhea:
    // at size10, two letters advance20 and the one LH10 line has baseline8.
    assert_eq!(
        packets(output.layout()),
        [
            (text::Range::new(0, 1), 35, 0.0, 8.0, 10.0),
            (text::Range::new(1, 2), 37, 10.0, 8.0, 10.0),
        ]
    );
    let lines = output.layout().lines();
    assert_eq!(lines.len(), 1);
    // Line bounds retain the allocated 100px band; packets above advance 20px.
    assert_eq!(lines[0].bounds(), text::Rect::new(0.0, 0.0, 100.0, 10.0));
    assert_eq!(
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == ROOT)
            .unwrap()
            .output()
            .size
            .height,
        S::from_f64(10.0)
    );
}

#[test]
fn ordinary_and_whole_joint_outputs_each_bind_one_uninterrupted_native_run() {
    for shape in [
        Shape::Single,
        Shape::Flat {
            joined: true,
            empty_middle: false,
        },
    ] {
        assert_one_run::<f32>(shape);
        assert_one_run::<f64>(shape);
    }
}

fn assert_child<S: LayoutScalar>(joined: bool) {
    let fixture = fixture(Shape::Child { joined });
    assert_eq!(fixture.processed.paragraphs().count(), 3);
    let parent = &fixture.prepared[0];
    if joined {
        let owner = parent.captured_owner().unwrap();
        assert_eq!(
            owner.members().collect::<Vec<_>>(),
            [fixture.raws[0].clone(), fixture.raws[2].clone()]
        );
        assert_eq!(owner.facts_owner(), fixture.raws[0]);
        assert_eq!(owner.joint_flow(), fixture.raws[0].joint_flow());
        assert!(owner.joint_flow().is_some());
        let segments: Vec<_> = parent.source_segments().collect();
        assert_eq!(segments.len(), 2);
        assert_ne!(segments[0], segments[1]);
        assert_eq!(
            segments[0].end_opportunity().view().boundary(),
            segments[0].end()
        );
        assert_eq!(
            segments[1].end_opportunity().view().boundary(),
            segments[1].end()
        );
        assert!(segments[0].exit_position().unwrap() < segments[1].entry_position().unwrap());
        assert!(owner.facts_owner().exit_position() < segments[1].exit_position().unwrap());
    }
    let (tree, bindings) = bind::<S>(&fixture);
    if joined {
        assert_eq!(bindings[0].nodes, [A, C]);
        assert_eq!(bindings[1].nodes, [B]);
    }
    let flow = capture(&tree, bindings);
    let batch = compute_layout_with_flow(&flow, ROOT, request(100.0)).unwrap();
    assert_eq!(batch.placed_text().len(), if joined { 2 } else { 3 });
    let mut all = batch
        .placed_text()
        .iter()
        .flat_map(|output| {
            assert_eq!(output.layout().source(), &fixture.source);
            let size = if output.container() == BLOCK {
                20.0
            } else {
                10.0
            };
            assert!(
                output
                    .layout()
                    .run_views()
                    .all(|run| run.font_size() == size)
            );
            let expected = if output.container() == BLOCK {
                &fixture.prepared[1]
            } else if joined || output.nodes() == [A] {
                &fixture.prepared[0]
            } else {
                assert_eq!(output.nodes(), [C]);
                &fixture.prepared[2]
            };
            provenance(output.layout(), expected);
            packets(output.layout())
        })
        .collect::<Vec<_>>();
    all.sort_unstable_by_key(|value| value.0.start);
    // A's LH10 occupies0..10, real B's LH20 occupies10..30, C's LH10
    // occupies30..40. Physical baselines are8, 10+16=26 and30+8=38.
    assert_eq!(
        all,
        [
            (text::Range::new(0, 1), 35, 0.0, 8.0, 10.0),
            (text::Range::new(1, 2), 36, 0.0, 26.0, 20.0),
            (text::Range::new(2, 3), 37, 0.0, 38.0, 10.0),
        ]
    );
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == ROOT)
            .unwrap()
            .output();
        let child = entries
            .iter()
            .find(|entry| entry.node() == BLOCK)
            .unwrap()
            .output();
        assert_eq!(root.size.height, S::from_f64(40.0));
        assert_eq!(child.location.y, S::from_f64(10.0));
        assert_eq!(child.size.height, S::from_f64(20.0));
    }
    if joined {
        let outputs: Vec<_> = batch
            .placed_text()
            .iter()
            .filter(|output| output.container() == ROOT)
            .collect();
        assert_eq!(
            outputs.len(),
            1,
            "one complete joint output, never one output per raw/run"
        );
        assert_eq!(outputs[0].nodes(), [A, C]);
        assert_eq!(outputs[0].layout().lines().len(), 2);
        assert_eq!(
            packets(outputs[0].layout())
                .iter()
                .map(|value| value.0)
                .collect::<Vec<_>>(),
            [text::Range::new(0, 1), text::Range::new(2, 3)]
        );
    }
}

#[test]
fn nonterminal_joint_facts_owner_completes_after_c_and_child_b_is_traversed_once() {
    assert_child::<f32>(true);
    assert_child::<f64>(true);
}

#[test]
fn independent_outputs_accept_separately_structured_native_runs() {
    assert_child::<f32>(false);
    assert_child::<f64>(false);
}

#[test]
fn empty_middle_member_can_own_joint_facts_without_inventing_a_line_or_run_seam() {
    let fixture = fixture(Shape::Flat {
        joined: true,
        empty_middle: true,
    });
    assert_eq!(fixture.raws.len(), 3);
    assert!(fixture.raws[1].enter_position() < fixture.raws[1].exit_position());
    let owner = fixture.prepared[0].captured_owner().unwrap();
    assert_eq!(owner.members().collect::<Vec<_>>(), fixture.raws);
    assert_eq!(owner.facts_owner(), fixture.raws[1]);
    assert!(owner.joint_flow().is_some());
    assert_eq!(fixture.prepared[0].origin_views().count(), 2);
    assert_eq!(fixture.prepared[0].source_segments().len(), 1);
    assert_one_run::<f32>(Shape::Flat {
        joined: true,
        empty_middle: true,
    });
    assert_one_run::<f64>(Shape::Flat {
        joined: true,
        empty_middle: true,
    });
}

#[test]
fn duplicate_whole_output_claim_refuses_overlapping_authentic_members() {
    let fixture = fixture(Shape::Child { joined: true });
    let (tree, mut bindings) = bind::<f64>(&fixture);
    bindings.push(bindings[0].clone());
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        ROOT,
        bindings,
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: vec![],
            child_order: vec![],
        },
        vec![],
    ) else {
        panic!("one authentic joint member inventory cannot be claimed twice");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::OverlappingParagraphs
        ))
    ));
}

#[test]
fn independent_outputs_cannot_split_one_uninterrupted_native_run_at_raw_markers() {
    let fixture = fixture(Shape::Flat {
        joined: false,
        empty_middle: false,
    });
    assert_eq!(
        fixture.prepared.len(),
        2,
        "independent None outputs genuinely prepared"
    );
    for (index, prepared) in fixture.prepared.iter().enumerate() {
        let owner = prepared.captured_owner().unwrap();
        assert_eq!(
            owner.members().collect::<Vec<_>>(),
            [fixture.raws[index].clone()]
        );
        assert!(owner.joint_flow().is_none());
    }
    let (tree, bindings) = bind::<f64>(&fixture);
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.nodes.clone())
            .collect::<Vec<_>>(),
        [vec![A], vec![C]]
    );
    let Err(error) = LayoutFlowInput::try_new(
        &tree,
        ROOT,
        bindings,
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: vec![],
            child_order: vec![],
        },
        vec![],
    ) else {
        panic!("a raw storage marker cannot fabricate native run segmentation");
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::RunMembership
        ))
    ));
}
