//! The nodes one capture plans, their subjects and the order they are expanded in.
//!
//! Every lookup goes through a `NodeId` index, so planning, subject lookup and
//! attachment each cost a constant number of charged steps per node. The index
//! and each backing vector are admitted against the adapter's subject limit and
//! charged before they allocate.

use std::collections::HashMap;

use surgeist_html_semantics::HtmlCaptureBuilderSubject;

use super::{Control, DomHtmlCaptureError, DomHtmlCaptureResource};
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

/// Planned entries in first-planned order, expanded lowest position first.
pub(super) struct Planned {
    entries: Vec<PlannedEntry>,
    /// The position of each planned node.
    index: HashMap<NodeId, usize>,
    /// Every entry at or beyond this position has not been expanded.
    cursor: usize,
    /// Positions below `cursor` to expand again, as a binary min-heap. An
    /// entry is here exactly when it was expanded before its children became
    /// complete; later positions only ever follow every one of them.
    revisit: Vec<usize>,
}

impl Planned {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::new(),
            cursor: 0,
            revisit: Vec::new(),
        }
    }

    /// The node planned at `position`, which must be below `len`.
    pub(super) fn node(&self, position: usize) -> NodeId {
        self.entries[position].node
    }

    /// Whether the entry at `position`, which must be below `len`, completes its children.
    pub(super) fn complete_children(&self, position: usize) -> bool {
        self.entries[position].complete_children
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
        control.charge(1)?;
        Ok(self.index.get(&node).copied())
    }

    /// Plans `node`, or completes the children of an entry already planned.
    pub(super) fn include(
        &mut self,
        control: &mut Control<'_>,
        node: NodeId,
        complete_children: bool,
    ) -> Result<(), DomHtmlCaptureError> {
        if let Some(position) = self.position(control, node)? {
            let entry = &mut self.entries[position];
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
        let count = self
            .entries
            .len()
            .checked_add(1)
            .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
        reserve_index(control, &mut self.index, count)?;
        self.index.insert(node, self.entries.len());
        self.entries.push(PlannedEntry {
            node,
            complete_children,
            expanded: false,
            subject: None,
        });
        Ok(())
    }

    /// The next entry to expand, lowest position first, with its children
    /// completeness as of now.
    pub(super) fn next(
        &mut self,
        control: &mut Control<'_>,
    ) -> Result<Option<(NodeId, bool)>, DomHtmlCaptureError> {
        control.charge(1)?;
        let position = if self.revisit.is_empty() {
            if self.cursor == self.entries.len() {
                return Ok(None);
            }
            self.cursor += 1;
            self.cursor - 1
        } else {
            self.pop_revisit(control)?
        };
        let entry = &mut self.entries[position];
        entry.expanded = true;
        Ok(Some((entry.node, entry.complete_children)))
    }

    /// Records the builder subject of the entry at `position`.
    pub(super) fn stage(&mut self, position: usize, subject: HtmlCaptureBuilderSubject) {
        self.entries[position].subject = Some(subject);
    }

    /// The builder subject staged for the entry at `position`.
    pub(super) fn staged(
        &self,
        control: &mut Control<'_>,
        position: usize,
    ) -> Result<HtmlCaptureBuilderSubject, DomHtmlCaptureError> {
        control.charge(1)?;
        self.entries
            .get(position)
            .and_then(|entry| entry.subject.clone())
            .ok_or(DomHtmlCaptureError::UnmappedSubject)
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
        control.reserve(&mut self.revisit, 1)?;
        let mut at = self.revisit.len();
        self.revisit.push(position);
        while at > 0 {
            control.charge(1)?;
            let above = (at - 1) / 2;
            if self.revisit[above] <= self.revisit[at] {
                break;
            }
            self.revisit.swap(above, at);
            at = above;
        }
        Ok(())
    }

    fn pop_revisit(&mut self, control: &mut Control<'_>) -> Result<usize, DomHtmlCaptureError> {
        let last = self.revisit.len() - 1;
        self.revisit.swap(0, last);
        let position = self.revisit.pop().expect("a revisit position was present");
        let mut at = 0;
        loop {
            let left = 2 * at + 1;
            if left >= self.revisit.len() {
                break;
            }
            control.charge(1)?;
            let lower =
                if left + 1 < self.revisit.len() && self.revisit[left + 1] < self.revisit[left] {
                    left + 1
                } else {
                    left
                };
            if self.revisit[at] <= self.revisit[lower] {
                break;
            }
            self.revisit.swap(at, lower);
            at = lower;
        }
        Ok(position)
    }
}

/// Admits and allocates room for one more indexed node: `count` entries must
/// already be within the subject limit, which the entry vector's own
/// reservation has checked.
fn reserve_index(
    control: &mut Control<'_>,
    index: &mut HashMap<NodeId, usize>,
    count: usize,
) -> Result<(), DomHtmlCaptureError> {
    control.charge(1)?;
    if count > control.limits.subjects {
        return Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: control.limits.subjects,
        });
    }
    count
        .checked_mul(std::mem::size_of::<(NodeId, usize)>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(DomHtmlCaptureError::RepresentationExhausted)?;
    // The map doubles its backing when it is full, so insertions cost a
    // constant amortized number of moves.
    index
        .try_reserve(1)
        .map_err(|_| DomHtmlCaptureError::AllocationFailed)
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
