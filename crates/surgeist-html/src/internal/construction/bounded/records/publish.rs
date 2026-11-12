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

/// What a publication has made before it moves the records: the root it publishes, and the vectors
/// reserved for the published records and the contents made for them.
struct Prepared {
    root: NodeIx,
    mode: Option<HtmlDocumentMode>,
    records: Vec<Record<DocumentString>>,
    kinds: Vec<PublishedContent>,
}

/// What a publication has recorded with its consumer for the banks it grew, in order, so that a
/// refusal can take each record back.
#[derive(Default)]
struct BankTrail {
    entries: [Option<TrailEntry>; 4],
    len: usize,
}

/// A bank of a publication and the dimensions the consumer holds for it.
#[derive(Clone, Copy)]
struct TrailEntry {
    growth: HtmlConstructionBankGrowth,
    recorded: HtmlConstructionBankCapacity,
}

impl BankTrail {
    /// Notes a bank the consumer holds `recorded` for. A publication grows at most four banks,
    /// so the room of the trail is never short.
    fn note(&mut self, growth: HtmlConstructionBankGrowth, recorded: HtmlConstructionBankCapacity) {
        if let Some(slot) = self.entries.get_mut(self.len) {
            *slot = Some(TrailEntry { growth, recorded });
            self.len += 1;
        }
    }
}

impl HtmlConstructionStorage {
    /// Publishes the tree rooted at `root`, a detached Document or fragment record, as an
    /// immutable result, after checking that every record agrees with its parent, its children and
    /// its associations. A refusal hands the storage back with its records, their order, depths,
    /// text and usage as they were. The validation pass may leave its scratch counts written,
    /// which are not state.
    ///
    /// The result holds its text as one block and its records in a vector of their own, because a
    /// record of a storage names its text by a range of the storage's text arena and a record of
    /// the result holds a string of the block. Every allocation of the publication is reserved
    /// before the conversion, each admitted and recorded in turn: the shared arena, the contents
    /// the published records are made from, the block when the arena has more than a quarter of
    /// its units spare, which it is then packed into, and the published records. A refusal at any
    /// step takes back, in reverse order, each record the consumer accepted, so the consumer holds
    /// what it held before the publication; the consumer's own refusal of a take-back is the
    /// refusal returned. The conversion that follows allocates no reservation: it moves the
    /// records into their vector and frees the vector they were in. The handle of the shared
    /// block is made before the conversion and the handle of the shared arena in the move, for
    /// which stable Rust has no fallible constructor: an allocator failure there aborts, as any
    /// infallible allocation does. The contents are freed when the move ends, which the consumer
    /// that recorded them accounts for itself.
    #[expect(clippy::result_large_err)] // Refusal hands the complete storage back without allocating after failure.
    pub fn publish<A: HtmlRecordAdmission>(
        mut self,
        root: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<HtmlConstructionResult, HtmlConstructionPublishRefusal<A::Stop>> {
        let prepared = match self.prepare_publication(root, admission) {
            Ok(prepared) => prepared,
            Err(error) => {
                return Err(HtmlConstructionPublishRefusal {
                    storage: self,
                    error,
                });
            }
        };
        Ok(self.share(prepared))
    }

    /// Prepares the publication, and takes back what it recorded with the consumer when it
    /// refuses.
    fn prepare_publication<A: HtmlRecordAdmission>(
        &mut self,
        root: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<Prepared, HtmlConstructionRecordError<A::Stop>> {
        let mut trail = BankTrail::default();
        let error = match self.reserve_publication(root, admission, &mut trail) {
            Ok(prepared) => return Ok(prepared),
            Err(error) => error,
        };
        for entry in trail.entries.iter().rev().flatten() {
            admission
                .record_growth(HtmlConstructionGrowthRecord {
                    growth: entry.growth,
                    chosen: entry.growth.before.capacity,
                    recorded: entry.recorded,
                    reached: entry.growth.before,
                })
                .map_err(stopped)?;
        }
        Err(error)
    }

    /// Grows `growth` as [`Self::grow`] does, and notes in `trail` what the consumer holds for it.
    fn grow_recorded<A: HtmlRecordAdmission>(
        &mut self,
        trail: &mut BankTrail,
        growth: HtmlConstructionBankGrowth,
        chosen: usize,
        admission: &mut A,
        reserve: impl FnMut(&mut Self, usize) -> Reserved,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let mut recorded = growth.before;
        let outcome = self.grow_tracked(growth, chosen, admission, reserve, &mut recorded);
        if recorded != growth.before {
            trail.note(growth, recorded);
        }
        outcome
    }

    fn reserve_publication<A: HtmlRecordAdmission>(
        &mut self,
        root: &HtmlBuilderNode,
        admission: &mut A,
        trail: &mut BankTrail,
    ) -> Result<Prepared, HtmlConstructionRecordError<A::Stop>> {
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
        // Every range a text or comment record names is in the arena, before anything is reserved
        // or moved.
        let mut used = 0usize;
        for record in &self.arena.records {
            if let NodeContent::Text(run) | NodeContent::Comment(run) = record.kind {
                self.text.get(run.text()).map_err(faulted)?;
                used = plus(used, run.len()).map_err(faulted)?;
            }
        }
        let pack = self.text.wants_packing();
        let wrapper = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Wrapper,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: 1,
            element_size: size_of::<PublishedArena>(),
            wrapper_bytes: size_of::<Vec<u16>>(),
        };
        let block = pack.then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::TextBlock,
            before: HtmlConstructionBankCapacity {
                initialized_len: 0,
                capacity: self.text.capacity(),
            },
            minimum_capacity: used,
            element_size: size_of::<u16>(),
            wrapper_bytes: 0,
        });
        let published = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::PublishedRecords,
            before: HtmlConstructionBankCapacity {
                initialized_len: 0,
                capacity: self.arena.records.capacity(),
            },
            minimum_capacity: count,
            element_size: size_of::<Record<DocumentString>>(),
            wrapper_bytes: 0,
        };
        let contents = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::PublishedContents,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: count,
            element_size: size_of::<PublishedContent>(),
            wrapper_bytes: 0,
        };
        let allocations = 3 + usize::from(pack) + 2;
        Work {
            provider_calls: allocations,
            ..Work::default()
        }
        .charge(admission)?;
        // The work of the conversion: the units a pack copies, then the records and contents it
        // moves and writes.
        if pack {
            for chunk in 0..used.div_ceil(CHUNK_UNITS) {
                let units = used
                    .saturating_sub(chunk.saturating_mul(CHUNK_UNITS))
                    .min(CHUNK_UNITS);
                Work {
                    copied_bytes: units.saturating_mul(size_of::<u16>()),
                    record_writes: 1,
                    ..Work::default()
                }
                .charge(admission)?;
            }
        }
        Work {
            record_visits: count,
            copied_bytes: times(
                count,
                size_of::<Record<DocumentString>>() + size_of::<PublishedContent>(),
            )
            .map_err(faulted)?,
            record_writes: times(count, 2).map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        // Each bank is admitted and recorded in turn, the banks that add bytes before the ones
        // that replace a bank of the storage, so that the bytes each admission checks are the
        // bytes the records before it leave and the high water the consumer observes is what the
        // admissions required. `trail` notes each record, which a refusal takes back.
        let wrapper_chosen = Self::admit_bank(wrapper, admission)?;
        let wrapper_planned = HtmlConstructionBankCapacity {
            initialized_len: 0,
            capacity: wrapper_chosen,
        };
        // The shared arena is made in the move that follows, which cannot refuse.
        self.grow_recorded(trail, wrapper, wrapper_chosen, admission, |_, _| Reserved {
            ok: true,
            reached: wrapper_planned,
        })?;
        let mut kinds = Vec::new();
        let contents_chosen = Self::admit_bank(contents, admission)?;
        self.grow_recorded(
            trail,
            contents,
            contents_chosen,
            admission,
            |_, additional| {
                kinds = Vec::new();
                let ok = kinds.try_reserve_exact(additional).is_ok();
                Reserved {
                    ok,
                    reached: HtmlConstructionBankCapacity {
                        initialized_len: 0,
                        capacity: kinds.capacity(),
                    },
                }
            },
        )?;
        let mut packed = Vec::new();
        if let Some(growth) = block {
            let chosen = Self::admit_bank(growth, admission)?;
            self.grow_recorded(trail, growth, chosen, admission, |_, additional| {
                packed = Vec::new();
                let ok = packed.try_reserve_exact(additional).is_ok();
                Reserved {
                    ok,
                    reached: HtmlConstructionBankCapacity {
                        initialized_len: 0,
                        capacity: packed.capacity(),
                    },
                }
            })?;
        }
        let mut records = Vec::new();
        let published_chosen = Self::admit_bank(published, admission)?;
        self.grow_recorded(
            trail,
            published,
            published_chosen,
            admission,
            |_, additional| {
                records = Vec::new();
                let ok = records.try_reserve_exact(additional).is_ok();
                Reserved {
                    ok,
                    reached: HtmlConstructionBankCapacity {
                        initialized_len: 0,
                        capacity: records.capacity(),
                    },
                }
            },
        )?;
        // The contents are made from the block while the storage still holds its records, so that
        // a text the block cannot share is refused here, with the storage whole. Nothing after
        // this point refuses.
        let units = if pack {
            for record in &self.arena.records {
                if let NodeContent::Text(run) | NodeContent::Comment(run) = record.kind {
                    packed.extend_from_slice(self.text.get(run.text()).map_err(faulted)?);
                }
            }
            None
        } else {
            Some(self.text.take_units())
        };
        let block = TextBlock::from_utf16(units.unwrap_or(packed));
        let mut cursor = pack.then_some(0usize);
        for record in &self.arena.records {
            match record.kind.published_with(&block, &mut cursor) {
                Ok(content) => kinds.push(content),
                Err(error) => {
                    // The strings made so far are the only other holders of the block, and they
                    // are dropped before the units are taken back, so `block` is then the only
                    // holder and `into_units` returns the vector. Its refusal is a typed defect,
                    // not a state this function reaches.
                    drop(kinds);
                    if !pack {
                        let Ok(units) = block.into_units() else {
                            return Err(fault(HtmlConstructionCause::SharedAllocationAliased));
                        };
                        self.text.restore(units);
                    }
                    return Err(fault(error.cause()));
                }
            }
        }
        Ok(Prepared {
            root,
            mode,
            records,
            kinds,
        })
    }

    /// Moves the records into the vector reserved for them, and the arena into its shared wrapper.
    fn share(self, prepared: Prepared) -> HtmlConstructionResult {
        let Prepared {
            root,
            mode,
            mut records,
            kinds,
        } = prepared;
        let BuildArena {
            records: built,
            names,
            usage,
            serial,
            attribute_revision,
        } = self.arena;
        records.extend(
            built
                .into_iter()
                .zip(kinds)
                .map(|(record, kind)| record.published(kind)),
        );
        let arena = Arc::new(PublishedArena {
            records,
            names,
            usage,
            serial,
            attribute_revision,
        });
        match mode {
            Some(mode) => HtmlConstructionResult::Document(HtmlDocument { arena, root, mode }),
            None => HtmlConstructionResult::Fragment(HtmlFragment { arena, root }),
        }
    }
}
