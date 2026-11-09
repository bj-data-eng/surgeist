//! Real Shape/metric refusal ordering and atomic publication around clearance.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{self, Phase};
use crate::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use surgeist_text as text;

const ROOT: u32 = 0;
const FLOAT: u32 = 1;
const TEXT: u32 = 2;
const CLEAR: u32 = 3;
const START: u32 = 10;
const END: u32 = 11;

/// The actual computed force precedes the allocated inline endpoints. With
/// AllocatedContributions, neither endpoint can supply metrics backward across
/// the force. This is the same Text-owned refusal as computed_line_boundaries'
/// allocated_metrics_cannot_be_borrowed_from_endpoints_beyond_the_force case.
fn paragraph(strut: text::ParagraphStrut) -> TextParagraphBinding<u32> {
    use text::SourceProcessingEvent as E;
    let style = text::Style {
        wrap: text::Wrap::None,
        ..crate::test_support::text::ahem_style(10.0, 10.0)
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: text::Source::new(""),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: Vec::new(),
            connections: Vec::new(),
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            computed_line_boundaries: vec![text::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: text::SourceBoxSide::Before,
            }],
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
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: strut,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::None);
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let controls = prepared
        .control_views()
        .zip([START, END])
        .map(|(view, node)| TextControlBinding {
            occurrence: view.handle(),
            node,
        })
        .collect();
    TextParagraphBinding {
        prepared,
        container: TEXT,
        nodes: vec![START, END],
        origins: Vec::new(),
        controls,
        atomics: Vec::new(),
        inline_box_fragments: InlineBoxFragmentationDemand::NotRequested,
        line_strut: None,
    }
}

struct FailureHost<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    publication_targets: LayoutPublicationTargets<u32>,
    published: Option<CompletedLayoutBatchOf<u32, S>>,
    caches: HashMap<u32, CacheOf<S>>,
    fail_shape: Cell<bool>,
    queries: RefCell<Vec<(u32, FloatExclusionQueryOf<S>)>>,
}

impl<S: LayoutScalar> FailureHost<S> {
    fn new() -> Self {
        let mut tree = PublicLayoutTreeOf::new()
            .children(ROOT, [FLOAT, TEXT, CLEAR])
            .children(TEXT, [START, END])
            .style(ROOT, super::block(0.0, 0.0, None))
            .style(
                TEXT,
                NodeInputOf {
                    // The initial point band lies strictly inside the float,
                    // rather than at its nonoverlapping top boundary.
                    padding: Edges {
                        top: LengthOf::px(S::ONE),
                        ..Edges::all(LengthOf::ZERO)
                    },
                    ..super::block(0.0, 0.0, None)
                },
            )
            .style(
                FLOAT,
                NodeInputOf {
                    float: Float::Left,
                    float_exclusion: FloatExclusion::Shape,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(50.0)),
                        PreferredSizeOf::px(S::from_f64(100.0)),
                    ),
                    ..super::block(0.0, 0.0, None)
                },
            )
            .style(
                CLEAR,
                NodeInputOf {
                    clear: Clear::Left,
                    ..super::block(10.0, 0.0, Some(10.0))
                },
            );
        for (node, kind) in [
            (START, InlineBoundaryKind::Start),
            (END, InlineBoundaryKind::End),
        ] {
            tree = tree.input(
                node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        kind,
                        InlineMetricsOf::try_new(
                            InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                ),
            );
        }
        Self {
            tree,
            publication_targets: LayoutPublicationTargets::new(),
            published: None,
            caches: HashMap::new(),
            fail_shape: Cell::new(false),
            queries: RefCell::new(Vec::new()),
        }
    }

    fn compute(
        &self,
        binding: &TextParagraphBinding<u32>,
        dirty: &[u32],
    ) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S, &'static str> {
        let capture = LayoutFlowInput::try_new(
            self,
            ROOT,
            vec![binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )?;
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            axes,
            Size::new(
                Some(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
                None,
            ),
            OrthogonalFallbackOf::new(
                Size::new(
                    Some(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
                    None,
                ),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ));
        compute_layout_with_flow_invalidated(&capture, ROOT, request, dirty)
    }
}

impl<S: LayoutScalar> Traverse for FailureHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for FailureHost<S> {
    type MeasureError = &'static str;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.caches.get(&node)?.get_with_context(input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<LayoutOutputEntryOf<u32, S>> {
        self.published
            .as_ref()?
            .node_layout(node)
            .map(|row| match row {
                NodeLayoutRefOf::SingleBox(row) => row.unrounded_entry(),
                NodeLayoutRefOf::NonBox(row) => row.unrounded_entry(),
                NodeLayoutRefOf::Fragments(_) => {
                    panic!("continuous failure fixture is not fragmented")
                }
            })
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, FLOAT);
        self.queries.borrow_mut().push((node, query));
        Some(if self.fail_shape.get() {
            Err("clearance fixture Shape refused")
        } else {
            // Shape emptiness does not remove the float's clearance bound.
            Ok(None)
        })
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for FailureHost<S> {
    type Error = &'static str;
    type Prepared = (HashMap<u32, CacheOf<S>>, CompletedLayoutBatchOf<u32, S>);
    fn publication_targets(&self) -> &LayoutPublicationTargets<u32> {
        &self.publication_targets
    }
    fn publication_targets_mut(&mut self) -> &mut LayoutPublicationTargets<u32> {
        &mut self.publication_targets
    }
    fn prepare_layout_batch(
        &self,
        preparation: LayoutBatchPreparation<'_, u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let batch = preparation.batch();
        let mut caches = self.caches.clone();
        for node in batch.invalidated_nodes() {
            caches.remove(node);
        }
        for row in batch.cache_clear_entries() {
            caches.remove(&row.node());
        }
        for row in batch.cache_store_entries() {
            caches.entry(row.node()).or_default().store_with_context(
                row.input(),
                row.context(),
                row.output(),
            );
        }
        Ok((caches, batch.clone()))
    }
    fn commit_layout_batch(&mut self, commit: LayoutBatchCommit<'_, u32, S, Self::Prepared>) {
        let (caches, batch) = commit.into_prepared();
        self.caches = caches;
        self.published = Some(batch);
    }
}

#[derive(Debug, PartialEq)]
struct TextSnapshot {
    container: u32,
    nodes: Vec<u32>,
    source: text::Source,
    key: text::Key,
    metrics: text::InlineMetricFacts,
    captured_members: Vec<text::CapturedParagraph>,
}

fn text_snapshot<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<TextSnapshot> {
    batch
        .placed_text()
        .iter()
        .map(|row| {
            let analyzed = row.layout().analyzed_source();
            TextSnapshot {
                container: row.container(),
                nodes: row.nodes().to_vec(),
                source: row.layout().source().clone(),
                key: row.layout().key(),
                metrics: row.layout().inline_metric_facts(),
                captured_members: analyzed.captured_owner().unwrap().members().collect(),
            }
        })
        .collect()
}

fn failures_preserve_publication<S: LayoutScalar>() {
    let good = paragraph(text::ParagraphStrut::DefaultFont);
    let missing = paragraph(text::ParagraphStrut::AllocatedContributions);
    let mut host = FailureHost::<S>::new();
    let baseline = host.compute(&good, &[]).unwrap();
    // Ahem's authored 10px line/endpoint contributions precede this clear.
    // The float ends at100, beyond those lines plus the clearing top margin10;
    // the real clearing block therefore starts at100 and ends at110.
    assert_eq!(
        readback::single_box_output(&baseline, CLEAR, Phase::Unrounded)
            .location
            .y,
        S::from_f64(100.0),
    );
    assert_eq!(
        readback::single_box_output(&baseline, ROOT, Phase::Unrounded)
            .size
            .height,
        S::from_f64(110.0),
    );
    assert!(!baseline.placed_text().is_empty());
    assert!(
        baseline
            .placed_text()
            .iter()
            .any(|row| !row.layout().lines().is_empty())
    );
    baseline.apply_to(&mut host).unwrap();
    let warm = host.compute(&good, &[]).unwrap();
    assert_eq!(text_snapshot(&warm), text_snapshot(&baseline));
    warm.apply_to(&mut host).unwrap();
    let prior = host.published.clone().unwrap();
    let caches = host.caches.clone();
    assert!(
        !caches.is_empty(),
        "fixture must retain real committed cache entries"
    );
    let targets = host.publication_targets.clone();
    assert!(!targets.published_members(ROOT).is_empty());
    let unchanged = |host: &FailureHost<S>| {
        let published = host.published.as_ref().unwrap();
        assert_eq!(host.caches, caches);
        assert_eq!(host.publication_targets, targets);
        assert_eq!(
            readback::continuous_custody(published, Phase::Unrounded),
            readback::continuous_custody(&prior, Phase::Unrounded)
        );
        assert_eq!(
            readback::continuous_custody(published, Phase::Final),
            readback::continuous_custody(&prior, Phase::Final)
        );
        assert_eq!(text_snapshot(published), text_snapshot(&prior));
    };
    let dirty = [ROOT, FLOAT, TEXT, CLEAR, START, END];
    host.fail_shape.set(true);
    // Repeated refusals keep actual attempted host work. Discarding a candidate
    // cannot erase this provider's query ledger or publish any scratch result.
    for _ in 0..2 {
        let before = host.queries.borrow().len();
        let error = host.compute(&missing, &dirty).unwrap_err();
        assert_eq!(
            error.operation(),
            LayoutOperation::FloatExclusionQuery,
            "actual error: {error:?}; attempted queries: {:?}",
            &host.queries.borrow()[before..],
        );
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: TEXT,
                subject: FLOAT,
            }
        );
        assert!(
            matches!(
                error.kind(),
                LayoutErrorKindOf::Measurement("clearance fixture Shape refused")
            ),
            "{error:?}"
        );
        let queries = host.queries.borrow();
        assert!(queries.len() > before);
        assert!(
            queries[before..].iter().any(|(node, query)| *node == FLOAT
                && query.band_minimum() == S::ONE
                && query.band_maximum() == S::ONE),
            "the real height-zero point query must reach Shape before metrics"
        );
        drop(queries);
        unchanged(&host);
    }
    host.fail_shape.set(false);
    let before = host.queries.borrow().len();
    let error = host.compute(&missing, &dirty).unwrap_err();
    assert_eq!(error.operation(), LayoutOperation::TextSelection);
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(TEXT));
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::Text(error) if matches!(
            error.detail(), Some(text::ErrorDetail::InvalidTextProtocol {
                reason: text::ProtocolError::MissingLineMetrics,
            })
        )),
        "removing Shape failure must reveal the actual later metric refusal: {error:?}"
    );
    assert!(host.queries.borrow().len() > before);
    unchanged(&host);
    // Restore only the missing metric authority. A later successful complete
    // batch remains unapplied until the host's ordinary atomic sink commits it.
    let retry = host.compute(&good, &dirty).unwrap();
    assert_eq!(text_snapshot(&retry), text_snapshot(&prior));
    assert_eq!(
        readback::continuous_custody(&retry, Phase::Unrounded),
        readback::continuous_custody(&prior, Phase::Unrounded)
    );
    unchanged(&host);
    retry.apply_to(&mut host).unwrap();
    assert_eq!(
        text_snapshot(host.published.as_ref().unwrap()),
        text_snapshot(&prior)
    );
}

#[test]
fn clearance_point_band_failure_precedes_metrics_and_preserves_publication_f32() {
    failures_preserve_publication::<f32>();
}

#[test]
fn clearance_point_band_failure_precedes_metrics_and_preserves_publication_f64() {
    failures_preserve_publication::<f64>();
}
