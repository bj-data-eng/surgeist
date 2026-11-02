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
        let width = known
            .width
            .unwrap_or_else(|| match input.available_content_size().width {
                MeasurementAvailableOf::MinContent => forty,
                MeasurementAvailableOf::MaxContent => eighty,
                MeasurementAvailableOf::Definite(limit) => limit.get().max(forty).min(eighty),
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
        (AvailableOf::MIN_CONTENT, 40.0),
        (AvailableOf::MAX_CONTENT, 80.0),
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
            Size::new(available, AvailableOf::MAX_CONTENT),
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
                    && actual.available_content_size().width.into_available() == available
            }),
            "real leaf callback must receive this intrinsic query"
        );
    }
    tree.observed.borrow_mut().clear();
}

fn case<S: LayoutScalar>(width: f64, cap: GrowthCap, overflow: Overflow, track: f64) {
    let tree = source::<S>(width, cap, overflow);
    qualify_actual_native_queries(&tree);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(width)),
            AvailableOf::definite(S::from_f64(10.0)),
        ))
        .unwrap(),
    )
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
