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

/// Performs every action of `command` and returns the storage with the
/// `record_writes` and `copied_bytes` its Commit action described.
fn commit_description(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, usize, usize) {
    let mut operation = Some(
        storage
            .prepare(command)
            .unwrap_or_else(|_| panic!("preparation")),
    );
    let mut described = None;
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        if let HtmlConstructionActionDescription::Commit {
            record_writes,
            copied_bytes,
            ..
        } = action.description()
        {
            assert!(described.replace((record_writes, copied_bytes)).is_none());
        }
        action.perform().unwrap().accept();
    }
    let HtmlConstructionCommit::Changed { storage, .. } =
        HtmlConstructionOperation::try_ready(&mut operation)
            .unwrap()
            .commit()
    else {
        panic!("mutation produced publication")
    };
    let (writes, bytes) = described.expect("a commit action");
    (storage, writes, bytes)
}

/// Changes at the front of an order, among `n` and `2n` siblings, whose commit
/// shifts the entries behind the change. The expected growth of the described
/// writes is derived by hand for each, from the order length `n`, where an
/// order edit writes one word per entry written or shifted.
#[test]
fn changes_at_the_front_of_an_order_describe_the_shifted_entries() {
    type Change =
        fn(&HtmlBuilderNode, &[HtmlBuilderNode], &HtmlBuilderNode) -> HtmlConstructionCommand;
    // (name, command, entries the order edit writes among n siblings)
    type Case = (&'static str, Change, fn(usize) -> usize);
    let cases: [Case; 3] = [
        (
            "insert before the first child",
            |root, children, fresh| HtmlConstructionCommand::Insert {
                node: fresh.clone(),
                parent: root.clone(),
                before: Some(children[0].clone()),
            },
            // The parent link (1), the inserted entry (1) and the n entries
            // behind the insertion place, which move one place right: n + 2.
            |n| n + 2,
        ),
        (
            "detach the first child",
            |_, children, _| HtmlConstructionCommand::Detach {
                node: children[0].clone(),
            },
            // The parent link (1), the removed entry (1) and the n - 1 entries
            // behind it, which move one place left: n + 1.
            |n| n + 1,
        ),
        (
            "move the first child to the end",
            |root, children, _| HtmlConstructionCommand::Insert {
                node: children[0].clone(),
                parent: root.clone(),
                before: None,
            },
            // The parent link (1), the removed entry (1) and the n - 1 entries
            // behind it moving left, then the re-inserted entry (1) at the end
            // of the n - 1 entries left, with nothing behind it: n + 2.
            |n| n + 2,
        ),
    ];
    for (name, change, entries) in cases {
        for n in [16, 128] {
            let described = |siblings: usize| {
                let (owner, root) = fragment(storage());
                let (owner, children) = fill(owner, &root, siblings);
                let (owner, fresh) = create(owner, element("fresh"));
                let command = change(&root, &children, &fresh);
                let (_, writes, bytes) = commit_description(owner, command);
                (writes, bytes)
            };
            let (writes, bytes) = described(n);
            let (double_writes, double_bytes) = described(2 * n);
            let extra = entries(2 * n) - entries(n);
            assert_eq!(extra, n, "{name}: the hand derivation grows by n");
            assert_eq!(
                double_writes - writes,
                extra,
                "{name}: writes among {n} and {} siblings",
                2 * n
            );
            assert_eq!(
                double_bytes - bytes,
                extra * size_of::<usize>(),
                "{name}: copied bytes among {n} and {} siblings",
                2 * n
            );
        }
    }
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

/// Containers: the root (0), two elements (1, 2) and the contents of two
/// templates (3, 4). Nodes: the two container elements (0, 1), the two
/// template elements (2, 3) and two leaves (4, 5).
struct TreeModel {
    containers: Vec<Vec<usize>>,
}

impl TreeModel {
    /// The node a container belongs to: an element is its own container, and a
    /// template owns its contents; the root belongs to no node.
    fn node_of_container(container: usize) -> Option<usize> {
        match container {
            1..=4 => Some(container - 1),
            _ => None,
        }
    }
    fn parent_of(&self, node: usize) -> Option<usize> {
        self.containers
            .iter()
            .position(|children| children.contains(&node))
    }
    /// Whether placing `node` in `container` would put it inside itself,
    /// following parents and template ownership upward.
    fn cycle(&self, node: usize, container: usize) -> bool {
        let mut cursor = Some(container);
        while let Some(current) = cursor {
            match Self::node_of_container(current) {
                Some(owner) if owner == node => return true,
                Some(owner) => cursor = self.parent_of(owner),
                None => return false,
            }
        }
        false
    }
    fn remove(&mut self, node: usize) {
        if let Some(parent) = self.parent_of(node) {
            self.containers[parent].retain(|&child| child != node);
        }
    }
    fn insert(&mut self, node: usize, container: usize, before: Option<usize>) {
        self.remove(node);
        let at = before.map_or(self.containers[container].len(), |before| {
            self.containers[container]
                .iter()
                .position(|&child| child == before)
                .unwrap()
        });
        self.containers[container].insert(at, node);
    }
}

#[test]
fn child_orders_match_the_reference_model_with_template_contents_and_cycles() {
    let mut owner = storage();
    let (next, root) = fragment(owner);
    owner = next;
    let mut nodes = Vec::new();
    for local in ["e", "e", "template", "template", "leaf", "leaf"] {
        let (next, node) = create(owner, element(local));
        owner = next;
        nodes.push(node);
    }
    let contents = |owner: &HtmlConstructionStorage, template: &HtmlBuilderNode| {
        owner
            .node(template)
            .unwrap()
            .template_contents()
            .expect("an HTML template owns contents")
            .handle()
    };
    let containers = [
        root,
        nodes[0].clone(),
        nodes[1].clone(),
        contents(&owner, &nodes[2]),
        contents(&owner, &nodes[3]),
    ];
    let mut model = TreeModel {
        containers: vec![Vec::new(); containers.len()],
    };
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move |bound: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % bound as u64) as usize
    };
    let (mut cycles, mut templates, mut strangers, mut moves) = (0, 0, 0, 0);
    for step in 0..1500 {
        let node = next(nodes.len());
        if next(6) == 0 {
            owner = change(
                owner,
                HtmlConstructionCommand::Detach {
                    node: nodes[node].clone(),
                },
            )
            .0;
            model.remove(node);
        } else if next(12) == 0 {
            // A template element is not a parent: its contents are.
            let template = 2 + next(2);
            owner = refuse(
                owner,
                HtmlConstructionCommand::Insert {
                    node: nodes[node].clone(),
                    parent: nodes[template].clone(),
                    before: None,
                },
                HtmlConstructionError::TemplateContentsRequired,
            );
            templates += 1;
        } else {
            let container = next(containers.len());
            let before = match next(4) {
                0 => None,
                1 => Some(next(nodes.len())),
                _ => {
                    let children = &model.containers[container];
                    (!children.is_empty()).then(|| children[next(children.len())])
                }
            };
            let command = HtmlConstructionCommand::Insert {
                node: nodes[node].clone(),
                parent: containers[container].clone(),
                before: before.map(|before| nodes[before].clone()),
            };
            let stranger = before.is_some_and(|b| model.parent_of(b) != Some(container));
            if stranger {
                owner = refuse(owner, command, HtmlConstructionError::WrongSibling);
                strangers += 1;
            } else if before == Some(node) {
                owner = change(owner, command).0;
            } else if model.cycle(node, container) {
                owner = refuse(owner, command, HtmlConstructionError::Cycle);
                cycles += 1;
            } else {
                moves += usize::from(model.parent_of(node).is_some());
                owner = change(owner, command).0;
                model.insert(node, container, before);
            }
        }
        for (index, container) in containers.iter().enumerate() {
            let expected: Vec<_> = model.containers[index]
                .iter()
                .map(|&child| nodes[child].clone())
                .collect();
            assert_eq!(children_of(&owner, container), expected, "step {step}");
        }
        for (index, node) in nodes.iter().enumerate() {
            let actual = owner.node(node).unwrap().parent().map(|parent| {
                containers
                    .iter()
                    .position(|c| *c == parent.handle())
                    .unwrap()
            });
            assert_eq!(actual, model.parent_of(index), "step {step}");
        }
    }
    // The sequence reached every refusal and many reparenting moves.
    assert!(cycles > 20 && templates > 20 && strangers > 20 && moves > 100);
}
