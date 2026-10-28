//! Physical clockwise Text keeps the actual child's physical content unrotated.
use super::*;
mod supplemental;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Captured {
    prepared: t::PreparedText,
    source: t::Source,
}

fn table(bytes: &[u8], tag: [u8; 4]) -> &[u8] {
    let u16_at = |at| u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap());
    let u32_at = |at| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    let record = (0..usize::from(u16_at(4)))
        .map(|i| 12 + 16 * i)
        .find(|at| bytes[*at..*at + 4] == tag)
        .unwrap();
    let start = usize::try_from(u32_at(record + 8)).unwrap();
    let len = usize::try_from(u32_at(record + 12)).unwrap();
    &bytes[start..start + len]
}
fn qualify_font(bytes: &[u8]) {
    let head = table(bytes, *b"head");
    assert_eq!(u16::from_be_bytes(head[18..20].try_into().unwrap()), 1000);
    let hhea = table(bytes, *b"hhea");
    assert_eq!(
        [4, 6, 8].map(|at| i16::from_be_bytes(hhea[at..at + 2].try_into().unwrap())),
        [800, -200, 0]
    );
    for id in [35, 36] {
        let hmtx = table(bytes, *b"hmtx");
        assert_eq!(
            u16::from_be_bytes(hmtx[id * 4..id * 4 + 2].try_into().unwrap()),
            1000
        );
    }
    // These witnesses use actual horizontal design metrics. They do not grant
    // a named Alphabetic reference or assume that another font's BASE is zero.
}

fn capture(
    bytes: &[u8],
    raw: &str,
    atomic: Option<(t::InlineBoxKind, t::Id)>,
    body: t::ParagraphBaseDirection,
    orientation: t::LineOrientation,
    size: f32,
) -> Captured {
    qualify_font(bytes);
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size,
        line_height: t::LineHeight::Absolute(f64::from(size)),
        white_space: t::WhiteSpace::Preserve,
        word_break: t::WordBreak::BreakAll,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    };
    let mut source = t::Source::new(raw);
    if let Some((kind, id)) = atomic {
        source.inline_box(t::InlineBox::new(
            id,
            kind,
            usize::from(!raw.is_empty()),
            t::Size::new(6.0, 14.0),
        ));
    }
    let ranges = if raw.is_empty() {
        vec![]
    } else if atomic.is_some() {
        vec![t::Range::new(0, 1), t::Range::new(1, 2)]
    } else {
        vec![t::Range::new(0, raw.encode_utf16().count())]
    };
    use t::SourceProcessingEvent as E;
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if !ranges.is_empty() {
        events.push(E::Origin {
            origin: 0,
            affinity: t::Affinity::After,
        });
    }
    if atomic.is_some() {
        events.push(E::Atomic {
            atomic: 0,
            affinity: t::Affinity::After,
        });
    }
    if ranges.len() == 2 {
        events.extend([
            E::Connection(0),
            E::Origin {
                origin: 1,
                affinity: t::Affinity::After,
            },
        ]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let mut definitions = if atomic.is_some() {
        vec![
            t::ParagraphBoundaryDefinition::AtomicBefore(0),
            t::ParagraphBoundaryDefinition::AtomicAfter(0),
        ]
    } else {
        vec![]
    };
    if ranges.len() == 2 {
        definitions.push(t::ParagraphBoundaryDefinition::Connection(0));
    }
    let processed = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: ranges
                .into_iter()
                .map(|range| t::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: range,
                    aggregate: range,
                    kind: t::ParagraphOriginKind::Authored,
                })
                .collect(),
            controls: vec![],
            connections: if atomic.is_some() && !raw.is_empty() {
                vec![t::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: t::ParagraphConnectionKind::Continuous,
                }]
            } else {
                vec![]
            },
            boundary_policies: definitions
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
                used_direction: body,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: body,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let analysis = t::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analysis
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analysis = analysis.with_wrapping_policies(policies).unwrap();
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
            t::PreparationOptions::default().with_orientation(orientation),
        )
        .expect("the actual Text producer must succeed before any Layout geometry assertion");
    assert_eq!(
        prepared.captured_paragraph(),
        processed.captured_paragraph(0)
    );
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(prepared.orientation(), orientation);
    assert_eq!(
        prepared.atomic_views().count(),
        usize::from(atomic.is_some())
    );
    if let Some((kind, id)) = atomic {
        let actual = prepared.atomic_views().next().unwrap();
        assert_eq!(actual.original_box().kind(), kind);
        assert_eq!(actual.original_box().id(), id);
    }
    Captured { prepared, source }
}

fn allocation(parent: &Captured, kind: t::InlineBoxKind) -> t::AtomicAllocation {
    t::AtomicAllocation {
        occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
        allocation: match kind {
            t::InlineBoxKind::OutOfFlow => t::AtomicLineAllocation::OutOfFlow,
            t::InlineBoxKind::InFlow => t::AtomicLineAllocation::InFlow {
                alignment: t::LineBoxAlignment::Baseline,
                // A Horizontal child exports no physical-x baseline. Physical
                // synthesis uses its left/line-under edge: over6, under0.
                flow: t::AtomicFlowAllocation {
                    advance: 14.0,
                    before_baseline: 6.0,
                    after_baseline: 0.0,
                    occupied_start: 0.0,
                    occupied_end: 14.0,
                },
                box_geometry: t::AtomicBoxAllocation {
                    border: t::Rect::new(0.0, -6.0, 14.0, 6.0),
                    content: t::Rect::new(0.0, -6.0, 14.0, 6.0),
                    baseline: 0.0,
                },
            },
        },
    }
}

fn selected(parent: &Captured, kind: t::InlineBoxKind) -> t::FinalizedLine {
    let p = &parent.prepared;
    let end = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    p.select(t::LineSelectionRequest {
        start: p.boundary_views().next().unwrap().handle(),
        end: end.view().boundary(),
        termination: t::PreparedTermination::Ordinary(end),
        participants: p.participants().collect(),
        alternative: None,
        incoming: None,
        atomics: vec![allocation(parent, kind)],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: t::LineContext {
            alignment_insets: t::LineAlignmentInsets::ZERO,
            formatted_line: t::FormattedLineContext::default(),
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            line_box_direction: t::LineBoxDirection::LeftToRight,
            orientation: t::LineOrientation::Sideways,
            start: t::LineStart::Paragraph,
            end: t::LineEnd::Paragraph,
            available_inline: Some(40.0),
            indent: 0.0,
            alignment: t::Alignment::Start,
            tabs: t::TabContext::NoTabs,
            spacing: t::SpacingPolicy::None,
        },
    })
    .unwrap()
    .finalize(t::SpacingAllocations::None)
    .unwrap()
}

fn explicit(
    parent: &Captured,
    line: t::FinalizedLine,
    anchor: t::LineAnchor,
    geometry: t::AtomicPlacement,
) -> t::Layout {
    t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: parent.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor,
            orientation: t::LineOrientation::Sideways,
            atomics: vec![geometry],
        })],
    })
    .unwrap()
}

fn qualify_parent(parent: &Captured, kind: t::InlineBoxKind) {
    let line = selected(parent, kind);
    let advance = if kind == t::InlineBoxKind::InFlow {
        34.0
    } else {
        20.0
    };
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (advance, 8.0, 10.0)
    );
    let slot = line.atomic_slots().next().unwrap();
    assert_eq!(slot.origin, t::Point::new(10.0, 8.0));
    assert_eq!(
        slot.occurrence,
        parent.prepared.atomic_views().next().unwrap().handle()
    );
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
        let instance = run.effective_instance().unwrap();
        let metrics = instance
            .metrics(parent.prepared.font_catalogue_identity())
            .unwrap();
        assert_eq!(
            (metrics.ascent(), metrics.descent(), metrics.leading()),
            (8.0, -2.0, 0.0)
        );
        glyphs.extend(run.glyphs().map(|g| (g.id(), g.x(), g.y(), g.advance())));
    }
    assert_eq!(glyphs, [(35, 0.0, 0.0, 10.0), (36, 0.0, 0.0, 10.0)]);
    let x = if kind == t::InlineBoxKind::InFlow {
        100.0
    } else {
        102.0
    };
    let physical = t::Rect::new(x, 30.0, 6.0, 14.0);
    let text = explicit(
        parent,
        line,
        t::LineAnchor {
            inline_origin: 20.0,
            baseline: 100.0,
        },
        t::AtomicPlacement {
            occurrence: slot.occurrence,
            border: physical,
            content: physical,
            baseline: x,
        },
    );
    assert_eq!(text.inline_boxes()[0].rect(), physical);
    assert_eq!(text.source(), &parent.source);
}

fn available<S: l::LayoutScalar>(width: f64, height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(n(width)),
        l::AvailableOf::definite(n(height)),
    )
}
fn child_input<S: l::LayoutScalar>(kind: t::InlineBoxKind) -> l::NodeInputOf<S> {
    let mut input = root(
        l::WritingMode::HorizontalTb,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.display = l::Display::InlineBlock;
    input.position = if kind == t::InlineBoxKind::OutOfFlow {
        l::Position::Absolute
    } else {
        l::Position::Relative
    };
    input.size = l::Size::new(px(6.0), px(14.0));
    input
}
fn child_binding(child: &Captured, container: u32, text: u32) -> l::TextParagraphBinding<u32> {
    l::TextParagraphBinding {
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
fn run<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: Vec<l::TextParagraphBinding<u32>>,
    changed: &[u32],
    space: l::Size<l::AvailableOf<S>>,
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow = l::LayoutFlowInput::try_new(host, 0, bindings, l::WrappingInput::default()).unwrap();
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(space).unwrap(),
        changed,
    )
}
fn unknown<S: l::LayoutScalar>(facts: l::NamedBaselinesOf<S>) {
    for name in [l::BaselineName::Alphabetic, l::BaselineName::Central] {
        let pair = facts.get(name);
        assert_eq!(
            (pair.first(), pair.last()),
            (l::BaselineFactOf::Unknown, l::BaselineFactOf::Unknown)
        );
    }
}
fn qualify_child<S: l::LayoutScalar>(child: &Captured) {
    let mut host = Host::new(child_input::<S>(t::InlineBoxKind::InFlow));
    host.change(0, |input| input.display = l::Display::Block);
    let binding = child_binding(child, 0, 1);
    host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(binding.origins[0].origin.clone())),
    );
    let batch = run(&host, vec![binding], &[], available(6.0, 14.0)).unwrap();
    let text = placed(&batch, 0);
    assert_eq!(text.orientation(), t::LineOrientation::Horizontal);
    assert_eq!(text.lines()[0].baseline(), 4.0);
    assert_eq!(text.lines()[0].bounds(), t::Rect::new(0.0, 0.0, 6.0, 5.0));
    assert_eq!(text.source(), &child.source);
    assert!(
        text.run_views()
            .all(|run| run.glyph_basis() == t::GlyphBasis::Unrotated)
    );
    unknown(node(batch.unrounded_entries(), 0).named_baselines);
}
fn attach<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
) -> Vec<l::TextParagraphBinding<u32>> {
    let origins = parent
        .prepared
        .origin_views()
        .map(|row| row.handle())
        .collect::<Vec<_>>();
    let first = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[0].clone())),
    );
    let atomic = host.push(
        Some(container),
        l::LayoutInputOf::box_input(child_input(kind)),
    );
    let last = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[1].clone())),
    );
    assert_eq!(
        parent
            .prepared
            .atomic_views()
            .next()
            .unwrap()
            .original_box()
            .id(),
        t::Id::from_u64(u64::from(atomic))
    );
    let text = host.push(
        Some(atomic),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(
            child.prepared.origin_views().next().unwrap().handle(),
        )),
    );
    vec![
        l::TextParagraphBinding {
            prepared: parent.prepared.clone(),
            container,
            nodes: vec![first, atomic, last],
            origins: origins
                .into_iter()
                .zip([first, last])
                .map(|(origin, node)| l::TextOriginBinding { origin, node })
                .collect(),
            atomics: vec![l::TextAtomicBinding {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                node: atomic,
            }],
            controls: vec![],
            line_strut: None,
        },
        child_binding(child, atomic, text),
    ]
}
fn scene<S: l::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
    direction: l::Direction,
    height: f64,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input = root(
        l::WritingMode::SidewaysRl,
        direction,
        l::InlineBaselineContext::Physical,
    );
    input.size = l::Size::new(px(40.0), px(height));
    let mut host = Host::new(input);
    let bindings = attach(&mut host, 0, parent, child, kind);
    (host, bindings)
}
fn receipt<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    source: u32,
) -> t::FinalizedLine {
    batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| matches!(row.owner(),l::InlineFragmentOwner::Node(node) if *node==source))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line()
        .clone()
}

fn core<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(2))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::Sideways,
            10.0,
        );
        qualify_parent(&parent, kind);
        let (host, bindings) = scene::<S>(&parent, &child, kind, l::Direction::Ltr, 40.0);
        let batch = run(&host, bindings, &[], available(40.0, 40.0)).unwrap();
        let x = if kind == t::InlineBoxKind::InFlow {
            32.0
        } else {
            34.0
        };
        let advance = if kind == t::InlineBoxKind::InFlow {
            34.0
        } else {
            20.0
        };
        let physical = t::Rect::new(x, 10.0, 6.0, 14.0);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let output = node(entries, 2);
            assert_eq!(output.size, l::Size::new(n(6.0), n(14.0)));
            assert_eq!(output.location, l::Point::new(n(x), n(10.0)));
            unknown(output.named_baselines);
            unknown(node(entries, 0).named_baselines);
        }
        let text = placed(&batch, 0);
        assert_eq!(text.orientation(), t::LineOrientation::Sideways);
        assert_eq!(text.lines().len(), 1);
        assert_eq!(text.lines()[0].baseline(), 32.0);
        assert_eq!(text.inline_boxes()[0].rect(), physical);
        assert_eq!(text.inline_boxes()[0].kind(), kind);
        assert_eq!(
            text.hit(t::Point::new(x + 1.0, 12.0)),
            t::Hit::InlineBox(t::Id::from_u64(2))
        );
        let line = receipt(&batch, 1);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (advance, 8.0, 10.0)
        );
        let expected = explicit(
            &parent,
            line,
            t::LineAnchor {
                inline_origin: 0.0,
                baseline: 32.0,
            },
            t::AtomicPlacement {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                border: physical,
                content: physical,
                baseline: x,
            },
        );
        assert_eq!(text.key(), expected.key());
        let second = if kind == t::InlineBoxKind::InFlow {
            24.0
        } else {
            10.0
        };
        let pens = text
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
                run.glyphs()
                    .map(|glyph| (glyph.id(), run.origin().x, run.origin().y + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(pens, [(35, 32.0, 0.0), (36, 32.0, second)]);
        let child_text = placed(&batch, 2);
        assert_eq!(child_text.orientation(), t::LineOrientation::Horizontal);
        assert_eq!(child_text.lines()[0].baseline(), 14.0);
        assert!(
            child_text
                .run_views()
                .all(|run| run.glyph_basis() == t::GlyphBasis::Unrotated)
        );
    }
}

#[test]
fn physical_clockwise_atoms_keep_real_child_geometry_and_content_basis_f32() {
    core::<f32>();
}
#[test]
fn physical_clockwise_atoms_keep_real_child_geometry_and_content_basis_f64() {
    core::<f64>();
}
