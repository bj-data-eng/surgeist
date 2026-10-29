//! A real query failure during a losing Balance trial keeps its provider owner.
use std::cell::RefCell;

use crate::*;
use surgeist_text::BidiLevel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContourFailure {
    CrossesTransition,
}

struct ContourTree<S: LayoutScalar> {
    inputs: [NodeInputOf<S>; 5],
    queries: RefCell<Vec<FloatExclusionQueryOf<S>>>,
}

fn preferred<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

impl<S: LayoutScalar> ContourTree<S> {
    fn new(style: TextWrapStyle) -> Self {
        let atomic = |width, height| NodeInputOf {
            display: Display::InlineBlock,
            direction: Direction::Ltr,
            vertical_align: VerticalAlignOf::Baseline,
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                InlineBreakOpportunity::Allowed,
            )),
            size: Size::new(preferred(width), preferred(height)),
            ..NodeInputOf::default()
        };
        Self {
            inputs: [
                NodeInputOf {
                    display: Display::Block,
                    direction: Direction::Ltr,
                    text_wrap_style: style,
                    text_align: TextAlign::Start,
                    text_align_last: TextAlignLast::Start,
                    size: Size::new(preferred(100.0), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    float_exclusion: FloatExclusion::Shape,
                    size: Size::new(preferred(50.0), preferred(50.0)),
                    ..NodeInputOf::default()
                },
                atomic(60.0, 10.0),
                atomic(40.0, 10.0),
                atomic(50.0, 20.0),
            ],
            queries: RefCell::new(Vec::new()),
        }
    }
}

impl<S: LayoutScalar> Traverse for ContourTree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1, 2, 3, 4] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 4 } else { 0 }
    }

    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!(node, 0);
        [1, 2, 3, 4][index]
    }
}

impl<S: LayoutScalar> LayoutTree for ContourTree<S> {
    type MeasureError = ContourFailure;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        &self.inputs[node]
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        LayoutInputViewOf::Box(&self.inputs[node])
    }

    fn float_exclusion_interval(
        &self,
        node: usize,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 1);
        assert_eq!(query.margin_box().origin(), Point::ZERO);
        assert_eq!(query.margin_box().size(), Size::splat(S::from_f64(50.0)));
        self.queries.borrow_mut().push(query);
        let transition = S::from_f64(10.0);
        let start = query.band_minimum();
        let end = query.band_maximum();
        // The contour is empty above y10 and occupies physical x0..50 below.
        // This fallible host cannot evaluate bands crossing that transition.
        // Its answer depends only on the genuine geometric query, never on a
        // target, a source cut, a trial identity or an invocation counter.
        if start < transition && transition < end {
            Some(Err(ContourFailure::CrossesTransition))
        } else if end <= transition {
            Some(Ok(None))
        } else {
            Some(Ok(FloatExclusionIntervalOf::try_new(
                query,
                S::ZERO,
                S::from_f64(50.0),
            )
            .unwrap()))
        }
    }
}

fn request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(S::from_f64(100.0)),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap()
}

fn output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<usize, S>,
    node: usize,
) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

#[test]
fn numeric_auto_uses_valid_contour_bands_without_the_trial_only_failure() {
    fn lane<S: LayoutScalar>() {
        let tree = ContourTree::<S>::new(TextWrapStyle::Auto);
        let batch = compute_layout(&tree, 0, request()).unwrap();
        // Native atoms60+40 fill the actual first100px band at y0..10.
        // The final50px atom fills the second50px band at y10..30.
        // Hence their natural residuals are independently0/0.
        assert_eq!(output(&batch, 1).location, Point::ZERO);
        assert_eq!(output(&batch, 1).size, Size::splat(S::from_f64(50.0)));
        assert_eq!(output(&batch, 2).location, Point::ZERO);
        assert_eq!(
            output(&batch, 3).location,
            Point::new(S::from_f64(60.0), S::ZERO)
        );
        assert_eq!(
            output(&batch, 4).location,
            Point::new(S::from_f64(50.0), S::from_f64(10.0))
        );
        assert_eq!(
            output(&batch, 2).size,
            Size::new(S::from_f64(60.0), S::from_f64(10.0))
        );
        assert_eq!(
            output(&batch, 3).size,
            Size::new(S::from_f64(40.0), S::from_f64(10.0))
        );
        assert_eq!(
            output(&batch, 4).size,
            Size::new(S::from_f64(50.0), S::from_f64(20.0))
        );
        let queries = tree.queries.borrow();
        assert!(
            queries
                .iter()
                .any(|q| q.band_minimum() == S::ZERO && q.band_maximum() == S::from_f64(10.0))
        );
        assert!(queries.iter().any(
            |q| q.band_minimum() == S::from_f64(10.0) && q.band_maximum() == S::from_f64(30.0)
        ));
        assert!(queries.iter().all(
            |q| !(q.band_minimum() < S::from_f64(10.0) && S::from_f64(10.0) < q.band_maximum())
        ));
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn numeric_balance_losing_trial_preserves_exact_provider_failure_before_replay_error() {
    fn lane<S: LayoutScalar>() {
        let tree = ContourTree::<S>::new(TextWrapStyle::Balance);
        // The reference and local Auto use the valid bands above. A real
        // whole-group candidate150px wide has nominal height20 and queries
        // y0..20. Its actual provider failure must escape even if the losing
        // heuristic trial or later replay observes different fallback bands.
        // Err supplies no completed batch; no candidate geometry is published.
        let error = match compute_layout(&tree, 0, request()) {
            Ok(_) => panic!("a genuine Balance trial provider error must refuse publication"),
            Err(error) => error,
        };
        assert!(
            tree.queries
                .borrow()
                .iter()
                .any(|q| { q.band_minimum() == S::ZERO && q.band_maximum() == S::from_f64(20.0) })
        );
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1,
            },
            "original provider context must survive: {error:?}"
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement(ContourFailure::CrossesTransition)
        ));
    }
    lane::<f32>();
    lane::<f64>();
}
