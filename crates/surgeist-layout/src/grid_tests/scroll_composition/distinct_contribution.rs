// Native min/max measurement host for ordinary Grid contribution cases.
// Only native leaf1 is responsive. Grid0 and its completed geometry are real.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{LayoutTree, Traverse};
use std::cell::RefCell;

struct DistinctLeaf<S: LayoutScalar> {
    source: PublicLayoutTreeOf<S>,
    observed: RefCell<Vec<LeafMeasureInputOf<S>>>,
}

impl<S: LayoutScalar> Traverse for DistinctLeaf<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
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

impl<S: LayoutScalar> LayoutTree for DistinctLeaf<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.source.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.source.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == 1
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        self.observed.borrow_mut().push(input);
        let forty = S::from_f64(40.0);
        let eighty = S::from_f64(80.0);
        let known = input.known_content_size();
        // Native width contract: minimum unbroken content40, full content80.
        // Finite availability permits wrapping between those extents. A known
        // constraint remains authoritative; this callback never fabricates a
        // Grid result, baseline, source occurrence or overflowing hidden box.
        let width = known.width.unwrap_or_else(|| {
            let request = input.content_sizing_request().axes().width;
            match request.operation() {
                SizingOperation::MinContent => forty,
                SizingOperation::MaxContent => eighty,
                SizingOperation::Normal => request
                    .space()
                    .definite_size()
                    .map_or(eighty, |limit| limit.get().max(forty).min(eighty)),
            }
        });
        Some(Ok(Size::new(
            width,
            known.height.unwrap_or(S::from_f64(10.0)),
        )))
    }
}

#[derive(Clone, Copy, Debug)]
enum GrowthCap {
    MinContent,
    MaxContent,
}
impl GrowthCap {
    fn track<S: LayoutScalar>(self) -> TrackComponentOf<S> {
        TrackComponentOf::minmax(
            MinTrackSizingOf::AUTO,
            match self {
                Self::MinContent => MaxTrackSizingOf::MIN_CONTENT,
                Self::MaxContent => MaxTrackSizingOf::MAX_CONTENT,
            },
        )
    }
}

fn source<S: LayoutScalar>(width: f64, cap: GrowthCap, overflow: Overflow) -> DistinctLeaf<S> {
    DistinctLeaf {
        source: PublicLayoutTreeOf::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(width)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    grid_template_columns: vec![cap.track()],
                    grid_template_rows: vec![TrackComponentOf::px(S::from_f64(10.0))],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    // Stretch makes the sole actual child expose used track width.
                    // This is item sizing, NOT auto-track stretching past the cap.
                    justify_items: Some(AlignItems::Stretch),
                    align_items: Some(AlignItems::Stretch),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::AUTO,
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    overflow: computed_overflow(overflow, overflow),
                    ..NodeInputOf::default()
                },
            ),
        observed: RefCell::new(Vec::new()),
    }
}

fn qualify_actual_native_queries<S: LayoutScalar>(tree: &DistinctLeaf<S>) {
    for (available, expected) in [
        (
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
            40.0,
        ),
        (
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            80.0,
        ),
    ] {
        tree.observed.borrow_mut().clear();
        let input = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::Grid,
            ),
            SizingRequestOf::new(
                Size::new(
                    available,
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        );
        let leaf = crate::measurement::compute_tree_leaf(tree, 1, input, tree.node_input(1))
            .expect("actual native leaf algorithm accepts the query");
        assert_eq!(
            leaf.size,
            Size::new(S::from_f64(expected), S::from_f64(10.0))
        );
        assert_eq!(leaf.content_size, leaf.size);
        assert_eq!(leaf.baselines(), BaselinesOf::NONE);
        assert_eq!(leaf.named_baselines, NamedBaselinesOf::unknown());
        assert!(
            tree.observed.borrow().iter().any(|actual| {
                actual.known_content_size().width.is_none()
                    && actual.content_sizing_request().axes().width.operation()
                        == available.operation()
            }),
            "real leaf callback must receive this intrinsic query"
        );
    }
    tree.observed.borrow_mut().clear();
}

fn case<S: LayoutScalar>(width: f64, cap: GrowthCap, overflow: Overflow, track: f64) {
    let tree = source::<S>(width, cap, overflow);
    qualify_actual_native_queries(&tree);
    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(width)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("actual ordinary Grid with responsive native content completes");
    let root = fri08_c01_placement_output(&batch, 0);
    let child = fri08_c01_placement_output(&batch, 1);
    assert_eq!(root.size, Size::new(S::from_f64(width), S::from_f64(10.0)));
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(
        (root.border, root.padding, root.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    assert_eq!(child.location, Point::ZERO);
    assert_eq!(
        (child.border, child.padding, child.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    // Single track + genuine auto stretched item: this is actual completed
    // normal border geometry, independently determined by minimum/growth cap.
    assert_eq!(
        child.size,
        Size::new(S::from_f64(track), S::from_f64(10.0)),
        "{overflow:?} {cap:?} parent{width}: minimum, min-content and max-content are distinct"
    );
    let own = child
        .scroll_geometry
        .expect("real native leaf owns scroll geometry");
    assert_eq!(own.border_box().size(), child.size);
}

macro_rules! contribution_case {
    ($name:ident, $scalar:ty, $width:expr, $cap:expr, $overflow:expr, $expected:expr) => {
        #[test]
        fn $name() {
            case::<$scalar>($width, $cap, $overflow, $expected);
        }
    };
}

contribution_case!(
    hidden_minimum_zero_preserves_min_content_cap_f32,
    f32,
    20.0,
    GrowthCap::MinContent,
    Overflow::Hidden,
    20.0
);
contribution_case!(
    hidden_minimum_zero_preserves_min_content_cap_f64,
    f64,
    20.0,
    GrowthCap::MinContent,
    Overflow::Hidden,
    20.0
);
contribution_case!(
    clip_content_minimum_survives_shortage_f32,
    f32,
    20.0,
    GrowthCap::MinContent,
    Overflow::Clip,
    40.0
);
contribution_case!(
    clip_content_minimum_survives_shortage_f64,
    f64,
    20.0,
    GrowthCap::MinContent,
    Overflow::Clip,
    40.0
);
contribution_case!(
    hidden_min_content_growth_stops_at_forty_f32,
    f32,
    60.0,
    GrowthCap::MinContent,
    Overflow::Hidden,
    40.0
);
contribution_case!(
    hidden_min_content_growth_stops_at_forty_f64,
    f64,
    60.0,
    GrowthCap::MinContent,
    Overflow::Hidden,
    40.0
);
contribution_case!(
    clip_min_content_growth_stops_at_forty_f32,
    f32,
    60.0,
    GrowthCap::MinContent,
    Overflow::Clip,
    40.0
);
contribution_case!(
    clip_min_content_growth_stops_at_forty_f64,
    f64,
    60.0,
    GrowthCap::MinContent,
    Overflow::Clip,
    40.0
);
contribution_case!(
    hidden_max_content_growth_uses_sixty_f32,
    f32,
    60.0,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    60.0
);
contribution_case!(
    hidden_max_content_growth_uses_sixty_f64,
    f64,
    60.0,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    60.0
);
contribution_case!(
    clip_max_content_growth_uses_sixty_f32,
    f32,
    60.0,
    GrowthCap::MaxContent,
    Overflow::Clip,
    60.0
);
contribution_case!(
    clip_max_content_growth_uses_sixty_f64,
    f64,
    60.0,
    GrowthCap::MaxContent,
    Overflow::Clip,
    60.0
);
contribution_case!(
    hidden_max_content_cap_stops_at_eighty_f32,
    f32,
    100.0,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    80.0
);
contribution_case!(
    hidden_max_content_cap_stops_at_eighty_f64,
    f64,
    100.0,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    80.0
);
contribution_case!(
    clip_max_content_cap_stops_at_eighty_f32,
    f32,
    100.0,
    GrowthCap::MaxContent,
    Overflow::Clip,
    80.0
);
contribution_case!(
    clip_max_content_cap_stops_at_eighty_f64,
    f64,
    100.0,
    GrowthCap::MaxContent,
    Overflow::Clip,
    80.0
);

// Fit-content has three distinct inputs: automatic minimum0/40, min-content40,
// and max-content80. The finite30 cap must not replace the automatic minimum.
#[derive(Clone, Copy, Debug)]
enum FitRequest {
    MinContent,
    MaxContent,
    Definite(f64),
}

fn fit_case<S: LayoutScalar>(request: FitRequest, overflow: Overflow, expected_track: f64) {
    // Reuse the exact same real native host and its40/80 leaf qualification.
    let mut tree = source::<S>(20.0, GrowthCap::MaxContent, overflow);
    let mut root_style = tree.node_input(0).clone();
    root_style.grid_template_columns =
        vec![TrackComponentOf::fit_content(SizingCalculationOf::value(
            LengthPercentageOf::from_coefficients(S::from_f64(30.0), S::ZERO)
                .expect("finite30 fit-content limit"),
        ))];
    let (root_width, available) = match request {
        FitRequest::MinContent => {
            root_style.size.width = PreferredSizeOf::AUTO;
            (
                expected_track,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
            )
        }
        FitRequest::MaxContent => {
            root_style.size.width = PreferredSizeOf::AUTO;
            (
                expected_track,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            )
        }
        FitRequest::Definite(width) => {
            root_style.size.width = PreferredSizeOf::px(S::from_f64(width));
            (
                width,
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(S::from_f64(width)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
            )
        }
    };
    tree.source
        .insert_input(0, LayoutInputOf::box_input(root_style));
    qualify_actual_native_queries(&tree);
    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            available,
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("actual Grid fit-content with responsive native content completes");
    let root = fri08_c01_placement_output(&batch, 0);
    let child = fri08_c01_placement_output(&batch, 1);
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(
        root.size,
        Size::new(S::from_f64(root_width), S::from_f64(10.0)),
        "{overflow:?} {request:?}: AUTO intrinsic root consumes actual fit-content track"
    );
    assert_eq!(
        (root.border, root.padding, root.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    assert_eq!(child.location, Point::ZERO);
    assert_eq!(
        (child.border, child.padding, child.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    // Parent20 can consume20 only with genuine minimum0. Clip's content minimum40
    // survives the30 cap. Parent60 stops at30/40; intrinsicMIN/MAX use the same
    // minimum floor and capped contribution without inventing a definite root.
    assert_eq!(
        child.size,
        Size::new(S::from_f64(expected_track), S::from_f64(10.0)),
        "{overflow:?} {request:?}: fit-content30 keeps independent minimum and content authority"
    );
    let own = child
        .scroll_geometry
        .expect("native measured child keeps own geometry");
    assert_eq!(own.border_box().size(), child.size);
}

macro_rules! fit_contribution_case {
    ($name:ident, $scalar:ty, $request:expr, $overflow:expr, $expected:expr) => {
        #[test]
        fn $name() {
            fit_case::<$scalar>($request, $overflow, $expected);
        }
    };
}

fit_contribution_case!(
    fit_content_hidden_intrinsic_min_f32,
    f32,
    FitRequest::MinContent,
    Overflow::Hidden,
    30.0
);
fit_contribution_case!(
    fit_content_hidden_intrinsic_min_f64,
    f64,
    FitRequest::MinContent,
    Overflow::Hidden,
    30.0
);
fit_contribution_case!(
    fit_content_clip_intrinsic_min_f32,
    f32,
    FitRequest::MinContent,
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_clip_intrinsic_min_f64,
    f64,
    FitRequest::MinContent,
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_hidden_intrinsic_max_f32,
    f32,
    FitRequest::MaxContent,
    Overflow::Hidden,
    30.0
);
fit_contribution_case!(
    fit_content_hidden_intrinsic_max_f64,
    f64,
    FitRequest::MaxContent,
    Overflow::Hidden,
    30.0
);
fit_contribution_case!(
    fit_content_clip_intrinsic_max_f32,
    f32,
    FitRequest::MaxContent,
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_clip_intrinsic_max_f64,
    f64,
    FitRequest::MaxContent,
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_hidden_definite_shortage_f32,
    f32,
    FitRequest::Definite(20.0),
    Overflow::Hidden,
    20.0
);
fit_contribution_case!(
    fit_content_hidden_definite_shortage_f64,
    f64,
    FitRequest::Definite(20.0),
    Overflow::Hidden,
    20.0
);
fit_contribution_case!(
    fit_content_clip_definite_shortage_f32,
    f32,
    FitRequest::Definite(20.0),
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_clip_definite_shortage_f64,
    f64,
    FitRequest::Definite(20.0),
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_hidden_definite_growth_cap_f32,
    f32,
    FitRequest::Definite(60.0),
    Overflow::Hidden,
    30.0
);
fit_contribution_case!(
    fit_content_hidden_definite_growth_cap_f64,
    f64,
    FitRequest::Definite(60.0),
    Overflow::Hidden,
    30.0
);
fit_contribution_case!(
    fit_content_clip_definite_growth_cap_f32,
    f32,
    FitRequest::Definite(60.0),
    Overflow::Clip,
    40.0
);
fit_contribution_case!(
    fit_content_clip_definite_growth_cap_f64,
    f64,
    FitRequest::Definite(60.0),
    Overflow::Clip,
    40.0
);

// A spanning item preserves allocation to a track with a fixed minimum.
fn mixed_span_nonaffected_fixed_min_base<S: LayoutScalar>() {
    let mut tree = source::<S>(20.0, GrowthCap::MaxContent, Overflow::Hidden);
    let mut root_style = tree.node_input(0).clone();
    root_style.size.width = PreferredSizeOf::AUTO;
    root_style.grid_template_columns = vec![
        TrackComponentOf::minmax(
            MinTrackSizingOf::AUTO,
            MaxTrackSizingOf::px(S::from_f64(10.0)),
        ),
        TrackComponentOf::minmax(MinTrackSizingOf::px(S::ZERO), MaxTrackSizingOf::MAX_CONTENT),
    ];
    tree.source
        .insert_input(0, LayoutInputOf::box_input(root_style));
    let mut child_style = tree.node_input(1).clone();
    child_style.grid_column = GridPlacement::try_lines(1, 3).unwrap();
    child_style.grid_row = GridPlacement::try_lines(1, 2).unwrap();
    tree.source
        .insert_input(1, LayoutInputOf::box_input(child_style));
    qualify_actual_native_queries(&tree);
    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("genuine spanning native Grid computes");
    let root = fri08_c01_placement_output(&batch, 0);
    let child = fri08_c01_placement_output(&batch, 1);
    // Limited min-content target40: affected auto-min A grows to fixed limit10;
    // then NON-affected B grows30 up to its infinite growth limit, before any
    // beyond-limits pass. MIN_CONTENT does not later maximize these bases.
    assert_eq!(root.size, Size::new(S::from_f64(40.0), S::from_f64(10.0)));
    assert_eq!(child.size, root.size);
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(child.location, Point::ZERO);
    for output in [root, child] {
        assert_eq!(
            (output.border, output.padding, output.margin),
            (Edges::ZERO, Edges::ZERO, Edges::ZERO)
        );
    }
}
#[test]
fn mixed_span_nonaffected_fixed_min_base_f32() {
    mixed_span_nonaffected_fixed_min_base::<f32>();
}
#[test]
fn mixed_span_nonaffected_fixed_min_base_f64() {
    mixed_span_nonaffected_fixed_min_base::<f64>();
}

fn mixed_span_visible_explicit_zero_minimum<S: LayoutScalar>(end_marker_in_a: bool) {
    let mut tree = source::<S>(20.0, GrowthCap::MaxContent, Overflow::Visible);
    let mut root_style = tree.node_input(0).clone();
    root_style.size.width = PreferredSizeOf::AUTO;
    root_style.grid_template_columns = vec![
        TrackComponentOf::minmax(
            MinTrackSizingOf::AUTO,
            MaxTrackSizingOf::px(S::from_f64(10.0)),
        ),
        TrackComponentOf::minmax(MinTrackSizingOf::px(S::ZERO), MaxTrackSizingOf::MAX_CONTENT),
    ];
    tree.source
        .insert_input(0, LayoutInputOf::box_input(root_style));
    let mut child_style = tree.node_input(1).clone();
    // Explicit minimum0, not computed-scrollability, supplies Minimum0.
    child_style.min_size.width = MinSizeOf::px(S::ZERO);
    child_style.grid_column = GridPlacement::try_lines(1, 3).unwrap();
    child_style.grid_row = GridPlacement::try_lines(1, 2).unwrap();
    tree.source
        .insert_input(1, LayoutInputOf::box_input(child_style));
    if end_marker_in_a {
        tree.source.insert_children(0, [1, 2]);
        tree.source.insert_children(2, []);
        tree.source.insert_input(
            2,
            LayoutInputOf::box_input(NodeInputOf {
                size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
                grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                justify_self: Some(AlignItems::End),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            }),
        );
    }
    // Same actual native leaf algorithm proves content40/80 independently of
    // the Minimum constructor and independently of every Grid result.
    qualify_actual_native_queries(&tree);
    // Actual known BORDER allocation0, with real fixed opposite10 and zeroBP.
    // This may lawfully use the leaf algorithm's complete-known-size shortcut;
    // it is not a claimed callback response or a fabricated ComputeOutput.
    let zero = crate::measurement::compute_tree_leaf(
        &tree,
        1,
        ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(S::ZERO), Some(S::from_f64(10.0))),
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::Grid,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MinContent,
                    ),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        ),
        tree.node_input(1),
    )
    .expect("actual native known-border-zero query succeeds");
    assert_eq!(zero.size, Size::new(S::ZERO, S::from_f64(10.0)));
    // A complete-known query establishes the border allocation; its shortcut
    // does not claim to have observed an intrinsic content extent.
    assert_eq!(zero.baselines(), BaselinesOf::NONE);
    assert_eq!(zero.named_baselines, NamedBaselinesOf::unknown());

    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(S::from_f64(10.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .expect("actual Visible spanning Grid computes without a clipping gate");
    let root = fri08_c01_placement_output(&batch, 0);
    let child = fri08_c01_placement_output(&batch, 1);
    assert_eq!(root.size, Size::new(S::from_f64(40.0), S::from_f64(10.0)));
    assert_eq!(child.size, root.size);
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(child.location, Point::ZERO);
    for output in [root, child] {
        assert_eq!(
            (output.border, output.padding, output.margin),
            (Edges::ZERO, Edges::ZERO, Edges::ZERO)
        );
    }
    if end_marker_in_a {
        let marker = fri08_c01_placement_output(&batch, 2);
        // The real zero item is in A, whose fixed maximum10 is identical with
        // or without this zero single item. It must NOT be put in B, where a
        // genuine zero would change absent infinity into a finite0 limit.
        assert_eq!(marker.location, Point::new(S::from_f64(10.0), S::ZERO));
        assert_eq!(marker.size, Size::ZERO);
        assert_eq!(
            (marker.border, marker.padding, marker.margin),
            (Edges::ZERO, Edges::ZERO, Edges::ZERO)
        );
    }
}
#[test]
fn mixed_span_visible_explicit_zero_minimum_f32() {
    mixed_span_visible_explicit_zero_minimum::<f32>(false);
}
#[test]
fn mixed_span_visible_explicit_zero_minimum_f64() {
    mixed_span_visible_explicit_zero_minimum::<f64>(false);
}
#[test]
fn mixed_span_visible_explicit_zero_minimum_a_end_f32() {
    mixed_span_visible_explicit_zero_minimum::<f32>(true);
}
#[test]
fn mixed_span_visible_explicit_zero_minimum_a_end_f64() {
    mixed_span_visible_explicit_zero_minimum::<f64>(true);
}

#[path = "../tracks_intrinsic/native_maximum_bound.rs"]
mod native_maximum_bound;

#[path = "../tracks_intrinsic/native_content_size_operation.rs"]
mod native_content_size_operation;

#[path = "../tracks_intrinsic/native_preferred_percent_floor.rs"]
mod native_preferred_percent_floor;
