//! The topmost ancestor of a record, cached in its node-join cell.
//!
//! Several per-element questions are asked of the tree a record sits in, not
//! of its ancestors one by one: whether that tree is a declarative shadow
//! root, whether it is the document's. Climbing the parent edges answers them
//! at a cost that grows with the depth of every element created. Each record
//! instead keeps the root it last found, stamped with the generation of the
//! cache, and a climb stops at the first record whose stamp is current.
//!
//! A cached root is exact while no topology change has altered the root of a
//! record that holds it. Exactly two commands change a parent edge, `Insert`
//! and `Detach`, and both are announced to [`SessionStorage::begin_reroot`]
//! before they are committed and to [`SessionStorage::end_reroot`] once the
//! construction owner has finished them:
//!
//! * A record with no ordinary children is the only record whose root the
//!   change alters, so its own cell is cleared and nothing else is touched.
//! * Otherwise the records below it change root unless the move stays inside
//!   one tree, which both roots prove; any other change starts a new
//!   generation, which makes every cell stale at once.
//!
//! Who owns a root is not cached: the owner of the fragment at the root is
//! read when the answer is needed, so attaching or releasing template or
//! shadow contents needs no announcement. A generation counter that can no
//! longer advance disables the cache instead of wrapping.
use super::*;

/// The root a record last found and the generation it found it in.
#[derive(Clone, Copy, Default)]
pub(super) enum TreeRootCell {
    #[default]
    Empty,
    Cached {
        /// The root's record position plus one.
        root: NonZeroU32,
        stamp: NonZeroU32,
    },
}

/// What a topology change does to the cached roots, decided before it is
/// committed and finished after.
#[derive(Clone, Copy)]
#[must_use = "a change announced with begin_reroot must be finished with end_reroot"]
pub(super) enum Reroot {
    /// The moved record has no ordinary children: only its own cell changes.
    Bare(usize),
    /// The records below the moved record keep their root.
    Same,
    /// The records below the moved record may change root.
    Cross,
}

impl PayloadStorage {
    /// The root cached for `node`, when its cell is current.
    fn cached_root(&self, node: usize) -> Option<usize> {
        let stamp = self.tree_stamp?;
        match self.node_joins.get(node)?.tree_root {
            TreeRootCell::Cached { root, stamp: held } if held == stamp => cell_slot(root),
            _ => None,
        }
    }
    /// Climbs from `node` to its root or to the first record with a current
    /// cached root that is still a root, returning that root and the record
    /// where the climb stopped. Nothing is written.
    fn climb_to_root(
        &self,
        control: &mut ParseControl,
        storage: &surgeist_html::bounded::construction::HtmlConstructionStorage,
        node: usize,
    ) -> Result<(usize, usize), SessionError> {
        let mut at = node;
        for _ in 0..storage.len() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(root) = self.cached_root(at) {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if storage
                    .node_at(root)
                    .ok_or(SessionError::ForeignNode)?
                    .parent()
                    .is_none()
                {
                    #[cfg(test)]
                    assert_eq!(
                        Some(root),
                        Self::full_climb_root(storage, node),
                        "a cached tree root of record {at} differs from a full climb from {node}"
                    );
                    return Ok((root, at));
                }
            }
            match storage
                .node_at(at)
                .ok_or(SessionError::ForeignNode)?
                .parent()
            {
                Some(parent) => at = parent.position(),
                None => return Ok((at, at)),
            }
        }
        Err(SessionError::Cycle)
    }
    /// The root found by following parent edges all the way, uncharged: the
    /// oracle every cached answer is compared with in test builds.
    #[cfg(test)]
    fn full_climb_root(
        storage: &surgeist_html::bounded::construction::HtmlConstructionStorage,
        node: usize,
    ) -> Option<usize> {
        let mut at = node;
        for _ in 0..=storage.len() {
            match storage.node_at(at)?.parent() {
                Some(parent) => at = parent.position(),
                None => return Some(at),
            }
        }
        None
    }
    /// The root of `node`'s tree along parent edges, read without caching what
    /// the climb learns.
    pub(super) fn tree_root_uncached(
        &self,
        control: &mut ParseControl,
        storage: &surgeist_html::bounded::construction::HtmlConstructionStorage,
        node: usize,
    ) -> Result<usize, SessionError> {
        Ok(self.climb_to_root(control, storage, node)?.0)
    }
    /// The root of `node`'s tree along parent edges. The records the climb
    /// passes keep the root it found.
    pub(super) fn tree_root(
        &mut self,
        control: &mut ParseControl,
        storage: &surgeist_html::bounded::construction::HtmlConstructionStorage,
        node: usize,
    ) -> Result<usize, SessionError> {
        let (root, stop) = self.climb_to_root(control, storage, node)?;
        let (Some(stamp), Some(cell_root)) = (self.tree_stamp, joined_cell(root).ok()) else {
            return Ok(root);
        };
        let mut at = node;
        for _ in 0..storage.len() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
            if let Some(cell) = self.node_joins.get_mut(at) {
                cell.tree_root = TreeRootCell::Cached {
                    root: cell_root,
                    stamp,
                };
            }
            if at == stop {
                return Ok(root);
            }
            match storage
                .node_at(at)
                .ok_or(SessionError::ForeignNode)?
                .parent()
            {
                Some(parent) => at = parent.position(),
                None => return Ok(root),
            }
        }
        Err(SessionError::Cycle)
    }
    /// Counts one more announced topology change. A count that cannot advance
    /// is dropped, as a generation is, rather than wrapped onto an earlier
    /// value: with no count nothing is taken for the same change twice.
    fn count_topology_change(&mut self) {
        self.topology_changes = self
            .topology_changes
            .and_then(|changes| changes.checked_add(1));
    }
    /// Makes the cell of `node` stale. The caller has charged the write.
    fn forget_tree_root(&mut self, node: usize) {
        if let Some(cell) = self.node_joins.get_mut(node) {
            cell.tree_root = TreeRootCell::Empty;
        }
    }
    /// Makes every cell stale, or disables the cache when no later generation
    /// can be named. The caller has charged the write.
    fn forget_every_tree_root(&mut self) {
        self.tree_stamp = self.tree_stamp.and_then(|stamp| stamp.checked_add(1));
    }
}

impl SessionStorage {
    /// Announces that `node` is about to move below `destination`, or be
    /// detached when there is none. Call before the change is committed.
    pub(super) fn begin_reroot(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        destination: Option<usize>,
    ) -> Result<Reroot, SessionError> {
        self.payloads.count_topology_change();
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let bare = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .first_child()
            .is_none();
        if bare {
            control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
            self.payloads.forget_tree_root(node);
            return Ok(Reroot::Bare(node));
        }
        if let Some(destination) = destination {
            let storage = &self.construction;
            let below = self.payloads.tree_root(control, storage, destination)?;
            let moved = self.payloads.tree_root(control, storage, node)?;
            if below == moved {
                return Ok(Reroot::Same);
            }
        }
        // Charged here so that finishing the change cannot be refused.
        control.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
        self.payloads.forget_every_tree_root();
        Ok(Reroot::Cross)
    }
    /// Finishes [`Self::begin_reroot`] once the change is committed. It
    /// repeats the invalidation, so a cell filled between the two cannot
    /// outlive the topology it described.
    pub(super) fn end_reroot(&mut self, reroot: Reroot) {
        self.payloads.count_topology_change();
        match reroot {
            Reroot::Bare(node) => self.payloads.forget_tree_root(node),
            Reroot::Same => {}
            Reroot::Cross => self.payloads.forget_every_tree_root(),
        }
    }
}

#[cfg(test)]
#[path = "tree_root_tests.rs"]
mod tests;
