//! A child-order change (insert before a sibling, move, detach) describes work
//! bounded by the entries it shifts, never by the siblings it leaves alone, and
//! leaves exactly the order the DOM's insert and remove steps define.
use super::*;

/// Sibling counts around the changed entries.
const FANOUTS: [usize; 3] = [16, 128, 1024];

fn append(
    storage: HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
) -> HtmlConstructionStorage {
    change(
        storage,
        HtmlConstructionCommand::Insert {
            node: node.clone(),
            parent: parent.clone(),
            before: None,
        },
    )
    .0
}

/// Appends `count` created elements under `parent`, returning them in order.
fn fill(
    mut storage: HtmlConstructionStorage,
    parent: &HtmlBuilderNode,
    count: usize,
) -> (HtmlConstructionStorage, Vec<HtmlBuilderNode>) {
    let mut children = Vec::new();
    for _ in 0..count {
        let (next, node) = create(storage, element("c"));
        storage = append(next, &node, parent);
        children.push(node);
    }
    (storage, children)
}

fn children_of(
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

/// Requires every fanout to describe the work the first one did.
fn assert_same_work(label: &str, measured: &[(usize, DescribedWork)]) {
    let (first_fanout, first) = measured[0];
    for (fanout, work) in measured {
        assert_eq!(
            *work, first,
            "{label} among {fanout} siblings described different work than among {first_fanout}"
        );
    }
}

#[test]
fn inserting_before_the_last_child_describes_the_same_work_at_every_fanout() {
    let mut measured = Vec::new();
    for fanout in FANOUTS {
        let (owner, root) = fragment(storage());
        let (owner, children) = fill(owner, &root, fanout);
        let (owner, fresh) = create(owner, element("fresh"));
        let last = children.last().unwrap();
        let (owner, work) = driven_insert(owner, &fresh, &root, Some(last));
        let order = children_of(&owner, &root);
        assert_eq!(order.len(), fanout + 1);
        assert_eq!(order[fanout - 1..], [fresh.clone(), last.clone()]);
        assert_eq!(owner.node(&fresh).unwrap().parent().unwrap().handle(), root);
        measured.push((fanout, work));
    }
    assert_same_work("inserting before the last child", &measured);
}

#[test]
fn moving_the_last_child_before_its_predecessor_describes_the_same_work_at_every_fanout() {
    let mut measured = Vec::new();
    for fanout in FANOUTS {
        let (owner, root) = fragment(storage());
        let (owner, children) = fill(owner, &root, fanout);
        let (last, second) = (&children[fanout - 1], &children[fanout - 2]);
        let (owner, work) = driven_insert(owner, last, &root, Some(second));
        let order = children_of(&owner, &root);
        assert_eq!(order.len(), fanout);
        assert_eq!(order[fanout - 2..], [last.clone(), second.clone()]);
        measured.push((fanout, work));
    }
    assert_same_work("moving the last child before its predecessor", &measured);
}

#[test]
fn moving_the_last_child_to_the_end_of_its_parent_describes_the_same_work_at_every_fanout() {
    let mut measured = Vec::new();
    for fanout in FANOUTS {
        let (owner, root) = fragment(storage());
        let (owner, children) = fill(owner, &root, fanout);
        let last = &children[fanout - 1];
        let (owner, work) = driven_insert(owner, last, &root, None);
        assert_eq!(children_of(&owner, &root), children);
        measured.push((fanout, work));
    }
    assert_same_work("moving the last child to the end", &measured);
}

#[test]
fn moving_the_last_child_into_another_parent_describes_the_same_work_at_every_fanout() {
    let mut measured = Vec::new();
    for fanout in FANOUTS {
        let (owner, from) = fragment(storage());
        let (owner, to) = fragment(owner);
        let (owner, moved) = fill(owner, &from, fanout);
        let (owner, kept) = fill(owner, &to, fanout);
        let node = moved.last().unwrap();
        let target = kept.last().unwrap();
        let (owner, work) = driven_insert(owner, node, &to, Some(target));
        assert_eq!(children_of(&owner, &from), moved[..fanout - 1]);
        let order = children_of(&owner, &to);
        assert_eq!(order.len(), fanout + 1);
        assert_eq!(order[fanout - 1..], [node.clone(), target.clone()]);
        assert_eq!(owner.node(node).unwrap().parent().unwrap().handle(), to);
        measured.push((fanout, work));
    }
    assert_same_work("moving the last child across parents", &measured);
}

#[test]
fn detaching_the_last_child_describes_the_same_work_at_every_fanout() {
    let mut measured = Vec::new();
    for fanout in FANOUTS {
        let (owner, root) = fragment(storage());
        let (owner, children) = fill(owner, &root, fanout);
        let last = children.last().unwrap();
        let (owner, work) = driven(
            owner,
            HtmlConstructionCommand::Detach { node: last.clone() },
        );
        assert_eq!(children_of(&owner, &root), children[..fanout - 1]);
        assert!(owner.node(last).unwrap().parent().is_none());
        measured.push((fanout, work));
    }
    assert_same_work("detaching the last child", &measured);
}

/// A reference model of the DOM's insert and remove steps over child lists.
struct Model {
    parents: Vec<Vec<usize>>,
}

impl Model {
    fn parent_of(&self, node: usize) -> Option<usize> {
        self.parents
            .iter()
            .position(|children| children.contains(&node))
    }
    fn remove(&mut self, node: usize) {
        if let Some(parent) = self.parent_of(node) {
            self.parents[parent].retain(|&child| child != node);
        }
    }
    /// Inserts `node` before `before` under `parent`, or last; false when
    /// `before` is not a child of `parent`.
    fn insert(&mut self, node: usize, parent: usize, before: Option<usize>) -> bool {
        if let Some(before) = before
            && !self.parents[parent].contains(&before)
        {
            return false;
        }
        if before == Some(node) {
            return true;
        }
        self.remove(node);
        let at = before.map_or(self.parents[parent].len(), |before| {
            self.parents[parent]
                .iter()
                .position(|&child| child == before)
                .unwrap()
        });
        self.parents[parent].insert(at, node);
        true
    }
}

#[test]
fn child_orders_match_the_reference_model_through_a_long_sequence_of_changes() {
    const PARENTS: usize = 3;
    const NODES: usize = 9;
    let mut owner = storage();
    let mut parents = Vec::new();
    for _ in 0..PARENTS {
        let (next, parent) = fragment(owner);
        owner = next;
        parents.push(parent);
    }
    let mut nodes = Vec::new();
    for _ in 0..NODES {
        let (next, node) = create(owner, element("n"));
        owner = next;
        nodes.push(node);
    }
    let mut model = Model {
        parents: vec![Vec::new(); PARENTS],
    };
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move |bound: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % bound as u64) as usize
    };
    for step in 0..600 {
        let node = next(NODES);
        let operation = next(5);
        if operation == 0 {
            owner = change(
                owner,
                HtmlConstructionCommand::Detach {
                    node: nodes[node].clone(),
                },
            )
            .0;
            model.remove(node);
        } else {
            let parent = next(PARENTS);
            // Half the insertions name a sibling, a tenth name one that is not
            // a child of the parent.
            let before = match operation {
                1 => None,
                4 => Some(next(NODES)),
                _ => {
                    let children = &model.parents[parent];
                    (!children.is_empty()).then(|| children[next(children.len())])
                }
            };
            let command = HtmlConstructionCommand::Insert {
                node: nodes[node].clone(),
                parent: parents[parent].clone(),
                before: before.map(|before| nodes[before].clone()),
            };
            if model.insert(node, parent, before) {
                owner = change(owner, command).0;
            } else {
                owner = refuse(owner, command, HtmlConstructionError::WrongSibling);
            }
        }
        for (index, parent) in parents.iter().enumerate() {
            let expected: Vec<_> = model.parents[index]
                .iter()
                .map(|&child| nodes[child].clone())
                .collect();
            assert_eq!(children_of(&owner, parent), expected, "step {step}");
        }
        for (index, node) in nodes.iter().enumerate() {
            let actual = owner
                .node(node)
                .unwrap()
                .parent()
                .map(|parent| parents.iter().position(|p| *p == parent.handle()).unwrap());
            assert_eq!(actual, model.parent_of(index), "step {step}");
        }
    }
}
