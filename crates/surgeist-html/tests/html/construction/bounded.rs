mod depth;
mod focus;
mod order;
mod refusal;
mod replace;
use crate::html::support::{Cause, cause, charged_by, has_cause, slots};
use std::sync::Arc;
use surgeist_html::{
    accounting::HtmlWork,
    bounded::construction::{
        HtmlConstructionActionDescription, HtmlConstructionAllocationRole,
        HtmlConstructionAttributeEdit, HtmlConstructionAttributeEditMode, HtmlConstructionCommand,
        HtmlConstructionCommit, HtmlConstructionContent, HtmlConstructionGrowth,
        HtmlConstructionLease, HtmlConstructionNode, HtmlConstructionReadyOperation,
        HtmlConstructionRefusal, HtmlConstructionResult, HtmlConstructionRetirementDescription,
        HtmlConstructionStorage, HtmlConstructionTextAppend,
    },
    construction::{
        HtmlBuilderNode, HtmlConstructionCause, HtmlDocumentMode, HtmlElement, HtmlFragmentOwner,
        HtmlNodeKind, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
        HtmlTemplateContentsKind,
    },
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

fn storage() -> HtmlConstructionStorage {
    let mut owner = Some(HtmlConstructionStorage::begin());
    while let Some(action) = owner.as_mut().unwrap().next_action().unwrap() {
        assert!(matches!(
            action.description(),
            HtmlConstructionActionDescription::InitializeIdentity { .. }
        ));
        let observation = action.perform();
        assert_eq!(observation.allocation().capacity(), 1);
        observation.accept();
    }
    slots::initialization(&mut owner).unwrap().commit()
}
fn ready(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> HtmlConstructionReadyOperation {
    let mut operation = Some(storage.prepare(command));
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        action.perform().unwrap().accept();
    }
    let ready = slots::operation(&mut operation).unwrap();
    assert!(operation.is_none());
    ready
}
fn change(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
    match ready(storage, command).commit().unwrap() {
        HtmlConstructionCommit::Changed { storage, created } => (storage, created),
        _ => panic!("mutation produced publication"),
    }
}
fn create(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let (storage, created) = change(storage, command);
    (storage, created.unwrap())
}
fn element(local: &str) -> HtmlConstructionCommand {
    HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(
            QualifiedName::new(Namespace::html(), None, local),
            OrderedAttributes::default(),
        )),
    }
}
fn fragment(storage: HtmlConstructionStorage) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(storage, HtmlConstructionCommand::CreateFragment)
}
fn properties() -> HtmlShadowRootProperties {
    HtmlShadowRootProperties::new(
        HtmlShadowRootMode::Closed,
        HtmlSlotAssignment::Manual,
        true,
        true,
        true,
    )
}
fn publish(
    storage: HtmlConstructionStorage,
    root: HtmlBuilderNode,
) -> (HtmlConstructionResult, HtmlConstructionLease) {
    match ready(storage, HtmlConstructionCommand::Publish { root })
        .commit()
        .unwrap()
    {
        HtmlConstructionCommit::Published { result, lease } => (result, lease),
        _ => panic!("publication produced mutation"),
    }
}
fn refuse(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    expected: Cause,
) -> HtmlConstructionStorage {
    let mut owner = Some(storage.prepare(command));
    loop {
        match owner.as_mut().unwrap().next_action() {
            Ok(Some(action)) => match action.perform() {
                Ok(observation) => observation.accept(),
                Err(failure) => {
                    assert_eq!(failure.into_parts().0, expected);
                    break;
                }
            },
            Ok(None) => panic!("invalid command became ready"),
            Err(error) => {
                assert_eq!(error, expected);
                break;
            }
        }
    }
    assert!(has_cause(
        owner.as_mut().unwrap().next_action().err().as_ref(),
        expected
    ));
    assert!(matches!(slots::operation(&mut owner), Err(error) if error == expected));
    owner.unwrap().abort().unwrap().into_parts().0
}

/// What the actions of one driven operation describe, excluding the
/// capacity-growth actions an amortizing consumer pays only occasionally.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DescribedWork {
    actions: usize,
    record_visits: usize,
    copied_bytes: usize,
    record_writes: usize,
}

/// Inserts `node` under `parent` before `before`, performing every action, and
/// returns the steady work the operation described.
fn driven_insert(
    storage: HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
    before: Option<&HtmlBuilderNode>,
) -> (HtmlConstructionStorage, DescribedWork) {
    driven(
        storage,
        HtmlConstructionCommand::Insert {
            node: node.clone(),
            parent: parent.clone(),
            before: before.cloned(),
        },
    )
}

/// Performs every action of `command` and returns the steady work it described.
fn driven(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, DescribedWork) {
    let mut operation = Some(storage.prepare(command));
    let mut work = DescribedWork::default();
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        let description = action.description();
        let (visits, copied, writes, growth) = match &description {
            HtmlConstructionActionDescription::Inspect { .. } => {
                let charged = charged_by(|meter| description.charge_inspection(meter));
                (charged.total(HtmlWork::RecordVisit) as usize, 0, 0, false)
            }
            HtmlConstructionActionDescription::Commit { .. } => {
                let charged = charged_by(|meter| description.charge_commit(meter));
                (
                    charged.total(HtmlWork::RecordVisit) as usize,
                    charged.total(HtmlWork::CopyByte) as usize,
                    charged.total(HtmlWork::RecordWrite) as usize,
                    false,
                )
            }
            // Reserving and reporting a bank's capacity is amortized growth.
            HtmlConstructionActionDescription::Reserve { .. }
            | HtmlConstructionActionDescription::PrepareAllocationReport { .. } => (0, 0, 0, true),
            _ => (0, 0, 0, false),
        };
        if !growth {
            work.actions += 1;
            work.record_visits += visits;
            work.copied_bytes += copied;
            work.record_writes += writes;
        }
        action.perform().unwrap().accept();
    }
    let ready = slots::operation(&mut operation).unwrap();
    let HtmlConstructionCommit::Changed { storage, .. } = ready.commit().unwrap() else {
        panic!("mutation produced publication")
    };
    (storage, work)
}

#[test]
fn appending_a_child_describes_the_same_work_at_every_fanout() {
    let (mut owner, root) = fragment(storage());
    let mut children = 0;
    let mut measured = Vec::new();
    for fanout in [1, 16, 128, 1024] {
        while children < fanout {
            let (next, node) = create(owner, element("c"));
            owner = driven_insert(next, &node, &root, None).0;
            children += 1;
        }
        let (next, node) = create(owner, element("measured"));
        let (next, work) = driven_insert(next, &node, &root, None);
        owner = next;
        children += 1;
        measured.push((fanout, work));
    }
    let (first_fanout, first) = measured[0];
    for (fanout, work) in &measured {
        assert_eq!(
            *work, first,
            "appending under {fanout} children described different work than under {first_fanout}"
        );
    }
}

/// Appends `node` last under `parent`, admitting each growth action with a
/// doubled capacity, and returns the slots the growth requested.
fn append_amortized(
    storage: HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
) -> (HtmlConstructionStorage, Vec<usize>) {
    let mut operation = Some(storage.prepare(HtmlConstructionCommand::Insert {
        node: node.clone(),
        parent: parent.clone(),
        before: None,
    }));
    let mut requested = Vec::new();
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        let capacity = match action.description() {
            HtmlConstructionActionDescription::Reserve {
                allocation,
                minimum_capacity,
                growth: HtmlConstructionGrowth::Geometric,
                ..
            } => Some(minimum_capacity.max(2 * allocation.capacity())),
            _ => None,
        };
        let observation = match capacity {
            Some(capacity) => {
                requested.push(capacity);
                action.perform_with_capacity(capacity).unwrap()
            }
            None => action.perform().unwrap(),
        };
        observation.accept();
    }
    let HtmlConstructionCommit::Changed { storage, .. } =
        slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!("mutation produced publication")
    };
    (storage, requested)
}

#[test]
fn amortized_appends_grow_the_child_order_geometrically_and_keep_their_order() {
    let (mut owner, root) = fragment(storage());
    let mut appended = Vec::new();
    let mut growth = Vec::new();
    for _ in 0..1024 {
        let (next, node) = create(owner, element("c"));
        let (next, requested) = append_amortized(next, &node, &root);
        owner = next;
        appended.push(node);
        growth.extend(requested);
    }
    // One growth at each power of two, never one per append.
    assert_eq!(growth.len(), 11, "{growth:?}");
    assert!(growth.iter().sum::<usize>() <= 2 * appended.len());
    assert_eq!(
        owner
            .node(&root)
            .unwrap()
            .children()
            .map(HtmlConstructionNode::handle)
            .collect::<Vec<_>>(),
        appended
    );
    assert!(
        appended
            .iter()
            .all(|node| owner.node(node).unwrap().parent().unwrap().handle() == root)
    );
}

#[test]
fn append_with_spare_child_order_capacity_requests_no_growth() {
    let (mut owner, root) = fragment(storage());
    let mut requests = Vec::new();
    for _ in 0..4 {
        let (next, node) = create(owner, element("c"));
        let (next, requested) = append_amortized(next, &node, &root);
        owner = next;
        requests.push(requested);
    }
    // Capacity 1, 2 and 4 fill at lengths 1, 2 and 3; the fourth child fits.
    assert_eq!(requests, [vec![1], vec![2], vec![4], vec![]]);
}

#[test]
fn a_child_order_allocation_reports_the_deque_that_holds_the_entries() {
    // A child-order bank is a deque of one-word entries. Its inline header
    // holds the head, the length, the buffer pointer and its capacity: four
    // words, counted here by hand rather than read off `size_of`. An append
    // reserves exactly the one slot it needs the first time, and the admitted
    // capacity of 8 the second time.
    let word = size_of::<usize>();
    let (owner, root) = fragment(storage());
    let (owner, a) = create(owner, element("a"));
    let (owner, b) = create(owner, element("b"));
    let (owner, requested) = append_amortized(owner, &a, &root);
    assert_eq!(requested, [1]);
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b,
        parent: root,
        before: None,
    }));
    let (mut before, mut grown) = (None, None);
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        if let HtmlConstructionActionDescription::Reserve { allocation, .. } = action.description()
        {
            before = Some(allocation.clone());
            let observation = action.perform_with_capacity(8).unwrap();
            grown = observation.allocation().cloned();
            observation.accept();
        } else {
            action.perform().unwrap().accept();
        }
    }
    for (snapshot, len, capacity) in [(before.unwrap(), 1, 1), (grown.unwrap(), 1, 8)] {
        assert_eq!(snapshot.role(), HtmlConstructionAllocationRole::Children);
        assert_eq!(snapshot.child_position(), Some(0));
        assert_eq!(snapshot.initialized_len(), len);
        assert_eq!(snapshot.capacity(), capacity);
        assert_eq!(snapshot.element_size(), word);
        assert_eq!(snapshot.payload_bytes(), capacity * word);
        // The wrapper is the deque header. Its layout is not a std guarantee, so compare with
        // the deque's own size and require it to differ from a `Vec` header.
        assert_eq!(
            snapshot.wrapper_payload_bytes(),
            std::mem::size_of::<std::collections::VecDeque<usize>>()
        );
        assert!(snapshot.wrapper_payload_bytes() > std::mem::size_of::<Vec<usize>>());
    }
}

#[test]
fn interrupted_append_reserve_returns_the_tree_and_keeps_the_grown_order_bank() {
    let (owner, root) = fragment(storage());
    let (owner, a) = create(owner, element("a"));
    let (owner, b) = create(owner, element("b"));
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: a.clone(),
            parent: root.clone(),
            before: None,
        },
    );
    let usage = owner.usage();
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b.clone(),
        parent: root.clone(),
        before: None,
    }));
    let mut before = None;
    let grown = loop {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        let reserve = match action.description() {
            HtmlConstructionActionDescription::Reserve {
                allocation,
                required_len,
                minimum_capacity,
                growth,
                ..
            } => {
                assert_eq!(growth, HtmlConstructionGrowth::Geometric);
                assert_eq!((required_len, minimum_capacity), (2, 2));
                assert_eq!(allocation.role(), HtmlConstructionAllocationRole::Children);
                assert_eq!(allocation.child_position(), Some(0));
                assert_eq!(allocation.initialized_len(), 1);
                before = Some(allocation.clone());
                true
            }
            _ => false,
        };
        if reserve {
            let observation = action.perform_with_capacity(8).unwrap();
            let grown = observation.allocation().unwrap().clone();
            observation.accept();
            break grown;
        }
        action.perform().unwrap().accept();
    };
    let before = before.unwrap();
    assert_eq!(grown.child_position(), Some(0));
    assert_eq!((grown.capacity(), grown.initialized_len()), (8, 1));
    assert_ne!(grown.id(), before.id());
    assert!(grown.id().same_owner_role(before.id()));
    let (owner, command, wrapper) = operation.unwrap().abort().unwrap().into_parts();
    assert!(matches!(command, HtmlConstructionCommand::Insert { .. }));
    assert!(wrapper.is_none());
    assert_eq!(owner.usage(), usage);
    assert_eq!(
        owner
            .node(&root)
            .unwrap()
            .children()
            .map(HtmlConstructionNode::handle)
            .collect::<Vec<_>>()
            .as_slice(),
        std::slice::from_ref(&a)
    );
    assert!(owner.node(&b).unwrap().parent().is_none());
    // The retry needs no growth and commits into the retained capacity.
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b.clone(),
        parent: root.clone(),
        before: None,
    }));
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        assert!(!matches!(
            action.description(),
            HtmlConstructionActionDescription::Reserve { .. }
        ));
        action.perform().unwrap().accept();
    }
    let HtmlConstructionCommit::Changed { storage: owner, .. } =
        slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!()
    };
    assert_eq!(
        owner
            .node(&root)
            .unwrap()
            .children()
            .map(HtmlConstructionNode::handle)
            .collect::<Vec<_>>(),
        [a, b]
    );
    let mut retirement = owner.retire();
    let mut capacity = None;
    while let Some(action) = retirement.next_action().unwrap() {
        if let HtmlConstructionRetirementDescription::Allocation(report) = action.description()
            && report.child_position() == Some(0)
        {
            assert_eq!(report.id(), grown.id());
            capacity = Some(report.capacity());
        }
        action.perform();
    }
    assert_eq!(capacity, Some(8));
}

#[test]
fn refused_append_reserve_reports_the_unchanged_bank_and_latches_the_first_failure() {
    let (owner, root) = fragment(storage());
    let (owner, a) = create(owner, element("a"));
    let (owner, b) = create(owner, element("b"));
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: a.clone(),
            parent: root.clone(),
            before: None,
        },
    );
    let usage = owner.usage();
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b.clone(),
        parent: root.clone(),
        before: None,
    }));
    let failure = loop {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        let before = match action.description() {
            HtmlConstructionActionDescription::Reserve { allocation, .. } => {
                Some(allocation.clone())
            }
            _ => None,
        };
        let Some(before) = before else {
            action.perform().unwrap().accept();
            continue;
        };
        // Below the described minimum is refused before any provider call.
        let refused = action.perform_with_capacity(1).err().unwrap();
        assert_eq!(refused.reason(), &cause::CAPACITY_MISMATCH);
        assert!(refused.allocation().is_none());
        assert!(has_cause(
            operation.as_mut().unwrap().next_action().err().as_ref(),
            cause::CAPACITY_MISMATCH
        ));
        break before;
    };
    assert_eq!(failure.capacity(), 1);
    let owner = operation.unwrap().abort().unwrap().into_parts().0;
    assert_eq!(owner.usage(), usage);
    // A capacity no provider can supply returns the unchanged bank report.
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b.clone(),
        parent: root.clone(),
        before: None,
    }));
    loop {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        if !matches!(
            action.description(),
            HtmlConstructionActionDescription::Reserve { .. }
        ) {
            action.perform().unwrap().accept();
            continue;
        }
        let failed = action
            .perform_with_capacity(isize::MAX as usize / size_of::<usize>())
            .err()
            .unwrap();
        assert_eq!(failed.reason(), &cause::ALLOCATION_FAILED);
        let report = failed.allocation().unwrap();
        assert_eq!(report.id(), failure.id());
        assert_eq!((report.capacity(), report.initialized_len()), (1, 1));
        break;
    }
    assert!(has_cause(
        operation.as_mut().unwrap().next_action().err().as_ref(),
        cause::ALLOCATION_FAILED
    ));
    let owner = operation.unwrap().abort().unwrap().into_parts().0;
    assert_eq!(owner.usage(), usage);
    assert_eq!(owner.node(&root).unwrap().child_count(), 1);
    assert!(owner.node(&b).unwrap().parent().is_none());
}

#[test]
fn capacity_is_only_accepted_by_a_described_geometric_reserve() {
    let (owner, root) = fragment(storage());
    let (owner, a) = create(owner, element("a"));
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: a,
        parent: root,
        before: None,
    }));
    let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
    let refused = action.perform_with_capacity(64).err().unwrap();
    assert_eq!(refused.reason(), &cause::CAPACITY_MISMATCH);
    assert!(refused.allocation().is_none());
    assert!(has_cause(
        slots::operation(&mut operation).err().as_ref(),
        cause::CAPACITY_MISMATCH
    ));
    let owner = operation.unwrap().abort().unwrap().into_parts().0;
    assert_eq!(owner.len(), 2);
}

#[test]
fn append_cycle_refuses_before_any_child_order_growth() {
    let (owner, outer) = create(storage(), element("outer"));
    let (owner, inner) = create(owner, element("inner"));
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: inner.clone(),
            parent: outer.clone(),
            before: None,
        },
    );
    // `inner` has no spare child capacity, yet the cycle is reached first.
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: outer.clone(),
        parent: inner.clone(),
        before: None,
    }));
    let error = loop {
        let action = match operation.as_mut().unwrap().next_action() {
            Ok(Some(action)) => action,
            Ok(None) => panic!("a cycle became ready"),
            Err(error) => break error,
        };
        assert!(!matches!(
            action.description(),
            HtmlConstructionActionDescription::Reserve { .. }
        ));
        match action.perform() {
            Ok(observation) => observation.accept(),
            Err(failure) => break failure.into_parts().0,
        }
    };
    assert_eq!(error, cause::CYCLE);
    let owner = operation.unwrap().abort().unwrap().into_parts().0;
    assert!(owner.node(&outer).unwrap().parent().is_none());
}

#[test]
fn initialization_and_observations_cannot_be_skipped_or_promoted_after_refusal() {
    let mut initialization = Some(HtmlConstructionStorage::begin());
    assert!(has_cause(
        slots::initialization(&mut initialization).err().as_ref(),
        cause::OPERATION_NOT_READY
    ));
    let observation = initialization
        .as_mut()
        .unwrap()
        .next_action()
        .unwrap()
        .unwrap()
        .perform();
    let ticket = observation.allocation().clone();
    drop(observation);
    assert!(has_cause(
        slots::initialization(&mut initialization).err().as_ref(),
        cause::OPERATION_NOT_READY
    ));
    let receipt = initialization.unwrap().abort().unwrap();
    assert_eq!(receipt.allocation(), &ticket);

    let mut operation = Some(storage().prepare(HtmlConstructionCommand::CreateFragment));
    assert!(has_cause(
        slots::operation(&mut operation).err().as_ref(),
        cause::OPERATION_NOT_READY
    ));
    {
        let _action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
    }
    let observation = operation
        .as_mut()
        .unwrap()
        .next_action()
        .unwrap()
        .unwrap()
        .perform()
        .unwrap();
    drop(observation);
    assert!(has_cause(
        operation.as_mut().unwrap().next_action().err().as_ref(),
        cause::OPERATION_NOT_READY
    ));
    assert!(has_cause(
        slots::operation(&mut operation).err().as_ref(),
        cause::OPERATION_NOT_READY
    ));
    let (retained, command, retired) = operation.unwrap().abort().unwrap().into_parts();
    assert!(retained.is_empty());
    assert!(matches!(command, HtmlConstructionCommand::CreateFragment));
    assert!(retired.is_none());
}

#[test]
fn reserve_observation_retains_actual_capacity_and_original_payload_on_abort() {
    let text = DocumentString::from_utf16(vec![0xd800, 0, 0xdfff]);
    let alias = text.clone();
    let mut owner = Some(storage().prepare(HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Text(text),
    }));
    let observed;
    loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        if let HtmlConstructionActionDescription::Reserve {
            allocation,
            required_len,
            minimum_capacity,
            ..
        } = action.description()
        {
            assert_eq!(allocation.capacity(), 0);
            assert_eq!(required_len, 1);
            assert_eq!(minimum_capacity, 1);
            let observation = action.perform().unwrap();
            observed = observation.allocation().unwrap().clone();
            assert!(observed.capacity() >= 1);
            assert_eq!(observed.initialized_len(), 0);
            assert_eq!(
                observed.payload_bytes(),
                observed.capacity() * observed.element_size()
            );
            drop(observation);
            break;
        }
        action.perform().unwrap().accept();
    }
    let (storage, command, _) = owner.unwrap().abort().unwrap().into_parts();
    assert_eq!(storage.usage().nodes(), 0);
    let HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Text(text),
    } = command
    else {
        panic!()
    };
    assert!(text.shares_storage_with(&alias));
    let mut retirement = storage.retire();
    let action = retirement.next_action().unwrap().unwrap();
    assert!(
        matches!(action.description(), HtmlConstructionRetirementDescription::Allocation(snapshot) if snapshot == &observed)
    );
    assert_eq!(action.perform().unwrap().allocation(), &observed);
    assert!(
        retirement
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .is_some()
    );
    assert!(retirement.next_action().unwrap().is_none());
}

#[test]
fn checked_handles_and_ordinals_preserve_identity_across_nonzero_document_publication() {
    let (owner, inert) = fragment(storage());
    let (owner, document) = create(
        owner,
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::Quirks,
        },
    );
    let (owner, template) = create(owner, element("template"));
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: template.clone(),
            parent: document.clone(),
            before: None,
        },
    );
    let id = owner.result_id(&document).unwrap();
    let template_id = owner.result_id(&template).unwrap();
    let contents_id = owner.result_id(&contents).unwrap();
    assert_eq!(owner.position(&document).unwrap(), 1);
    assert!(owner.node_at(usize::MAX).is_none());
    assert_eq!(owner.node_by_result_id(&id).unwrap().handle(), document);
    let (foreign, foreign_handle) = fragment(storage());
    assert!(has_cause(
        owner.position(&foreign_handle).err().as_ref(),
        cause::FOREIGN_BUILDER
    ));
    assert!(
        owner
            .node_by_result_id(&foreign.result_id(&foreign_handle).unwrap())
            .is_err()
    );
    let owner = refuse(
        owner,
        HtmlConstructionCommand::Insert {
            node: foreign_handle,
            parent: inert,
            before: None,
        },
        cause::FOREIGN_BUILDER,
    );
    let (result, _) = publish(owner, document);
    let document = result.document().unwrap();
    assert_eq!(document.root().id(), id);
    assert_eq!(document.mode(), HtmlDocumentMode::Quirks);
    assert_eq!(
        document.node(&template_id).unwrap().template_contents(),
        Some(contents_id.clone())
    );
    assert_eq!(
        document.node(&contents_id).unwrap().fragment_owner(),
        Some(&HtmlFragmentOwner::Template(template_id))
    );
    assert_eq!(document.usage().nodes(), 4);
    assert_eq!(document.usage().edges(), 2);
    assert_eq!(document.usage().depth(), 2);
}

#[test]
fn template_shadow_reference_is_not_an_owning_depth_or_edge_and_reassignment_preserves_host() {
    let (owner, root) = fragment(storage());
    let (owner, host) = create(owner, element("div"));
    let (owner, template) = create(owner, element("template"));
    let ordinary = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, shadow) = create(
        owner,
        HtmlConstructionCommand::CreateShadowContents {
            properties: properties(),
        },
    );
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: host.clone(),
            parent: root.clone(),
            before: None,
        },
    );
    let edges = owner.usage().edges();
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::AttachTemplateShadow {
            template: template.clone(),
            host: host.clone(),
            contents: shadow.clone(),
        },
    );
    assert_eq!(owner.usage().edges(), edges);
    assert_eq!(owner.usage().depth(), 2);
    assert_eq!(
        owner.node(&template).unwrap().template_contents_kind(),
        Some(HtmlTemplateContentsKind::ShadowReference)
    );
    assert!(owner.node(&ordinary).unwrap().fragment_owner().is_none());
    assert_eq!(
        owner.node(&shadow).unwrap().shadow_properties(),
        Some(properties())
    );
    assert_eq!(
        owner
            .node(&host)
            .unwrap()
            .shadow_root()
            .unwrap()
            .properties(),
        properties()
    );
    let owner = refuse(
        owner,
        HtmlConstructionCommand::AttachTemplateShadow {
            template: template.clone(),
            host: host.clone(),
            contents: shadow.clone(),
        },
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    let owner = refuse(
        owner,
        HtmlConstructionCommand::ReassignTemplateContents {
            template: template.clone(),
            contents: shadow.clone(),
        },
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::ReassignTemplateContents {
            template: template.clone(),
            contents: ordinary.clone(),
        },
    );
    assert_eq!(owner.usage().edges(), edges + 1);
    assert_eq!(
        owner.node(&template).unwrap().template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
    assert_eq!(
        owner.node(&shadow).unwrap().shadow_properties(),
        Some(properties())
    );
    let shadow_id = owner.result_id(&shadow).unwrap();
    let host_id = owner.result_id(&host).unwrap();
    assert_eq!(
        owner.node(&shadow).unwrap().fragment_owner(),
        Some(&HtmlFragmentOwner::ShadowHost(host_id.clone()))
    );
    let before = owner.usage();
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::ReassignTemplateContents {
            template,
            contents: ordinary,
        },
    );
    assert_eq!(owner.usage(), before);
    let (result, _) = publish(owner, root);
    assert_eq!(
        result
            .node(&host_id)
            .unwrap()
            .declarative_shadow_root()
            .unwrap()
            .contents(),
        &shadow_id
    );
}

#[test]
fn captured_shadow_role_retains_properties_without_a_local_owner() {
    let (owner, root) = fragment(storage());
    let (owner, template) = create(owner, element("template"));
    let (owner, shadow) = create(
        owner,
        HtmlConstructionCommand::CreateShadowContents {
            properties: properties(),
        },
    );
    let old = owner.usage();
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::SetTemplateShadowContents {
            template: template.clone(),
            contents: shadow.clone(),
        },
    );
    assert_eq!(owner.usage().edges(), old.edges() - 1);
    assert_eq!(owner.usage().depth(), 0);
    assert!(owner.node(&shadow).unwrap().fragment_owner().is_none());
    let owner = refuse(
        owner,
        HtmlConstructionCommand::SetTemplateShadowContents {
            template: template.clone(),
            contents: shadow.clone(),
        },
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    let template_id = owner.result_id(&template).unwrap();
    let shadow_id = owner.result_id(&shadow).unwrap();
    let (result, _) = publish(owner, root);
    assert_eq!(
        result.node(&template_id).unwrap().template_contents(),
        Some(shadow_id.clone())
    );
    let contents = result.node(&shadow_id).unwrap();
    assert!(contents.is_shadow_contents());
    assert_eq!(contents.shadow_properties(), Some(properties()));
    assert!(contents.fragment_owner().is_none());
}

#[test]
fn publication_lease_rejects_foreign_or_aliased_results_without_taking_slots() {
    let (owner, root) = fragment(storage());
    let root_id = owner.result_id(&root).unwrap();
    let (result, lease) = publish(owner, root);
    let alias = result.clone();
    let (other, other_root) = fragment(storage());
    let (other, other_lease) = publish(other, other_root);
    let mut result = Some(result);
    let mut lease = Some(lease);
    assert!(has_cause(
        slots::recover(&mut result, &mut Some(other_lease))
            .err()
            .as_ref(),
        cause::FOREIGN_ALLOCATION
    ));
    assert!(has_cause(
        slots::recover(&mut result, &mut lease).err().as_ref(),
        cause::SHARED_ALLOCATION_ALIASED
    ));
    assert!(result.is_some() && lease.is_some());
    assert_eq!(alias.node(&root_id).unwrap().id(), root_id);
    drop(alias);
    drop(other);
    let wrapper = lease.as_ref().unwrap().allocation().clone();
    let (storage, receipt) = slots::recover(&mut result, &mut lease)
        .unwrap()
        .into_parts();
    assert!(result.is_none() && lease.is_none());
    assert_eq!(receipt.allocation(), &wrapper);
    assert_eq!(storage.node_by_result_id(&root_id).unwrap().position(), 0);
    let root = storage.node_by_result_id(&root_id).unwrap().handle();
    let (result, next_lease) = publish(storage, root);
    drop((result, next_lease));
}

/// Performs `performed` actions of `command`, then aborts; returns whether the
/// operation had no further action, and the storage the abort returned.
fn aborted_after(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    performed: usize,
) -> (bool, HtmlConstructionStorage) {
    let mut operation = Some(storage.prepare(command));
    for _ in 0..performed {
        match operation.as_mut().unwrap().next_action().unwrap() {
            Some(action) => action.perform().unwrap().accept(),
            None => return (true, operation.unwrap().abort().unwrap().into_parts().0),
        }
    }
    let finished = operation.as_mut().unwrap().next_action().unwrap().is_none();
    let (owner, command, wrapper) = operation.unwrap().abort().unwrap().into_parts();
    assert!(!matches!(command, HtmlConstructionCommand::Publish { .. }));
    assert!(wrapper.is_none());
    (finished, owner)
}

#[test]
fn aborting_a_child_order_change_after_any_action_leaves_the_old_orders_and_parents() {
    // Orders [a, b] under `root` and [x] under `other`.
    type Prepared = (
        HtmlConstructionStorage,
        [HtmlBuilderNode; 6],
        Vec<HtmlConstructionCommand>,
    );
    fn prepared() -> Prepared {
        let (owner, root) = fragment(storage());
        let (owner, other) = fragment(owner);
        let (owner, a) = create(owner, element("a"));
        let (owner, b) = create(owner, element("b"));
        let (owner, c) = create(owner, element("c"));
        let (owner, x) = create(owner, element("x"));
        let mut owner = owner;
        for (node, parent) in [(&a, &root), (&b, &root), (&x, &other)] {
            owner = change(
                owner,
                HtmlConstructionCommand::Insert {
                    node: node.clone(),
                    parent: parent.clone(),
                    before: None,
                },
            )
            .0;
        }
        let insert =
            |node: &HtmlBuilderNode, parent: &HtmlBuilderNode, before: &HtmlBuilderNode| {
                HtmlConstructionCommand::Insert {
                    node: node.clone(),
                    parent: parent.clone(),
                    before: Some(before.clone()),
                }
            };
        let commands = vec![
            insert(&c, &root, &a),
            insert(&b, &root, &a),
            insert(&a, &other, &x),
            insert(&c, &other, &x),
            HtmlConstructionCommand::Detach { node: a.clone() },
            HtmlConstructionCommand::Detach { node: b.clone() },
        ];
        (owner, [root, other, a, b, c, x], commands)
    }
    let (_, _, commands) = prepared();
    for index in 0..commands.len() {
        let mut performed = 0;
        loop {
            let (owner, nodes, mut commands) = prepared();
            let [root, other, a, b, c, x] = nodes;
            let before = owner.usage();
            let (finished, owner) = aborted_after(owner, commands.swap_remove(index), performed);
            assert_eq!(owner.usage(), before, "command {index} after {performed}");
            assert_eq!(
                children_snapshot(&owner, &root),
                [a.clone(), b.clone()],
                "command {index} after {performed}"
            );
            assert_eq!(
                children_snapshot(&owner, &other),
                std::slice::from_ref(&x),
                "command {index} after {performed}"
            );
            assert_eq!(owner.node(&a).unwrap().parent().unwrap().handle(), root);
            assert_eq!(owner.node(&b).unwrap().parent().unwrap().handle(), root);
            assert_eq!(owner.node(&x).unwrap().parent().unwrap().handle(), other);
            assert!(owner.node(&c).unwrap().parent().is_none());
            if finished {
                break;
            }
            performed += 1;
        }
    }
}

fn children_snapshot(
    storage: &HtmlConstructionStorage,
    parent: &HtmlBuilderNode,
) -> Vec<HtmlBuilderNode> {
    storage
        .node(parent)
        .unwrap()
        .children()
        .map(HtmlConstructionNode::handle)
        .collect()
}

#[test]
fn record_layout_exhaustion_refuses_before_provider_growth_and_preserves_existing_handles() {
    let (owner, root) = fragment(storage());
    let before = owner.usage();
    let mut sizing = owner.prepare(HtmlConstructionCommand::ReserveRecords { additional: 1 });
    let size;
    loop {
        let action = sizing.next_action().unwrap().unwrap();
        if let HtmlConstructionActionDescription::Reserve { allocation, .. } = action.description()
        {
            size = allocation.element_size();
            break;
        }
        action.perform().unwrap().accept();
    }
    let owner = sizing.abort().unwrap().into_parts().0;
    let additional = (isize::MAX as usize) / size + 1;
    let owner = refuse(
        owner,
        HtmlConstructionCommand::ReserveRecords { additional },
        cause::REPRESENTATION_EXHAUSTED,
    );
    assert_eq!(owner.usage(), before);
    assert_eq!(owner.position(&root), Ok(0));
    assert_eq!(owner.len(), 1);
}

#[test]
fn retained_pending_child_report_survives_abort_and_recreate_without_reusing_identity() {
    let (owner, root) = create(storage(), HtmlConstructionCommand::CreateFragment);
    let root_id = owner.result_id(&root).unwrap();
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::CreateFragment));
    let old = loop {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        let child = matches!(
            action.description(),
            HtmlConstructionActionDescription::InitializeChildren { .. }
        );
        let observation = action.perform().unwrap();
        let report = child.then(|| observation.allocation().unwrap().clone());
        observation.accept();
        if let Some(report) = report {
            break report;
        }
    };
    let (owner, _, _) = operation.unwrap().abort().unwrap().into_parts();
    assert_eq!(old.role(), HtmlConstructionAllocationRole::Children);
    assert_eq!(old.child_position(), Some(1));
    assert!(owner.node_at(1).is_none());
    assert_eq!(owner.position(&root), Ok(0));
    assert_eq!(owner.result_id(&root).unwrap(), root_id);
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::CreateFragment));
    let mut current = None;
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        let child = matches!(
            action.description(),
            HtmlConstructionActionDescription::InitializeChildren { .. }
        );
        let observation = action.perform().unwrap();
        if child {
            current = Some(observation.allocation().unwrap().clone());
        }
        observation.accept();
    }
    let HtmlConstructionCommit::Changed {
        storage: owner,
        created,
        ..
    } = slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!("creation")
    };
    let current = current.unwrap();
    assert_ne!(old.id(), current.id());
    assert_eq!(owner.position(&created.unwrap()), Ok(1));
    assert_eq!(owner.result_id(&root).unwrap(), root_id);
    let mut retirement = owner.retire();
    let mut actual = None;
    while let Some(action) = retirement.next_action().unwrap() {
        if let HtmlConstructionRetirementDescription::Allocation(report) = action.description()
            && report.child_position() == Some(1)
        {
            actual = Some(report.id().clone());
        }
        action.perform();
    }
    assert_eq!(actual.as_ref(), Some(current.id()));
    assert_ne!(actual.as_ref(), Some(old.id()));
}

#[test]
fn grown_child_order_bank_never_reissues_retired_identity_after_abort() {
    let (mut owner, root) = create(storage(), HtmlConstructionCommand::CreateFragment);
    let (next, a) = create(owner, element("a"));
    owner = next;
    let (next, b) = create(owner, element("b"));
    owner = next;
    owner = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: a.clone(),
            parent: root.clone(),
            before: None,
        },
    )
    .0;
    // The order holds one entry in capacity one, so inserting `b` before it
    // grows the parent's actual bank in place, with a new incarnation.
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b.clone(),
        parent: root.clone(),
        before: Some(a.clone()),
    }));
    let mut previous = None;
    let mut preflight = false;
    let grown = loop {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        match action.description() {
            HtmlConstructionActionDescription::Preflight { .. } => preflight = true,
            HtmlConstructionActionDescription::PrepareAllocationReport { .. } => {}
            HtmlConstructionActionDescription::Reserve {
                allocation,
                growth: HtmlConstructionGrowth::Geometric,
                ..
            } => {
                assert!(preflight, "usage must be admitted before the provider");
                previous = Some(allocation.clone());
            }
            _ => {}
        }
        let observation = action.perform().unwrap();
        let grown = previous
            .is_some()
            .then(|| observation.allocation().cloned())
            .flatten();
        observation.accept();
        if let Some(grown) = grown {
            break grown;
        }
    };
    let previous = previous.unwrap();
    assert_ne!(grown.id(), previous.id());
    assert!(grown.id().same_owner_role(previous.id()));
    assert!(grown.capacity() > previous.capacity());
    let (owner, _, _) = operation.unwrap().abort().unwrap().into_parts();
    assert_eq!(children_snapshot(&owner, &root), std::slice::from_ref(&a));
    assert!(owner.node(&b).unwrap().parent().is_none());
    // The retry finds the grown bank and requests no growth.
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: b.clone(),
        parent: root.clone(),
        before: Some(a.clone()),
    }));
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        assert!(!matches!(
            action.description(),
            HtmlConstructionActionDescription::Reserve { .. }
        ));
        action.perform().unwrap().accept();
    }
    let HtmlConstructionCommit::Changed { storage: owner, .. } =
        slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!()
    };
    assert_eq!(children_snapshot(&owner, &root), [b, a]);
    let mut retirement = owner.retire();
    let mut retired = None;
    while let Some(action) = retirement.next_action().unwrap() {
        if let HtmlConstructionRetirementDescription::Allocation(report) = action.description()
            && report.child_position() == Some(0)
        {
            retired = Some(report.id().clone());
        }
        action.perform();
    }
    assert_eq!(retired.as_ref(), Some(grown.id()));
}

#[test]
fn already_correct_insertions_change_no_order_and_allocate_nothing() {
    for before_successor in [false, true] {
        let (owner, root) = fragment(storage());
        let (owner, a) = create(owner, element("a"));
        let (owner, b) = create(owner, element("b"));
        let (owner, _) = change(
            owner,
            HtmlConstructionCommand::Insert {
                node: a.clone(),
                parent: root.clone(),
                before: None,
            },
        );
        let (owner, _) = change(
            owner,
            HtmlConstructionCommand::Insert {
                node: b.clone(),
                parent: root.clone(),
                before: None,
            },
        );
        let usage = owner.usage();
        let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
            node: if before_successor {
                a.clone()
            } else {
                b.clone()
            },
            parent: root.clone(),
            before: before_successor.then(|| b.clone()),
        }));
        let mut preflight = false;
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            if let HtmlConstructionActionDescription::Preflight { usage: computed } =
                action.description()
            {
                assert_eq!(computed, usage);
                let view = action.storage_view().unwrap();
                assert_eq!(
                    view.node(&root)
                        .unwrap()
                        .children()
                        .map(HtmlConstructionNode::handle)
                        .collect::<Vec<_>>(),
                    [a.clone(), b.clone()]
                );
                preflight = true;
            } else {
                assert!(action.storage_view().is_none());
            }
            // The node leaves the order before it returns, so the full order
            // needs no growth.
            assert!(!matches!(
                action.description(),
                HtmlConstructionActionDescription::Reserve { .. }
            ));
            let observation = action.perform().unwrap();
            assert!(observation.allocation().is_none());
            observation.accept();
        }
        assert!(preflight);
        let HtmlConstructionCommit::Changed { storage: owner, .. } =
            slots::operation(&mut operation).unwrap().commit().unwrap()
        else {
            panic!()
        };
        assert_eq!(owner.usage(), usage);
        assert_eq!(
            owner
                .node(&root)
                .unwrap()
                .children()
                .map(HtmlConstructionNode::handle)
                .collect::<Vec<_>>(),
            [a.clone(), b.clone()]
        );
        assert_eq!(owner.node(&a).unwrap().parent().unwrap().handle(), root);
        assert_eq!(owner.node(&b).unwrap().parent().unwrap().handle(), root);
    }
}

#[test]
fn self_reference_validates_parent_before_noop_and_valid_noop_has_no_allocation() {
    let (owner, root) = fragment(storage());
    let (owner, other) = create(owner, element("other"));
    let (owner, a) = create(owner, element("a"));
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Insert {
            node: a.clone(),
            parent: root.clone(),
            before: None,
        },
    );
    let usage = owner.usage();
    let owner = refuse(
        owner,
        HtmlConstructionCommand::Insert {
            node: a.clone(),
            parent: other.clone(),
            before: Some(a.clone()),
        },
        cause::WRONG_SIBLING,
    );
    assert_eq!(owner.node(&a).unwrap().parent().unwrap().handle(), root);
    assert_eq!(owner.node(&other).unwrap().child_count(), 0);
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::Insert {
        node: a.clone(),
        parent: root.clone(),
        before: Some(a.clone()),
    }));
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        assert!(!matches!(
            action.description(),
            HtmlConstructionActionDescription::Preflight { .. }
                | HtmlConstructionActionDescription::Reserve { .. }
        ));
        action.perform().unwrap().accept();
    }
    let HtmlConstructionCommit::Changed { storage: owner, .. } =
        slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!()
    };
    assert_eq!(owner.usage(), usage);
    assert_eq!(
        owner
            .node(&root)
            .unwrap()
            .children()
            .map(HtmlConstructionNode::handle)
            .collect::<Vec<_>>(),
        [a]
    );
}

#[test]
fn element_creation_installs_supplied_link_history_only_in_atomic_ready_commit() {
    for supplied in [
        None,
        Some(surgeist_html::elements::link::HtmlLinkFlags::fresh()),
    ] {
        let (owner, _) = fragment(storage());
        let attributes = OrderedAttributes::new([surgeist_html::values::Attribute::new(
            QualifiedName::new(Namespace::none(), None, "rel"),
            "stylesheet",
        )])
        .unwrap();
        let backing = attributes.shared_attributes().clone();
        let command = HtmlConstructionCommand::CreateElement {
            name: QualifiedName::new(Namespace::html(), None, "link"),
            attributes,
            link_flags: supplied,
        };
        let ready = ready(owner, command);
        let HtmlConstructionCommit::Changed {
            storage: owner,
            created: Some(link),
            ..
        } = ready.commit().unwrap()
        else {
            panic!()
        };
        let HtmlNodeKind::Element(element) = owner.node(&link).unwrap().kind() else {
            panic!()
        };
        assert_eq!(element.link_flags(), supplied);
        assert!(Arc::ptr_eq(
            element.attributes().shared_attributes(),
            &backing
        ));
        assert_eq!(
            element.attributes().as_slice()[0]
                .value()
                .to_utf8()
                .unwrap(),
            "stylesheet"
        );
        assert_eq!(owner.usage().attributes(), 1);
    }
}

#[test]
fn invalid_link_history_refuses_before_providers_and_recovers_exact_element_inputs() {
    for (namespace, local) in [
        (Namespace::html(), "div"),
        (Namespace::svg(), "link"),
        (Namespace::html(), "LINK"),
    ] {
        let owner = storage();
        let name = QualifiedName::new(namespace, None, local);
        let name_alias = name.local_name().to_shared_utf16();
        let attributes = OrderedAttributes::new([surgeist_html::values::Attribute::new(
            QualifiedName::new(Namespace::none(), None, "x"),
            "v",
        )])
        .unwrap();
        let attributes_alias = attributes.shared_attributes().clone();
        let mut operation = Some(owner.prepare(HtmlConstructionCommand::CreateElement {
            name,
            attributes,
            link_flags: Some(surgeist_html::elements::link::HtmlLinkFlags::fresh()),
        }));
        loop {
            let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
            assert!(!matches!(
                action.description(),
                HtmlConstructionActionDescription::Reserve { .. }
                    | HtmlConstructionActionDescription::InitializeChildren { .. }
                    | HtmlConstructionActionDescription::ShareReferences { .. }
            ));
            match action.perform() {
                Ok(observation) => observation.accept(),
                Err(error) => {
                    assert_eq!(error.reason(), &cause::WRONG_NODE_KIND);
                    break;
                }
            }
        }
        assert!(has_cause(
            slots::operation(&mut operation).err().as_ref(),
            cause::WRONG_NODE_KIND
        ));
        let (owner, command, _) = operation.unwrap().abort().unwrap().into_parts();
        assert_eq!(owner.len(), 0);
        let HtmlConstructionCommand::CreateElement {
            name,
            attributes,
            link_flags: Some(_),
        } = command
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(
            &name.local_name().to_shared_utf16(),
            &name_alias
        ));
        assert!(Arc::ptr_eq(
            attributes.shared_attributes(),
            &attributes_alias
        ));
    }
}

#[test]
fn interrupted_element_creation_returns_original_command_without_publishing_a_node() {
    let (owner, root) = fragment(storage());
    let attributes = OrderedAttributes::new([surgeist_html::values::Attribute::new(
        QualifiedName::new(Namespace::none(), None, "href"),
        "x",
    )])
    .unwrap();
    let backing = attributes.shared_attributes().clone();
    let mut operation = Some(owner.prepare(HtmlConstructionCommand::CreateElement {
        name: QualifiedName::new(Namespace::html(), None, "link"),
        attributes,
        link_flags: Some(surgeist_html::elements::link::HtmlLinkFlags::fresh()),
    }));
    loop {
        let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
        if matches!(
            action.description(),
            HtmlConstructionActionDescription::Commit { .. }
        ) {
            break;
        }
        action.perform().unwrap().accept();
    }
    assert!(has_cause(
        slots::operation(&mut operation).err().as_ref(),
        cause::OPERATION_NOT_READY
    ));
    let (owner, command, _) = operation.unwrap().abort().unwrap().into_parts();
    assert_eq!(owner.len(), 1);
    assert_eq!(owner.position(&root), Ok(0));
    assert!(owner.node_at(1).is_none());
    let HtmlConstructionCommand::CreateElement {
        attributes,
        link_flags: Some(_),
        ..
    } = command
    else {
        panic!()
    };
    assert!(Arc::ptr_eq(attributes.shared_attributes(), &backing));
    let (owner, link) = create(
        owner,
        HtmlConstructionCommand::CreateElement {
            name: QualifiedName::new(Namespace::html(), None, "link"),
            attributes,
            link_flags: None,
        },
    );
    let HtmlNodeKind::Element(element) = owner.node(&link).unwrap().kind() else {
        panic!()
    };
    assert_eq!(element.link_flags(), None);
    assert!(Arc::ptr_eq(
        element.attributes().shared_attributes(),
        &backing
    ));
}

/// The charge of the Commit action of `command`, with the storage handed back after the command
/// is applied.
fn commit_charge(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, crate::html::support::Recorder) {
    let mut operation = Some(storage.prepare(command));
    let mut charged = None;
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        let description = action.description();
        if matches!(
            description,
            HtmlConstructionActionDescription::Commit { .. }
        ) {
            charged = Some(charged_by(|meter| description.charge_commit(meter)));
        }
        action.perform().unwrap().accept();
    }
    let HtmlConstructionCommit::Changed { storage, .. } =
        slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!("mutation produced publication")
    };
    (
        storage,
        charged.expect("every operation ends in a Commit action"),
    )
}

#[test]
fn a_commit_records_the_largest_layout_and_observes_depth_only_without_an_earlier_usage_boundary() {
    use surgeist_html::accounting::HtmlResource;
    let (owner, root) = fragment(storage());
    // A creation observes depth at its commit. The layout is the bytes the commit initializes.
    let (owner, created) = {
        let (owner, charged) = commit_charge(owner, element("leaf"));
        let layouts: Vec<_> = charged
            .gauges()
            .iter()
            .filter(|(resource, _)| *resource == HtmlResource::AtomicLayoutBytes)
            .collect();
        assert_eq!(layouts.len(), 1);
        assert_eq!(
            charged
                .gauges()
                .iter()
                .filter(|(resource, _)| *resource == HtmlResource::TreeDepth)
                .count(),
            1
        );
        // The gauges come first, then the three work changes in their order.
        assert_eq!(charged.batches().len(), 3);
        let created = owner.node_at(owner.len() - 1).unwrap().handle();
        (owner, created)
    };
    // An insert reaches its depth through the usage its preflight charged, not at the commit.
    let (_, charged) = commit_charge(
        owner,
        HtmlConstructionCommand::Insert {
            node: created,
            parent: root,
            before: None,
        },
    );
    assert_eq!(
        charged
            .gauges()
            .iter()
            .filter(|(resource, _)| *resource == HtmlResource::TreeDepth)
            .count(),
        0
    );
    assert_eq!(
        charged
            .gauges()
            .iter()
            .filter(|(resource, _)| *resource == HtmlResource::AtomicLayoutBytes)
            .count(),
        1
    );
}

#[test]
fn a_description_charges_only_the_parts_it_has() {
    use surgeist_html::{
        accounting::HtmlChargedError, bounded::construction::HtmlConstructionChargeFault,
    };
    let (owner, _) = fragment(storage());
    let mut operation = Some(owner.prepare(element("probe")));
    let action = operation.as_mut().unwrap().next_action().unwrap().unwrap();
    let description = action.description();
    let mut meter = crate::html::support::Recorder::default();
    assert!(matches!(
        description,
        HtmlConstructionActionDescription::Inspect { .. }
    ));
    assert_eq!(
        description.charge_commit(&mut meter),
        Err(HtmlChargedError::Faulted(
            HtmlConstructionChargeFault::WrongStep
        ))
    );
    assert!(meter.batches().is_empty() && meter.gauges().is_empty());
    description.charge_inspection(&mut meter).unwrap();
    // The records visited, then the bytes inspected, each their own batch.
    assert_eq!(meter.batches().len(), 2);
}
