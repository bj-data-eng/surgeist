#![forbid(unsafe_code)]

//! A caller-owned keyed list composed from the actual Runtime and DOM APIs.
//! The map records instantiated identities; it is not another semantic tree.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    CreatedNode, DocumentId, Dom, DomBatch, DomError, DomLimits, DomResource, DomSnapshot,
    DomStamp, NodeId, NodeOperand,
};
use surgeist_runtime::{
    AppInput, InputProvenance, Reducer, ReducerCommit, ReducerResult, Runtime, RuntimeBudget,
    UiInput,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ItemKey {
    A,
    B,
    C,
}
impl ItemKey {
    fn label(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

struct RequestReducer;
impl Reducer<Vec<ItemKey>, Vec<ItemKey>> for RequestReducer {
    fn reduce(
        &mut self,
        _: &Vec<ItemKey>,
        input: &AppInput<Vec<ItemKey>>,
    ) -> ReducerResult<Vec<ItemKey>> {
        ReducerResult::changed(input.payload().clone(), ReducerCommit::new())
    }
}
type Requests = Runtime<Vec<ItemKey>, RequestReducer, Vec<ItemKey>>;

fn schedule(requests: &mut Requests, keys: &[ItemKey]) {
    requests
        .enqueue_ui(UiInput::new(keys.to_vec(), InputProvenance::system()).unwrap())
        .unwrap();
    let turn = requests.drain_once(RuntimeBudget::new(1, 1, 0, 0)).unwrap();
    assert_eq!(turn.drained_inputs(), 1);
    assert_eq!(requests.state(), keys);
}

#[derive(Debug, Eq, PartialEq)]
enum ComposeError {
    DuplicateKey(ItemKey),
    StalePreparation,
    Allocation,
    Dom(DomError),
}
impl From<DomError> for ComposeError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

enum PlannedNode {
    Existing(NodeId),
    Created(CreatedNode),
}

// Preparation is private caller state, not a materialization protocol exported
// by Runtime. All allocation for the next live map happens before DOM apply.
struct Prepared {
    basis: DomStamp,
    batch: DomBatch,
    entries: Vec<(ItemKey, PlannedNode)>,
    committed_map: Vec<(ItemKey, NodeId)>,
}
struct MountedItems {
    dom: Dom,
    document: DocumentId,
    parent: NodeId,
    live: Vec<(ItemKey, NodeId)>,
}

fn limits(nodes: usize) -> DomLimits {
    DomLimits::new(nodes, 4096, 64, 16, 65536).unwrap()
}
impl MountedItems {
    fn new() -> Self {
        let mut dom = Dom::new(limits(64)).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element(&document, "Items");
        batch.insert_before(&parent, &document, None);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        Self {
            dom,
            document: commit.document(&document).unwrap(),
            parent: commit.created(&parent).unwrap().unwrap(),
            live: Vec::new(),
        }
    }

    fn prepare(&self, desired: &[ItemKey]) -> Result<Prepared, ComposeError> {
        for (index, key) in desired.iter().enumerate() {
            if desired[..index].contains(key) {
                return Err(ComposeError::DuplicateKey(*key));
            }
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(desired.len())
            .map_err(|_| ComposeError::Allocation)?;
        let mut committed_map = Vec::new();
        committed_map
            .try_reserve_exact(desired.len())
            .map_err(|_| ComposeError::Allocation)?;
        let mut batch = DomBatch::new();
        for &(key, node) in &self.live {
            if !desired.contains(&key) {
                batch.detach(node);
                batch.destroy(node);
            }
        }
        for &key in desired {
            let (planned, operand) =
                if let Some(&(_, node)) = self.live.iter().find(|(live_key, _)| *live_key == key) {
                    (PlannedNode::Existing(node), NodeOperand::Existing(node))
                } else {
                    let row = batch.create_element(self.document, "Row");
                    batch.set_attribute(&row, "data-key", key.label());
                    let text = batch.create_text(self.document, key.label());
                    batch.insert_before(&text, &row, None);
                    let operand = NodeOperand::from(&row);
                    (PlannedNode::Created(row), operand)
                };
            batch.insert_before(operand, self.parent, None);
            entries.push((key, planned));
        }
        Ok(Prepared {
            basis: self.dom.snapshot().stamp(),
            batch,
            entries,
            committed_map,
        })
    }

    fn publish(&mut self, prepared: Prepared, cancelled: &AtomicBool) -> Result<(), ComposeError> {
        if prepared.basis != self.dom.snapshot().stamp() {
            return Err(ComposeError::StalePreparation);
        }
        let Prepared {
            batch,
            entries,
            mut committed_map,
            ..
        } = prepared;
        let commit = self.dom.apply(batch, cancelled)?;
        // These tickets came from this exact batch and none was destroyed by
        // its commands. Resolution is therefore a proved invariant, not a new
        // recoverable failure after canonical publication. Capacity is reserved.
        for (key, planned) in entries {
            let node = match planned {
                PlannedNode::Existing(node) => node,
                PlannedNode::Created(ticket) => commit
                    .created(&ticket)
                    .expect("composer owns this batch's ticket")
                    .expect("composer never destroys a new row"),
            };
            committed_map.push((key, node));
        }
        self.live = committed_map;
        Ok(())
    }

    fn apply_request(&mut self, requests: &Requests) -> Result<(), ComposeError> {
        let prepared = self.prepare(requests.state())?;
        self.publish(prepared, &AtomicBool::new(false))
    }

    fn node(&self, key: ItemKey) -> NodeId {
        self.live.iter().find(|(item, _)| *item == key).unwrap().1
    }

    fn assert_keys(&self, expected: &[ItemKey]) {
        assert_eq!(
            self.live.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            expected
        );
        let snapshot = self.dom.snapshot();
        let document = snapshot.document(self.document).unwrap();
        let children = document.node(self.parent).unwrap().children();
        assert_eq!(
            children,
            self.live.iter().map(|(_, node)| *node).collect::<Vec<_>>()
        );
        for (index, &(key, node)) in self.live.iter().enumerate() {
            assert!(!children[..index].contains(&node));
            let row = document.node(node).unwrap();
            assert_eq!(row.node_name().to_utf8().unwrap(), "Row");
            assert_eq!(row.parent(), Some(self.parent));
            assert!(row.is_connected());
            assert_eq!(
                row.as_element()
                    .unwrap()
                    .attribute(&"data-key".into())
                    .unwrap()
                    .to_utf8()
                    .unwrap(),
                key.label()
            );
            assert_eq!(row.children().len(), 1);
            assert_eq!(
                document
                    .node(row.children()[0])
                    .unwrap()
                    .data()
                    .unwrap()
                    .to_utf8()
                    .unwrap(),
                key.label()
            );
        }
    }
}

fn mounted_pair() -> (MountedItems, Requests) {
    let mut requests = Runtime::new(Vec::new(), RequestReducer);
    let mut mounted = MountedItems::new();
    schedule(&mut requests, &[ItemKey::A, ItemKey::B]);
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::A, ItemKey::B]);
    (mounted, requests)
}

fn assert_unchanged(mounted: &MountedItems, before: &DomSnapshot, map: &[(ItemKey, NodeId)]) {
    assert_eq!(mounted.live, map);
    let current = mounted.dom.snapshot();
    assert_eq!(current.stamp(), before.stamp());
    assert_eq!(
        current.node(mounted.parent).unwrap().children(),
        before.node(mounted.parent).unwrap().children()
    );
    for &(_, node) in map {
        assert_eq!(
            current.node(node).unwrap().kind(),
            before.node(node).unwrap().kind()
        );
        assert_eq!(
            current.node(node).unwrap().children(),
            before.node(node).unwrap().children()
        );
    }
    mounted.assert_keys(&[ItemKey::A, ItemKey::B]);
}

#[test]
fn runtime_key_requests_preserve_live_identity_reorder_and_retire_without_aliasing() {
    let (mut mounted, mut requests) = mounted_pair();
    let old = mounted.dom.snapshot();
    let a = mounted.node(ItemKey::A);
    let b = mounted.node(ItemKey::B);
    assert_ne!(a, b);
    schedule(&mut requests, &[ItemKey::B, ItemKey::C]);
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::B, ItemKey::C]);
    let c = mounted.node(ItemKey::C);
    assert_eq!(mounted.node(ItemKey::B), b);
    assert_ne!(c, a);
    assert_ne!(c, b);
    assert_eq!(
        mounted.dom.snapshot().node(a).unwrap_err(),
        DomError::StaleNode
    );

    schedule(&mut requests, &[ItemKey::C, ItemKey::B]);
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::C, ItemKey::B]);
    assert_eq!(mounted.node(ItemKey::B), b);
    assert_eq!(mounted.node(ItemKey::C), c);

    schedule(&mut requests, &[ItemKey::A, ItemKey::C, ItemKey::B]);
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::A, ItemKey::C, ItemKey::B]);
    let fresh_a = mounted.node(ItemKey::A);
    assert_ne!(fresh_a, a);
    assert_ne!(fresh_a, b);
    assert_ne!(fresh_a, c);
    assert_eq!(mounted.node(ItemKey::B), b);
    assert_eq!(mounted.node(ItemKey::C), c);
    assert_eq!(
        mounted.dom.snapshot().node(a).unwrap_err(),
        DomError::StaleNode
    );
    assert_eq!(old.node(mounted.parent).unwrap().children(), &[a, b]);
    assert_eq!(old.node(a).unwrap().node_name().to_utf8().unwrap(), "Row");
    assert_eq!(old.node(a).unwrap().parent(), Some(mounted.parent));
    assert_eq!(
        old.node(a)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"data-key".into())
            .unwrap()
            .to_utf8()
            .unwrap(),
        "A"
    );
}

#[test]
fn selected_caller_item_data_survives_retirement_and_is_republished_on_a_fresh_node() {
    // Selection belongs to the caller's logical records, independently of the
    // live identity map. This ordinary data attribute is not HTML control state.
    let item_data = [(ItemKey::A, true), (ItemKey::B, false)];
    let mut requests = Runtime::new(Vec::new(), RequestReducer);
    let mut mounted = MountedItems::new();

    let publish = |mounted: &mut MountedItems, requests: &Requests| {
        let mut prepared = mounted.prepare(requests.state()).unwrap();
        for (key, planned) in &prepared.entries {
            let selected = item_data.iter().find(|(item, _)| item == key).unwrap().1;
            let operand = match planned {
                PlannedNode::Existing(node) => NodeOperand::Existing(*node),
                PlannedNode::Created(ticket) => NodeOperand::from(ticket),
            };
            prepared.batch.set_attribute(
                operand,
                "data-item-selected",
                if selected { "true" } else { "false" },
            );
        }
        mounted.publish(prepared, &AtomicBool::new(false)).unwrap();
    };
    let selected_content = |snapshot: &DomSnapshot, node| {
        snapshot
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"data-item-selected".into())
            .unwrap()
            .to_utf8()
            .unwrap()
    };

    schedule(&mut requests, &[ItemKey::A, ItemKey::B]);
    publish(&mut mounted, &requests);
    mounted.assert_keys(&[ItemKey::A, ItemKey::B]);
    let a = mounted.node(ItemKey::A);
    let b = mounted.node(ItemKey::B);
    let old = mounted.dom.snapshot();
    assert_eq!(selected_content(&old, a), "true");
    assert_eq!(selected_content(&old, b), "false");

    schedule(&mut requests, &[ItemKey::B]);
    publish(&mut mounted, &requests);
    mounted.assert_keys(&[ItemKey::B]);
    assert_eq!(mounted.node(ItemKey::B), b);
    assert_eq!(item_data, [(ItemKey::A, true), (ItemKey::B, false)]);
    let absent = mounted.dom.snapshot();
    assert_eq!(absent.node(a).unwrap_err(), DomError::StaleNode);
    assert_eq!(selected_content(&absent, b), "false");

    schedule(&mut requests, &[ItemKey::B, ItemKey::A]);
    publish(&mut mounted, &requests);
    mounted.assert_keys(&[ItemKey::B, ItemKey::A]);
    let fresh_a = mounted.node(ItemKey::A);
    assert_ne!(fresh_a, a);
    assert_ne!(fresh_a, b);
    assert_eq!(mounted.node(ItemKey::B), b);
    let current = mounted.dom.snapshot();
    assert_eq!(current.node(a).unwrap_err(), DomError::StaleNode);
    assert_eq!(selected_content(&current, fresh_a), "true");
    assert_eq!(selected_content(&current, b), "false");
    assert_eq!(item_data, [(ItemKey::A, true), (ItemKey::B, false)]);
    assert_eq!(old.node(mounted.parent).unwrap().children(), &[a, b]);
    assert_eq!(selected_content(&old, a), "true");
    // Reintroduction does not retroactively materialize A in either older view.
    assert_eq!(absent.node(a).unwrap_err(), DomError::StaleNode);
    assert_eq!(absent.node(fresh_a).unwrap_err(), DomError::StaleNode);
}

#[test]
fn cancelled_dom_publication_keeps_committed_map_and_tree_while_request_remains_pending() {
    let (mut mounted, mut requests) = mounted_pair();
    let before = mounted.dom.snapshot();
    let map = mounted.live.clone();
    schedule(&mut requests, &[ItemKey::B, ItemKey::C]);
    let prepared = mounted.prepare(requests.state()).unwrap();
    assert_eq!(
        mounted.publish(prepared, &AtomicBool::new(true)),
        Err(ComposeError::Dom(DomError::Cancelled))
    );
    assert_eq!(requests.state(), &[ItemKey::B, ItemKey::C]);
    assert_unchanged(&mounted, &before, &map);
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::B, ItemKey::C]);
    assert_eq!(mounted.node(ItemKey::B), map[1].1);
}

#[test]
fn actual_late_invalid_attribute_rolls_back_retirement_creation_reorder_and_map() {
    let (mut mounted, mut requests) = mounted_pair();
    let before = mounted.dom.snapshot();
    let map = mounted.live.clone();
    let usage = mounted.dom.usage();
    schedule(&mut requests, &[ItemKey::B, ItemKey::C]);
    let mut prepared = mounted.prepare(requests.state()).unwrap();
    // This actual final DOM command follows the ordinary removal, creation and
    // reorder edits. It is not an injected failure or a fake dependency.
    prepared
        .batch
        .set_attribute(mounted.parent, "bad name", "rejected");
    assert_eq!(
        mounted.publish(prepared, &AtomicBool::new(false)),
        Err(ComposeError::Dom(DomError::InvalidCharacter))
    );
    assert_eq!(mounted.dom.usage(), usage);
    assert_eq!(requests.state(), &[ItemKey::B, ItemKey::C]);
    assert_unchanged(&mounted, &before, &map);
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::B, ItemKey::C]);
    assert_eq!(mounted.node(ItemKey::B), map[1].1);
    assert_eq!(
        before.node(mounted.parent).unwrap().children(),
        &[map[0].1, map[1].1]
    );
}

#[test]
fn exact_managed_node_capacity_admits_three_rows_and_one_short_keeps_map_and_dom_atomic() {
    let (mut mounted, mut requests) = mounted_pair();
    // Document + Items + two nodes (Row and Text) per materialized item.
    assert_eq!(mounted.dom.usage().managed_nodes(), 6);
    mounted.dom.set_limits(limits(7)).unwrap();
    let before = mounted.dom.snapshot();
    let map = mounted.live.clone();
    schedule(&mut requests, &[ItemKey::A, ItemKey::B, ItemKey::C]);
    assert_eq!(
        mounted.apply_request(&requests),
        Err(ComposeError::Dom(DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 7
        }))
    );
    assert_eq!(mounted.dom.usage().managed_nodes(), 6);
    assert_eq!(requests.state(), &[ItemKey::A, ItemKey::B, ItemKey::C]);
    assert_unchanged(&mounted, &before, &map);
    mounted.dom.set_limits(limits(8)).unwrap();
    mounted.apply_request(&requests).unwrap();
    mounted.assert_keys(&[ItemKey::A, ItemKey::B, ItemKey::C]);
    assert_eq!(mounted.dom.usage().managed_nodes(), 8);
    assert_eq!(mounted.node(ItemKey::A), map[0].1);
    assert_eq!(mounted.node(ItemKey::B), map[1].1);
}

#[test]
fn duplicate_logical_key_request_cannot_assign_one_live_node_to_two_items() {
    let (mounted, mut requests) = mounted_pair();
    let before = mounted.dom.snapshot();
    let map = mounted.live.clone();
    schedule(&mut requests, &[ItemKey::B, ItemKey::B]);
    assert!(matches!(
        mounted.prepare(requests.state()),
        Err(ComposeError::DuplicateKey(ItemKey::B))
    ));
    assert_unchanged(&mounted, &before, &map);
}
