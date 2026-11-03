use super::*;

pub(super) fn empty_pair<S: LayoutScalar>(
    start_edge: InlineBoundaryEdgeOf<S>,
    end_edge: InlineBoundaryEdgeOf<S>,
) -> (NumericTree<S>, usize, usize) {
    let mut tree = NumericTree::<S>::new(60.0);
    let actual = strut(8.0, 10.0, LineBoxAlignment::Baseline);
    let start = tree.boundary(InlineBoundaryKind::Start, actual.clone());
    let end = tree.boundary(InlineBoundaryKind::End, actual);
    tree.associate(start, end, None);
    let parent = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    assert_eq!(
        FlowAxes::new(
            tree.node_input(0).writing_mode,
            tree.node_input(0).direction
        ),
        parent
    );
    for (node, edge) in [(start, start_edge), (end, end_edge)] {
        let input = tree.layout_input(node).as_inline_boundary().unwrap();
        assert_eq!(input.metrics().baseline(), S::from_f64(8.0));
        assert_eq!(input.metrics().line_extent(), S::from_f64(10.0));
        tree.inputs[node] = LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
            input.kind(),
            input.metrics(),
            edge,
            parent,
        ));
    }
    assert!(tree.atomics.is_empty());
    assert_eq!(tree.children, [start, end]);
    (tree, start, end)
}
