//! A real native Text child supplies Central2 inside its physical6x14 border.
//! Parent flow progression and Text's right-to-left cell map stay distinct.
use super::*;
mod supplemental;

fn central() -> l::InlineBaselineContext {
    l::InlineBaselineContext::Named(l::BaselineName::Central)
}

fn available<S: l::LayoutScalar>(width: f64, height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(n(width)),
        l::AvailableOf::definite(n(height)),
    )
}

fn solve<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::TextParagraphBinding<u32>>,
    width: f64,
    height: f64,
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(host, 0, bindings, l::WrappingInput::default()).expect(
        "actual processed captures and issued occurrence roles must bind to their real nodes",
    );
    l::compute_layout_with_flow(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(available(width, height)).unwrap(),
    )
    .expect("native atomic adapter must retain physical size and match the child's actual Central")
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

fn assert_central<S: l::LayoutScalar>(facts: l::NamedBaselinesOf<S>, expected: f64) {
    let pair = facts.get(l::BaselineName::Central);
    let coordinate =
        l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Horizontal, n(expected)).unwrap();
    assert_eq!(pair.first(), l::BaselineFactOf::Available(coordinate));
    assert_eq!(pair.last(), l::BaselineFactOf::Available(coordinate));
}

fn child_input<S: l::LayoutScalar>() -> l::NodeInputOf<S> {
    let mut input = root(l::WritingMode::VerticalLr, l::Direction::Ltr, central());
    input.display = l::Display::InlineBlock;
    input.size = l::Size::new(px(6.0), px(14.0));
    input
}

fn child_binding(fixture: &Fixture, container: u32, text: u32) -> l::TextParagraphBinding<u32> {
    let origin = fixture.prepared.origin_views().next().unwrap().handle();
    l::TextParagraphBinding {
        prepared: fixture.prepared.clone(),
        container,
        nodes: vec![text],
        origins: vec![l::TextOriginBinding { origin, node: text }],
        atomics: vec![],
        controls: vec![],
        // Its real font2/2 establishes Central2. The parent's separately
        // allocated8/2 strut must never be copied into this child binding.
        line_strut: None,
    }
}

fn qualify_child<S: l::LayoutScalar>(fixture: &Fixture) {
    let mut host = Host::<S>::new(child_input());
    let binding = child_binding(fixture, 0, 1);
    assert_eq!(
        host.push(
            Some(0),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(
                binding.origins[0].origin.clone(),
            )),
        ),
        1
    );
    let batch = solve(&host, vec![binding], 6.0, 14.0);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = node(entries, 0);
        assert_eq!(output.size, l::Size::new(n(6.0), n(14.0)));
        assert_central(output.named_baselines, 2.0);
    }
    let text = placed(&batch, 0);
    assert_eq!(text.source(), &fixture.source);
    assert_eq!(text.orientation(), t::LineOrientation::Vertical);
    assert_eq!(text.lines().len(), 1);
    assert_eq!(text.lines()[0].baseline(), 2.0);
    assert_eq!(text.lines()[0].bounds(), t::Rect::new(0.0, 0.0, 4.0, 14.0));
    let mut saw_source = false;
    for row in batch.unrounded_inline_fragments() {
        if matches!(row.owner(), l::InlineFragmentOwner::Node(1)) {
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
            assert_eq!(fragment.baseline(), l::Point::new(n(2.0), n(0.0)));
        }
    }
    assert!(
        saw_source,
        "actual child source must retain its finalized receipt"
    );
}

struct Parent {
    prepared: t::PreparedText,
    source: t::Source,
}

fn parent(bytes: &[u8]) -> Parent {
    let parent = captured_parent(bytes, t::Size::new(6.0, 14.0), false, t::Id::from_u64(2));
    let atomic = parent.prepared.atomic_views().next().unwrap().handle();
    qualify_parent_text(&parent.prepared, atomic);
    parent
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
        t::Id::from_u64(1_552_100 + u64::from(atomic_only)),
        1,
    );
    // This is the authored fixture identity of the actual bound node. All
    // capture/preparation capabilities are issued by Text, never synthesized.
    source.inline_box(t::InlineBox::new(
        id,
        t::InlineBoxKind::InFlow,
        usize::from(!atomic_only),
        size,
    ));
    use t::SourceProcessingEvent as E;
    let capture = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
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
    let prepared = system
        .prepare(
            &analysis,
            t::PreparationOptions::default().with_orientation(t::LineOrientation::Vertical),
        )
        .expect(
            "the real captured InFlow parent must satisfy the independently GREEN Text producer",
        );
    assert_eq!(prepared.captured_paragraph(), capture.captured_paragraph(0));
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

fn qualify_parent_text(prepared: &t::PreparedText, atomic: t::PreparedAtomic) {
    let start = prepared.boundary_views().next().unwrap().handle();
    let end = prepared
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = prepared
        .select(t::LineSelectionRequest {
            start,
            end: end.view().boundary(),
            termination: t::PreparedTermination::Ordinary(end),
            participants: prepared.participants().collect(),
            alternative: None,
            incoming: None,
            controls: vec![],
            boundary_context: vec![],
            line_box_struts: vec![],
            line_strut: Some(STRUT),
            atomics: vec![t::AtomicAllocation {
                occurrence: atomic.clone(),
                allocation: t::AtomicLineAllocation::InFlow {
                    flow: t::AtomicFlowAllocation {
                        advance: 14.0,
                        before_baseline: 4.0,
                        after_baseline: 2.0,
                        occupied_start: 0.0,
                        occupied_end: 14.0,
                    },
                    box_geometry: t::AtomicBoxAllocation {
                        border: t::Rect::new(0.0, -4.0, 14.0, 6.0),
                        content: t::Rect::new(0.0, -4.0, 14.0, 6.0),
                        baseline: 0.0,
                    },
                    alignment: t::LineBoxAlignment::Baseline,
                },
            }],
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
        .finalize(t::SpacingAllocations::None)
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
    assert_eq!(slot.origin, t::Point::new(4.0, 8.0));
    let physical = t::Rect::new(98.0, 4.0, 6.0, 14.0);
    let layout = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 0.0,
                baseline: 100.0,
            },
            orientation: t::LineOrientation::Vertical,
            atomics: vec![t::AtomicPlacement {
                occurrence: atomic,
                border: physical,
                content: physical,
                baseline: 100.0,
            }],
        })],
    })
    .expect("this new captured parent must also complete actual standalone Text placement");
    assert_eq!(layout.inline_boxes()[0].rect(), physical);
}

fn native_atomic<S: l::LayoutScalar>() {
    let child = fixture();
    // This actual owning operation precedes preparation/binding of the parent.
    // It is evidence for the child's named fact, not a fabricated provider row.
    qualify_child::<S>(&child);
    let parent = parent(&child.bytes);
    for (mode, c) in [
        (l::WritingMode::VerticalRl, 32.0),
        (l::WritingMode::VerticalLr, 2.0),
    ] {
        let mut input = root(mode, l::Direction::Ltr, central());
        input.size = l::Size::new(px(40.0), px(22.0));
        let mut host = Host::<S>::new(input);
        let origins = parent
            .prepared
            .origin_views()
            .map(|row| row.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            host.push(
                Some(0),
                l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[0].clone()))
            ),
            1
        );
        assert_eq!(
            host.push(Some(0), l::LayoutInputOf::box_input(child_input())),
            2
        );
        assert_eq!(
            host.push(
                Some(0),
                l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[1].clone()))
            ),
            3
        );
        let inner = child_binding(&child, 2, 4);
        assert_eq!(
            host.push(
                Some(2),
                l::LayoutInputOf::inline_text(l::TextParticipation::new(
                    inner.origins[0].origin.clone()
                ))
            ),
            4
        );
        let atomic = parent.prepared.atomic_views().next().unwrap().handle();
        let outer = l::TextParagraphBinding {
            prepared: parent.prepared.clone(),
            container: 0,
            nodes: vec![1, 2, 3],
            origins: origins
                .into_iter()
                .zip([1, 3])
                .map(|(origin, node)| l::TextOriginBinding { origin, node })
                .collect(),
            atomics: vec![l::TextAtomicBinding {
                occurrence: atomic.clone(),
                node: 2,
            }],
            controls: vec![],
            line_strut: Some(STRUT),
        };
        let batch = solve(&host, vec![outer, inner], 40.0, 22.0);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let root = node(entries, 0);
            assert_eq!(root.size, l::Size::new(n(40.0), n(22.0)));
            assert_central(root.named_baselines, c);
            let child = node(entries, 2);
            assert_eq!(child.size, l::Size::new(n(6.0), n(14.0)));
            assert_eq!(child.location, l::Point::new(n(c - 2.0), n(4.0)));
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
            t::Rect::new(c - 2.0, 0.0, 10.0, 22.0)
        );
        let boxes = text.inline_boxes();
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].id(), t::Id::from_u64(2));
        assert_eq!(boxes[0].rect(), t::Rect::new(c - 2.0, 4.0, 6.0, 14.0));
        assert_eq!(
            text.hit(t::Point::new(c, 10.0)),
            t::Hit::InlineBox(t::Id::from_u64(2))
        );
        let mut glyphs = Vec::new();
        for run in text.run_views() {
            assert_eq!(run.glyph_basis(), t::GlyphBasis::Unrotated);
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
            assert_eq!(run.glyph_basis(), t::GlyphBasis::Unrotated);
        }
        let mut source_ranges = Vec::new();
        for row in batch.unrounded_inline_fragments() {
            let owner = match row.owner() {
                l::InlineFragmentOwner::Node(node) if [1, 3].contains(node) => *node,
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
                t::Range::new(0, 1)
            } else {
                t::Range::new(1, 2)
            };
            assert_eq!(receipt.view().aggregate_range(), Some(range));
            assert_eq!(receipt.view().local_range(), Some(range));
            source_ranges.push(range);
            assert_eq!(fragment.baseline(), l::Point::new(n(c), n(0.0)));
            let rect = fragment.rect().unwrap();
            assert_eq!(
                rect.origin(),
                l::Point::new(n(c - 2.0), n(if owner == 1 { 0.0 } else { 18.0 }))
            );
            assert_eq!(rect.size(), l::Size::new(n(4.0), n(4.0)));
        }
        source_ranges.sort_by_key(|range| range.start);
        assert_eq!(source_ranges, [t::Range::new(0, 1), t::Range::new(1, 2)]);
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
