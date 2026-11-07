#![forbid(unsafe_code)]
//! Real Text processing, Ahem preparation and Layout placement, joined to Paint
//! through exact caller-owned producer associations. Resolved CSS paint facts
//! are native inputs; this fixture does not implement a computed Style adapter.

use std::{convert::Infallible, ptr, sync::Arc};
use surgeist_layout as l;
use surgeist_paint as p;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!("../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

struct Host {
    inputs: Vec<l::LayoutInput>,
    children: Vec<Vec<usize>>,
    non_box: l::NodeInput,
}

impl l::Traverse for Host {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        self.children[node].iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.children[node].len()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children[node][index]
    }
}

impl l::LayoutTree for Host {
    type MeasureError = Infallible;
    fn node_input(&self, node: usize) -> &l::NodeInput {
        self.inputs[node].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> l::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn px(value: f32) -> l::PreferredSize {
    l::PreferredSize::value(l::LengthPercentageOf::px(value).unwrap())
}

fn host() -> Host {
    Host {
        inputs: (0..3)
            .map(|node| {
                l::LayoutInput::box_input(l::NodeInput {
                    display: if node == 0 {
                        l::Display::Flex
                    } else {
                        l::Display::Block
                    },
                    flex_direction: l::FlexDirection::RowReverse,
                    size: l::Size::new(
                        px(if node == 0 { 200.0 } else { 100.0 }),
                        l::PreferredSize::AUTO,
                    ),
                    ..l::NodeInput::DEFAULT
                })
            })
            .collect(),
        children: vec![vec![1, 2], vec![], vec![]],
        non_box: l::NodeInput::non_box(),
    }
}

fn paragraph(host: &mut Host, container: usize, parts: &[&str]) -> l::TextParagraphBinding<usize> {
    use t::SourceProcessingEvent as E;
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Collapse,
        wrap: t::Wrap::Word,
        hyphens: t::Hyphens::None,
        ..t::Style::default()
    };
    let raw = parts.concat();
    let mut source = t::Source::new(raw.clone());
    source.span(t::Range::new(0, raw.encode_utf16().count()), style.clone());
    let mut origins = Vec::new();
    let mut connections = Vec::new();
    let mut boundary_policies = Vec::new();
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    let mut at = 0;
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            let connection = connections.len();
            connections.push(t::SourceProcessingConnection {
                before_origin: index - 1,
                after_origin: index,
                kind: t::ParagraphConnectionKind::Continuous,
            });
            boundary_policies.push(t::ParagraphBoundaryPolicy {
                definition: t::ParagraphBoundaryDefinition::Connection(connection),
                preceding_wrap: style.wrap,
                following_wrap: style.wrap,
                ancestor_wrap: style.wrap,
            });
            events.push(E::Connection(connection));
        }
        let range = t::Range::new(at, at + part.encode_utf16().count());
        origins.push(t::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: t::ParagraphOriginKind::Authored,
        });
        events.push(E::Origin {
            origin: index,
            affinity: t::Affinity::After,
        });
        at = range.end;
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            source,
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            origins,
            controls: Vec::new(),
            connections,
            boundary_policies,
            replacements: Vec::new(),
            computed_line_boundaries: Vec::new(),
            scopes: vec![t::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style.clone(),
                used_direction: t::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let analyzed = t::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: style.wrap,
        })
        .collect();
    let mut system = t::System::default();
    system
        .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let prepared = system
        .prepare(
            &analyzed.with_wrapping_policies(wrapping).unwrap(),
            t::PreparationOptions::default(),
        )
        .unwrap();
    let mut nodes = Vec::new();
    let mut bindings = Vec::new();
    for origin in prepared.origin_views() {
        let node = host.inputs.len();
        host.inputs
            .push(l::LayoutInput::inline_text(l::TextParticipation::new(
                origin.handle(),
            )));
        host.children.push(Vec::new());
        host.children[container].push(node);
        nodes.push(node);
        bindings.push(l::TextOriginBinding {
            origin: origin.handle(),
            node,
        });
    }
    assert_eq!(nodes.len(), parts.len());
    l::TextParagraphBinding {
        prepared,
        inline_box_fragments: l::InlineBoxFragmentationDemand::NotRequested,
        container,
        nodes,
        origins: bindings,
        atomics: Vec::new(),
        controls: Vec::new(),
        line_strut: None,
    }
}

fn root_request() -> l::LayoutRootRequest {
    let extent = l::Size::new(Some(l::NonNegativeFiniteOf::new(200.0).unwrap()), None);
    l::LayoutRootRequest::standalone(
        l::SizingRequest::new(
            l::Size::new(
                l::AxisSizingRequest::new(
                    l::AvailableSpace::Definite(l::NonNegativeFiniteOf::new(200.0).unwrap()),
                    l::SizingOperation::Normal,
                ),
                l::AxisSizingRequest::new(
                    l::AvailableSpace::Indefinite,
                    l::SizingOperation::Normal,
                ),
            ),
            l::OrthogonalFallback::new(
                extent,
                l::NearestScrollport::Absent,
                l::Size::splat(l::OrthogonalFallbackLimit::Unbounded),
            ),
        ),
        l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr),
        extent,
    )
}

fn source_index(batch: &l::CompletedLayoutBatch<usize>, node: usize) -> l::SourceIndex {
    let output = batch.node_layout(node).unwrap();
    if let Some(single) = output.single_box() {
        single.final_output().source_index
    } else {
        output.non_box().unwrap().source_index()
    }
}

struct NativeLine<'a> {
    binding: &'a l::TextParagraphBinding<usize>,
    placed: &'a l::PlacedTextOutput<usize>,
    line: t::FinalizedLine,
    local_index: usize,
}

#[test]
fn issued_text_fragments_keep_exact_binding_owner_and_line_when_native_line_indices_repeat() {
    let mut host = host();
    let first = paragraph(&mut host, 1, &["A", "", "B"]);
    let second = paragraph(&mut host, 2, &["C", "D"]);
    assert_eq!(first.nodes, [3, 4, 5]);
    assert_eq!(second.nodes, [6, 7]);
    let flow = l::LayoutFlowInput::try_new(
        &host,
        0,
        vec![first.clone(), second.clone()],
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = l::compute_layout_with_flow(&flow, 0, root_request()).unwrap();
    let single = |node| {
        batch
            .node_layout(node)
            .unwrap()
            .single_box()
            .unwrap()
            .final_output()
    };
    assert_eq!(single(1).location.x, 100.0);
    assert_eq!(single(2).location.x, 0.0);
    // Paint's line slots are qualified by actual binding, placement and issued
    // finalized-line owner. Both independent native runs use local index zero.
    let native_lines: Vec<_> = [&second, &first]
        .into_iter()
        .map(|binding| {
            let placed = batch
                .placed_text()
                .iter()
                .find(|placed| placed.container() == binding.container)
                .unwrap();
            assert_eq!(placed.nodes(), binding.nodes);
            assert_eq!(placed.layout().lines().len(), 1);
            assert_eq!(
                placed.layout().analyzed_source().identity(),
                binding.prepared.analyzed_source().identity()
            );
            let row = batch
                .final_inline_fragments()
                .iter()
                .find(|row| row.owner() == &l::InlineFragmentOwner::Node(binding.nodes[0]))
                .unwrap();
            NativeLine {
                binding,
                placed,
                line: row.fragment().finalized_fragment().finalized_line(),
                local_index: row.fragment().line_index(),
            }
        })
        .collect();
    assert_eq!(native_lines[0].local_index, 0);
    assert_eq!(native_lines[1].local_index, 0);
    assert_ne!(native_lines[0].line, native_lines[1].line);
    assert_ne!(
        native_lines[0].binding.prepared,
        native_lines[1].binding.prepared
    );
    let box_nodes = [2, 0, 1];
    let box_id = |node| p::BoxId::new(box_nodes.iter().position(|&value| value == node).unwrap());
    let boxes: Vec<_> = box_nodes
        .iter()
        .map(|&node| {
            p::BoxFacts::new(
                if node == 0 { None } else { Some(box_id(0)) },
                source_index(&batch, node),
                if node == 0 {
                    p::ChildOrder::Flex
                } else {
                    p::ChildOrder::Source
                },
                if node == 0 {
                    p::Participation::Ordinary
                } else {
                    p::Participation::FlexItem(l::ItemOrder::new(0))
                },
                p::Position::Static,
                p::ZIndex::Auto,
                p::ContextEffect::None,
            )
        })
        .collect();
    let paints = [p::BoxPaint::new(p::BoxPaintKind::Block, p::FloatParticipation::None); 3];
    let text_nodes = [7, 4, 3, 6, 5];
    let text_id =
        |node| p::TextId::new(text_nodes.iter().position(|&value| value == node).unwrap());
    let texts: Vec<_> = text_nodes
        .iter()
        .map(|&node| {
            p::TextSource::new(
                box_id(if first.nodes.contains(&node) { 1 } else { 2 }),
                source_index(&batch, node),
            )
        })
        .collect();
    // Root and container occurrence slots are independent of box storage.
    let root_fragment = p::FragmentId::new(1);
    let container_fragment = |node| p::FragmentId::new(if node == 1 { 2 } else { 0 });
    let lines = [
        p::PaintLine::new(container_fragment(2), 0),
        p::PaintLine::new(container_fragment(1), 0),
    ];
    let mut fragments = vec![
        p::PaintFragment::box_fragment(box_id(2), Some(root_fragment), None, 0),
        p::PaintFragment::box_fragment(box_id(0), None, None, 0),
        p::PaintFragment::box_fragment(box_id(1), Some(root_fragment), None, 0),
    ];
    let mut native_fragments = vec![None; 3];
    // Deliberately reverse the producer association table. Paint gets explicit
    // source IDs and occurrence order rather than inheriting its storage order.
    for row in batch.final_inline_fragments().iter().rev() {
        let l::InlineFragmentOwner::Node(node) = row.owner() else {
            panic!("fixture has only origin-owned fragments")
        };
        let line_slot = if first.nodes.contains(node) { 1 } else { 0 };
        let association = &native_lines[line_slot];
        let origin = association
            .binding
            .origins
            .iter()
            .find(|origin| origin.node == *node)
            .unwrap();
        assert_eq!(
            row.fragment().finalized_fragment().view().owner(),
            t::FinalizedFragmentOwner::Origin(origin.origin.clone())
        );
        assert_eq!(
            row.fragment().finalized_fragment().finalized_line(),
            association.line
        );
        assert_eq!(row.fragment().line_index(), association.local_index);
        if *node != 4 {
            let bounds = row
                .fragment()
                .rect()
                .expect("one Ahem character has source geometry");
            assert_eq!(bounds.size(), l::Size::new(10.0, 10.0));
            assert_eq!(
                bounds.origin().x,
                if matches!(*node, 3 | 6) { 0.0 } else { 10.0 }
            );
        }
        fragments.push(p::PaintFragment::text_fragment(
            text_id(*node),
            container_fragment(association.binding.container),
            p::LineId::new(line_slot),
            0,
        ));
        native_fragments.push(Some(row));
    }
    let text_fragment = |node| {
        p::FragmentId::new(
            native_fragments
                .iter()
                .position(|row| {
                    row.is_some_and(|row| row.owner() == &l::InlineFragmentOwner::Node(node))
                })
                .unwrap(),
        )
    };
    let text =
        |node| p::PaintPrimitive::fragment(text_fragment(node), p::FragmentPaintKind::Text, 0);
    let background = |node| {
        p::PaintPrimitive::fragment(
            container_fragment(node),
            p::FragmentPaintKind::BackgroundColor,
            0,
        )
    };
    let primitives = [
        text(7),
        p::PaintPrimitive::decoration(text_fragment(3), box_id(1), p::DecorationKind::Underline, 0),
        p::PaintPrimitive::canvas(p::CanvasPaintKind::BackgroundColor, 0),
        text(5),
        background(2),
        text(3),
        p::PaintPrimitive::decoration(text_fragment(3), box_id(0), p::DecorationKind::Underline, 0),
        background(1),
        text(6),
    ];
    // Empty source custody is an actual issued occurrence and receives no
    // invented Text primitive, regardless of its optional producer geometry.
    let empty = native_fragments[text_fragment(4).get()].unwrap();
    assert_eq!(
        empty
            .fragment()
            .finalized_fragment()
            .view()
            .aggregate_range(),
        Some(t::Range::new(1, 1))
    );
    let request = p::PaintRequest::new(
        &boxes,
        &paints,
        &texts,
        &lines,
        &fragments,
        &primitives,
        box_id(0),
    );
    let plan = p::plan_paint_order(
        request,
        p::PaintOptions::new(
            p::PaintLimits::unlimited(),
            p::ProfileMode::Summary { timing: false },
        ),
    )
    .result
    .unwrap();
    let events: Vec<_> = plan
        .events()
        .map(|event| match event {
            p::PaintOrderEvent::BeginContext(owner) => ('[', box_nodes[owner.id().get()]),
            p::PaintOrderEvent::Draw(primitive) => ('d', primitive.id().get()),
            p::PaintOrderEvent::EndContext(owner) => (']', box_nodes[owner.id().get()]),
        })
        .collect();
    // Source-leading item1 paints completely before item2 despite row-reverse
    // coordinates. Applicable underlines use applying-element ancestor order.
    assert_eq!(
        events,
        [
            ('[', 0),
            ('d', 2),
            ('d', 7),
            ('d', 6),
            ('d', 1),
            ('d', 5),
            ('d', 3),
            ('d', 4),
            ('d', 8),
            ('d', 0),
            (']', 0)
        ]
    );
    for event in plan.events() {
        if let p::PaintOrderEvent::Draw(primitive) = event {
            assert!(ptr::eq(
                primitive.record(),
                &primitives[primitive.id().get()]
            ));
            if let Some(fragment) = primitive.fragment() {
                assert!(ptr::eq(fragment.record(), &fragments[fragment.id().get()]));
                if let Some(row) = native_fragments[fragment.id().get()] {
                    let p::FragmentOwner::Text(id) = fragment.record().owner() else {
                        panic!("actual text occurrence retains a text source")
                    };
                    let node = text_nodes[id.get()];
                    assert_eq!(row.owner(), &l::InlineFragmentOwner::Node(node));
                    assert!(ptr::eq(fragment.text_source().unwrap(), &texts[id.get()]));
                    let line = fragment.line().unwrap();
                    assert!(ptr::eq(line.record(), &lines[line.id().get()]));
                    let native = &native_lines[line.id().get()];
                    assert_eq!(fragment.owner().id(), box_id(native.binding.container));
                    assert_eq!(
                        line.container().id(),
                        container_fragment(native.placed.container())
                    );
                    assert_eq!(
                        row.fragment().finalized_fragment().finalized_line(),
                        native.line
                    );
                    assert_eq!(native.line.prepared_text(), native.binding.prepared);
                    let original = batch
                        .final_inline_fragments()
                        .iter()
                        .find(|original| ptr::eq(*original, row))
                        .unwrap();
                    assert!(ptr::eq(
                        row.fragment().finalized_fragment(),
                        original.fragment().finalized_fragment()
                    ));
                    assert_eq!(row.fragment().rect(), original.fragment().rect());
                    assert_eq!(row.fragment().baseline(), original.fragment().baseline());
                }
            }
        }
    }
    let empty_view = plan.fragment_view(text_fragment(4)).unwrap();
    assert_eq!(
        empty_view.record().owner(),
        p::FragmentOwner::Text(text_id(4))
    );
    assert_eq!(empty_view.line().unwrap().id(), p::LineId::new(1));
    assert_eq!(
        empty_view.text_source().unwrap().source_index(),
        source_index(&batch, 4)
    );
    assert!(!plan.events().any(|event| matches!(event, p::PaintOrderEvent::Draw(primitive) if primitive.fragment().is_some_and(|fragment| fragment.id() == empty_view.id()))));
}
