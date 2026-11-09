//! HTML877d slot.assign literal state/effect oracles. The fixture owns only
//! identity records and ordered lists. Root assignment is recorded as an
//! intention; DOM's assignment/query algorithms are not faked here.
use std::convert::Infallible;
use surgeist_html::bounded::{
    native::native_element_initialization_with,
    template::{
        HtmlSlotAssignAction as Action, HtmlSlotAssignCursor, HtmlSlotAssignReply as Reply,
    },
};
use surgeist_html::{
    elements::native::{HtmlElementClass, HtmlNativeFreshSlot},
    values::{Namespace, QualifiedName},
    vocabulary::KnownHtmlElement,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Node {
    A,
    B,
    C,
    D,
    Q,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Slot {
    Target,
    Other,
}
#[derive(Clone, Debug)]
struct NodeRecord {
    id: Node,
    manual: Option<Slot>,
}
#[derive(Clone, Debug)]
struct SlotRecord {
    id: Slot,
    manual: Vec<Node>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Effect {
    Clear(Node),
    Begin,
    Read(Node, Option<Slot>),
    Remove(Node, Slot),
    Set(Node),
    Append(Node),
    AlreadyPresent(Node),
    Replace,
    AssignTargetRoot,
}
#[derive(Clone, Debug)]
struct Records {
    nodes: Vec<NodeRecord>,
    slots: Vec<SlotRecord>,
    incoming: Vec<Node>,
    pending: Option<Vec<Node>>,
    referred: Option<(Node, Slot)>,
    effects: Vec<Effect>,
}
impl Records {
    fn new(target: &[Node], other: &[Node], incoming: &[Node]) -> Self {
        let mut value = Self {
            nodes: [Node::A, Node::B, Node::C, Node::D, Node::Q]
                .into_iter()
                .map(|id| NodeRecord { id, manual: None })
                .collect(),
            slots: vec![
                SlotRecord {
                    id: Slot::Target,
                    manual: target.to_vec(),
                },
                SlotRecord {
                    id: Slot::Other,
                    manual: other.to_vec(),
                },
            ],
            incoming: incoming.to_vec(),
            pending: None,
            referred: None,
            effects: Vec::new(),
        };
        for (slot, nodes) in [(Slot::Target, target), (Slot::Other, other)] {
            for &id in nodes {
                let record = value.node_mut(id);
                assert!(record.manual.is_none(), "initial ordered sets are disjoint");
                record.manual = Some(slot);
            }
        }
        value
    }
    fn node_mut(&mut self, id: Node) -> &mut NodeRecord {
        self.nodes
            .iter_mut()
            .find(|record| record.id == id)
            .unwrap()
    }
    fn slot(&self, id: Slot) -> &[Node] {
        &self
            .slots
            .iter()
            .find(|record| record.id == id)
            .unwrap()
            .manual
    }
    fn slot_mut(&mut self, id: Slot) -> &mut Vec<Node> {
        &mut self
            .slots
            .iter_mut()
            .find(|record| record.id == id)
            .unwrap()
            .manual
    }
    fn cursor(&self) -> HtmlSlotAssignCursor {
        HtmlSlotAssignCursor::new(self.slot(Slot::Target).len(), self.incoming.len())
    }
    fn apply(&mut self, action: Action) -> Reply {
        match action {
            Action::ClearPreviousManualInverse { previous_ordinal } => {
                let node = self.slot(Slot::Target)[previous_ordinal];
                self.node_mut(node).manual = None;
                self.effects.push(Effect::Clear(node));
            }
            Action::BeginPendingManualSet => {
                assert!(self.pending.is_none());
                self.pending = Some(Vec::new());
                self.effects.push(Effect::Begin);
            }
            Action::ReadIncomingManualAssignment { ordinal } => {
                let node = self.incoming[ordinal];
                let manual = self.node_mut(node).manual;
                self.referred = manual.map(|slot| (node, slot));
                self.effects.push(Effect::Read(node, manual));
                return Reply::ManualAssignmentPresent(manual.is_some());
            }
            Action::RemoveIncomingFromReferredManualSet { ordinal } => {
                let node = self.incoming[ordinal];
                let (observed, slot) = self.referred.take().unwrap();
                assert_eq!(observed, node);
                self.slot_mut(slot).retain(|&entry| entry != node);
                self.effects.push(Effect::Remove(node, slot));
            }
            Action::SetIncomingManualAssignmentToTarget { ordinal } => {
                let node = self.incoming[ordinal];
                self.node_mut(node).manual = Some(Slot::Target);
                self.effects.push(Effect::Set(node));
            }
            Action::AppendIncomingToPendingSet { ordinal } => {
                let node = self.incoming[ordinal];
                // The ordered set append: a node the set already holds is not added again.
                let pending = self.pending.as_mut().unwrap();
                if pending.contains(&node) {
                    self.effects.push(Effect::AlreadyPresent(node));
                } else {
                    pending.push(node);
                    self.effects.push(Effect::Append(node));
                }
            }
            Action::ReplaceTargetManualSet => {
                *self.slot_mut(Slot::Target) = self.pending.take().unwrap();
                self.effects.push(Effect::Replace);
            }
            Action::AssignSlottablesForTargetRoot => {
                self.effects.push(Effect::AssignTargetRoot);
            }
            Action::Complete => panic!("completion is not an effect"),
        }
        Reply::Applied
    }
    fn run(&mut self, wrong_replies: bool) -> HtmlSlotAssignCursor {
        let mut cursor = self.cursor();
        while cursor.action() != Action::Complete {
            let action = cursor.action();
            if wrong_replies {
                let wrong = match action {
                    Action::ReadIncomingManualAssignment { .. } => Reply::Applied,
                    _ => Reply::ManualAssignmentPresent(false),
                };
                assert!(matches!(cursor.reply(wrong), Err(ref e) if e.is_reply()));
                assert_eq!(cursor.action(), action);
            }
            let reply = self.apply(action);
            cursor.reply(reply).unwrap();
        }
        cursor
    }
    fn run_admitted(
        &mut self,
        cursor: &mut HtmlSlotAssignCursor,
        mut admit: impl FnMut(Action) -> Result<(), usize>,
    ) -> Result<(), usize> {
        while cursor.action() != Action::Complete {
            let action = cursor.action();
            admit(action)?;
            let reply = self.apply(action);
            cursor.reply(reply).unwrap();
        }
        Ok(())
    }
    fn pointers(&self) -> Vec<(Node, Option<Slot>)> {
        self.nodes
            .iter()
            .map(|record| (record.id, record.manual))
            .collect()
    }
}

fn transfer() -> Records {
    use Node::*;
    Records::new(&[A, B, Q], &[C, D], &[B, B, C, A])
}
fn transfer_effects() -> Vec<Effect> {
    use Effect::*;
    use Node::*;
    vec![
        Clear(A),
        Clear(B),
        Clear(Q),
        Begin,
        Read(B, None),
        Set(B),
        Append(B),
        Read(B, Some(Slot::Target)),
        Remove(B, Slot::Target),
        Set(B),
        AlreadyPresent(B),
        Read(C, Some(Slot::Other)),
        Remove(C, Slot::Other),
        Set(C),
        Append(C),
        Read(A, None),
        Set(A),
        Append(A),
        Replace,
        AssignTargetRoot,
    ]
}
fn assert_transfer(records: &Records) {
    use Node::*;
    assert_eq!(records.slot(Slot::Target), [B, C, A]);
    assert_eq!(records.slot(Slot::Other), [D]);
    assert_eq!(
        records.pointers(),
        [
            (A, Some(Slot::Target)),
            (B, Some(Slot::Target)),
            (C, Some(Slot::Target)),
            (D, Some(Slot::Other)),
            (Q, None)
        ]
    );
    assert!(records.pending.is_none());
    assert_eq!(records.effects, transfer_effects());
}

#[test]
fn fresh_manual_set_is_empty_only_for_the_actual_qualified_slot_requirement() {
    for (namespace, prefix, local, expected) in [
        (
            Namespace::html(),
            None,
            "slot",
            Some(HtmlNativeFreshSlot::EmptyManualAssignments),
        ),
        (
            Namespace::html(),
            Some("p".into()),
            "slot",
            Some(HtmlNativeFreshSlot::EmptyManualAssignments),
        ),
        (Namespace::html(), None, "Slot", None),
        (Namespace::html(), None, "div", None),
        (Namespace::none(), None, "slot", None),
        (Namespace::svg(), None, "slot", None),
        (Namespace::mathml(), None, "slot", None),
    ] {
        let initial = native_element_initialization_with(
            &QualifiedName::new(namespace, prefix, local),
            |_| Ok::<_, Infallible>(()),
        )
        .unwrap();
        assert_eq!(initial.slot(), expected);
        if expected.is_some() {
            assert_eq!(
                initial.requirement(),
                HtmlElementClass::KnownHtml(KnownHtmlElement::Slot)
            );
        }
    }
}

#[test]
fn repeated_raw_nodes_transfer_before_first_occurrence_append_and_final_replacement() {
    let mut records = transfer();
    records.run(false);
    assert_transfer(&records);
}

#[test]
fn interleaved_duplicates_keep_first_order_and_leave_unrelated_former_entries_in_order() {
    use Node::*;
    let mut records = Records::new(&[], &[Q, A, B, C, D], &[A, B, A, C, B]);
    records.run(false);
    assert_eq!(records.slot(Slot::Target), [A, B, C]);
    assert_eq!(records.slot(Slot::Other), [Q, D]);
    assert_eq!(
        records.pointers(),
        [
            (A, Some(Slot::Target)),
            (B, Some(Slot::Target)),
            (C, Some(Slot::Target)),
            (D, Some(Slot::Other)),
            (Q, Some(Slot::Other))
        ]
    );
    let writes: Vec<_> = records
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                Effect::Remove(..) | Effect::Set(..) | Effect::Append(..)
            )
        })
        .cloned()
        .collect();
    assert_eq!(
        writes,
        [
            Effect::Remove(A, Slot::Other),
            Effect::Set(A),
            Effect::Append(A),
            Effect::Remove(B, Slot::Other),
            Effect::Set(B),
            Effect::Append(B),
            Effect::Remove(A, Slot::Target),
            Effect::Set(A),
            Effect::Remove(C, Slot::Other),
            Effect::Set(C),
            Effect::Append(C),
            Effect::Remove(B, Slot::Target),
            Effect::Set(B),
        ]
    );
    assert_eq!(records.effects.last(), Some(&Effect::AssignTargetRoot));
}

#[test]
fn empty_and_unchanged_intent_still_replace_and_request_only_the_target_root() {
    use Effect::*;
    use Node::*;
    for (old, incoming, expected) in [
        (vec![], vec![], vec![Begin, Replace, AssignTargetRoot]),
        (
            vec![A, B],
            vec![],
            vec![Clear(A), Clear(B), Begin, Replace, AssignTargetRoot],
        ),
        (
            vec![A, B],
            vec![A, B],
            vec![
                Clear(A),
                Clear(B),
                Begin,
                Read(A, None),
                Set(A),
                Append(A),
                Read(B, None),
                Set(B),
                Append(B),
                Replace,
                AssignTargetRoot,
            ],
        ),
    ] {
        let mut records = Records::new(&old, &[], &incoming);
        records.run(false);
        assert_eq!(records.slot(Slot::Target), incoming);
        assert_eq!(records.effects, expected);
        for record in &records.nodes {
            assert_eq!(
                record.manual,
                incoming.contains(&record.id).then_some(Slot::Target)
            );
        }
    }
}

#[test]
fn wrong_replies_preserve_the_pending_action_and_all_subsequent_source_effects() {
    let mut records = transfer();
    let mut cursor = records.run(true);
    assert_transfer(&records);
    for reply in [
        Reply::Applied,
        Reply::ManualAssignmentPresent(false),
        Reply::ManualAssignmentPresent(true),
    ] {
        assert!(matches!(cursor.reply(reply), Err(ref e) if e.is_reply()));
        assert_eq!(cursor.action(), Action::Complete);
    }
}

#[test]
fn every_effect_boundary_is_visible_before_an_adapter_refuses_it() {
    let expected = transfer_effects();
    for stop in 0..expected.len() {
        let mut records = transfer();
        let mut cursor = records.cursor();
        let mut calls = 0;
        let mut refused = None;
        let result = records.run_admitted(&mut cursor, |action| {
            let at = calls;
            calls += 1;
            if at == stop {
                refused = Some(action);
                return Err(stop);
            }
            Ok(())
        });
        assert_eq!(result, Err(stop));
        assert_eq!(calls, stop + 1, "no callback after the first refusal");
        // No operand read/write or acknowledgment followed refused admission.
        assert_eq!(records.effects, expected[..stop]);
        assert_eq!(cursor.action(), refused.unwrap());
    }
}

/// A consumer that keeps incoming nodes by number and answers every identity question from its
/// own records. Returns the number of replies one `slot.assign()` takes and the resulting set.
fn identity_map_replies(incoming: &[usize]) -> (usize, Vec<usize>) {
    use std::collections::HashSet;
    let mut cursor = HtmlSlotAssignCursor::new(0, incoming.len());
    let mut assigned_to_target = HashSet::new();
    let mut pending: Vec<usize> = Vec::new();
    let mut members = HashSet::new();
    let mut replies = 0;
    while cursor.action() != Action::Complete {
        let reply = match cursor.action() {
            Action::ReadIncomingManualAssignment { ordinal } => {
                Reply::ManualAssignmentPresent(assigned_to_target.contains(&incoming[ordinal]))
            }
            Action::SetIncomingManualAssignmentToTarget { ordinal } => {
                assigned_to_target.insert(incoming[ordinal]);
                Reply::Applied
            }
            Action::AppendIncomingToPendingSet { ordinal } => {
                if members.insert(incoming[ordinal]) {
                    pending.push(incoming[ordinal]);
                }
                Reply::Applied
            }
            _ => Reply::Applied,
        };
        cursor.reply(reply).unwrap();
        replies += 1;
    }
    (replies, pending)
}

#[test]
fn a_consumer_with_an_identity_map_takes_a_bounded_number_of_replies_per_incoming_node() {
    let mut previous = None;
    for distinct in [32_usize, 64, 128, 256] {
        // Every node twice: the first occurrences are distinct, the second ones duplicates.
        let incoming: Vec<usize> = (0..distinct).chain(0..distinct).collect();
        let (replies, set) = identity_map_replies(&incoming);
        assert_eq!(set, (0..distinct).collect::<Vec<_>>());
        assert!(
            replies <= 5 * incoming.len() + 4,
            "{} incoming nodes took {replies} replies",
            incoming.len()
        );
        if let Some(earlier) = previous {
            assert!(
                replies * 10 <= earlier * 22,
                "doubling the incoming nodes took {earlier} then {replies} replies"
            );
        }
        previous = Some(replies);
    }
}

#[test]
fn maximum_extents_do_not_require_a_product_allocation_or_overflow_at_construction() {
    let previous = HtmlSlotAssignCursor::new(usize::MAX, usize::MAX);
    assert_eq!(
        previous.action(),
        Action::ClearPreviousManualInverse {
            previous_ordinal: 0
        }
    );
    let mut incoming = HtmlSlotAssignCursor::new(0, usize::MAX);
    assert_eq!(incoming.action(), Action::BeginPendingManualSet);
    incoming.reply(Reply::Applied).unwrap();
    assert_eq!(
        incoming.action(),
        Action::ReadIncomingManualAssignment { ordinal: 0 }
    );
    incoming
        .reply(Reply::ManualAssignmentPresent(false))
        .unwrap();
    incoming.reply(Reply::Applied).unwrap();
    assert_eq!(
        incoming.action(),
        Action::AppendIncomingToPendingSet { ordinal: 0 }
    );
    incoming.reply(Reply::Applied).unwrap();
    assert_eq!(
        incoming.action(),
        Action::ReadIncomingManualAssignment { ordinal: 1 }
    );
}
