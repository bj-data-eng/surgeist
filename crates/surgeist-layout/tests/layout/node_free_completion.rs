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
            trim: Default::default(),
            style: None,
            boundaries: None,
        };
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
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
        let analyzed =
            t::AnalyzedSource::from_processed_paragraph(capture.paragraphs().next().unwrap())
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
        assert_eq!(prepared.captured_paragraph(), capture.captured_paragraph(0));
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
        let incidences = self.prepared.boundary_incidences(&boundary).unwrap();
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
                .boundary_incidences(&offers[1].view().boundary())
                .unwrap(),
            incidences
        );
        incidences
    }
}

fn metric(baseline: f64, line_extent: f64) -> t::LineBoxStrut {
    t::LineBoxStrut {
        metrics: t::LineStrut {
            baseline,
            line_extent,
        },
        alignment: t::LineBoxAlignment::Baseline,
    }
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
        let flow = l::LayoutFlowInput::try_new(&self.tree, 0, vec![self.binding()], inventory)?;
        assert_eq!(flow.paragraphs().len(), 1);
        assert_eq!(flow.paragraphs()[0].prepared, self.prepared);
        assert_eq!(
            flow.paragraphs()[0].prepared.captured_paragraph(),
            self.capture.captured_paragraph(0)
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
        &[metric(16.0, 20.0), metric(12.0, 30.0)],
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
