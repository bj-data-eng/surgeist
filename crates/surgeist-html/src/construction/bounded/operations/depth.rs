//! Exact node depths across topology changes.
//!
//! Every record caches its depth within its own tree, and the arena's usage
//! depth is the greatest cached depth. A topology change moves at most two
//! subtrees to new depths, so only their caches change: planning walks each
//! subtree once, one action per step, and the commit places each of its records
//! at the new depth before the structural writes, so both see the same edges.
//!
//! The usage depth after the change is the greater of the moved subtrees' new
//! depths and the greatest depth of every other record. The latter is the old
//! usage depth unless a moved subtree held the deepest record and moved
//! shallower, when a scan down the arena's depth histogram, less the moved
//! subtrees' own counts at each level, finds it. The scan is bounded by the
//! depth.
use super::*;

/// One subtree a topology change moves, and what planning learned of it.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Move {
    root: usize,
    /// The root's cached depth before the change.
    from: usize,
    /// The root's depth after the change.
    to: usize,
    /// The deepest level reached below the root so far.
    height: usize,
    /// Records reached so far, including the root.
    size: usize,
    /// Edge lookups the walk made, which the commit's walk repeats.
    calls: usize,
}

impl Move {
    /// Whether the move changes any depth.
    fn walked(&self) -> bool {
        self.from != self.to
    }
    fn top(&self) -> usize {
        self.from.saturating_add(self.height)
    }
    fn new_top(&self) -> usize {
        self.to.saturating_add(self.height)
    }
    /// Whether the walk counted records at the old depth `level`.
    fn counted(&self, level: usize) -> bool {
        self.walked() && level >= self.from && level <= self.top()
    }
}

/// How a topology change updates the cached depths.
#[derive(Clone, Copy, Debug, Default)]
pub(super) enum DepthPlan {
    /// No depth changes.
    #[default]
    Unchanged,
    /// A fresh leaf, or a template with only its empty contents, joins at one
    /// known depth and cannot lower any other record's.
    Fast {
        node: usize,
        depth: usize,
        /// The node, and its contents when it is a template.
        records: usize,
    },
    /// Up to two subtrees move: one detached from its owner and one attached.
    Moves { moves: [Move; 2], count: usize },
}

impl DepthPlan {
    fn moves(&self) -> impl Iterator<Item = &Move> {
        let (moves, count) = match self {
            Self::Moves { moves, count } => (&moves[..], *count),
            _ => (&[][..], 0),
        };
        moves.iter().take(count)
    }
    fn mov(&mut self, slot: usize) -> Result<&mut Move, HtmlConstructionError> {
        match self {
            Self::Moves { moves, count } if slot < *count => moves
                .get_mut(slot)
                .ok_or(HtmlConstructionError::OperationNotReady),
            _ => Err(HtmlConstructionError::OperationNotReady),
        }
    }
    /// The record visits and writes the commit's depth work performs: per
    /// edge lookup two visits and one cursor write, and per placed record a
    /// cursor reset, its depth, and the two histogram counts it leaves and joins.
    pub(super) fn commit_work(&self) -> Result<(usize, usize), HtmlConstructionError> {
        match self {
            Self::Unchanged => Ok((0, 0)),
            Self::Fast { records, .. } => Ok((0, records.checked_mul(3).ok_or(OVERFLOW)?)),
            Self::Moves { .. } => {
                let (mut visits, mut writes) = (0usize, 0usize);
                for mov in self.moves().filter(|mov| mov.walked()) {
                    visits = add(visits, mov.calls.checked_mul(2).ok_or(OVERFLOW)?)?;
                    writes = add(
                        add(writes, mov.calls)?,
                        mov.size.checked_mul(4).ok_or(OVERFLOW)?,
                    )?;
                }
                Ok((visits, writes))
            }
        }
    }
}

const OVERFLOW: HtmlConstructionError = HtmlConstructionError::RepresentationExhausted;

/// The most steps a walk of an arena of `records` can take: each record is
/// entered and left once, and a record names at most three kinds of edge.
fn walk_budget(records: usize) -> usize {
    records.saturating_mul(3).saturating_add(3)
}

/// The record that owns `record` in its tree: its parent, or the host or
/// template that owns the fragment.
pub(super) fn owning_parent(record: &Record) -> Option<usize> {
    record.parent.or(match &record.owner {
        Some(HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) => {
            Some(owner.index)
        }
        None => None,
    })
}

/// The next record below `node` that its walk has not entered: ordinary
/// children in order, then the shadow contents, then the owned template
/// contents, except `skip`. Advances the node's cursor past every edge it
/// reports or skips, with one write.
fn next_edge(
    arena: &mut Arena,
    node: usize,
    skip: Option<usize>,
) -> Result<Option<usize>, HtmlConstructionError> {
    let record = arena
        .records
        .get_mut(node)
        .ok_or(HtmlConstructionError::UnknownNode)?;
    let children = record.children.len();
    let mut cursor = record.walk_cursor;
    let edge = loop {
        let candidate = if cursor < children {
            record.children.get(cursor).copied()
        } else if cursor == children {
            record.shadow.as_ref().map(|shadow| shadow.contents.index)
        } else if cursor == children.saturating_add(1) {
            match record.template_contents {
                Some(TemplateContents::Owned(contents)) if Some(contents) != skip => Some(contents),
                _ => None,
            }
        } else {
            break None;
        };
        cursor = cursor.saturating_add(1);
        if candidate.is_some() {
            break candidate;
        }
    };
    record.walk_cursor = cursor;
    Ok(edge)
}

impl Arena {
    /// Places the subtree rooted at `root` so that the root sits at depth `to`,
    /// leaving out the template contents `skip`. The walk was planned, so it
    /// cannot fail; an inconsistent arena only ends it early.
    pub(super) fn relocate_subtree(&mut self, root: usize, to: usize, skip: Option<usize>) {
        let budget = walk_budget(self.records.len());
        let Some(record) = self.records.get_mut(root) else {
            return;
        };
        record.walk_cursor = 0;
        self.place(root, to);
        let (mut node, mut level) = (root, 0usize);
        for _ in 0..budget {
            match next_edge(self, node, skip) {
                Ok(Some(child)) => {
                    let Some(record) = self.records.get_mut(child) else {
                        return;
                    };
                    record.walk_cursor = 0;
                    level = level.saturating_add(1);
                    self.place(child, to.saturating_add(level));
                    node = child;
                }
                Ok(None) if node == root => return,
                Ok(None) => {
                    let Some(parent) = self.records.get(node).and_then(owning_parent) else {
                        return;
                    };
                    node = parent;
                    level = level.saturating_sub(1);
                }
                Err(_) => return,
            }
        }
    }
}

impl HtmlConstructionOperation {
    /// Settles the usage depth the change leaves, and the depth work its commit
    /// performs, before any caller observes the computed usage.
    pub(super) fn topology(&mut self) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        let p = self.plan.positions;
        let old = self.plan.old_contents;
        let missing = HtmlConstructionError::InvalidTemplateAssociation;
        // The roots to move and where each goes: below a record, or detached.
        let (detached, attached): (Option<usize>, Option<(usize, usize)>) = match self.command() {
            Command::Publish { .. } => return Ok(Phase::Layout),
            Command::Insert { .. } => (None, Some((p[0], p[1]))),
            Command::Detach { .. } => (Some(p[0]), None),
            Command::AttachShadow { .. } => (None, Some((p[1], p[0]))),
            Command::DetachShadow { .. } | Command::SetTemplateShadowContents { .. } => {
                (Some(old.ok_or(missing)?), None)
            }
            Command::ReassignTemplateContents { .. } => (old, Some((p[1], p[0]))),
            Command::AttachTemplateShadow { .. } => (Some(old.ok_or(missing)?), Some((p[2], p[1]))),
            _ => return Ok(Phase::Layout),
        };
        if let (Some((node, parent)), None) = (attached, detached)
            && let Some(fast) = self.fast_depth(node, parent)?
        {
            return Ok(fast);
        }
        let arena = self.arena();
        let root_move = |root: usize, to: usize| -> Result<Move, HtmlConstructionError> {
            Ok(Move {
                root,
                from: arena
                    .records
                    .get(root)
                    .ok_or(HtmlConstructionError::UnknownNode)?
                    .depth,
                to,
                height: 0,
                size: 1,
                calls: 0,
            })
        };
        let depth_of = |index: usize| {
            arena
                .records
                .get(index)
                .map(|record| record.depth)
                .ok_or(HtmlConstructionError::UnknownNode)
        };
        let first = detached.map(|root| root_move(root, 0)).transpose()?;
        // The host of a template shadow may sit inside the contents the template
        // gives up, so its depth is measured after the change, by a walk.
        let host = match self.command() {
            Command::AttachTemplateShadow { .. } => attached.map(|(_, host)| host),
            _ => None,
        };
        let second = match attached {
            Some((root, _)) if host.is_some() => Some(root_move(root, 0)?),
            Some((root, parent)) => Some(root_move(root, add(depth_of(parent)?, 1)?)?),
            None => None,
        };
        let (moves, count) = match (first, second) {
            (Some(first), Some(second)) => ([first, second], 2),
            (Some(only), None) | (None, Some(only)) => ([only, Move::default()], 1),
            (None, None) => ([Move::default(); 2], 0),
        };
        self.plan.depth = DepthPlan::Moves { moves, count };
        match host {
            Some(host) => Ok(Phase::Host {
                cursor: host,
                depth: 0,
                steps: 0,
            }),
            None => self.begin_moves(0),
        }
    }
    /// The fresh-leaf case: a shallow node joins below `parent` at one depth.
    fn fast_depth(
        &mut self,
        node: usize,
        parent: usize,
    ) -> Result<Option<Phase>, HtmlConstructionError> {
        if self.plan.old_parent.is_some() {
            return Ok(None);
        }
        let arena = self.arena();
        let record = arena
            .records
            .get(node)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        let contents = record.template_contents.and_then(|value| match value {
            TemplateContents::Owned(index) => Some(index),
            TemplateContents::ShadowReference(_) => None,
        });
        let shallow = record.children.is_empty()
            && record.shadow.is_none()
            && contents.is_none_or(|index| {
                arena.records.get(index).is_some_and(|contents| {
                    contents.children.is_empty() && contents.shadow.is_none()
                })
            });
        if !shallow {
            return Ok(None);
        }
        let depth = add(
            arena
                .records
                .get(parent)
                .ok_or(HtmlConstructionError::UnknownNode)?
                .depth,
            1,
        )?;
        let template = usize::from(contents.is_some());
        let records = 1 + template;
        self.plan.usage.depth = self.plan.usage.depth.max(add(depth, template)?);
        self.plan.depth = DepthPlan::Fast {
            node,
            depth,
            records,
        };
        self.before_writes().map(Some)
    }
    /// One step up the host's chain of owners, as it stands after the change.
    pub(super) fn host_step(
        &mut self,
        cursor: usize,
        depth: usize,
        steps: usize,
    ) -> Result<Phase, HtmlConstructionError> {
        let records = self.arena().records.len();
        if cursor >= records {
            return Err(HtmlConstructionError::UnknownNode);
        }
        if steps >= records {
            return Err(HtmlConstructionError::Cycle);
        }
        // The contents the template gives up stop being owned by it.
        let parent = if Some(cursor) == self.plan.old_contents {
            None
        } else {
            structural_parent(self.arena(), cursor)
        };
        match parent {
            Some(parent) => Ok(Phase::Host {
                cursor: parent,
                depth: add(depth, 1)?,
                steps: steps + 1,
            }),
            None => {
                let mov = self.plan.depth.mov(1)?;
                mov.to = add(depth, 1)?;
                self.begin_moves(0)
            }
        }
    }
    /// Starts walking the first move from `slot` on that changes a depth, or
    /// settles the usage depth when no move remains.
    fn begin_moves(&mut self, slot: usize) -> Result<Phase, HtmlConstructionError> {
        for slot in slot..2 {
            let Ok(mov) = self.plan.depth.mov(slot).copied() else {
                break;
            };
            if !mov.walked() {
                continue;
            }
            let arena = self.arena_mut()?;
            let root = arena
                .records
                .get_mut(mov.root)
                .ok_or(HtmlConstructionError::UnknownNode)?;
            root.walk_cursor = 0;
            arena
                .records
                .get_mut(mov.from)
                .ok_or(HtmlConstructionError::UnknownNode)?
                .level_count = 1;
            return Ok(Phase::Shift {
                slot,
                node: mov.root,
                level: 0,
                steps: 0,
            });
        }
        self.settle_depth()
    }
    /// One step of the planning walk over a moved subtree: counts the records
    /// at each old depth below the root and its height.
    pub(super) fn shift_step(
        &mut self,
        slot: usize,
        node: usize,
        level: usize,
        steps: usize,
    ) -> Result<Phase, HtmlConstructionError> {
        let mut mov = *self.plan.depth.mov(slot)?;
        if steps > walk_budget(self.arena().records.len()) {
            return Err(HtmlConstructionError::Cycle);
        }
        let skip = self.plan.old_contents;
        let edge = next_edge(self.arena_mut()?, node, skip)?;
        let Some(child) = edge else {
            if node == mov.root {
                self.plan.depth.mov(slot)?.calls = add(steps, 1)?;
                return self.begin_moves(slot + 1);
            }
            let up = self
                .arena()
                .records
                .get(node)
                .and_then(owning_parent)
                .ok_or(HtmlConstructionError::ParentChildConflict)?;
            return Ok(Phase::Shift {
                slot,
                node: up,
                level: level
                    .checked_sub(1)
                    .ok_or(HtmlConstructionError::ParentChildConflict)?,
                steps: add(steps, 1)?,
            });
        };
        let level = add(level, 1)?;
        let absolute = add(mov.from, level)?;
        // A walk descends one level at a time, so a deeper level is new.
        let fresh = level > mov.height;
        if fresh {
            mov.height = level;
        }
        let arena = self.arena_mut()?;
        arena
            .records
            .get_mut(child)
            .ok_or(HtmlConstructionError::UnknownNode)?
            .walk_cursor = 0;
        let bucket = arena
            .records
            .get_mut(absolute)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        if fresh {
            bucket.level_count = 0;
        }
        bucket.level_count = add(bucket.level_count, 1)?;
        mov.size = add(mov.size, 1)?;
        *self.plan.depth.mov(slot)? = mov;
        Ok(Phase::Shift {
            slot,
            node: child,
            level,
            steps: add(steps, 1)?,
        })
    }
    /// Every walk is complete: settles the usage depth, or scans for it.
    fn settle_depth(&mut self) -> Result<Phase, HtmlConstructionError> {
        let current = self.plan.usage.depth;
        let moves = || self.plan.depth.moves().filter(|mov| mov.walked());
        let new_top = moves().map(Move::new_top).max();
        let Some(new_top) = new_top else {
            return self.before_writes();
        };
        if new_top >= current || moves().all(|mov| mov.top() < current) {
            // Some record outside the moved subtrees is as deep as the old
            // usage depth, or a moved subtree itself reaches the new one.
            self.plan.usage.depth = current.max(new_top);
            return self.before_writes();
        }
        // A moved subtree held the deepest record and every one moves
        // shallower: the depth is the deepest level that holds a record
        // outside them.
        Ok(Phase::Settle { level: current })
    }
    /// One level of the scan for the deepest record outside the moved subtrees.
    ///
    /// Each level's count of moved records lives in the record at that index. A
    /// second walk may overwrite the first's counts, but only at levels no
    /// deeper than its own new top: it starts at a fresh root, depth zero, and
    /// moves deeper, so every level it counts is at most that top. The scan
    /// reads only levels deeper than the greatest new top, which no overwrite
    /// reaches.
    pub(super) fn settle_step(&mut self, level: usize) -> Result<Phase, HtmlConstructionError> {
        let new_top = self
            .plan
            .depth
            .moves()
            .filter(|mov| mov.walked())
            .map(Move::new_top)
            .max()
            .unwrap_or(0);
        if level <= new_top {
            self.plan.usage.depth = new_top;
            return self.before_writes();
        }
        let arena = self.arena();
        let inside = if self.plan.depth.moves().any(|mov| mov.counted(level)) {
            arena
                .records
                .get(level)
                .map_or(0, |bucket| bucket.level_count)
        } else {
            0
        };
        if arena.population(level) > inside {
            self.plan.usage.depth = level;
            return self.before_writes();
        }
        Ok(Phase::Settle {
            level: level.saturating_sub(1),
        })
    }
}

impl Plan {
    /// The depth work the commit performs, before its structural writes so
    /// that it sees the edges planning walked.
    pub(super) fn commit_depths(&self, arena: &mut Arena) {
        match &self.depth {
            DepthPlan::Unchanged => {}
            DepthPlan::Fast { node, depth, .. } => {
                arena.place(*node, *depth);
                if let Some(TemplateContents::Owned(contents)) = arena
                    .records
                    .get(*node)
                    .and_then(|record| record.template_contents)
                {
                    arena.place(contents, depth.saturating_add(1));
                }
            }
            plan @ DepthPlan::Moves { .. } => {
                for mov in plan.moves().filter(|mov| mov.walked()) {
                    arena.relocate_subtree(mov.root, mov.to, self.old_contents);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drive(
        storage: HtmlConstructionStorage,
        command: HtmlConstructionCommand,
    ) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
        let mut operation = Some(
            storage
                .prepare(command)
                .unwrap_or_else(|_| panic!("retained command")),
        );
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed {
            storage, created, ..
        } = HtmlConstructionOperation::try_ready(&mut operation)
            .unwrap()
            .commit()
        else {
            panic!("mutation published")
        };
        (storage, created)
    }

    fn storage() -> HtmlConstructionStorage {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        initialization
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .accept();
        HtmlConstructionInitialization::try_ready(&mut initialization)
            .unwrap()
            .commit()
    }

    fn element(local: &str) -> HtmlConstructionCommand {
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                QualifiedName::new(Namespace::html(), None, local),
                OrderedAttributes::default(),
            )),
        }
    }

    /// Whether every histogram bucket equals the records actually at its depth.
    fn histogram_is_exact(arena: &Arena) -> bool {
        (0..arena.records.len()).all(|depth| {
            arena.population(depth)
                == arena
                    .records
                    .iter()
                    .filter(|record| record.depth == depth)
                    .count()
        })
    }

    #[test]
    fn rolling_back_a_native_append_removes_its_records_from_the_histogram() {
        let (storage, _) = drive(storage(), HtmlConstructionCommand::CreateFragment);
        let (len, usage) = (storage.len(), storage.usage());
        assert!(histogram_is_exact(&storage.arena));
        // A template creates two records, at depths 0 and 1.
        let (mut storage, _) = drive(storage, element("template"));
        assert_eq!(storage.len(), len + 2);
        assert!(histogram_is_exact(&storage.arena));
        storage.rollback_native_append(len, usage);
        assert_eq!(storage.len(), len);
        assert!(histogram_is_exact(&storage.arena));
        assert_eq!(storage.arena.population(0), 1);
    }

    /// Drives `command`; a refused command is aborted and leaves the storage as it was.
    fn attempt(
        storage: HtmlConstructionStorage,
        command: HtmlConstructionCommand,
    ) -> (HtmlConstructionStorage, bool) {
        let mut operation = Some(
            storage
                .prepare(command)
                .unwrap_or_else(|_| panic!("retained command")),
        );
        let mut refused = false;
        loop {
            match operation.as_mut().unwrap().next_action() {
                Ok(Some(action)) => match action.perform() {
                    Ok(observation) => observation.accept(),
                    Err(_) => {
                        refused = true;
                        break;
                    }
                },
                Ok(None) => break,
                Err(_) => {
                    refused = true;
                    break;
                }
            }
        }
        if refused {
            let (storage, _, _) = operation.take().unwrap().abort().into_parts();
            return (storage, false);
        }
        let HtmlConstructionCommit::Changed { storage, .. } =
            HtmlConstructionOperation::try_ready(&mut operation)
                .unwrap()
                .commit()
        else {
            panic!("mutation published")
        };
        (storage, true)
    }

    /// Checks every cached depth, the usage depth and the histogram against
    /// depths recounted from each record's own parent or owner.
    fn assert_exact(arena: &Arena, context: &str) {
        let recount = |start: usize| {
            let (mut index, mut depth) = (start, 0usize);
            while let Some(parent) = arena.records.get(index).and_then(owning_parent) {
                index = parent;
                depth += 1;
                assert!(depth <= arena.records.len(), "{context}: owner cycle");
            }
            depth
        };
        let mut deepest = 0;
        for (index, record) in arena.records.iter().enumerate() {
            let expected = recount(index);
            assert_eq!(record.depth, expected, "{context}: depth of record {index}");
            deepest = deepest.max(expected);
        }
        assert_eq!(arena.usage.depth, deepest, "{context}: usage depth");
        assert!(histogram_is_exact(arena), "{context}: histogram");
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
    fn every_topology_command_keeps_depths_usage_and_histogram_exact_under_random_use() {
        use crate::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
        const KINDS: usize = 8;
        let properties = || {
            HtmlShadowRootProperties::new(
                HtmlShadowRootMode::Closed,
                HtmlSlotAssignment::Manual,
                true,
                true,
                true,
            )
        };
        let mut successes = [0usize; KINDS];
        for seed in [1, 2, 3, 4, 5, 6] {
            let mut sequence = Sequence(seed);
            let mut storage = storage();
            let create = |storage: HtmlConstructionStorage, command| {
                let (storage, created) = drive(storage, command);
                (storage, created.unwrap())
            };
            let (mut elements, mut templates, mut shadows, mut fragments) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for _ in 0..8 {
                let (next, node) = create(storage, element("e"));
                storage = next;
                elements.push(node);
            }
            for _ in 0..4 {
                let (next, node) = create(storage, element("template"));
                storage = next;
                templates.push(node);
            }
            for _ in 0..3 {
                let (next, node) = create(
                    storage,
                    HtmlConstructionCommand::CreateShadowContents {
                        properties: properties(),
                    },
                );
                storage = next;
                shadows.push(node);
                let (next, node) = create(storage, HtmlConstructionCommand::CreateFragment);
                storage = next;
                fragments.push(node);
            }
            let contents: Vec<HtmlBuilderNode> = templates
                .iter()
                .map(|template| {
                    storage
                        .node(template)
                        .unwrap()
                        .template_contents()
                        .unwrap()
                        .handle()
                })
                .collect();
            // Everything that may hold children, host a shadow root or move.
            let movable: Vec<HtmlBuilderNode> =
                elements.iter().chain(&templates).cloned().collect();
            let parents: Vec<HtmlBuilderNode> = elements
                .iter()
                .chain(&fragments)
                .chain(&shadows)
                .chain(&contents)
                .cloned()
                .collect();
            let sources: Vec<HtmlBuilderNode> =
                fragments.iter().chain(&contents).cloned().collect();
            assert_exact(&storage.arena, "initial");
            for step in 0..3000 {
                let pick = |sequence: &mut Sequence, from: &[HtmlBuilderNode]| {
                    from[sequence.next(from.len())].clone()
                };
                let kind = sequence.next(KINDS);
                let command = match kind {
                    0 | 1 => HtmlConstructionCommand::Insert {
                        node: pick(&mut sequence, &movable),
                        parent: pick(&mut sequence, &parents),
                        before: None,
                    },
                    2 => HtmlConstructionCommand::Detach {
                        node: pick(&mut sequence, &movable),
                    },
                    3 => HtmlConstructionCommand::AttachShadow {
                        host: pick(&mut sequence, &movable),
                        contents: pick(&mut sequence, &shadows),
                    },
                    4 => HtmlConstructionCommand::DetachShadow {
                        host: pick(&mut sequence, &movable),
                    },
                    5 => HtmlConstructionCommand::AttachTemplateShadow {
                        template: pick(&mut sequence, &templates),
                        host: pick(&mut sequence, &movable),
                        contents: pick(&mut sequence, &shadows),
                    },
                    6 => HtmlConstructionCommand::SetTemplateShadowContents {
                        template: pick(&mut sequence, &templates),
                        contents: pick(&mut sequence, &shadows),
                    },
                    _ => HtmlConstructionCommand::ReassignTemplateContents {
                        template: pick(&mut sequence, &templates),
                        contents: pick(&mut sequence, &sources),
                    },
                };
                let (next, accepted) = attempt(storage, command);
                storage = next;
                successes[kind] += usize::from(accepted);
                assert_exact(
                    &storage.arena,
                    &format!("seed {seed}, step {step}, kind {kind}"),
                );
            }
        }
        // Every command must have been accepted often enough to matter.
        assert!(
            successes.iter().all(|count| *count > 50),
            "accepted commands per kind: {successes:?}"
        );
    }
}
