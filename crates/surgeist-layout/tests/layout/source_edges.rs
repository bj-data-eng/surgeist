//! Public source-to-layout lifetime, forced-edge and semantic receipt checks.
use std::{convert::Infallible, slice, sync::Arc};
use surgeist_layout as layout;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Host<S: layout::LayoutScalar> {
    root: layout::NodeInputOf<S>,
    non_box: layout::NodeInputOf<S>,
    children: [Vec<u32>; 3],
    origins: Vec<(usize, u32)>,
    declarations: Vec<layout::SourceWrappingScopeInput<u32>>,
    scopes: Vec<layout::SourceScopeAssociation<u32>>,
    orders: Vec<layout::InlineChildOrderInput<u32, usize>>,
    installed: Vec<Option<layout::LayoutInputOf<S>>>,
}
impl<S: layout::LayoutScalar> Host<S> {
    fn new(before: bool, edge: layout::WrapEdge, direction: layout::Direction) -> Self {
        let s = layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::Scope(1));
        let child = |box_| layout::GeneratedChildBoxInput {
            box_,
            participation: layout::GeneratedBoxParticipation::InFlow,
        };
        let row = |parent, children| layout::InlineChildOrderInput {
            parent,
            children,
            prefix: layout::ChildOrderCompleteness::Complete,
            suffix: layout::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        Self {
            root: layout::NodeInputOf {
                display: layout::Display::Block,
                direction,
                size: layout::Size::new(
                    layout::PreferredSizeOf::value(
                        layout::LengthPercentageOf::px(S::from_f64(40.0)).unwrap(),
                    ),
                    layout::PreferredSizeOf::AUTO,
                ),
                ..layout::NodeInputOf::default()
            },
            non_box: layout::NodeInputOf::non_box(),
            children: [vec![1, 2], vec![], vec![]],
            origins: vec![(0, 1), (1, 2)],
            declarations: vec![layout::SourceWrappingScopeInput {
                box_: layout::InlineBoxRef::Scope(1),
                wrap_inside: layout::WrapInside::Auto,
                edges: if before {
                    layout::WrapEdges {
                        before: edge,
                        after: layout::WrapEdge::Auto,
                    }
                } else {
                    layout::WrapEdges {
                        before: layout::WrapEdge::Auto,
                        after: edge,
                    }
                },
            }],
            scopes: vec![layout::SourceScopeAssociation {
                scope: 1,
                flow: 0,
                pair: None,
            }],
            orders: vec![
                row(
                    layout::GeneratedBoxRef::Node(0),
                    if before {
                        vec![child(layout::GeneratedBoxRef::Node(1)), child(s.clone())]
                    } else {
                        vec![child(s.clone()), child(layout::GeneratedBoxRef::Node(2))]
                    },
                ),
                row(
                    s,
                    vec![child(layout::GeneratedBoxRef::Node(if before {
                        2
                    } else {
                        1
                    }))],
                ),
            ],
            installed: vec![None, None, None],
        }
    }
    fn occurrences(&self) -> layout::SourceOccurrenceAssociations<u32> {
        layout::SourceOccurrenceAssociations {
            origins: self.origins.clone(),
            controls: vec![],
            atomics: vec![],
        }
    }
    fn install(&mut self, p: &text::PreparedText) -> layout::TextParagraphBinding<u32> {
        let origins = p
            .origin_views()
            .enumerate()
            .map(|(ordinal, origin)| {
                let node = self.origins.iter().find(|(i, _)| *i == ordinal).unwrap().1;
                assert_eq!(
                    origin.source().id(),
                    Some(text::Id::from_u64(656001 + ordinal as u64))
                );
                assert_eq!(origin.source().revision(), 15);
                assert_eq!(
                    origin.aggregate_range(),
                    text::Range::new(ordinal, ordinal + 1)
                );
                self.installed[node as usize] = Some(layout::LayoutInputOf::inline_text(
                    layout::TextParticipation::new(origin.handle()),
                ));
                layout::TextOriginBinding {
                    origin: origin.handle(),
                    node,
                }
            })
            .collect();
        layout::TextParagraphBinding {
            prepared: p.clone(),
            container: 0,
            nodes: self.children[0].clone(),
            origins,
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        }
    }
    fn final_wrapping(
        &self,
        processed: &text::ProcessedSource,
        contribution: Option<layout::InlineBoxMetricContribution>,
    ) -> layout::WrappingInput<u32> {
        let convert = |box_: &layout::GeneratedBoxRef<u32, usize>| match box_ {
            layout::GeneratedBoxRef::Node(node) => layout::GeneratedBoxRef::Node(*node),
            layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::Scope(scope)) => {
                layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::Scope(
                    processed.captured_scope(*scope).unwrap(),
                ))
            }
            layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::NativePair { start, end }) => {
                layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::NativePair {
                    start: *start,
                    end: *end,
                })
            }
        };
        layout::WrappingInput {
            scopes: self
                .declarations
                .iter()
                .map(|d| {
                    let layout::InlineBoxRef::Scope(scope) = d.box_ else {
                        unreachable!("this fixture authors a genuine source scope")
                    };
                    layout::WrappingScopeInput::Captured {
                        scope: processed.captured_scope(scope).unwrap(),
                        wrap_inside: d.wrap_inside,
                        edges: d.edges,
                        metric_contribution: contribution,
                    }
                })
                .collect(),
            child_order: self
                .orders
                .iter()
                .map(|o| layout::InlineChildOrderInput {
                    parent: convert(&o.parent),
                    children: o
                        .children
                        .iter()
                        .map(|c| layout::GeneratedChildBoxInput {
                            box_: convert(&c.box_),
                            participation: c.participation,
                        })
                        .collect(),
                    prefix: o.prefix,
                    suffix: o.suffix,
                    formatting_context: o.formatting_context,
                })
                .collect(),
        }
    }
}
struct Early<'a, S: layout::LayoutScalar>(&'a Host<S>);
struct Final<'a, S: layout::LayoutScalar>(&'a Host<S>);
macro_rules! traversal {
    ($view:ident) => {
        impl<S: layout::LayoutScalar> layout::Traverse for $view<'_, S> {
            type Node = u32;
            type Scalar = S;
            type Children<'a>
                = std::iter::Copied<slice::Iter<'a, u32>>
            where
                Self: 'a;
            fn children(&self, node: u32) -> Self::Children<'_> {
                self.0.children[node as usize].iter().copied()
            }
            fn child_count(&self, node: u32) -> usize {
                self.0.children[node as usize].len()
            }
            fn child(&self, node: u32, index: usize) -> u32 {
                self.0.children[node as usize][index]
            }
        }
        impl<S: layout::LayoutScalar> layout::WrapEdgeSource for $view<'_, S> {
            fn wrap_edge_source_node(&self, node: u32) -> layout::WrapEdgeSourceNodeView<'_, S> {
                if node == 0 {
                    layout::WrapEdgeSourceNodeView::Box(&self.0.root)
                } else {
                    layout::WrapEdgeSourceNodeView::SourceOrigin {
                        origin: self.0.origins.iter().find(|(_, n)| *n == node).unwrap().0,
                    }
                }
            }
            fn source_wrapping_scopes(&self) -> &[layout::SourceWrappingScopeInput<u32>] {
                &self.0.declarations
            }
            fn source_scope_associations(&self) -> &[layout::SourceScopeAssociation<u32>] {
                &self.0.scopes
            }
            fn generated_child_order(&self) -> &[layout::InlineChildOrderInput<u32, usize>] {
                &self.0.orders
            }
        }
    };
}
traversal!(Early);
traversal!(Final);
// Early intentionally has no LayoutTree implementation or prepared input.
impl<S: layout::LayoutScalar> layout::LayoutTree for Final<'_, S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &layout::NodeInputOf<S> {
        if node == 0 {
            &self.0.root
        } else {
            &self.0.non_box
        }
    }
    fn layout_input(&self, node: u32) -> layout::LayoutInputViewOf<'_, S> {
        if node == 0 {
            layout::LayoutInputViewOf::Box(&self.0.root)
        } else {
            self.0.installed[node as usize]
                .as_ref()
                .expect("normal preparation was installed only after the early borrow ended")
                .as_view()
        }
    }
}
fn source(before: bool, direction: layout::Direction) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let base = if direction == layout::Direction::Ltr {
        text::ParagraphBaseDirection::LeftToRight
    } else {
        text::ParagraphBaseDirection::RightToLeft
    };
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        wrap: text::Wrap::None,
        hyphens: text::Hyphens::None,
        ..text::Style::default()
    };
    let mut source = text::Source::identified("AB", text::Id::from_u64(656000), 15);
    source.span(text::Range::new(0, 2), style.clone());
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if before {
        events.extend([
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(1),
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
        ]);
    } else {
        events.extend([
            E::EnterScope(1),
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
            E::Connection(0),
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
        ]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    text::ContextualSourceInput {
        source,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins: ["A", "B"]
            .into_iter()
            .enumerate()
            .map(|(i, raw)| text::ParagraphOrigin {
                text_node: None,
                source: text::Source::identified(raw, text::Id::from_u64(656001 + i as u64), 15),
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(i, i + 1),
                kind: text::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![text::SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: text::ParagraphConnectionKind::Continuous,
        }],
        boundary_policies: vec![text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: text::Wrap::None,
            following_wrap: text::Wrap::None,
            ancestor_wrap: text::Wrap::None,
        }],
        controls: vec![],
        computed_line_boundaries: vec![],
        replacements: vec![],
        scopes: vec![
            text::SourceProcessingScope::Block {
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: base,
            },
            text::SourceProcessingScope::Inline {
                trim: Default::default(),
                style: None,
                boundaries: None,
            },
        ],
        paragraphs: vec![text::SourceProcessingParagraph {
            base_direction: base,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
        }],
        events,
    }
}
fn normal_prepare(processed: &Arc<text::ProcessedSource>) -> text::PreparedText {
    let a = text::AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
        .unwrap();
    let missing = a
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| text::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: text::Wrap::None,
        })
        .collect();
    let a = a.with_wrapping_policies(missing).unwrap();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
        .prepare(&a, text::PreparationOptions::default())
        .unwrap()
}
fn request<S: layout::LayoutScalar>() -> layout::LayoutRootRequestOf<S> {
    layout::LayoutRootRequestOf::viewport(layout::Size::new(
        layout::AvailableOf::definite(S::from_f64(40.0)),
        layout::AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}
fn finish<S: layout::LayoutScalar>(
    host: &Host<S>,
    processed: &text::ProcessedSource,
    p: &text::PreparedText,
    binding: layout::TextParagraphBinding<u32>,
    receipt: &layout::ResolvedWrapEdges<u32>,
    metric: layout::InlineBoxMetricContribution,
) -> layout::CompletedLayoutBatchOf<u32, S> {
    let tree = Final(host);
    let flow = layout::LayoutFlowInput::try_new_with_edges(
        &tree,
        0,
        vec![binding],
        host.final_wrapping(processed, Some(metric)),
        receipt,
    )
    .unwrap();
    let result = layout::compute_layout_with_flow(&flow, 0, request()).unwrap();
    assert!(
        result.placed_text()[0]
            .layout()
            .analyzed_source()
            .identity()
            == p.analyzed_source().identity()
    );
    result
}
fn qualify<S: layout::LayoutScalar>(
    result: &layout::CompletedLayoutBatchOf<u32, S>,
    p: &text::PreparedText,
    direction: layout::Direction,
    forced: bool,
    heights: &[f64],
    baselines: &[f64],
) {
    let placed = result.placed_text()[0].layout();
    assert_eq!(placed.source().id(), Some(text::Id::from_u64(656000)));
    assert_eq!(placed.source().revision(), 15);
    assert_eq!(placed.source().text(), &text::Text::from("AB"));
    assert_eq!(
        placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        if forced {
            vec![text::Range::new(0, 1), text::Range::new(1, 2)]
        } else {
            vec![text::Range::new(0, 2)]
        }
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        heights
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        baselines
    );
    let root = result
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        layout::Size::new(S::from_f64(40.0), S::from_f64(heights.iter().sum()))
    );
    let mut glyphs = vec![];
    for run in placed.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            let x = run.origin().x + glyph.pen() + f64::from(glyph.x());
            let expected = if direction == layout::Direction::Ltr {
                if forced {
                    0.0
                } else {
                    glyphs.len() as f64 * 10.0
                }
            } else if forced {
                30.0
            } else {
                20.0 + glyphs.len() as f64 * 10.0
            };
            assert_eq!(x, expected);
            glyphs.push((glyph.id(), glyph.advance()));
        }
    }
    assert_eq!(glyphs, [(35, 10.0), (36, 10.0)]);
    let offers = p
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().source_before().get()))
        .collect::<Vec<_>>();
    assert_eq!(
        offers,
        if forced {
            vec![
                (text::PreparedBreakKind::Forced, 1),
                (text::PreparedBreakKind::ParagraphEnd, 2),
            ]
        } else {
            vec![(text::PreparedBreakKind::ParagraphEnd, 2)]
        }
    );
    assert_eq!(p.control_views().count(), 0);
    assert_eq!(
        p.participants()
            .filter(|part| matches!(part, text::PreparedParticipant::ComputedLineBoundary(_)))
            .count(),
        usize::from(forced)
    );
}
fn first_pipeline<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for before in [false, true] {
            for edge in [layout::WrapEdge::Auto, layout::WrapEdge::Line] {
                let mut host = Host::<S>::new(before, edge, direction);
                let output = {
                    let early = Early(&host);
                    layout::process_wrapping_source(
                        &early,
                        0,
                        source(before, direction),
                        host.occurrences(),
                    )
                    .unwrap()
                };
                let (processed, receipt) = output.into_parts();
                let processed = Arc::new(processed);
                let p = normal_prepare(&processed); // first font system exists only now
                let binding = host.install(&p); // mutable install proves early borrow ended
                let forced = edge == layout::WrapEdge::Line;
                if forced {
                    let cause = processed.captured_computed_line_boundary(0).unwrap();
                    assert_eq!(
                        cause.target(),
                        text::CapturedComputedLineBoundaryTarget::Scope {
                            scope: processed.captured_scope(1).unwrap(),
                            side: if before {
                                text::SourceBoxSide::Before
                            } else {
                                text::SourceBoxSide::After
                            }
                        }
                    );
                    assert_eq!(cause.domain(), processed.captured_scope(0).unwrap());
                    let tree = Final(&host);
                    let error = layout::LayoutFlowInput::try_new(
                        &tree,
                        0,
                        vec![binding.clone()],
                        host.final_wrapping(
                            &processed,
                            Some(layout::InlineBoxMetricContribution::NoContribution),
                        ),
                    )
                    .err()
                    .expect("ordinary late route does not authenticate an early Layout plan");
                    assert!(matches!(
                        error.kind(),
                        layout::LayoutErrorKindOf::InvalidInput(
                            layout::LayoutInvalidInputOf::WrappingScope(
                                layout::WrappingScopeError::SourceProcessingRequired
                            )
                        )
                    ));
                }
                let batch = finish(
                    &host,
                    &processed,
                    &p,
                    binding,
                    &receipt,
                    layout::InlineBoxMetricContribution::NoContribution,
                );
                qualify(
                    &batch,
                    &p,
                    direction,
                    forced,
                    if forced { &[10.0, 10.0] } else { &[10.0] },
                    if forced { &[8.0, 18.0] } else { &[8.0] },
                );
            }
        }
    }
}
#[test]
fn real_early_borrow_normal_prepare_and_final_source_receipt_execute_before_after_force_under_nowrap()
 {
    first_pipeline::<f32>();
    first_pipeline::<f64>();
}
fn reusable_receipt<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        let mut host = Host::<S>::new(true, layout::WrapEdge::Line, direction);
        let output = {
            let early = Early(&host);
            layout::process_wrapping_source(&early, 0, source(true, direction), host.occurrences())
                .unwrap()
        };
        let (processed, receipt) = output.into_parts();
        let processed = Arc::new(processed);
        let p1 = normal_prepare(&processed);
        let b1 = host.install(&p1);
        let old = finish(
            &host,
            &processed,
            &p1,
            b1,
            &receipt,
            layout::InlineBoxMetricContribution::NoContribution,
        );
        let p2 = normal_prepare(&processed);
        let b2 = host.install(&p2);
        let metric = layout::InlineBoxMetricContribution::Strut(text::LineBoxStrut {
            metrics: text::LineStrut {
                baseline: 16.0,
                line_extent: 20.0,
            },
            alignment: text::LineBoxAlignment::Baseline,
        });
        let current = finish(&host, &processed, &p2, b2, &receipt, metric);
        qualify(&old, &p1, direction, true, &[10.0, 10.0], &[8.0, 18.0]);
        qualify(&current, &p2, direction, true, &[10.0, 20.0], &[8.0, 26.0]);
    }
}
#[test]
fn same_owned_early_receipt_accepts_fresh_coherent_preparation_and_final_only_used_metrics() {
    reusable_receipt::<f32>();
    reusable_receipt::<f64>();
}
fn changed_declaration<S: layout::LayoutScalar>() {
    let mut host = Host::<S>::new(true, layout::WrapEdge::Line, layout::Direction::Ltr);
    let output = {
        let early = Early(&host);
        layout::process_wrapping_source(
            &early,
            0,
            source(true, layout::Direction::Ltr),
            host.occurrences(),
        )
        .unwrap()
    };
    let (processed, receipt) = output.into_parts();
    let processed = Arc::new(processed);
    let p = normal_prepare(&processed);
    let binding = host.install(&p);
    host.declarations[0].edges = layout::WrapEdges::AUTO;
    let tree = Final(&host);
    let error = layout::LayoutFlowInput::try_new_with_edges(
        &tree,
        0,
        vec![binding],
        host.final_wrapping(
            &processed,
            Some(layout::InlineBoxMetricContribution::NoContribution),
        ),
        &receipt,
    )
    .err()
    .expect("changed current declaration is not authenticated by old Text cause geometry");
    assert!(matches!(
        error.kind(),
        layout::LayoutErrorKindOf::InvalidInput(layout::LayoutInvalidInputOf::WrappingScope(
            layout::WrappingScopeError::PlanMismatch
        ))
    ));
}
#[test]
fn semantic_declaration_mutation_refuses_even_though_prepared_forced_geometry_still_exists() {
    changed_declaration::<f32>();
    changed_declaration::<f64>();
}
