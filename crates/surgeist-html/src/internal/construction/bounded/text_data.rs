//! Appends to the data of a text record of the construction storage.
//!
//! An append is one method of [`HtmlConstructionStorage`]. It qualifies the record, charges its
//! consumer for the work of the copy, and asks the consumer once to admit the append. Only then
//! does it change the text: it reserves room, records the units, and writes them. A refusal at
//! any step before the write leaves the text, its length and the usage of the storage as they
//! were. Two residues remain. When the allocator reserves more room than was admitted, or
//! refuses the admitted room and the fallback reserves the room that is needed, and the consumer
//! then refuses to record the capacity the text reached, the text keeps the capacity it reached
//! while the consumer holds the capacity it recorded before. When the consumer refuses to record
//! the units after the room is reserved, the text keeps the room it reserved. Neither changes a
//! unit of the text.
//!
//! The consumer owns the policy: it charges work through [`Charge`], decides the capacity a
//! growing text reserves, and records the capacity and the units.

use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use surgeist_ledger::{Change, Charge};

/// The units one charge covers when an append copies a run of units, so that a consumer that
/// polls at its charges polls within every quantum of the work.
const CHUNK_UNITS: usize = 256 / size_of::<u16>();

/// Actual dimensions of the data of a text record; no allocation or release authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionTextCapacity {
    initialized_len: usize,
    capacity: usize,
}
impl HtmlConstructionTextCapacity {
    /// The number of UTF-16 units the text holds.
    pub fn initialized_len(self) -> usize {
        self.initialized_len
    }
    /// The reserved capacity in units; at least [`initialized_len`](Self::initialized_len).
    pub fn capacity(self) -> usize {
        self.capacity
    }
}

/// What an append asks its consumer to admit before it changes anything: `added` more units on
/// a text that must then hold at least [`minimum_capacity`](Self::minimum_capacity) units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionTextGrowth {
    before: HtmlConstructionTextCapacity,
    added: usize,
    minimum_capacity: usize,
}
impl HtmlConstructionTextGrowth {
    /// The dimensions of the text before the append.
    pub fn before(self) -> HtmlConstructionTextCapacity {
        self.before
    }
    /// The number of units the append adds.
    pub fn added(self) -> usize {
        self.added
    }
    /// The least capacity that holds the text after the append.
    pub fn minimum_capacity(self) -> usize {
        self.minimum_capacity
    }
    /// Whether the text must reserve room: its capacity is below the minimum. An append that
    /// does not grow the text reserves nothing and records no capacity.
    pub fn grows(self) -> bool {
        self.minimum_capacity > self.before.capacity
    }
}

/// The consumer of a text append. It charges the append's work like any [`Charge`], admits the
/// append, decides the capacity that a growing text reserves, and records the capacity and the
/// units.
pub trait HtmlTextAdmission: Charge<HtmlWork, HtmlResource> {
    /// Admits one append before anything changes. Returns the capacity to reserve when the
    /// growth grows the text, which must be at least its minimum; a smaller one is refused with
    /// the capacity-mismatch cause and nothing is reserved. The value is ignored for an append
    /// that does not grow the text. Nothing is recorded or reserved before this call, so a
    /// refusal changes nothing.
    fn admit_append(&mut self, growth: HtmlConstructionTextGrowth) -> Result<usize, Self::Stop>;

    /// Records the capacity of the text. It is called with the capacity the consumer chose
    /// before the room is reserved, so a refusal there leaves the text as it was. It is called
    /// again with the capacity the reservation reached when that differs from the one chosen,
    /// which includes the unchanged capacity of a reservation the allocator refused. A refusal
    /// of that second record leaves the text with the capacity it reached and no unit added.
    fn record_growth(&mut self, reached: HtmlConstructionTextCapacity) -> Result<(), Self::Stop>;

    /// Records the units the append adds, after the room for them is reserved and before the
    /// first is written. A refusal leaves the text without the units and with the room it
    /// reserved.
    fn record_units(&mut self, added: usize) -> Result<(), Self::Stop>;
}

/// A refusal of a text append: the consumer's stop, or the append's own fault.
pub type HtmlConstructionTextError<S> = HtmlChargedError<S, HtmlConstructionError>;

fn fault<S>(cause: HtmlConstructionCause) -> HtmlConstructionTextError<S> {
    HtmlChargedError::Faulted(cause.error())
}
fn plus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_add(b)
        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())
}
fn minus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_sub(b)
        .ok_or(HtmlConstructionCause::ReleaseUnderflow.error())
}

/// The work of one batch of the append.
#[derive(Clone, Copy, Debug, Default)]
struct Work {
    record_visits: usize,
    provider_calls: usize,
    copied_bytes: usize,
    record_writes: usize,
}
impl Work {
    fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionTextError<C::Stop>> {
        charge
            .charge(&[
                Change::Work(HtmlWork::RecordVisit, wide(self.record_visits)),
                Change::Work(HtmlWork::ProviderCall, wide(self.provider_calls)),
                Change::Work(HtmlWork::CopyByte, wide(self.copied_bytes)),
                Change::Work(HtmlWork::RecordWrite, wide(self.record_writes)),
            ])
            .map_err(HtmlChargedError::Stopped)
    }
}

impl HtmlConstructionStorage {
    /// The position of `node` when it is a text record.
    fn text_target(&self, node: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        let position = self.position(node)?;
        match self.arena.records.get(position).map(|record| &record.kind) {
            Some(NodeContent::Text(_)) => Ok(position),
            _ => Err(HtmlConstructionCause::WrongNodeKind.into()),
        }
    }

    /// The units of the text at `target` for a change. Refused with the shared-allocation cause
    /// when another string holds the text's block, so that the units never change under it.
    fn text_units_mut(&mut self, target: usize) -> Result<&mut Vec<u16>, HtmlConstructionError> {
        match self
            .arena
            .records
            .get_mut(target)
            .map(|record| &mut record.kind)
        {
            Some(NodeContent::Text(value)) => value
                .unique_units_mut()
                .ok_or_else(|| HtmlConstructionCause::SharedAllocationAliased.into()),
            _ => Err(HtmlConstructionCause::WrongNodeKind.into()),
        }
    }

    /// Appends `units` to the data of the text record `node` and returns the length the text
    /// had, which is the offset of the first appended unit. The block of the text must have no
    /// other holder, or the append is refused with the shared-allocation cause.
    ///
    /// The append charges the work of its copy, admits itself once through `admission`, reserves
    /// the room the admission chose when the text is full, records the units, and writes them.
    /// The text grows in place, to the capacity the consumer chooses, so a consumer that at
    /// least doubles the capacity makes the units copied by `n` appends of `m` units in all
    /// linear in `n * m`. Appending no unit changes nothing.
    pub fn append_text<A: HtmlTextAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        units: &[u16],
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionTextError<A::Stop>> {
        Work {
            record_visits: 3,
            provider_calls: 1,
            ..Work::default()
        }
        .charge(admission)?;
        let target = self.text_target(node).map_err(HtmlChargedError::Faulted)?;
        let added = units.len();
        let before = {
            let vector = self
                .text_units_mut(target)
                .map_err(HtmlChargedError::Faulted)?;
            HtmlConstructionTextCapacity {
                initialized_len: vector.len(),
                capacity: vector.capacity(),
            }
        };
        if added == 0 {
            return Ok(before.initialized_len);
        }
        let grown = plus(before.initialized_len, added).map_err(HtmlChargedError::Faulted)?;
        let mut usage = self.arena.usage;
        usage.utf16_units = plus(usage.utf16_units, added).map_err(HtmlChargedError::Faulted)?;
        grown
            .checked_mul(size_of::<u16>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        // Every copy is charged before any change is made.
        for chunk in units.chunks(CHUNK_UNITS) {
            Work {
                copied_bytes: size_of_val(chunk),
                record_writes: 1,
                ..Work::default()
            }
            .charge(admission)?;
        }
        Work {
            record_visits: 2,
            copied_bytes: size_of_val(&usage),
            record_writes: 2,
            ..Work::default()
        }
        .charge(admission)?;
        let growth = HtmlConstructionTextGrowth {
            before,
            added,
            minimum_capacity: grown,
        };
        let capacity = admission
            .admit_append(growth)
            .map_err(HtmlChargedError::Stopped)?;
        if growth.grows() {
            self.grow_text(target, growth, capacity, admission)?;
        }
        admission
            .record_units(added)
            .map_err(HtmlChargedError::Stopped)?;
        self.text_units_mut(target)
            .map_err(HtmlChargedError::Faulted)?
            .extend_from_slice(units);
        self.arena.usage = usage;
        Ok(before.initialized_len)
    }

    /// Reserves the room of a growth of the text at `target`: records the capacity the consumer
    /// chose, reserves it, and records the capacity reached.
    fn grow_text<A: HtmlTextAdmission>(
        &mut self,
        target: usize,
        growth: HtmlConstructionTextGrowth,
        capacity: usize,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionTextError<A::Stop>> {
        let (before, minimum) = (growth.before, growth.minimum_capacity);
        if capacity < minimum {
            return Err(fault(HtmlConstructionCause::CapacityMismatch));
        }
        // The consumer records the capacity before anything is reserved, so that its refusal
        // leaves the text as it was.
        admission
            .record_growth(HtmlConstructionTextCapacity {
                initialized_len: before.initialized_len,
                capacity,
            })
            .map_err(HtmlChargedError::Stopped)?;
        let reserve = |storage: &mut Self, additional: usize| {
            storage.text_units_mut(target).map(|vector| {
                let result = vector.try_reserve_exact(additional);
                (result, vector.capacity())
            })
        };
        let needed = minus(minimum, before.initialized_len).map_err(HtmlChargedError::Faulted)?;
        let spare = minus(capacity, before.initialized_len).map_err(HtmlChargedError::Faulted)?;
        let (mut result, mut reached) = reserve(self, spare).map_err(HtmlChargedError::Faulted)?;
        // Only an allocator refusal of the spare room falls back to the room that is needed.
        if result.is_err() && capacity > minimum {
            (result, reached) = reserve(self, needed).map_err(HtmlChargedError::Faulted)?;
        }
        // An overshoot, or the unchanged capacity of a reservation the allocator refused, takes
        // the place of the capacity recorded above.
        if reached != capacity {
            admission
                .record_growth(HtmlConstructionTextCapacity {
                    initialized_len: before.initialized_len,
                    capacity: reached,
                })
                .map_err(HtmlChargedError::Stopped)?;
        }
        result.map_err(|_| fault(HtmlConstructionCause::AllocationFailed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::values::DocumentString;

    /// The refusal of a [`Meter`].
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Refused;

    /// A consumer that records what it is asked, and refuses at a chosen charge, record or
    /// admission. It chooses double the capacity, as a consumer that makes appends linear does.
    #[derive(Default)]
    struct Meter {
        charges: usize,
        refuse_from_charge: Option<usize>,
        refuse_admission: bool,
        /// Refuses the record of a growth whose place among the records is at least this.
        refuse_record_from: Option<usize>,
        refuse_units: bool,
        /// The capacity to choose, instead of double the capacity.
        choose: Option<usize>,
        work: u64,
        copied: u64,
        reserved: u64,
        growths: Vec<HtmlConstructionTextGrowth>,
        recorded: Vec<HtmlConstructionTextCapacity>,
        units: usize,
    }
    impl Charge<HtmlWork, HtmlResource> for Meter {
        type Stop = Refused;
        fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
            self.charges += 1;
            if self
                .refuse_from_charge
                .is_some_and(|from| self.charges > from)
            {
                return Err(Refused);
            }
            for change in batch {
                if let Change::Work(work, amount) = *change {
                    self.work += amount;
                    if work == HtmlWork::CopyByte {
                        self.copied += amount;
                    }
                }
            }
            Ok(())
        }
        fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
            Ok(())
        }
    }
    impl HtmlTextAdmission for Meter {
        fn admit_append(&mut self, growth: HtmlConstructionTextGrowth) -> Result<usize, Refused> {
            self.growths.push(growth);
            if self.refuse_admission {
                return Err(Refused);
            }
            let chosen = self
                .choose
                .unwrap_or_else(|| growth.before().capacity.saturating_mul(2))
                .max(growth.minimum_capacity());
            if growth.grows() {
                // The overlap of the old and the new layout, which a consumer charges as reserved
                // bytes.
                self.reserved += ((growth.before().capacity + chosen) * 2) as u64;
            }
            Ok(chosen)
        }
        fn record_growth(&mut self, reached: HtmlConstructionTextCapacity) -> Result<(), Refused> {
            if self
                .refuse_record_from
                .is_some_and(|from| self.recorded.len() >= from)
            {
                return Err(Refused);
            }
            self.recorded.push(reached);
            Ok(())
        }
        fn record_units(&mut self, added: usize) -> Result<(), Refused> {
            if self.refuse_units {
                return Err(Refused);
            }
            self.units += added;
            Ok(())
        }
    }

    fn make_storage() -> HtmlConstructionStorage {
        super::super::free::storage()
    }
    fn text_node(text: DocumentString) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut storage = make_storage();
        let node = super::super::free::text(&mut storage, text.code_units());
        (storage, node)
    }
    fn units(storage: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> Vec<u16> {
        match &storage.arena.records[storage.position(node).unwrap()].kind {
            NodeContent::Text(value) => value.code_units().to_vec(),
            _ => panic!("a text record"),
        }
    }
    fn capacity(storage: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> usize {
        match &storage.arena.records[storage.position(node).unwrap()].kind {
            NodeContent::Text(value) => value.retained_units(),
            _ => panic!("a text record"),
        }
    }
    fn cause(error: HtmlConstructionTextError<Refused>) -> HtmlConstructionCause {
        match error {
            HtmlChargedError::Faulted(error) => error.cause(),
            HtmlChargedError::Stopped(Refused) => panic!("the consumer refused"),
        }
    }
    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn an_append_adds_the_units_to_the_end_and_counts_them() {
        let (mut storage, node) = text_node("ab".into());
        let mut meter = Meter::default();
        let offset = storage
            .append_text(&node, &utf16("cd\u{1f642}"), &mut meter)
            .unwrap();
        assert_eq!(offset, 2);
        assert_eq!(units(&storage, &node), utf16("abcd\u{1f642}"));
        assert_eq!(storage.usage().utf16_units(), 6);
        assert_eq!(meter.units, 4);
        let offset = storage.append_text(&node, &utf16("e"), &mut meter).unwrap();
        assert_eq!(offset, 6);
        assert_eq!(units(&storage, &node), utf16("abcd\u{1f642}e"));
        assert_eq!(storage.usage().utf16_units(), 7);
    }

    #[test]
    fn an_append_keeps_lone_surrogates_and_nul_units_exactly() {
        let (mut storage, node) = text_node(DocumentString::from_utf16(vec![0xd800]));
        let incoming = [0xdc00, 0, 0xd83d];
        storage
            .append_text(&node, &incoming, &mut Meter::default())
            .unwrap();
        assert_eq!(units(&storage, &node), [0xd800, 0xdc00, 0, 0xd83d]);
    }

    #[test]
    fn an_append_of_no_unit_changes_nothing_and_admits_nothing() {
        let (mut storage, node) = text_node("ab".into());
        let mut meter = Meter::default();
        assert_eq!(storage.append_text(&node, &[], &mut meter).unwrap(), 2);
        assert!(meter.growths.is_empty());
        assert_eq!(units(&storage, &node), utf16("ab"));
        assert_eq!(storage.usage().utf16_units(), 2);
    }

    #[test]
    fn a_full_text_grows_once_to_the_capacity_the_consumer_chose() {
        let (mut storage, node) = text_node("ab".into());
        let before = capacity(&storage, &node);
        let mut meter = Meter {
            choose: Some(before + 40),
            ..Meter::default()
        };
        storage
            .append_text(&node, &vec![0x61; before + 3], &mut meter)
            .unwrap();
        assert_eq!(meter.growths.len(), 1);
        let growth = meter.growths[0];
        assert_eq!(growth.added(), before + 3);
        assert_eq!(growth.before().initialized_len(), 2);
        assert_eq!(growth.before().capacity, before);
        assert_eq!(growth.minimum_capacity(), before + 5);
        assert!(growth.grows());
        // The text reserved at least the capacity chosen, and recorded what it reached.
        let reached = capacity(&storage, &node);
        assert!(reached >= before + 40);
        assert_eq!(
            meter.recorded.first().map(|c| c.capacity),
            Some(before + 40)
        );
        assert!(meter.recorded.iter().all(|c| c.initialized_len() == 2));
        assert_eq!(meter.recorded.last().map(|c| c.capacity), Some(reached));
    }

    #[test]
    fn a_text_with_room_reserves_and_records_no_capacity() {
        let (mut storage, node) = text_node("ab".into());
        // The first append fills the room the string was made with, and grows past it.
        let mut meter = Meter {
            choose: Some(100),
            ..Meter::default()
        };
        storage.append_text(&node, &[0x61; 40], &mut meter).unwrap();
        let room = capacity(&storage, &node);
        assert!(room >= 100);
        let mut meter = Meter::default();
        storage.append_text(&node, &utf16("c"), &mut meter).unwrap();
        assert!(!meter.growths[0].grows());
        assert!(meter.recorded.is_empty());
        assert_eq!(capacity(&storage, &node), room);
    }

    #[test]
    fn an_allocator_refusal_of_the_spare_room_falls_back_to_the_room_needed() {
        let (mut storage, node) = text_node("ab".into());
        // The spare room is as many units as one allocation can name, which no allocator gives.
        let impossible = isize::MAX as usize / size_of::<u16>();
        let mut meter = Meter {
            choose: Some(impossible),
            ..Meter::default()
        };
        let before = capacity(&storage, &node);
        storage
            .append_text(&node, &vec![0x61; before + 1], &mut meter)
            .unwrap();
        assert_eq!(units(&storage, &node).len(), before + 3);
        let reached = capacity(&storage, &node);
        assert!(reached >= before + 3 && reached < impossible);
        assert_eq!(
            meter
                .recorded
                .iter()
                .map(|c| c.capacity)
                .collect::<Vec<_>>(),
            [impossible, reached]
        );
    }

    #[test]
    fn a_refusal_of_the_second_record_keeps_the_room_reached_and_no_unit() {
        let (mut storage, node) = text_node("ab".into());
        let impossible = isize::MAX as usize / size_of::<u16>();
        let mut meter = Meter {
            choose: Some(impossible),
            refuse_record_from: Some(1),
            ..Meter::default()
        };
        let before = capacity(&storage, &node);
        let usage = storage.usage();
        let error = storage
            .append_text(&node, &vec![0x61; before + 1], &mut meter)
            .unwrap_err();
        assert_eq!(error, HtmlChargedError::Stopped(Refused));
        assert_eq!(units(&storage, &node), utf16("ab"));
        assert_eq!(storage.usage(), usage);
        assert!(capacity(&storage, &node) >= before + 3);
        assert_eq!(meter.units, 0);
    }

    #[test]
    fn a_refusal_before_the_room_is_reserved_changes_nothing() {
        for refuse in ["charge", "admission", "first record"] {
            let (mut storage, node) = text_node("ab".into());
            let before = capacity(&storage, &node);
            let usage = storage.usage();
            let mut meter = Meter::default();
            match refuse {
                "charge" => meter.refuse_from_charge = Some(2),
                "admission" => meter.refuse_admission = true,
                _ => meter.refuse_record_from = Some(0),
            }
            let error = storage
                .append_text(&node, &vec![0x61; before + 1], &mut meter)
                .unwrap_err();
            assert_eq!(error, HtmlChargedError::Stopped(Refused), "{refuse}");
            assert_eq!(units(&storage, &node), utf16("ab"), "{refuse}");
            assert_eq!(storage.usage(), usage, "{refuse}");
            assert_eq!(capacity(&storage, &node), before, "{refuse}");
            assert_eq!(meter.units, 0, "{refuse}");
        }
    }

    #[test]
    fn a_refusal_to_record_the_units_keeps_the_room_and_no_unit() {
        let (mut storage, node) = text_node("ab".into());
        let before = capacity(&storage, &node);
        let usage = storage.usage();
        let mut meter = Meter {
            refuse_units: true,
            ..Meter::default()
        };
        let error = storage
            .append_text(&node, &vec![0x61; before + 1], &mut meter)
            .unwrap_err();
        assert_eq!(error, HtmlChargedError::Stopped(Refused));
        assert_eq!(units(&storage, &node), utf16("ab"));
        assert_eq!(storage.usage(), usage);
        assert!(capacity(&storage, &node) > before);
        assert_eq!(
            meter.recorded.last().map(|c| c.capacity),
            Some(capacity(&storage, &node))
        );
    }

    #[test]
    fn a_work_cut_at_every_charge_of_an_append_leaves_the_text_or_the_whole_append() {
        let incoming = vec![0x62u16; 300];
        let (storage, node) = text_node("ab".into());
        let mut probe = storage;
        let mut counting = Meter::default();
        probe.append_text(&node, &incoming, &mut counting).unwrap();
        let total = counting.charges;
        assert!(total > 4, "the copy of 300 units is charged in quanta");
        for cut in 0..=total {
            let (mut storage, node) = text_node("ab".into());
            let before = capacity(&storage, &node);
            let usage = storage.usage();
            let mut meter = Meter {
                refuse_from_charge: Some(cut),
                ..Meter::default()
            };
            match storage.append_text(&node, &incoming, &mut meter) {
                Ok(offset) => {
                    assert!(cut >= total, "cut {cut} of {total} completed");
                    assert_eq!(offset, 2);
                    let mut expected = utf16("ab");
                    expected.extend_from_slice(&incoming);
                    assert_eq!(units(&storage, &node), expected);
                    assert_eq!(storage.usage().utf16_units(), 302);
                }
                Err(error) => {
                    assert_eq!(error, HtmlChargedError::Stopped(Refused), "cut {cut}");
                    assert_eq!(units(&storage, &node), utf16("ab"), "cut {cut}");
                    assert_eq!(storage.usage(), usage, "cut {cut}");
                    assert_eq!(capacity(&storage, &node), before, "cut {cut}");
                }
            }
        }
    }

    #[test]
    fn a_text_another_string_holds_is_refused_and_unchanged() {
        let (mut storage, node) = text_node("ab".into());
        // A text node made by the storage holds its own string, so another owner is given a
        // handle on it the way a reader of the record would be.
        let position = storage.position(&node).unwrap();
        let held = match &storage.arena.records[position].kind {
            NodeContent::Text(text) => text.clone(),
            _ => panic!("a text node"),
        };
        let usage = storage.usage();
        let error = storage
            .append_text(&node, &utf16("c"), &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::SharedAllocationAliased);
        assert_eq!(held.code_units(), utf16("ab"));
        assert_eq!(units(&storage, &node), utf16("ab"));
        assert_eq!(storage.usage(), usage);
    }

    #[test]
    fn a_node_that_is_not_text_is_refused() {
        let mut storage = make_storage();
        let node = super::super::free::comment(&mut storage, "ab");
        let error = storage
            .append_text(&node, &utf16("c"), &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::WrongNodeKind);
    }

    #[test]
    fn a_capacity_below_the_minimum_is_the_capacity_mismatch() {
        struct Low(Meter);
        impl Charge<HtmlWork, HtmlResource> for Low {
            type Stop = Refused;
            fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
                self.0.charge(batch)
            }
            fn record_largest(
                &mut self,
                resource: HtmlResource,
                value: u64,
            ) -> Result<(), Refused> {
                self.0.record_largest(resource, value)
            }
        }
        impl HtmlTextAdmission for Low {
            fn admit_append(
                &mut self,
                growth: HtmlConstructionTextGrowth,
            ) -> Result<usize, Refused> {
                Ok(growth.minimum_capacity() - 1)
            }
            fn record_growth(
                &mut self,
                reached: HtmlConstructionTextCapacity,
            ) -> Result<(), Refused> {
                self.0.record_growth(reached)
            }
            fn record_units(&mut self, added: usize) -> Result<(), Refused> {
                self.0.record_units(added)
            }
        }
        let (mut storage, node) = text_node("ab".into());
        let before = capacity(&storage, &node);
        let error = storage
            .append_text(&node, &vec![0x61; before + 1], &mut Low(Meter::default()))
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::CapacityMismatch);
        assert_eq!(units(&storage, &node), utf16("ab"));
        assert_eq!(capacity(&storage, &node), before);
    }

    /// The units an append copies and the bytes it reserves, which a consumer charges, for
    /// `appends` appends of `size` units to one text.
    fn cost_of(appends: usize, size: usize) -> u64 {
        let (mut storage, node) = text_node("".into());
        let mut meter = Meter::default();
        let incoming = vec![0x61u16; size];
        for _ in 0..appends {
            storage.append_text(&node, &incoming, &mut meter).unwrap();
        }
        assert_eq!(units(&storage, &node).len(), appends * size);
        meter.copied + meter.reserved
    }

    #[test]
    fn the_work_of_appends_is_linear_in_the_units_appended() {
        for size in [1, 7] {
            let costs: Vec<u64> = [1_000, 2_000, 4_000, 8_000]
                .iter()
                .map(|&appends| cost_of(appends, size))
                .collect();
            for pair in costs.windows(2) {
                // A doubling of the appends at most doubles the work, with a tenth to spare.
                assert!(
                    pair[1] * 10 <= pair[0] * 22,
                    "size {size}: {} then {}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    #[test]
    fn one_append_costs_the_same_whatever_length_the_text_already_has() {
        let mut works = Vec::new();
        for length in [16, 1_024, 16_384] {
            let (mut storage, node) = text_node(DocumentString::from_utf16(vec![0x61; length]));
            // Make room for the append, so that the work measured is the append's own.
            let mut meter = Meter {
                choose: Some(4 * length + 8),
                ..Meter::default()
            };
            storage
                .append_text(&node, &vec![0x62; length], &mut meter)
                .unwrap();
            let mut meter = Meter::default();
            storage.append_text(&node, &utf16("c"), &mut meter).unwrap();
            assert!(!meter.growths[0].grows());
            works.push(meter.work);
        }
        assert!(works.iter().all(|work| *work == works[0]), "{works:?}");
    }
}
