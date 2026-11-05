//! Content-size queries suppress the Grid's own authored preferred size.
use super::*;
use crate::engine::contracts::Compute;

fn content_subject<S: LayoutScalar>(preferred: PreferredSizeOf<S>) -> DistinctLeaf<S> {
    let mut tree = source::<S>(20.0, GrowthCap::MaxContent, Overflow::Hidden);
    let mut root = tree.node_input(0).clone();
    root.display = Display::InlineGrid;
    root.size = Size::new(preferred, PreferredSizeOf::AUTO);
    root.grid_template_rows = vec![TrackComponentOf::AUTO];
    tree.source.insert_input(0, LayoutInputOf::box_input(root));
    let mut child = tree.node_input(1).clone();
    child.size.height = PreferredSizeOf::AUTO;
    tree.source.insert_input(1, LayoutInputOf::box_input(child));
    tree
}

fn query_content_subject<S: LayoutScalar>(
    tree: &DistinctLeaf<S>,
    mode: SizingMode,
    width: AxisSizingRequestOf<S>,
) -> ComputeOutputOf<S> {
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        mode,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        SizingRequestOf::new(
            Size::new(
                width,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    assert_eq!(input.known(), Size::NONE);
    assert_eq!(input.parent(), Size::NONE);
    let mut session = crate::engine::ComputeSession::new(tree, Vec::new());
    session
        .compute_child(0, input)
        .expect("the native Grid query completes")
}

fn content_operation<S: LayoutScalar>(
    preferred: PreferredSizeOf<S>,
    operation: AxisSizingRequestOf<S>,
    authored_width: f64,
    issued_width: f64,
) {
    let tree = content_subject(preferred);
    qualify_actual_native_queries(&tree);
    let minimum = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(S::ZERO), None),
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::Grid,
        ),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    let zero =
        crate::measurement::compute_tree_leaf(&tree, 1, minimum, tree.node_input(1)).unwrap();
    assert_eq!(zero.size, Size::new(S::ZERO, S::from_f64(10.0)));
    assert!(
        tree.observed
            .borrow()
            .iter()
            .any(|input| input.known_content_size() == Size::new(Some(S::ZERO), None)),
        "the same real leaf executes the zero-minimum input"
    );

    let inherent = query_content_subject(&tree, SizingMode::InherentSize, operation);
    assert_eq!(
        inherent.size,
        Size::new(S::from_f64(authored_width), S::from_f64(10.0)),
        "InherentSize preserves the actual authored intrinsic preferred width"
    );
    let automatic = content_subject::<S>(PreferredSizeOf::AUTO);
    let automatic = query_content_subject(&automatic, SizingMode::ContentSize, operation);
    assert_eq!(
        automatic.size,
        Size::new(S::from_f64(issued_width), S::from_f64(10.0)),
        "ContentSize with Auto preferred width respects the issued operation"
    );

    let content = query_content_subject(&tree, SizingMode::ContentSize, operation);
    assert_eq!(
        content.size,
        Size::new(S::from_f64(issued_width), S::from_f64(10.0)),
        "ContentSize suppresses authored preferred width instead of restoring its operation"
    );
}

#[test]
fn grid_content_size_max_ignores_authored_min_preference_f32() {
    content_operation::<f32>(
        PreferredSizeOf::MIN_CONTENT,
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        40.0,
        80.0,
    );
}
#[test]
fn grid_content_size_max_ignores_authored_min_preference_f64() {
    content_operation::<f64>(
        PreferredSizeOf::MIN_CONTENT,
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        40.0,
        80.0,
    );
}
#[test]
fn grid_content_size_min_ignores_authored_max_preference_f32() {
    content_operation::<f32>(
        PreferredSizeOf::MAX_CONTENT,
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
        80.0,
        40.0,
    );
}
#[test]
fn grid_content_size_min_ignores_authored_max_preference_f64() {
    content_operation::<f64>(
        PreferredSizeOf::MAX_CONTENT,
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
        80.0,
        40.0,
    );
}
