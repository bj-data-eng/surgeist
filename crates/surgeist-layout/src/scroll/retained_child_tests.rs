use super::{
    CanonicalScrollRangeSeedPolicy, RetainedChildReconstruction, RetainedChildScrollSourceOf,
    ScrollBoxProjection, ScrollTargetProjection, retained_child_scroll_geometry,
};
use crate::{
    geometry::{Edges, Point, Size},
    node_input::{ComputedOverflow, NodeInputOf, Overflow, ScrollbarWidthOf},
    scalar::LayoutScalar,
    scroll::ScrollGeometryOf,
};

const INCLUDE: RetainedChildReconstruction =
    RetainedChildReconstruction::DirectLine(CanonicalScrollRangeSeedPolicy::IncludeReservedGutter);
const EXCLUDE: RetainedChildReconstruction =
    RetainedChildReconstruction::DirectLine(CanonicalScrollRangeSeedPolicy::ExcludeReservedGutter);
const MEASURED_LEAF: RetainedChildReconstruction = RetainedChildReconstruction::MeasuredLeaf;
const ALL_RECONSTRUCTIONS: [RetainedChildReconstruction; 3] = [INCLUDE, EXCLUDE, MEASURED_LEAF];

fn style<S: LayoutScalar>(x: Overflow, y: Overflow) -> NodeInputOf<S> {
    NodeInputOf::<S> {
        overflow: ComputedOverflow::try_new(x, y).expect("canonical overflow pair"),
        scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(15.0))
            .expect("finite scrollbar width"),
        ..NodeInputOf::default()
    }
}

fn border_box<S: LayoutScalar>() -> Size<S> {
    Size::new(S::from_f64(100.0), S::from_f64(80.0))
}

fn reconstruct<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    reconstruction: RetainedChildReconstruction,
) -> ScrollGeometryOf<S> {
    // Content smaller than the scrollport leaves the container seed as the
    // only source of range beyond the scrollport.
    let content_size = Size::new(S::from_f64(50.0), S::from_f64(40.0));
    retain(
        style,
        reconstruction,
        content_size,
        Edges::ZERO,
        Edges::ZERO,
        None,
    )
}

fn retain<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    reconstruction: RetainedChildReconstruction,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<ScrollGeometryOf<S>>,
) -> ScrollGeometryOf<S> {
    retained_child_scroll_geometry(
        RetainedChildScrollSourceOf {
            box_projection: ScrollBoxProjection::from_node(style),
            target_projection: ScrollTargetProjection::from_node(style),
            size: border_box(),
            content_size,
            padding,
            border,
            child_compute_geometry,
        },
        reconstruction,
    )
    .expect("finite child retains scroll geometry")
}

fn range_maxima<S: LayoutScalar>(geometry: ScrollGeometryOf<S>) -> (S, S) {
    let range = geometry.physical_range();
    (range.x().maximum(), range.y().maximum())
}

fn assert_reconstruction_ranges<S: LayoutScalar>() {
    let zero = S::ZERO;
    let gutter = S::from_f64(15.0);

    // Both axes scroll: the vertical and horizontal gutters are both reserved,
    // so the scrollport is 85x65 inside the 100x80 padding box.
    let both = style::<S>(Overflow::Scroll, Overflow::Scroll);
    assert_eq!(
        reconstruct(&both, INCLUDE).scrollport().size(),
        Size::new(S::from_f64(85.0), S::from_f64(65.0))
    );
    // A range seeded with the padding box scrolls by the gutter thickness; one
    // seeded with the scrollport has nothing to scroll.
    assert_eq!(range_maxima(reconstruct(&both, INCLUDE)), (gutter, gutter));
    assert_eq!(range_maxima(reconstruct(&both, EXCLUDE)), (zero, zero));
    // The measured-leaf path excludes the gutter on each scroll-container axis.
    assert_eq!(
        range_maxima(reconstruct(&both, MEASURED_LEAF)),
        (zero, zero)
    );

    // Only the horizontal axis scrolls: the horizontal scrollbar reserves the
    // block-end gutter and the 100x65 scrollport sits in a 100x80 padding box.
    // The vertical axis is not a scrolling axis, so the measured-leaf path
    // keeps the padding box as its vertical seed, while the direct-line
    // exclusion policy narrows both axes to the scrollport.
    let horizontal = style::<S>(Overflow::Scroll, Overflow::Hidden);
    assert_eq!(
        reconstruct(&horizontal, EXCLUDE).scrollport().size(),
        Size::new(S::from_f64(100.0), S::from_f64(65.0))
    );
    assert_eq!(
        range_maxima(reconstruct(&horizontal, EXCLUDE)),
        (zero, zero)
    );
    assert_eq!(
        range_maxima(reconstruct(&horizontal, INCLUDE)),
        (zero, gutter)
    );
    assert_eq!(
        range_maxima(reconstruct(&horizontal, MEASURED_LEAF)),
        (zero, gutter)
    );

    for reconstruction in ALL_RECONSTRUCTIONS {
        let geometry = reconstruct(&both, reconstruction);
        assert_eq!(geometry.border_box().size(), border_box::<S>());
        assert_eq!(geometry.border_box().origin(), Point::ZERO);
    }
}

#[test]
fn reconstructed_child_range_follows_the_selected_reconstruction() {
    assert_reconstruction_ranges::<f32>();
    assert_reconstruction_ranges::<f64>();
}

fn assert_matching_geometry_is_retained<S: LayoutScalar>() {
    let style = style::<S>(Overflow::Scroll, Overflow::Scroll);
    let large_content = Size::new(S::from_f64(500.0), S::from_f64(500.0));

    for reconstruction in ALL_RECONSTRUCTIONS {
        let computed = reconstruct(&style, reconstruction);
        let retained = retain(
            &style,
            reconstruction,
            large_content,
            Edges::ZERO,
            Edges::ZERO,
            Some(computed),
        );

        assert_eq!(retained, computed);
    }
}

#[test]
fn matching_computed_child_geometry_is_retained_under_every_reconstruction() {
    assert_matching_geometry_is_retained::<f32>();
    assert_matching_geometry_is_retained::<f64>();
}

fn assert_changed_box_facts_are_not_reused<S: LayoutScalar>() {
    let style = style::<S>(Overflow::Scroll, Overflow::Scroll);
    let content_size = Size::new(S::from_f64(50.0), S::from_f64(40.0));
    let five = S::from_f64(5.0);
    let seven = S::from_f64(7.0);
    let border = Edges::new(five, five, five, five);
    let padding = Edges::new(seven, seven, seven, seven);

    for reconstruction in ALL_RECONSTRUCTIONS {
        let computed = reconstruct(&style, reconstruction);

        // The same border-box size with a new border moves the padding box.
        let bordered = retain(
            &style,
            reconstruction,
            content_size,
            Edges::ZERO,
            border,
            Some(computed),
        );
        assert_ne!(bordered, computed);
        assert_eq!(bordered.padding_box().origin(), Point::new(five, five));
        assert_eq!(
            bordered.padding_box().size(),
            Size::new(S::from_f64(90.0), S::from_f64(70.0))
        );

        // The same border-box size with new padding moves the content box.
        let padded = retain(
            &style,
            reconstruction,
            content_size,
            padding,
            Edges::ZERO,
            Some(computed),
        );
        assert_ne!(padded, computed);
        assert_eq!(padded.padding_box(), computed.padding_box());
        assert_eq!(padded.content_box().origin(), Point::new(seven, seven));
    }
}

#[test]
fn computed_child_geometry_with_different_border_or_padding_is_rebuilt() {
    assert_changed_box_facts_are_not_reused::<f32>();
    assert_changed_box_facts_are_not_reused::<f64>();
}
