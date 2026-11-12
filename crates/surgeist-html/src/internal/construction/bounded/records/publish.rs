//! Publication of the storage as an immutable result.

use super::shape::validate_record;
use super::*;
use crate::internal::construction::attribute_table::{PublishedAttr, RowSlice};

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
    entries: [Option<TrailEntry>; 5],
    len: usize,
}

/// A bank of a publication and the dimensions the consumer holds for it.
#[derive(Clone, Copy)]
struct TrailEntry {
    growth: HtmlConstructionBankGrowth,
    recorded: HtmlConstructionBankCapacity,
}

impl BankTrail {
    /// Notes a bank the consumer holds `recorded` for. A publication grows at most five banks,
    /// so the room of the trail is never short; a full trail is the typed capacity-mismatch
    /// defect, which the publication takes back like any other refusal.
    fn note(
        &mut self,
        growth: HtmlConstructionBankGrowth,
        recorded: HtmlConstructionBankCapacity,
    ) -> Result<(), HtmlConstructionError> {
        let slot = self
            .entries
            .get_mut(self.len)
            .ok_or(HtmlConstructionCause::CapacityMismatch.error())?;
        *slot = Some(TrailEntry { growth, recorded });
        self.len += 1;
        Ok(())
    }
}

impl HtmlConstructionStorage {
    /// Publishes the tree rooted at `root`, a detached Document or fragment record, as an
    /// immutable result, after checking that every record agrees with its parent, its children and
    /// its associations. A refusal hands the storage back with its records, their order, depths,
    /// text and usage as they were. The validation pass may leave its scratch counts written,
    /// which are not state.
    ///
    /// The result holds its text as one block, its attributes in one table and its records in a
    /// vector of their own, because a record of a storage names its text by a range of the
    /// storage's text arena and a record of the result holds a string of the block. Every
    /// allocation of the publication is reserved before the conversion, each admitted and recorded
    /// in turn: the shared arena, the contents the published records are made from, the
    /// published attributes (one row for each live row of every run), the block when the arena has
    /// more than a quarter of its units spare, which it is then packed into, and the published
    /// records. A refusal at any step takes back, in reverse order, each record the consumer
    /// accepted, so the consumer holds what it held before the publication; the consumer's own
    /// refusal of a take-back is the refusal returned. The conversion that follows allocates no
    /// reservation: it moves the records into their vector and frees the vector they were in. The
    /// handle of the shared block is made before the conversion, the handle of the table of
    /// published attributes in its first pass and the handle of the shared arena in the move, for
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
            trail.note(growth, recorded).map_err(faulted)?;
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
        // Every range a text or comment record names, and every run and value an element names,
        // is in the storage, before anything is reserved or moved.
        let mut used = 0usize;
        let mut rows_total = 0usize;
        for record in &self.arena.records {
            match &record.kind {
                NodeContent::Text(run) | NodeContent::Comment(run) => {
                    self.text.get(run.text()).map_err(faulted)?;
                    used = plus(used, run.len()).map_err(faulted)?;
                }
                NodeContent::Element(element) => {
                    let rows = self.attrs.live(element.attrs).map_err(faulted)?;
                    for row in rows {
                        self.text.get(row.value()).map_err(faulted)?;
                        used = plus(used, row.value().len()).map_err(faulted)?;
                    }
                    rows_total = plus(rows_total, rows.len()).map_err(faulted)?;
                }
                _ => {}
            }
        }
        let pack = self.text.wants_packing();
        let has_rows = rows_total != 0;
        let wrapper = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Wrapper,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: 1,
            element_size: size_of::<PublishedArena>(),
            // The handle of the shared block, and the handle of the shared attribute table when
            // an element has attributes.
            wrapper_bytes: times(size_of::<Vec<u16>>(), 1 + usize::from(has_rows))
                .map_err(faulted)?,
        };
        let attributes = has_rows.then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::PublishedAttributes,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: rows_total,
            element_size: size_of::<PublishedAttr>(),
            wrapper_bytes: 0,
        });
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
        let allocations = 3 + usize::from(pack) + 2 + 2 * usize::from(has_rows);
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
            copied_bytes: plus(
                times(
                    count,
                    size_of::<Record<DocumentString>>() + size_of::<PublishedContent>(),
                )
                .map_err(faulted)?,
                times(rows_total, size_of::<PublishedAttr>()).map_err(faulted)?,
            )
            .map_err(faulted)?,
            record_writes: plus(times(count, 2).map_err(faulted)?, rows_total).map_err(faulted)?,
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
        let mut published_rows = Vec::new();
        if let Some(growth) = attributes {
            let chosen = Self::admit_bank(growth, admission)?;
            self.grow_recorded(trail, growth, chosen, admission, |_, additional| {
                published_rows = Vec::new();
                let ok = published_rows.try_reserve_exact(additional).is_ok();
                Reserved {
                    ok,
                    reached: HtmlConstructionBankCapacity {
                        initialized_len: 0,
                        capacity: published_rows.capacity(),
                    },
                }
            })?;
        }
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
        // a text or value the block cannot share is refused here, with the storage whole. Nothing
        // after this point refuses.
        let units = if pack {
            for record in &self.arena.records {
                match &record.kind {
                    NodeContent::Text(run) | NodeContent::Comment(run) => {
                        packed.extend_from_slice(self.text.get(run.text()).map_err(faulted)?);
                    }
                    NodeContent::Element(element) => {
                        for row in self.attrs.live(element.attrs).map_err(faulted)? {
                            packed.extend_from_slice(self.text.get(row.value()).map_err(faulted)?);
                        }
                    }
                    _ => {}
                }
            }
            None
        } else {
            Some(self.text.take_units())
        };
        let block = TextBlock::from_utf16(units.unwrap_or(packed));
        // The first pass makes the rows of every element in record order, and the second makes
        // the contents, an element getting the slice of the table that holds its rows. Each
        // reads the units of the packed block in the same order, the units of a text or of the
        // values of an element at a time.
        let made = self.publish_rows(&block, pack, has_rows.then_some(published_rows));
        let rows = match made {
            Ok(rows) => rows,
            Err(error) => {
                // The strings made so far are the only other holders of the block, and they are
                // dropped before the units are taken back, so `block` is then the only holder
                // and `into_units` returns the vector. Its refusal is a typed defect, not a
                // state this function reaches.
                return Err(self.restore_block(block, pack, error));
            }
        };
        let made = self.publish_contents(&block, pack, rows.as_ref(), &mut kinds);
        if let Err(error) = made {
            drop(kinds);
            drop(rows);
            return Err(self.restore_block(block, pack, error));
        }
        Ok(Prepared {
            root,
            mode,
            records,
            kinds,
        })
    }

    /// Hands the units of an unpacked block back to the text arena after a refused conversion,
    /// and returns the refusal. Every string of the block was dropped by the caller.
    fn restore_block<S>(
        &mut self,
        block: TextBlock,
        pack: bool,
        error: HtmlConstructionError,
    ) -> HtmlConstructionRecordError<S> {
        if !pack {
            let Ok(units) = block.into_units() else {
                return fault(HtmlConstructionCause::SharedAllocationAliased);
            };
            self.text.restore(units);
        }
        fault(error.cause())
    }

    /// The first pass of the conversion: the published rows of every element, in record order,
    /// made in `rows`, the vector reserved for them, and shared as one table. `None` when the
    /// storage has no live row.
    fn publish_rows(
        &self,
        block: &TextBlock,
        pack: bool,
        rows: Option<Vec<PublishedAttr>>,
    ) -> Result<Option<Arc<Vec<PublishedAttr>>>, HtmlConstructionError> {
        let Some(mut rows) = rows else {
            return Ok(None);
        };
        let mut cursor = pack.then_some(0usize);
        for record in &self.arena.records {
            match &record.kind {
                NodeContent::Element(element) => {
                    for row in self.attrs.live(element.attrs)? {
                        let value = block_string(block, &mut cursor, row.value())?;
                        rows.push(PublishedAttr::new(row.entry().clone(), value));
                    }
                }
                NodeContent::Text(run) | NodeContent::Comment(run) => {
                    skip_units(&mut cursor, run.len())?;
                }
                _ => {}
            }
        }
        Ok(Some(Arc::new(rows)))
    }

    /// The second pass of the conversion: the published content of every record, in record
    /// order, pushed to `kinds`. The first element's slice starts at the first row; each later
    /// slice starts after the rows of the elements before it.
    fn publish_contents(
        &self,
        block: &TextBlock,
        pack: bool,
        rows: Option<&Arc<Vec<PublishedAttr>>>,
        kinds: &mut Vec<PublishedContent>,
    ) -> Result<(), HtmlConstructionError> {
        let mut cursor = pack.then_some(0usize);
        let mut next_row = 0usize;
        for record in &self.arena.records {
            let content = match &record.kind {
                NodeContent::Document(mode) => NodeContent::Document(*mode),
                NodeContent::Fragment => NodeContent::Fragment,
                NodeContent::Element(element) => {
                    let live = self.attrs.live(element.attrs)?;
                    for row in live {
                        skip_units(&mut cursor, row.value().len())?;
                    }
                    let attrs = match rows {
                        Some(rows) if !live.is_empty() => {
                            Some(RowSlice::new(rows, next_row, live.len())?)
                        }
                        _ => None,
                    };
                    next_row = next_row
                        .checked_add(live.len())
                        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
                    NodeContent::Element(ElementContent {
                        link_flags: element.link_flags,
                        name: element.name.clone(),
                        attrs,
                    })
                }
                NodeContent::Text(run) => {
                    NodeContent::Text(block_string(block, &mut cursor, run.text())?)
                }
                NodeContent::Comment(run) => {
                    NodeContent::Comment(block_string(block, &mut cursor, run.text())?)
                }
                NodeContent::ProcessingInstruction(value) => {
                    NodeContent::ProcessingInstruction(value.clone())
                }
                NodeContent::Doctype(value) => NodeContent::Doctype(value.clone()),
            };
            kinds.push(content);
        }
        Ok(())
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

/// The string of `block` for the units of `text`: the range `text` names, or the next range of a
/// packed block, whose units follow in record order, when `cursor` holds its position.
fn block_string(
    block: &TextBlock,
    cursor: &mut Option<usize>,
    text: crate::internal::construction::text::TextRef,
) -> Result<DocumentString, HtmlConstructionError> {
    let range = match cursor {
        Some(next) => {
            let start = *next;
            let end = start
                .checked_add(text.len())
                .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
            *next = end;
            start..end
        }
        None => text.range(),
    };
    #[cfg(test)]
    if crate::internal::amortized::fault::refuses_text_share() {
        return Err(HtmlConstructionCause::InvalidTextReference.error());
    }
    block
        .string(range)
        .map_err(|_| HtmlConstructionCause::InvalidTextReference.error())
}

/// Moves a packed block's cursor past `units` units that this pass does not make a string of.
fn skip_units(cursor: &mut Option<usize>, units: usize) -> Result<(), HtmlConstructionError> {
    if let Some(next) = cursor {
        *next = next
            .checked_add(units)
            .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
    }
    Ok(())
}
