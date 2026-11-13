//! Publication of the storage as an immutable result.

use super::shape::validate_record;
use super::*;
use crate::internal::construction::attribute_table::PublishedAttr;
use crate::internal::construction::tree::{
    Assoc, AssocKind, AttrSpan, Leaf, PackedShadow, RowOwner, Span,
};
use std::ops::Range;

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

/// What a publication has made before it frees the records: the root it publishes and the vectors
/// reserved and filled for the result.
struct Prepared {
    root: NodeIx,
    mode: Option<HtmlDocumentMode>,
    rows: Vec<PublishedRow>,
    children: Vec<NodeIx>,
    associations: Vec<Assoc>,
    attrs: Vec<PublishedAttr>,
    runs: Vec<AttrSpan>,
    leaves: Vec<Leaf>,
    block: TextBlock,
}

/// The banks a publication records with its consumer: the wrapper, the attribute rows, the runs,
/// the text block, the rows, the children, the associations and the leaves.
const TRAIL_BANKS: usize = 8;

/// What a publication has recorded with its consumer for the banks it grew, in order, so that a
/// refusal can take each record back.
#[derive(Default)]
struct BankTrail {
    entries: [Option<TrailEntry>; TRAIL_BANKS],
    len: usize,
}

/// A bank of a publication and the dimensions the consumer holds for it.
#[derive(Clone, Copy)]
struct TrailEntry {
    growth: HtmlConstructionBankGrowth,
    recorded: HtmlConstructionBankCapacity,
}

impl BankTrail {
    /// Notes a bank the consumer holds `recorded` for. A publication grows at most eight banks,
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
    /// its associations, and that every name, text range, attribute run and value a record names is
    /// in the storage. A refusal hands the storage back with its records, their order, depths,
    /// text and usage as they were. The validation passes may leave their scratch counts written,
    /// which are not state.
    ///
    /// The result holds one 20-byte row for each record, the children of every row in one array,
    /// the template and shadow associations in one table, the attributes of every element in one
    /// table with a run for each element that has attributes, and its text as one block. Every
    /// allocation of the publication is reserved before the conversion, each admitted and recorded
    /// in turn: the shared tree, the published attributes (one row for each live row of every
    /// run) and their runs, the block when the arena has more than a quarter of its units spare,
    /// which it is then packed into, the rows, the child entries, the associations and the leaves
    /// (the processing instructions and doctypes). A bank the result has no entry for is not
    /// reserved. A refusal at any step takes back, in reverse order, each record the consumer
    /// accepted, so the consumer holds what it held before the publication; the consumer's own
    /// refusal of a take-back is the refusal returned. The conversion that follows allocates no
    /// reservation: one walk fills the vectors, and the move that ends it frees the records of the
    /// storage and the table of its attributes. The handle of the shared block is made before the
    /// conversion and the handle of the shared tree in the move, for which stable Rust has no
    /// fallible constructor: an allocator failure there aborts, as any infallible allocation does.
    /// The banks of the storage that the move frees are the consumer's to credit.
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

    /// Reserves and fills the vectors of the result. The parent and child lists, the associations
    /// and the names, text ranges, runs and values of the records are checked before the first
    /// bank is admitted, so a refusal of that validation reserves nothing. The walk that fills the
    /// vectors follows the last admission and allocates nothing, but it can still refuse: it
    /// shares each text and value from the block, and a range the block cannot share
    /// (`text_span`, `shared_string`) or an index that no longer fits is refused as the typed
    /// cause. That refusal returns the units of the block to the arena, and the caller takes back
    /// every bank the consumer recorded, so it leaves the storage and the consumer as they were.
    /// The leaves have no reference to check: the walk assigns the position of a leaf as it pushes
    /// the leaf.
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
        // Every name, range, run and value a record names is in the storage, before anything is
        // reserved or moved, and the sizes of the vectors of the result are counted.
        let mut used = 0usize;
        let mut rows_total = 0usize;
        let mut runs_total = 0usize;
        let mut children_total = 0usize;
        let mut associations_total = 0usize;
        let mut leaves_total = 0usize;
        for record in &self.arena.records {
            children_total = plus(children_total, record.children.len()).map_err(faulted)?;
            associations_total = plus(
                associations_total,
                usize::from(record.template_contents.is_some())
                    + usize::from(record.shadow.is_some()),
            )
            .map_err(faulted)?;
            match &record.kind {
                NodeContent::Text(run) | NodeContent::Comment(run) => {
                    self.text.get(run.text()).map_err(faulted)?;
                    used = plus(used, run.len()).map_err(faulted)?;
                }
                NodeContent::Element(element) => {
                    let held = usize::try_from(element.name.number())
                        .ok()
                        .and_then(|number| self.arena.names.get(number));
                    if !held.is_some_and(|entry| Arc::ptr_eq(entry, &element.name)) {
                        return Err(fault(HtmlConstructionCause::InvalidNameReference));
                    }
                    let rows = self.attrs.live(element.attrs).map_err(faulted)?;
                    for row in rows {
                        self.text.get(row.value()).map_err(faulted)?;
                        used = plus(used, row.value().len()).map_err(faulted)?;
                    }
                    rows_total = plus(rows_total, rows.len()).map_err(faulted)?;
                    runs_total =
                        plus(runs_total, usize::from(!rows.is_empty())).map_err(faulted)?;
                }
                NodeContent::ProcessingInstruction(_) | NodeContent::Doctype(_) => {
                    leaves_total = plus(leaves_total, 1).map_err(faulted)?;
                }
                NodeContent::Document(_) | NodeContent::Fragment => {}
            }
        }
        let pack = self.text.wants_packing();
        let has_rows = rows_total != 0;
        let bank =
            |bank, minimum_capacity: usize, element_size: usize| HtmlConstructionBankGrowth {
                bank,
                before: HtmlConstructionBankCapacity::default(),
                minimum_capacity,
                element_size,
                wrapper_bytes: 0,
            };
        let wrapper = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Wrapper,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: 1,
            element_size: size_of::<Tree>(),
            // The handle of the shared block.
            wrapper_bytes: size_of::<Vec<u16>>(),
        };
        let attributes = has_rows.then(|| {
            bank(
                HtmlConstructionBank::PublishedAttributes,
                rows_total,
                size_of::<PublishedAttr>(),
            )
        });
        let runs = (runs_total != 0).then(|| {
            bank(
                HtmlConstructionBank::PublishedRuns,
                runs_total,
                size_of::<AttrSpan>(),
            )
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
        let published = bank(
            HtmlConstructionBank::PublishedRows,
            count,
            size_of::<PublishedRow>(),
        );
        let children = (children_total != 0).then(|| {
            bank(
                HtmlConstructionBank::PublishedChildren,
                children_total,
                size_of::<NodeIx>(),
            )
        });
        let associations = (associations_total != 0).then(|| {
            bank(
                HtmlConstructionBank::PublishedAssociations,
                associations_total,
                size_of::<Assoc>(),
            )
        });
        let leaves = (leaves_total != 0).then(|| {
            bank(
                HtmlConstructionBank::PublishedLeaves,
                leaves_total,
                size_of::<Leaf>(),
            )
        });
        // The banks, the handle of the shared block and the allocation of the shared tree.
        let allocations = plus(
            [
                attributes.is_some(),
                runs.is_some(),
                block.is_some(),
                children.is_some(),
                associations.is_some(),
                leaves.is_some(),
            ]
            .into_iter()
            .filter(|present| *present)
            .count(),
            4,
        )
        .map_err(faulted)?;
        Work {
            provider_calls: allocations,
            ..Work::default()
        }
        .charge(admission)?;
        // The work of the conversion: the units a pack copies, then the rows, child entries, runs,
        // associations, leaves and attribute rows it writes.
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
        let written = |count: usize, size: usize| times(count, size).map_err(faulted);
        Work {
            record_visits: count,
            copied_bytes: plus(
                plus(
                    plus(
                        written(count, size_of::<PublishedRow>())?,
                        written(children_total, size_of::<NodeIx>())?,
                    )
                    .map_err(faulted)?,
                    plus(
                        written(runs_total, size_of::<AttrSpan>())?,
                        written(associations_total, size_of::<Assoc>())?,
                    )
                    .map_err(faulted)?,
                )
                .map_err(faulted)?,
                plus(
                    written(leaves_total, size_of::<Leaf>())?,
                    written(rows_total, size_of::<PublishedAttr>())?,
                )
                .map_err(faulted)?,
            )
            .map_err(faulted)?,
            record_writes: plus(
                plus(
                    plus(count, children_total).map_err(faulted)?,
                    plus(runs_total, associations_total).map_err(faulted)?,
                )
                .map_err(faulted)?,
                plus(leaves_total, rows_total).map_err(faulted)?,
            )
            .map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        // Each bank is admitted and recorded in turn, the banks that add bytes before the one
        // that replaces the arena, so that the bytes each admission checks are the bytes the
        // records before it leave and the high water the consumer observes is what the
        // admissions required. `trail` notes each record, which a refusal takes back.
        let wrapper_chosen = Self::admit_bank(wrapper, admission)?;
        let wrapper_planned = HtmlConstructionBankCapacity {
            initialized_len: 0,
            capacity: wrapper_chosen,
        };
        // The shared tree is made in the share that follows, which cannot refuse.
        self.grow_recorded(trail, wrapper, wrapper_chosen, admission, |_, _| Reserved {
            ok: true,
            reached: wrapper_planned,
        })?;
        let mut attrs = Vec::new();
        if let Some(growth) = attributes {
            self.reserve_published(trail, growth, admission, &mut attrs)?;
        }
        let mut spans = Vec::new();
        if let Some(growth) = runs {
            self.reserve_published(trail, growth, admission, &mut spans)?;
        }
        let mut packed = Vec::new();
        if let Some(growth) = block {
            self.reserve_published(trail, growth, admission, &mut packed)?;
        }
        let mut rows = Vec::new();
        self.reserve_published(trail, published, admission, &mut rows)?;
        let mut listed = Vec::new();
        if let Some(growth) = children {
            self.reserve_published(trail, growth, admission, &mut listed)?;
        }
        let mut associated = Vec::new();
        if let Some(growth) = associations {
            self.reserve_published(trail, growth, admission, &mut associated)?;
        }
        let mut held = Vec::new();
        if let Some(growth) = leaves {
            self.reserve_published(trail, growth, admission, &mut held)?;
        }
        // The block is made from the units while the storage still holds its records, so that a
        // text or value the block cannot share is refused by the walk below, with the storage
        // whole: the walk restores the units of an unpacked block, and the caller takes back the
        // banks.
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
        // One walk makes the rows, the child entries, the runs, the attribute rows, the
        // associations and the leaves in record order, reading the units of the packed block in
        // the order they were packed.
        let mut parts = Parts {
            rows,
            children: listed,
            associations: associated,
            attrs,
            runs: spans,
            leaves: held,
        };
        if let Err(error) = self.fill(&block, pack, &mut parts) {
            drop(parts);
            // The strings made so far are the only other holders of the block, and they are
            // dropped before the units are taken back, so `block` is then the only holder and
            // `into_units` returns the vector. Its refusal is a typed defect, not a state this
            // function reaches.
            return Err(self.restore_block(block, pack, error));
        }
        let Parts {
            rows,
            children,
            associations,
            attrs,
            runs,
            leaves,
        } = parts;
        Ok(Prepared {
            root,
            mode,
            rows,
            children,
            associations,
            attrs,
            runs,
            leaves,
            block,
        })
    }

    /// Admits `growth`, and grows `slot` to the capacity the consumer chose for it, recording
    /// the growth in `trail`.
    fn reserve_published<T, A: HtmlRecordAdmission>(
        &mut self,
        trail: &mut BankTrail,
        growth: HtmlConstructionBankGrowth,
        admission: &mut A,
        slot: &mut Vec<T>,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let chosen = Self::admit_bank(growth, admission)?;
        self.grow_recorded(trail, growth, chosen, admission, |_, additional| {
            *slot = Vec::new();
            let ok = slot.try_reserve_exact(additional).is_ok();
            Reserved {
                ok,
                reached: HtmlConstructionBankCapacity {
                    initialized_len: 0,
                    capacity: slot.capacity(),
                },
            }
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

    /// The walk of the conversion: the rows of every record in record order, the child entries of
    /// each row after the previous row's, the runs and attribute rows of the elements, the
    /// associations, which are in owner order because the records are, and the leaves.
    fn fill(
        &self,
        block: &TextBlock,
        pack: bool,
        parts: &mut Parts,
    ) -> Result<(), HtmlConstructionError> {
        let mut cursor = pack.then_some(0usize);
        for (position, record) in self.arena.records.iter().enumerate() {
            let owner = NodeIx::new(position)
                .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
            let content = match &record.kind {
                NodeContent::Document(mode) => RowContent::Document {
                    mode: *mode,
                    known_null_focus: record.document_focused_area_designation
                        == Some(HtmlFocusedAreaDesignation::KnownNull),
                },
                NodeContent::Fragment => RowContent::Fragment {
                    shadow: record.shadow_properties.map(PackedShadow::pack),
                    owner: record.owner.map(|owner| match owner {
                        HtmlFragmentOwner::Template(id) => RowOwner {
                            node: id.index,
                            shadow_host: false,
                        },
                        HtmlFragmentOwner::ShadowHost(id) => RowOwner {
                            node: id.index,
                            shadow_host: true,
                        },
                    }),
                },
                NodeContent::Element(element) => {
                    let live = self.attrs.live(element.attrs)?;
                    let start = parts.attrs.len();
                    for row in live {
                        let value = block_string(block, &mut cursor, row.value())?;
                        parts
                            .attrs
                            .push(PublishedAttr::new(row.entry().clone(), value));
                    }
                    let run = if live.is_empty() {
                        None
                    } else {
                        parts.runs.push(
                            AttrSpan::new(start, live.len())
                                .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?,
                        );
                        let stored = u32::try_from(parts.runs.len())
                            .ok()
                            .and_then(NonZeroU32::new)
                            .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
                        Some(stored)
                    };
                    RowContent::Element {
                        link_flags: element.link_flags,
                        name: element.name.number(),
                        run,
                    }
                }
                NodeContent::Text(run) => {
                    RowContent::Text(text_span(block, &mut cursor, run.text())?)
                }
                NodeContent::Comment(run) => {
                    RowContent::Comment(text_span(block, &mut cursor, run.text())?)
                }
                NodeContent::ProcessingInstruction(value) => {
                    let at = leaf_position(parts.leaves.len())?;
                    parts
                        .leaves
                        .push(Leaf::ProcessingInstruction(value.clone()));
                    RowContent::ProcessingInstruction(at)
                }
                NodeContent::Doctype(value) => {
                    let at = leaf_position(parts.leaves.len())?;
                    parts.leaves.push(Leaf::Doctype(value.clone()));
                    RowContent::Doctype(at)
                }
            };
            parts.children.extend(record.children.iter().copied());
            parts.rows.push(PublishedRow {
                parent: record.parent,
                child_end: u32::try_from(parts.children.len())
                    .map_err(|_| HtmlConstructionCause::RepresentationExhausted.error())?,
                content,
            });
            if let Some(contents) = record.template_contents {
                parts.associations.push(Assoc {
                    owner,
                    contents: contents.position(),
                    kind: match contents {
                        TemplateContents::Owned(_) => AssocKind::Owned,
                        TemplateContents::ShadowReference(_) => AssocKind::ShadowReference,
                    },
                });
            }
            if let Some(shadow) = &record.shadow {
                let properties = self
                    .record(shadow.contents.index)?
                    .shadow_properties
                    .ok_or(HtmlConstructionCause::InvalidShadowAssociation.error())?;
                parts.associations.push(Assoc {
                    owner,
                    contents: shadow.contents.index,
                    kind: AssocKind::Shadow(PackedShadow::pack(properties)),
                });
            }
        }
        Ok(())
    }

    /// Frees the records and the tables of the storage and makes the shared tree of the result.
    fn share(self, prepared: Prepared) -> HtmlConstructionResult {
        let Prepared {
            root,
            mode,
            rows,
            children,
            associations,
            attrs,
            runs,
            leaves,
            block,
        } = prepared;
        let BuildArena {
            names,
            usage,
            serial,
            ..
        } = self.arena;
        let tree = Arc::new(Tree {
            rows,
            children,
            associations,
            names,
            attrs,
            runs,
            block,
            leaves,
            usage,
            serial,
        });
        match mode {
            Some(mode) => HtmlConstructionResult::Document(HtmlDocument { tree, root, mode }),
            None => HtmlConstructionResult::Fragment(HtmlFragment { tree, root }),
        }
    }
}

/// The vectors a publication fills.
struct Parts {
    rows: Vec<PublishedRow>,
    children: Vec<NodeIx>,
    associations: Vec<Assoc>,
    attrs: Vec<PublishedAttr>,
    runs: Vec<AttrSpan>,
    leaves: Vec<Leaf>,
}

/// The position of the next leaf as the index a row holds.
fn leaf_position(len: usize) -> Result<u32, HtmlConstructionError> {
    u32::try_from(len).map_err(|_| HtmlConstructionCause::RepresentationExhausted.error())
}

/// The range of the block for the units of `text`: the range `text` names, or the next range of a
/// packed block, whose units follow in record order, when `cursor` holds its position.
fn next_range(
    cursor: &mut Option<usize>,
    text: crate::internal::construction::text::TextRef,
) -> Result<Range<usize>, HtmlConstructionError> {
    Ok(match cursor {
        Some(next) => {
            let start = *next;
            let end = start
                .checked_add(text.len())
                .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
            *next = end;
            start..end
        }
        None => text.range(),
    })
}

/// The string of `block` for the range `range`, or the typed refusal that the block cannot share
/// it.
fn shared_string(
    block: &TextBlock,
    range: Range<usize>,
) -> Result<DocumentString, HtmlConstructionError> {
    #[cfg(test)]
    if crate::internal::amortized::fault::refuses_text_share() {
        return Err(HtmlConstructionCause::InvalidTextReference.error());
    }
    block
        .string(range)
        .map_err(|_| HtmlConstructionCause::InvalidTextReference.error())
}

/// The string of `block` for the units of `text`; see [`next_range`].
fn block_string(
    block: &TextBlock,
    cursor: &mut Option<usize>,
    text: crate::internal::construction::text::TextRef,
) -> Result<DocumentString, HtmlConstructionError> {
    shared_string(block, next_range(cursor, text)?)
}

/// The span a text row holds for the units of `text`, after checking that the block shares the
/// range, which is the one reading the row makes.
fn text_span(
    block: &TextBlock,
    cursor: &mut Option<usize>,
    text: crate::internal::construction::text::TextRef,
) -> Result<Span, HtmlConstructionError> {
    let range = next_range(cursor, text)?;
    shared_string(block, range.clone())?;
    Span::new(range).ok_or(HtmlConstructionCause::RepresentationExhausted.error())
}
