//! Publication of the storage as an immutable result.

use super::shape::validate_record;
use super::*;

/// A publication the storage or the consumer refused, which hands the storage back unchanged.
#[derive(Debug)]
pub struct HtmlConstructionPublishRefusal<S> {
    storage: HtmlConstructionStorage,
    error: HtmlConstructionRecordError<S>,
}
impl<S> HtmlConstructionPublishRefusal<S> {
    /// Returns the unchanged storage and the refusal.
    pub fn into_parts(self) -> (HtmlConstructionStorage, HtmlConstructionRecordError<S>) {
        (self.storage, self.error)
    }
}

impl HtmlConstructionStorage {
    /// Publishes the tree rooted at `root`, a detached Document or fragment record, as an
    /// immutable result, after checking that every record agrees with its parent, its children and
    /// its associations. A
    /// refusal hands the storage back with its records, their order, depths and usage as they
    /// were. The validation pass may leave its scratch counts written, which are not state. The
    /// capacity of the shared arena is recorded in the last step that can refuse, so a refusal
    /// leaves no record of it with the consumer. The move into the shared arena allocates its
    /// handle and cannot refuse: stable Rust has no fallible constructor for it, so an allocator
    /// failure there aborts, as any infallible allocation does.
    #[expect(clippy::result_large_err)] // Refusal hands the complete storage back without allocating after failure.
    pub fn publish<A: HtmlRecordAdmission>(
        mut self,
        root: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<HtmlConstructionResult, HtmlConstructionPublishRefusal<A::Stop>> {
        match self.prepare_publication(root, admission) {
            Ok((root, mode)) => Ok(self.share(root, mode)),
            Err(error) => Err(HtmlConstructionPublishRefusal {
                storage: self,
                error,
            }),
        }
    }

    fn prepare_publication<A: HtmlRecordAdmission>(
        &mut self,
        root: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<(NodeIx, Option<HtmlDocumentMode>), HtmlConstructionRecordError<A::Stop>> {
        let root = self.operand(root, admission)?;
        let record = self.record(root).map_err(faulted)?;
        let mode = match record.kind {
            NodeContent::Document(mode) => Some(mode),
            NodeContent::Fragment => None,
            _ => return Err(fault(HtmlConstructionCause::InvalidOutputRoot)),
        };
        if record.parent.is_some() {
            return Err(fault(HtmlConstructionCause::InvalidOutputRoot));
        }
        let count = self.arena.records.len();
        // Every record's membership count is reset, then each record's entries in its parent's
        // child list are counted once, then every record is validated in index order and its
        // count read in constant time.
        for position in 0..count {
            Work {
                record_visits: 1,
                record_writes: 1,
                ..Work::default()
            }
            .charge(admission)?;
            if let Some(record) = self.arena.records.get_mut(position) {
                record.publish_found = 0;
            }
        }
        for position in 0..count {
            let parent = NodeIx::new(position)
                .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
            let children = self.record(parent).map_err(faulted)?.children.len();
            Work {
                record_visits: plus(1, children).map_err(faulted)?,
                record_writes: children,
                ..Work::default()
            }
            .charge(admission)?;
            for offset in 0..children {
                let Some(child) = self
                    .record(parent)
                    .map_err(faulted)?
                    .children
                    .get(offset)
                    .copied()
                else {
                    break;
                };
                // Only an entry naming this list's owner as its parent counts; the validation
                // refuses every other entry when it reaches it.
                if let Some(target) = self.arena.records.get_mut(child.position())
                    && target.parent == Some(parent)
                {
                    target.publish_found = target.publish_found.saturating_add(1).min(2);
                }
            }
        }
        for position in 0..count {
            let index = NodeIx::new(position)
                .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
            let children = self.record(index).map_err(faulted)?.children.len();
            Work {
                record_visits: plus(3, children).map_err(faulted)?,
                ..Work::default()
            }
            .charge(admission)?;
            validate_record(&self.arena, index).map_err(faulted)?;
            let record = self.record(index).map_err(faulted)?;
            for &child in &record.children {
                let target = self.record(child).map_err(faulted)?;
                if target.parent != Some(index)
                    || target.owner.is_some()
                    || matches!(
                        target.kind,
                        NodeContent::Document(_) | NodeContent::Fragment
                    )
                {
                    return Err(fault(HtmlConstructionCause::ParentChildConflict));
                }
            }
            // Every entry of every parent's list was counted before this pass: the record must
            // appear in its parent's list exactly once.
            if record.parent.is_some() && record.publish_found != 1 {
                return Err(fault(HtmlConstructionCause::ParentChildConflict));
            }
        }
        let growth = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Wrapper,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: 1,
            element_size: size_of::<Arena>(),
            wrapper_bytes: 0,
        };
        Work {
            provider_calls: 3,
            ..Work::default()
        }
        .charge(admission)?;
        let chosen = Self::admit_bank(growth, admission)?;
        let planned = HtmlConstructionBankCapacity {
            initialized_len: 0,
            capacity: chosen,
        };
        // The shared arena is made in the move below, which cannot refuse.
        self.grow(growth, chosen, admission, |_, _| Reserved {
            ok: true,
            reached: planned,
        })?;
        Ok((root, mode))
    }

    /// Moves the arena into its shared wrapper.
    fn share(self, root: NodeIx, mode: Option<HtmlDocumentMode>) -> HtmlConstructionResult {
        let arena = Arc::new(self.arena);
        match mode {
            Some(mode) => HtmlConstructionResult::Document(HtmlDocument { arena, root, mode }),
            None => HtmlConstructionResult::Fragment(HtmlFragment { arena, root }),
        }
    }
}
