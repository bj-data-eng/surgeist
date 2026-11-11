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
//! (cursors and level counts) is not state and may be left written. The one allocation that
//! follows the last refusable step is the shared arena of a publication, which stable Rust gives
//! no fallible constructor: an allocator failure there aborts, as any infallible allocation does.
//!
//! The work of these operations is counted in the records they write, the bytes of rows and units
//! they copy, one provider call for every request to the allocator, and the records they visit. A
//! visit is charged for each record a qualification or a walk reads. An insertion of a fresh
//! childless node visits a number of records that does not depend on the fanout of its parent or
//! the depth of the tree.

use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use crate::internal::values::{DocumentString, OrderedAttributes, QualifiedName};
use surgeist_ledger::{Change, Charge};

mod association;
mod depth;
mod order;
mod publish;
#[cfg(test)]
mod tests;

pub use publish::HtmlConstructionPublishRefusal;

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
    /// The child order of the record at `position`.
    Children {
        /// The ordinal of the record whose child order grows.
        position: usize,
    },
    /// The units of one string of a leaf record that a creation makes. It has no capacity before
    /// the creation, and the consumer also holds the wrapper that shares the string.
    Text,
    /// The shared arena that publication creates.
    Wrapper,
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
/// refused.
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
    before: HtmlConstructionUsage,
    after: HtmlConstructionUsage,
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
    /// refusal of the second record leaves the bank with the capacity it reached.
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
        let layout = size_of::<Arena>();
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
            arena: Arena {
                identity: Arc::new(Identity),
                records: Vec::new(),
                usage: HtmlConstructionUsage::default(),
                records_generation: 0,
                wrapper_generation: 0,
                temporary_generation: 0,
                serial,
                attribute_revision: 0,
            },
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
    fn admit_bank<A: HtmlRecordAdmission>(
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
        mut reserve: impl FnMut(&mut Self, usize) -> Reserved,
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
        let needed = minus(growth.minimum_capacity, len).map_err(faulted)?;
        let spare = minus(chosen, len).map_err(faulted)?;
        let mut outcome = reserve(self, spare);
        if !outcome.ok && chosen > growth.minimum_capacity {
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
        let name_bytes = match &self.record(position).map_err(faulted)?.kind {
            NodeContent::Element(element) => {
                times(element.name().local_name().len(), size_of::<u16>()).map_err(faulted)?
            }
            _ => return Err(fault(HtmlConstructionCause::WrongNodeKind)),
        };
        Work {
            inspected_bytes: name_bytes,
            ..Work::default()
        }
        .charge(charge)?;
        let is_link = matches!(
            &self.record(position).map_err(faulted)?.kind,
            NodeContent::Element(element)
                if crate::vocabulary::KnownHtmlElement::from_name(element.name())
                    == Some(crate::vocabulary::KnownHtmlElement::Link)
        );
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

/// The values of a record a creation makes, before it is stored.
enum Created<'a> {
    Document(HtmlDocumentMode),
    Fragment,
    ShadowContents(HtmlShadowRootProperties),
    Element {
        name: &'a QualifiedName,
        attributes: OrderedAttributes,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
        template: bool,
    },
    Text(&'a [u16]),
    Comment(&'a [u16]),
    ProcessingInstruction(&'a [u16], &'a [u16]),
    Doctype(&'a [u16], Option<&'a [u16]>, Option<&'a [u16]>),
}

impl<'a> Created<'a> {
    /// The strings of a leaf, in the order its record holds them.
    fn strings(&self) -> [Option<&'a [u16]>; 3] {
        match *self {
            Self::Text(units) | Self::Comment(units) => [Some(units), None, None],
            Self::ProcessingInstruction(target, data) => [Some(target), Some(data), None],
            Self::Doctype(name, public, system) => [Some(name), public, system],
            _ => [None; 3],
        }
    }
    fn template(&self) -> bool {
        matches!(self, Self::Element { template: true, .. })
    }
}

impl HtmlConstructionStorage {
    fn create<A: HtmlRecordAdmission>(
        &mut self,
        created: Created<'_>,
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
        // The values the record counts, and the work of reading them: one visit for each attribute
        // of an element, charged before the attributes are read.
        let strings = created.strings();
        let attribute_count = match &created {
            Created::Element { attributes, .. } => attributes.len(),
            _ => 0,
        };
        Work {
            record_visits: attribute_count,
            ..Work::default()
        }
        .charge(admission)?;
        let units = match &created {
            Created::Element {
                name, attributes, ..
            } => {
                let mut units = operations::name_units(name).map_err(faulted)?;
                for attribute in attributes.iter() {
                    units = plus(
                        units,
                        operations::attribute_units(attribute).map_err(faulted)?,
                    )
                    .map_err(faulted)?;
                }
                units
            }
            _ => {
                let mut units = 0usize;
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
        let mut after = before;
        after.nodes = plus(after.nodes, count).map_err(faulted)?;
        after.edges = plus(after.edges, usize::from(template)).map_err(faulted)?;
        after.attributes = plus(after.attributes, attribute_count).map_err(faulted)?;
        after.utf16_units = plus(after.utf16_units, units).map_err(faulted)?;
        after.depth = after.depth.max(usize::from(template));
        let change = HtmlConstructionChange { before, after };
        let records_growth = self.records_growth(required);
        let string_count = strings.iter().flatten().count();
        let growths = plus(usize::from(records_growth.is_some()), string_count).map_err(faulted)?;
        admission
            .record_largest(
                HtmlResource::AtomicLayoutBytes,
                wide(times(count, size_of::<Record>()).map_err(faulted)?),
            )
            .map_err(stopped)?;
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
        for string in strings.iter().flatten() {
            for chunk in string.chunks(CHUNK_UNITS) {
                Work {
                    copied_bytes: size_of_val(chunk),
                    record_writes: 1,
                    ..Work::default()
                }
                .charge(admission)?;
            }
        }
        // Every growth is admitted before the first is recorded.
        let records_chosen = records_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let mut text_chosen = [None; 3];
        for (slot, string) in text_chosen.iter_mut().zip(strings) {
            if let Some(string) = string {
                let growth = HtmlConstructionBankGrowth {
                    bank: HtmlConstructionBank::Text,
                    before: HtmlConstructionBankCapacity::default(),
                    minimum_capacity: string.len(),
                    element_size: size_of::<u16>(),
                    wrapper_bytes: size_of::<Vec<u16>>(),
                };
                *slot = Some((growth, Self::admit_bank(growth, admission)?));
            }
        }
        if let (Some(growth), Some(chosen)) = (records_growth, records_chosen) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_record_bank(additional)
            })?;
        }
        // Each string is built where it is reserved, so every allocation of a leaf, its units and
        // the shared handle that holds them, comes before the last step that can refuse.
        let mut built: [Option<DocumentString>; 3] = [None, None, None];
        for ((slot, made), string) in text_chosen.iter().zip(built.iter_mut()).zip(strings) {
            let (Some((growth, chosen)), Some(units)) = (*slot, string) else {
                continue;
            };
            let mut reserved = Vec::new();
            // The reservation goes through `grow` like any other, on a vector the storage does not
            // hold yet.
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
            *made = Some(DocumentString::from_utf16(reserved));
        }
        let take = |built: &mut [Option<DocumentString>; 3], index: usize| {
            built
                .get_mut(index)
                .and_then(Option::take)
                .ok_or(HtmlConstructionCause::OperationNotReady.error())
        };
        let (content, properties, document) = match created {
            Created::Document(mode) => (NodeContent::Document(mode), None, true),
            Created::Fragment => (NodeContent::Fragment, None, false),
            Created::ShadowContents(properties) => (NodeContent::Fragment, Some(properties), false),
            Created::Element {
                name,
                attributes,
                link_flags,
                ..
            } => (
                NodeContent::Element(ElementContent {
                    link_flags,
                    name: name.clone(),
                    attributes,
                }),
                None,
                false,
            ),
            Created::Text(_) => (
                NodeContent::Text(take(&mut built, 0).map_err(faulted)?),
                None,
                false,
            ),
            Created::Comment(_) => (
                NodeContent::Comment(take(&mut built, 0).map_err(faulted)?),
                None,
                false,
            ),
            Created::ProcessingInstruction(..) => (
                NodeContent::ProcessingInstruction(HtmlProcessingInstruction::new(
                    take(&mut built, 0).map_err(faulted)?,
                    take(&mut built, 1).map_err(faulted)?,
                )),
                None,
                false,
            ),
            Created::Doctype(..) => (
                NodeContent::Doctype(HtmlDoctype::new(
                    take(&mut built, 0).map_err(faulted)?,
                    strings[1]
                        .map(|_| take(&mut built, 1))
                        .transpose()
                        .map_err(faulted)?,
                    strings[2]
                        .map(|_| take(&mut built, 2))
                        .transpose()
                        .map_err(faulted)?,
                )),
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
        let contents_record = contents_ix.map(|_| {
            let mut fragment = Record::new(NodeContent::Fragment, None, 1);
            fragment.owner = Some(HtmlFragmentOwner::Template(self.arena.id(first)));
            fragment
        });
        admission.record_change(self, change).map_err(stopped)?;
        // The records exist and the bank has room for them: the pushes below allocate nothing.
        let arena = &mut self.arena;
        arena.records.push(record);
        arena.populate(0);
        if let Some(fragment) = contents_record {
            arena.records.push(fragment);
            arena.populate(1);
        }
        arena.usage = after;
        Ok(HtmlBuilderNode {
            tree: arena.serial,
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

    /// Creates a detached element with the name and the ordered attributes given. An HTML
    /// `template` element also creates its owned contents fragment, which follows it among the
    /// records. The name is stored as given; the attributes were validated by their owner. Link
    /// construction flags may be given only for an element named `link` in the HTML namespace,
    /// or the creation is refused with the wrong-node-kind cause.
    pub fn create_element<A: HtmlRecordAdmission>(
        &mut self,
        name: &QualifiedName,
        attributes: OrderedAttributes,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        Work {
            inspected_bytes: times(name.local_name().len(), size_of::<u16>()).map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        let known = crate::vocabulary::KnownHtmlElement::from_name(name);
        let template = known == Some(crate::vocabulary::KnownHtmlElement::Template);
        if link_flags.is_some() && known != Some(crate::vocabulary::KnownHtmlElement::Link) {
            return Err(fault(HtmlConstructionCause::WrongNodeKind));
        }
        self.create(
            Created::Element {
                name,
                attributes,
                link_flags,
                template,
            },
            admission,
        )
    }

    /// Creates a detached text record holding a copy of `units`.
    pub fn create_text<A: HtmlRecordAdmission>(
        &mut self,
        units: &[u16],
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        self.create(Created::Text(units), admission)
    }

    /// Creates a detached comment record holding a copy of `units`.
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
        self.create(Created::ProcessingInstruction(target, data), admission)
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
        self.create(Created::Doctype(name, public_id, system_id), admission)
    }
}
