//! Record operations of the construction storage: creation, insertion, detachment, shadow and
//! template associations, publication.
//!
//! Each operation is one method of [`HtmlConstructionStorage`]. It qualifies its operands, charges
//! its consumer for its work before the work runs, and admits itself once through
//! [`HtmlRecordAdmission`]. Only then does it change the storage, and it changes it last, after
//! every step that can refuse:
//!
//! 1. qualification and planning, which read the records and charge the records they visit;
//! 2. [`HtmlRecordAdmission::admit_change`], where the consumer checks the limits of the usage the
//!    change leaves, after the operation recorded the depth and the layout it reaches;
//! 3. the charge of the writes and copies the change makes, then
//!    [`HtmlRecordAdmission::admit_growth`] for every bank the change grows;
//! 4. for each such bank in turn, [`HtmlRecordAdmission::record_growth`] with the capacity the
//!    consumer chose, the reservation, and a second record when the reservation reached another
//!    capacity; a refused reservation of spare room is retried with the room that is needed, and
//!    the retry is charged as one provider call;
//! 5. the strings of a leaf, built where they are reserved, so that every allocation of a leaf
//!    comes before the last step that can refuse;
//! 6. a check that every record the writes name exists, then [`HtmlRecordAdmission::record_change`],
//!    the last step that can refuse;
//! 7. the writes. They reserve and allocate nothing. A lookup of a record that qualification read
//!    cannot fail; were one to fail, the typed unknown-node defect is returned and no write is
//!    skipped.
//!
//! A refusal at any step before the writes leaves the records, their order, their depths and the
//! usage as they were. The residues are the capacities of the banks: a bank that grew keeps the
//! room it reserved, and when a later step of the same operation refuses, the consumer holds the
//! capacity it last recorded for that bank. A string that a leaf creation had reserved and then
//! dropped is such a bank; the consumer holds the capacity it recorded for it. Walk scratch
//! (cursors and level counts) is not state and may be left written. A publication takes back
//! what it recorded when it refuses, so it leaves no residue. The one allocation that follows
//! the last refusable step is the handle of the shared arena of a publication, for which stable
//! Rust has no fallible constructor: an allocator failure there aborts, as any infallible
//! allocation does.
//!
//! The work of these operations is counted in the records they write, the bytes of rows and units
//! they copy, one provider call for every request to the allocator, and the records they visit. A
//! visit is charged for each record a qualification or a walk reads. An insertion of a fresh
//! childless node visits a number of records that does not depend on the fanout of its parent or
//! the depth of the tree.

use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use crate::internal::construction::attribute_table::{AttrRow, AttrRun, RowAppend};
use crate::internal::construction::names::NameKey;
use crate::internal::values::{DocumentString, OrderedAttributes, QualifiedName};
use surgeist_ledger::{Change, Charge};

mod association;
mod depth;
mod names;
mod order;
mod publish;
mod shape;
#[cfg(test)]
mod tests;

pub use names::NameId;
pub use publish::HtmlConstructionPublishRefusal;
pub(in crate::internal::construction) use shape::{attribute_units, name_units};

/// The units one charge covers when a creation copies a run of units, so that a consumer that
/// polls at its charges polls within every quantum of the work.
const CHUNK_UNITS: usize = 256 / size_of::<u16>();

/// A refusal of a record operation: the consumer's stop, or the operation's own fault.
pub type HtmlConstructionRecordError<S> = HtmlChargedError<S, HtmlConstructionError>;

/// A bank of the storage that a record operation grows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionBank {
    /// The vector of retained records.
    Records,
    /// The rows of the attribute table, which hold the attributes of every element of the
    /// storage. It grows with an attribute that finds its element's run full, and with a run
    /// that an element reserves.
    AttributeRows,
    /// The runs of the attribute table, one for each element that has attributes.
    AttributeRuns,
    /// The child order of the record at `position`.
    Children {
        /// The ordinal of the record whose child order grows.
        position: usize,
    },
    /// The units of one string of a processing-instruction or doctype record that a creation
    /// makes. It has no capacity before the creation, and the consumer also holds the wrapper
    /// that shares the string.
    Text,
    /// The text arena, which holds the units of every text and comment record of the storage. It
    /// grows with a creation of a text or comment and with an append that moves or extends a text.
    TextArena,
    /// The block that publication packs the text arena into when more than a quarter of the
    /// arena's units are spare. Its growth names the capacity of the arena, which the block
    /// replaces, as the bank it grows from.
    TextBlock,
    /// The vector of rows that publication makes, one for each record of the storage.
    PublishedRows,
    /// The array that holds the children of every published row, one entry for each record that
    /// has a parent.
    PublishedChildren,
    /// The table of the attributes of the published elements, which publication makes from the
    /// live rows of the storage's attribute table.
    PublishedAttributes,
    /// The runs of the published attribute table, one for each element that has attributes.
    PublishedRuns,
    /// The table of the template and shadow associations of the result, one entry for each
    /// template contents and each shadow root.
    PublishedAssociations,
    /// The processing instructions and doctypes of the result.
    PublishedLeaves,
    /// The shared tree that publication creates, with the wrapper that shares the block of the
    /// result.
    Wrapper,
    /// The vector of the entries of the names the storage has interned.
    Names,
    /// The slots of the table that finds an interned name, a power of two in number.
    NameSlots,
    /// The allocation of one interned name entry. It has no capacity before the name is
    /// interned.
    NameEntry,
}

/// Actual dimensions of a bank; no allocation or release authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionBankCapacity {
    initialized_len: usize,
    capacity: usize,
}
impl HtmlConstructionBankCapacity {
    /// The number of elements the bank holds.
    pub fn initialized_len(self) -> usize {
        self.initialized_len
    }
    /// The reserved capacity in elements; at least [`initialized_len`](Self::initialized_len).
    pub fn capacity(self) -> usize {
        self.capacity
    }
}

/// What an operation asks its consumer to admit before it reserves a bank: the bank must hold at
/// least [`minimum_capacity`](Self::minimum_capacity) elements afterwards.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionBankGrowth {
    bank: HtmlConstructionBank,
    before: HtmlConstructionBankCapacity,
    minimum_capacity: usize,
    element_size: usize,
    wrapper_bytes: usize,
}
impl HtmlConstructionBankGrowth {
    /// The bank that grows.
    pub fn bank(self) -> HtmlConstructionBank {
        self.bank
    }
    /// The dimensions of the bank before the growth.
    pub fn before(self) -> HtmlConstructionBankCapacity {
        self.before
    }
    /// The least capacity that holds what the operation stores.
    pub fn minimum_capacity(self) -> usize {
        self.minimum_capacity
    }
    /// The size in bytes of one element of the bank.
    pub fn element_size(self) -> usize {
        self.element_size
    }
    /// The bytes of the wrapper that holds the bank, besides its elements: the vector a shared
    /// string is held in. Zero for a bank that its owner holds inline.
    pub fn wrapper_bytes(self) -> usize {
        self.wrapper_bytes
    }
}

/// One record of a growth, for the consumer to hold: the bank held `recorded` before this record
/// and holds `reached` after it. The first record of a growth carries the capacity the consumer
/// chose, before anything is reserved. A second carries the capacity the reservation reached,
/// when that differs, which includes the unchanged capacity of a reservation the allocator
/// refused. A third takes a bank back to the dimensions it had before the growth, when a
/// publication refuses after it recorded the bank: `reached` equals the dimensions before the
/// growth and `chosen` their capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionGrowthRecord {
    growth: HtmlConstructionBankGrowth,
    chosen: usize,
    recorded: HtmlConstructionBankCapacity,
    reached: HtmlConstructionBankCapacity,
}
impl HtmlConstructionGrowthRecord {
    /// The growth this record belongs to.
    pub fn growth(self) -> HtmlConstructionBankGrowth {
        self.growth
    }
    /// The capacity the consumer chose for the growth.
    pub fn chosen(self) -> usize {
        self.chosen
    }
    /// The dimensions the consumer held for the bank before this record.
    pub fn recorded(self) -> HtmlConstructionBankCapacity {
        self.recorded
    }
    /// The dimensions the bank has now.
    pub fn reached(self) -> HtmlConstructionBankCapacity {
        self.reached
    }
}

/// The usage an operation leaves, beside the usage before it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionChange {
    pub(super) before: HtmlConstructionUsage,
    pub(super) after: HtmlConstructionUsage,
}
impl HtmlConstructionChange {
    /// The usage before the operation.
    pub fn before(self) -> HtmlConstructionUsage {
        self.before
    }
    /// The usage the operation leaves in place.
    pub fn after(self) -> HtmlConstructionUsage {
        self.after
    }
}

/// The consumer of a record operation. It charges the operation's work like any [`Charge`], and
/// admits and records what the operation changes.
pub trait HtmlRecordAdmission: Charge<HtmlWork, HtmlResource> {
    /// Admits the usage a change leaves, once, before anything changes or is reserved. `storage`
    /// is the storage as it is. The operation has already recorded the depth and the layout the
    /// change reaches as the largest of their resources. A refusal changes nothing.
    fn admit_change(
        &mut self,
        storage: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), Self::Stop>;

    /// Admits the growth of one bank before it is reserved. Returns the capacity to reserve,
    /// which must be at least the growth's minimum; a smaller one is refused with the
    /// capacity-mismatch cause and nothing is reserved. Every growth of an operation is admitted
    /// before the first is recorded.
    fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, Self::Stop>;

    /// Records the capacity of a bank. The first record of a growth is made with the capacity the
    /// consumer chose, before it is reserved, so a refusal there leaves the bank as it was. A
    /// refusal of the second record leaves the bank with the capacity it reached, and the
    /// consumer holds the capacity it last accepted, the one chosen; a consumer that holds the
    /// record it refuses, as one does for an overshoot of the layout it admitted, holds the
    /// capacity reached, and keeps the difference `reached − recorded` after a take-back until it
    /// releases the bank.
    ///
    /// A third kind of record arrives from a publication that refuses after it recorded banks: it
    /// takes a bank back. Its `reached` is the dimensions the bank had before the growth, its
    /// `recorded` is what the consumer holds for the bank, and its `chosen` is the capacity of
    /// those dimensions. It credits the bank and admits no growth, so a consumer that reports a
    /// layout for the growth under way keeps the layout of the last growth it admitted.
    fn record_growth(&mut self, record: HtmlConstructionGrowthRecord) -> Result<(), Self::Stop>;

    /// Records the usage the change leaves: the last step that can refuse, after every bank is
    /// reserved and before the first write.
    fn record_change(
        &mut self,
        storage: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), Self::Stop>;
}

pub(super) fn fault<S>(cause: HtmlConstructionCause) -> HtmlConstructionRecordError<S> {
    HtmlChargedError::Faulted(cause.error())
}
pub(super) fn faulted<S>(error: HtmlConstructionError) -> HtmlConstructionRecordError<S> {
    HtmlChargedError::Faulted(error)
}
pub(super) fn stopped<S>(stop: S) -> HtmlConstructionRecordError<S> {
    HtmlChargedError::Stopped(stop)
}
pub(super) fn plus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_add(b)
        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())
}
pub(super) fn minus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_sub(b)
        .ok_or(HtmlConstructionCause::ReleaseUnderflow.error())
}
pub(super) fn times(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_mul(b)
        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())
}

/// The work of one batch of an operation.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Work {
    pub(super) record_visits: usize,
    pub(super) inspected_bytes: usize,
    pub(super) provider_calls: usize,
    pub(super) copied_bytes: usize,
    pub(super) record_writes: usize,
}
impl Work {
    pub(super) fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        charge
            .charge(&[
                Change::Work(HtmlWork::RecordVisit, wide(self.record_visits)),
                Change::Work(HtmlWork::InspectByte, wide(self.inspected_bytes)),
                Change::Work(HtmlWork::ProviderCall, wide(self.provider_calls)),
                Change::Work(HtmlWork::CopyByte, wide(self.copied_bytes)),
                Change::Work(HtmlWork::RecordWrite, wide(self.record_writes)),
            ])
            .map_err(stopped)
    }
    /// One visit of one record.
    pub(super) fn visit<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        Self {
            record_visits: 1,
            ..Self::default()
        }
        .charge(charge)
    }
}

/// The bank a growth targets, and what reserving it leaves behind.
struct Reserved {
    ok: bool,
    reached: HtmlConstructionBankCapacity,
}

impl HtmlConstructionStorage {
    /// Makes an empty construction storage with zero usage, charging `charge` for the arena it
    /// initializes. The storage takes the next process-wide serial, which names it among all
    /// storages: `RepresentationExhausted` when none is left.
    pub fn new<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
    ) -> Result<Self, HtmlConstructionRecordError<C::Stop>> {
        let layout = size_of::<BuildArena>();
        Work {
            provider_calls: 1,
            copied_bytes: layout,
            record_writes: 1,
            ..Work::default()
        }
        .charge(charge)?;
        charge
            .record_largest(HtmlResource::AtomicLayoutBytes, wide(layout))
            .map_err(stopped)?;
        let serial = issue_storage_serial(&NEXT_STORAGE_SERIAL).map_err(fault)?;
        Ok(Self {
            arena: BuildArena {
                records: Vec::new(),
                names: NameTable::new(),
                usage: HtmlConstructionUsage::default(),
                serial,
                attribute_revision: 0,
            },
            text: TextArena::default(),
            attrs: AttributeTable::default(),
        })
    }

    /// The growth the record bank needs to hold `required` records, or `None` when it holds them.
    fn records_growth(&self, required: usize) -> Option<HtmlConstructionBankGrowth> {
        let records = &self.arena.records;
        (required > records.capacity()).then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Records,
            before: HtmlConstructionBankCapacity {
                initialized_len: records.len(),
                capacity: records.capacity(),
            },
            minimum_capacity: required,
            element_size: size_of::<Record>(),
            wrapper_bytes: 0,
        })
    }

    /// The growth the child order of `parent` needs to hold `required` entries, or `None` when it
    /// holds them.
    fn children_growth(
        &self,
        parent: NodeIx,
        required: usize,
    ) -> Option<HtmlConstructionBankGrowth> {
        let children = &self.arena.records.get(parent.position())?.children;
        (required > children.capacity()).then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Children {
                position: parent.position(),
            },
            before: HtmlConstructionBankCapacity {
                initialized_len: children.len(),
                capacity: children.capacity(),
            },
            minimum_capacity: required,
            element_size: size_of::<NodeIx>(),
            wrapper_bytes: 0,
        })
    }

    /// Admits `growth`: the layout it chooses must be representable and at least the minimum.
    pub(super) fn admit_bank<A: HtmlRecordAdmission>(
        growth: HtmlConstructionBankGrowth,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionRecordError<A::Stop>> {
        let chosen = admission.admit_growth(growth).map_err(stopped)?;
        if chosen < growth.minimum_capacity {
            return Err(fault(HtmlConstructionCause::CapacityMismatch));
        }
        times(chosen, growth.element_size)
            .ok()
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        Ok(chosen)
    }

    /// Records `growth` at the capacity `chosen`, reserves it through `reserve`, and records the
    /// capacity reached when it is not the one chosen. `reserve` takes the room to add to the
    /// elements the bank holds and returns whether the allocator gave it and what the bank has
    /// then. Only an allocator refusal of the spare room falls back to the room that is needed.
    fn grow<A: HtmlRecordAdmission>(
        &mut self,
        growth: HtmlConstructionBankGrowth,
        chosen: usize,
        admission: &mut A,
        reserve: impl FnMut(&mut Self, usize) -> Reserved,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let mut recorded = growth.before;
        self.grow_tracked(growth, chosen, admission, reserve, &mut recorded)
    }

    /// [`Self::grow`], which also leaves in `recorded` the dimensions the consumer holds for the
    /// bank after each record it accepted, so that an operation can take them back.
    fn grow_tracked<A: HtmlRecordAdmission>(
        &mut self,
        growth: HtmlConstructionBankGrowth,
        chosen: usize,
        admission: &mut A,
        mut reserve: impl FnMut(&mut Self, usize) -> Reserved,
        recorded: &mut HtmlConstructionBankCapacity,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let len = growth.before.initialized_len;
        let planned = HtmlConstructionBankCapacity {
            initialized_len: len,
            capacity: chosen,
        };
        admission
            .record_growth(HtmlConstructionGrowthRecord {
                growth,
                chosen,
                recorded: growth.before,
                reached: planned,
            })
            .map_err(stopped)?;
        *recorded = planned;
        let needed = minus(growth.minimum_capacity, len).map_err(faulted)?;
        let spare = minus(chosen, len).map_err(faulted)?;
        // A test arms a deterministic refusal of the allocator for the spare room: every growth
        // is counted as one request, and only one that asks for more than it needs is refused.
        #[cfg(test)]
        let spare_refused = crate::internal::amortized::fault::refuses_next_reservation()
            && chosen > growth.minimum_capacity;
        #[cfg(not(test))]
        let spare_refused = false;
        // A test can also refuse every reservation of a child order, the retry included.
        #[cfg(test)]
        let children_refused = matches!(growth.bank, HtmlConstructionBank::Children { .. })
            && crate::internal::amortized::fault::refuses_children();
        #[cfg(not(test))]
        let children_refused = false;
        let refused = spare_refused || children_refused;
        let mut outcome = if refused {
            Reserved {
                ok: false,
                reached: growth.before,
            }
        } else {
            reserve(self, spare)
        };
        if !outcome.ok && chosen > growth.minimum_capacity && !children_refused {
            // The retry is a second request to the allocator.
            Work {
                provider_calls: 1,
                ..Work::default()
            }
            .charge(admission)?;
            outcome = reserve(self, needed);
        }
        if outcome.reached != planned {
            admission
                .record_growth(HtmlConstructionGrowthRecord {
                    growth,
                    chosen,
                    recorded: planned,
                    reached: outcome.reached,
                })
                .map_err(stopped)?;
            *recorded = outcome.reached;
        }
        if outcome.ok {
            Ok(())
        } else {
            Err(fault(HtmlConstructionCause::AllocationFailed))
        }
    }

    fn reserve_record_bank(&mut self, additional: usize) -> Reserved {
        let records = &mut self.arena.records;
        let ok = records.try_reserve_exact(additional).is_ok();
        Reserved {
            ok,
            reached: HtmlConstructionBankCapacity {
                initialized_len: records.len(),
                capacity: records.capacity(),
            },
        }
    }

    fn reserve_child_bank(&mut self, parent: NodeIx, additional: usize) -> Reserved {
        let Some(record) = self.arena.records.get_mut(parent.position()) else {
            return Reserved {
                ok: false,
                reached: HtmlConstructionBankCapacity::default(),
            };
        };
        let ok = record.children.try_reserve_exact(additional).is_ok();
        Reserved {
            ok,
            reached: HtmlConstructionBankCapacity {
                initialized_len: record.children.len(),
                capacity: record.children.capacity(),
            },
        }
    }

    /// Reserves room for `additional` more records, so that creations that follow need not
    /// allocate. The record count and usage are unchanged. A bank that already holds them is not
    /// grown and nothing is admitted.
    pub fn reserve_records<A: HtmlRecordAdmission>(
        &mut self,
        additional: usize,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let required = plus(self.arena.records.len(), additional).map_err(faulted)?;
        u32::try_from(required)
            .map_err(|_| fault(HtmlConstructionCause::RepresentationExhausted))?;
        let growth = self.records_growth(required);
        Work {
            record_visits: 1,
            provider_calls: usize::from(growth.is_some()),
            ..Work::default()
        }
        .charge(admission)?;
        let Some(growth) = growth else {
            return Ok(());
        };
        let chosen = Self::admit_bank(growth, admission)?;
        self.grow(growth, chosen, admission, |storage, additional| {
            storage.reserve_record_bank(additional)
        })
    }

    /// The position of `node` after the visit that reads it.
    fn operand<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        node: &HtmlBuilderNode,
        charge: &mut C,
    ) -> Result<NodeIx, HtmlConstructionRecordError<C::Stop>> {
        Work::visit(charge)?;
        self.arena.index(node).map_err(faulted)
    }

    /// The record at `index` for a write, or the unknown-node error.
    pub(super) fn record_mut(
        &mut self,
        index: NodeIx,
    ) -> Result<&mut Record, HtmlConstructionError> {
        self.arena
            .records
            .get_mut(index.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())
    }

    /// Sets the compatibility mode of the Document record `document`.
    pub fn set_document_mode<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        document: &HtmlBuilderNode,
        mode: HtmlDocumentMode,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        let position = self.operand(document, charge)?;
        let record = self.record_mut(position).map_err(faulted)?;
        if !matches!(record.kind, NodeContent::Document(_)) {
            return Err(fault(HtmlConstructionCause::WrongNodeKind));
        }
        Work {
            copied_bytes: size_of::<NodeContent>(),
            record_writes: 1,
            ..Work::default()
        }
        .charge(charge)?;
        // The record was read above and nothing has changed since.
        self.record_mut(position).map_err(faulted)?.kind = NodeContent::Document(mode);
        Ok(())
    }

    /// Records link construction flags on the element `element`, which must be a `link` element
    /// in the HTML namespace.
    pub fn set_link_flags<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        element: &HtmlBuilderNode,
        flags: crate::internal::elements::rel::HtmlLinkFlags,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        let position = self.operand(element, charge)?;
        let is_link = match &self.record(position).map_err(faulted)?.kind {
            NodeContent::Element(element) => {
                element.key().html() == Some(crate::vocabulary::KnownHtmlElement::Link)
            }
            _ => return Err(fault(HtmlConstructionCause::WrongNodeKind)),
        };
        if !is_link {
            return Err(fault(HtmlConstructionCause::WrongNodeKind));
        }
        Work {
            copied_bytes: size_of::<Option<crate::internal::elements::rel::HtmlLinkFlags>>(),
            record_writes: 1,
            ..Work::default()
        }
        .charge(charge)?;
        // The record is the element read above and nothing has changed since.
        match &mut self.record_mut(position).map_err(faulted)?.kind {
            NodeContent::Element(element) => element.link_flags = Some(flags),
            _ => return Err(fault(HtmlConstructionCause::WrongNodeKind)),
        }
        Ok(())
    }
}

/// Where the units of a leaf string come from: a slice the storage copies into a string of its
/// own, or a string the caller made that the record holds as it is.
pub(in crate::internal::construction) enum Units<'a> {
    Copied(&'a [u16]),
    Shared(DocumentString),
}
impl Units<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Copied(units) => units.len(),
            Self::Shared(string) => string.len(),
        }
    }
}

/// A leaf string after its capacity was admitted: the units still to copy with the growth of the
/// bank they go to and the capacity chosen for it, or the string that needs no copy.
enum Admitted<'a> {
    Copied(&'a [u16], HtmlConstructionBankGrowth, usize),
    Shared(DocumentString),
}

/// The attributes an element creation gives its element: their interned names, which the creation
/// found before it admitted anything, and the validated attributes whose values it copies.
#[derive(Clone, Copy)]
pub(super) struct Authored<'a> {
    entries: &'a [Arc<NameEntry>],
    attributes: Option<&'a OrderedAttributes>,
}
impl<'a> Authored<'a> {
    pub(super) const NONE: Authored<'static> = Authored {
        entries: &[],
        attributes: None,
    };
    fn new(entries: &'a [Arc<NameEntry>], attributes: &'a OrderedAttributes) -> Self {
        Self {
            entries,
            attributes: Some(attributes),
        }
    }
    fn len(&self) -> usize {
        self.entries.len()
    }
    /// The interned name and the value units of each attribute, in order.
    fn iter(&self) -> impl Iterator<Item = (&'a Arc<NameEntry>, &'a [u16])> + use<'a> {
        let entries = self.entries;
        self.attributes
            .into_iter()
            .flat_map(|attributes| attributes.iter())
            .zip(entries)
            .map(|(attribute, entry)| (entry, attribute.value().code_units()))
    }
}

/// The values of a record a creation makes, before it is stored. A processing instruction or a
/// doctype holds its strings as `S`, so that each stage of the creation (the units given, the
/// capacity admitted for each, the string built) is a mapping of the same shape, and no stage
/// looks a string up by position. A text or a comment holds its units as `L` in every stage: they
/// are written to the text arena of the storage, which is one bank for all of them.
enum Created<S, L> {
    Document(HtmlDocumentMode),
    Fragment,
    ShadowContents(HtmlShadowRootProperties),
    Element {
        name: Arc<NameEntry>,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
        template: bool,
    },
    Text(L),
    Comment(L),
    ProcessingInstruction(S, S),
    Doctype(S, Option<S>, Option<S>),
}

impl<S, L> Created<S, L> {
    /// The strings of a processing instruction or a doctype, in the order its record holds them.
    fn strings(&self) -> [Option<&S>; 3] {
        match self {
            Self::ProcessingInstruction(target, data) => [Some(target), Some(data), None],
            Self::Doctype(name, public, system) => [Some(name), public.as_ref(), system.as_ref()],
            _ => [None; 3],
        }
    }
    /// The units of a text or a comment.
    fn leaf(&self) -> Option<&L> {
        match self {
            Self::Text(units) | Self::Comment(units) => Some(units),
            _ => None,
        }
    }
    fn template(&self) -> bool {
        matches!(self, Self::Element { template: true, .. })
    }
    /// The same creation with each string mapped, in the order the record holds them. The first
    /// refusal ends the mapping.
    fn try_map<T, E>(self, mut map: impl FnMut(S) -> Result<T, E>) -> Result<Created<T, L>, E> {
        Ok(match self {
            Self::Document(mode) => Created::Document(mode),
            Self::Fragment => Created::Fragment,
            Self::ShadowContents(properties) => Created::ShadowContents(properties),
            Self::Element {
                name,
                link_flags,
                template,
            } => Created::Element {
                name,
                link_flags,
                template,
            },
            Self::Text(units) => Created::Text(units),
            Self::Comment(units) => Created::Comment(units),
            Self::ProcessingInstruction(target, data) => {
                let target = map(target)?;
                Created::ProcessingInstruction(target, map(data)?)
            }
            Self::Doctype(name, public, system) => {
                let name = map(name)?;
                let public = public.map(&mut map).transpose()?;
                Created::Doctype(name, public, system.map(&mut map).transpose()?)
            }
        })
    }
}

impl HtmlConstructionStorage {
    /// The growth the text arena needs to hold `added` more units, or `None` when it has room for
    /// them. The arena is one bank for every text and comment of the storage.
    fn text_growth(
        &self,
        added: usize,
    ) -> Result<Option<HtmlConstructionBankGrowth>, HtmlConstructionError> {
        let required = plus(self.text.len(), added)?;
        Ok(
            (required > self.text.capacity()).then_some(HtmlConstructionBankGrowth {
                bank: HtmlConstructionBank::TextArena,
                before: HtmlConstructionBankCapacity {
                    initialized_len: self.text.len(),
                    capacity: self.text.capacity(),
                },
                minimum_capacity: required,
                element_size: size_of::<u16>(),
                wrapper_bytes: 0,
            }),
        )
    }

    /// The growths of the runs and the rows that `plan` writes, each when its bank has no room.
    fn attribute_row_growths(
        &self,
        plan: RowAppend,
    ) -> Result<
        (
            Option<HtmlConstructionBankGrowth>,
            Option<HtmlConstructionBankGrowth>,
        ),
        HtmlConstructionError,
    > {
        let table = &self.attrs;
        let rows_required = plus(table.rows_len(), plan.written())?;
        let runs_required = plus(table.runs_len(), plan.runs())?;
        let rows = (rows_required > table.rows_capacity()).then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::AttributeRows,
            before: HtmlConstructionBankCapacity {
                initialized_len: table.rows_len(),
                capacity: table.rows_capacity(),
            },
            minimum_capacity: rows_required,
            element_size: size_of::<AttrRow>(),
            wrapper_bytes: 0,
        });
        let runs = (runs_required > table.runs_capacity()).then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::AttributeRuns,
            before: HtmlConstructionBankCapacity {
                initialized_len: table.runs_len(),
                capacity: table.runs_capacity(),
            },
            minimum_capacity: runs_required,
            element_size: size_of::<AttrRun>(),
            wrapper_bytes: 0,
        });
        Ok((runs, rows))
    }

    /// Admits and grows the banks an attribute edit needs: the runs and the rows that `plan`
    /// writes and the text arena for `text_added` units. Every growth is admitted before the
    /// first is recorded, and each is recorded before it is reserved, as every bank growth is.
    pub(super) fn grow_attribute_banks<A: HtmlRecordAdmission>(
        &mut self,
        plan: RowAppend,
        text_added: usize,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let (runs, rows) = self.attribute_row_growths(plan).map_err(faulted)?;
        let text = self.text_growth(text_added).map_err(faulted)?;
        let growths = plus(
            plus(usize::from(rows.is_some()), usize::from(runs.is_some())).map_err(faulted)?,
            usize::from(text.is_some()),
        )
        .map_err(faulted)?;
        for growth in [runs, rows, text].into_iter().flatten() {
            admission
                .record_largest(
                    HtmlResource::AtomicLayoutBytes,
                    wide(times(growth.minimum_capacity, growth.element_size).map_err(faulted)?),
                )
                .map_err(stopped)?;
        }
        Work {
            provider_calls: growths,
            ..Work::default()
        }
        .charge(admission)?;
        let chosen_runs = runs
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let chosen_rows = rows
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let chosen_text = text
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        if let (Some(growth), Some(chosen)) = (runs, chosen_runs) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_attribute_runs(additional)
            })?;
        }
        if let (Some(growth), Some(chosen)) = (rows, chosen_rows) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_attribute_rows(additional)
            })?;
        }
        if let (Some(growth), Some(chosen)) = (text, chosen_text) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_text_bank(additional)
            })?;
        }
        Ok(())
    }

    fn reserve_attribute_rows(&mut self, additional: usize) -> Reserved {
        let ok = self.attrs.reserve_rows(additional).is_ok();
        Reserved {
            ok,
            reached: HtmlConstructionBankCapacity {
                initialized_len: self.attrs.rows_len(),
                capacity: self.attrs.rows_capacity(),
            },
        }
    }

    fn reserve_attribute_runs(&mut self, additional: usize) -> Reserved {
        let ok = self.attrs.reserve_runs(additional).is_ok();
        Reserved {
            ok,
            reached: HtmlConstructionBankCapacity {
                initialized_len: self.attrs.runs_len(),
                capacity: self.attrs.runs_capacity(),
            },
        }
    }

    fn reserve_text_bank(&mut self, additional: usize) -> Reserved {
        let ok = self.text.reserve_exact(additional).is_ok();
        Reserved {
            ok,
            reached: HtmlConstructionBankCapacity {
                initialized_len: self.text.len(),
                capacity: self.text.capacity(),
            },
        }
    }

    fn create<A: HtmlRecordAdmission>(
        &mut self,
        created: Created<Units<'_>, &[u16]>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create_with(created, Authored::NONE, admission)
    }

    /// Creates the record `created` names. An element takes the attributes `authored` gives it
    /// in the same operation: the whole creation is one admitted change, every bank it needs is
    /// reserved before that change is recorded, and the writes after it allocate nothing and
    /// admit nothing.
    fn create_with<A: HtmlRecordAdmission>(
        &mut self,
        created: Created<Units<'_>, &[u16]>,
        authored: Authored<'_>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        let before = self.arena.usage;
        let template = created.template();
        let count = 1 + usize::from(template);
        let len = self.arena.records.len();
        let required = plus(len, count).map_err(faulted)?;
        u32::try_from(required)
            .map_err(|_| fault(HtmlConstructionCause::RepresentationExhausted))?;
        let first = NodeIx::new(len)
            .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        let contents_ix = if template {
            Some(
                first
                    .offset(1)
                    .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?,
            )
        } else {
            None
        };
        // The values the record counts.
        let strings = created.strings();
        let leaf = created.leaf().copied();
        let units = match &created {
            Created::Element { name, .. } => name_units(name.name()).map_err(faulted)?,
            _ => {
                let mut units = leaf.map_or(0, <[u16]>::len);
                for string in strings.iter().flatten() {
                    units = plus(units, string.len()).map_err(faulted)?;
                }
                units
            }
        };
        for string in strings.iter().flatten() {
            times(string.len(), size_of::<u16>())
                .ok()
                .filter(|bytes| *bytes <= isize::MAX as usize)
                .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        }
        // The arena holds the units of a text or a comment, and a range of it is a handle of the
        // block it becomes, so it holds no more than a block holds. The values of the attributes
        // of an element are written to it as well.
        let mut units = units;
        let mut value_units = 0usize;
        for (entry, value) in authored.iter() {
            units = plus(
                units,
                attribute_units(entry.name(), value.len()).map_err(faulted)?,
            )
            .map_err(faulted)?;
            value_units = plus(value_units, value.len()).map_err(faulted)?;
        }
        let leaf_units = leaf.map_or(0, <[u16]>::len);
        let text_units = plus(leaf_units, value_units).map_err(faulted)?;
        self.text.check_add(text_units).map_err(faulted)?;
        let attribute_plan = if authored.len() == 0 {
            None
        } else {
            Some(self.attrs.plan(None, authored.len()).map_err(faulted)?)
        };
        let mut after = before;
        after.nodes = plus(after.nodes, count).map_err(faulted)?;
        after.edges = plus(after.edges, usize::from(template)).map_err(faulted)?;
        after.attributes = plus(after.attributes, authored.len()).map_err(faulted)?;
        after.utf16_units = plus(after.utf16_units, units).map_err(faulted)?;
        after.depth = after.depth.max(usize::from(template));
        let change = HtmlConstructionChange { before, after };
        let records_growth = self.records_growth(required);
        let text_growth = self.text_growth(text_units).map_err(faulted)?;
        let (runs_growth, rows_growth) = match attribute_plan {
            Some(plan) => self.attribute_row_growths(plan).map_err(faulted)?,
            None => (None, None),
        };
        // Only a string that is copied is a bank the creation grows.
        let string_count = strings
            .iter()
            .flatten()
            .filter(|string| matches!(string, Units::Copied(_)))
            .count();
        let growths = plus(
            plus(
                plus(
                    usize::from(records_growth.is_some()),
                    usize::from(text_growth.is_some()),
                )
                .map_err(faulted)?,
                plus(
                    usize::from(runs_growth.is_some()),
                    usize::from(rows_growth.is_some()),
                )
                .map_err(faulted)?,
            )
            .map_err(faulted)?,
            string_count,
        )
        .map_err(faulted)?;
        admission
            .record_largest(
                HtmlResource::AtomicLayoutBytes,
                wide(times(count, size_of::<Record>()).map_err(faulted)?),
            )
            .map_err(stopped)?;
        for growth in [runs_growth, rows_growth].into_iter().flatten() {
            admission
                .record_largest(
                    HtmlResource::AtomicLayoutBytes,
                    wide(times(growth.minimum_capacity, growth.element_size).map_err(faulted)?),
                )
                .map_err(stopped)?;
        }
        admission
            .record_largest(HtmlResource::TreeDepth, wide(after.depth))
            .map_err(stopped)?;
        admission.admit_change(self, change).map_err(stopped)?;
        // Every write and copy is charged before any change is made: the rows, the usage and the
        // depth counts, then each run of leaf units.
        Work {
            provider_calls: growths,
            copied_bytes: plus(
                times(count, size_of::<Record>() + size_of::<usize>()).map_err(faulted)?,
                size_of::<HtmlConstructionUsage>(),
            )
            .map_err(faulted)?,
            record_writes: plus(times(count, 2).map_err(faulted)?, 1).map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        let copied = strings
            .iter()
            .flatten()
            .filter_map(|string| match string {
                Units::Copied(units) => Some(*units),
                Units::Shared(_) => None,
            })
            .chain(leaf);
        for units in copied {
            for chunk in units.chunks(CHUNK_UNITS) {
                Work {
                    copied_bytes: size_of_val(chunk),
                    record_writes: 1,
                    ..Work::default()
                }
                .charge(admission)?;
            }
        }
        // The rows the attributes are written to, then their values, and the run that names them.
        if let Some(plan) = attribute_plan {
            Self::charge_filled_rows(admission, plan)?;
            Work {
                record_visits: 1,
                copied_bytes: size_of::<AttrRun>(),
                record_writes: 2,
                ..Work::default()
            }
            .charge(admission)?;
            for (_, value) in authored.iter() {
                Self::charge_value(admission, value)?;
                Work {
                    record_visits: 2,
                    copied_bytes: size_of::<AttrRow>(),
                    record_writes: 3,
                    ..Work::default()
                }
                .charge(admission)?;
            }
        }
        // Every growth is admitted before the first is recorded.
        let records_chosen = records_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let text_chosen = text_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let runs_chosen = runs_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let rows_chosen = rows_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let admitted = created.try_map(|units| match units {
            Units::Shared(string) => {
                Ok::<_, HtmlConstructionRecordError<A::Stop>>(Admitted::Shared(string))
            }
            Units::Copied(units) => {
                let growth = HtmlConstructionBankGrowth {
                    bank: HtmlConstructionBank::Text,
                    before: HtmlConstructionBankCapacity::default(),
                    minimum_capacity: units.len(),
                    element_size: size_of::<u16>(),
                    wrapper_bytes: size_of::<Vec<u16>>(),
                };
                Ok(Admitted::Copied(
                    units,
                    growth,
                    Self::admit_bank(growth, admission)?,
                ))
            }
        })?;
        if let (Some(growth), Some(chosen)) = (records_growth, records_chosen) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_record_bank(additional)
            })?;
        }
        if let (Some(growth), Some(chosen)) = (text_growth, text_chosen) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_text_bank(additional)
            })?;
        }
        if let (Some(growth), Some(chosen)) = (runs_growth, runs_chosen) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_attribute_runs(additional)
            })?;
        }
        if let (Some(growth), Some(chosen)) = (rows_growth, rows_chosen) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_attribute_rows(additional)
            })?;
        }
        // Each string is built where it is reserved, so every allocation of a leaf, its units and
        // the shared handle that holds them, comes before the last step that can refuse. The
        // reservation goes through `grow` like any other, on a vector the storage does not hold
        // yet.
        let built = admitted.try_map(|admitted| {
            let (units, growth, chosen) = match admitted {
                Admitted::Shared(string) => {
                    return Ok::<_, HtmlConstructionRecordError<A::Stop>>(string);
                }
                Admitted::Copied(units, growth, chosen) => (units, growth, chosen),
            };
            let mut reserved = Vec::new();
            self.grow(growth, chosen, admission, |_, additional| {
                reserved = Vec::new();
                let ok = reserved.try_reserve_exact(additional).is_ok();
                Reserved {
                    ok,
                    reached: HtmlConstructionBankCapacity {
                        initialized_len: 0,
                        capacity: reserved.capacity(),
                    },
                }
            })?;
            reserved.extend_from_slice(units);
            Ok(DocumentString::from_utf16(reserved))
        })?;
        let fragment = contents_ix.map(|_| {
            let mut fragment = Record::new(NodeContent::Fragment, None, 1);
            fragment.owner = Some(HtmlFragmentOwner::Template(self.arena.id(first)));
            fragment
        });
        admission.record_change(self, change).map_err(stopped)?;
        // The records exist and the banks have room for them: the writes below allocate nothing.
        // The units of a text or a comment are written to the arena now, so that a refusal above
        // leaves the arena as it was.
        let (content, properties, document) = match built {
            Created::Document(mode) => (NodeContent::Document(mode), None, true),
            Created::Fragment => (NodeContent::Fragment, None, false),
            Created::ShadowContents(properties) => (NodeContent::Fragment, Some(properties), false),
            Created::Element {
                name, link_flags, ..
            } => (
                NodeContent::Element(ElementContent {
                    link_flags,
                    name,
                    attrs: None,
                }),
                None,
                false,
            ),
            Created::Text(units) => (
                NodeContent::Text(self.text.push(units).map_err(faulted)?),
                None,
                false,
            ),
            Created::Comment(units) => (
                NodeContent::Comment(self.text.push(units).map_err(faulted)?),
                None,
                false,
            ),
            Created::ProcessingInstruction(target, data) => (
                NodeContent::ProcessingInstruction(HtmlProcessingInstruction::new(target, data)),
                None,
                false,
            ),
            Created::Doctype(name, public, system) => (
                NodeContent::Doctype(HtmlDoctype::new(name, public, system)),
                None,
                false,
            ),
        };
        let mut record = Record::new(content, None, 0);
        if document {
            // This actual admitted constructor issues original state; generic raw kinds and later
            // mode edits cannot issue it.
            record.document_focused_area_designation = Some(HtmlFocusedAreaDesignation::KnownNull);
        }
        record.shadow_properties = properties;
        if let Some(contents) = contents_ix {
            record.template_contents = Some(TemplateContents::Owned(contents));
        }
        let serial = self.arena.serial;
        self.arena.records.push(record);
        self.arena.populate(0).map_err(faulted)?;
        if let Some(fragment) = fragment {
            self.arena.records.push(fragment);
            self.arena.populate(1).map_err(faulted)?;
        }
        if let Some(plan) = attribute_plan {
            // The banks have room for every row and every value: the writes allocate nothing.
            let run = self.attrs.make_room(None, plan).map_err(faulted)?;
            self.set_element_run(len, run).map_err(faulted)?;
            for (entry, value) in authored.iter() {
                let text = self.text.push(value).map_err(faulted)?.text();
                self.attrs
                    .push(run, AttrRow::new(entry.clone(), text))
                    .map_err(faulted)?;
            }
            self.arena.attribute_revision = self.arena.attribute_revision.wrapping_add(1);
        }
        self.arena.usage = after;
        Ok(HtmlBuilderNode {
            tree: serial,
            index: first,
        })
    }

    /// Creates a detached Document record in `mode`.
    pub fn create_document<A: HtmlRecordAdmission>(
        &mut self,
        mode: HtmlDocumentMode,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(Created::Document(mode), admission)
    }

    /// Creates a detached document-fragment record.
    pub fn create_fragment<A: HtmlRecordAdmission>(
        &mut self,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(Created::Fragment, admission)
    }

    /// Creates a detached fragment that will serve as declarative shadow contents with
    /// `properties`. It has no host or owner until attached.
    pub fn create_shadow_contents<A: HtmlRecordAdmission>(
        &mut self,
        properties: HtmlShadowRootProperties,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(Created::ShadowContents(properties), admission)
    }

    /// Creates a detached element with the name and the ordered attributes given. The name is
    /// interned first, as [`intern_name`](Self::intern_name) does, and is held by the storage
    /// once for every element that has it; a name the storage did not hold has the strings of
    /// `name` as its own. The names of the attributes are interned the same way, and the values
    /// are copied to the storage's text arena. An HTML `template` element also creates its owned
    /// contents fragment, which follows it among the records. The name is stored as given; the
    /// attributes were validated by their owner. Link construction flags may be given only for an
    /// element named `link` in the HTML namespace, or the creation is refused with the
    /// wrong-node-kind cause, after the name is interned and its work is charged. A name that a
    /// refused creation interned stays interned. The element and its attributes are one change: the
    /// admission is asked once, for the usage of the element and all its attributes, and every bank
    /// the creation needs is admitted and reserved before the change is recorded, so a refusal at
    /// any step leaves the storage as it was and records no usage. The bank growths recorded before
    /// a later refusal stay recorded, and the capacity of a bank follows the residue rule of the
    /// contracts document. The temporary vector that holds the interned names of the attributes is
    /// admitted as an atomic layout of its pointers before it is reserved, so a refusal of that
    /// layout reports it.
    pub fn create_element<A: HtmlRecordAdmission>(
        &mut self,
        name: &QualifiedName,
        attributes: OrderedAttributes,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        let (_, entry) = self.intern(NameKey::of(name), Some(name), admission)?;
        // The names of the attributes are interned before anything is admitted, as the name of the
        // element is, so that the creation is one change: the entries are held in a vector of
        // exactly their number, which is reserved here and dropped before the creation returns. Its
        // layout is admitted first, so that a refusal reports it.
        admission
            .record_largest(
                HtmlResource::AtomicLayoutBytes,
                wide(times(attributes.len(), size_of::<Arc<NameEntry>>()).map_err(faulted)?),
            )
            .map_err(stopped)?;
        // A test arms a refusal of the allocator for this reservation, counted as one request,
        // only when the vector is not empty and so reaches the allocator.
        #[cfg(test)]
        if !attributes.is_empty() && crate::internal::amortized::fault::refuses_next_reservation() {
            return Err(fault(HtmlConstructionCause::AllocationFailed));
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(attributes.len())
            .map_err(|_| fault(HtmlConstructionCause::AllocationFailed))?;
        for attribute in attributes.iter() {
            let (_, entry) = self.intern(
                NameKey::of(attribute.name()),
                Some(attribute.name()),
                admission,
            )?;
            entries.push(entry);
        }
        self.create_element_of(
            entry,
            link_flags,
            Authored::new(&entries, &attributes),
            admission,
        )
    }

    /// Creates a detached text record whose units are a copy of `units` in the storage's text arena.
    pub fn create_text<A: HtmlRecordAdmission>(
        &mut self,
        units: &[u16],
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(Created::Text(units), admission)
    }

    /// Creates a detached comment record whose units are a copy of `units` in the storage's text
    /// arena.
    pub fn create_comment<A: HtmlRecordAdmission>(
        &mut self,
        units: &[u16],
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(Created::Comment(units), admission)
    }

    /// Creates a detached processing-instruction record holding copies of `target` and `data`,
    /// stored without native authoring checks.
    pub fn create_processing_instruction<A: HtmlRecordAdmission>(
        &mut self,
        target: &[u16],
        data: &[u16],
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(
            Created::ProcessingInstruction(Units::Copied(target), Units::Copied(data)),
            admission,
        )
    }

    /// Creates a detached doctype record holding copies of `name` and of the identifiers that are
    /// present. An absent identifier stays absent, and an empty one is kept.
    pub fn create_doctype<A: HtmlRecordAdmission>(
        &mut self,
        name: &[u16],
        public_id: Option<&[u16]>,
        system_id: Option<&[u16]>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(
            Created::Doctype(
                Units::Copied(name),
                public_id.map(Units::Copied),
                system_id.map(Units::Copied),
            ),
            admission,
        )
    }

    /// Creates a detached processing-instruction record that holds the strings of `instruction`
    /// as they are.
    pub(in crate::internal::construction) fn create_shared_processing_instruction<
        A: HtmlRecordAdmission,
    >(
        &mut self,
        instruction: HtmlProcessingInstruction,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(
            Created::ProcessingInstruction(
                Units::Shared(instruction.target().clone()),
                Units::Shared(instruction.data().clone()),
            ),
            admission,
        )
    }

    /// Creates a detached doctype record that holds the strings of `doctype` as they are.
    pub(in crate::internal::construction) fn create_shared_doctype<A: HtmlRecordAdmission>(
        &mut self,
        doctype: HtmlDoctype,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(
            Created::Doctype(
                Units::Shared(doctype.name().clone()),
                doctype.public_id().cloned().map(Units::Shared),
                doctype.system_id().cloned().map(Units::Shared),
            ),
            admission,
        )
    }
}
