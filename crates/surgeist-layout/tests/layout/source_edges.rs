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
                        metric_contribution: contribution.clone(),
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
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
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
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: base,
            },
            text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
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
    let a = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
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
        Vec::new(),
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
                        Vec::new(),
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
            scope: None,
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
        Vec::new(),
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

fn semantic_source(
    left: &[u16],
    right: &[u16],
    delimiters: text::DocumentSegmentBreaks,
    lithuanian: bool,
) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let split = left.len();
    let mut units = left.to_vec();
    units.extend_from_slice(right);
    let end = units.len();
    let source = text::Source::identified(
        text::Text::from_code_units(units),
        text::Id::from_u64(657000),
        17,
    );
    let mut style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::None,
        hyphens: text::Hyphens::None,
        ..text::Style::default()
    };
    if lithuanian {
        style.locale = Some("lt".into());
        style.text_transform = text::TextTransform::Set(
            text::TextTransformSet::none().with_case(text::CaseTransform::Uppercase),
        );
    }
    // Each source occurrence selects its real half-range from the same exact
    // aggregate source, matching the lower Text scalar/delimiter fixture.
    text::ContextualSourceInput {
        joint_flows: Vec::new(),
        scope_alignment_limits: None,
        source: source.clone(),
        segment_breaks: delimiters,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins: [text::Range::new(0, split), text::Range::new(split, end)]
            .into_iter()
            .map(|range| text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
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
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: text::ParagraphBaseDirection::LeftToRight,
            },
            text::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: None,
                boundaries: None,
            },
        ],
        paragraphs: vec![text::SourceProcessingParagraph {
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
        }],
        events: vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: text::Affinity::After,
            },
            E::EnterScope(1),
            E::Connection(0),
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(1),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    }
}
fn source_phase_casing_boundary<S: layout::LayoutScalar>() {
    for edge in [layout::WrapEdge::Auto, layout::WrapEdge::Line] {
        let host = Host::<S>::new(true, edge, layout::Direction::Ltr);
        let result = {
            let view = Early(&host);
            layout::process_wrapping_source(
                &view,
                0,
                semantic_source(
                    &[0x69],
                    &[0x307],
                    text::DocumentSegmentBreaks::LineFeedDefault,
                    true,
                ),
                host.occurrences(),
            )
            .unwrap()
        };
        let (processed, _receipt) = result.into_parts();
        // This is intentionally Phase-I/source proof. Casing executes in the
        // later owned analysis transform: ProcessedSource must NOT be mistaken
        // for its private transformed text or a font/glyph observation.
        assert_eq!(processed.source().id(), Some(text::Id::from_u64(657000)));
        assert_eq!(processed.source().revision(), 17);
        assert_eq!(processed.source().text().code_units(), [0x69, 0x307]);
        assert_eq!(processed.processed_text().code_units(), [0x69, 0x307]);
        assert_eq!(processed.base_style().locale.as_deref(), Some("lt"));
        assert_eq!(
            processed.base_style().text_transform,
            text::TextTransform::Set(
                text::TextTransformSet::none().with_case(text::CaseTransform::Uppercase)
            )
        );
        for at in 0..=2 {
            for affinity in [text::Affinity::Before, text::Affinity::After] {
                let original =
                    text::SourcePosition::try_new(processed.source().text(), at).unwrap();
                assert_eq!(
                    processed
                        .source_to_processed(original, affinity)
                        .unwrap()
                        .get(),
                    at
                );
                assert_eq!(
                    processed
                        .processed_to_source(processed.position(at).unwrap(), affinity)
                        .unwrap()
                        .get(),
                    at
                );
            }
        }
        let scope = processed.captured_scope(1).unwrap();
        assert_eq!(scope.parent(), processed.captured_scope(0));
        assert!(processed.captured_origin(0).is_some());
        assert!(processed.captured_origin(1).is_some());
        assert!(processed.captured_control(0).is_none());
        if edge == layout::WrapEdge::Line {
            let cause = processed.captured_computed_line_boundary(0).unwrap();
            assert_eq!(
                cause.target(),
                text::CapturedComputedLineBoundaryTarget::Scope {
                    scope,
                    side: text::SourceBoxSide::Before
                }
            );
            assert_eq!(cause.source_position().get(), 1);
            assert_eq!(cause.domain(), processed.captured_scope(0).unwrap());
            assert_eq!(cause.paragraph(), processed.captured_paragraph(0).unwrap());
            assert!(processed.captured_computed_line_boundary(1).is_none());
        } else {
            assert!(processed.captured_computed_line_boundary(0).is_none());
        }
    }
}
#[test]
fn source_only_lithuanian_input_keeps_phase_i_mapping_and_authentic_early_dot_boundary_cause() {
    source_phase_casing_boundary::<f32>();
    source_phase_casing_boundary::<f64>();
}
fn typed_early_text_errors<S: layout::LayoutScalar>() {
    for (left, right, delimiters, expected) in [
        (
            vec![0xd83d],
            vec![0xde00],
            text::DocumentSegmentBreaks::LineFeedDefault,
            text::ComputedLineBoundaryError::InsideScalar,
        ),
        (
            vec![0x0d],
            vec![0x0a],
            text::DocumentSegmentBreaks::Defined(vec![text::Range::new(0, 2)]),
            text::ComputedLineBoundaryError::InsideDelimiter,
        ),
    ] {
        // This is the exact requested-force error profile. Do not assert a
        // successful neutral control for defined CRLF crossing the scope:
        // lower Text separately rejects that unforced input as DelimiterCrossing.
        let host = Host::<S>::new(true, layout::WrapEdge::Line, layout::Direction::Ltr);
        let result = {
            let view = Early(&host);
            layout::process_wrapping_source(
                &view,
                0,
                semantic_source(&left, &right, delimiters, false),
                host.occurrences(),
            )
        };
        let error = match result {
            Ok(_) => {
                panic!("an interior decoded scalar/document delimiter is not a hard-cut target")
            }
            Err(error) => error,
        };
        match error.kind() {
            layout::LayoutErrorKindOf::Text(cause) => assert!(
                matches!(cause.detail(),
                    Some(text::ErrorDetail::SourceProcessing {reason:text::SourceProcessingError::ComputedLineBoundary {request:0,reason}}) if *reason==expected
                ),
                "{cause:?}"
            ),
            other => panic!("the actual Text admission detail must be retained: {other:?}"),
        }
    }
}
#[test]
fn source_wrapper_preserves_typed_inside_scalar_and_defined_crlf_boundary_refusals() {
    typed_early_text_errors::<f32>();
    typed_early_text_errors::<f64>();
}

fn association_error<S: layout::LayoutScalar>(error: layout::LayoutErrorOf<u32, S>) {
    assert!(matches!(
        error.kind(),
        layout::LayoutErrorKindOf::InvalidInput(layout::LayoutInvalidInputOf::WrappingScope(
            layout::WrappingScopeError::SourceAssociation
        ))
    ));
}
fn contrary_canonical_order<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        // The source is independently valid and canonically contains A in S;
        // the borrowed producer instead claims A followed by S containing B.
        // Equal source lengths and a valid input-local scope ordinal do not
        // authenticate that contrary first-child/order claim.
        let raw = text::process_contextual_source(source(false, direction)).unwrap();
        let scope = raw.captured_scope(1).unwrap();
        let a = raw.captured_origin(0).unwrap();
        let b = raw.captured_origin(1).unwrap();
        assert!(scope.enter_position() < a);
        assert!(raw.captured_origin(0).unwrap() < scope.exit_position());
        assert!(scope.exit_position() < b);
        assert_eq!(
            scope.domain_topology(),
            text::SourceProcessingTopology::Complete
        );
        assert!(raw.captured_computed_line_boundary(0).is_none());
        let host = Host::<S>::new(true, layout::WrapEdge::Line, direction);
        let result = layout::process_wrapping_source(
            &Early(&host),
            0,
            source(false, direction),
            host.occurrences(),
        );
        match result {
            Ok(_) => panic!("a complete contrary generated order cannot issue a resolved receipt"),
            Err(error) => association_error(error),
        }
    }
}
#[test]
fn complete_source_capture_contradicts_claimed_generated_order_before_receipt_publication() {
    contrary_canonical_order::<f32>();
    contrary_canonical_order::<f64>();
}

fn coherent_foreign_capture<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        let mut host = Host::<S>::new(true, layout::WrapEdge::Line, direction);
        let process = || {
            layout::process_wrapping_source(
                &Early(&host),
                0,
                source(true, direction),
                host.occurrences(),
            )
            .unwrap()
            .into_parts()
        };
        let (original, receipt) = process();
        let (foreign, foreign_receipt) = process();
        assert_eq!(original.source().id(), foreign.source().id());
        assert_eq!(original.source().revision(), foreign.source().revision());
        assert_eq!(original.source().text(), foreign.source().text());
        assert!(
            !original
                .captured_origin(0)
                .unwrap()
                .same_capture(&foreign.captured_origin(0).unwrap())
        );
        let foreign = Arc::new(foreign);
        let p = normal_prepare(&foreign);
        let binding = host.install(&p);
        // Every supplied participant, scope and current row is internally
        // coherent with the second capture. Its own receipt remains valid.
        let positive = finish(
            &host,
            &foreign,
            &p,
            binding.clone(),
            &foreign_receipt,
            layout::InlineBoxMetricContribution::NoContribution,
        );
        qualify(&positive, &p, direction, true, &[10.0, 10.0], &[8.0, 18.0]);
        let tree = Final(&host);
        let result = layout::LayoutFlowInput::try_new_with_edges(
            &tree,
            0,
            vec![binding],
            host.final_wrapping(
                &foreign,
                Some(layout::InlineBoxMetricContribution::NoContribution),
            ),
            Vec::new(),
            &receipt,
        );
        let error = result
            .err()
            .expect("equal source bytes do not make a foreign capture authentic");
        assert!(matches!(
            error.kind(),
            layout::LayoutErrorKindOf::InvalidInput(layout::LayoutInvalidInputOf::WrappingScope(
                layout::WrappingScopeError::PlanMismatch
            ))
        ));
    }
}
#[test]
fn internally_coherent_foreign_capture_refuses_old_receipt_despite_equal_source_and_geometry() {
    coherent_foreign_capture::<f32>();
    coherent_foreign_capture::<f64>();
}

fn caller_causes_and_duplicate_ordinals<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        let host = Host::<S>::new(true, layout::WrapEdge::Line, direction);
        let mut raw = source(true, direction);
        // A caller's After cause is distinct. Two literal caller Before
        // requests and the adapter's same genuine Before edge share one cause.
        raw.computed_line_boundaries = vec![
            text::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::SourceBoxSide::After,
            },
            text::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::SourceBoxSide::Before,
            },
            text::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::SourceBoxSide::Before,
            },
        ];
        let (processed, _receipt) =
            layout::process_wrapping_source(&Early(&host), 0, raw, host.occurrences())
                .unwrap()
                .into_parts();
        assert_eq!(processed.source().text(), &text::Text::from("AB"));
        assert_eq!(processed.processed_text().code_units(), [0x41, 0x42]);
        assert!(processed.captured_control(0).is_none());
        let after = processed.captured_computed_line_boundary(0).unwrap();
        let before = processed.captured_computed_line_boundary(1).unwrap();
        assert_ne!(after, before);
        assert_eq!(
            Some(before.clone()),
            processed.captured_computed_line_boundary(2)
        );
        assert_eq!(
            Some(before.clone()),
            processed.captured_computed_line_boundary(3)
        );
        assert!(processed.captured_computed_line_boundary(4).is_none());
        let s = processed.captured_scope(1).unwrap();
        assert_eq!(
            after.target(),
            text::CapturedComputedLineBoundaryTarget::Scope {
                scope: s.clone(),
                side: text::SourceBoxSide::After,
            }
        );
        assert_eq!(
            before.target(),
            text::CapturedComputedLineBoundaryTarget::Scope {
                scope: s,
                side: text::SourceBoxSide::Before,
            }
        );
        assert_eq!(after.source_position().get(), 2);
        assert_eq!(before.source_position().get(), 1);
        assert!(before.preceding_event().unwrap() < before.following_event().unwrap());
        assert_eq!(after.domain(), before.domain());
        assert_eq!(after.paragraph(), before.paragraph());
    }
}
#[test]
fn caller_distinct_cause_survives_and_duplicate_request_ordinals_resolve_idempotently() {
    caller_causes_and_duplicate_ordinals::<f32>();
    caller_causes_and_duplicate_ordinals::<f64>();
}

// A separate early borrow with genuine physical source controls. It never
// implements LayoutTree or pretends that preparation has already occurred.
struct PairedEarly<S: layout::LayoutScalar> {
    base: Host<S>,
    children: [Vec<u32>; 5],
    start: layout::InlineBoundaryInputOf<S>,
    end: layout::InlineBoundaryInputOf<S>,
    declarations: Vec<layout::SourceWrappingScopeInput<u32>>,
    scopes: Vec<layout::SourceScopeAssociation<u32>>,
}
impl<S: layout::LayoutScalar> PairedEarly<S> {
    fn new(direction: layout::Direction, reverse: bool) -> Self {
        let mut base = Host::new(true, layout::WrapEdge::Line, direction);
        let pair = layout::SourceWrappingScopeInput {
            box_: layout::InlineBoxRef::NativePair { start: 3, end: 4 },
            wrap_inside: layout::WrapInside::Auto,
            edges: base.declarations[0].edges,
        };
        let mut declarations = vec![base.declarations[0].clone(), pair];
        if reverse {
            declarations.reverse();
        }
        let metrics = layout::InlineMetricsOf::try_new(
            surgeist_layout::InlineBaselineContext::Physical,
            S::ZERO,
            S::ZERO,
        )
        .unwrap();
        let start = layout::InlineBoundaryInputOf::new(
            layout::InlineBoundaryKind::Start,
            metrics,
            layout::InlineBoundaryEdgeOf::ZERO,
        )
        .with_direction(direction);
        let end = layout::InlineBoundaryInputOf::new(
            layout::InlineBoundaryKind::End,
            metrics,
            layout::InlineBoundaryEdgeOf::ZERO,
        )
        .with_direction(direction);
        // The physical flow contains the actual pair's endpoints, while the
        // generated tree contains the one paired inline box, not two children.
        base.children[0] = vec![1, 3, 2, 4];
        Self {
            base,
            children: [vec![1, 3, 2, 4], vec![], vec![], vec![], vec![]],
            start,
            end,
            declarations,
            scopes: vec![layout::SourceScopeAssociation {
                scope: 1,
                flow: 0,
                pair: Some((3, 4)),
            }],
        }
    }
    fn occurrences(&self) -> layout::SourceOccurrenceAssociations<u32> {
        layout::SourceOccurrenceAssociations {
            origins: self.base.origins.clone(),
            controls: vec![(0, 3), (1, 4)],
            atomics: vec![],
        }
    }
}
impl<S: layout::LayoutScalar> layout::Traverse for PairedEarly<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}
impl<S: layout::LayoutScalar> layout::WrapEdgeSource for PairedEarly<S> {
    fn wrap_edge_source_node(&self, node: u32) -> layout::WrapEdgeSourceNodeView<'_, S> {
        match node {
            0 => layout::WrapEdgeSourceNodeView::Box(&self.base.root),
            1 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 0 },
            2 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 1 },
            3 => layout::WrapEdgeSourceNodeView::Boundary(self.start),
            4 => layout::WrapEdgeSourceNodeView::Boundary(self.end),
            _ => unreachable!("the fixture has exactly five genuine nodes"),
        }
    }
    fn source_wrapping_scopes(&self) -> &[layout::SourceWrappingScopeInput<u32>] {
        &self.declarations
    }
    fn source_scope_associations(&self) -> &[layout::SourceScopeAssociation<u32>] {
        &self.scopes
    }
    fn generated_child_order(&self) -> &[layout::InlineChildOrderInput<u32, usize>] {
        &self.base.orders
    }
}
fn paired_source(direction: layout::Direction) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let mut raw = source(true, direction);
    raw.controls = vec![
        text::ParagraphControl {
            at: 1,
            kind: text::ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: 2,
            kind: text::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let text::SourceProcessingScope::Inline { boundaries, .. } = &mut raw.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 });
    raw.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        },
        E::Connection(0),
        E::EnterScope(1),
        E::Control {
            control: 0,
            affinity: text::Affinity::After,
        },
        E::Origin {
            origin: 1,
            affinity: text::Affinity::After,
        },
        E::Control {
            control: 1,
            affinity: text::Affinity::Before,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    raw.computed_line_boundaries = vec![
        text::ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: text::SourceBoxSide::Before,
        },
        text::ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: text::SourceBoxSide::Before,
        },
        text::ComputedLineBoundaryInput::Endpoint { control: 0 },
    ];
    raw
}
fn genuine_source_native_alias<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for reverse in [false, true] {
            let host = PairedEarly::<S>::new(direction, reverse);
            let (processed, _receipt) = layout::process_wrapping_source(
                &host,
                0,
                paired_source(direction),
                host.occurrences(),
            )
            .unwrap()
            .into_parts();
            let scope = processed.captured_scope(1).unwrap();
            let start = processed.captured_control(0).unwrap();
            let end = processed.captured_control(1).unwrap();
            assert_eq!(scope.entry_control(), Some(start.clone()));
            assert_eq!(scope.exit_control(), Some(end.clone()));
            assert_eq!(
                start.kind(),
                &text::ParagraphControlKind::InlineBoundaryStart
            );
            assert_eq!(end.kind(), &text::ParagraphControlKind::InlineBoundaryEnd);
            assert!(start.event_position() < processed.captured_origin(1).unwrap());
            assert!(processed.captured_origin(1).unwrap() < end.event_position());
            let before = processed.captured_computed_line_boundary(0).unwrap();
            let endpoint = processed.captured_computed_line_boundary(2).unwrap();
            assert_eq!(
                Some(before.clone()),
                processed.captured_computed_line_boundary(1)
            );
            assert_eq!(
                Some(before.clone()),
                processed.captured_computed_line_boundary(3)
            );
            assert!(processed.captured_computed_line_boundary(4).is_none());
            assert_eq!(
                before.target(),
                text::CapturedComputedLineBoundaryTarget::Scope {
                    scope: scope.clone(),
                    side: text::SourceBoxSide::Before,
                }
            );
            assert_eq!(
                endpoint.target(),
                text::CapturedComputedLineBoundaryTarget::Endpoint(start)
            );
            assert_ne!(before, endpoint);
            assert_eq!(processed.source().text(), &text::Text::from("AB"));
            assert_eq!(processed.processed_text().code_units(), [0x41, 0x42]);
            let processed = Arc::new(processed);
            let prepared = normal_prepare(&processed);
            assert_eq!(prepared.control_views().count(), 2);
            assert_eq!(
                prepared
                    .participants()
                    .filter(|p| matches!(p, text::PreparedParticipant::ComputedLineBoundary(_)))
                    .count(),
                1
            );
            let offers = prepared.opportunity_views().collect::<Vec<_>>();
            assert_eq!(
                offers
                    .iter()
                    .map(|o| (o.kind(), o.boundary().view().source_before().get()))
                    .collect::<Vec<_>>(),
                [
                    (text::PreparedBreakKind::Forced, 1),
                    (text::PreparedBreakKind::ParagraphEnd, 2),
                ]
            );
            let group = offers[0].computed_line_boundary().unwrap();
            let causes = group.view().causes().collect::<Vec<_>>();
            assert_eq!(causes.len(), 2);
            assert_eq!(causes.iter().filter(|c| **c == before).count(), 1);
            assert_eq!(causes.iter().filter(|c| **c == endpoint).count(), 1);
            assert_eq!(group.view().domain(), scope.domain());
        }
    }
}
#[test]
fn genuine_source_native_box_alias_has_one_effective_force_and_keeps_distinct_caller_endpoint_cause()
 {
    genuine_source_native_alias::<f32>();
    genuine_source_native_alias::<f64>();
}

mod unscoped_native_pair;

mod phantom_receivers;
