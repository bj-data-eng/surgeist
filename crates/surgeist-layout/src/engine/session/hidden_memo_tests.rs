use super::*;
use crate::engine::validation::ValidationGraph;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    AlignContent, AvailableSpaceOf, AxisSizingRequestOf, ContainingLayoutContext, Direction,
    Display, FlowAxes, LayoutInputViewOf, NearestScrollportOf, NodeInputOf,
    OrthogonalFallbackLimitOf, OrthogonalFallbackOf, ParentFormattingContext, PreferredSizeOf,
    SizingOperation, SizingRequestOf, TrackComponentOf, Traverse, WrappingInput, WritingMode,
};

const ROOT: u32 = 0;
const HIDDEN: u32 = 1;
const VISIBLE: u32 = 2;
const SHARED: u32 = 3;
const FILLER: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OriginalRefusal;

struct SharedTree<S: LayoutScalar> {
    source: PublicLayoutTreeOf<S>,
    refuse: bool,
    measurements: Cell<usize>,
}

impl<S: LayoutScalar> Traverse for SharedTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a> = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>;

    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.source, node)
    }

    fn child_count(&self, node: u32) -> usize {
        self.source.child_count(node)
    }

    fn child(&self, node: u32, index: usize) -> u32 {
        self.source.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for SharedTree<S> {
    type MeasureError = OriginalRefusal;

    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.source.node_input(node)
    }

    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.source.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: u32) -> bool {
        self.refuse && node == SHARED
    }

    fn measure_leaf(
        &self,
        node: u32,
        _: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, OriginalRefusal>> {
        (self.refuse && node == SHARED).then(|| {
            self.measurements.set(self.measurements.get() + 1);
            Err(OriginalRefusal)
        })
    }
}

fn shared_tree<S: LayoutScalar>(hidden_first: bool, refuse: bool) -> SharedTree<S> {
    let n = S::from_f64;
    let grid = |width, height, column, row| NodeInputOf {
        display: Display::Grid,
        size: Size::new(
            PreferredSizeOf::px(n(width)),
            PreferredSizeOf::px(n(height)),
        ),
        grid_template_columns: vec![TrackComponentOf::px(n(column))],
        grid_template_rows: vec![TrackComponentOf::px(n(row))],
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    };
    SharedTree {
        source: PublicLayoutTreeOf::new()
            .children(
                ROOT,
                if hidden_first {
                    [HIDDEN, VISIBLE]
                } else {
                    [VISIBLE, HIDDEN]
                },
            )
            .children(HIDDEN, [FILLER, SHARED])
            .children(VISIBLE, [SHARED])
            .style(ROOT, grid(40.0, 20.0, 30.0, 10.0))
            .style(VISIBLE, grid(30.0, 10.0, 7.0, 5.0))
            .style(
                HIDDEN,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            )
            .style(
                FILLER,
                NodeInputOf {
                    display: Display::None,
                    ..NodeInputOf::default()
                },
            )
            .style(
                SHARED,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::px(n(7.0)), PreferredSizeOf::px(n(5.0))),
                    ..NodeInputOf::default()
                },
            ),
        refuse,
        measurements: Cell::new(0),
    }
}

fn request<S: LayoutScalar>(parent: ParentFormattingContext) -> ComputeInputOf<S> {
    let extent = |value| {
        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(S::from_f64(value)).unwrap())
    };
    ComputeInputOf::for_child(
        crate::RunMode::PerformLayout,
        crate::SizingMode::InherentSize,
        crate::RequestedAxis::Both,
        Size::new(Some(S::from_f64(40.0)), Some(S::from_f64(20.0))),
        Size::new(Some(S::from_f64(40.0)), Some(S::from_f64(20.0))),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            parent,
        ),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(extent(40.0), SizingOperation::Normal),
                AxisSizingRequestOf::new(extent(20.0), SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}

fn session<'a, S: LayoutScalar>(
    tree: &'a SharedTree<S>,
    graph: &'a ValidationGraph<u32>,
    invalidated: Vec<u32>,
) -> ComputeSession<'a, SharedTree<S>> {
    let wrapping = crate::inline_wrapping::WrappingContext::new(
        tree,
        graph,
        &WrappingInput::default(),
        &[],
        |_| false,
    )
    .unwrap();
    let mut session = ComputeSession::unobserved(tree, invalidated);
    session.install_wrapping(Arc::new(wrapping), graph);
    session.enable_request_memo();
    session
}

fn clear_staging<S: LayoutScalar>(session: &mut ComputeSession<'_, SharedTree<S>>) {
    session.outputs.unrounded.clear();
    session.outputs.unrounded_fragments.clear();
    session.cache.stores.clear();
    session.cache.clears.clear();
}

fn assert_shared_effects<S: LayoutScalar>(
    session: &ComputeSession<'_, SharedTree<S>>,
    hidden_first: bool,
) {
    let n = S::from_f64;
    let shared = session.staged_unrounded(SHARED).unwrap();
    assert_eq!(shared.output().location, crate::Point::ZERO);
    assert_eq!(
        shared.output().size,
        if hidden_first {
            Size::new(n(7.0), n(5.0))
        } else {
            Size::ZERO
        }
    );
    assert_eq!(
        shared.output().source_index,
        crate::SourceIndex::new(usize::from(!hidden_first))
    );
    let visible = session.staged_unrounded(VISIBLE).unwrap();
    assert_eq!(visible.output().size, Size::new(n(30.0), n(10.0)));
    assert_eq!(
        visible.output().source_index,
        crate::SourceIndex::new(usize::from(hidden_first))
    );
    let hidden = session.staged_unrounded(HIDDEN).unwrap();
    assert_eq!(hidden.output().size, Size::ZERO);
    assert_eq!(
        hidden.output().source_index,
        crate::SourceIndex::new(usize::from(!hidden_first))
    );
    let filler = session.staged_unrounded(FILLER).unwrap();
    assert_eq!(filler.output().size, Size::ZERO);
    assert_eq!(filler.output().source_index, crate::SourceIndex::ZERO);
    assert!(
        session
            .cache
            .clears
            .iter()
            .map(|entry| entry.node())
            .eq([HIDDEN, FILLER, SHARED])
    );
    // Performed containers bypass host-cache stores. The only possible store
    // is the shared terminal: a later hidden placement removes that store.
    assert_eq!(session.cache.stores.is_empty(), !hidden_first);
    for entry in &session.cache.stores {
        assert_eq!(entry.node(), SHARED);
        assert_eq!(entry.output().size, Size::new(n(7.0), n(5.0)));
    }
    assert!(
        session
            .outputs
            .unrounded_fragments
            .iter()
            .all(|entry| entry.fragments.is_none())
    );
}

fn assert_shared_replay<S: LayoutScalar>() {
    for hidden_first in [true, false] {
        let tree = shared_tree::<S>(hidden_first, false);
        let (graph, invalidated) =
            crate::engine::validation::validate_layout_request_with_text_graph(
                &tree,
                ROOT,
                &[],
                &[],
            )
            .unwrap();
        assert_eq!(graph.parents[graph.index(SHARED).unwrap()].len(), 2);
        let mut session = session(&tree, &graph, invalidated);
        let observer = session.memo.as_ref().and_then(Memo::fork).unwrap();
        let input = request(ParentFormattingContext::Grid);
        observer.start_admission_trace();
        let output = session.compute_child(ROOT, input).unwrap();
        assert_eq!(output.size, Size::new(S::from_f64(40.0), S::from_f64(20.0)));
        assert!(
            observer
                .take_admission_trace()
                .iter()
                .any(|row| row.node == ROOT)
        );
        assert_shared_effects(&session, hidden_first);
        let stores = session.cache.stores.clone();
        for context in [
            ParentFormattingContext::Grid,
            ParentFormattingContext::BlockFlow,
            ParentFormattingContext::Grid,
        ] {
            clear_staging(&mut session);
            observer.start_admission_trace();
            assert_eq!(
                session.compute_child(ROOT, request(context)).unwrap(),
                output
            );
            let admissions = observer.take_admission_trace();
            if context == ParentFormattingContext::BlockFlow {
                assert!(
                    admissions.iter().any(|row| row.node == ROOT),
                    "changed containing context must compute a new root request"
                );
            } else {
                assert!(
                    admissions.is_empty(),
                    "exact repeat must replay, not recompute"
                );
            }
            assert_shared_effects(&session, hidden_first);
            assert_eq!(
                session.cache.stores, stores,
                "replay preserves the complete store order"
            );
        }
    }
}

fn assert_shared_refusal<S: LayoutScalar>() {
    for hidden_first in [true, false] {
        let tree = shared_tree::<S>(hidden_first, true);
        let (graph, invalidated) =
            crate::engine::validation::validate_layout_request_with_text_graph(
                &tree,
                ROOT,
                &[],
                &[],
            )
            .unwrap();
        let parent = session(&tree, &graph, invalidated);
        for _ in 0..2 {
            // Each immutable refusal runs in a fresh disposable session. Failed
            // captures are dropped; they never become completed shared packets.
            let mut trial = parent
                .child_staging(ROOT, super::super::FloatChildTrial::Detached, None)
                .unwrap();
            let error = trial
                .compute_child(ROOT, request(ParentFormattingContext::Grid))
                .unwrap_err();
            assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(SHARED));
            assert_eq!(error.operation(), crate::LayoutOperation::LeafMeasurement);
            assert!(matches!(
                error.kind(),
                crate::LayoutErrorKindOf::Measurement(OriginalRefusal)
            ));
        }
        assert_eq!(tree.measurements.get(), 2);
        assert!(
            parent.outputs.unrounded.is_empty()
                && parent.cache.stores.is_empty()
                && parent.cache.clears.is_empty()
        );
    }
}

#[test]
fn shared_hidden_visible_nodes_replay_in_both_source_orders_f32() {
    assert_shared_replay::<f32>();
}
#[test]
fn shared_hidden_visible_nodes_replay_in_both_source_orders_f64() {
    assert_shared_replay::<f64>();
}
#[test]
fn failed_shared_hidden_trials_preserve_original_refusal_f32() {
    assert_shared_refusal::<f32>();
}
#[test]
fn failed_shared_hidden_trials_preserve_original_refusal_f64() {
    assert_shared_refusal::<f64>();
}

fn assert_hidden_numeric_replay<S: LayoutScalar>() {
    const BREAK: u32 = 5;
    const START: u32 = 6;
    const END: u32 = 7;
    for hidden_first in [true, false] {
        let mut tree = shared_tree::<S>(hidden_first, false);
        let metrics = crate::InlineMetricsOf::try_new(
            crate::InlineBaselineContext::Named(crate::BaselineName::Alphabetic),
            S::from_f64(8.0),
            S::from_f64(10.0),
        )
        .unwrap();
        let edge =
            crate::InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(2.0), S::from_f64(1.0))
                .unwrap();
        // These controls are authored visible. Only their Box ancestor hides
        // them; their numeric metrics and strips must not publish visible work.
        tree.source = tree
            .source
            .children(HIDDEN, [BREAK, START, END])
            .input(
                BREAK,
                crate::LayoutInputOf::line_break(
                    crate::LineBreakInputOf::new().with_metrics(metrics),
                ),
            )
            .input(
                START,
                crate::LayoutInputOf::inline_boundary(
                    crate::InlineBoundaryInputOf::contribution(
                        crate::InlineBoundaryKind::Start,
                        metrics,
                        edge,
                    )
                    .unwrap(),
                ),
            )
            .input(
                END,
                crate::LayoutInputOf::inline_boundary(
                    crate::InlineBoundaryInputOf::contribution(
                        crate::InlineBoundaryKind::End,
                        metrics,
                        edge,
                    )
                    .unwrap(),
                ),
            );
        let (graph, invalidated) =
            crate::engine::validation::validate_layout_request_with_text_graph(
                &tree,
                ROOT,
                &[],
                &[],
            )
            .unwrap();
        let mut session = session(&tree, &graph, invalidated);
        let observer = session.memo.as_ref().and_then(Memo::fork).unwrap();
        let input = request(ParentFormattingContext::Grid);
        observer.start_admission_trace();
        let output = session.compute_child(ROOT, input).unwrap();
        assert!(
            observer
                .take_admission_trace()
                .iter()
                .any(|row| row.node == ROOT)
        );
        let stores = session.cache.stores.clone();
        for replay in [false, true] {
            if replay {
                clear_staging(&mut session);
                observer.start_admission_trace();
                assert_eq!(session.compute_child(ROOT, input).unwrap(), output);
                assert!(
                    observer.take_admission_trace().is_empty(),
                    "exact inherited-control repeat replays without body admission"
                );
            }
            assert!(
                session
                    .cache
                    .clears
                    .iter()
                    .map(|entry| entry.node())
                    .eq([HIDDEN, BREAK, START, END])
            );
            for (source, node) in [BREAK, START, END].into_iter().enumerate() {
                let entry = session.staged_unrounded(node).unwrap();
                assert_eq!(entry.non_box, Some(crate::NonBoxKind::Hidden));
                assert_eq!(entry.output().location, crate::Point::ZERO);
                assert_eq!(entry.output().size, Size::ZERO);
                assert_eq!(entry.output().source_index, crate::SourceIndex::new(source));
            }
            assert_eq!(
                session.staged_unrounded(SHARED).unwrap().output().size,
                Size::new(S::from_f64(7.0), S::from_f64(5.0))
            );
            assert!(!session.cache.stores.is_empty());
            assert!(
                session
                    .cache
                    .stores
                    .iter()
                    .all(|entry| entry.node() == SHARED)
            );
            assert_eq!(
                session.cache.stores, stores,
                "numeric clear replay preserves the complete store order"
            );
        }
    }
}

#[test]
fn inherited_hidden_numeric_descendants_replay_in_both_source_orders_f32() {
    assert_hidden_numeric_replay::<f32>();
}
#[test]
fn inherited_hidden_numeric_descendants_replay_in_both_source_orders_f64() {
    assert_hidden_numeric_replay::<f64>();
}
