use super::output::*;
use crate::output::{CompletedBatchEntries, NormalGeometryOf};
use crate::*;
use std::cell::Cell;

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn point<S: LayoutScalar>(x: f64, y: f64) -> Point<S> {
    Point::new(scalar(x), scalar(y))
}
fn bounds<S: LayoutScalar>(x: f64, y: f64, width: f64, height: f64) -> ScrollRectOf<S> {
    ScrollRectOf::try_new(point(x, y), Size::new(scalar(width), scalar(height))).unwrap()
}
fn empty_batch<S: LayoutScalar>() -> CompletedLayoutBatchOf<usize, S> {
    CompletedLayoutBatchOf::from_entries(
        0,
        CompletedBatchEntries {
            unrounded: vec![],
            final_entries: vec![],
            unrounded_inline: vec![],
            final_inline: vec![],
            stores: vec![],
            clears: vec![],
            invalidated: vec![],
        },
    )
}
fn actual<S: LayoutScalar>(node: usize) -> BoxGeometryPairOf<usize, S> {
    let normal =
        NormalGeometryOf::box_geometry(point(0.25, 0.75), Size::ZERO, bounds(0.0, 0.0, 0.0, 0.0));
    let unrounded = NodeOutputOf {
        location: point(0.25, 0.75),
        ..NodeOutputOf::new()
    };
    let final_output = NodeOutputOf {
        location: point(0.0, 1.0),
        ..NodeOutputOf::new()
    };
    BoxGeometryPairOf {
        unrounded: LayoutOutputEntryOf::ordinary_unrounded(node, normal, unrounded),
        final_entry: LayoutOutputEntryOf::ordinary_canonical(node, normal, final_output),
    }
}
fn record<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<usize, S>,
    index: usize,
    node: usize,
    payload: FragmentPayloadOf<usize, S>,
) -> FragmentOutputOf<usize, S> {
    FragmentOutputOf {
        id: LayoutFragmentId {
            result: batch.result.clone(),
            index,
        },
        node,
        role: BlockGeneration::Principal,
        frame: None,
        parent: Some(FragmentCoordinateSpace::Root),
        order: 0,
        previous: None,
        next: None,
        children: vec![],
        graphical: None,
        payload,
    }
}
fn overflow<S: LayoutScalar>(x: f64) -> FragmentPayloadOf<usize, S> {
    FragmentPayloadOf::OverflowOnly(OverflowGeometryPairOf {
        unrounded: OverflowFragmentGeometryOf {
            origin: point(x, 3.75),
            content_bounds: bounds(x - 2.0, 2.25, 31.5, 27.25),
        },
        final_geometry: OverflowFragmentGeometryOf {
            origin: point(x.round(), 4.0),
            content_bounds: bounds((x - 2.0).round(), 2.0, 32.0, 28.0),
        },
    })
}
fn request<S: LayoutScalar>() -> LayoutBoundRequestOf<S> {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let fallback = OrthogonalFallbackOf::new(
        Size::NONE,
        NearestScrollportOf::Unknown,
        Size::splat(OrthogonalFallbackLimitOf::Unknown),
    );
    let sizing = SizingRequestOf::new(
        Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::Normal,
        )),
        fallback,
    );
    LayoutBoundRequestOf::continuous(
        LayoutSourceBasis::fresh(),
        LayoutRootRequestOf::standalone(sizing, axes, Size::NONE),
    )
}
fn graph<S: LayoutScalar>() -> CompletedLayoutBatchOf<usize, S> {
    let mut batch = empty_batch();
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    batch.fragmentainers = [(10.25, 20.75), (200.5, 100.25)]
        .into_iter()
        .enumerate()
        .map(|(index, (x, y))| LayoutFragmentainerOutputOf {
            id: LayoutFragmentainerId {
                result: batch.result.clone(),
                index,
            },
            kind: FragmentationKind::Page {
                progression: crate::PageProgression::LeftToRight,
            },
            axes,
            frame: BlockFragmentainerOf {
                origin: point(x, y),
                size: Size::new(scalar(60.0), scalar(80.0)),
                breaks: FragmentationBreakKinds::PAGE,
                page_side: None,
            },
        })
        .collect();
    let frame = |index: usize| batch.fragmentainers[index].id.clone();
    let mut root = record(&batch, 0, 0, FragmentPayloadOf::ActualBox(actual(0)));
    root.frame = Some(frame(0));
    root.parent = Some(FragmentCoordinateSpace::Fragmentainer(frame(0)));
    root.children = vec![(SourceIndex::ZERO, 1)];
    let mut carrier = record(&batch, 1, 0, overflow(7.25));
    carrier.frame = Some(frame(1));
    carrier.parent = Some(FragmentCoordinateSpace::Fragmentainer(frame(1)));
    carrier.order = 1;
    carrier.previous = Some(root.id.clone());
    root.next = Some(carrier.id.clone());
    carrier.children = vec![(SourceIndex::ZERO, 1)];
    let mut ancestor = record(&batch, 2, 1, FragmentPayloadOf::ActualBox(actual(1)));
    ancestor.frame = Some(frame(0));
    ancestor.parent = Some(FragmentCoordinateSpace::Fragment(root.id.clone()));
    let mut ancestor_carrier = record(&batch, 3, 1, overflow(4.5));
    ancestor_carrier.frame = Some(frame(1));
    ancestor_carrier.parent = Some(FragmentCoordinateSpace::Fragment(carrier.id.clone()));
    ancestor_carrier.order = 1;
    ancestor_carrier.previous = Some(ancestor.id.clone());
    ancestor.next = Some(ancestor_carrier.id.clone());
    ancestor_carrier.children = vec![(SourceIndex::ZERO, 2)];
    let mut leaf = record(&batch, 4, 2, FragmentPayloadOf::ActualBox(actual(2)));
    leaf.frame = Some(frame(1));
    leaf.parent = Some(FragmentCoordinateSpace::Fragment(
        ancestor_carrier.id.clone(),
    ));
    batch.nodes = vec![
        NodePublication::Fragments {
            node: 0,
            fragments: vec![root, carrier],
        },
        NodePublication::Fragments {
            node: 1,
            fragments: vec![ancestor, ancestor_carrier],
        },
        NodePublication::One(leaf),
    ];
    batch.target = Some(request::<S>().target_basis().clone());
    batch
}

fn singular_custody<S: LayoutScalar>() {
    let pair = actual::<S>(0);
    let batch = CompletedLayoutBatchOf::from_entries(
        0,
        CompletedBatchEntries {
            unrounded: vec![pair.unrounded],
            final_entries: vec![pair.final_entry],
            unrounded_inline: vec![],
            final_inline: vec![],
            stores: vec![],
            clears: vec![],
            invalidated: vec![],
        },
    );
    let single = batch.node_layout(0).unwrap().single_box().unwrap();
    assert_eq!(single.unrounded_entry(), pair.unrounded);
    assert_eq!(single.final_entry(), pair.final_entry);
    let fragment = batch.fragment(single.id()).unwrap();
    assert_eq!(fragment.fragmentainer(), None);
    assert_eq!(
        fragment.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Root)
    );
    assert!(matches!(
        fragment.payload(),
        FragmentPayloadOf::ActualBox(_)
    ));
    assert!(fragment.overflow_only().is_none());
    let geometry = fragment.principal_box().unwrap();
    assert_eq!(geometry.unrounded().size, Size::ZERO);
    assert_eq!(geometry.final_geometry().size, Size::ZERO);
    assert_eq!(geometry.unrounded().location, point(0.25, 0.75));
    assert_eq!(geometry.final_geometry().location, point(0.0, 1.0));
    assert_eq!(
        geometry.unrounded(),
        BoxFragmentGeometryOf::from_packet(single.unrounded_output())
    );
    assert_eq!(
        geometry.final_geometry(),
        BoxFragmentGeometryOf::from_packet(single.final_output())
    );
}

fn overflow_graph<S: LayoutScalar>() {
    let batch = graph::<S>();
    let roots = batch.node_layout(0).unwrap().fragments().unwrap();
    assert_eq!(roots.len(), 2);
    assert!(batch.node_layout(0).unwrap().single_box().is_none());
    let carrier = &roots[1];
    assert!(carrier.principal_box().is_none());
    assert!(matches!(
        carrier.payload(),
        FragmentPayloadOf::OverflowOnly(_)
    ));
    let geometry = carrier.overflow_only().unwrap();
    assert_eq!(geometry.unrounded().origin, point(7.25, 3.75));
    assert_eq!(geometry.final_geometry().origin, point(7.0, 4.0));
    assert_eq!(
        geometry.unrounded().content_bounds,
        bounds(5.25, 2.25, 31.5, 27.25)
    );
    assert_eq!(
        geometry.final_geometry().content_bounds,
        bounds(5.0, 2.0, 32.0, 28.0)
    );
    assert_eq!(carrier.previous(), Some(roots[0].id()));
    assert_eq!(
        batch.fragment(carrier.previous().unwrap()).unwrap().next(),
        Some(carrier.id())
    );
    assert_eq!(carrier.continuation_order(), 1);
    assert_eq!(carrier.source_children(), &[(SourceIndex::ZERO, 1)]);
    let children: Vec<_> = batch.child_fragments(carrier.id()).unwrap().collect();
    assert_eq!(children.len(), 1);
    let ancestor = children[0];
    assert!(ancestor.overflow_only().is_some());
    assert_eq!(ancestor.node(), 1);
    assert_eq!(ancestor.fragmentainer(), carrier.fragmentainer());
    let leaves: Vec<_> = batch.child_fragments(ancestor.id()).unwrap().collect();
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].node(), 2);
    assert!(leaves[0].principal_box().is_some());
    assert_eq!(leaves[0].fragmentainer(), carrier.fragmentainer());
    assert_eq!(
        batch
            .child_fragments(roots[0].id())
            .unwrap()
            .map(FragmentOutputOf::node)
            .collect::<Vec<_>>(),
        vec![1]
    );
    let fragments: Vec<_> = batch.fragments().collect();
    for (index, fragment) in fragments.iter().enumerate() {
        assert!(
            !fragments[..index]
                .iter()
                .any(|prior| prior.id() == fragment.id())
        );
        if let Some(previous) = fragment.previous() {
            let previous = batch.fragment(previous).unwrap();
            assert_eq!(previous.next(), Some(fragment.id()));
            assert_eq!(previous.node(), fragment.node());
        }
        if let Some(FragmentCoordinateSpace::Fragment(parent)) = fragment.coordinate_parent() {
            assert_eq!(
                batch.fragment(parent).unwrap().fragmentainer(),
                fragment.fragmentainer()
            );
        }
    }
    assert_eq!(batch.fragments().count(), 5);
    let frame = batch
        .fragmentainer(carrier.fragmentainer().unwrap())
        .unwrap();
    assert_eq!(frame.origin(), point(200.5, 100.25));
    let mut only_overflow = empty_batch::<S>();
    let only = record(&only_overflow, 0, 0, overflow(7.25));
    only_overflow.nodes.push(NodePublication::One(only));
    assert!(only_overflow.node_layout(0).unwrap().single_box().is_none());
    assert_eq!(
        only_overflow
            .node_layout(0)
            .unwrap()
            .fragments()
            .unwrap()
            .len(),
        1
    );
    assert!(
        only_overflow
            .fragments()
            .next()
            .unwrap()
            .principal_box()
            .is_none()
    );
    let other = graph::<S>();
    assert!(matches!(
        other.fragment(carrier.id()),
        Err(LayoutHandleError::WrongResult)
    ));
    assert!(matches!(
        other.child_fragments(carrier.id()),
        Err(LayoutHandleError::WrongResult)
    ));
    let missing = LayoutFragmentId {
        result: batch.result.clone(),
        index: 99,
    };
    assert!(matches!(
        batch.fragment(&missing),
        Err(LayoutHandleError::MissingFragment)
    ));
    assert!(matches!(
        batch.child_fragments(&missing),
        Err(LayoutHandleError::MissingFragment)
    ));
    let retained = batch.clone();
    assert_eq!(retained.fragment(carrier.id()).unwrap().id(), carrier.id());
}

struct Sink<S: LayoutScalar> {
    targets: LayoutPublicationTargets<usize>,
    visible: Vec<FragmentOutputOf<usize, S>>,
    reject: Cell<bool>,
    commits: usize,
}
impl<S: LayoutScalar> LayoutBatchSink<usize, S> for Sink<S> {
    type Error = &'static str;
    type Prepared = Vec<FragmentOutputOf<usize, S>>;
    fn publication_targets(&self) -> &LayoutPublicationTargets<usize> {
        &self.targets
    }
    fn publication_targets_mut(&mut self) -> &mut LayoutPublicationTargets<usize> {
        &mut self.targets
    }
    fn prepare_layout_batch(
        &self,
        preparation: LayoutBatchPreparation<'_, usize, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut next = self.visible.clone();
        next.retain(|fragment| !preparation.previous_members().contains(&fragment.node()));
        next.extend(preparation.batch().fragments().cloned());
        if self.reject.get() {
            Err("rejected")
        } else {
            Ok(next)
        }
    }
    fn commit_layout_batch(&mut self, commit: LayoutBatchCommit<'_, usize, S, Self::Prepared>) {
        self.visible = commit.into_prepared();
        self.commits += 1;
    }
}
fn atomic_replacement<S: LayoutScalar>() {
    let batch = graph::<S>();
    let old = batch.node_layout(0).unwrap().fragments().unwrap()[1]
        .id()
        .clone();
    let mut sink = Sink {
        targets: LayoutPublicationTargets::new(),
        visible: vec![],
        reject: Cell::new(false),
        commits: 0,
    };
    sink.targets
        .install(0, batch.target_basis().unwrap().clone())
        .unwrap();
    batch.apply_to(&mut sink).unwrap();
    assert_eq!(
        sink.visible
            .iter()
            .filter(|f| f.overflow_only().is_some())
            .count(),
        2
    );
    let mut replacement = empty_batch::<S>();
    replacement.nodes.push(NodePublication::One(record(
        &replacement,
        0,
        0,
        FragmentPayloadOf::ActualBox(actual(0)),
    )));
    replacement.target = Some(request::<S>().target_basis().clone());
    sink.targets
        .install(0, replacement.target_basis().unwrap().clone())
        .unwrap();
    sink.reject.set(true);
    assert_eq!(
        replacement.apply_to(&mut sink),
        Err(LayoutBatchApplyError::Preparation("rejected"))
    );
    assert_eq!(sink.commits, 1);
    assert!(sink.visible.iter().any(|f| f.id() == &old));
    sink.reject.set(false);
    replacement.apply_to(&mut sink).unwrap();
    assert_eq!(sink.commits, 2);
    assert_eq!(sink.visible.len(), 1);
    assert!(sink.visible[0].principal_box().is_some());
    assert_eq!(sink.targets.published_members(0), &[0]);
    assert!(matches!(
        replacement.fragment(&old),
        Err(LayoutHandleError::WrongResult)
    ));
    assert_eq!(
        batch.apply_to(&mut sink),
        Err(LayoutBatchApplyError::StaleSource)
    );
    assert_eq!(sink.commits, 2);
    assert_eq!(sink.visible.len(), 1);
}

#[test]
fn zero_box_retains_singular_rounding_custody_f32() {
    singular_custody::<f32>();
}
#[test]
fn zero_box_retains_singular_rounding_custody_f64() {
    singular_custody::<f64>();
}
#[test]
fn overflow_payload_resolves_two_ancestor_levels_f32() {
    overflow_graph::<f32>();
}
#[test]
fn overflow_payload_resolves_two_ancestor_levels_f64() {
    overflow_graph::<f64>();
}
#[test]
fn failed_preparation_preserves_carriers_then_replacement_removes_them_f32() {
    atomic_replacement::<f32>();
}
#[test]
fn failed_preparation_preserves_carriers_then_replacement_removes_them_f64() {
    atomic_replacement::<f64>();
}
