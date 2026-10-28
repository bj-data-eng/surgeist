//! Actual clockwise parent text and independently horizontal atomic child text.
//! Render borrows each exact Layout result; it does not rotate child contents.
use super::*;

const AHEM: &[u8] = include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const ATOMIC_NODE: usize = 2;

struct AtomicTree {
    inputs: Vec<layout::LayoutInput>,
    children: Vec<Vec<usize>>,
    non_box: layout::NodeInput,
}

impl layout::Traverse for AtomicTree {
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

impl layout::LayoutTree for AtomicTree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }

    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn style() -> text::Style {
    text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        ..text::Style::default()
    }
}

fn prepare(
    system: &mut text::System,
    source: &text::Source,
    atomic: bool,
    orientation: text::LineOrientation,
) -> text::PreparedText {
    use text::SourceProcessingEvent as E;
    let authored = style();
    let ranges = if atomic {
        vec![text::Range::new(0, 1), text::Range::new(1, 2)]
    } else {
        vec![text::Range::new(0, 1)]
    };
    let capture = text::process_contextual_source(text::ContextualSourceInput {
        source: source.clone(),
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins: ranges
            .into_iter()
            .map(|range| text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: text::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: if atomic {
            vec![text::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: text::ParagraphConnectionKind::Continuous,
            }]
        } else {
            vec![]
        },
        controls: vec![],
        boundary_policies: if atomic {
            [
                text::ParagraphBoundaryDefinition::AtomicBefore(0),
                text::ParagraphBoundaryDefinition::AtomicAfter(0),
                text::ParagraphBoundaryDefinition::Connection(0),
            ]
            .into_iter()
            .map(|definition| text::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: authored.wrap,
                following_wrap: authored.wrap,
                ancestor_wrap: authored.wrap,
            })
            .collect()
        } else {
            vec![]
        },
        replacements: vec![],
        computed_line_boundaries: vec![],
        scopes: vec![text::SourceProcessingScope::Block {
            topology: text::SourceProcessingTopology::Complete,
            used_direction: text::ParagraphBaseDirection::LeftToRight,
            trim: text::WhiteSpaceTrim::default(),
            base_style: authored.clone(),
        }],
        paragraphs: vec![text::SourceProcessingParagraph {
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
        }],
        events: if atomic {
            vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::Atomic {
                    atomic: 0,
                    affinity: text::Affinity::After,
                },
                E::Connection(0),
                E::Origin {
                    origin: 1,
                    affinity: text::Affinity::After,
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
                    affinity: text::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ]
        },
    })
    .unwrap();
    let capture = std::sync::Arc::new(capture);
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: authored.wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert_eq!(prepared.captured_paragraph(), capture.captured_paragraph(0));
    prepared
}

fn px(value: f32) -> layout::PreferredSize {
    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(value).unwrap())
}

fn limits() -> render::TextLayoutLimits {
    render::TextLayoutLimits::new(
        render::GlyphRunLimits::new(16, 1024, 8 << 20, 8 << 20),
        8,
        8,
        8,
    )
}

fn placed(batch: &layout::CompletedLayoutBatch<usize>, container: usize) -> &text::Layout {
    batch
        .placed_text()
        .iter()
        .find(|entry| entry.container() == container)
        .unwrap()
        .layout()
}

fn assert_composition(kind: text::InlineBoxKind) {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    // Ahem chi is deliberately asymmetric: UPEM1000, advance1000,
    // outline x0..1000/y400..600. At size10 its ink is x0..10/y-6..-4.
    // These licensed fixture facts are also qualified by Render's owning
    // text_layout_adapter/sideways.rs tests; no synthetic font is introduced.
    let mut source = text::Source::identified("\u{3a7}\u{3a7}", text::Id::from_u64(1556001), 7);
    source.inline_box(text::InlineBox::new(
        text::Id::from_u64(2),
        kind,
        1,
        text::Size::new(6.0, 14.0),
    ));
    let child_source = text::Source::identified("\u{3a7}", text::Id::from_u64(1556002), 11);
    let parent = prepare(&mut system, &source, true, text::LineOrientation::Sideways);
    let child = prepare(
        &mut system,
        &child_source,
        false,
        text::LineOrientation::Horizontal,
    );
    let origins = parent
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let atomic = parent.atomic_views().next().unwrap().handle();
    let child_origin = child.origin_views().next().unwrap().handle();
    assert_eq!(atomic.view().original_box().kind(), kind);
    assert_eq!(atomic.view().original_box().id(), text::Id::from_u64(2));
    let tree = AtomicTree {
        inputs: vec![
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                writing_mode: layout::WritingMode::SidewaysRl,
                inline_baseline_context: layout::InlineBaselineContext::Physical,
                size: layout::Size::new(px(40.0), px(40.0)),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origins[0].clone())),
            layout::LayoutInput::box_input(layout::NodeInput {
                display: if kind == text::InlineBoxKind::InFlow {
                    layout::Display::InlineBlock
                } else {
                    layout::Display::Block
                },
                position: if kind == text::InlineBoxKind::OutOfFlow {
                    layout::Position::Absolute
                } else {
                    layout::Position::Relative
                },
                writing_mode: layout::WritingMode::HorizontalTb,
                inline_baseline_context: layout::InlineBaselineContext::Physical,
                size: layout::Size::new(px(6.0), px(14.0)),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origins[1].clone())),
            layout::LayoutInput::inline_text(layout::TextParticipation::new(child_origin.clone())),
        ],
        children: vec![vec![1, 2, 3], vec![], vec![4], vec![], vec![]],
        non_box: layout::NodeInput::non_box(),
    };
    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![
            layout::TextParagraphBinding {
                prepared: parent.clone(),
                container: 0,
                nodes: vec![1, 2, 3],
                origins: origins
                    .iter()
                    .cloned()
                    .zip([1, 3])
                    .map(|(origin, node)| layout::TextOriginBinding { origin, node })
                    .collect(),
                atomics: vec![layout::TextAtomicBinding {
                    occurrence: atomic,
                    node: ATOMIC_NODE,
                }],
                controls: vec![],
                line_strut: None,
            },
            layout::TextParagraphBinding {
                prepared: child.clone(),
                container: ATOMIC_NODE,
                nodes: vec![4],
                origins: vec![layout::TextOriginBinding {
                    origin: child_origin.clone(),
                    node: 4,
                }],
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            },
        ],
        layout::WrappingInput::default(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &flow,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(40.0),
            layout::Available::Definite(40.0),
        ))
        .unwrap(),
    )
    .unwrap();
    let (child_x, following_y, advance) = match kind {
        text::InlineBoxKind::InFlow => (32.0, 24.0, 34.0),
        text::InlineBoxKind::OutOfFlow => (34.0, 10.0, 20.0),
    };
    let child_rect = text::Rect::new(child_x, 10.0, 6.0, 14.0);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = entries
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(output.location, layout::Point::new(child_x as f32, 10.0));
        assert_eq!(output.size, layout::Size::new(6.0, 14.0));
        // Its real horizontal10px glyph overflows a6px physical child width.
        assert_eq!(output.content_size.width, 10.0);
    }
    let parent_layout = placed(&batch, 0);
    let child_layout = placed(&batch, 2);
    assert_eq!(batch.placed_text().len(), 2);
    assert_eq!(parent_layout.orientation(), text::LineOrientation::Sideways);
    assert_eq!(
        child_layout.orientation(),
        text::LineOrientation::Horizontal
    );
    assert_eq!(parent_layout.source(), &source);
    assert_eq!(child_layout.source(), &child_source);
    assert_eq!(parent_layout.lines().len(), 1);
    assert_eq!(child_layout.lines().len(), 1);
    assert_eq!(parent_layout.lines()[0].baseline(), 32.0);
    assert_eq!(child_layout.lines()[0].baseline(), 18.0);
    let positioned = parent_layout.inline_boxes();
    assert_eq!(positioned.len(), 1);
    assert_eq!(positioned[0].kind(), kind);
    assert_eq!(positioned[0].rect(), child_rect);
    assert_eq!(
        parent_layout.hit(text::Point::new(child_x + 1.0, 15.0)),
        text::Hit::InlineBox(text::Id::from_u64(2))
    );
    let line = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| matches!(entry.owner(), layout::InlineFragmentOwner::Node(1)))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(line.feedback().advance, advance);
    let parent_glyphs = parent_layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(
                run.face().catalogue_identity(),
                parent.font_catalogue_identity()
            );
            assert_eq!(run.glyph_basis(), text::GlyphBasis::Clockwise);
            assert_eq!(run.origin().x, 32.0);
            run.glyphs()
                .map(|glyph| {
                    assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (0.0, 0.0, 10.0));
                    (glyph.id(), run.origin().y + glyph.pen())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(parent_glyphs, [(274, 0.0), (274, following_y)]);
    let child_runs = child_layout.run_views().collect::<Vec<_>>();
    assert_eq!(child_runs.len(), 1);
    assert_eq!(child_runs[0].face().bytes(), AHEM);
    assert_eq!(
        child_runs[0].face().catalogue_identity(),
        child.font_catalogue_identity()
    );
    assert_eq!(child_runs[0].glyph_basis(), text::GlyphBasis::Unrotated);
    assert_eq!(child_runs[0].origin(), text::Point::new(child_x, 18.0));
    assert_eq!(
        child_runs[0]
            .glyphs()
            .map(|glyph| (glyph.id(), glyph.pen(), glyph.advance()))
            .collect::<Vec<_>>(),
        [(274, 0.0, 10.0)]
    );
    for (layout, origin, range, expected) in [
        (
            parent_layout,
            &origins[0],
            text::Range::new(0, 1),
            text::Rect::new(30.0, 0.0, 10.0, 10.0),
        ),
        (
            parent_layout,
            &origins[1],
            text::Range::new(1, 2),
            text::Rect::new(30.0, following_y, 10.0, 10.0),
        ),
    ] {
        let fragments = layout.source_range_fragments(origin, range).unwrap();
        assert_eq!(
            fragments
                .units()
                .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [expected]
        );
    }
    assert_eq!(
        parent_layout
            .cursor(text::Cursor::new(0, text::Affinity::After))
            .unwrap()
            .rect(),
        text::Rect::new(30.0, 0.0, 10.0, 1.0)
    );
    assert_eq!(
        child_layout
            .cursor(text::Cursor::new(0, text::Affinity::After))
            .unwrap()
            .rect(),
        text::Rect::new(child_x, 10.0, 1.0, 10.0)
    );
    let before_render = system.stats();
    for outer in [
        render::Transform::identity(),
        render::Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap(),
    ] {
        for actual in [parent_layout, child_layout] {
            let paint = render::prepare_text_layout(actual, outer, limits()).unwrap();
            assert!(std::ptr::eq(paint.layout(), actual));
            assert!(!paint.scene().is_empty());
            assert_eq!(paint.layout().source(), actual.source());
            assert_eq!(paint.layout().lines(), actual.lines());
        }
    }
    assert_eq!(system.stats(), before_render);
    assert_eq!(system.stats().layout_misses(), 0);
}

#[test]
fn sideways_in_flow_child_keeps_horizontal_content_through_layout_and_render() {
    assert_composition(text::InlineBoxKind::InFlow);
}

#[test]
fn sideways_out_of_flow_child_keeps_horizontal_content_through_layout_and_render() {
    assert_composition(text::InlineBoxKind::OutOfFlow);
}

#[test]
fn registered_chi_ink_is_asymmetric_before_clockwise_or_horizontal_placement() {
    let font = render::FontData::try_from_bytes(AHEM.to_vec(), 0).unwrap();
    let instance =
        render::TextInstance::try_new(10.0, vec![], render::GlyphSynthesis::none()).unwrap();
    let glyphs = [render::TextGlyph::try_new(274, 0.0, 0.0, 10.0).unwrap()];
    let admitted = font
        .prepare_glyph_run(
            &instance,
            &glyphs,
            render::GlyphRunLimits::new(1, 64, 1 << 20, 1 << 20),
        )
        .unwrap();
    assert_eq!(admitted.instance(), &instance);
    assert_eq!(admitted.glyphs(), glyphs);
    assert_eq!(
        admitted.bounds().ink_rect(),
        Some(render::Rect::try_new(0.0, -6.0, 10.0, 2.0).unwrap())
    );
    // Scene command transforms are private to Render. Its owning sideways
    // tests execute this stripe's CW transform and noncommuting outer scale;
    // the Root tests above qualify the genuine composed input to that adapter.
}
