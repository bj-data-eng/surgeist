//! A child-order change (insert before a sibling, move, detach) leaves exactly the order the DOM's
//! insert and remove steps define.
use super::*;
use crate::html::support::cause;

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
            owner = change(owner, Op::Detach { node: nodes[node] }).0;
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
            let command = Op::Insert {
                node: nodes[node],
                parent: parents[parent],
                before: before.map(|before| nodes[before]),
            };
            if model.insert(node, parent, before) {
                owner = change(owner, command).0;
            } else {
                owner = refuse(owner, command, cause::WRONG_SIBLING);
            }
        }
        for (index, parent) in parents.iter().enumerate() {
            let expected: Vec<_> = model.parents[index]
                .iter()
                .map(|&child| nodes[child])
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
        nodes[0],
        nodes[1],
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
            owner = change(owner, Op::Detach { node: nodes[node] }).0;
            model.remove(node);
        } else if next(12) == 0 {
            // A template element is not a parent: its contents are.
            let template = 2 + next(2);
            owner = refuse(
                owner,
                Op::Insert {
                    node: nodes[node],
                    parent: nodes[template],
                    before: None,
                },
                cause::TEMPLATE_CONTENTS_REQUIRED,
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
            let command = Op::Insert {
                node: nodes[node],
                parent: containers[container],
                before: before.map(|before| nodes[before]),
            };
            let stranger = before.is_some_and(|b| model.parent_of(b) != Some(container));
            if stranger {
                owner = refuse(owner, command, cause::WRONG_SIBLING);
                strangers += 1;
            } else if before == Some(node) {
                owner = change(owner, command).0;
            } else if model.cycle(node, container) {
                owner = refuse(owner, command, cause::CYCLE);
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
                .map(|&child| nodes[child])
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
