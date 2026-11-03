//! Cyclic percentage tracks resolve against the intrinsically chosen container.
use super::*;
use crate::engine::contracts::Compute;

fn percent_subject<S: LayoutScalar>(preferred: PreferredSizeOf<S>) -> DistinctLeaf<S> {
    let mut tree = source::<S>(100.0, GrowthCap::MaxContent, Overflow::Clip);
    let mut root = tree.node_input(0).clone();
    root.display = Display::InlineGrid;
    root.size = Size::new(preferred, PreferredSizeOf::AUTO);
    root.grid_template_columns = vec![TrackComponentOf::percent(S::ONE)];
    root.grid_template_rows = vec![TrackComponentOf::AUTO];
    tree.source.insert_input(0, LayoutInputOf::box_input(root));
    let mut child = tree.node_input(1).clone();
    child.size.height = PreferredSizeOf::AUTO;
    tree.source.insert_input(1, LayoutInputOf::box_input(child));
    tree
}

fn intrinsic_preference_owns_percent_basis<S: LayoutScalar>(
    preferred: PreferredSizeOf<S>,
    expected: f64,
) {
    let tree = percent_subject::<S>(preferred);
    qualify_actual_native_queries(&tree);
    let available = Size::new(
        AvailableOf::definite(S::from_f64(100.0)),
        AvailableOf::MAX_CONTENT,
    );
    let input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(S::from_f64(100.0)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        available,
    );
    assert_eq!(input.known(), Size::NONE);
    assert_eq!(input.available(), available);
    let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let intrinsic = session
        .compute_child(0, input)
        .expect("intrinsic preferred Grid size is admitted with an external finite host");
    let expected_size = Size::new(S::from_f64(expected), S::from_f64(10.0));
    assert_eq!(
        intrinsic.size, expected_size,
        "ComputeSize uses the authored intrinsic preference, not external available100"
    );
    drop(session);

    // Grid 2 sections 7.2 and 12: treat the cyclic100% track as Auto to size the
    // container, then resolve it against that resulting40/80 container size.
    // The external100 host is not the percentage track's containing grid.
    let batch = compute_layout(&tree, 0, LayoutRootRequestOf::viewport(available).unwrap())
        .expect("normal layout of the intrinsically preferred InlineGrid is admitted");
    for (phase, entries) in [
        ("unrounded", batch.unrounded_entries()),
        ("final", batch.final_entries()),
    ] {
        for node in [0, 1] {
            let output = entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap_or_else(|| panic!("{phase} node{node}: missing actual layout"))
                .output();
            assert_eq!(
                output.size, expected_size,
                "{phase} node{node}: actual normal container and100% track use intrinsic{expected}"
            );
            assert_eq!(
                output.location,
                Point::ZERO,
                "{phase} node{node}: normal placement"
            );
            assert_eq!(
                output.content_box_size(),
                expected_size,
                "{phase} node{node}: zero-edge content box"
            );
            assert_eq!(
                (output.border, output.padding, output.margin),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO),
                "{phase} node{node}: source has no box edges"
            );
            assert_eq!(
                output.source_index,
                SourceIndex::ZERO,
                "{phase} node{node}: actual sole child rank"
            );
        }
    }
}

#[test]
fn preferred_min_grid_does_not_floor_cyclic_percent_track_to_external_host_f32() {
    intrinsic_preference_owns_percent_basis::<f32>(PreferredSizeOf::MIN_CONTENT, 40.0);
}
#[test]
fn preferred_min_grid_does_not_floor_cyclic_percent_track_to_external_host_f64() {
    intrinsic_preference_owns_percent_basis::<f64>(PreferredSizeOf::MIN_CONTENT, 40.0);
}
#[test]
fn preferred_max_grid_does_not_floor_cyclic_percent_track_to_external_host_f32() {
    intrinsic_preference_owns_percent_basis::<f32>(PreferredSizeOf::MAX_CONTENT, 80.0);
}
#[test]
fn preferred_max_grid_does_not_floor_cyclic_percent_track_to_external_host_f64() {
    intrinsic_preference_owns_percent_basis::<f64>(PreferredSizeOf::MAX_CONTENT, 80.0);
}
