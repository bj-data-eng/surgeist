//! Node depths stay valid across every topology change, so one change's depth
//! work depends on the depth or the moved subtree and never on the record count.
use super::*;

/// Inserts `node` under `parent`, appended last, and returns the storage.
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

/// Creates `count` elements appended under `parent`.
fn filler(
    mut storage: HtmlConstructionStorage,
    parent: &HtmlBuilderNode,
    count: usize,
) -> HtmlConstructionStorage {
    for _ in 0..count {
        let (next, node) = create(storage, element("filler"));
        storage = append(next, &node, parent);
    }
    storage
}

/// Appends a created element with a created child, so that the element is a
/// non-shallow subtree whose insertion changes topology below its own record.
fn subtree_with_child(
    storage: HtmlConstructionStorage,
    parent: &HtmlBuilderNode,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let (storage, node) = create(storage, element("subtree"));
    let (storage, child) = create(storage, element("child"));
    let storage = append(storage, &child, &node);
    (append(storage, &node, parent), node)
}

#[test]
fn appending_a_leaf_after_a_subtree_move_describes_the_same_work_at_every_record_count() {
    let mut measured = Vec::new();
    for unrelated in [16, 128, 1024] {
        let (owner, root) = fragment(storage());
        let owner = filler(owner, &root, unrelated);
        // A non-shallow insertion is the topology change that used to leave
        // every later insertion recomputing the depth of every record.
        let (owner, _) = subtree_with_child(owner, &root);
        let (owner, leaf) = create(owner, element("measured"));
        let (_, work) = driven_insert(owner, &leaf, &root, None);
        measured.push((unrelated, work));
    }
    let (first_count, first) = measured[0];
    for (count, work) in &measured {
        assert_eq!(
            *work, first,
            "appending a leaf beside {count} unrelated records described different work \
             than beside {first_count}"
        );
    }
}

#[test]
fn moving_a_subtree_describes_the_same_work_at_every_unrelated_record_count() {
    let mut measured = Vec::new();
    for unrelated in [16, 128, 1024] {
        let (owner, unrelated_root) = fragment(storage());
        let owner = filler(owner, &unrelated_root, unrelated);
        let (owner, from) = create(owner, element("from"));
        let (owner, to) = create(owner, element("to"));
        let (owner, moved) = subtree_with_child(owner, &from);
        let (owner, _) = subtree_with_child(owner, &to);
        // The move lowers the subtree by one level and is non-shallow.
        let (_, work) = driven_insert(owner, &moved, &to, None);
        measured.push((unrelated, work));
    }
    let (first_count, first) = measured[0];
    for (count, work) in &measured {
        assert_eq!(
            *work, first,
            "moving a subtree beside {count} unrelated records described different work \
             than beside {first_count}"
        );
    }
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
                    HtmlConstructionCommand::Detach {
                        node: handles[node].clone(),
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
                    HtmlConstructionCommand::Insert {
                        node: handles[node].clone(),
                        parent: handles[parent].clone(),
                        before: before.map(|before| handles[before].clone()),
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
        HtmlConstructionCommand::CreateShadowContents {
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
        HtmlConstructionCommand::AttachShadow {
            host: host.clone(),
            contents: shadow.clone(),
        },
    );
    assert_eq!(owner.usage().depth(), 5);
    let (owner, d) = create(owner, element("d"));
    let owner = append(owner, &d, &c);
    assert_eq!(owner.usage().depth(), 6);
    // Detaching it makes the contents a root again: d sits at depth 4.
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::DetachShadow { host: host.clone() },
    );
    assert_eq!(owner.usage().depth(), 4);
    let (owner, e) = create(owner, element("e"));
    let owner = append(owner, &e, &d);
    assert_eq!(owner.usage().depth(), 5);
    // Moving the chain's top under the host: a at 2, b 3, c 4, d 5, e 6.
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
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Detach {
            node: template.clone(),
        },
    );
    assert_eq!(owner.usage().depth(), 6);
    let owner = append(owner, &template, &root);
    assert_eq!(owner.usage().depth(), 6);
    let (owner, g) = create(owner, element("g"));
    let owner = append(owner, &g, &f);
    // template 1, contents 2, x 3, f 4, g 5.
    assert_eq!(owner.usage().depth(), 6);
    let (result, _) = publish(owner, root);
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
    parent = top.clone();
    for _ in 0..2 {
        let (next, node) = create(owner, element("s"));
        owner = append(next, &node, &parent);
        parent = node;
    }
    assert_eq!(owner.usage().depth(), 6);
    // Lifting s1 to a root leaves the other chain's depth 4 as the deepest.
    let (owner, _) = change(owner, HtmlConstructionCommand::Detach { node: top.clone() });
    assert_eq!(owner.usage().depth(), 4);
    // Moving it below the other chain's end puts s1 at 5 and s3 at 7.
    let owner = append(owner, &top, &chain_end);
    assert_eq!(owner.usage().depth(), 7);
}

#[test]
fn depth_stays_exact_after_template_shadow_and_contents_reassignment() {
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
        HtmlConstructionCommand::CreateShadowContents {
            properties: properties(),
        },
    );
    let (owner, r) = create(owner, element("r"));
    let owner = append(owner, &r, &shadow);
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::AttachTemplateShadow {
            template: template.clone(),
            host: host.clone(),
            contents: shadow.clone(),
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
    // Reassigning the old contents puts them below the template at depth 2:
    // p at 3, q 4, s 5 and t 6.
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::ReassignTemplateContents {
            template: template.clone(),
            contents: contents.clone(),
        },
    );
    assert_eq!(owner.usage().depth(), 6);
    let (owner, w) = create(owner, element("w"));
    let owner = append(owner, &w, &t);
    assert_eq!(owner.usage().depth(), 7);
    // Lifting the template to a root lowers its records: contents 1, p 2,
    // q 3, s 4, t 5 and w 6, while the shadow chain reaches 5.
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::Detach {
            node: template.clone(),
        },
    );
    assert_eq!(owner.usage().depth(), 6);
    // Under v at 5 it puts the template at 6, its contents at 7 and w at 12.
    let owner = append(owner, &template, &v);
    assert_eq!(owner.usage().depth(), 12);
}

/// A detached chain of `len` elements, each the only child of the one above.
fn chain(
    mut storage: HtmlConstructionStorage,
    len: usize,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let (next, top) = create(storage, element("link"));
    storage = next;
    let mut parent = top.clone();
    for _ in 1..len {
        let (next, node) = create(storage, element("link"));
        storage = append(next, &node, &parent);
        parent = node;
    }
    (storage, top)
}

/// The work described by moving a chain of `length` elements one level deeper.
fn described_chain_move(length: usize) -> DescribedWork {
    let (owner, from) = create(storage(), element("from"));
    let (owner, below) = create(owner, element("below"));
    let (owner, to) = create(owner, element("to"));
    let owner = append(owner, &to, &below);
    let (owner, top) = chain(owner, length);
    let owner = append(owner, &top, &from);
    // The chain moves one level deeper, below `to` rather than `from`.
    driven_insert(owner, &top, &to, None).1
}

#[test]
fn moving_a_subtree_describes_at_least_three_writes_per_record_it_moves() {
    // Each moved record's depth and the two histogram counts it leaves and
    // joins are three writes, so the described writes grow by at least that
    // for every record a longer chain adds to the move.
    let (short, long) = (4usize, 36usize);
    let (short_work, long_work) = (described_chain_move(short), described_chain_move(long));
    assert!(
        long_work.record_writes >= short_work.record_writes + 3 * (long - short),
        "moving {long} records described {} writes, moving {short} described {}",
        long_work.record_writes,
        short_work.record_writes
    );
}

#[test]
fn moving_a_subtree_describes_a_visit_for_every_record_the_commit_walk_reads() {
    // The commit walks the moved subtree again to place each record, entering
    // and leaving every record, so the described visits grow by at least one
    // per record a longer chain adds.
    let (short, long) = (4usize, 36usize);
    let (short_work, long_work) = (described_chain_move(short), described_chain_move(long));
    assert!(
        long_work.record_visits >= short_work.record_visits + 2 * (long - short),
        "moving {long} records described {} visits, moving {short} described {}",
        long_work.record_visits,
        short_work.record_visits
    );
}

#[test]
fn detaching_a_subtree_describes_the_same_work_at_every_unrelated_record_count() {
    let mut measured = Vec::new();
    for unrelated in [16, 128, 1024] {
        let (owner, unrelated_root) = fragment(storage());
        let owner = filler(owner, &unrelated_root, unrelated);
        let (owner, parent) = create(owner, element("parent"));
        let (owner, moved) = subtree_with_child(owner, &parent);
        let (_, work) = driven(owner, HtmlConstructionCommand::Detach { node: moved });
        measured.push((unrelated, work));
    }
    let (first_count, first) = measured[0];
    for (count, work) in &measured {
        assert_eq!(
            *work, first,
            "detaching a subtree beside {count} unrelated records described different work \
             than beside {first_count}"
        );
    }
}

#[test]
fn attaching_a_shadow_root_describes_the_same_work_at_every_unrelated_record_count() {
    let mut measured = Vec::new();
    for unrelated in [16, 128, 1024] {
        let (owner, unrelated_root) = fragment(storage());
        let owner = filler(owner, &unrelated_root, unrelated);
        let (owner, host) = create(owner, element("host"));
        let (owner, shadow) = create(
            owner,
            HtmlConstructionCommand::CreateShadowContents {
                properties: properties(),
            },
        );
        // Non-shallow contents, so attaching walks a subtree.
        let (owner, child) = create(owner, element("child"));
        let owner = append(owner, &child, &shadow);
        let (_, work) = driven(
            owner,
            HtmlConstructionCommand::AttachShadow {
                host,
                contents: shadow,
            },
        );
        measured.push((unrelated, work));
    }
    let (first_count, first) = measured[0];
    for (count, work) in &measured {
        assert_eq!(
            *work, first,
            "attaching a shadow root beside {count} unrelated records described different work \
             than beside {first_count}"
        );
    }
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
    let (owner, _) = change(owner, HtmlConstructionCommand::Detach { node: leaf });
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
        HtmlConstructionCommand::CreateShadowContents {
            properties: properties(),
        },
    );
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::SetTemplateShadowContents {
            template,
            contents: shadow,
        },
    );
    assert_eq!(owner.usage().depth(), 3);
    // Only b, at depth 2, remains as deep as the lifted leaf's old level less one.
    let (owner, _) = change(owner, HtmlConstructionCommand::Detach { node: c });
    assert_eq!(owner.usage().depth(), 2);
}

/// Asserts that `measure` describes the same work beside 16, 128 and 1024
/// unrelated records.
fn assert_same_work_at_every_unrelated_record_count(
    what: &str,
    measure: impl Fn(HtmlConstructionStorage) -> DescribedWork,
) {
    let mut measured = Vec::new();
    for unrelated in [16, 128, 1024] {
        let (owner, unrelated_root) = fragment(storage());
        let owner = filler(owner, &unrelated_root, unrelated);
        measured.push((unrelated, measure(owner)));
    }
    let (first_count, first) = measured[0];
    for (count, work) in &measured {
        assert_eq!(
            *work, first,
            "{what} beside {count} unrelated records described different work than beside \
             {first_count}"
        );
    }
}

/// A detached template whose contents hold one element.
fn template_with_child(
    storage: HtmlConstructionStorage,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let (storage, template) = create(storage, element("template"));
    let contents = storage
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (storage, child) = create(storage, element("child"));
    (append(storage, &child, &contents), template)
}

/// Unowned shadow contents that hold one element.
fn shadow_with_child(
    storage: HtmlConstructionStorage,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let (storage, shadow) = create(
        storage,
        HtmlConstructionCommand::CreateShadowContents {
            properties: properties(),
        },
    );
    let (storage, child) = create(storage, element("child"));
    (append(storage, &child, &shadow), shadow)
}

#[test]
fn giving_a_template_a_shadow_host_describes_the_same_work_at_every_unrelated_record_count() {
    assert_same_work_at_every_unrelated_record_count("attaching template shadow", |owner| {
        let (owner, template) = template_with_child(owner);
        let (owner, host) = create(owner, element("host"));
        let (owner, shadow) = shadow_with_child(owner);
        driven(
            owner,
            HtmlConstructionCommand::AttachTemplateShadow {
                template,
                host,
                contents: shadow,
            },
        )
        .1
    });
}

#[test]
fn replacing_template_contents_with_shadow_contents_describes_the_same_work_at_every_unrelated_record_count()
 {
    assert_same_work_at_every_unrelated_record_count("setting template shadow contents", |owner| {
        let (owner, template) = template_with_child(owner);
        let (owner, shadow) = shadow_with_child(owner);
        driven(
            owner,
            HtmlConstructionCommand::SetTemplateShadowContents {
                template,
                contents: shadow,
            },
        )
        .1
    });
}

#[test]
fn detaching_a_shadow_root_describes_the_same_work_at_every_unrelated_record_count() {
    assert_same_work_at_every_unrelated_record_count("detaching a shadow root", |owner| {
        let (owner, host) = create(owner, element("host"));
        let (owner, shadow) = shadow_with_child(owner);
        let (owner, _) = change(
            owner,
            HtmlConstructionCommand::AttachShadow {
                host: host.clone(),
                contents: shadow,
            },
        );
        driven(owner, HtmlConstructionCommand::DetachShadow { host }).1
    });
}

#[test]
fn reassigning_template_contents_describes_the_same_work_at_every_unrelated_record_count() {
    assert_same_work_at_every_unrelated_record_count("reassigning template contents", |owner| {
        let (owner, template) = template_with_child(owner);
        let (owner, replacement) = fragment(owner);
        let (owner, child) = create(owner, element("child"));
        let owner = append(owner, &child, &replacement);
        driven(
            owner,
            HtmlConstructionCommand::ReassignTemplateContents {
                template,
                contents: replacement,
            },
        )
        .1
    });
}

/// The record visits and record writes the Commit action of `command` describes.
fn commit_description(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (usize, usize) {
    let mut operation = Some(
        storage
            .prepare(command)
            .unwrap_or_else(|_| panic!("preparation")),
    );
    let mut described = None;
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        if let HtmlConstructionActionDescription::Commit {
            record_visits,
            record_writes,
            ..
        } = action.description()
        {
            described = Some((record_visits, record_writes));
        }
        action.perform().unwrap().accept();
    }
    drop(operation);
    described.expect("every operation ends in a Commit action")
}

// The commit's cost model: a depth walk makes one edge lookup per step, each
// costing 2 record visits and 1 cursor write; each placed record costs 4 writes
// (cursor reset, its depth, and the two histogram counts it leaves and joins);
// a fast-path record costs 3 writes (its depth and the two histogram counts).
// An append also writes the usage, the child push and the parent link.

#[test]
fn appending_a_leaf_describes_the_fast_path_commit_work() {
    // 1 usage + 1 child push + 1 parent link, and 3 for the one fast-path leaf;
    // a fast path walks nothing, so no visits.
    let (owner, root) = fragment(storage());
    let (owner, leaf) = create(owner, element("leaf"));
    let command = HtmlConstructionCommand::Insert {
        node: leaf,
        parent: root,
        before: None,
    };
    assert_eq!(commit_description(owner, command), (0, 1 + 1 + 1 + 3));
}

#[test]
fn appending_a_template_describes_the_fast_path_commit_work_of_both_records() {
    // As a leaf, but the template and its empty contents are two fast-path
    // records: 3 fixed writes and 2 * 3.
    let (owner, root) = fragment(storage());
    let (owner, template) = create(owner, element("template"));
    let command = HtmlConstructionCommand::Insert {
        node: template,
        parent: root,
        before: None,
    };
    assert_eq!(commit_description(owner, command), (0, 1 + 1 + 1 + 2 * 3));
}

#[test]
fn moving_a_two_record_subtree_describes_its_walk_and_placements() {
    // The same move under a parent at the same depth walks nothing, so its
    // description is the fixed part; subtracting it leaves the depth work of
    // moving `moved` and its one child one level deeper.
    //
    // The walk from `moved` makes 3 edge lookups: `moved` yields its child, the
    // child yields nothing, and `moved` again yields nothing. That is 3 * 2 = 6
    // visits. The writes are 3 cursor writes plus 4 for each of the 2 placed
    // records: 3 + 2 * 4 = 11.
    let measure = |deeper: bool| {
        let (owner, from) = create(storage(), element("from"));
        let (owner, to) = create(owner, element("to"));
        let owner = if deeper {
            let (owner, above) = create(owner, element("above"));
            append(owner, &to, &above)
        } else {
            owner
        };
        let (owner, moved) = create(owner, element("moved"));
        let (owner, child) = create(owner, element("child"));
        let owner = append(owner, &child, &moved);
        let owner = append(owner, &moved, &from);
        commit_description(
            owner,
            HtmlConstructionCommand::Insert {
                node: moved,
                parent: to,
                before: None,
            },
        )
    };
    let (flat_visits, flat_writes) = measure(false);
    let (deeper_visits, deeper_writes) = measure(true);
    assert_eq!(flat_visits, 0);
    assert_eq!(
        (deeper_visits - flat_visits, deeper_writes - flat_writes),
        (6, 11)
    );
}

#[test]
fn giving_a_template_a_shadow_host_describes_both_moved_subtrees() {
    // Two subtrees move: the template's old contents, detached to depth zero,
    // and the shadow contents, attached below the host. Each holds `children`
    // elements. Subtracting the commit of the empty shape from that of the
    // shape with one child in each leaves the work the children add.
    //
    // Each child adds, to its subtree's walk, one lookup that enters it and one
    // that returns: 2 more lookups, so 2 * 2 = 4 more visits and 2 more cursor
    // writes, and 1 more placed record, so 4 more writes: 2 + 4 = 6 writes.
    // Two subtrees double both: 8 visits and 12 writes.
    let measure = |children: usize| {
        let (owner, template) = create(storage(), element("template"));
        let contents = owner
            .node(&template)
            .unwrap()
            .template_contents()
            .unwrap()
            .handle();
        let (owner, host) = create(owner, element("host"));
        let (owner, shadow) = create(
            owner,
            HtmlConstructionCommand::CreateShadowContents {
                properties: properties(),
            },
        );
        let mut owner = owner;
        for parent in [&contents, &shadow] {
            for _ in 0..children {
                let (next, child) = create(owner, element("child"));
                owner = append(next, &child, parent);
            }
        }
        commit_description(
            owner,
            HtmlConstructionCommand::AttachTemplateShadow {
                template,
                host,
                contents: shadow,
            },
        )
    };
    let (empty_visits, empty_writes) = measure(0);
    let (full_visits, full_writes) = measure(1);
    assert_eq!(
        (full_visits - empty_visits, full_writes - empty_writes),
        (8, 12)
    );
}

#[test]
fn attaching_a_new_templates_contents_as_a_shadow_describes_the_same_work_at_every_host_depth() {
    // The host ends a chain of `depth` elements. The template is new, so it is
    // detached and its contents are empty: the host cannot lie inside them, and
    // its depth is the one the chain gives it, whatever the chain's length.
    let mut measured = Vec::new();
    for depth in [16, 128, 1024] {
        let (mut owner, root) = fragment(storage());
        let mut host = root;
        for _ in 0..depth {
            let (next, node) = create(owner, element("div"));
            owner = append(next, &node, &host);
            host = node;
        }
        let (owner, template) = create(owner, element("template"));
        let (owner, shadow) = create(
            owner,
            HtmlConstructionCommand::CreateShadowContents {
                properties: properties(),
            },
        );
        let (_, work) = driven(
            owner,
            HtmlConstructionCommand::AttachTemplateShadow {
                template,
                host,
                contents: shadow,
            },
        );
        measured.push((depth, work));
    }
    let (first_depth, first) = measured[0];
    for (depth, work) in &measured {
        assert_eq!(
            *work, first,
            "attaching a shadow to a host at depth {depth} described different work \
             than at depth {first_depth}"
        );
    }
}
