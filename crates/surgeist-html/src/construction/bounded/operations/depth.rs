//! Exact node depths across topology changes.
//!
//! Every record caches its depth within its own tree, and the arena's usage
//! depth is the greatest cached depth. A topology change moves exactly one
//! subtree to a new depth, so only that subtree's cache changes: planning walks
//! the subtree once, one action per step, and the commit places each of its
//! records at the new depth. The other association changes recompute every
//! record, as a full depth pass always did, and leave the cache exact too.
//!
//! The usage depth after a move is the greater of the moved subtree's new
//! depth and the greatest depth of every other record. The latter is the old
//! usage depth unless the moved subtree held the deepest record and moved
//! shallower, when a scan down the arena's depth histogram, less the subtree's
//! own count at each level, finds it. The scan is bounded by the depth.
use super::*;

/// The subtree one topology change moves, and what planning learned of it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Shift {
    root: usize,
    /// The root's cached depth before the change.
    from: usize,
    /// The root's depth after the change.
    to: usize,
    /// The deepest level reached below the root so far.
    height: usize,
    /// Records reached so far, including the root.
    size: usize,
}

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
/// contents. Advances the node's cursor past every edge it reports or skips.
fn next_edge(arena: &mut Arena, node: usize) -> Result<Option<usize>, HtmlConstructionError> {
    let record = arena
        .records
        .get_mut(node)
        .ok_or(HtmlConstructionError::UnknownNode)?;
    loop {
        let cursor = record.walk_cursor;
        let children = record.children.len();
        let edge = if cursor < children {
            record.children.get(cursor).copied()
        } else if cursor == children {
            record.shadow.as_ref().map(|shadow| shadow.contents.index)
        } else if cursor == children.saturating_add(1) {
            match record.template_contents {
                Some(TemplateContents::Owned(contents)) => Some(contents),
                _ => None,
            }
        } else {
            return Ok(None);
        };
        record.walk_cursor = cursor.saturating_add(1);
        if let Some(edge) = edge {
            return Ok(Some(edge));
        }
    }
}

impl Arena {
    /// Places the subtree rooted at `root` so that the root sits at depth `to`.
    /// The walk was planned, so it cannot fail; an inconsistent arena only
    /// ends it early.
    pub(super) fn relocate_subtree(&mut self, root: usize, to: usize) {
        let budget = walk_budget(self.records.len());
        let Some(record) = self.records.get_mut(root) else {
            return;
        };
        record.walk_cursor = 0;
        self.place(root, to);
        let (mut node, mut level) = (root, 0usize);
        for _ in 0..budget {
            match next_edge(self, node) {
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
    /// Adopts the depth every record was planned at and rebuilds the histogram.
    pub(super) fn adopt_planned_depths(&mut self) {
        for record in &mut self.records {
            record.depth = record.planned_depth;
            record.population = 0;
        }
        for index in 0..self.records.len() {
            if let Some(depth) = self.records.get(index).map(|record| record.depth) {
                self.populate(depth);
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
        let (node, parent) = match self.command() {
            Command::Publish { .. } => return Ok(Phase::Layout),
            Command::Insert { .. } => (p[0], Some(p[1])),
            Command::Detach { .. } => (p[0], None),
            Command::AttachShadow { .. } => (p[1], Some(p[0])),
            // The other association changes can detach one fragment and attach
            // another, and nest one moved subtree inside the other.
            _ => {
                self.plan.rebuild = true;
                self.plan.max_depth = 0;
                return Ok(Phase::Depth {
                    node: 0,
                    cursor: 0,
                    depth: 0,
                    steps: 0,
                });
            }
        };
        let arena = self.arena();
        let record = arena
            .records
            .get(node)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        let from = record.depth;
        let to = match parent {
            Some(parent) => add(
                arena
                    .records
                    .get(parent)
                    .ok_or(HtmlConstructionError::UnknownNode)?
                    .depth,
                1,
            )?,
            None => 0,
        };
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
        if parent.is_some() && self.plan.old_parent.is_none() && shallow {
            // A fresh leaf, or a template with only its empty contents, joins
            // at one known depth and cannot lower any other record's.
            self.plan.usage.depth = self
                .plan
                .usage
                .depth
                .max(add(to, usize::from(contents.is_some()))?);
            self.plan.fast_depth = Some(to);
            return self.before_writes();
        }
        if from == to {
            // Every depth, and so the usage depth, is unchanged.
            return self.before_writes();
        }
        self.plan.shift = Some(Shift {
            root: node,
            from,
            to,
            height: 0,
            size: 1,
        });
        let arena = self.arena_mut()?;
        arena
            .records
            .get_mut(node)
            .ok_or(HtmlConstructionError::UnknownNode)?
            .walk_cursor = 0;
        arena
            .records
            .first_mut()
            .ok_or(HtmlConstructionError::UnknownNode)?
            .level_count = 1;
        Ok(Phase::Shift {
            node,
            level: 0,
            steps: 0,
        })
    }
    /// One step of the planning walk over the moved subtree: counts the records
    /// at each level below the root and its height.
    pub(super) fn shift_step(
        &mut self,
        node: usize,
        level: usize,
        steps: usize,
    ) -> Result<Phase, HtmlConstructionError> {
        let mut shift = self
            .plan
            .shift
            .ok_or(HtmlConstructionError::OperationNotReady)?;
        if steps > walk_budget(self.arena().records.len()) {
            return Err(HtmlConstructionError::Cycle);
        }
        let edge = next_edge(self.arena_mut()?, node)?;
        let Some(child) = edge else {
            if node == shift.root {
                return self.settle_depth(shift);
            }
            let up = self
                .arena()
                .records
                .get(node)
                .and_then(owning_parent)
                .ok_or(HtmlConstructionError::ParentChildConflict)?;
            return Ok(Phase::Shift {
                node: up,
                level: level
                    .checked_sub(1)
                    .ok_or(HtmlConstructionError::ParentChildConflict)?,
                steps: add(steps, 1)?,
            });
        };
        let arena = self.arena_mut()?;
        arena
            .records
            .get_mut(child)
            .ok_or(HtmlConstructionError::UnknownNode)?
            .walk_cursor = 0;
        // A walk descends one level at a time, so a deeper level is new.
        let level = add(level, 1)?;
        if level > shift.height {
            shift.height = level;
            arena
                .records
                .get_mut(level)
                .ok_or(HtmlConstructionError::UnknownNode)?
                .level_count = 0;
        }
        let bucket = arena
            .records
            .get_mut(level)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        bucket.level_count = add(bucket.level_count, 1)?;
        shift.size = add(shift.size, 1)?;
        self.plan.shift = Some(shift);
        Ok(Phase::Shift {
            node: child,
            level,
            steps: add(steps, 1)?,
        })
    }
    /// The walk is complete: settles the usage depth, or scans for it.
    fn settle_depth(&mut self, shift: Shift) -> Result<Phase, HtmlConstructionError> {
        self.plan.depth_writes = shift.size;
        let current = self.plan.usage.depth;
        let top = add(shift.from, shift.height)?;
        let new_top = add(shift.to, shift.height)?;
        if top < current || new_top >= current {
            // Some record outside the subtree is as deep as the old usage
            // depth, or the subtree itself reaches the new one.
            self.plan.usage.depth = current.max(new_top);
            return self.before_writes();
        }
        // The subtree held the deepest record and moves shallower: the depth
        // is the deepest level that holds a record outside the subtree.
        Ok(Phase::Settle { level: current })
    }
    /// One level of the scan for the deepest record outside the moved subtree.
    pub(super) fn settle_step(&mut self, level: usize) -> Result<Phase, HtmlConstructionError> {
        let shift = self
            .plan
            .shift
            .ok_or(HtmlConstructionError::OperationNotReady)?;
        let new_top = add(shift.to, shift.height)?;
        if level <= new_top {
            self.plan.usage.depth = new_top;
            return self.before_writes();
        }
        let arena = self.arena();
        let inside = level
            .checked_sub(shift.from)
            .filter(|relative| *relative <= shift.height)
            .and_then(|relative| arena.records.get(relative))
            .map_or(0, |bucket| bucket.level_count);
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
    /// The depth work the commit performs after its structural writes.
    pub(super) fn commit_depths(&self, arena: &mut Arena, attach_shadow: bool) {
        let p = self.positions;
        if let Some(depth) = self.fast_depth {
            let index = if attach_shadow { p[1] } else { p[0] };
            arena.place(index, depth);
            if let Some(TemplateContents::Owned(contents)) = arena
                .records
                .get(index)
                .and_then(|record| record.template_contents)
            {
                arena.place(contents, depth.saturating_add(1));
            }
        } else if let Some(shift) = self.shift {
            arena.relocate_subtree(shift.root, shift.to);
        } else if self.rebuild {
            arena.adopt_planned_depths();
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
}
