//! Public early-source propagation with actual four-control BP ownership.
//! P/A/Q are real Ahem10 cells. The actual endpoint advances 5/3/2/4 total14.
//! First-child Before propagates outside S: 10 / 34, receiving A at8,Q at24.
//! Last-child After propagates outside S: 34 / 10, outgoing A at18.
//! An actual empty first child E stops Before at T: 15 / 29, A at3,Q at19.
//! All endpoint metrics are real host Baseline8/10; E has its own explicit8/10
//! witness. Each native line remains10 high with baseline8 (then18 after placement).
use std::{convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));
// Original Ahem cmap/hmtx: P50/A35/Q51 each advance1000, UPEM1000;
// hhea ascent800/descent-200 independently qualifies the 10px 8/2 font frame.
const ORIGIN_NODES: [u32; 3] = [1, 4, 7];
const CONTROL_NODES: [u32; 4] = [2, 3, 5, 6];
const ADVANCES: [f64; 4] = [5.0, 3.0, 2.0, 4.0];

#[derive(Clone, Copy)]
enum Case {
    Before,
    After,
    EmptyBefore,
}

struct Host<S: l::LayoutScalar> {
    root: l::NodeInputOf<S>,
    non_box: l::NodeInputOf<S>,
    children: [Vec<u32>; 8],
    boundaries: [l::InlineBoundaryInputOf<S>; 4],
    declarations: Vec<l::SourceWrappingScopeInput<u32>>,
    associations: Vec<l::SourceScopeAssociation<u32>>,
    orders: Vec<l::InlineChildOrderInput<u32, usize>>,
    installed: Vec<Option<l::LayoutInputOf<S>>>,
}

fn pair(start: u32, end: u32) -> l::InlineBoxRef<u32, usize> {
    l::InlineBoxRef::NativePair { start, end }
}
fn box_pair(start: u32, end: u32) -> l::GeneratedBoxRef<u32, usize> {
    l::GeneratedBoxRef::Inline(pair(start, end))
}
fn edges(case: Case) -> l::WrapEdges {
    match case {
        Case::Before | Case::EmptyBefore => l::WrapEdges {
            before: l::WrapEdge::Line,
            after: l::WrapEdge::Auto,
        },
        Case::After => l::WrapEdges {
            before: l::WrapEdge::Auto,
            after: l::WrapEdge::Line,
        },
    }
}
impl<S: l::LayoutScalar> Host<S> {
    fn new(case: Case) -> Self {
        let child = |box_| l::GeneratedChildBoxInput {
            box_,
            participation: l::GeneratedBoxParticipation::InFlow,
        };
        let order = |parent, children| l::InlineChildOrderInput {
            parent,
            children,
            prefix: l::ChildOrderCompleteness::Complete,
            suffix: l::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        let mut declarations = vec![
            l::SourceWrappingScopeInput {
                box_: pair(2, 6),
                wrap_inside: l::WrapInside::Auto,
                edges: l::WrapEdges::AUTO,
            },
            l::SourceWrappingScopeInput {
                box_: pair(3, 5),
                wrap_inside: l::WrapInside::Auto,
                edges: edges(case),
            },
        ];
        let mut associations = vec![
            l::SourceScopeAssociation {
                scope: 1,
                flow: 0,
                pair: Some((2, 6)),
            },
            l::SourceScopeAssociation {
                scope: 2,
                flow: 0,
                pair: Some((3, 5)),
            },
        ];
        let mut inside_s = vec![child(box_pair(3, 5))];
        if matches!(case, Case::EmptyBefore) {
            declarations.push(l::SourceWrappingScopeInput {
                box_: l::InlineBoxRef::Scope(3),
                wrap_inside: l::WrapInside::Auto,
                edges: l::WrapEdges::AUTO,
            });
            associations.push(l::SourceScopeAssociation {
                scope: 3,
                flow: 0,
                pair: None,
            });
            inside_s.insert(
                0,
                child(l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(3))),
            );
        }
        let mut orders = vec![
            order(
                l::GeneratedBoxRef::Node(0),
                vec![
                    child(l::GeneratedBoxRef::Node(1)),
                    child(box_pair(2, 6)),
                    child(l::GeneratedBoxRef::Node(7)),
                ],
            ),
            order(box_pair(2, 6), inside_s),
            order(box_pair(3, 5), vec![child(l::GeneratedBoxRef::Node(4))]),
        ];
        if matches!(case, Case::EmptyBefore) {
            orders.push(order(
                l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(3)),
                vec![],
            ));
        }
        Self {
            root: l::NodeInputOf {
                display: l::Display::Block,
                size: l::Size::new(
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(100.0)).unwrap(),
                    ),
                    l::PreferredSizeOf::AUTO,
                ),
                ..l::NodeInputOf::default()
            },
            non_box: l::NodeInputOf::non_box(),
            children: std::array::from_fn(|i| if i == 0 { (1..=7).collect() } else { vec![] }),
            boundaries: std::array::from_fn(|i| {
                l::InlineBoundaryInputOf::new(
                    if i < 2 {
                        l::InlineBoundaryKind::Start
                    } else {
                        l::InlineBoundaryKind::End
                    },
                    l::InlineMetricsOf::try_new(S::from_f64(8.0), S::from_f64(10.0)).unwrap(),
                    l::InlineBoundaryEdgeOf::try_new(S::from_f64(ADVANCES[i]), S::ZERO).unwrap(),
                )
            }),
            declarations,
            associations,
            orders,
            installed: (0..8).map(|_| None).collect(),
        }
    }
    fn occurrences(&self) -> l::SourceOccurrenceAssociations<u32> {
        l::SourceOccurrenceAssociations {
            origins: ORIGIN_NODES.into_iter().enumerate().collect(),
            controls: CONTROL_NODES.into_iter().enumerate().collect(),
            atomics: vec![],
        }
    }
    fn install(
        &mut self,
        processed: &t::ProcessedSource,
        p: &t::PreparedText,
    ) -> l::TextParagraphBinding<u32> {
        let origins = p
            .origin_views()
            .enumerate()
            .map(|(i, o)| {
                assert_eq!(o.aggregate_range(), t::Range::new(i, i + 1));
                assert_eq!(o.local_range(), t::Range::new(0, 1));
                assert_eq!(o.source().id(), Some(t::Id::from_u64(658001 + i as u64)));
                assert_eq!(o.source().revision(), 21);
                let node = ORIGIN_NODES[i];
                self.installed[node as usize] = Some(l::LayoutInputOf::inline_text(
                    l::TextParticipation::new(o.handle()),
                ));
                l::TextOriginBinding {
                    origin: o.handle(),
                    node,
                }
            })
            .collect();
        let controls = (0..4)
            .map(|i| {
                let captured = processed.captured_control(i).unwrap();
                let actual = p
                    .control_views()
                    .find(|c| c.handle().captured_control().as_ref() == Some(&captured))
                    .unwrap();
                assert_eq!(
                    actual.kind(),
                    &if i < 2 {
                        t::ParagraphControlKind::InlineBoundaryStart
                    } else {
                        t::ParagraphControlKind::InlineBoundaryEnd
                    }
                );
                let node = CONTROL_NODES[i];
                self.installed[node as usize] =
                    Some(l::LayoutInputOf::inline_boundary(self.boundaries[i]));
                l::TextControlBinding {
                    occurrence: actual.handle(),
                    node,
                }
            })
            .collect();
        l::TextParagraphBinding {
            prepared: p.clone(),
            container: 0,
            nodes: self.children[0].clone(),
            origins,
            controls,
            atomics: vec![],
            line_strut: None,
        }
    }
    fn final_wrapping(&self, processed: &t::ProcessedSource) -> l::WrappingInput<u32> {
        let convert = |b: &l::GeneratedBoxRef<u32, usize>| match b {
            l::GeneratedBoxRef::Node(n) => l::GeneratedBoxRef::Node(*n),
            l::GeneratedBoxRef::Inline(l::InlineBoxRef::NativePair { start, end }) => {
                l::GeneratedBoxRef::Inline(l::InlineBoxRef::NativePair {
                    start: *start,
                    end: *end,
                })
            }
            l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(s)) => l::GeneratedBoxRef::Inline(
                l::InlineBoxRef::Scope(processed.captured_scope(*s).unwrap()),
            ),
        };
        l::WrappingInput {
            scopes: self
                .declarations
                .iter()
                .map(|d| match d.box_ {
                    l::InlineBoxRef::NativePair { start, end } => {
                        l::WrappingScopeInput::NativePair {
                            start,
                            end,
                            wrap_inside: d.wrap_inside,
                            edges: d.edges,
                            metric_contribution: None,
                        }
                    }
                    l::InlineBoxRef::Scope(s) => l::WrappingScopeInput::Captured {
                        scope: processed.captured_scope(s).unwrap(),
                        wrap_inside: d.wrap_inside,
                        edges: d.edges,
                        metric_contribution: Some(l::InlineBoxMetricContribution::Strut(
                            t::LineBoxStrut {
                                metrics: t::LineStrut {
                                    baseline: 8.0,
                                    line_extent: 10.0,
                                },
                                alignment: t::LineBoxAlignment::Baseline,
                            },
                        )),
                    },
                })
                .collect(),
            child_order: self
                .orders
                .iter()
                .map(|o| l::InlineChildOrderInput {
                    parent: convert(&o.parent),
                    children: o
                        .children
                        .iter()
                        .map(|c| l::GeneratedChildBoxInput {
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

struct Early<'a, S: l::LayoutScalar>(&'a Host<S>);
struct Final<'a, S: l::LayoutScalar>(&'a Host<S>);
macro_rules! source_view {
    ($view:ident) => {
        impl<S: l::LayoutScalar> l::Traverse for $view<'_, S> {
            type Node = u32;
            type Scalar = S;
            type Children<'a>
                = std::iter::Copied<slice::Iter<'a, u32>>
            where
                Self: 'a;
            fn children(&self, n: u32) -> Self::Children<'_> {
                self.0.children[n as usize].iter().copied()
            }
            fn child_count(&self, n: u32) -> usize {
                self.0.children[n as usize].len()
            }
            fn child(&self, n: u32, i: usize) -> u32 {
                self.0.children[n as usize][i]
            }
        }
        impl<S: l::LayoutScalar> l::WrapEdgeSource for $view<'_, S> {
            fn wrap_edge_source_node(&self, n: u32) -> l::WrapEdgeSourceNodeView<'_, S> {
                if n == 0 {
                    l::WrapEdgeSourceNodeView::Box(&self.0.root)
                } else if let Some(i) = CONTROL_NODES.iter().position(|&node| node == n) {
                    l::WrapEdgeSourceNodeView::Boundary(self.0.boundaries[i])
                } else {
                    l::WrapEdgeSourceNodeView::SourceOrigin {
                        origin: ORIGIN_NODES.iter().position(|&node| node == n).unwrap(),
                    }
                }
            }
            fn source_wrapping_scopes(&self) -> &[l::SourceWrappingScopeInput<u32>] {
                &self.0.declarations
            }
            fn source_scope_associations(&self) -> &[l::SourceScopeAssociation<u32>] {
                &self.0.associations
            }
            fn generated_child_order(&self) -> &[l::InlineChildOrderInput<u32, usize>] {
                &self.0.orders
            }
        }
    };
}
source_view!(Early);
source_view!(Final);
// Early implements no LayoutTree and cannot borrow prepared participation.
impl<S: l::LayoutScalar> l::LayoutTree for Final<'_, S> {
    type MeasureError = Infallible;
    fn node_input(&self, n: u32) -> &l::NodeInputOf<S> {
        if n == 0 {
            &self.0.root
        } else {
            &self.0.non_box
        }
    }
    fn layout_input(&self, n: u32) -> l::LayoutInputViewOf<'_, S> {
        if n == 0 {
            l::LayoutInputViewOf::Box(&self.0.root)
        } else {
            self.0.installed[n as usize].as_ref().unwrap().as_view()
        }
    }
}

fn source(case: Case) -> t::ContextualSourceInput {
    use t::SourceProcessingEvent as E;
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        wrap: t::Wrap::None,
        hyphens: t::Hyphens::None,
        ..t::Style::default()
    };
    let mut source = t::Source::identified("PAQ", t::Id::from_u64(658000), 21);
    source.span(t::Range::new(0, 3), style.clone());
    let inline = |boundaries| t::SourceProcessingScope::Inline {
        trim: Default::default(),
        style: None,
        boundaries,
    };
    let mut scopes = vec![
        t::SourceProcessingScope::Block {
            topology: t::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style,
            used_direction: t::ParagraphBaseDirection::LeftToRight,
        },
        inline(Some(t::SourceProcessingScopeBoundaries {
            start: 0,
            end: 3,
        })),
        inline(Some(t::SourceProcessingScopeBoundaries {
            start: 1,
            end: 2,
        })),
    ];
    let origin = |origin| E::Origin {
        origin,
        affinity: t::Affinity::After,
    };
    let control = |control| E::Control {
        control,
        affinity: t::Affinity::After,
    };
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::Connection(0),
        E::EnterScope(1),
        control(0),
    ];
    if matches!(case, Case::EmptyBefore) {
        scopes.push(inline(None));
        events.extend([E::EnterScope(3), E::ExitScope(3)]);
    }
    events.extend([
        E::EnterScope(2),
        control(1),
        origin(1),
        control(2),
        E::ExitScope(2),
        control(3),
        E::ExitScope(1),
        E::Connection(1),
        origin(2),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ]);
    t::ContextualSourceInput {
        source,
        segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
        envelope: t::SourceProcessingEnvelope::Complete,
        origins: ["P", "A", "Q"]
            .into_iter()
            .enumerate()
            .map(|(i, raw)| t::ParagraphOrigin {
                text_node: None,
                source: t::Source::identified(raw, t::Id::from_u64(658001 + i as u64), 21),
                local: t::Range::new(0, 1),
                aggregate: t::Range::new(i, i + 1),
                kind: t::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: (0..2)
            .map(|i| t::SourceProcessingConnection {
                before_origin: i,
                after_origin: i + 1,
                kind: t::ParagraphConnectionKind::Continuous,
            })
            .collect(),
        controls: (0..4)
            .map(|i| t::ParagraphControl {
                at: if i < 2 { 1 } else { 2 },
                kind: if i < 2 {
                    t::ParagraphControlKind::InlineBoundaryStart
                } else {
                    t::ParagraphControlKind::InlineBoundaryEnd
                },
            })
            .collect(),
        boundary_policies: (0..2)
            .map(|i| t::ParagraphBoundaryPolicy {
                definition: t::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: t::Wrap::None,
                following_wrap: t::Wrap::None,
                ancestor_wrap: t::Wrap::None,
            })
            .collect(),
        computed_line_boundaries: vec![],
        replacements: vec![],
        scopes,
        paragraphs: vec![t::SourceProcessingParagraph {
            base_direction: t::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: t::ParagraphStrut::DefaultFont,
        }],
        events,
    }
}

fn qualify_empty_source() {
    // A separate producer control authenticates the SAME literal input factory.
    // This capture is discarded here, never used as Layout input or a receipt;
    // it is neither an adapter processing pass nor a single-call counter.
    let processed = t::process_contextual_source(source(Case::EmptyBefore)).unwrap();
    assert_eq!(processed.source().text(), &t::Text::from("PAQ"));
    let outer = processed.captured_scope(1).unwrap();
    let inner = processed.captured_scope(2).unwrap();
    let empty = processed.captured_scope(3).unwrap();
    assert_eq!(empty.parent(), Some(outer.clone()));
    assert_eq!(inner.parent(), Some(outer.clone()));
    assert!(empty.exit_position() < inner.enter_position());
    assert!(empty.entry_control().is_none());
    assert!(empty.exit_control().is_none());
    for (scope, start, end) in [(outer, 0, 3), (inner, 1, 2)] {
        assert_eq!(scope.entry_control(), processed.captured_control(start));
        assert_eq!(scope.exit_control(), processed.captured_control(end));
    }
    for i in 0..4 {
        let control = processed.captured_control(i).unwrap();
        assert_eq!(
            control.kind(),
            &if i < 2 {
                t::ParagraphControlKind::InlineBoundaryStart
            } else {
                t::ParagraphControlKind::InlineBoundaryEnd
            }
        );
        assert_eq!(control.source_position().get(), if i < 2 { 1 } else { 2 });
    }
    assert!(processed.captured_control(4).is_none());
    assert!(processed.captured_computed_line_boundary(0).is_none());
}

fn run<S: l::LayoutScalar>(case: Case) {
    if matches!(case, Case::EmptyBefore) {
        qualify_empty_source();
    }
    let mut host = Host::<S>::new(case);
    let output = {
        let early = Early(&host);
        l::process_wrapping_source(&early, 0, source(case), host.occurrences()).unwrap()
    };
    let (processed, receipt) = output.into_parts();
    let processed = Arc::new(processed);
    assert_eq!(processed.source().text(), &t::Text::from("PAQ"));
    let s = processed.captured_scope(1).unwrap();
    let inner = processed.captured_scope(2).unwrap();
    assert_eq!(inner.parent(), Some(s.clone()));
    for (scope, start, end) in [(s.clone(), 0, 3), (inner.clone(), 1, 2)] {
        assert_eq!(scope.entry_control(), processed.captured_control(start));
        assert_eq!(scope.exit_control(), processed.captured_control(end));
    }
    if matches!(case, Case::EmptyBefore) {
        let empty = processed.captured_scope(3).unwrap();
        assert_eq!(empty.parent(), Some(s.clone()));
        assert!(empty.entry_control().is_none());
        assert!(empty.exit_control().is_none());
    }
    let (target, side, cut, flows, positions, control_rows) = match case {
        Case::Before => (
            s,
            t::SourceBoxSide::Before,
            1,
            [10.0, 34.0],
            [(0.0, 8.0), (8.0, 18.0), (24.0, 18.0)],
            [vec![], vec![0, 1, 2, 3]],
        ),
        Case::After => (
            s,
            t::SourceBoxSide::After,
            2,
            [34.0, 10.0],
            [(0.0, 8.0), (18.0, 8.0), (0.0, 18.0)],
            [vec![0, 1, 2, 3], vec![]],
        ),
        Case::EmptyBefore => (
            inner,
            t::SourceBoxSide::Before,
            1,
            [15.0, 29.0],
            [(0.0, 8.0), (3.0, 18.0), (19.0, 18.0)],
            [vec![0], vec![1, 2, 3]],
        ),
    };
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        cause.target(),
        t::CapturedComputedLineBoundaryTarget::Scope {
            scope: target,
            side
        }
    );
    assert_eq!(cause.source_position().get(), cut);
    assert!(processed.captured_computed_line_boundary(1).is_none());
    let analyzed =
        t::AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
            .unwrap();
    let missing = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::Wrap::None,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(missing).unwrap();
    // The first font system/prepared object is created only after source borrow.
    let mut fonts = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = fonts
        .prepare(&analyzed, t::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.control_views().count(), 4);
    assert_eq!(
        prepared
            .participants()
            .filter(|p| matches!(p, t::PreparedParticipant::ComputedLineBoundary(_)))
            .count(),
        1
    );
    let offers: Vec<_> = prepared
        .opportunity_views()
        .map(|o| (o.kind(), o.boundary().view().source_before().get()))
        .collect();
    assert_eq!(
        offers,
        [
            (t::PreparedBreakKind::Forced, cut),
            (t::PreparedBreakKind::ParagraphEnd, 3)
        ]
    );
    let forced = prepared
        .opportunity_views()
        .find(|o| o.kind() == t::PreparedBreakKind::Forced)
        .unwrap();
    assert_eq!(
        forced
            .computed_line_boundary()
            .unwrap()
            .view()
            .causes()
            .collect::<Vec<_>>(),
        [cause]
    );
    let binding = host.install(&processed, &prepared);
    let handles: Vec<_> = binding
        .controls
        .iter()
        .map(|c| c.occurrence.clone())
        .collect();
    let tree = Final(&host);
    let flow = l::LayoutFlowInput::try_new_with_edges(
        &tree,
        0,
        vec![binding],
        host.final_wrapping(&processed),
        &receipt,
    )
    .unwrap();
    let request = l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(100.0)),
        l::AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = l::compute_layout_with_flow(&flow, 0, request).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().id(), Some(t::Id::from_u64(658000)));
    assert_eq!(placed.source().revision(), 21);
    assert_eq!(placed.source().text(), &t::Text::from("PAQ"));
    assert_eq!(
        placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [t::Range::new(0, cut), t::Range::new(cut, 3)]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        [8.0, 18.0]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        [10.0, 10.0]
    );
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(S::from_f64(100.0), S::from_f64(20.0))
    );
    let mut lines = vec![None, None];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    let mut source_glyphs = vec![];
    for (i, line) in lines.into_iter().enumerate() {
        let line = line.unwrap();
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            for glyph in run.glyphs() {
                source_glyphs.push((glyph.range(), glyph.id(), glyph.advance()));
            }
        }
        assert_eq!(line.measurements().fit().advance(), flows[i]);
        let selected = line.selected_line();
        let request = selected.request();
        let mut actual: Vec<_> = request
            .controls
            .iter()
            .map(|allocation| {
                let ordinal = handles
                    .iter()
                    .position(|h| *h == allocation.occurrence)
                    .unwrap();
                assert_eq!(
                    allocation.strut,
                    Some(t::ControlStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                        alignment: t::LineBoxAlignment::Baseline
                    })
                );
                assert_eq!(allocation.boundary.unwrap().advance(), ADVANCES[ordinal]);
                ordinal
            })
            .collect();
        // Allocation list order is not box custody. Preserve exact identities
        // and duplicate counts while comparing each line's selected inventory.
        actual.sort_unstable();
        assert_eq!(actual, control_rows[i]);
        assert_eq!(
            request.line_box_struts,
            if matches!(case, Case::EmptyBefore) && i == 0 {
                vec![t::LineBoxStrut {
                    metrics: t::LineStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                    },
                    alignment: t::LineBoxAlignment::Baseline,
                }]
            } else {
                vec![]
            }
        );
    }
    assert_eq!(
        source_glyphs,
        [
            (t::Range::new(0, 1), 50, 10.0),
            (t::Range::new(1, 2), 35, 10.0),
            (t::Range::new(2, 3), 51, 10.0),
        ]
    );
    let mut glyphs = vec![];
    for run in placed.run_views() {
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
    assert_eq!(
        glyphs,
        [
            (50, 10.0, positions[0].0, positions[0].1),
            (35, 10.0, positions[1].0, positions[1].1),
            (51, 10.0, positions[2].0, positions[2].1)
        ]
    );
}

#[test]
fn first_child_before_force_propagates_to_outer_scope_with_all_opening_bp_receiving() {
    run::<f32>(Case::Before);
    run::<f64>(Case::Before);
}
#[test]
fn last_child_after_force_propagates_to_outer_scope_with_all_closing_bp_outgoing() {
    run::<f32>(Case::After);
    run::<f64>(Case::After);
}
#[test]
fn genuine_empty_first_child_stops_before_propagation_and_retains_outer_start_outgoing() {
    run::<f32>(Case::EmptyBefore);
    run::<f64>(Case::EmptyBefore);
}

mod late_canonical_order;
