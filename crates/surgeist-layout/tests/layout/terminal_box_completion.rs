//! Real terminal Forced -> Paragraph completion retains its original prefix pin.
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
    children: [u32; 2],
}
impl<S: l::LayoutScalar> l::Traverse for Tree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, n: u32) -> Self::Children<'_> {
        let children: &[u32] = if n == 0 { &self.children } else { &[] };
        children.iter().copied()
    }
    fn child_count(&self, n: u32) -> usize {
        if n == 0 { 2 } else { 0 }
    }
    fn child(&self, n: u32, i: usize) -> u32 {
        assert_eq!(n, 0);
        self.children[i]
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

fn metric(baseline: f64, extent: f64) -> t::LineBoxStrut {
    t::LineBoxStrut {
        scope: None,
        metrics: t::LineStrut {
            baseline,
            line_extent: extent,
        },
        alignment: t::LineBoxAlignment::Baseline,
    }
}

fn expected_metric(scope: &t::CapturedScope, baseline: f64, extent: f64) -> t::LineBoxStrut {
    let mut strut = metric(baseline, extent);
    strut.scope = Some(scope.clone());
    strut
}

struct Fixture<S: l::LayoutScalar> {
    processed: Arc<t::ProcessedSource>,
    prepared: t::PreparedText,
    binding: l::TextParagraphBinding<u32>,
    scopes: [t::CapturedScope; 2],
    tree: Tree<S>,
}
impl<S: l::LayoutScalar> Fixture<S> {
    fn new(direction: l::Direction, forced: bool) -> Self {
        use t::SourceProcessingEvent as E;
        let base = if direction == l::Direction::Ltr {
            t::ParagraphBaseDirection::LeftToRight
        } else {
            t::ParagraphBaseDirection::RightToLeft
        };
        let style = t::Style {
            font: t::Font::new().family("Ahem"),
            size: 10.0,
            line_height: t::LineHeight::Absolute(10.0),
            word_break: t::WordBreak::BreakAll,
            hyphens: t::Hyphens::None,
            wrap: t::Wrap::Word,
            ..t::Style::default()
        };
        let mut source = t::Source::identified("AB", t::Id::from_u64(652900), 9);
        source.span(t::Range::new(0, 2), style.clone());
        let inline = || t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: None,
            boundaries: None,
        };
        let processed = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source,
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: ["A", "B"]
                    .into_iter()
                    .enumerate()
                    .map(|(i, raw)| t::ParagraphOrigin {
                        text_node: None,
                        source: t::Source::identified(raw, t::Id::from_u64(652901 + i as u64), 9),
                        local: t::Range::new(0, 1),
                        aggregate: t::Range::new(i, i + 1),
                        kind: t::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                connections: vec![t::SourceProcessingConnection {
                    before_origin: 0,
                    after_origin: 1,
                    kind: t::ParagraphConnectionKind::Continuous,
                }],
                boundary_policies: vec![t::ParagraphBoundaryPolicy {
                    definition: t::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: t::Wrap::Word,
                    following_wrap: t::Wrap::Word,
                    ancestor_wrap: t::Wrap::Word,
                }],
                controls: vec![],
                replacements: vec![],
                computed_line_boundaries: if forced {
                    vec![t::ComputedLineBoundaryInput::Scope {
                        scope: 2,
                        side: t::SourceBoxSide::Before,
                    }]
                } else {
                    vec![]
                },
                scopes: vec![
                    t::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: None,
                        topology: t::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: style,
                        used_direction: base,
                    },
                    inline(),
                    inline(),
                ],
                paragraphs: vec![t::SourceProcessingParagraph {
                    base_direction: base,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                }],
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::Origin {
                        origin: 0,
                        affinity: t::Affinity::After,
                    },
                    E::Connection(0),
                    E::Origin {
                        origin: 1,
                        affinity: t::Affinity::After,
                    },
                    E::EnterScope(1),
                    E::EnterScope(2),
                    E::ExitScope(2),
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let scopes = [
            processed.captured_scope(1).unwrap(),
            processed.captured_scope(2).unwrap(),
        ];
        assert_eq!(scopes[0].parent(), processed.captured_scope(0));
        assert_eq!(scopes[1].parent(), Some(scopes[0].clone()));
        let paragraph = processed.captured_paragraph(0).unwrap();
        for scope in &scopes {
            assert!(scope.exit_position() < paragraph.exit_position());
            assert!(scope.entry_control().is_none());
            assert!(scope.exit_control().is_none());
        }
        let analyzed = t::AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            t::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let policies = analyzed
            .wrapping_relations()
            .filter(|r| r.ancestor_wrap().is_none())
            .map(|r| t::WrappingPolicy {
                relation: r.handle(),
                ancestor_wrap: t::Wrap::Word,
            })
            .collect();
        let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
        let mut fonts = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = fonts
            .prepare(&analyzed, t::PreparationOptions::default())
            .unwrap();
        assert_eq!(prepared.terminal_edge(), t::ParagraphTerminalEdge::BlockEnd);
        assert_eq!(prepared.control_views().count(), 0);
        assert_eq!(prepared.atomic_views().count(), 0);
        assert_eq!(prepared.participants().count(), if forced { 3 } else { 2 });
        let mut inputs = vec![l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            direction,
            size: l::Size::new(
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(30.0)).unwrap()),
                l::PreferredSizeOf::AUTO,
            ),
            ..l::NodeInputOf::default()
        })];
        let origins = prepared
            .origin_views()
            .enumerate()
            .map(|(i, origin)| {
                assert_eq!(
                    origin.source().id(),
                    Some(t::Id::from_u64(652901 + i as u64))
                );
                assert_eq!(origin.source().revision(), 9);
                assert_eq!(origin.aggregate_range(), t::Range::new(i, i + 1));
                assert_eq!(origin.local_range(), t::Range::new(0, 1));
                inputs.push(l::LayoutInputOf::inline_text(l::TextParticipation::new(
                    origin.handle(),
                )));
                l::TextOriginBinding {
                    origin: origin.handle(),
                    node: i as u32 + 1,
                }
            })
            .collect();
        let binding = l::TextParagraphBinding {
            prepared: prepared.clone(),
            container: 0,
            nodes: vec![1, 2],
            origins,
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        };
        Self {
            processed,
            prepared,
            binding,
            scopes,
            tree: Tree {
                inputs,
                non_box: l::NodeInputOf::non_box(),
                children: [1, 2],
            },
        }
    }

    fn inventory(&self) -> l::WrappingInput<u32> {
        let child = |box_| l::GeneratedChildBoxInput {
            box_,
            participation: l::GeneratedBoxParticipation::InFlow,
        };
        let row = |parent, children| l::InlineChildOrderInput {
            parent,
            children,
            prefix: l::ChildOrderCompleteness::Complete,
            suffix: l::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        let s = l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(self.scopes[0].clone()));
        let inner = l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(self.scopes[1].clone()));
        l::WrappingInput {
            scopes: self
                .scopes
                .iter()
                .zip([metric(16.0, 20.0), metric(12.0, 30.0)])
                .map(|(scope, strut)| l::WrappingScopeInput::Captured {
                    scope: scope.clone(),
                    wrap_inside: l::WrapInside::Auto,
                    edges: l::WrapEdges::AUTO,
                    metric_contribution: Some(l::InlineBoxMetricContribution::Strut(strut)),
                })
                .collect(),
            child_order: vec![
                row(
                    l::GeneratedBoxRef::Node(0),
                    vec![
                        child(l::GeneratedBoxRef::Node(1)),
                        child(l::GeneratedBoxRef::Node(2)),
                        child(s.clone()),
                    ],
                ),
                row(s, vec![child(inner.clone())]),
                row(inner, vec![]),
            ],
        }
    }

    fn offers(&self, forced: bool) -> Vec<t::PreparedOpportunity> {
        let offers = self
            .prepared
            .opportunity_views()
            .map(|o| o.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            offers
                .iter()
                .map(|o| {
                    let v = o.view();
                    let b = v.boundary();
                    (
                        v.kind(),
                        b.view().participant_offset(),
                        b.view().source_before().get(),
                        b.view().source_after().get(),
                    )
                })
                .collect::<Vec<_>>(),
            if forced {
                vec![
                    (t::PreparedBreakKind::Allowed, 1, 1, 1),
                    (t::PreparedBreakKind::Forced, 3, 2, 2),
                    (t::PreparedBreakKind::ParagraphEnd, 3, 2, 2),
                ]
            } else {
                vec![
                    (t::PreparedBreakKind::Allowed, 1, 1, 1),
                    (t::PreparedBreakKind::ParagraphEnd, 2, 2, 2),
                ]
            }
        );
        offers
    }

    fn pinned(&self) -> Vec<t::PreparedBoundaryIncidence> {
        let offers = self.offers(true);
        let forced = offers[1].view().boundary();
        assert_eq!(offers[2].view().boundary(), forced);
        let computed = offers[1].view().computed_line_boundary().unwrap();
        assert_eq!(computed.view().source_position().get(), 2);
        assert_eq!(
            computed
                .view()
                .causes()
                .map(|c| c.target())
                .collect::<Vec<_>>(),
            [t::CapturedComputedLineBoundaryTarget::Scope {
                scope: self.scopes[1].clone(),
                side: t::SourceBoxSide::Before
            }]
        );
        assert!(offers[2].view().computed_line_boundary().is_none());
        assert_eq!(
            self.processed
                .captured_computed_line_boundary(0)
                .unwrap()
                .target(),
            t::CapturedComputedLineBoundaryTarget::Scope {
                scope: self.scopes[1].clone(),
                side: t::SourceBoxSide::Before
            }
        );
        let incidences = self.prepared.boundary_incidences(&forced).unwrap();
        assert_eq!(incidences.len(), 1);
        assert_eq!(incidences[0].boundary(), forced);
        for (scope, expected) in self.scopes.iter().zip([
            t::CapturedScopeIncidence::Inside,
            t::CapturedScopeIncidence::AtBefore,
        ]) {
            assert_eq!(
                self.prepared
                    .incidence_scope_relation(&incidences[0], scope)
                    .unwrap(),
                expected
            );
        }
        assert_eq!(
            self.prepared
                .boundary_incidences(&offers[2].view().boundary())
                .unwrap(),
            incidences
        );
        incidences
    }

    fn compute(&self) -> l::CompletedLayoutBatchOf<u32, S> {
        let flow = l::LayoutFlowInput::try_new(
            &self.tree,
            0,
            vec![self.binding.clone()],
            self.inventory(),
            Vec::new(),
        )
        .unwrap();
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(30.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request).expect(
            "real terminal ParagraphEnd must complete boxes without changing the earlier forced prefix pin",
        )
    }
}

fn assert_struts(actual: &[t::LineBoxStrut], expected: &[t::LineBoxStrut]) {
    assert_eq!(actual.len(), expected.len());
    for value in expected {
        assert_eq!(actual.iter().filter(|row| *row == value).count(), 1);
    }
}

fn check<S: l::LayoutScalar>(direction: l::Direction, forced: bool) {
    let fixture = Fixture::<S>::new(direction, forced);
    let before = if forced {
        fixture.pinned()
    } else {
        fixture.offers(false);
        vec![]
    };
    let prepared = fixture.prepared.clone();
    let batch = fixture.compute();
    assert_eq!(batch.placed_text().len(), 1);
    let entry = &batch.placed_text()[0];
    assert_eq!(entry.container(), 0);
    assert_eq!(entry.nodes(), [1, 2]);
    let placed = entry.layout();
    assert_eq!(
        placed.analyzed_source().identity(),
        prepared.analyzed_source().identity()
    );
    assert_eq!(placed.source().id(), Some(t::Id::from_u64(652900)));
    assert_eq!(placed.source().revision(), 9);
    assert_eq!(placed.source().text().code_units(), [65, 66]);
    let lines = placed.lines();
    assert_eq!(
        lines.iter().map(|l| l.range()).collect::<Vec<_>>(),
        vec![t::Range::new(0, 2)]
    );
    assert_eq!(
        lines
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        if forced { vec![20.0] } else { vec![34.0] }
    );
    assert_eq!(
        lines.iter().map(|l| l.baseline()).collect::<Vec<_>>(),
        vec![16.0]
    );
    assert_eq!(
        lines
            .iter()
            .map(|line| line.bounds().size.width)
            .collect::<Vec<_>>(),
        // These bounds are the supplied 30-unit line frames, not glyph
        // advances. The actual body glyph advances and empty receiver are
        // checked independently below.
        vec![30.0]
    );
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(
            S::from_f64(30.0),
            S::from_f64(if forced { 20.0 } else { 34.0 })
        )
    );
    let body_entry = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().line_index() == 0)
        .unwrap();
    let body = body_entry.fragment().finalized_fragment().finalized_line();
    let selected = body.selected_line();
    let request = selected.request();
    let offers = fixture.offers(forced);
    assert_eq!(
        request.context.end,
        if forced {
            t::LineEnd::Forced
        } else {
            t::LineEnd::Paragraph
        }
    );
    assert_eq!(
        request.termination,
        t::PreparedTermination::Ordinary(offers[1].clone())
    );
    assert_eq!(request.end, offers[1].view().boundary());
    assert_eq!(
        request.participants,
        prepared.participants().collect::<Vec<_>>()
    );
    assert!(request.controls.is_empty());
    assert!(request.atomics.is_empty());
    let expected_struts = if forced {
        vec![expected_metric(&fixture.scopes[0], 16.0, 20.0)]
    } else {
        vec![
            expected_metric(&fixture.scopes[0], 16.0, 20.0),
            expected_metric(&fixture.scopes[1], 12.0, 30.0),
        ]
    };
    assert_struts(&request.line_box_struts, &expected_struts);
    let source_glyphs = body
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        source_glyphs,
        [
            (t::Range::new(0, 1), 35, 10.0),
            (t::Range::new(1, 2), 36, 10.0)
        ]
    );
    let mut glyphs = vec![];
    for run in placed.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            glyphs.push((
                run.line(),
                glyph.id(),
                glyph.advance(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                run.origin().y,
            ));
        }
    }
    let x = if direction == l::Direction::Ltr {
        0.0
    } else {
        10.0
    };
    assert_eq!(
        glyphs,
        [(0, 35, 10.0, x, 16.0), (0, 36, 10.0, x + 10.0, 16.0)]
    );
    // Nonformatted completion has no published source fragment. Its actual
    // retained receipt is checked separately without adding a fake origin.
    assert!(
        batch
            .final_inline_fragments()
            .iter()
            .all(|entry| entry.fragment().line_index() == 0)
    );
    assert_eq!(prepared.participants().count(), if forced { 3 } else { 2 });
    assert_eq!(prepared.control_views().count(), 0);
    assert_eq!(prepared.atomic_views().count(), 0);
    if forced {
        assert_retained_phantom_receipt(&fixture, &batch, 30.0, 34.0);
        assert_eq!(fixture.pinned(), before);
    } else {
        assert_eq!(placed.nonformatted_lines().count(), 0);
        fixture.offers(false);
    }
}

#[test]
fn genuine_terminal_force_completes_boxes_without_publishing_phantom_receiver() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        check::<f32>(direction, true);
        check::<f64>(direction, true);
    }
}

#[test]
fn without_terminal_force_trailing_boxes_complete_on_the_existing_body_line() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        check::<f32>(direction, false);
        check::<f64>(direction, false);
    }
}

impl<S: l::LayoutScalar> Fixture<S> {
    fn try_compute_inventory(
        &self,
        inventory: l::WrappingInput<u32>,
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
        let flow = l::LayoutFlowInput::try_new(
            &self.tree,
            0,
            vec![self.binding.clone()],
            inventory,
            Vec::new(),
        )?;
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(30.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request)
    }
}

fn replace_t_metric(
    inventory: &mut l::WrappingInput<u32>,
    target: &t::CapturedScope,
    contribution: Option<l::InlineBoxMetricContribution>,
) {
    let mut replaced = 0;
    for row in &mut inventory.scopes {
        match row {
            l::WrappingScopeInput::Captured {
                scope,
                metric_contribution,
                ..
            } if scope == target => {
                *metric_contribution = contribution.clone();
                replaced += 1;
            }
            _ => {}
        }
    }
    assert_eq!(
        replaced, 1,
        "fixture targets the actual unique T occurrence"
    );
}

fn missing_trailing_metric<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, true);
    let pin = fixture.pinned();
    let mut inventory = fixture.inventory();
    replace_t_metric(&mut inventory, &fixture.scopes[1], None);
    let error = fixture
        .try_compute_inventory(inventory)
        .expect_err("the real trailing T cannot borrow S's or the paragraph font's metric witness");
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
                l::WrappingScopeError::MissingBoxMetrics
            ))
        ),
        "{error:?}"
    );
    assert_eq!(fixture.pinned(), pin);
    assert_eq!(fixture.prepared.participants().count(), 3);
    assert_eq!(fixture.prepared.control_views().count(), 0);
}

#[test]
fn trailing_box_without_its_actual_metric_witness_refuses_precisely() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        missing_trailing_metric::<f32>(direction);
        missing_trailing_metric::<f64>(direction);
    }
}

fn assert_phantom_terminal_result<S: l::LayoutScalar>(
    fixture: &Fixture<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    direction: l::Direction,
) {
    assert_eq!(batch.placed_text().len(), 1);
    let entry = &batch.placed_text()[0];
    assert_eq!(entry.container(), 0);
    assert_eq!(entry.nodes(), [1, 2]);
    let placed = entry.layout();
    assert_eq!(
        placed.analyzed_source().identity(),
        fixture.prepared.analyzed_source().identity()
    );
    assert_eq!(placed.source().id(), Some(t::Id::from_u64(652900)));
    assert_eq!(placed.source().revision(), 9);
    assert_eq!(placed.source().text().code_units(), [65, 66]);
    // A real Forced line is formatted; its final strut-only Paragraph receiver
    // retains completion authority but is not an ordinary published line.
    let lines = placed.lines();
    assert_eq!(
        lines.len(),
        1,
        "the Paragraph-ended empty strut receiver is phantom"
    );
    assert_eq!(lines[0].range(), t::Range::new(0, 2));
    assert_eq!(lines[0].baseline(), 16.0);
    assert_eq!(lines[0].bounds().size, t::Size::new(30.0, 20.0));
    assert_eq!(placed.metrics().line_count(), 1);
    assert_eq!(placed.metrics().first_baseline(), Some(16.0));
    assert_eq!(placed.metrics().last_baseline(), Some(16.0));
    assert_eq!(placed.metrics().size().height, 20.0);
    assert_eq!(placed.bounds().size.height, 20.0);
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(S::from_f64(30.0), S::from_f64(20.0))
    );
    assert_eq!(root.content_size.height, S::from_f64(20.0));
    let body_entry = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().line_index() == 0)
        .unwrap();
    let body = body_entry.fragment().finalized_fragment().finalized_line();
    let selected = body.selected_line();
    let request = selected.request();
    let offers = fixture.offers(true);
    assert_eq!(request.context.end, t::LineEnd::Forced);
    assert_eq!(
        request.termination,
        t::PreparedTermination::Ordinary(offers[1].clone())
    );
    assert_eq!(request.end, offers[1].view().boundary());
    assert_eq!(
        request.participants,
        fixture.prepared.participants().collect::<Vec<_>>()
    );
    assert_struts(
        &request.line_box_struts,
        &[expected_metric(&fixture.scopes[0], 16.0, 20.0)],
    );
    assert!(request.controls.is_empty());
    assert!(request.atomics.is_empty());
    assert_eq!(body.measurements().fit().advance(), 20.0);
    assert_eq!(offers[2].view().kind(), t::PreparedBreakKind::ParagraphEnd);
    assert_eq!(offers[2].view().boundary(), request.end);
    assert!(offers[2].view().computed_line_boundary().is_none());
    // Source-aware glyph facts come from the actual finalized body, while
    // committed physical glyph placements intentionally carry no source range.
    let sourced = body
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sourced,
        [
            (t::Range::new(0, 1), 35, 10.0),
            (t::Range::new(1, 2), 36, 10.0)
        ]
    );
    let x = if direction == l::Direction::Ltr {
        0.0
    } else {
        10.0
    };
    let mut posed = vec![];
    for run in placed.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            posed.push((
                run.line(),
                glyph.id(),
                glyph.advance(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                run.origin().y,
            ));
        }
    }
    assert_eq!(
        posed,
        [(0, 35, 10.0, x, 16.0), (0, 36, 10.0, x + 10.0, 16.0)]
    );
    assert!(
        batch
            .final_inline_fragments()
            .iter()
            .all(|entry| entry.fragment().line_index() == 0)
    );
    assert_eq!(fixture.prepared.participants().count(), 3);
    assert_eq!(fixture.prepared.origin_views().count(), 2);
    assert_eq!(fixture.prepared.control_views().count(), 0);
    assert_eq!(fixture.prepared.atomic_views().count(), 0);
}

fn assert_retained_phantom_receipt<S: l::LayoutScalar>(
    fixture: &Fixture<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    tail_extent: f64,
    nominal_height: f64,
) {
    let completions = batch.placed_text()[0]
        .layout()
        .nonformatted_lines()
        .collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let placement = completions[0];
    assert!(placement.origin.x.is_finite());
    assert_eq!(placement.origin.y, 20.0);
    assert_eq!(placement.orientation, t::LineOrientation::Horizontal);
    assert!(placement.atomics.is_empty());
    let finalized = placement.line.finalized_line();
    let selected = finalized.selected_line();
    let request = selected.request();
    let offers = fixture.offers(true);
    assert_eq!(request.context.end, t::LineEnd::Paragraph);
    assert_eq!(
        request.termination,
        t::PreparedTermination::Ordinary(offers[2].clone())
    );
    assert_eq!(request.start, offers[1].view().boundary());
    assert_eq!(request.end, offers[2].view().boundary());
    assert_eq!(request.start.prepared_text(), fixture.prepared);
    assert!(request.participants.is_empty());
    assert!(request.controls.is_empty());
    assert!(request.atomics.is_empty());
    assert_struts(
        &request.line_box_struts,
        &[
            expected_metric(&fixture.scopes[0], 16.0, 20.0),
            expected_metric(&fixture.scopes[1], 12.0, tail_extent),
        ],
    );
    // Actual numerical receipts remain real: phantom disposition is not a
    // zero-allocation substitution or permission to discard T's witness.
    assert_eq!(finalized.feedback().line_height, nominal_height);
    assert_eq!(finalized.feedback().baseline, 16.0);
    assert_eq!(finalized.measurements().fit().advance(), 0.0);
    assert!(!finalized.has_in_flow_content());
}

fn phantom_terminal<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, true);
    let pin = fixture.pinned();
    let batch = fixture.compute();
    assert_phantom_terminal_result(&fixture, &batch, direction);
    assert_retained_phantom_receipt(&fixture, &batch, 30.0, 34.0);
    assert_eq!(fixture.pinned(), pin);
}

fn phantom_t_metric<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, true);
    let pin = fixture.pinned();
    let old = fixture.compute();
    assert_phantom_terminal_result(&fixture, &old, direction);
    assert_retained_phantom_receipt(&fixture, &old, 30.0, 34.0);
    let mut inventory = fixture.inventory();
    replace_t_metric(
        &mut inventory,
        &fixture.scopes[1],
        Some(l::InlineBoxMetricContribution::Strut(metric(12.0, 40.0))),
    );
    let changed = fixture
        .try_compute_inventory(inventory)
        .expect("real phantom completion still finalizes the supplied actual T metric");
    // T's actual numerical receiver metrics change; no text, inline-axis BP,
    // atomic or Forced ending appears on that receiver to make it formatted.
    assert_phantom_terminal_result(&fixture, &changed, direction);
    assert_retained_phantom_receipt(&fixture, &changed, 40.0, 44.0);
    assert_eq!(fixture.pinned(), pin);
    // The earlier successful immutable publication also remains unchanged.
    assert_phantom_terminal_result(&fixture, &old, direction);
    assert_retained_phantom_receipt(&fixture, &old, 30.0, 34.0);
    assert_eq!(fixture.pinned(), pin);
}

#[test]
fn terminal_strut_only_paragraph_completion_has_no_published_line_or_extent() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        phantom_terminal::<f32>(direction);
        phantom_terminal::<f64>(direction);
    }
}

#[test]
fn larger_phantom_tail_metrics_do_not_create_extent_or_mutate_prior_output() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        phantom_t_metric::<f32>(direction);
        phantom_t_metric::<f64>(direction);
    }
}

#[test]
fn phantom_completion_still_requires_the_actual_trailing_metric_witness() {
    // Existing genuine metric-demand control: None is not zero or a font proxy.
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        missing_trailing_metric::<f32>(direction);
        missing_trailing_metric::<f64>(direction);
    }
}

fn omitted_paragraph_receipt<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, true);
    let pin = fixture.pinned();
    let batch = fixture.compute();
    let entry = &batch.placed_text()[0];
    let placed = entry.layout();
    let actual_body = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let selected = actual_body.selected_line();
    let request = selected.request();
    let offers = fixture.offers(true);
    assert_eq!(request.context.end, t::LineEnd::Forced);
    assert_eq!(
        request.termination,
        t::PreparedTermination::Ordinary(offers[1].clone())
    );
    assert_eq!(request.end, offers[2].view().boundary());
    assert_eq!(offers[2].view().kind(), t::PreparedBreakKind::ParagraphEnd);
    let first = placed.lines()[0];
    let anchor = t::LineAnchor {
        inline_origin: first.bounds().origin.x,
        baseline: first.baseline(),
    };
    assert!(anchor.inline_origin.is_finite() && anchor.baseline.is_finite());
    // Exact terminal-boundary equality is insufficient: the actual selected
    // ending remains Forced, while WholeParagraph needs its real Paragraph receipt.
    let error = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: fixture.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line: actual_body,
            anchor,
            orientation: t::LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .expect_err("dropping the genuine ParagraphEnd receipt cannot complete WholeParagraph");
    assert!(
        matches!(
            error.detail(),
            Some(t::ErrorDetail::InvalidTextProtocol {
                reason: t::ProtocolError::IncompleteLineCoverage { line: 0 },
            })
        ),
        "{error:?}"
    );
    assert_eq!(fixture.pinned(), pin);
}

#[test]
fn whole_paragraph_rejects_only_forced_receipt_even_at_its_final_boundary() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        omitted_paragraph_receipt::<f32>(direction);
        omitted_paragraph_receipt::<f64>(direction);
    }
}
