use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn partition<S: LayoutScalar>() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let block = || NodeInputOf::<S> {
        display: Display::Block,
        ..Default::default()
    };
    let cap = || {
        let mut value = block();
        value.max_size.height =
            MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(20.0)).unwrap());
        value
    };
    let mut outer = cap();
    outer.break_inside = FragmentationBreakInside::Avoid;
    let mut first = block();
    first.break_before = FragmentationBreak::Page;
    first.break_after = FragmentationBreak::Column;
    let mut last = block();
    last.break_after = FragmentationBreak::Recto;
    let tree = PublicLayoutTreeOf::new()
        .style(0, block())
        .style(1, outer)
        .style(2, first)
        .style(3, cap())
        .style(4, last)
        .style(5, block())
        .children(0, [1, 5])
        .children(1, [2, 3])
        .children(3, [4]);
    let (graph, _) =
        crate::engine::validation::validate_layout_request_with_text_graph(&tree, 0, &[], &[])
            .unwrap();
    let source = Inventory::new(&tree, &graph, axes).unwrap();
    let scopes = Scopes::new(source, &tree, axes).unwrap();
    scopes.validate().unwrap();
    let keys = |scope| {
        scopes
            .occurrences(ScopeId(scope))
            .into_iter()
            .map(|value| value.key)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keys(0),
        [
            SourceKey::Owner(1),
            SourceKey::PrincipalEnd(1),
            SourceKey::Atomic(5)
        ]
    );
    assert_eq!(
        keys(1),
        [
            SourceKey::Atomic(2),
            SourceKey::Owner(3),
            SourceKey::PrincipalEnd(3),
            SourceKey::BoxEnd(1)
        ]
    );
    assert_eq!(keys(2), [SourceKey::Atomic(4), SourceKey::BoxEnd(3)]);
    let real: Vec<_> = (0..3)
        .flat_map(keys)
        .filter_map(|key| {
            if let SourceKey::Atomic(node) = key {
                Some(node)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(real, [5, 2, 4]);
    let before = scopes.controls(&tree, ScopeId(0), 0).0;
    assert!(
        before
            .iter()
            .any(|value| value.value == FragmentationBreak::Page)
    );
    assert!(
        !scopes
            .controls(&tree, ScopeId(1), 0)
            .0
            .iter()
            .any(|value| value.value == FragmentationBreak::Page)
    );
    assert!(
        scopes
            .controls(&tree, ScopeId(0), 2)
            .1
            .iter()
            .any(|value| value.value == FragmentationBreak::Recto)
    );
    assert!(
        !scopes
            .controls(&tree, ScopeId(1), 4)
            .1
            .iter()
            .any(|value| value.value == FragmentationBreak::Recto)
    );
    assert!(
        scopes
            .controls(&tree, ScopeId(1), 1)
            .1
            .iter()
            .any(|value| value.value == FragmentationBreak::Column)
    );
    assert!(
        !scopes
            .controls(&tree, ScopeId(0), 2)
            .1
            .iter()
            .any(|value| value.value == FragmentationBreak::Column)
    );
    assert!(
        scopes
            .controls(&tree, ScopeId(1), 1)
            .2
            .contains(&FragmentationBreakInside::Avoid)
    );
    let nested = scopes
        .source
        .nodes
        .iter()
        .position(|value| value.node == 3)
        .unwrap();
    assert_eq!(scopes.range(ScopeId(1), nested), Some(1..3));
    assert_eq!(scopes.range(ScopeId(2), nested), Some(0..2));
}
#[test]
fn scope_partition_consumes_children_once_and_routes_outer_controls_f32() {
    partition::<f32>();
}
#[test]
fn scope_partition_consumes_children_once_and_routes_outer_controls_f64() {
    partition::<f64>();
}
