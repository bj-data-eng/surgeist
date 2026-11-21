use super::*;
use crate::internal::identity::DomOwnerId;
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Error {
    TransitionDenied,
    RelationDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Relation {
    Parent,
    FirstChild,
    LastChild,
    PreviousSibling,
    NextSibling,
}

struct Row {
    id: NodeId,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

/// Ordinary-tree relations are live between protocol calls. The fake permits
/// mutation only between calls, just as synchronous filtering does; it does not
/// model DOM custody/publication, which belong to the integration boundary.
struct Tree {
    rows: RefCell<Vec<Row>>,
    transitions: Cell<usize>,
    reads: RefCell<Vec<(Relation, NodeId)>>,
    deny_transition: Cell<bool>,
    deny_relation: Cell<Option<Relation>>,
}

impl Tree {
    fn new(parents: &[Option<usize>]) -> Self {
        let owner = DomOwnerId::allocate().expect("test owner identity");
        let ids: Vec<_> = (0..parents.len())
            .map(|slot| NodeId {
                owner,
                slot,
                generation: 1,
            })
            .collect();
        let mut rows: Vec<_> = ids
            .iter()
            .copied()
            .map(|id| Row {
                id,
                parent: None,
                children: Vec::new(),
            })
            .collect();
        for (slot, parent) in parents.iter().enumerate() {
            if let Some(parent) = *parent {
                rows[slot].parent = Some(ids[parent]);
                rows[parent].children.push(ids[slot]);
            }
        }
        Self {
            rows: RefCell::new(rows),
            transitions: Cell::new(0),
            reads: RefCell::new(Vec::new()),
            deny_transition: Cell::new(false),
            deny_relation: Cell::new(None),
        }
    }

    // 0
    // ├─ 1
    // │  ├─ 2
    // │  └─ 3
    // └─ 4
    //    └─ 5
    fn branching() -> Self {
        Self::new(&[None, Some(0), Some(1), Some(1), Some(0), Some(4)])
    }

    fn id(&self, slot: usize) -> NodeId {
        self.rows.borrow()[slot].id
    }

    fn reparent(&self, slot: usize, parent: Option<usize>) {
        let mut rows = self.rows.borrow_mut();
        let id = rows[slot].id;
        if let Some(old) = rows[slot].parent {
            rows[old.slot].children.retain(|child| *child != id);
        }
        rows[slot].parent = parent.map(|parent| rows[parent].id);
        if let Some(parent) = parent {
            rows[parent].children.push(id);
        }
    }

    fn read(&self, relation: Relation, node: NodeId) -> Result<(), Error> {
        self.reads.borrow_mut().push((relation, node));
        if self.deny_relation.get() == Some(relation) {
            return Err(Error::RelationDenied);
        }
        assert_eq!(self.rows.borrow()[node.slot].id, node);
        Ok(())
    }

    fn sibling(&self, node: NodeId, forward: bool) -> Option<NodeId> {
        let rows = self.rows.borrow();
        let parent = rows[node.slot].parent?;
        let siblings = &rows[parent.slot].children;
        let ordinal = siblings.iter().position(|sibling| *sibling == node)?;
        let ordinal = if forward {
            ordinal.checked_add(1)?
        } else {
            ordinal.checked_sub(1)?
        };
        siblings.get(ordinal).copied()
    }
}

impl OrdinaryTree for Tree {
    type Error = Error;

    fn step(&self) -> Result<(), Error> {
        self.transitions.set(self.transitions.get() + 1);
        if self.deny_transition.get() {
            Err(Error::TransitionDenied)
        } else {
            Ok(())
        }
    }

    fn parent(&self, node: NodeId) -> Result<Option<NodeId>, Error> {
        self.read(Relation::Parent, node)?;
        Ok(self.rows.borrow()[node.slot].parent)
    }

    fn first_child(&self, node: NodeId) -> Result<Option<NodeId>, Error> {
        self.read(Relation::FirstChild, node)?;
        Ok(self.rows.borrow()[node.slot].children.first().copied())
    }

    fn last_child(&self, node: NodeId) -> Result<Option<NodeId>, Error> {
        self.read(Relation::LastChild, node)?;
        Ok(self.rows.borrow()[node.slot].children.last().copied())
    }

    fn previous_sibling(&self, node: NodeId) -> Result<Option<NodeId>, Error> {
        self.read(Relation::PreviousSibling, node)?;
        Ok(self.sibling(node, false))
    }

    fn next_sibling(&self, node: NodeId) -> Result<Option<NodeId>, Error> {
        self.read(Relation::NextSibling, node)?;
        Ok(self.sibling(node, true))
    }
}

fn pending(progress: Progress, expected: NodeId) -> Filter {
    let Progress::Filter(filter) = progress else {
        panic!("expected filtering of {expected:?}, got {progress:?}");
    };
    assert_eq!(filter.node(), expected);
    filter
}

fn matched(progress: Progress, expected: NodeId) {
    assert!(
        matches!(progress, Progress::Match(node) if node == expected),
        "expected match {expected:?}, got {progress:?}"
    );
}

fn none(progress: Progress) {
    assert!(
        matches!(progress, Progress::None),
        "expected no match, got {progress:?}"
    );
}

fn finish(tree: &Tree, mut progress: Progress, current: NodeId, decisions: &[u16]) -> Progress {
    loop {
        match progress {
            Progress::Filter(filter) => {
                let result = decisions[filter.node().slot];
                progress = filter
                    .resume(tree, current, result)
                    .expect("navigation resume");
            }
            terminal => return terminal,
        }
    }
}

#[test]
fn all_directions_follow_their_ordinary_tree_order() {
    // Independently derived from the pinned parent/child/sibling/tree orders.
    for (direction, current, expected) in [
        (DomTreeWalkerDirection::Parent, 1, 0),
        (DomTreeWalkerDirection::FirstChild, 0, 1),
        (DomTreeWalkerDirection::LastChild, 0, 4),
        (DomTreeWalkerDirection::PreviousSibling, 4, 1),
        (DomTreeWalkerDirection::NextSibling, 1, 4),
        (DomTreeWalkerDirection::PreviousNode, 4, 3),
        (DomTreeWalkerDirection::NextNode, 1, 2),
    ] {
        let tree = Tree::branching();
        let current = tree.id(current);
        let progress = start(&tree, tree.id(0), current, direction).expect("start");
        matched(
            finish(&tree, progress, current, &[ACCEPT; 6]),
            tree.id(expected),
        );
    }
}

#[test]
fn root_bounds_parent_sibling_and_previous_but_allows_descendants() {
    let tree = Tree::branching();
    let root = tree.id(0);
    for direction in [
        DomTreeWalkerDirection::Parent,
        DomTreeWalkerDirection::PreviousSibling,
        DomTreeWalkerDirection::NextSibling,
        DomTreeWalkerDirection::PreviousNode,
    ] {
        none(start(&tree, root, root, direction).expect("start at root"));
    }
    for direction in [
        DomTreeWalkerDirection::FirstChild,
        DomTreeWalkerDirection::NextNode,
    ] {
        let progress = start(&tree, root, root, direction).expect("start descendants");
        matched(finish(&tree, progress, root, &[ACCEPT; 6]), tree.id(1));
    }
}

#[test]
fn child_skip_exposes_descendants_while_reject_and_unknown_prune_them() {
    for (decision, expected) in [(SKIP, 2), (REJECT, 4), (0, 4), (u16::MAX, 4)] {
        let tree = Tree::branching();
        let root = tree.id(0);
        let filter = pending(
            start(&tree, root, root, DomTreeWalkerDirection::FirstChild).expect("start"),
            tree.id(1),
        );
        let progress = filter.resume(&tree, root, decision).expect("resume");
        matched(
            finish(&tree, progress, root, &[ACCEPT; 6]),
            tree.id(expected),
        );
    }
}

#[test]
fn unknown_sibling_and_next_results_expose_descendants() {
    let tree = Tree::branching();
    for (direction, current, candidate, descendant) in [
        (DomTreeWalkerDirection::NextSibling, 1, 4, 5),
        (DomTreeWalkerDirection::PreviousSibling, 4, 1, 3),
        (DomTreeWalkerDirection::NextNode, 0, 1, 2),
        (DomTreeWalkerDirection::PreviousNode, 4, 1, 3),
    ] {
        let current = tree.id(current);
        let filter = pending(
            start(&tree, tree.id(0), current, direction).expect("start"),
            tree.id(candidate),
        );
        let progress = filter
            .resume(&tree, current, u16::MAX)
            .expect("unknown result");
        matched(
            finish(&tree, progress, current, &[ACCEPT; 6]),
            tree.id(descendant),
        );
    }
}

#[test]
fn child_climb_tests_live_current_instead_of_initial_current() {
    let tree = Tree::branching();
    let initial = tree.id(1);
    let filter = pending(
        start(
            &tree,
            tree.id(0),
            initial,
            DomTreeWalkerDirection::LastChild,
        )
        .expect("start"),
        tree.id(3),
    );
    let progress = filter
        .resume(&tree, tree.id(0), REJECT)
        .expect("resume with changed current");
    let filter = pending(progress, tree.id(2));
    // With initial current 1, the climb would stop there. Live current 0 makes
    // the climb continue through 1 and read its preceding sibling (none).
    none(filter.resume(&tree, tree.id(0), REJECT).expect("resume"));
    assert!(
        tree.reads
            .borrow()
            .contains(&(Relation::PreviousSibling, tree.id(1)))
    );
}

#[test]
fn child_climb_can_reach_outer_sibling_after_current_changes() {
    let tree = Tree::branching();
    let initial = tree.id(1);
    let filter = pending(
        start(
            &tree,
            tree.id(0),
            initial,
            DomTreeWalkerDirection::FirstChild,
        )
        .expect("start"),
        tree.id(2),
    );
    let filter = pending(
        filter.resume(&tree, tree.id(0), REJECT).expect("resume"),
        tree.id(3),
    );
    let filter = pending(
        filter.resume(&tree, tree.id(0), REJECT).expect("resume"),
        tree.id(4),
    );
    matched(
        filter.resume(&tree, tree.id(0), ACCEPT).expect("accept"),
        tree.id(4),
    );
}

#[test]
fn previous_accept_reads_new_last_child_after_filter() {
    let tree = Tree::branching();
    let filter = pending(
        start(
            &tree,
            tree.id(0),
            tree.id(4),
            DomTreeWalkerDirection::PreviousNode,
        )
        .expect("start"),
        tree.id(1),
    );
    tree.reparent(5, Some(1));
    let filter = pending(
        filter.resume(&tree, tree.id(4), ACCEPT).expect("resume"),
        tree.id(5),
    );
    matched(
        filter
            .resume(&tree, tree.id(4), ACCEPT)
            .expect("accept leaf"),
        tree.id(5),
    );
}

#[test]
fn next_reject_uses_live_sibling_after_filter_reparents_it() {
    let tree = Tree::branching();
    let root = tree.id(0);
    let filter = pending(
        start(&tree, root, root, DomTreeWalkerDirection::NextNode).expect("start"),
        tree.id(1),
    );
    tree.reparent(4, Some(1));
    none(
        filter
            .resume(&tree, root, REJECT)
            .expect("live sibling lookup"),
    );
}

#[test]
fn detached_candidate_is_still_the_match_after_accept() {
    let tree = Tree::branching();
    let root = tree.id(0);
    let filter = pending(
        start(&tree, root, root, DomTreeWalkerDirection::FirstChild).expect("start"),
        tree.id(1),
    );
    tree.reparent(1, None);
    matched(
        filter
            .resume(&tree, tree.id(5), ACCEPT)
            .expect("accept original candidate"),
        tree.id(1),
    );
}

#[test]
fn outside_root_current_follows_real_relations_without_ancestry_rejection() {
    let tree = Tree::new(&[None, None, Some(1), Some(1)]);
    let root = tree.id(0);
    for (direction, current, expected) in [
        (DomTreeWalkerDirection::Parent, 2, 1),
        (DomTreeWalkerDirection::FirstChild, 1, 2),
        (DomTreeWalkerDirection::LastChild, 1, 3),
        (DomTreeWalkerDirection::PreviousSibling, 3, 2),
        (DomTreeWalkerDirection::NextSibling, 2, 3),
        (DomTreeWalkerDirection::PreviousNode, 2, 1),
        (DomTreeWalkerDirection::NextNode, 1, 2),
    ] {
        let current = tree.id(current);
        let progress = start(&tree, root, current, direction).expect("outside root");
        matched(
            finish(&tree, progress, current, &[ACCEPT; 4]),
            tree.id(expected),
        );
    }
    for direction in [
        DomTreeWalkerDirection::Parent,
        DomTreeWalkerDirection::PreviousNode,
    ] {
        none(start(&tree, root, tree.id(1), direction).expect("detached current"));
    }
}

#[test]
fn sibling_parent_accept_ends_traversal_without_returning_parent() {
    let tree = Tree::branching();
    let current = tree.id(3);
    let filter = pending(
        start(
            &tree,
            tree.id(0),
            current,
            DomTreeWalkerDirection::NextSibling,
        )
        .expect("start"),
        tree.id(1),
    );
    none(
        filter
            .resume(&tree, current, ACCEPT)
            .expect("accepted parent bounds siblings"),
    );
}

#[test]
fn sibling_parent_skip_exposes_its_next_sibling() {
    let tree = Tree::branching();
    let current = tree.id(3);
    let filter = pending(
        start(
            &tree,
            tree.id(0),
            current,
            DomTreeWalkerDirection::NextSibling,
        )
        .expect("start"),
        tree.id(1),
    );
    let filter = pending(
        filter.resume(&tree, current, SKIP).expect("skip parent"),
        tree.id(4),
    );
    matched(
        filter
            .resume(&tree, current, ACCEPT)
            .expect("accept sibling"),
        tree.id(4),
    );
}

#[test]
fn sibling_reject_still_observes_child_read_failure() {
    let tree = Tree::branching();
    let current = tree.id(1);
    let filter = pending(
        start(
            &tree,
            tree.id(0),
            current,
            DomTreeWalkerDirection::NextSibling,
        )
        .expect("start"),
        tree.id(4),
    );
    tree.deny_relation.set(Some(Relation::FirstChild));
    assert!(matches!(
        filter.resume(&tree, current, REJECT),
        Err(Error::RelationDenied)
    ));
}

#[test]
fn denied_transition_prevents_relation_reads_and_terminal_match() {
    let tree = Tree::branching();
    let root = tree.id(0);
    tree.deny_transition.set(true);
    assert!(matches!(
        start(&tree, root, root, DomTreeWalkerDirection::NextNode),
        Err(Error::TransitionDenied)
    ));
    assert!(tree.reads.borrow().is_empty());
    tree.deny_transition.set(false);
    let filter = pending(
        start(&tree, root, root, DomTreeWalkerDirection::NextNode).expect("start"),
        tree.id(1),
    );
    let reads_before_resume = tree.reads.borrow().len();
    tree.deny_transition.set(true);
    assert!(matches!(
        filter.resume(&tree, root, ACCEPT),
        Err(Error::TransitionDenied)
    ));
    assert_eq!(tree.reads.borrow().len(), reads_before_resume);
}

#[test]
fn parent_reject_keeps_climbing_and_can_return_root() {
    let tree = Tree::branching();
    let current = tree.id(2);
    let filter = pending(
        start(&tree, tree.id(0), current, DomTreeWalkerDirection::Parent).expect("start"),
        tree.id(1),
    );
    let filter = pending(
        filter
            .resume(&tree, current, REJECT)
            .expect("climb rejected parent"),
        tree.id(0),
    );
    matched(
        filter.resume(&tree, current, ACCEPT).expect("accept root"),
        tree.id(0),
    );
}

#[test]
fn node_reject_prunes_descendants_in_both_orders() {
    let tree = Tree::branching();
    let root = tree.id(0);
    let filter = pending(
        start(&tree, root, root, DomTreeWalkerDirection::NextNode).expect("next"),
        tree.id(1),
    );
    let filter = pending(
        filter.resume(&tree, root, REJECT).expect("skip subtree"),
        tree.id(4),
    );
    matched(
        filter
            .resume(&tree, root, ACCEPT)
            .expect("accept following subtree"),
        tree.id(4),
    );

    let current = tree.id(4);
    let filter = pending(
        start(&tree, root, current, DomTreeWalkerDirection::PreviousNode).expect("previous"),
        tree.id(1),
    );
    let filter = pending(
        filter.resume(&tree, current, REJECT).expect("skip subtree"),
        root,
    );
    matched(
        filter
            .resume(&tree, current, ACCEPT)
            .expect("accept parent"),
        root,
    );
}
