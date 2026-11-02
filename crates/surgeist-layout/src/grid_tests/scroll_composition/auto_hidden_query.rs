// Intended private child of grid_tests/scroll_composition_tests.rs.
// Ordinary Grid only: real measured AUTO leaf, no percentage/subgrid/Lanes.
use super::*;
use crate::LayoutTree;

#[derive(Clone, Copy, Debug)]
enum Tracks {
    Auto,
    MaxContent,
}
impl Tracks {
    fn component<S: LayoutScalar>(self) -> TrackComponentOf<S> {
        match self {
            Self::Auto => TrackComponentOf::AUTO,
            Self::MaxContent => TrackComponentOf::MAX_CONTENT,
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Preferred {
    Auto,
    MaxContent,
}
impl Preferred {
    fn dimension<S: LayoutScalar>(self) -> PreferredSizeOf<S> {
        match self {
            Self::Auto => PreferredSizeOf::AUTO,
            Self::MaxContent => PreferredSizeOf::MAX_CONTENT,
        }
    }
}
fn border<S: LayoutScalar>() -> Size<S> {
    Size::new(S::from_f64(20.0), S::from_f64(30.0))
}
fn natural<S: LayoutScalar>() -> Size<S> {
    Size::new(S::from_f64(80.0), S::from_f64(90.0))
}
fn source<S: LayoutScalar>(
    tracks: Tracks,
    preferred: Preferred,
    overflow: Overflow,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                grid_template_columns: vec![tracks.component()],
                grid_template_rows: vec![tracks.component()],
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(preferred.dimension(), preferred.dimension()),
                max_size: Size::new(
                    MaxSizeOf::px(S::from_f64(20.0)),
                    MaxSizeOf::px(S::from_f64(30.0)),
                ),
                overflow: computed_overflow(overflow, overflow),
                ..NodeInputOf::default()
            },
        )
        .measure(1, natural())
}
fn qualify_leaf<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>, preferred: Preferred) {
    let style = tree.node_input(1);
    assert_eq!(style.size.width, preferred.dimension());
    assert_eq!(style.size.height, preferred.dimension());
    assert!(tree.has_leaf_measurement(1));
    let query = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::Grid,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT),
    );
    // Actual leaf algorithm and SAME host provider, not an authored Grid or
    // ComputeOutput response. AUTO does not take the definite-size shortcut.
    let output = crate::measurement::compute_tree_leaf(tree, 1, query, style)
        .expect("actual native leaf intrinsic query succeeds");
    assert_eq!(output.size, border());
    assert_eq!(output.content_size, natural());
    assert_eq!(output.baselines(), BaselinesOf::NONE);
    assert_eq!(output.named_baselines, NamedBaselinesOf::unknown());
}
fn completed<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(
        tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .expect("actual ordinary Grid completes")
}
fn geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    tracks: Tracks,
    overflow: Overflow,
    preferred: Preferred,
) {
    let root = fri08_c01_placement_output(batch, 0);
    // The MAX_CONTENT root request consumes content contributions even when
    // Hidden makes the child's automatic MINIMUM zero. A sole auto/max-content
    // track still needs the actual20x30 constrained intrinsic border.
    assert_eq!(
        root.size,
        border(),
        "{tracks:?} {overflow:?} {preferred:?}: zero automatic minimum must not erase content contributions"
    );
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(
        (root.border, root.padding, root.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    let child = fri08_c01_placement_output(batch, 1);
    assert_eq!(child.location, Point::ZERO);
    assert_eq!(child.size, border());
    assert_eq!(
        (child.border, child.padding, child.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    if overflow == Overflow::Visible {
        // Only Visible propagation is asserted. Clip/Hidden retain their own
        // owning overflow rules; they do not donate geometry to normal sizing.
        let normal_rect = ScrollRectOf::try_new(Point::ZERO, border()).unwrap();
        let overflow_rect = ScrollRectOf::try_new(Point::ZERO, natural()).unwrap();
        assert_eq!(child.content_size, natural());
        let child_scroll = child.scroll_geometry.expect("actual visible leaf geometry");
        assert_eq!(child_scroll.border_box(), normal_rect);
        assert_eq!(child_scroll.scrollable_overflow(), overflow_rect);
        assert_eq!(root.content_size, natural());
        let root_scroll = root
            .scroll_geometry
            .expect("actual Grid visible-overflow geometry");
        assert_eq!(root_scroll.border_box(), normal_rect);
        assert_eq!(root_scroll.scrollable_overflow(), overflow_rect);
    }
}
fn auto_child_contributes<S: LayoutScalar>(tracks: Tracks, overflow: Overflow) {
    let auto = source::<S>(tracks, Preferred::Auto, overflow);
    qualify_leaf(&auto, Preferred::Auto);
    // Explicit max-content is a separate supported native control. It uses
    // the same natural metric and maxima, not a fixed20x30 preferred-size fix.
    let explicit = source::<S>(tracks, Preferred::MaxContent, overflow);
    qualify_leaf(&explicit, Preferred::MaxContent);
    let prior = completed(&explicit);
    geometry(&prior, tracks, overflow, Preferred::MaxContent);
    let actual = completed(&auto);
    geometry(&actual, tracks, overflow, Preferred::Auto);
    geometry(&prior, tracks, overflow, Preferred::MaxContent);
}
macro_rules! cohort {
    ($name:ident, $scalar:ty, $tracks:expr, $overflow:expr) => {
        #[test]
        fn $name() {
            auto_child_contributes::<$scalar>($tracks, $overflow);
        }
    };
}
cohort!(
    hidden_auto_child_contributes_to_auto_tracks_f32,
    f32,
    Tracks::Auto,
    Overflow::Hidden
);
cohort!(
    hidden_auto_child_contributes_to_auto_tracks_f64,
    f64,
    Tracks::Auto,
    Overflow::Hidden
);
cohort!(
    hidden_auto_child_contributes_to_max_content_tracks_f32,
    f32,
    Tracks::MaxContent,
    Overflow::Hidden
);
cohort!(
    hidden_auto_child_contributes_to_max_content_tracks_f64,
    f64,
    Tracks::MaxContent,
    Overflow::Hidden
);
cohort!(
    visible_auto_child_contributes_to_auto_tracks_f32,
    f32,
    Tracks::Auto,
    Overflow::Visible
);
cohort!(
    visible_auto_child_contributes_to_auto_tracks_f64,
    f64,
    Tracks::Auto,
    Overflow::Visible
);
cohort!(
    clip_auto_child_contributes_to_auto_tracks_f32,
    f32,
    Tracks::Auto,
    Overflow::Clip
);
cohort!(
    clip_auto_child_contributes_to_auto_tracks_f64,
    f64,
    Tracks::Auto,
    Overflow::Clip
);
cohort!(
    visible_auto_child_contributes_to_max_content_tracks_f32,
    f32,
    Tracks::MaxContent,
    Overflow::Visible
);
cohort!(
    visible_auto_child_contributes_to_max_content_tracks_f64,
    f64,
    Tracks::MaxContent,
    Overflow::Visible
);
cohort!(
    clip_auto_child_contributes_to_max_content_tracks_f32,
    f32,
    Tracks::MaxContent,
    Overflow::Clip
);
cohort!(
    clip_auto_child_contributes_to_max_content_tracks_f64,
    f64,
    Tracks::MaxContent,
    Overflow::Clip
);
