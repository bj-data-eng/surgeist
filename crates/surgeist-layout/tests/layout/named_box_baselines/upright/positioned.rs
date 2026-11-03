//! Genuine native OutOfFlow capture uses actual absolute child geometry.
use super::*;
mod supplemental;

struct Parent {
    prepared: t::PreparedText,
    source: t::Source,
}

fn central() -> l::InlineBaselineContext {
    l::InlineBaselineContext::Named(l::BaselineName::Central)
}

fn available<S: l::LayoutScalar>(width: f64, height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(n(width)),
        l::AvailableOf::definite(n(height)),
    )
}

fn captured_parent(bytes: &[u8], size: t::Size, atomic_only: bool, id: t::Id) -> Parent {
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 4.0,
        line_height: t::LineHeight::Absolute(4.0),
        white_space: t::WhiteSpace::Preserve,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    };
    let mut source = t::Source::identified(
        if atomic_only { "" } else { "\u{4e00}\u{4e00}" },
        t::Id::from_u64(1_554_100 + u64::from(atomic_only)),
        1,
    );
    // This is the authored fixture identity of the actual bound node. All
    // capture/preparation capabilities are issued by Text, never synthesized.
    source.inline_box(t::InlineBox::new(
        id,
        t::InlineBoxKind::OutOfFlow,
        usize::from(!atomic_only),
        size,
    ));
    use t::SourceProcessingEvent as E;
    let capture = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: if atomic_only {
                vec![]
            } else {
                [t::Range::new(0, 1), t::Range::new(1, 2)]
                    .into_iter()
                    .map(|range| t::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: t::ParagraphOriginKind::Authored,
                    })
                    .collect()
            },
            controls: vec![],
            connections: if atomic_only {
                vec![]
            } else {
                vec![t::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: t::ParagraphConnectionKind::Continuous,
                }]
            },
            boundary_policies: if atomic_only {
                vec![
                    t::ParagraphBoundaryDefinition::AtomicBefore(0),
                    t::ParagraphBoundaryDefinition::AtomicAfter(0),
                ]
            } else {
                vec![
                    t::ParagraphBoundaryDefinition::AtomicBefore(0),
                    t::ParagraphBoundaryDefinition::AtomicAfter(0),
                    t::ParagraphBoundaryDefinition::Connection(0),
                ]
            }
            .into_iter()
            .map(|definition| t::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: t::Wrap::Word,
                following_wrap: t::Wrap::Word,
                ancestor_wrap: t::Wrap::Word,
            })
            .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: t::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events: if atomic_only {
                vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Atomic {
                        atomic: 0,
                        affinity: t::Affinity::After,
                    },
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ]
            } else {
                vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Origin {
                        origin: 0,
                        affinity: t::Affinity::After,
                    },
                    E::Atomic {
                        atomic: 0,
                        affinity: t::Affinity::After,
                    },
                    E::Connection(0),
                    E::Origin {
                        origin: 1,
                        affinity: t::Affinity::After,
                    },
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ]
            },
        })
        .unwrap(),
    );
    let typography = t::AnalysisTypography::Upright {
        computed_direction: t::ParagraphBaseDirection::RightToLeft,
    };
    let analysis = t::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    let wrapping = analysis
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analysis = analysis.with_wrapping_policies(wrapping).unwrap();
    assert_eq!(analysis.typography(), typography);
    assert_eq!(analysis.direction(), t::Direction::LeftToRight);
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    assert_eq!(
        system.registered_font_faces().next().unwrap().bytes(),
        bytes
    );
    let prepared = system
        .prepare(
            &analysis,
            t::PreparationOptions::default().with_orientation(t::LineOrientation::Vertical),
        )
        .expect(
            "the real captured OutOfFlow parent must satisfy the independently GREEN Text producer",
        );
    assert_eq!(
        prepared
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        vec![capture.captured_paragraph(0).unwrap()]
    );
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(
        prepared.origin_views().count(),
        if atomic_only { 0 } else { 2 }
    );
    assert_eq!(prepared.atomic_views().count(), 1);
    let participants = prepared.participants().collect::<Vec<_>>();
    let atomic = if atomic_only {
        let [t::PreparedParticipant::Atomic(atomic)] = participants.as_slice() else {
            panic!("source-free input must retain exactly its actual atomic occurrence");
        };
        atomic
    } else {
        let [
            t::PreparedParticipant::Text(before),
            t::PreparedParticipant::Atomic(atomic),
            t::PreparedParticipant::Text(after),
        ] = participants.as_slice()
        else {
            panic!("actual processed participant order must be Hani, atomic, Hani");
        };
        assert_eq!(before.view().source_range(), t::Range::new(0, 1));
        assert_eq!(after.view().source_range(), t::Range::new(1, 2));
        atomic
    };
    let atomic_view = atomic.view();
    assert_eq!(atomic_view.original_box().id(), id);
    assert_eq!(atomic_view.original_box().size(), size);
    Parent { prepared, source }
}

fn selected_parent(parent: &Parent) -> t::FinalizedLine {
    let prepared = &parent.prepared;
    let end = prepared
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    prepared
        .select(t::LineSelectionRequest {
            inline_boxes: None,
            segment: prepared.source_segments().next().unwrap(),
            start: prepared.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: t::PreparedTermination::Ordinary(end),
            participants: prepared.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: prepared
                .atomic_views()
                .map(|row| t::AtomicAllocation {
                    occurrence: row.handle(),
                    allocation: t::AtomicLineAllocation::OutOfFlow,
                })
                .collect(),
            controls: vec![],
            boundary_context: vec![],
            line_strut: Some(STRUT),
            line_box_struts: vec![],
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation: t::LineOrientation::Vertical,
                start: t::LineStart::Paragraph,
                end: t::LineEnd::Paragraph,
                available_inline: Some(22.0),
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap()
}

fn explicit_parent(
    parent: &Parent,
    line: t::FinalizedLine,
    c: f64,
    border: t::Rect,
    content: t::Rect,
    physical_baseline: f64,
) -> t::Layout {
    t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: parent.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 0.0,
                baseline: c,
            },
            orientation: t::LineOrientation::Vertical,
            atomics: vec![t::AtomicPlacement {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                border,
                content,
                baseline: physical_baseline,
            }],
        })],
    })
    .unwrap()
}

fn qualify_parent(parent: &Parent) {
    let line = selected_parent(parent);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (8.0, 8.0, 10.0)
    );
    let slot = line.atomic_slots().next().unwrap();
    assert_eq!(
        slot.occurrence,
        parent.prepared.atomic_views().next().unwrap().handle()
    );
    assert_eq!(slot.origin, t::Point::new(4.0, 8.0));
    assert_eq!(slot.allocation, t::AtomicLineAllocation::OutOfFlow);
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.glyph_basis(), t::GlyphBasis::Unrotated);
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(274, 4.0), (274, 4.0)]);
    // A positioned box is supplied by its external physical owner, independent
    // of this standalone line's Central100 anchor and zero-flow allocation.
    let physical = t::Rect::new(34.0, 4.0, 6.0, 14.0);
    let text = explicit_parent(parent, line, 100.0, physical, physical, 36.0);
    assert_eq!(text.source(), &parent.source);
    assert_eq!(text.inline_boxes()[0].rect(), physical);
}

fn absolute_input<S: l::LayoutScalar>() -> l::NodeInputOf<S> {
    let mut input = root(
        l::WritingMode::VerticalLr,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.position = l::Position::Absolute;
    input.size = l::Size::new(px(6.0), px(14.0));
    input
}

fn child_binding(child: &Fixture, container: u32, text: u32) -> l::TextParagraphBinding<u32> {
    l::TextParagraphBinding {
        inline_box_fragments: surgeist_layout::InlineBoxFragmentationDemand::NotRequested,
        prepared: child.prepared.clone(),
        container,
        nodes: vec![text],
        origins: vec![l::TextOriginBinding {
            origin: child.prepared.origin_views().next().unwrap().handle(),
            node: text,
        }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    }
}

fn placed<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> &t::Layout {
    batch
        .placed_text()
        .iter()
        .find(|row| row.container() == container)
        .unwrap()
        .layout()
}

fn compute<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::TextParagraphBinding<u32>>,
    changed: &[u32],
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow =
        l::LayoutFlowInput::try_new(host, 0, bindings, l::WrappingInput::default(), Vec::new())
            .expect("genuine OutOfFlow occurrence must bind to its actual absolute child");
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(available(40.0, 22.0)).unwrap(),
        changed,
    )
}

fn qualify_child<S: l::LayoutScalar>(child: &Fixture) {
    let mut input = absolute_input::<S>();
    input.position = l::Position::Relative;
    let mut host = Host::new(input);
    let binding = child_binding(child, 0, 1);
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(binding.origins[0].origin.clone())),
    );
    let flow = l::LayoutFlowInput::try_new(
        &host,
        0,
        vec![binding],
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = l::compute_layout_with_flow(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(available(6.0, 14.0)).unwrap(),
    )
    .unwrap();
    let text = placed(&batch, 0);
    assert_eq!(text.source(), &child.source);
    assert_eq!(text.lines()[0].baseline(), 2.0);
    assert_eq!(text.lines()[0].bounds(), t::Rect::new(0.0, 0.0, 4.0, 14.0));
    // Performed nonleaf Text computations need not publish a root cache row.
    // The owning public placement above proves the physical child reference2;
    // BaselinesOf::first_or_synthesize_block retains that physical coordinate
    // by contract, rather than converting it to a flow-block-start distance.
    let pair = node(batch.unrounded_entries(), 0)
        .named_baselines
        .get(l::BaselineName::Central);
    assert_eq!(
        (pair.first(), pair.last()),
        (l::BaselineFactOf::Unknown, l::BaselineFactOf::Unknown)
    );
}

fn scene<S: l::LayoutScalar>(
    parent: &Parent,
    child: &Fixture,
    mode: l::WritingMode,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input = root(mode, l::Direction::Ltr, central());
    input.size = l::Size::new(px(40.0), px(22.0));
    let mut host = Host::new(input);
    let origins = parent
        .prepared
        .origin_views()
        .map(|row| row.handle())
        .collect::<Vec<_>>();
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[0].clone())),
    );
    assert_eq!(
        host.push(Some(0), l::LayoutInputOf::box_input(absolute_input())),
        2
    );
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[1].clone())),
    );
    let inner = child_binding(child, 2, 4);
    host.push(
        Some(2),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(inner.origins[0].origin.clone())),
    );
    let outer = l::TextParagraphBinding {
        inline_box_fragments: surgeist_layout::InlineBoxFragmentationDemand::NotRequested,
        prepared: parent.prepared.clone(),
        container: 0,
        nodes: vec![1, 2, 3],
        origins: origins
            .into_iter()
            .zip([1, 3])
            .map(|(origin, node)| l::TextOriginBinding { origin, node })
            .collect(),
        atomics: vec![l::TextAtomicBinding {
            occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
            node: 2,
        }],
        controls: vec![],
        line_strut: Some(STRUT),
    };
    (host, vec![outer, inner])
}

fn native_positioned<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(2),
    );
    qualify_parent(&parent);
    for (mode, c, child_x, child_baseline) in [
        (l::WritingMode::VerticalRl, 32.0, 34.0, 36.0),
        (l::WritingMode::VerticalLr, 2.0, 0.0, 2.0),
    ] {
        let (host, bindings) = scene::<S>(&parent, &child, mode);
        let batch = compute(&host, bindings, &[])
            .expect("actual native absolute geometry must compose after public Text succeeds");
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let output = node(entries, 2);
            assert_eq!(output.location, l::Point::new(n(child_x), n(4.0)));
            assert_eq!(output.size, l::Size::new(n(6.0), n(14.0)));
            let pair = output.named_baselines.get(l::BaselineName::Central);
            assert_eq!(
                (pair.first(), pair.last()),
                (l::BaselineFactOf::Unknown, l::BaselineFactOf::Unknown)
            );
        }
        let text = placed(&batch, 0);
        assert_eq!(text.source(), &parent.source);
        assert_eq!(text.lines().len(), 1);
        assert_eq!(text.lines()[0].baseline(), c);
        let physical = t::Rect::new(child_x, 4.0, 6.0, 14.0);
        assert_eq!(text.inline_boxes()[0].kind(), t::InlineBoxKind::OutOfFlow);
        assert_eq!(text.inline_boxes()[0].rect(), physical);
        assert_eq!(placed(&batch, 2).lines()[0].baseline(), child_baseline);
        let receipt = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|row| matches!(row.owner(), l::InlineFragmentOwner::Node(1)))
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(
            (
                receipt.feedback().advance,
                receipt.feedback().baseline,
                receipt.feedback().line_height
            ),
            (8.0, 8.0, 10.0)
        );
        assert_eq!(
            receipt.atomic_slots().next().unwrap().origin,
            t::Point::new(4.0, 8.0)
        );
        assert_eq!(
            receipt.selected_line().request().atomics[0].allocation,
            t::AtomicLineAllocation::OutOfFlow
        );
        // Public explicit-placement cache identity includes the supplied physical
        // atomic baseline, without adding a fresh opaque placement identity.
        // Reuse only the actual finalized authority; all placement numbers here
        // come from the independent static-edge/absolute-box derivation above.
        let expected = explicit_parent(
            &parent,
            receipt.clone(),
            c,
            physical,
            physical,
            child_baseline,
        );
        assert_eq!(text.key(), expected.key());
        let wrong_y = explicit_parent(&parent, receipt.clone(), c, physical, physical, 6.0);
        assert_ne!(expected.key(), wrong_y.key());
        let pens = text
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.glyph_basis(), t::GlyphBasis::Unrotated);
                run.glyphs()
                    .map(|glyph| (run.origin().x, run.origin().y + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(pens, [(c, 0.0), (c, 4.0)]);
    }
}

#[test]
fn native_upright_positioned_static_anchor_and_physical_baseline_f32() {
    native_positioned::<f32>();
}
#[test]
fn native_upright_positioned_static_anchor_and_physical_baseline_f64() {
    native_positioned::<f64>();
}
