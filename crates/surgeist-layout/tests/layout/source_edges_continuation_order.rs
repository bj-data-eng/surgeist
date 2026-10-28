//! One original inline S continues across two parent paragraphs and a real block.
use std::{convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Tree<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    non_box: l::NodeInputOf<S>,
    children: [Vec<u32>; 7],
}
impl<S: l::LayoutScalar> l::Traverse for Tree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, n: u32) -> Self::Children<'_> {
        self.children[n as usize].iter().copied()
    }
    fn child_count(&self, n: u32) -> usize {
        self.children[n as usize].len()
    }
    fn child(&self, n: u32, i: usize) -> u32 {
        self.children[n as usize][i]
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for Tree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, n: u32) -> &l::NodeInputOf<S> {
        self.inputs[n as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, n: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[n as usize].as_view()
    }
}

fn processed() -> Arc<t::ProcessedSource> {
    use t::SourceProcessingEvent as E;
    let base = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        wrap: t::Wrap::None,
        hyphens: t::Hyphens::None,
        ..t::Style::default()
    };
    let mut source = t::Source::identified("AXB", t::Id::from_u64(658100), 31);
    source.span(t::Range::new(0, 3), base.clone());
    let block = || t::SourceProcessingScope::Block {
        topology: t::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: base.clone(),
        used_direction: t::ParagraphBaseDirection::LeftToRight,
    };
    let origin = |origin| E::Origin {
        origin,
        affinity: t::Affinity::After,
    };
    let control = |control| E::Control {
        control,
        affinity: t::Affinity::After,
    };
    Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: (0..3)
                .map(|i| t::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: t::Range::new(i, i + 1),
                    aggregate: t::Range::new(i, i + 1),
                    kind: t::ParagraphOriginKind::Authored,
                })
                .collect(),
            controls: vec![
                t::ParagraphControl {
                    at: 0,
                    kind: t::ParagraphControlKind::InlineBoundaryStart,
                },
                t::ParagraphControl {
                    at: 3,
                    kind: t::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            connections: vec![],
            boundary_policies: vec![],
            computed_line_boundaries: vec![],
            replacements: vec![],
            scopes: vec![
                block(),
                t::SourceProcessingScope::Inline {
                    trim: Default::default(),
                    style: Some(base.clone()),
                    boundaries: Some(t::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
                block(),
            ],
            paragraphs: (0..3)
                .map(|_| t::SourceProcessingParagraph {
                    base_direction: t::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // This is the published owning continuation profile, reduced to ASCII.
            // S remains genuinely open across childBlock, rather than being cloned.
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                origin(0),
                E::ExitParagraph(0),
                E::EnterScope(2),
                E::EnterParagraph(1),
                origin(1),
                E::ExitParagraph(1),
                E::ExitScope(2),
                E::EnterParagraph(2),
                origin(2),
                control(1),
                E::ExitScope(1),
                E::ExitParagraph(2),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    )
}

fn fixture<S: l::LayoutScalar>() -> (
    Tree<S>,
    Vec<l::TextParagraphBinding<u32>>,
    Arc<t::ProcessedSource>,
) {
    let capture = processed();
    let s = capture.captured_scope(1).unwrap();
    assert_eq!(s.parent(), capture.captured_scope(0));
    assert_eq!(capture.captured_scope(2).unwrap().parent(), Some(s.clone()));
    assert_eq!(s.entry_control(), capture.captured_control(0));
    assert_eq!(s.exit_control(), capture.captured_control(1));
    let p0 = capture.captured_paragraph(0).unwrap();
    let p1 = capture.captured_paragraph(1).unwrap();
    let p2 = capture.captured_paragraph(2).unwrap();
    assert_eq!(p0.domain(), capture.captured_scope(0).unwrap());
    assert_eq!(p1.domain(), capture.captured_scope(2).unwrap());
    assert_eq!(p2.domain(), p0.domain());
    assert!(p0.enter_position() < s.enter_position());
    assert!(s.exit_position() < p2.exit_position());
    let parent_paragraphs = capture
        .captured_scope(0)
        .unwrap()
        .flow_events()
        .unwrap()
        .filter_map(|event| match event {
            t::CapturedFlowEvent::ParagraphStart(p) => Some(p),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(parent_paragraphs, [p0, p2]);
    let mut fonts = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(AHEM.to_vec()).unwrap();
    let mut inputs: Vec<_> = (0..7)
        .map(|_| l::LayoutInputOf::box_input(l::NodeInputOf::default()))
        .collect();
    inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::Block,
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(100.0)).unwrap()),
            l::PreferredSizeOf::AUTO,
        ),
        ..l::NodeInputOf::default()
    });
    inputs[3] = l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::Block,
        ..l::NodeInputOf::default()
    });
    for (node, kind) in [
        (1, l::InlineBoundaryKind::Start),
        (5, l::InlineBoundaryKind::End),
    ] {
        inputs[node] = l::LayoutInputOf::inline_boundary(l::InlineBoundaryInputOf::new(
            kind,
            l::InlineMetricsOf::try_new(
                surgeist_layout::InlineBaselineContext::Physical,
                S::from_f64(8.0),
                S::from_f64(10.0),
            )
            .unwrap(),
            l::InlineBoundaryEdgeOf::ZERO,
        ));
    }
    let bindings = capture
        .paragraphs()
        .enumerate()
        .map(|(i, p)| {
            assert!(Arc::ptr_eq(p.processed_source(), &capture));
            let a = t::AnalyzedSource::from_processed_paragraph(
                p,
                t::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let missing = a
                .wrapping_relations()
                .filter(|r| r.ancestor_wrap().is_none())
                .map(|r| t::WrappingPolicy {
                    relation: r.handle(),
                    ancestor_wrap: t::Wrap::None,
                })
                .collect();
            let a = a.with_wrapping_policies(missing).unwrap();
            let prepared = fonts.prepare(&a, t::PreparationOptions::default()).unwrap();
            assert_eq!(
                prepared
                    .opportunity_views()
                    .map(|o| (o.kind(), o.boundary().view().source_before().get()))
                    .collect::<Vec<_>>(),
                [(t::PreparedBreakKind::ParagraphEnd, i + 1)]
            );
            let (container, nodes, origin_node) = match i {
                0 => (0, vec![1, 2], 2),
                1 => (3, vec![6], 6),
                2 => (0, vec![4, 5], 4),
                _ => unreachable!(),
            };
            let origin = prepared.origin_views().next().unwrap();
            assert_eq!(origin.source().id(), Some(t::Id::from_u64(658100)));
            assert_eq!(origin.aggregate_range(), t::Range::new(i, i + 1));
            assert_eq!(origin.local_range(), t::Range::new(i, i + 1));
            let origin = origin.handle();
            inputs[origin_node as usize] =
                l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone()));
            let controls = prepared
                .control_views()
                .map(|c| {
                    let occurrence = c.handle();
                    let actual = occurrence.captured_control().unwrap();
                    let node = if Some(actual.clone()) == capture.captured_control(0) {
                        1
                    } else {
                        assert_eq!(Some(actual), capture.captured_control(1));
                        5
                    };
                    l::TextControlBinding { occurrence, node }
                })
                .collect();
            l::TextParagraphBinding {
                prepared,
                container,
                nodes,
                origins: vec![l::TextOriginBinding {
                    origin,
                    node: origin_node,
                }],
                controls,
                atomics: vec![],
                line_strut: None,
            }
        })
        .collect();
    let children = std::array::from_fn(|i| match i {
        0 => vec![1, 2, 3, 4, 5],
        3 => vec![6],
        _ => vec![],
    });
    (
        Tree {
            inputs,
            children,
            non_box: l::NodeInputOf::non_box(),
        },
        bindings,
        capture,
    )
}

fn wrapping(reverse: bool, captured: Option<t::CapturedScope>) -> l::WrappingInput<u32> {
    let children = if reverse { [4, 3, 2] } else { [2, 3, 4] }
        .into_iter()
        .map(|node| l::GeneratedChildBoxInput {
            box_: l::GeneratedBoxRef::Node(node),
            participation: l::GeneratedBoxParticipation::InFlow,
        })
        .collect();
    l::WrappingInput {
        scopes: vec![l::WrappingScopeInput::NativePair {
            start: 1,
            end: 5,
            wrap_inside: l::WrapInside::Auto,
            edges: l::WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![l::InlineChildOrderInput {
            parent: l::GeneratedBoxRef::Inline(captured.map_or(
                l::InlineBoxRef::NativePair { start: 1, end: 5 },
                l::InlineBoxRef::Scope,
            )),
            children,
            prefix: l::ChildOrderCompleteness::Complete,
            suffix: l::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        }],
    }
}
fn request<S: l::LayoutScalar>() -> l::LayoutRootRequestOf<S> {
    l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(100.0)),
        l::AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}

fn full<S: l::LayoutScalar>(captured_order: bool) {
    let (tree, bindings, capture) = fixture::<S>();
    let scope = captured_order.then(|| capture.captured_scope(1).unwrap());
    let flow=l::LayoutFlowInput::try_new(&tree,0,bindings.clone(),wrapping(false, scope)).expect("all real parent paragraphs plus actual child block must cover S's complete continued extent");
    let batch = l::compute_layout_with_flow(&flow, 0, request()).unwrap();
    let output = |node| {
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
    };
    assert_eq!(
        output(0).size,
        l::Size::new(S::from_f64(100.0), S::from_f64(30.0))
    );
    assert_eq!(
        output(3).location,
        l::Point::new(S::ZERO, S::from_f64(10.0))
    );
    assert_eq!(
        output(3).size,
        l::Size::new(S::from_f64(100.0), S::from_f64(10.0))
    );
    assert_eq!(batch.placed_text().len(), 3);
    for (i, nodes, container, gid, baseline) in [
        (0, vec![1, 2], 0, 35, 8.0),
        (1, vec![6], 3, 58, 18.0),
        (2, vec![4, 5], 0, 36, 28.0),
    ] {
        let entry = batch
            .placed_text()
            .iter()
            .find(|entry| entry.nodes() == nodes.as_slice())
            .unwrap();
        assert_eq!(entry.container(), container);
        let layout = entry.layout();
        assert_eq!(
            layout.analyzed_source().identity(),
            bindings[i].prepared.analyzed_source().identity()
        );
        assert_eq!(layout.source().id(), Some(t::Id::from_u64(658100)));
        assert_eq!(layout.source().revision(), 31);
        assert_eq!(layout.source().text(), &t::Text::from("AXB"));
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [t::Range::new(i, i + 1)]
        );
        assert_eq!(layout.lines()[0].baseline(), baseline);
        assert_eq!(layout.lines()[0].bounds().size.height, 10.0);
        let mut glyphs = vec![];
        for run in layout.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            for glyph in run.glyphs() {
                glyphs.push((
                    glyph.id(),
                    glyph.advance(),
                    run.origin().x + glyph.pen() + f64::from(glyph.x()),
                    run.origin().y,
                ));
            }
        }
        assert_eq!(glyphs, [(gid, 10.0, 0.0, baseline)]);
    }
    // The captured S is still one global box with Start in p0 and End in p2.
    assert_eq!(
        bindings[0].controls[0].occurrence.captured_control(),
        capture.captured_scope(1).unwrap().entry_control()
    );
    assert_eq!(
        bindings[2].controls[0].occurrence.captured_control(),
        capture.captured_scope(1).unwrap().exit_control()
    );
    let mut sourced = vec![];
    for entry in batch.final_inline_fragments() {
        let l::InlineFragmentOwner::Node(node) = entry.owner() else {
            panic!("no generated source was requested")
        };
        if ![2, 4, 6].contains(node) {
            continue;
        }
        let line = entry.fragment().finalized_fragment().finalized_line();
        for run in line.run_views() {
            for glyph in run.glyphs() {
                sourced.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
    }
    sourced.sort_by_key(|(range, _, _)| range.start);
    assert_eq!(
        sourced,
        [
            (t::Range::new(0, 1), 35, 10.0),
            (t::Range::new(1, 2), 58, 10.0),
            (t::Range::new(2, 3), 36, 10.0)
        ]
    );
}

fn partial<S: l::LayoutScalar>() {
    let (tree, mut bindings, capture) = fixture::<S>();
    bindings.remove(2);
    let error = l::LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        wrapping(false, capture.captured_scope(1)),
    )
    .err()
    .expect("real ending text paragraph cannot be omitted");
    // With this actual unmodified public tree, text node4 remains uncovered;
    // the public completeness check necessarily precedes canonical coverage.
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TextBinding(
                l::TextBindingError::MissingParagraph
            ))
        ),
        "{error:?}"
    );
}

fn contradictory<S: l::LayoutScalar>() {
    let (tree, bindings, capture) = fixture::<S>();
    let error = l::LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        wrapping(true, capture.captured_scope(1)),
    )
    .err()
    .expect("complete order B,Block,A contradicts actual source and native A,Block,B");
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
                l::WrappingScopeError::ChildOrder
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn complete_original_inline_order_across_real_paragraphs_and_block_is_supported() {
    full::<f32>(false);
    full::<f64>(false);
}
#[test]
fn complete_captured_inline_order_across_real_paragraphs_and_block_is_supported() {
    full::<f32>(true);
    full::<f64>(true);
}
#[test]
fn omitted_real_ending_paragraph_is_precisely_missing_coverage() {
    partial::<f32>();
    partial::<f64>();
}
#[test]
fn contradictory_continued_box_order_is_not_accepted() {
    contradictory::<f32>();
    contradictory::<f64>();
}
