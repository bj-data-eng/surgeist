//! A real native Text child supplies Central2 inside its physical6x14 border.
//! Parent flow progression and Text's right-to-left cell map stay distinct.
use super::*;
use surgeist_text::font::FontBaseline;
mod supplemental;

fn central() -> l::baseline::InlineBaselineContext {
    l::baseline::InlineBaselineContext::Named(FontBaseline::Central)
}

fn available<S: l::scalar::LayoutScalar>(
    width: f64,
    height: f64,
) -> surgeist_layout::constraint::SizingRequestOf<S> {
    crate::layout::native_request::sizing(
        surgeist_layout::geometry::Size::new(Some(n(width)), Some(n(height))),
        surgeist_layout::geometry::Size::new(
            surgeist_layout::constraint::SizingOperation::Normal,
            surgeist_layout::constraint::SizingOperation::Normal,
        ),
    )
}

fn solve<S: l::scalar::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::text_layout::TextParagraphBinding<u32>>,
    width: f64,
    height: f64,
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let flow = l::text_layout::LayoutFlowInput::try_new(
        host,
        0,
        bindings,
        l::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .expect("actual processed captures and issued occurrence roles must bind to their real nodes");
    l::compute_layout_with_flow(&flow, 0, {
        let sizing_request = available(width, height);
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(host, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(host, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    })
    .expect("native atomic adapter must retain physical size and match the child's actual Central")
}

fn placed<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> &t::layout::Layout {
    batch
        .placed_text()
        .iter()
        .find(|row| row.container() == container)
        .unwrap()
        .layout()
}

fn assert_central<S: l::scalar::LayoutScalar>(
    facts: l::baseline::NamedBaselinesOf<S>,
    expected: f64,
) {
    let pair = facts.get(FontBaseline::Central);
    let coordinate = l::baseline::BaselineCoordinateOf::try_new(
        l::geometry::PhysicalAxis::Horizontal,
        n(expected),
    )
    .unwrap();
    assert_eq!(
        pair.first(),
        l::baseline::BaselineFactOf::Available(coordinate)
    );
    assert_eq!(
        pair.last(),
        l::baseline::BaselineFactOf::Available(coordinate)
    );
}

fn child_input<S: l::scalar::LayoutScalar>() -> l::node::NodeInputOf<S> {
    let mut input = root(
        l::geometry::WritingMode::VerticalLr,
        l::geometry::Direction::Ltr,
        central(),
    );
    input.display = l::box_model::Display::InlineBlock;
    input.size = l::geometry::Size::new(px(6.0), px(14.0));
    input
}

fn child_binding(
    fixture: &Fixture,
    container: u32,
    text: u32,
) -> l::text_layout::TextParagraphBinding<u32> {
    let origin = fixture.prepared.origin_views().next().unwrap().handle();
    l::text_layout::TextParagraphBinding {
        inline_box_fragments:
            surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
        prepared: fixture.prepared.clone(),
        container,
        nodes: vec![text],
        origins: vec![l::text_layout::TextOriginBinding { origin, node: text }],
        atomics: vec![],
        controls: vec![],
        // Its real font2/2 establishes Central2. The parent's separately
        // allocated8/2 strut must never be copied into this child binding.
        line_strut: None,
    }
}

fn qualify_child<S: l::scalar::LayoutScalar>(fixture: &Fixture) {
    let mut host = Host::<S>::new(child_input());
    let binding = child_binding(fixture, 0, 1);
    assert_eq!(
        host.push(
            Some(0),
            l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                binding.origins[0].origin.clone(),
            )),
        ),
        1
    );
    let batch = solve(&host, vec![binding], 6.0, 14.0);
    for entries in [
        crate::layout::observation::phase(&batch, false),
        crate::layout::observation::phase(&batch, true),
    ] {
        let output = node(entries, 0);
        assert_eq!(output.size, l::geometry::Size::new(n(6.0), n(14.0)));
        assert_central(output.named_baselines, 2.0);
    }
    let text = placed(&batch, 0);
    assert_eq!(text.source(), &fixture.source);
    assert_eq!(text.orientation(), t::selection::LineOrientation::Vertical);
    assert_eq!(text.lines().len(), 1);
    assert_eq!(text.lines()[0].baseline(), 2.0);
    assert_eq!(
        text.lines()[0].bounds(),
        t::geometry::Rect::new(0.0, 0.0, 4.0, 14.0)
    );
    let mut saw_source = false;
    for row in batch.unrounded_inline_fragments() {
        if matches!(row.owner(), l::text_layout::InlineFragmentOwner::Node(1)) {
            saw_source = true;
            let fragment = row.fragment();
            let line = fragment.finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), fixture.prepared);
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (8.0, 2.0, 4.0)
            );
            assert_eq!(line.selected_line().request().line_strut, None);
            assert_eq!(fragment.baseline(), l::geometry::Point::new(n(2.0), n(0.0)));
        }
    }
    assert!(
        saw_source,
        "actual child source must retain its finalized receipt"
    );
}

struct Parent {
    prepared: t::prepared::PreparedText,
    source: t::source::Source,
}

fn parent(bytes: &[u8]) -> Parent {
    let parent = captured_parent(
        bytes,
        t::geometry::Size::new(6.0, 14.0),
        false,
        t::source::Id::from_u64(2),
    );
    let atomic = parent.prepared.atomic_views().next().unwrap().handle();
    qualify_parent_text(&parent.prepared, atomic);
    parent
}

fn captured_parent(
    bytes: &[u8],
    size: t::geometry::Size,
    atomic_only: bool,
    id: t::source::Id,
) -> Parent {
    let style = t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 4.0,
        line_height: t::style::LineHeight::Absolute(4.0),
        white_space: t::style::WhiteSpace::Preserve,
        wrap: t::style::Wrap::Word,
        ..t::style::Style::default()
    };
    let mut source = t::source::Source::identified(
        if atomic_only { "" } else { "\u{4e00}\u{4e00}" },
        t::source::Id::from_u64(1_552_100 + u64::from(atomic_only)),
        1,
    );
    // This is the authored fixture identity of the actual bound node. All
    // capture/preparation capabilities are issued by Text, never synthesized.
    source.inline_box(t::source::InlineBox::new(
        id,
        t::source::InlineBoxKind::InFlow,
        usize::from(!atomic_only),
        size,
    ));
    use t::capture::SourceProcessingEvent as E;
    let capture = Arc::new(
        t::capture::process_contextual_source(t::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::capture::SourceProcessingEnvelope::Complete,
            origins: if atomic_only {
                vec![]
            } else {
                [t::source::Range::new(0, 1), t::source::Range::new(1, 2)]
                    .into_iter()
                    .map(|range| t::paragraph::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: t::paragraph::ParagraphOriginKind::Authored,
                    })
                    .collect()
            },
            controls: vec![],
            connections: if atomic_only {
                vec![]
            } else {
                vec![t::capture::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: t::paragraph::ParagraphConnectionKind::Continuous,
                }]
            },
            boundary_policies: if atomic_only {
                vec![
                    t::paragraph::ParagraphBoundaryDefinition::AtomicBefore(0),
                    t::paragraph::ParagraphBoundaryDefinition::AtomicAfter(0),
                ]
            } else {
                vec![
                    t::paragraph::ParagraphBoundaryDefinition::AtomicBefore(0),
                    t::paragraph::ParagraphBoundaryDefinition::AtomicAfter(0),
                    t::paragraph::ParagraphBoundaryDefinition::Connection(0),
                ]
            }
            .into_iter()
            .map(|definition| t::paragraph::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: t::style::Wrap::Word,
                following_wrap: t::style::Wrap::Word,
                ancestor_wrap: t::style::Wrap::Word,
            })
            .collect(),
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::capture::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::capture::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::capture::SourceProcessingParagraph {
                base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: if atomic_only {
                vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Atomic {
                        atomic: 0,
                        affinity: t::layout::Affinity::After,
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
                        affinity: t::layout::Affinity::After,
                    },
                    E::Atomic {
                        atomic: 0,
                        affinity: t::layout::Affinity::After,
                    },
                    E::Connection(0),
                    E::Origin {
                        origin: 1,
                        affinity: t::layout::Affinity::After,
                    },
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ]
            },
        })
        .unwrap(),
    );
    let typography = t::analyzed::AnalysisTypography::Upright {
        computed_direction: t::paragraph::ParagraphBaseDirection::RightToLeft,
    };
    let analysis = t::analyzed::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    let wrapping = analysis
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::analyzed::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: t::style::Wrap::Word,
        })
        .collect();
    let analysis = analysis.with_wrapping_policies(wrapping).unwrap();
    assert_eq!(analysis.typography(), typography);
    assert_eq!(analysis.direction(), t::style::Direction::LeftToRight);
    let mut system = t::System::default();
    system
        .register_fonts(bytes.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let prepared = system
        .prepare(
            &analysis,
            t::prepared::PreparationOptions::default()
                .with_orientation(t::selection::LineOrientation::Vertical),
        )
        .expect(
            "the real captured InFlow parent must satisfy the independently GREEN Text producer",
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
        let [t::prepared::PreparedParticipant::Atomic(atomic)] = participants.as_slice() else {
            panic!("source-free input must retain exactly its actual atomic occurrence");
        };
        atomic
    } else {
        let [
            t::prepared::PreparedParticipant::Text(before),
            t::prepared::PreparedParticipant::Atomic(atomic),
            t::prepared::PreparedParticipant::Text(after),
        ] = participants.as_slice()
        else {
            panic!("actual processed participant order must be Hani, atomic, Hani");
        };
        assert_eq!(before.view().source_range(), t::source::Range::new(0, 1));
        assert_eq!(after.view().source_range(), t::source::Range::new(1, 2));
        atomic
    };
    let atomic_view = atomic.view();
    assert_eq!(atomic_view.original_box().id(), id);
    assert_eq!(atomic_view.original_box().size(), size);
    Parent { prepared, source }
}

fn qualify_parent_text(prepared: &t::prepared::PreparedText, atomic: t::prepared::PreparedAtomic) {
    let start = prepared.boundary_views().next().unwrap().handle();
    let end = prepared
        .opportunity_views()
        .find(|row| row.kind() == t::prepared::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(t::selection::LineSelectionRequest {
            inline_boxes: None,
            segment: prepared.source_segments().next().unwrap(),
            start,
            end: end.view().boundary(),
            termination: t::selection::PreparedTermination::Ordinary(end),
            participants: prepared.participants().collect(),
            alternative: None,
            incoming: None,
            controls: vec![],
            boundary_context: vec![],
            line_box_struts: vec![],
            line_strut: Some(STRUT),
            atomics: vec![t::allocation::AtomicAllocation {
                occurrence: atomic.clone(),
                allocation: t::allocation::AtomicLineAllocation::InFlow {
                    fit: surgeist_text::allocation::AtomicFitAllocation::NativeOccupied,
                    flow: t::allocation::AtomicFlowAllocation {
                        normal_border_inline_size: 14.0,
                        advance: 14.0,
                        before_baseline: 4.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 14.0,
                    },
                    box_geometry: t::allocation::AtomicBoxAllocation {
                        border: t::geometry::Rect::new(0.0, -4.0, 14.0, 6.0),
                        content: t::geometry::Rect::new(0.0, -4.0, 14.0, 6.0),
                        baseline: 0.0,
                    },
                    alignment: t::allocation::LineBoxAlignment::Baseline,
                },
            }],
            context: t::selection::LineContext {
                alignment_insets: t::allocation::LineAlignmentInsets::ZERO,
                formatted_line: t::selection::FormattedLineContext::default(),
                base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::selection::LineBoxDirection::LeftToRight,
                orientation: t::selection::LineOrientation::Vertical,
                start: t::selection::LineStart::Paragraph,
                end: t::selection::LineEnd::Paragraph,
                available_inline: Some(22.0),
                indent: 0.0,
                alignment: t::options::Alignment::Start,
                tabs: t::tabs::TabContext::NoTabs,
                spacing: t::spacing::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::scope_alignment::LineAllocations {
            spacing: t::spacing::SpacingAllocations::None,
            scopes: surgeist_text::scope_alignment::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (22.0, 8.0, 10.0)
    );
    let slot = line.atomic_slots().next().unwrap();
    assert_eq!(slot.occurrence, atomic);
    assert_eq!(slot.origin, t::geometry::Point::new(4.0, 8.0));
    let physical = t::geometry::Rect::new(98.0, 4.0, 6.0, 14.0);
    let layout = t::layout::Layout::from_finalized_lines(t::placement::PlacementRequest {
        prepared: prepared.clone(),
        coverage: t::placement::PlacementCoverage::WholeParagraph,
        entries: vec![t::placement::PlacementEntry::Formatted(
            t::placement::LinePlacement {
                line,
                anchor: t::placement::LineAnchor {
                    inline_origin: 0.0,
                    baseline: 100.0,
                },
                orientation: t::selection::LineOrientation::Vertical,
                atomics: vec![t::placement::AtomicPlacement {
                    occurrence: atomic,
                    border: physical,
                    content: physical,
                    baseline: 100.0,
                }],
            },
        )],
    })
    .expect("this new captured parent must also complete actual standalone Text placement");
    assert_eq!(layout.inline_boxes()[0].rect(), physical);
}

fn native_atomic<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    // This actual owning operation precedes preparation/binding of the parent.
    // It is evidence for the child's named fact, not a fabricated provider row.
    qualify_child::<S>(&child);
    let parent = parent(&child.bytes);
    for (mode, c) in [
        (l::geometry::WritingMode::VerticalRl, 32.0),
        (l::geometry::WritingMode::VerticalLr, 2.0),
    ] {
        let mut input = root(mode, l::geometry::Direction::Ltr, central());
        input.size = l::geometry::Size::new(px(40.0), px(22.0));
        let mut host = Host::<S>::new(input);
        let origins = parent
            .prepared
            .origin_views()
            .map(|row| row.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            host.push(
                Some(0),
                l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                    origins[0].clone()
                ))
            ),
            1
        );
        assert_eq!(
            host.push(Some(0), l::node::LayoutInputOf::box_input(child_input())),
            2
        );
        assert_eq!(
            host.push(
                Some(0),
                l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                    origins[1].clone()
                ))
            ),
            3
        );
        let inner = child_binding(&child, 2, 4);
        assert_eq!(
            host.push(
                Some(2),
                l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                    inner.origins[0].origin.clone()
                ))
            ),
            4
        );
        let atomic = parent.prepared.atomic_views().next().unwrap().handle();
        let outer = l::text_layout::TextParagraphBinding {
            inline_box_fragments:
                surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: parent.prepared.clone(),
            container: 0,
            nodes: vec![1, 2, 3],
            origins: origins
                .into_iter()
                .zip([1, 3])
                .map(|(origin, node)| l::text_layout::TextOriginBinding { origin, node })
                .collect(),
            atomics: vec![l::text_layout::TextAtomicBinding {
                occurrence: atomic.clone(),
                node: 2,
            }],
            controls: vec![],
            line_strut: Some(STRUT),
        };
        let batch = solve(&host, vec![outer, inner], 40.0, 22.0);
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            let root = node(entries, 0);
            assert_eq!(root.size, l::geometry::Size::new(n(40.0), n(22.0)));
            assert_central(root.named_baselines, c);
            let child = node(entries, 2);
            assert_eq!(child.size, l::geometry::Size::new(n(6.0), n(14.0)));
            assert_eq!(child.location, l::geometry::Point::new(n(c - 2.0), n(4.0)));
            assert_central(child.named_baselines, 2.0);
        }
        let text = placed(&batch, 0);
        assert_eq!(text.source(), &parent.source);
        assert_eq!(
            text.analyzed_source().identity(),
            parent.prepared.analyzed_source().identity()
        );
        assert_eq!(text.lines().len(), 1);
        assert_eq!(text.lines()[0].baseline(), c);
        assert_eq!(
            text.lines()[0].bounds(),
            t::geometry::Rect::new(c - 2.0, 0.0, 10.0, 22.0)
        );
        let boxes = text.inline_boxes();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].id(), t::source::Id::from_u64(2));
        assert_eq!(
            boxes[0].rect(),
            t::geometry::Rect::new(c - 2.0, 4.0, 6.0, 14.0)
        );
        assert_eq!(
            text.hit(t::geometry::Point::new(c, 10.0)),
            t::layout::Hit::InlineBox(t::source::Id::from_u64(2))
        );
        let mut glyphs = Vec::new();
        for run in text.run_views() {
            assert_eq!(run.glyph_basis(), t::geometry::GlyphBasis::Unrotated);
            assert_eq!(run.face().bytes(), child.bytes);
            glyphs.extend(run.glyphs().map(|glyph| {
                assert_eq!(
                    (glyph.id(), glyph.x(), glyph.y(), glyph.advance()),
                    (274, -2.0, 3.2, 4.0)
                );
                (run.origin().x, run.origin().y + glyph.pen())
            }));
        }
        assert_eq!(glyphs, [(c, 0.0), (c, 18.0)]);
        let child_text = placed(&batch, 2);
        assert_eq!(child_text.source(), &child.source);
        assert_eq!(child_text.lines()[0].baseline(), c);
        for run in child_text.run_views() {
            assert_eq!(run.glyph_basis(), t::geometry::GlyphBasis::Unrotated);
        }
        let mut source_ranges = Vec::new();
        for row in batch.unrounded_inline_fragments() {
            let owner = match row.owner() {
                l::text_layout::InlineFragmentOwner::Node(node) if [1, 3].contains(node) => *node,
                _ => continue,
            };
            let fragment = row.fragment();
            let receipt = fragment.finalized_fragment();
            let line = receipt.finalized_line();
            assert_eq!(line.prepared_text(), parent.prepared);
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (22.0, 8.0, 10.0)
            );
            assert_eq!(line.selected_line().request().line_strut, Some(STRUT));
            assert_eq!(line.atomic_slots().next().unwrap().occurrence, atomic);
            let range = if owner == 1 {
                t::source::Range::new(0, 1)
            } else {
                t::source::Range::new(1, 2)
            };
            assert_eq!(receipt.view().aggregate_range(), Some(range));
            assert_eq!(receipt.view().local_range(), Some(range));
            source_ranges.push(range);
            assert_eq!(fragment.baseline(), l::geometry::Point::new(n(c), n(0.0)));
            let rect = fragment.rect().unwrap();
            assert_eq!(
                rect.origin(),
                l::geometry::Point::new(n(c - 2.0), n(if owner == 1 { 0.0 } else { 18.0 }))
            );
            assert_eq!(rect.size(), l::geometry::Size::new(n(4.0), n(4.0)));
        }
        source_ranges.sort_by_key(|range| range.start);
        assert_eq!(
            source_ranges,
            [t::source::Range::new(0, 1), t::source::Range::new(1, 2)]
        );
    }
}

#[test]
fn native_atomic_matches_real_child_central_without_rotating_physical_box_f32() {
    native_atomic::<f32>();
}

#[test]
fn native_atomic_matches_real_child_central_without_rotating_physical_box_f64() {
    native_atomic::<f64>();
}
