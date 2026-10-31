//! Genuine node-free computed Force scheduling and retained phantom completion.
//! Public capture, preparation, association and actual layout with no host children.
use std::{convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Root<S: l::LayoutScalar> {
    input: l::NodeInputOf<S>,
    children: [u32; 0],
}
impl<S: l::LayoutScalar> l::Traverse for Root<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        assert_eq!(node, 0);
        self.children.iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        assert_eq!(node, 0);
        0
    }
    fn child(&self, _node: u32, _index: usize) -> u32 {
        panic!("the genuine root has no host children")
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for Root<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        assert_eq!(node, 0);
        &self.input
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        assert_eq!(node, 0);
        l::LayoutInputViewOf::Box(&self.input)
    }
}

struct Fixture<S: l::LayoutScalar> {
    capture: Arc<t::ProcessedSource>,
    prepared: t::PreparedText,
    scopes: [t::CapturedScope; 2],
    tree: Root<S>,
}
impl<S: l::LayoutScalar> Fixture<S> {
    fn new(direction: l::Direction, force: bool) -> Self {
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
            wrap: t::Wrap::None,
            hyphens: t::Hyphens::None,
            ..t::Style::default()
        };
        let inline = || t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: None,
            boundaries: None,
        };
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source: t::Source::identified("", t::Id::from_u64(658300), 51),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: vec![],
                controls: vec![],
                connections: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                computed_line_boundaries: if force {
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
            capture.captured_scope(1).unwrap(),
            capture.captured_scope(2).unwrap(),
        ];
        assert_eq!(scopes[0].parent(), capture.captured_scope(0));
        assert_eq!(scopes[1].parent(), Some(scopes[0].clone()));
        let paragraph = capture.captured_paragraph(0).unwrap();
        for scope in &scopes {
            assert!(scope.exit_position() < paragraph.exit_position());
            assert!(scope.entry_control().is_none());
            assert!(scope.exit_control().is_none());
        }
        assert!(capture.source().text().code_units().is_empty());
        assert!(capture.processed_text().code_units().is_empty());
        assert!(capture.captured_origin(0).is_none());
        assert!(capture.captured_control(0).is_none());
        assert!(capture.captured_atomic(0).is_none());
        let analyzed = t::AnalyzedSource::from_processed_paragraph(
            capture.paragraphs().next().unwrap(),
            t::AnalysisTypography::HorizontalComposition,
        )
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
        let mut fonts = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = fonts
            .prepare(&analyzed, t::PreparationOptions::default())
            .unwrap();
        assert_eq!(
            prepared
                .captured_owner()
                .unwrap()
                .members()
                .collect::<Vec<_>>(),
            vec![capture.captured_paragraph(0).unwrap()]
        );
        assert_eq!(prepared.terminal_edge(), t::ParagraphTerminalEdge::BlockEnd);
        assert_eq!(prepared.origin_views().count(), 0);
        assert_eq!(prepared.control_views().count(), 0);
        assert_eq!(prepared.atomic_views().count(), 0);
        assert_eq!(prepared.virtual_separators().count(), 0);
        assert_eq!(prepared.scopes().count(), 0);
        Self {
            capture,
            prepared,
            scopes,
            tree: Root {
                input: l::NodeInputOf {
                    display: l::Display::Block,
                    direction,
                    size: l::Size::new(
                        l::PreferredSizeOf::value(
                            l::LengthPercentageOf::px(S::from_f64(30.0)).unwrap(),
                        ),
                        l::PreferredSizeOf::AUTO,
                    ),
                    ..l::NodeInputOf::default()
                },
                children: [],
            },
        }
    }
    fn binding(&self) -> l::TextParagraphBinding<u32> {
        l::TextParagraphBinding {
            prepared: self.prepared.clone(),
            container: 0,
            nodes: vec![],
            origins: vec![],
            controls: vec![],
            atomics: vec![],
            line_strut: None,
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
                .zip([(16.0, 20.0), (12.0, 30.0)])
                .map(
                    |(scope, (baseline, line_extent))| l::WrappingScopeInput::Captured {
                        scope: scope.clone(),
                        wrap_inside: l::WrapInside::Auto,
                        edges: l::WrapEdges::AUTO,
                        metric_contribution: Some(l::InlineBoxMetricContribution::Strut(
                            t::LineBoxStrut {
                                scope: None,
                                metrics: t::LineStrut {
                                    baseline,
                                    line_extent,
                                },
                                alignment: t::LineBoxAlignment::Baseline,
                            },
                        )),
                    },
                )
                .collect(),
            child_order: vec![
                row(l::GeneratedBoxRef::Node(0), vec![child(s.clone())]),
                row(s, vec![child(inner.clone())]),
                row(inner, vec![]),
            ],
        }
    }
    fn inventory_facts(&self, force: bool) -> Vec<t::PreparedBoundaryIncidence> {
        assert_eq!(
            self.prepared.analyzed_source().source().id(),
            Some(t::Id::from_u64(658300))
        );
        assert_eq!(self.prepared.analyzed_source().source().revision(), 51);
        assert!(
            self.prepared
                .analyzed_source()
                .source()
                .text()
                .code_units()
                .is_empty()
        );
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
            if force {
                vec![
                    (t::PreparedBreakKind::Forced, 1, 0, 0),
                    (t::PreparedBreakKind::ParagraphEnd, 1, 0, 0),
                ]
            } else {
                vec![(t::PreparedBreakKind::ParagraphEnd, 0, 0, 0)]
            }
        );
        if !force {
            assert_eq!(self.prepared.participants().count(), 0);
            assert!(self.capture.captured_computed_line_boundary(0).is_none());
            return vec![];
        }
        let participants = self.prepared.participants().collect::<Vec<_>>();
        assert_eq!(participants.len(), 1);
        let t::PreparedParticipant::ComputedLineBoundary(group) = &participants[0] else {
            panic!("only the genuine Text semantic anchor may participate")
        };
        let boundary = offers[0].view().boundary();
        assert_eq!(offers[1].view().boundary(), boundary);
        assert_eq!(
            offers[0].view().computed_line_boundary().as_ref(),
            Some(group)
        );
        assert!(offers[1].view().computed_line_boundary().is_none());
        let cause = self.capture.captured_computed_line_boundary(0).unwrap();
        assert_eq!(
            cause.target(),
            t::CapturedComputedLineBoundaryTarget::Scope {
                scope: self.scopes[1].clone(),
                side: t::SourceBoxSide::Before
            }
        );
        assert_eq!(group.view().causes().collect::<Vec<_>>(), [cause]);
        let incidences = self
            .prepared
            .boundary_incidences(&self.prepared.source_segments().next().unwrap(), &boundary)
            .unwrap();
        assert_eq!(incidences.len(), 1);
        assert_eq!(incidences[0].boundary(), boundary);
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
                .boundary_incidences(
                    &self.prepared.source_segments().next().unwrap(),
                    &offers[1].view().boundary()
                )
                .unwrap(),
            incidences
        );
        incidences
    }
}

fn metric(baseline: f64, line_extent: f64) -> t::LineBoxStrut {
    t::LineBoxStrut {
        scope: None,
        metrics: t::LineStrut {
            baseline,
            line_extent,
        },
        alignment: t::LineBoxAlignment::Baseline,
    }
}

fn expected_metric(scope: &t::CapturedScope, baseline: f64, line_extent: f64) -> t::LineBoxStrut {
    let mut strut = metric(baseline, line_extent);
    strut.scope = Some(scope.clone());
    strut
}

fn assert_struts(actual: &[t::LineBoxStrut], expected: &[t::LineBoxStrut]) {
    assert_eq!(actual.len(), expected.len());
    for wanted in expected {
        assert_eq!(actual.iter().filter(|item| *item == wanted).count(), 1);
    }
}

impl<S: l::LayoutScalar> Fixture<S> {
    fn try_compute(
        &self,
        inventory: l::WrappingInput<u32>,
        available: l::AvailableOf<S>,
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
        let flow = l::LayoutFlowInput::try_new(
            &self.tree,
            0,
            vec![self.binding()],
            inventory,
            Vec::new(),
        )?;
        assert_eq!(flow.paragraphs().len(), 1);
        assert_eq!(flow.paragraphs()[0].prepared, self.prepared);
        assert_eq!(
            flow.paragraphs()[0]
                .prepared
                .captured_owner()
                .unwrap()
                .members()
                .collect::<Vec<_>>(),
            vec![self.capture.captured_paragraph(0).unwrap()]
        );
        assert!(flow.paragraphs()[0].nodes.is_empty());
        let request =
            l::LayoutRootRequestOf::viewport(l::Size::new(available, l::AvailableOf::MAX_CONTENT))
                .unwrap();
        l::compute_layout_with_flow(&flow, 0, request)
    }
}

fn assert_semantic_output<S: l::LayoutScalar>(
    fixture: &Fixture<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(
        batch.placed_text().len(),
        1,
        "the genuine paragraph must actually execute"
    );
    let entry = &batch.placed_text()[0];
    assert_eq!(entry.container(), 0);
    assert!(entry.nodes().is_empty());
    let layout = entry.layout();
    assert_eq!(
        layout.analyzed_source().identity(),
        fixture.prepared.analyzed_source().identity()
    );
    assert_eq!(layout.source(), fixture.prepared.analyzed_source().source());
    assert_eq!(layout.source().id(), Some(t::Id::from_u64(658300)));
    assert_eq!(layout.source().revision(), 51);
    assert!(layout.source().text().code_units().is_empty());
    assert_eq!(
        layout.direction(),
        fixture.prepared.analyzed_source().direction()
    );
    // The actual Forced ending prevents phantom disposition despite no glyph.
    let lines = layout.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].range(), t::Range::new(0, 0));
    assert_eq!(lines[0].baseline(), 16.0);
    assert_eq!(lines[0].bounds().size, t::Size::new(30.0, 20.0));
    assert_eq!(layout.metrics().line_count(), 1);
    assert_eq!(layout.metrics().first_baseline(), Some(16.0));
    assert_eq!(layout.metrics().last_baseline(), Some(16.0));
    assert_eq!(layout.metrics().size().height, 20.0);
    assert_eq!(layout.bounds().size.height, 20.0);
    assert_eq!(
        layout
            .run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    assert!(batch.final_inline_fragments().is_empty());
    let root = batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(S::from_f64(30.0), S::from_f64(20.0))
    );

    // End completion uses its genuine Paragraph capability at the same pinned
    // cut. Real S/T metric allocations remain positive but create no second
    // formatted line or source fragment.
    let completions = layout.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let placement = completions[0];
    assert!(placement.origin.x.is_finite());
    assert_eq!(placement.origin.y, 20.0);
    assert_eq!(placement.orientation, t::LineOrientation::Horizontal);
    assert!(placement.atomics.is_empty());
    let end = placement.line.finalized_line();
    assert_eq!(end.prepared_text(), fixture.prepared);
    let selected = end.selected_line();
    let request = selected.request();
    let offers = fixture
        .prepared
        .opportunity_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    assert_eq!(request.context.end, t::LineEnd::Paragraph);
    assert_eq!(
        request.termination,
        t::PreparedTermination::Ordinary(offers[1].clone())
    );
    assert_eq!(request.start, offers[0].view().boundary());
    assert_eq!(request.end, offers[1].view().boundary());
    assert!(request.incoming.is_none());
    assert!(request.alternative.is_none());
    assert!(request.participants.is_empty());
    assert!(request.controls.is_empty());
    assert!(request.atomics.is_empty());
    assert_struts(
        &request.line_box_struts,
        &[
            expected_metric(&fixture.scopes[0], 16.0, 20.0),
            expected_metric(&fixture.scopes[1], 12.0, 30.0),
        ],
    );
    // max before=16, max after=max(20-16,30-12)=18: nominal34.
    assert_eq!(end.feedback().baseline, 16.0);
    assert_eq!(end.feedback().line_height, 34.0);
    assert_eq!(end.feedback().advance, 0.0);
    assert_eq!(end.measurements().fit().advance(), 0.0);
    assert!(!end.has_in_flow_content());
    assert_eq!(
        end.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    assert_eq!(end.fragments().count(), 0);

    // The retained receiver alone is not WholeParagraph. This uses the actual
    // output capability, without a fabricated first-line accessor or source node.
    let incomplete = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: fixture.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::NonFormatted((*placement).clone())],
    })
    .unwrap_err();
    assert!(matches!(
        incomplete.detail(),
        Some(t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::IncompleteLineCoverage { line: 0 }
        })
    ));
}

fn semantic_scheduling<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, true);
    let pin = fixture.inventory_facts(true);
    // The root's actual specified width is30 in every request. These vary the
    // public request mode, not the independently specified root width.
    for available in [
        l::AvailableOf::definite(S::from_f64(30.0)),
        l::AvailableOf::MIN_CONTENT,
        l::AvailableOf::MAX_CONTENT,
    ] {
        let batch = fixture.try_compute(fixture.inventory(), available)
            .expect("genuine node-free computed-Force paragraph must admit and execute in its unique empty-container gap");
        assert_semantic_output(&fixture, &batch);
        assert_eq!(fixture.inventory_facts(true), pin);
        let replay = fixture.try_compute(fixture.inventory(), available).unwrap();
        assert_semantic_output(&fixture, &replay);
        assert_eq!(
            batch.placed_text()[0].layout().key(),
            replay.placed_text()[0].layout().key()
        );
        // The earlier immutable publication is still intact after replay.
        assert_semantic_output(&fixture, &batch);
        assert_eq!(fixture.inventory_facts(true), pin);
    }
    let mut missing = fixture.inventory();
    match &mut missing.scopes[1] {
        l::WrappingScopeInput::Captured {
            scope,
            metric_contribution,
            ..
        } => {
            assert_eq!(&*scope, &fixture.scopes[1]);
            *metric_contribution = None;
        }
        _ => panic!("actual captured T row required"),
    }
    let error = fixture
        .try_compute(missing, l::AvailableOf::definite(S::from_f64(30.0)))
        .expect_err("phantom receiver must still demand the actual T metric witness");
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
                l::WrappingScopeError::MissingBoxMetrics
            ))
        ),
        "{error:?}"
    );
    assert_eq!(fixture.inventory_facts(true), pin);
}

fn neutral_metadata<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, false);
    fixture.inventory_facts(false);
    for available in [
        l::AvailableOf::definite(S::from_f64(30.0)),
        l::AvailableOf::MIN_CONTENT,
        l::AvailableOf::MAX_CONTENT,
    ] {
        let batch = fixture.try_compute(fixture.inventory(), available).expect(
            "strict zero-participation captured metadata retains its actual container association",
        );
        assert!(batch.placed_text().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        let root = batch
            .final_entries()
            .iter()
            .find(|row| row.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size, l::Size::new(S::from_f64(30.0), S::ZERO));
        fixture.inventory_facts(false);
    }
    // Actual positive S/T allocations do not turn AssociationOnly into a run.
}

#[test]
fn real_node_free_computed_paragraph_schedules_forced_line_and_phantom_completion() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        semantic_scheduling::<f32>(direction);
        semantic_scheduling::<f64>(direction);
    }
}

#[test]
fn neutral_node_free_metadata_does_not_publish_text_from_scope_struts() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        neutral_metadata::<f32>(direction);
        neutral_metadata::<f64>(direction);
    }
}

struct SharedGap<S: l::LayoutScalar> {
    capture: Arc<t::ProcessedSource>,
    prepared: Vec<t::PreparedText>,
    scopes: Vec<t::CapturedScope>,
    tree: Root<S>,
}

impl<S: l::LayoutScalar> SharedGap<S> {
    fn new(direction: l::Direction) -> Self {
        use t::SourceProcessingEvent as E;
        let base = if direction == l::Direction::Ltr {
            t::ParagraphBaseDirection::LeftToRight
        } else {
            t::ParagraphBaseDirection::RightToLeft
        };
        let inline = || t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: None,
            boundaries: None,
        };
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source: t::Source::identified("", t::Id::from_u64(658301), 52),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: vec![],
                controls: vec![],
                connections: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                computed_line_boundaries: vec![
                    t::ComputedLineBoundaryInput::Scope {
                        scope: 2,
                        side: t::SourceBoxSide::Before,
                    },
                    t::ComputedLineBoundaryInput::Scope {
                        scope: 4,
                        side: t::SourceBoxSide::Before,
                    },
                ],
                scopes: vec![
                    t::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: None,
                        topology: t::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: t::Style {
                            font: t::Font::new().family("Ahem"),
                            size: 10.0,
                            line_height: t::LineHeight::Absolute(10.0),
                            wrap: t::Wrap::None,
                            hyphens: t::Hyphens::None,
                            ..t::Style::default()
                        },
                        used_direction: base,
                    },
                    inline(),
                    inline(),
                    inline(),
                    inline(),
                ],
                paragraphs: (0..3)
                    .map(|_| t::SourceProcessingParagraph {
                        base_direction: base,
                        paragraph_strut: t::ParagraphStrut::DefaultFont,
                    })
                    .collect(),
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::EnterScope(1),
                    E::EnterScope(2),
                    E::ExitScope(2),
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::EnterParagraph(1),
                    E::ExitParagraph(1),
                    E::EnterParagraph(2),
                    E::EnterScope(3),
                    E::EnterScope(4),
                    E::ExitScope(4),
                    E::ExitScope(3),
                    E::ExitParagraph(2),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let scopes = (1..5)
            .map(|i| capture.captured_scope(i).unwrap())
            .collect::<Vec<_>>();
        for outer in [&scopes[0], &scopes[2]] {
            assert_eq!(outer.parent(), capture.captured_scope(0));
        }
        assert_eq!(scopes[1].parent(), Some(scopes[0].clone()));
        assert_eq!(scopes[3].parent(), Some(scopes[2].clone()));
        let mut fonts = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = capture
            .paragraphs()
            .map(|paragraph| {
                let analyzed = t::AnalyzedSource::from_processed_paragraph(
                    paragraph,
                    t::AnalysisTypography::HorizontalComposition,
                )
                .unwrap();
                let policies = analyzed
                    .wrapping_relations()
                    .filter(|row| row.ancestor_wrap().is_none())
                    .map(|row| t::WrappingPolicy {
                        relation: row.handle(),
                        ancestor_wrap: t::Wrap::None,
                    })
                    .collect();
                fonts
                    .prepare(
                        &analyzed.with_wrapping_policies(policies).unwrap(),
                        t::PreparationOptions::default(),
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(prepared.len(), 3);
        let fixture = Self {
            capture,
            prepared,
            scopes,
            tree: Root {
                input: l::NodeInputOf {
                    display: l::Display::Block,
                    direction,
                    size: l::Size::new(
                        l::PreferredSizeOf::value(
                            l::LengthPercentageOf::px(S::from_f64(30.0)).unwrap(),
                        ),
                        l::PreferredSizeOf::AUTO,
                    ),
                    ..l::NodeInputOf::default()
                },
                children: [],
            },
        };
        fixture.facts();
        fixture
    }

    fn binding(&self, paragraph: usize) -> l::TextParagraphBinding<u32> {
        l::TextParagraphBinding {
            prepared: self.prepared[paragraph].clone(),
            container: 0,
            nodes: vec![],
            origins: vec![],
            controls: vec![],
            atomics: vec![],
            line_strut: None,
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
        let boxes = self
            .scopes
            .iter()
            .map(|scope| l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(scope.clone())))
            .collect::<Vec<_>>();
        l::WrappingInput {
            scopes: self
                .scopes
                .iter()
                .zip([(16.0, 20.0), (12.0, 30.0), (8.0, 10.0), (6.0, 24.0)])
                .map(
                    |(scope, (baseline, extent))| l::WrappingScopeInput::Captured {
                        scope: scope.clone(),
                        wrap_inside: l::WrapInside::Auto,
                        edges: l::WrapEdges::AUTO,
                        metric_contribution: Some(l::InlineBoxMetricContribution::Strut(metric(
                            baseline, extent,
                        ))),
                    },
                )
                .collect(),
            child_order: vec![
                row(
                    l::GeneratedBoxRef::Node(0),
                    vec![child(boxes[0].clone()), child(boxes[2].clone())],
                ),
                row(boxes[0].clone(), vec![child(boxes[1].clone())]),
                row(boxes[1].clone(), vec![]),
                row(boxes[2].clone(), vec![child(boxes[3].clone())]),
                row(boxes[3].clone(), vec![]),
            ],
        }
    }

    fn facts(&self) -> Vec<Vec<t::PreparedBoundaryIncidence>> {
        assert!(self.capture.source().text().code_units().is_empty());
        assert!(self.capture.processed_text().code_units().is_empty());
        assert!(self.capture.captured_origin(0).is_none());
        assert!(self.capture.captured_control(0).is_none());
        assert!(self.capture.captured_atomic(0).is_none());
        let paragraphs = (0..3)
            .map(|i| self.capture.captured_paragraph(i).unwrap())
            .collect::<Vec<_>>();
        assert!(paragraphs[0].exit_position() < paragraphs[1].enter_position());
        assert!(paragraphs[1].exit_position() < paragraphs[2].enter_position());
        assert_ne!(paragraphs[0], paragraphs[2]);
        assert_ne!(self.prepared[0], self.prepared[2]);
        let mut pins = vec![];
        for (i, p) in self.prepared.iter().enumerate() {
            assert_eq!(
                p.captured_owner().unwrap().members().collect::<Vec<_>>(),
                vec![paragraphs[i].clone()]
            );
            assert_eq!(p.analyzed_source().source(), self.capture.source());
            assert_eq!(p.origin_views().count(), 0);
            assert_eq!(p.control_views().count(), 0);
            assert_eq!(p.atomic_views().count(), 0);
            assert_eq!(p.virtual_separators().count(), 0);
            assert_eq!(p.scopes().count(), 0);
            let offers = p
                .opportunity_views()
                .map(|view| view.handle())
                .collect::<Vec<_>>();
            assert_eq!(
                offers
                    .iter()
                    .map(|offer| {
                        let boundary = offer.view().boundary();
                        (
                            offer.view().kind(),
                            boundary.view().participant_offset(),
                            boundary.view().source_before().get(),
                            boundary.view().source_after().get(),
                        )
                    })
                    .collect::<Vec<_>>(),
                if i == 1 {
                    vec![(t::PreparedBreakKind::ParagraphEnd, 0, 0, 0)]
                } else {
                    vec![
                        (t::PreparedBreakKind::Forced, 1, 0, 0),
                        (t::PreparedBreakKind::ParagraphEnd, 1, 0, 0),
                    ]
                }
            );
            if i == 1 {
                assert_eq!(p.participants().count(), 0);
                continue;
            }
            let participants = p.participants().collect::<Vec<_>>();
            assert_eq!(participants.len(), 1);
            let t::PreparedParticipant::ComputedLineBoundary(group) = &participants[0] else {
                panic!("real computed participant required");
            };
            let cause_index = if i == 0 { 0 } else { 1 };
            let pair_start = if i == 0 { 0 } else { 2 };
            let cause = self
                .capture
                .captured_computed_line_boundary(cause_index)
                .unwrap();
            assert_eq!(cause.paragraph(), paragraphs[i]);
            assert_eq!(
                cause.target(),
                t::CapturedComputedLineBoundaryTarget::Scope {
                    scope: self.scopes[pair_start + 1].clone(),
                    side: t::SourceBoxSide::Before,
                }
            );
            assert_eq!(group.view().causes().collect::<Vec<_>>(), vec![cause]);
            assert_eq!(
                offers[0].view().computed_line_boundary(),
                Some(group.clone())
            );
            assert!(offers[1].view().computed_line_boundary().is_none());
            assert_eq!(offers[0].view().boundary(), offers[1].view().boundary());
            let pin = p
                .boundary_incidences(
                    &p.source_segments().next().unwrap(),
                    &offers[0].view().boundary(),
                )
                .unwrap();
            assert_eq!(pin.len(), 1);
            for (scope, relation) in self.scopes.iter().zip(if i == 0 {
                [
                    t::CapturedScopeIncidence::Inside,
                    t::CapturedScopeIncidence::AtBefore,
                    t::CapturedScopeIncidence::Before,
                    t::CapturedScopeIncidence::Before,
                ]
            } else {
                [
                    t::CapturedScopeIncidence::After,
                    t::CapturedScopeIncidence::After,
                    t::CapturedScopeIncidence::Inside,
                    t::CapturedScopeIncidence::AtBefore,
                ]
            }) {
                assert_eq!(
                    p.incidence_scope_relation(&pin[0], scope).unwrap(),
                    relation
                );
            }
            pins.push(pin);
        }
        assert!(self.capture.captured_computed_line_boundary(2).is_none());
        pins
    }

    fn compute(&self, order: [usize; 3]) -> l::CompletedLayoutBatchOf<u32, S> {
        let flow = l::LayoutFlowInput::try_new(
            &self.tree,
            0,
            order.into_iter().map(|i| self.binding(i)).collect(),
            self.inventory(),
            Vec::new(),
        )
        .unwrap();
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(30.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request).unwrap()
    }

    fn assert_output(&self, batch: &l::CompletedLayoutBatchOf<u32, S>) {
        assert_eq!(batch.placed_text().len(), 2);
        assert!(batch.final_inline_fragments().is_empty());
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(30.0), S::from_f64(30.0))
        );
        for (output_index, paragraph, top, height, baseline, nominal, struts) in [
            (
                0,
                0,
                0.0,
                20.0,
                16.0,
                34.0,
                [
                    expected_metric(&self.scopes[0], 16.0, 20.0),
                    expected_metric(&self.scopes[1], 12.0, 30.0),
                ],
            ),
            (
                1,
                2,
                20.0,
                10.0,
                8.0,
                26.0,
                [
                    expected_metric(&self.scopes[2], 8.0, 10.0),
                    expected_metric(&self.scopes[3], 6.0, 24.0),
                ],
            ),
        ] {
            let entry = &batch.placed_text()[output_index];
            assert_eq!(entry.container(), 0);
            assert!(entry.nodes().is_empty());
            let layout = entry.layout();
            assert_eq!(
                layout.analyzed_source().identity(),
                self.prepared[paragraph].analyzed_source().identity()
            );
            assert_eq!(layout.source(), self.capture.source());
            assert_eq!(layout.source().id(), Some(t::Id::from_u64(658301)));
            assert_eq!(layout.source().revision(), 52);
            assert_eq!(
                layout
                    .run_views()
                    .map(|run| run.glyphs().count())
                    .sum::<usize>(),
                0
            );
            let lines = layout.lines();
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].range(), t::Range::new(0, 0));
            assert_eq!(lines[0].bounds().origin.y, top);
            assert_eq!(lines[0].bounds().size, t::Size::new(30.0, height));
            assert_eq!(lines[0].baseline(), top + baseline);
            assert_eq!(layout.metrics().line_count(), 1);
            let tails = layout.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(tails.len(), 1);
            assert_eq!(tails[0].origin.y, top + height);
            assert!(tails[0].origin.x.is_finite());
            assert_eq!(tails[0].orientation, t::LineOrientation::Horizontal);
            assert!(tails[0].atomics.is_empty());
            let tail = tails[0].line.finalized_line();
            assert_eq!(tail.prepared_text(), self.prepared[paragraph]);
            assert_eq!(
                tail.prepared_text()
                    .captured_owner()
                    .unwrap()
                    .members()
                    .collect::<Vec<_>>(),
                vec![self.capture.captured_paragraph(paragraph).unwrap()]
            );
            assert_eq!(tail.feedback().line_height, nominal);
            assert_eq!(tail.feedback().baseline, baseline);
            assert_eq!(tail.feedback().advance, 0.0);
            assert!(!tail.has_in_flow_content());
            assert_eq!(tail.fragments().count(), 0);
            let selected = tail.selected_line();
            let request = selected.request();
            let offers = self.prepared[paragraph]
                .opportunity_views()
                .map(|view| view.handle())
                .collect::<Vec<_>>();
            assert_eq!(request.context.end, t::LineEnd::Paragraph);
            assert_eq!(
                request.termination,
                t::PreparedTermination::Ordinary(offers[1].clone())
            );
            assert_eq!(request.start, offers[0].view().boundary());
            assert_eq!(request.end, offers[1].view().boundary());
            assert!(request.participants.is_empty());
            assert!(request.controls.is_empty());
            assert!(request.atomics.is_empty());
            assert_struts(&request.line_box_struts, &struts);
        }
    }
}

fn shared_gap_order<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = SharedGap::<S>::new(direction);
    let pins = fixture.facts();
    let forward = fixture.compute([0, 1, 2]);
    fixture.assert_output(&forward);
    let reverse = fixture.compute([2, 1, 0]);
    fixture.assert_output(&reverse);
    let replay = fixture.compute([2, 1, 0]);
    fixture.assert_output(&replay);
    for i in 0..2 {
        assert_eq!(
            forward.placed_text()[i].layout().key(),
            reverse.placed_text()[i].layout().key()
        );
        assert_eq!(
            reverse.placed_text()[i].layout().key(),
            replay.placed_text()[i].layout().key()
        );
    }
    fixture.assert_output(&forward);
    assert_eq!(fixture.facts(), pins);
    // This is the same genuine CapturedParagraph twice, not merely two empty
    // numeric ranges or two independently captured paragraphs at offset0.
    let duplicate = l::LayoutFlowInput::try_new(
        &fixture.tree,
        0,
        vec![
            fixture.binding(0),
            fixture.binding(1),
            fixture.binding(2),
            fixture.binding(0),
        ],
        fixture.inventory(),
        Vec::new(),
    )
    .err()
    .expect("duplicate genuine paragraph binding must refuse");
    assert!(
        matches!(
            duplicate.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TextBinding(
                l::TextBindingError::OverlappingParagraphs
            ))
        ),
        "{duplicate:?}"
    );
    assert_eq!(fixture.facts(), pins);
}

#[test]
fn shared_empty_gap_keeps_actual_paragraph_order_across_binding_order_and_replay() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        shared_gap_order::<f32>(direction);
        shared_gap_order::<f64>(direction);
    }
}

#[path = "node_free_completion/intrinsic_sizing.rs"]
mod intrinsic_sizing;
#[path = "node_free_completion/warm_custody.rs"]
mod warm_custody;

struct SemanticBlockTree<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    non_box_facts: l::NodeInputOf<S>,
    children: [Vec<u32>; 3],
}
impl<S: l::LayoutScalar> l::Traverse for SemanticBlockTree<S> {
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
impl<S: l::LayoutScalar> l::LayoutTree for SemanticBlockTree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        self.inputs[node as usize]
            .as_box()
            .unwrap_or(&self.non_box_facts)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}

struct BlockFence<S: l::LayoutScalar> {
    capture: Arc<t::ProcessedSource>,
    prepared: Vec<t::PreparedText>,
    scopes: Vec<t::CapturedScope>,
    tree: SemanticBlockTree<S>,
}
impl<S: l::LayoutScalar> BlockFence<S> {
    fn new(direction: l::Direction) -> Self {
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
            wrap: t::Wrap::None,
            hyphens: t::Hyphens::None,
            ..t::Style::default()
        };
        let mut source = t::Source::identified("X", t::Id::from_u64(658302), 53);
        source.span(t::Range::new(0, 1), style.clone());
        let block = || t::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: t::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style.clone(),
            used_direction: base,
        };
        let inline = || t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: None,
            boundaries: None,
        };
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source: source.clone(),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: vec![t::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: t::Range::new(0, 1),
                    aggregate: t::Range::new(0, 1),
                    kind: t::ParagraphOriginKind::Authored,
                }],
                controls: vec![],
                connections: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                computed_line_boundaries: vec![
                    t::ComputedLineBoundaryInput::Scope {
                        scope: 2,
                        side: t::SourceBoxSide::Before,
                    },
                    t::ComputedLineBoundaryInput::Scope {
                        scope: 5,
                        side: t::SourceBoxSide::Before,
                    },
                ],
                scopes: vec![block(), inline(), inline(), block(), inline(), inline()],
                paragraphs: (0..3)
                    .map(|_| t::SourceProcessingParagraph {
                        base_direction: base,
                        paragraph_strut: t::ParagraphStrut::DefaultFont,
                    })
                    .collect(),
                events: vec![
                    E::EnterScope(0),
                    E::EnterParagraph(0),
                    E::EnterScope(1),
                    E::EnterScope(2),
                    E::ExitScope(2),
                    E::ExitScope(1),
                    E::ExitParagraph(0),
                    E::EnterScope(3),
                    E::EnterParagraph(1),
                    E::Origin {
                        origin: 0,
                        affinity: t::Affinity::After,
                    },
                    E::ExitParagraph(1),
                    E::ExitScope(3),
                    E::EnterParagraph(2),
                    E::EnterScope(4),
                    E::EnterScope(5),
                    E::ExitScope(5),
                    E::ExitScope(4),
                    E::ExitParagraph(2),
                    E::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let scopes = [1, 2, 4, 5]
            .into_iter()
            .map(|i| capture.captured_scope(i).unwrap())
            .collect::<Vec<_>>();
        let mut fonts = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = capture
            .paragraphs()
            .map(|paragraph| {
                let analyzed = t::AnalyzedSource::from_processed_paragraph(
                    paragraph,
                    t::AnalysisTypography::HorizontalComposition,
                )
                .unwrap();
                let policies = analyzed
                    .wrapping_relations()
                    .filter(|row| row.ancestor_wrap().is_none())
                    .map(|row| t::WrappingPolicy {
                        relation: row.handle(),
                        ancestor_wrap: t::Wrap::None,
                    })
                    .collect();
                fonts
                    .prepare(
                        &analyzed.with_wrapping_policies(policies).unwrap(),
                        t::PreparationOptions::default(),
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(prepared.len(), 3);
        let origin = prepared[1].origin_views().next().unwrap().handle();
        let tree = SemanticBlockTree {
            inputs: vec![
                l::LayoutInputOf::box_input(l::NodeInputOf {
                    display: l::Display::Block,
                    direction,
                    size: l::Size::new(
                        l::PreferredSizeOf::value(
                            l::LengthPercentageOf::px(S::from_f64(30.0)).unwrap(),
                        ),
                        l::PreferredSizeOf::AUTO,
                    ),
                    ..l::NodeInputOf::default()
                }),
                l::LayoutInputOf::box_input(l::NodeInputOf {
                    display: l::Display::Block,
                    direction,
                    ..l::NodeInputOf::default()
                }),
                l::LayoutInputOf::inline_text(l::TextParticipation::new(origin)),
            ],
            // Required canonical non-box facts for actual InlineText node2;
            // this value is not a placeholder node or a box participation.
            non_box_facts: l::NodeInputOf::non_box(),
            children: [vec![1], vec![2], vec![]],
        };
        let fixture = Self {
            capture,
            prepared,
            scopes,
            tree,
        };
        fixture.facts();
        fixture
    }
    fn binding(&self, paragraph: usize) -> l::TextParagraphBinding<u32> {
        let (container, nodes, origins) = if paragraph == 1 {
            (
                1,
                vec![2],
                vec![l::TextOriginBinding {
                    origin: self.prepared[1].origin_views().next().unwrap().handle(),
                    node: 2,
                }],
            )
        } else {
            (0, vec![], vec![])
        };
        l::TextParagraphBinding {
            prepared: self.prepared[paragraph].clone(),
            container,
            nodes,
            origins,
            controls: vec![],
            atomics: vec![],
            line_strut: None,
        }
    }
    fn inventory(&self, reverse: bool) -> l::WrappingInput<u32> {
        let child = |box_| l::GeneratedChildBoxInput {
            box_,
            participation: l::GeneratedBoxParticipation::InFlow,
        };
        let row = |parent, children, formatting_context| l::InlineChildOrderInput {
            parent,
            children,
            prefix: l::ChildOrderCompleteness::Complete,
            suffix: l::ChildOrderCompleteness::Complete,
            formatting_context,
        };
        let boxes = self
            .scopes
            .iter()
            .map(|scope| l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(scope.clone())))
            .collect::<Vec<_>>();
        let mut root_children = vec![
            child(boxes[0].clone()),
            child(l::GeneratedBoxRef::Node(1)),
            child(boxes[2].clone()),
        ];
        if reverse {
            root_children.reverse();
        }
        l::WrappingInput {
            scopes: self
                .scopes
                .iter()
                .zip([(16.0, 20.0), (12.0, 30.0), (8.0, 10.0), (6.0, 24.0)])
                .map(
                    |(scope, (baseline, extent))| l::WrappingScopeInput::Captured {
                        scope: scope.clone(),
                        wrap_inside: l::WrapInside::Auto,
                        edges: l::WrapEdges::AUTO,
                        metric_contribution: Some(l::InlineBoxMetricContribution::Strut(metric(
                            baseline, extent,
                        ))),
                    },
                )
                .collect(),
            child_order: vec![
                row(l::GeneratedBoxRef::Node(0), root_children, 0),
                row(boxes[0].clone(), vec![child(boxes[1].clone())], 0),
                row(boxes[1].clone(), vec![], 0),
                row(boxes[2].clone(), vec![child(boxes[3].clone())], 0),
                row(boxes[3].clone(), vec![], 0),
                row(
                    l::GeneratedBoxRef::Node(1),
                    vec![child(l::GeneratedBoxRef::Node(2))],
                    1,
                ),
            ],
        }
    }
    fn facts(&self) -> Vec<Vec<t::PreparedBoundaryIncidence>> {
        assert_eq!(self.capture.source().text().code_units(), [88]);
        assert_eq!(self.capture.processed_text().code_units(), [88]);
        assert!(self.capture.captured_control(0).is_none());
        assert!(self.capture.captured_atomic(0).is_none());
        let root = self.capture.captured_scope(0).unwrap();
        let block = self.capture.captured_scope(3).unwrap();
        assert_eq!(block.parent(), Some(root.clone()));
        for outer in [&self.scopes[0], &self.scopes[2]] {
            assert_eq!(outer.parent(), Some(root.clone()));
        }
        assert_eq!(self.scopes[1].parent(), Some(self.scopes[0].clone()));
        assert_eq!(self.scopes[3].parent(), Some(self.scopes[2].clone()));
        let paragraphs = (0..3)
            .map(|i| self.capture.captured_paragraph(i).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(paragraphs[0].domain(), root);
        assert_eq!(paragraphs[1].domain(), block);
        assert_eq!(paragraphs[2].domain(), paragraphs[0].domain());
        assert!(paragraphs[0].exit_position() < block.enter_position());
        assert!(block.exit_position() < paragraphs[2].enter_position());
        let mut pins = vec![];
        for (i, p) in self.prepared.iter().enumerate() {
            assert_eq!(
                p.captured_owner().unwrap().members().collect::<Vec<_>>(),
                vec![paragraphs[i].clone()]
            );
            assert_eq!(p.analyzed_source().source(), self.capture.source());
            assert_eq!(p.control_views().count(), 0);
            assert_eq!(p.atomic_views().count(), 0);
            assert_eq!(p.virtual_separators().count(), 0);
            let offers = p
                .opportunity_views()
                .map(|view| view.handle())
                .collect::<Vec<_>>();
            let expected = if i == 1 {
                vec![(t::PreparedBreakKind::ParagraphEnd, 1, 1, 1)]
            } else {
                let at = if i == 0 { 0 } else { 1 };
                vec![
                    (t::PreparedBreakKind::Forced, 1, at, at),
                    (t::PreparedBreakKind::ParagraphEnd, 1, at, at),
                ]
            };
            assert_eq!(
                offers
                    .iter()
                    .map(|offer| {
                        let b = offer.view().boundary();
                        (
                            offer.view().kind(),
                            b.view().participant_offset(),
                            b.view().source_before().get(),
                            b.view().source_after().get(),
                        )
                    })
                    .collect::<Vec<_>>(),
                expected
            );
            if i == 1 {
                assert_eq!(p.origin_views().count(), 1);
                let origin = p.origin_views().next().unwrap();
                assert_eq!(origin.local_range(), t::Range::new(0, 1));
                assert_eq!(origin.aggregate_range(), t::Range::new(0, 1));
                assert!(self.capture.captured_origin(0).is_some());
                assert_eq!(p.participants().count(), 1);
                continue;
            }
            assert_eq!(p.origin_views().count(), 0);
            let participants = p.participants().collect::<Vec<_>>();
            assert_eq!(participants.len(), 1);
            let t::PreparedParticipant::ComputedLineBoundary(group) = &participants[0] else {
                panic!("parent semantic work must be the actual computed participant");
            };
            let pair = if i == 0 { 0 } else { 2 };
            let cause = self
                .capture
                .captured_computed_line_boundary(if i == 0 { 0 } else { 1 })
                .unwrap();
            assert_eq!(cause.paragraph(), paragraphs[i]);
            assert_eq!(
                cause.target(),
                t::CapturedComputedLineBoundaryTarget::Scope {
                    scope: self.scopes[pair + 1].clone(),
                    side: t::SourceBoxSide::Before
                }
            );
            assert_eq!(group.view().causes().collect::<Vec<_>>(), vec![cause]);
            assert_eq!(
                offers[0].view().computed_line_boundary(),
                Some(group.clone())
            );
            assert!(offers[1].view().computed_line_boundary().is_none());
            assert_eq!(offers[0].view().boundary(), offers[1].view().boundary());
            let pin = p
                .boundary_incidences(
                    &p.source_segments().next().unwrap(),
                    &offers[0].view().boundary(),
                )
                .unwrap();
            assert_eq!(pin.len(), 1);
            assert_eq!(
                p.incidence_scope_relation(&pin[0], &self.scopes[pair])
                    .unwrap(),
                t::CapturedScopeIncidence::Inside
            );
            assert_eq!(
                p.incidence_scope_relation(&pin[0], &self.scopes[pair + 1])
                    .unwrap(),
                t::CapturedScopeIncidence::AtBefore
            );
            pins.push(pin);
        }
        pins
    }
    fn compute(&self, order: [usize; 3]) -> l::CompletedLayoutBatchOf<u32, S> {
        let flow = l::LayoutFlowInput::try_new(
            &self.tree,
            0,
            order.into_iter().map(|i| self.binding(i)).collect(),
            self.inventory(false),
            Vec::new(),
        )
        .unwrap();
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(30.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request).unwrap()
    }
    fn assert_output(&self, batch: &l::CompletedLayoutBatchOf<u32, S>, direction: l::Direction) {
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
            l::Size::new(S::from_f64(30.0), S::from_f64(40.0))
        );
        assert_eq!(
            output(1).location,
            l::Point::new(S::ZERO, S::from_f64(20.0))
        );
        assert_eq!(
            output(1).size,
            l::Size::new(S::from_f64(30.0), S::from_f64(10.0))
        );
        assert_eq!(batch.placed_text().len(), 3);
        for (i, container, nodes, range, top, height, baseline) in [
            (0, 0, vec![], t::Range::new(0, 0), 0.0, 20.0, 16.0),
            (1, 1, vec![2], t::Range::new(0, 1), 20.0, 10.0, 8.0),
            (2, 0, vec![], t::Range::new(1, 1), 30.0, 10.0, 8.0),
        ] {
            let entry = &batch.placed_text()[i];
            assert_eq!(entry.container(), container);
            assert_eq!(entry.nodes(), nodes.as_slice());
            let layout = entry.layout();
            assert_eq!(
                layout.analyzed_source().identity(),
                self.prepared[i].analyzed_source().identity()
            );
            assert_eq!(layout.source(), self.capture.source());
            assert_eq!(layout.source().text().code_units(), [88]);
            let lines = layout.lines();
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].range(), range);
            assert_eq!(lines[0].bounds().origin.y, top);
            assert_eq!(lines[0].bounds().size, t::Size::new(30.0, height));
            assert_eq!(lines[0].baseline(), top + baseline);
            if i == 1 {
                assert_eq!(layout.nonformatted_lines().count(), 0);
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
                let x = if direction == l::Direction::Ltr {
                    0.0
                } else {
                    20.0
                };
                assert_eq!(glyphs, vec![(58, 10.0, x, 28.0)]);
                continue;
            }
            assert_eq!(
                layout
                    .run_views()
                    .map(|run| run.glyphs().count())
                    .sum::<usize>(),
                0
            );
            let tails = layout.nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(tails.len(), 1);
            assert_eq!(tails[0].origin.y, top + height);
            assert!(tails[0].origin.x.is_finite());
            let tail = tails[0].line.finalized_line();
            assert_eq!(tail.prepared_text(), self.prepared[i]);
            assert_eq!(
                tail.prepared_text()
                    .captured_owner()
                    .unwrap()
                    .members()
                    .collect::<Vec<_>>(),
                vec![self.capture.captured_paragraph(i).unwrap()]
            );
            assert_eq!(tail.feedback().baseline, baseline);
            assert_eq!(
                tail.feedback().line_height,
                if i == 0 { 34.0 } else { 26.0 }
            );
            assert_eq!(tail.feedback().advance, 0.0);
            assert!(!tail.has_in_flow_content());
            assert_eq!(tail.fragments().count(), 0);
            let selected = tail.selected_line();
            let request = selected.request();
            let offers = self.prepared[i]
                .opportunity_views()
                .map(|view| view.handle())
                .collect::<Vec<_>>();
            assert_eq!(request.context.end, t::LineEnd::Paragraph);
            assert_eq!(
                request.termination,
                t::PreparedTermination::Ordinary(offers[1].clone())
            );
            assert_eq!(request.start, offers[0].view().boundary());
            assert_eq!(request.end, offers[1].view().boundary());
            assert!(request.participants.is_empty());
            assert!(request.controls.is_empty());
            assert!(request.atomics.is_empty());
            let struts = if i == 0 {
                [
                    expected_metric(&self.scopes[0], 16.0, 20.0),
                    expected_metric(&self.scopes[1], 12.0, 30.0),
                ]
            } else {
                [
                    expected_metric(&self.scopes[2], 8.0, 10.0),
                    expected_metric(&self.scopes[3], 6.0, 24.0),
                ]
            };
            assert_struts(&request.line_box_struts, &struts);
        }
        let fragments = batch.final_inline_fragments().iter().collect::<Vec<_>>();
        assert_eq!(fragments.len(), 1);
        assert!(matches!(
            fragments[0].owner(),
            l::InlineFragmentOwner::Node(2)
        ));
        let actual = fragments[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(actual.prepared_text(), self.prepared[1]);
        assert_eq!(
            actual
                .run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| (g.range(), g.id(), g.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            vec![(t::Range::new(0, 1), 58, 10.0)]
        );
    }
}
fn semantic_block_fence<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = BlockFence::<S>::new(direction);
    let pins = fixture.facts();
    let forward = fixture.compute([0, 1, 2]);
    fixture.assert_output(&forward, direction);
    let reverse = fixture.compute([2, 1, 0]);
    fixture.assert_output(&reverse, direction);
    let replay = fixture.compute([2, 1, 0]);
    fixture.assert_output(&replay, direction);
    for i in 0..3 {
        assert_eq!(
            forward.placed_text()[i].layout().key(),
            reverse.placed_text()[i].layout().key()
        );
        assert_eq!(
            reverse.placed_text()[i].layout().key(),
            replay.placed_text()[i].layout().key()
        );
    }
    fixture.assert_output(&forward, direction);
    assert_eq!(fixture.facts(), pins);
    let missing = l::LayoutFlowInput::try_new(
        &fixture.tree,
        0,
        vec![fixture.binding(0), fixture.binding(2)],
        fixture.inventory(false),
        Vec::new(),
    )
    .err()
    .expect("actual InlineText node2 still requires its child-domain paragraph binding");
    assert!(
        matches!(
            missing.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TextBinding(
                l::TextBindingError::MissingParagraph
            ))
        ),
        "{missing:?}"
    );
    let reversed = l::LayoutFlowInput::try_new(
        &fixture.tree,
        0,
        vec![fixture.binding(0), fixture.binding(1), fixture.binding(2)],
        fixture.inventory(true),
        Vec::new(),
    )
    .err()
    .expect("claimed U,Block,S order contradicts the actual complete capture S,Block,U");
    assert!(
        matches!(
            reversed.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
                l::WrappingScopeError::ChildOrder
            ))
        ),
        "{reversed:?}"
    );
    assert_eq!(fixture.facts(), pins);
}
#[test]
fn semantic_parent_paragraphs_stay_on_actual_sides_of_real_child_block() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        semantic_block_fence::<f32>(direction);
        semantic_block_fence::<f64>(direction);
    }
}

struct UnrepresentedBlock<S: l::LayoutScalar> {
    inputs: [l::NodeInputOf<S>; 2],
    children: [Vec<u32>; 2],
}
impl<S: l::LayoutScalar> l::Traverse for UnrepresentedBlock<S> {
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
impl<S: l::LayoutScalar> l::LayoutTree for UnrepresentedBlock<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        &self.inputs[node as usize]
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutInputViewOf::Box(&self.inputs[node as usize])
    }
}
fn assert_scope_association_refusal<S: l::LayoutScalar>(error: &l::LayoutErrorOf<u32, S>) {
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::TextBinding(
                l::TextBindingError::ScopeAssociation
            ))
        ),
        "{error:?}"
    );
}
fn ambiguous_semantic_gap<S: l::LayoutScalar>(direction: l::Direction) {
    let fixture = Fixture::<S>::new(direction, true);
    let pin = fixture.inventory_facts(true);
    let tree = UnrepresentedBlock {
        inputs: [
            l::NodeInputOf {
                display: l::Display::Block,
                direction,
                size: l::Size::new(
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(30.0)).unwrap(),
                    ),
                    l::PreferredSizeOf::AUTO,
                ),
                ..l::NodeInputOf::default()
            },
            l::NodeInputOf {
                display: l::Display::Block,
                direction,
                size: l::Size::new(
                    l::PreferredSizeOf::AUTO,
                    l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(7.0)).unwrap()),
                ),
                ..l::NodeInputOf::default()
            },
        ],
        children: [vec![1], vec![]],
    };
    let mut inventory = fixture.inventory();
    // S is genuinely known, but the relation of the real unmatched Block1 to
    // that source scope is unknown on BOTH sides. No complete order is claimed.
    let root = inventory
        .child_order
        .iter_mut()
        .find(|row| row.parent == l::GeneratedBoxRef::Node(0))
        .unwrap();
    root.prefix = l::ChildOrderCompleteness::Unknown;
    root.suffix = l::ChildOrderCompleteness::Unknown;
    let error =
        l::LayoutFlowInput::try_new(&tree, 0, vec![fixture.binding()], inventory, Vec::new())
            .err()
            .expect(
                "the actual unrepresented child leaves native gaps0 and1; neither is authenticated",
            );
    assert_scope_association_refusal(&error);
    assert_eq!(fixture.inventory_facts(true), pin);
}
#[test]
fn semantic_paragraph_refuses_two_possible_gaps_around_unrepresented_real_block() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        ambiguous_semantic_gap::<f32>(direction);
        ambiguous_semantic_gap::<f64>(direction);
    }
}

fn independent_capture_gap<S: l::LayoutScalar>(direction: l::Direction) {
    let a = Fixture::<S>::new(direction, true);
    let b = Fixture::<S>::new(direction, true);
    let pins = [a.inventory_facts(true), b.inventory_facts(true)];
    assert_eq!(a.capture.source(), b.capture.source());
    let a_members = a
        .prepared
        .captured_owner()
        .unwrap()
        .members()
        .collect::<Vec<_>>();
    let b_members = b
        .prepared
        .captured_owner()
        .unwrap()
        .members()
        .collect::<Vec<_>>();
    assert_ne!(a_members, b_members);
    let [a_paragraph]: [_; 1] = a_members
        .try_into()
        .expect("fixture has one captured paragraph");
    let [b_paragraph]: [_; 1] = b_members
        .try_into()
        .expect("fixture has one captured paragraph");
    assert!(
        !a_paragraph
            .enter_position()
            .same_capture(&b_paragraph.enter_position())
    );
    for reverse in [false, true] {
        let mut inventory = a.inventory();
        let mut second = b.inventory();
        inventory.scopes.append(&mut second.scopes);
        // Each real S[T] interior is complete. Deliberately supply NO root
        // child-order row purporting to order scopes from unrelated captures.
        inventory
            .child_order
            .retain(|row| row.parent != l::GeneratedBoxRef::Node(0));
        second
            .child_order
            .retain(|row| row.parent != l::GeneratedBoxRef::Node(0));
        inventory.child_order.append(&mut second.child_order);
        let bindings = if reverse {
            vec![b.binding(), a.binding()]
        } else {
            vec![a.binding(), b.binding()]
        };
        let error = l::LayoutFlowInput::try_new(&a.tree, 0, bindings, inventory, Vec::new())
            .err().expect("independent captures cannot impose a canonical order in the same unique native gap");
        assert_scope_association_refusal(&error);
    }
    assert_eq!([a.inventory_facts(true), b.inventory_facts(true)], pins);
}
#[test]
fn same_empty_native_gap_does_not_order_independent_captures_by_binding_vector() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        independent_capture_gap::<f32>(direction);
        independent_capture_gap::<f64>(direction);
    }
}

impl<S: l::LayoutScalar> Fixture<S> {
    fn two_cuts(direction: l::Direction, duplicate_requests: bool) -> Self {
        use t::SourceProcessingEvent as E;
        let base = if direction == l::Direction::Ltr {
            t::ParagraphBaseDirection::LeftToRight
        } else {
            t::ParagraphBaseDirection::RightToLeft
        };
        let before = t::ComputedLineBoundaryInput::Scope {
            scope: 2,
            side: t::SourceBoxSide::Before,
        };
        let after = t::ComputedLineBoundaryInput::Scope {
            scope: 2,
            side: t::SourceBoxSide::After,
        };
        let inline = || t::SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            alignment_boundary: None,
            font_reference: None,
            trim: Default::default(),
            style: None,
            boundaries: None,
        };
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source: t::Source::identified("", t::Id::from_u64(658303), 54),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: vec![],
                controls: vec![],
                connections: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                computed_line_boundaries: if duplicate_requests {
                    vec![before, after, before, after]
                } else {
                    vec![before, after]
                },
                scopes: vec![
                    t::SourceProcessingScope::Block {
                        writing_axis: None,
                        horizontal_inner_direction: None,
                        font_reference: None,
                        topology: t::SourceProcessingTopology::Complete,
                        trim: Default::default(),
                        base_style: t::Style {
                            font: t::Font::new().family("Ahem"),
                            size: 10.0,
                            line_height: t::LineHeight::Absolute(10.0),
                            wrap: t::Wrap::None,
                            hyphens: t::Hyphens::None,
                            ..t::Style::default()
                        },
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
            capture.captured_scope(1).unwrap(),
            capture.captured_scope(2).unwrap(),
        ];
        let analyzed = t::AnalyzedSource::from_processed_paragraph(
            capture.paragraphs().next().unwrap(),
            t::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let policies = analyzed
            .wrapping_relations()
            .filter(|row| row.ancestor_wrap().is_none())
            .map(|row| t::WrappingPolicy {
                relation: row.handle(),
                ancestor_wrap: t::Wrap::None,
            })
            .collect();
        let mut fonts = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = fonts
            .prepare(
                &analyzed.with_wrapping_policies(policies).unwrap(),
                t::PreparationOptions::default(),
            )
            .unwrap();
        Self {
            capture,
            prepared,
            scopes,
            tree: Root {
                input: l::NodeInputOf {
                    display: l::Display::Block,
                    direction,
                    size: l::Size::new(
                        l::PreferredSizeOf::value(
                            l::LengthPercentageOf::px(S::from_f64(30.0)).unwrap(),
                        ),
                        l::PreferredSizeOf::AUTO,
                    ),
                    ..l::NodeInputOf::default()
                },
                children: [],
            },
        }
    }
    fn two_cut_facts(&self, duplicates: bool) -> Vec<Vec<t::PreparedBoundaryIncidence>> {
        assert!(self.capture.source().text().code_units().is_empty());
        assert!(self.capture.processed_text().code_units().is_empty());
        assert_eq!(
            self.prepared
                .captured_owner()
                .unwrap()
                .members()
                .collect::<Vec<_>>(),
            vec![self.capture.captured_paragraph(0).unwrap()]
        );
        assert_eq!(self.prepared.origin_views().count(), 0);
        assert_eq!(self.prepared.control_views().count(), 0);
        assert_eq!(self.prepared.atomic_views().count(), 0);
        assert_eq!(self.prepared.virtual_separators().count(), 0);
        let causes = [
            self.capture.captured_computed_line_boundary(0).unwrap(),
            self.capture.captured_computed_line_boundary(1).unwrap(),
        ];
        assert_ne!(causes[0], causes[1]);
        for (i, side) in [t::SourceBoxSide::Before, t::SourceBoxSide::After]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                causes[i].target(),
                t::CapturedComputedLineBoundaryTarget::Scope {
                    scope: self.scopes[1].clone(),
                    side
                }
            );
            assert_eq!(
                vec![causes[i].paragraph()],
                self.prepared
                    .captured_owner()
                    .unwrap()
                    .members()
                    .collect::<Vec<_>>()
            );
        }
        if duplicates {
            assert_eq!(
                self.capture.captured_computed_line_boundary(2),
                Some(causes[0].clone())
            );
            assert_eq!(
                self.capture.captured_computed_line_boundary(3),
                Some(causes[1].clone())
            );
            assert!(self.capture.captured_computed_line_boundary(4).is_none());
        } else {
            assert!(self.capture.captured_computed_line_boundary(2).is_none());
        }
        let participants = self.prepared.participants().collect::<Vec<_>>();
        assert_eq!(participants.len(), 2);
        let offers = self
            .prepared
            .opportunity_views()
            .map(|view| view.handle())
            .collect::<Vec<_>>();
        assert_eq!(
            offers
                .iter()
                .map(|offer| {
                    let b = offer.view().boundary();
                    (
                        offer.view().kind(),
                        b.view().participant_offset(),
                        b.view().source_before().get(),
                        b.view().source_after().get(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (t::PreparedBreakKind::Forced, 1, 0, 0),
                (t::PreparedBreakKind::Forced, 2, 0, 0),
                (t::PreparedBreakKind::ParagraphEnd, 2, 0, 0),
            ]
        );
        assert_ne!(offers[0].view().boundary(), offers[1].view().boundary());
        assert_eq!(offers[1].view().boundary(), offers[2].view().boundary());
        assert!(offers[2].view().computed_line_boundary().is_none());
        let mut pins = vec![];
        for i in 0..2 {
            let t::PreparedParticipant::ComputedLineBoundary(group) = &participants[i] else {
                panic!("each distinct logical cut needs its genuine computed participant");
            };
            assert_eq!(
                offers[i].view().computed_line_boundary(),
                Some(group.clone())
            );
            assert_eq!(
                group.view().causes().collect::<Vec<_>>(),
                vec![causes[i].clone()]
            );
            let pin = self
                .prepared
                .boundary_incidences(
                    &self.prepared.source_segments().next().unwrap(),
                    &offers[i].view().boundary(),
                )
                .unwrap();
            assert_eq!(pin.len(), 1);
            assert_eq!(
                self.prepared
                    .incidence_scope_relation(&pin[0], &self.scopes[0])
                    .unwrap(),
                t::CapturedScopeIncidence::Inside
            );
            assert_eq!(
                self.prepared
                    .incidence_scope_relation(&pin[0], &self.scopes[1])
                    .unwrap(),
                if i == 0 {
                    t::CapturedScopeIncidence::AtBefore
                } else {
                    t::CapturedScopeIncidence::AtAfter
                }
            );
            pins.push(pin);
        }
        pins
    }
    fn assert_two_cut_output(&self, batch: &l::CompletedLayoutBatchOf<u32, S>) {
        assert_eq!(batch.placed_text().len(), 1);
        let entry = &batch.placed_text()[0];
        assert_eq!(entry.container(), 0);
        assert!(entry.nodes().is_empty());
        let layout = entry.layout();
        assert_eq!(
            layout.analyzed_source().identity(),
            self.prepared.analyzed_source().identity()
        );
        assert_eq!(layout.source(), self.capture.source());
        let lines = layout.lines();
        assert_eq!(
            lines
                .iter()
                .map(|line| (line.range(), line.bounds().size, line.baseline()))
                .collect::<Vec<_>>(),
            vec![
                (t::Range::new(0, 0), t::Size::new(30.0, 20.0), 16.0),
                (t::Range::new(0, 0), t::Size::new(30.0, 34.0), 36.0),
            ]
        );
        assert_eq!(layout.metrics().line_count(), 2);
        assert_eq!(layout.metrics().last_baseline(), Some(36.0));
        assert_eq!(
            layout
                .run_views()
                .map(|run| run.glyphs().count())
                .sum::<usize>(),
            0
        );
        assert!(batch.final_inline_fragments().is_empty());
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(30.0), S::from_f64(54.0))
        );
        let tails = layout.nonformatted_lines().collect::<Vec<_>>();
        assert_eq!(tails.len(), 1);
        assert_eq!(tails[0].origin.y, 54.0);
        assert!(tails[0].origin.x.is_finite());
        assert!(tails[0].atomics.is_empty());
        let tail = tails[0].line.finalized_line();
        assert_eq!(tail.prepared_text(), self.prepared);
        assert_eq!(tail.feedback().line_height, 20.0);
        assert_eq!(tail.feedback().baseline, 16.0);
        assert_eq!(tail.feedback().advance, 0.0);
        assert!(!tail.has_in_flow_content());
        let selected = tail.selected_line();
        let request = selected.request();
        let offers = self
            .prepared
            .opportunity_views()
            .map(|view| view.handle())
            .collect::<Vec<_>>();
        assert_eq!(request.context.end, t::LineEnd::Paragraph);
        assert_eq!(request.start, offers[1].view().boundary());
        assert_eq!(request.end, offers[2].view().boundary());
        assert_eq!(
            request.termination,
            t::PreparedTermination::Ordinary(offers[2].clone())
        );
        assert!(request.participants.is_empty());
        assert!(request.controls.is_empty());
        assert!(request.atomics.is_empty());
        assert_struts(
            &request.line_box_struts,
            &[expected_metric(&self.scopes[0], 16.0, 20.0)],
        );
    }
}
fn distinct_cut_sequence<S: l::LayoutScalar>(direction: l::Direction) {
    for duplicate in [false, true] {
        let fixture = Fixture::<S>::two_cuts(direction, duplicate);
        let pins = fixture.two_cut_facts(duplicate);
        let batch = fixture
            .try_compute(
                fixture.inventory(),
                l::AvailableOf::definite(S::from_f64(30.0)),
            )
            .unwrap();
        fixture.assert_two_cut_output(&batch);
        let replay = fixture
            .try_compute(
                fixture.inventory(),
                l::AvailableOf::definite(S::from_f64(30.0)),
            )
            .unwrap();
        fixture.assert_two_cut_output(&replay);
        assert_eq!(
            batch.placed_text()[0].layout().key(),
            replay.placed_text()[0].layout().key()
        );
        fixture.assert_two_cut_output(&batch);
        assert_eq!(fixture.two_cut_facts(duplicate), pins);
    }
}
#[test]
fn one_node_free_paragraph_keeps_distinct_zero_offset_forces_and_idempotent_cause_requests() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        distinct_cut_sequence::<f32>(direction);
        distinct_cut_sequence::<f64>(direction);
    }
}
