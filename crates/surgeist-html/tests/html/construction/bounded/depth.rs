//! Node depths stay valid across every topology change: the usage depth is the greatest depth of
//! any record, as an independent model of the forest says.
use super::*;

/// Inserts `node` under `parent`, appended last, and returns the storage.
fn append(
    storage: HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
) -> HtmlConstructionStorage {
    change(
        storage,
        Op::Insert {
            node: *node,
            parent: *parent,
            before: None,
        },
    )
    .0
}

/// A forest model: the independent authority for every record's depth.
struct Forest {
    parent: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
}

impl Forest {
    fn new(len: usize) -> Self {
        Self {
            parent: vec![None; len],
            children: vec![Vec::new(); len],
        }
    }
    fn depth(&self, mut node: usize) -> usize {
        let mut depth = 0;
        while let Some(parent) = self.parent[node] {
            node = parent;
            depth += 1;
        }
        depth
    }
    fn max_depth(&self) -> usize {
        (0..self.parent.len())
            .map(|node| self.depth(node))
            .max()
            .unwrap_or(0)
    }
    fn is_within(&self, mut node: usize, ancestor: usize) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            match self.parent[node] {
                Some(parent) => node = parent,
                None => return false,
            }
        }
    }
    fn detach(&mut self, node: usize) {
        if let Some(parent) = self.parent[node].take() {
            self.children[parent].retain(|child| *child != node);
        }
    }
    fn insert(&mut self, node: usize, parent: usize, before: Option<usize>) {
        self.detach(node);
        let position = before
            .and_then(|before| {
                self.children[parent]
                    .iter()
                    .position(|child| *child == before)
            })
            .unwrap_or(self.children[parent].len());
        self.children[parent].insert(position, node);
        self.parent[node] = Some(parent);
    }
}

/// A deterministic pseudo-random sequence; the seed is the only input.
struct Sequence(u64);

impl Sequence {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound
    }
}

#[test]
fn usage_depth_follows_an_independent_forest_through_random_moves_and_detaches() {
    const ROOTS: usize = 2;
    const ELEMENTS: usize = 24;
    for seed in [1, 2, 3, 4] {
        let mut sequence = Sequence(seed);
        let mut owner = storage();
        let mut handles = Vec::new();
        for _ in 0..ROOTS {
            let (next, root) = fragment(owner);
            owner = next;
            handles.push(root);
        }
        for _ in 0..ELEMENTS {
            let (next, node) = create(owner, element("e"));
            owner = next;
            handles.push(node);
        }
        let mut forest = Forest::new(ROOTS + ELEMENTS);
        let (mut rises, mut falls) = (0, 0);
        for step in 0..400 {
            let node = ROOTS + sequence.next(ELEMENTS);
            let before_depth = forest.max_depth();
            if sequence.next(4) == 0 {
                forest.detach(node);
                owner = change(
                    owner,
                    Op::Detach {
                        node: handles[node],
                    },
                )
                .0;
            } else {
                let parent = sequence.next(ROOTS + ELEMENTS);
                if forest.is_within(parent, node) {
                    continue;
                }
                let siblings: Vec<usize> = forest.children[parent]
                    .iter()
                    .copied()
                    .filter(|child| *child != node)
                    .collect();
                let before = (!siblings.is_empty() && sequence.next(3) == 0)
                    .then(|| siblings[sequence.next(siblings.len())]);
                forest.insert(node, parent, before);
                owner = change(
                    owner,
                    Op::Insert {
                        node: handles[node],
                        parent: handles[parent],
                        before: before.map(|before| handles[before]),
                    },
                )
                .0;
            }
            let expected = forest.max_depth();
            rises += usize::from(expected > before_depth);
            falls += usize::from(expected < before_depth);
            assert_eq!(
                owner.usage().depth(),
                expected,
                "seed {seed}, step {step}: the usage depth differs from the forest's"
            );
        }
        // The sequence must exercise both directions, or it proves little.
        assert!(
            rises > 5 && falls > 5,
            "seed {seed}: {rises} rises, {falls} falls"
        );
    }
}

#[test]
fn depth_stays_exact_after_shadow_and_template_changes_that_move_whole_subtrees() {
    let (owner, root) = fragment(storage());
    let (owner, host) = create(owner, element("div"));
    let owner = append(owner, &host, &root);
    let (owner, shadow) = create(
        owner,
        Make::ShadowContents {
            properties: properties(),
        },
    );
    // A chain a > b > c below the shadow contents: depths 1, 2 and 3 there.
    let (owner, a) = create(owner, element("a"));
    let (owner, b) = create(owner, element("b"));
    let (owner, c) = create(owner, element("c"));
    let owner = append(owner, &c, &b);
    let owner = append(owner, &b, &a);
    let owner = append(owner, &a, &shadow);
    assert_eq!(owner.usage().depth(), 3);
    // Attaching the shadow root puts its contents at depth 2 below the host.
    let (owner, _) = change(
        owner,
        Op::AttachShadow {
            host,
            contents: shadow,
        },
    );
    assert_eq!(owner.usage().depth(), 5);
    let (owner, d) = create(owner, element("d"));
    let owner = append(owner, &d, &c);
    assert_eq!(owner.usage().depth(), 6);
    let (owner, e) = create(owner, element("e"));
    let owner = append(owner, &e, &d);
    assert_eq!(owner.usage().depth(), 7);
    // Moving the chain's top out of the shadow tree and under the host lifts
    // it by one level: a at 2, b 3, c 4, d 5, e 6.
    let owner = append(owner, &a, &host);
    assert_eq!(owner.usage().depth(), 6);
    // A template counts its contents as one more level, and moving the template
    // moves them with it.
    let (owner, template) = create(owner, element("template"));
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, x) = create(owner, element("x"));
    let owner = append(owner, &x, &contents);
    assert_eq!(owner.usage().depth(), 6);
    let owner = append(owner, &template, &e);
    // template 7, contents 8, x 9.
    assert_eq!(owner.usage().depth(), 9);
    let (owner, f) = create(owner, element("f"));
    let owner = append(owner, &f, &x);
    assert_eq!(owner.usage().depth(), 10);
    // Lifting the template to the root lowers everything it owns.
    let (owner, _) = change(owner, Op::Detach { node: template });
    assert_eq!(owner.usage().depth(), 6);
    let owner = append(owner, &template, &root);
    assert_eq!(owner.usage().depth(), 6);
    let (owner, g) = create(owner, element("g"));
    let owner = append(owner, &g, &f);
    // template 1, contents 2, x 3, f 4, g 5.
    assert_eq!(owner.usage().depth(), 6);
    let result = publish(owner, root);
    assert_eq!(result.usage().depth(), 6);
}

#[test]
fn lifting_the_deepest_subtree_leaves_the_deepest_record_elsewhere() {
    let (mut owner, tree) = fragment(storage());
    let (next, other) = fragment(owner);
    owner = next;
    // A chain of four below `other`: its deepest record is at depth 4.
    let mut parent = other;
    for _ in 0..4 {
        let (next, node) = create(owner, element("g"));
        owner = append(next, &node, &parent);
        parent = node;
    }
    let chain_end = parent;
    // A chain of three below `tree`, then a chain s1 > s2 > s3 below those:
    // s1 is at depth 4 and s3, the deepest record, at 6.
    let mut parent = tree;
    for _ in 0..3 {
        let (next, node) = create(owner, element("t"));
        owner = append(next, &node, &parent);
        parent = node;
    }
    let (next, top) = create(owner, element("s"));
    owner = append(next, &top, &parent);
    parent = top;
    for _ in 0..2 {
        let (next, node) = create(owner, element("s"));
        owner = append(next, &node, &parent);
        parent = node;
    }
    assert_eq!(owner.usage().depth(), 6);
    // Lifting s1 to a root leaves the other chain's depth 4 as the deepest.
    let (owner, _) = change(owner, Op::Detach { node: top });
    assert_eq!(owner.usage().depth(), 4);
    // Moving it below the other chain's end puts s1 at 5 and s3 at 7.
    let owner = append(owner, &top, &chain_end);
    assert_eq!(owner.usage().depth(), 7);
}

#[test]
fn depth_stays_exact_after_a_template_takes_a_shadow_reference_and_moves() {
    let (owner, root) = fragment(storage());
    let (owner, host) = create(owner, element("div"));
    let owner = append(owner, &host, &root);
    let (owner, template) = create(owner, element("template"));
    let owner = append(owner, &template, &root);
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    // A chain p > q below the template contents: p at 3 and q at 4.
    let (owner, p) = create(owner, element("p"));
    let (owner, q) = create(owner, element("q"));
    let owner = append(owner, &p, &contents);
    let owner = append(owner, &q, &p);
    assert_eq!(owner.usage().depth(), 4);
    let (owner, shadow) = create(
        owner,
        Make::ShadowContents {
            properties: properties(),
        },
    );
    let (owner, r) = create(owner, element("r"));
    let owner = append(owner, &r, &shadow);
    let (owner, _) = change(
        owner,
        Op::AttachTemplateShadow {
            template,
            host,
            contents: shadow,
        },
    );
    // The old contents are a root again: p at 1 and q at 2. The shadow
    // contents sit below the host at 2, and r at 3.
    assert_eq!(owner.usage().depth(), 3);
    let (owner, s) = create(owner, element("s"));
    let owner = append(owner, &s, &q);
    assert_eq!(owner.usage().depth(), 3);
    let (owner, t) = create(owner, element("t"));
    let owner = append(owner, &t, &s);
    assert_eq!(owner.usage().depth(), 4);
    let (owner, u) = create(owner, element("u"));
    let (owner, v) = create(owner, element("v"));
    let owner = append(owner, &u, &r);
    let owner = append(owner, &v, &u);
    assert_eq!(owner.usage().depth(), 5);
    // Lifting the template to a root leaves its shadow reference out of the depth: the old
    // contents (p 1, q 2, s 3, t 4) and the shadow chain (host 1, shadow contents 2, r 3, u 4,
    // v 5) are the deepest records, at 4 and 5.
    let (owner, _) = change(owner, Op::Detach { node: template });
    assert_eq!(owner.usage().depth(), 5);
    // Under v at 5 it puts the template at 6. A reference to shadow contents is not owned
    // contents, so nothing below the template is lowered or lifted with it.
    let owner = append(owner, &template, &v);
    assert_eq!(owner.usage().depth(), 6);
}

#[test]
fn lifting_the_deepest_record_out_of_template_contents_leaves_the_contents_level() {
    // The template is at depth 0 and its contents at 1: after the leaf below
    // the contents is lifted out, the contents are the deepest record.
    let (owner, template) = create(storage(), element("template"));
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, leaf) = create(owner, element("leaf"));
    let owner = append(owner, &leaf, &contents);
    assert_eq!(owner.usage().depth(), 2);
    let (owner, _) = change(owner, Op::Detach { node: leaf });
    assert_eq!(owner.usage().depth(), 1);
}

#[test]
fn lifting_the_deepest_record_after_a_full_recomputation_counts_each_level_once() {
    let (owner, root) = fragment(storage());
    let (owner, a) = create(owner, element("a"));
    let (owner, b) = create(owner, element("b"));
    let (owner, c) = create(owner, element("c"));
    let owner = append(owner, &c, &b);
    let owner = append(owner, &b, &a);
    let owner = append(owner, &a, &root);
    assert_eq!(owner.usage().depth(), 3);
    // Giving a template shadow contents rewrites every record's depth.
    let (owner, template) = create(owner, element("template"));
    let (owner, shadow) = create(
        owner,
        Make::ShadowContents {
            properties: properties(),
        },
    );
    let (owner, _) = change(
        owner,
        Op::SetTemplateShadowContents {
            template,
            contents: shadow,
        },
    );
    assert_eq!(owner.usage().depth(), 3);
    // Only b, at depth 2, remains as deep as the lifted leaf's old level less one.
    let (owner, _) = change(owner, Op::Detach { node: c });
    assert_eq!(owner.usage().depth(), 2);
}
