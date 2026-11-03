//! The nodes one capture plans, their subjects and the order they are expanded in.
//!
//! Planned-node membership, subject and parent lookups go through a `NodeId`
//! index and each cost a constant number of charged steps. Choosing the next
//! entry to expand costs a constant number of steps, except that revisiting an
//! entry whose children became complete after it was expanded costs a number
//! of steps logarithmic in the entries waiting to be revisited. Every backing
//! vector and the index are admitted against the adapter's subject limit and
//! charged before they allocate; a growth charges the entries it moves.

use std::{cmp::Reverse, collections::BinaryHeap};

use surgeist_html_semantics::HtmlCaptureBuilderSubject;

use super::{
    Control, DomHtmlCaptureError, DomHtmlCaptureResource,
    nodes::{Keyed, NodeTable},
};
use crate::NodeId;

struct PlannedEntry {
    node: NodeId,
    complete_children: bool,
    /// False from planning until the entry is next taken for expansion, and
    /// again after its children become complete.
    expanded: bool,
    /// The builder subject, once staged.
    subject: Option<HtmlCaptureBuilderSubject>,
}
impl Keyed for PlannedEntry {
    fn node_id(&self) -> NodeId {
        self.node
    }
}

/// Planned entries in first-planned order, expanded lowest position first.
pub(super) struct Planned {
    entries: Vec<PlannedEntry>,
    /// The position of each planned node.
    index: NodeTable,
    /// Every entry at or beyond this position has not been expanded.
    cursor: usize,
    /// Positions below `cursor` to expand again, lowest first. An entry is
    /// here exactly when it was expanded before its children became complete;
    /// later positions only ever follow every one of them.
    revisit: BinaryHeap<Reverse<usize>>,
}

/// The steps a heap operation over `len` entries takes at most.
fn heap_steps(len: usize) -> usize {
    (usize::BITS - len.leading_zeros()) as usize + 1
}

impl Planned {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: NodeTable::default(),
            cursor: 0,
            revisit: BinaryHeap::new(),
        }
    }

    /// The node planned at `position`.
    pub(super) fn node(&self, position: usize) -> Result<NodeId, DomHtmlCaptureError> {
        self.entries
            .get(position)
            .map(|entry| entry.node)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
    }

    /// Whether the entry at `position` completes its children.
    pub(super) fn complete_children(&self, position: usize) -> Result<bool, DomHtmlCaptureError> {
        self.entries
            .get(position)
            .map(|entry| entry.complete_children)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
    }

    pub(super) fn positions(&self) -> std::ops::Range<usize> {
        0..self.entries.len()
    }

    /// The position of `node`, if planned.
    pub(super) fn position(
        &self,
        control: &mut Control<'_>,
        node: NodeId,
    ) -> Result<Option<usize>, DomHtmlCaptureError> {
        self.index.get(control, &self.entries, node)
    }

    /// Plans `node`, or completes the children of an entry already planned.
    pub(super) fn include(
        &mut self,
        control: &mut Control<'_>,
        node: NodeId,
        complete_children: bool,
    ) -> Result<(), DomHtmlCaptureError> {
        if let Some(position) = self.position(control, node)? {
            let entry = self
                .entries
                .get_mut(position)
                .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
            if complete_children && !entry.complete_children {
                entry.complete_children = true;
                if entry.expanded {
                    entry.expanded = false;
                    self.revisit_later(control, position)?;
                }
            }
            return Ok(());
        }
        control.reserve(&mut self.entries, 1)?;
        let position = self.entries.len();
        self.entries.push(PlannedEntry {
            node,
            complete_children,
            expanded: false,
            subject: None,
        });
        self.index.insert(control, &self.entries, position)
    }

    /// The next entry to expand, lowest position first, with its children
    /// completeness as of now.
    pub(super) fn next(
        &mut self,
        control: &mut Control<'_>,
    ) -> Result<Option<(NodeId, bool)>, DomHtmlCaptureError> {
        control.charge(1)?;
        let position = match self.revisit.pop() {
            Some(Reverse(position)) => {
                control.charge(heap_steps(self.revisit.len()))?;
                position
            }
            None if self.cursor < self.entries.len() => {
                self.cursor += 1;
                self.cursor - 1
            }
            None => return Ok(None),
        };
        let entry = self
            .entries
            .get_mut(position)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
        entry.expanded = true;
        Ok(Some((entry.node, entry.complete_children)))
    }

    /// Records the builder subject of the entry at `position`.
    pub(super) fn stage(
        &mut self,
        position: usize,
        subject: HtmlCaptureBuilderSubject,
    ) -> Result<(), DomHtmlCaptureError> {
        self.entries
            .get_mut(position)
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?
            .subject = Some(subject);
        Ok(())
    }

    /// The builder subject staged for the entry at `position`.
    pub(super) fn staged(
        &self,
        control: &mut Control<'_>,
        position: usize,
    ) -> Result<HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
        control.charge(1)?;
        self.staged_ref(position).cloned()
    }

    /// The builder subject staged for the entry at `position`, uncharged: the
    /// caller charges its own step.
    pub(super) fn staged_ref(
        &self,
        position: usize,
    ) -> Result<&HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
        self.entries
            .get(position)
            .and_then(|entry| entry.subject.as_ref())
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
    }

    /// The builder subject staged for `node`.
    pub(super) fn subject(
        &self,
        control: &mut Control<'_>,
        node: NodeId,
    ) -> Result<HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
        let position = self
            .position(control, node)?
            .ok_or(DomHtmlCaptureError::UnmappedSubject)?;
        self.staged(control, position)
    }

    fn revisit_later(
        &mut self,
        control: &mut Control<'_>,
        position: usize,
    ) -> Result<(), DomHtmlCaptureError> {
        control.charge(1)?;
        let count = self
            .revisit
            .len()
            .checked_add(1)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if count > control.limits.subjects {
            return Err(DomHtmlCaptureError::ResourceLimit {
                resource: DomHtmlCaptureResource::Subjects,
                limit: control.limits.subjects,
            });
        }
        count
            .checked_mul(std::mem::size_of::<Reverse<usize>>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        if count > self.revisit.capacity() {
            // The heap moves its entries when it grows.
            control.charge(self.revisit.len())?;
            self.revisit
                .try_reserve(1)
                .map_err(|_| DomHtmlCaptureError::AllocationFailed)?;
        }
        control.charge(heap_steps(count))?;
        self.revisit.push(Reverse(position));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::{Dom, DomBatch, DomHtmlCaptureLimits, DomLimits, DomNameLimits, Namespace};

    /// `count` distinct node identities from a real store.
    fn nodes(count: usize) -> Vec<NodeId> {
        let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
        let limits = DomLimits::new(count + 8, 4096, 32, 0, 0, 1, 65536, names).unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        for _ in 0..count {
            batch.create_element_ns(&document, Namespace::html(), "div");
        }
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let ids: Vec<NodeId> = dom.snapshot().nodes().take(count).collect();
        assert_eq!(ids.len(), count);
        ids
    }

    /// The expansion order the capture defined before it had an index: every
    /// step rescans the entries from the first for the lowest not yet expanded.
    #[derive(Default)]
    struct Rescan(Vec<(NodeId, bool, bool)>);
    impl Rescan {
        fn include(&mut self, node: NodeId, complete: bool) {
            match self.0.iter_mut().find(|entry| entry.0 == node) {
                Some(entry) if complete && !entry.1 => {
                    entry.1 = true;
                    entry.2 = false;
                }
                Some(_) => {}
                None => self.0.push((node, complete, false)),
            }
        }
        fn next(&mut self) -> Option<(NodeId, bool)> {
            let entry = self.0.iter_mut().find(|entry| !entry.2)?;
            entry.2 = true;
            Some((entry.0, entry.1))
        }
    }

    #[test]
    fn expansion_follows_the_lowest_unexpanded_entry_through_completions() {
        let ids = nodes(12);
        let cancellation = AtomicBool::new(false);
        let mut control = Control {
            limits: DomHtmlCaptureLimits::new(64, usize::MAX),
            work: 0,
            cancellation: &cancellation,
        };
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut random = move |bound: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % bound as u64) as usize
        };
        let (mut revisited, mut expanded) = (0, 0);
        for _ in 0..400 {
            let mut planned = Planned::new();
            let mut model = Rescan::default();
            for _ in 0..60 {
                if random(3) == 0 {
                    let next = planned.next(&mut control).unwrap();
                    assert_eq!(next, model.next());
                    expanded += usize::from(next.is_some());
                } else {
                    let node = ids[random(ids.len())];
                    let complete = random(2) == 0;
                    let before = planned.revisit.len();
                    planned.include(&mut control, node, complete).unwrap();
                    model.include(node, complete);
                    revisited += usize::from(planned.revisit.len() > before);
                }
            }
            while let Some(expect) = model.next() {
                assert_eq!(planned.next(&mut control).unwrap(), Some(expect));
            }
            assert_eq!(planned.next(&mut control).unwrap(), None);
        }
        assert!(
            revisited > 100 && expanded > 1000,
            "completions after expansion must be exercised: {revisited} {expanded}"
        );
    }
}
