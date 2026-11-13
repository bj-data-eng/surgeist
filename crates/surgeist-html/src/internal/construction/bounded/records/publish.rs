//! Publication of the storage as an immutable result.

use super::shape::{validate_association, validate_row};
use super::*;
use crate::internal::construction::attribute_table::PublishedAttr;
use crate::internal::construction::rows::{BuildAssocKind, NodeFlags, RowKind};
use crate::internal::construction::tree::{Assoc, AssocKind, AttrSpan, Leaf, RowOwner, Span};
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

/// What the validation of a publication counts, which sizes the banks of the result.
#[derive(Default)]
struct Totals {
    /// The units the block holds: the texts and the attribute values.
    used: usize,
    /// The attribute rows and runs of the elements.
    rows: usize,
    runs: usize,
    /// The children listed, and so the rows that have a parent.
    children: usize,
    associations: usize,
    leaves: usize,
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
    /// text and usage as they were. The validation walk writes nothing.
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
        let row = self.arena.row(root).map_err(faulted)?;
        let mode = match row.kind {
            RowKind::Document(mode) => Some(mode),
            RowKind::Fragment => None,
            _ => return Err(fault(HtmlConstructionCause::InvalidOutputRoot)),
        };
        if row.parent.is_some() {
            return Err(fault(HtmlConstructionCause::InvalidOutputRoot));
        }
        let count = self.arena.rows.len();
        // One walk over the links validates every row and lists every child once, and checks
        // every name, text range, run, value and leaf a row names. It reads the storage and
        // changes nothing, and it comes before the first bank is admitted, so a refusal of the
        // validation reserves nothing.
        let Totals {
            used,
            rows: rows_total,
            runs: runs_total,
            children: children_total,
            associations: associations_total,
            leaves: leaves_total,
        } = self.validate_rows(admission)?;
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
            for row in &self.arena.rows {
                match row.kind {
                    RowKind::Text | RowKind::Comment => {
                        let run = row
                            .text_run()
                            .ok_or_else(|| fault(HtmlConstructionCause::InvalidTextReference))?;
                        packed.extend_from_slice(self.text.get(run.text()).map_err(faulted)?);
                    }
                    RowKind::Element => {
                        for attribute in self.attrs.live(row.run()).map_err(faulted)? {
                            packed.extend_from_slice(
                                self.text.get(attribute.value()).map_err(faulted)?,
                            );
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

    /// The one walk over the rows and their links that validates the storage for publication.
    ///
    /// Each row is checked with its parent, its place among its siblings and its association, and
    /// the walk follows the chain of its children from its first child: every child names the row
    /// as its parent and the child before it as its previous sibling, and the chain ends at the
    /// row's last child. A row names one parent and one previous sibling, so a row that the chains
    /// list is listed once, and the children listed equal the rows that have a parent exactly when
    /// every such row is listed. The associations are checked first, against the rows they name,
    /// and then each row against the associations that name it.
    fn validate_rows<A: HtmlRecordAdmission>(
        &self,
        admission: &mut A,
    ) -> Result<Totals, HtmlConstructionRecordError<A::Stop>> {
        let conflict = || fault(HtmlConstructionCause::ParentChildConflict);
        let count = self.arena.rows.len();
        let mut totals = Totals::default();
        let mut last: Option<(u32, usize)> = None;
        let mut owners = 0usize;
        for entry in &self.arena.associations {
            Work::visit(admission)?;
            validate_association(&self.arena, entry).map_err(faulted)?;
            let key = (
                entry.owner.stored(),
                usize::from(entry.kind == BuildAssocKind::Shadow),
            );
            if last.is_some_and(|last| last >= key) {
                return Err(fault(HtmlConstructionCause::InvalidTemplateAssociation));
            }
            owners = plus(owners, usize::from(last.is_none_or(|last| last.0 != key.0)))
                .map_err(faulted)?;
            last = Some(key);
        }
        let (mut with_parent, mut with_associations) = (0usize, 0usize);
        for position in 0..count {
            Work::visit(admission)?;
            let index = NodeIx::new(position)
                .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
            validate_row(&self.arena, index).map_err(faulted)?;
            let row = self.arena.row(index).map_err(faulted)?;
            with_parent = plus(with_parent, usize::from(row.parent.is_some())).map_err(faulted)?;
            with_associations = plus(
                with_associations,
                usize::from(row.flags.has(NodeFlags::HAS_ASSOC)),
            )
            .map_err(faulted)?;
            let (mut cursor, mut previous, mut steps) = (row.first(), None, 0usize);
            while let Some(child) = cursor {
                Work::visit(admission)?;
                steps = plus(steps, 1).map_err(faulted)?;
                if steps > count {
                    return Err(conflict());
                }
                let target = self.arena.row(child).map_err(faulted)?;
                if target.parent != Some(index)
                    || target.prev != previous
                    || target.flags.has(NodeFlags::OWNED)
                    || matches!(target.kind, RowKind::Document(_) | RowKind::Fragment)
                {
                    return Err(conflict());
                }
                totals.children = plus(totals.children, 1).map_err(faulted)?;
                previous = Some(child);
                cursor = target.next;
            }
            if previous != row.last() {
                return Err(conflict());
            }
            match row.kind {
                RowKind::Text | RowKind::Comment => {
                    let run = row
                        .text_run()
                        .ok_or_else(|| fault(HtmlConstructionCause::InvalidTextReference))?;
                    self.text.get(run.text()).map_err(faulted)?;
                    totals.used = plus(totals.used, run.len()).map_err(faulted)?;
                }
                RowKind::Element => {
                    let held = usize::try_from(row.a)
                        .ok()
                        .and_then(|number| self.arena.names.get(number));
                    if held.is_none() {
                        return Err(fault(HtmlConstructionCause::InvalidNameReference));
                    }
                    let attributes = self.attrs.live(row.run()).map_err(faulted)?;
                    for attribute in attributes {
                        self.text.get(attribute.value()).map_err(faulted)?;
                        totals.used =
                            plus(totals.used, attribute.value().len()).map_err(faulted)?;
                    }
                    totals.rows = plus(totals.rows, attributes.len()).map_err(faulted)?;
                    totals.runs =
                        plus(totals.runs, usize::from(!attributes.is_empty())).map_err(faulted)?;
                }
                RowKind::ProcessingInstruction | RowKind::Doctype => {
                    let held = usize::try_from(row.a)
                        .ok()
                        .and_then(|position| self.arena.leaves.get(position));
                    if !matches!(
                        (row.kind, held),
                        (
                            RowKind::ProcessingInstruction,
                            Some(Leaf::ProcessingInstruction(_))
                        ) | (RowKind::Doctype, Some(Leaf::Doctype(_)))
                    ) {
                        return Err(fault(HtmlConstructionCause::InvalidLeafReference));
                    }
                    totals.leaves = plus(totals.leaves, 1).map_err(faulted)?;
                }
                RowKind::Document(_) | RowKind::Fragment => {}
            }
        }
        if totals.children != with_parent {
            return Err(conflict());
        }
        if owners != with_associations {
            return Err(fault(HtmlConstructionCause::InvalidTemplateAssociation));
        }
        totals.associations = self.arena.associations.len();
        Ok(totals)
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
    /// each row after the previous row's, which the links of the row list in order, the runs and
    /// attribute rows of the elements, the leaves, and then the associations, which are in owner
    /// order because the table is. Validation checked every reference and chain this walk follows,
    /// so a lookup that fails here is a defect and is returned.
    fn fill(
        &self,
        block: &TextBlock,
        pack: bool,
        parts: &mut Parts,
    ) -> Result<(), HtmlConstructionError> {
        let mut cursor = pack.then_some(0usize);
        for row in &self.arena.rows {
            let content = match row.kind {
                RowKind::Document(mode) => RowContent::Document {
                    mode,
                    known_null_focus: row.flags.has(NodeFlags::KNOWN_NULL_FOCUS),
                },
                RowKind::Fragment => RowContent::Fragment {
                    shadow: row.shadow(),
                    owner: row.owner().map(|node| RowOwner {
                        node,
                        shadow_host: row.shadow().is_some(),
                    }),
                },
                RowKind::Element => {
                    let live = self.attrs.live(row.run())?;
                    let start = parts.attrs.len();
                    for attribute in live {
                        let value = block_string(block, &mut cursor, attribute.value())?;
                        parts
                            .attrs
                            .push(PublishedAttr::new(attribute.entry().clone(), value));
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
                        link_flags: row.link_flags(),
                        name: row.a,
                        run,
                    }
                }
                RowKind::Text | RowKind::Comment => {
                    let run = row
                        .text_run()
                        .ok_or(HtmlConstructionCause::InvalidTextReference.error())?;
                    let span = text_span(block, &mut cursor, run.text())?;
                    if row.kind == RowKind::Text {
                        RowContent::Text(span)
                    } else {
                        RowContent::Comment(span)
                    }
                }
                RowKind::ProcessingInstruction | RowKind::Doctype => {
                    let at = leaf_position(parts.leaves.len())?;
                    let leaf = usize::try_from(row.a)
                        .ok()
                        .and_then(|position| self.arena.leaves.get(position))
                        .ok_or(HtmlConstructionCause::InvalidLeafReference.error())?;
                    parts.leaves.push(leaf.clone());
                    if row.kind == RowKind::Doctype {
                        RowContent::Doctype(at)
                    } else {
                        RowContent::ProcessingInstruction(at)
                    }
                }
            };
            let mut child = row.first();
            while let Some(next) = child {
                if parts.children.len() >= parts.children.capacity() {
                    return Err(HtmlConstructionCause::CapacityMismatch.error());
                }
                parts.children.push(next);
                child = self.arena.row(next)?.next;
            }
            parts.rows.push(PublishedRow {
                parent: row.parent,
                child_end: u32::try_from(parts.children.len())
                    .map_err(|_| HtmlConstructionCause::RepresentationExhausted.error())?,
                content,
            });
        }
        for entry in &self.arena.associations {
            parts.associations.push(Assoc {
                owner: entry.owner,
                contents: entry.contents,
                kind: match entry.kind {
                    BuildAssocKind::Owned => AssocKind::Owned,
                    BuildAssocKind::ShadowReference => AssocKind::ShadowReference,
                    BuildAssocKind::Shadow => AssocKind::Shadow(
                        self.arena
                            .row(entry.contents)?
                            .shadow()
                            .ok_or(HtmlConstructionCause::InvalidShadowAssociation.error())?,
                    ),
                },
            });
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
