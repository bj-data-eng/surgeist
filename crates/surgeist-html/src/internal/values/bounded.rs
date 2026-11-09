//! Exact expanded-name validation without a controller or scratch allocation.

use super::attribute_name_index::{
    CLEAR_CHUNK, HtmlAttributeNameIndex, Occupant, entry, occupant, tag,
};
use super::finish::HtmlFinishFailure;
use super::name_index::{HtmlNameIndex, HtmlNameIndexProbe, HtmlNameIndexSlot, HtmlNameIndexStep};
use super::{Attribute, DocumentString, OrderedAttributes};
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork};
use std::{error::Error, fmt, mem::size_of, sync::Arc};
use surgeist_ledger::{Change, Charge};

/// The actual immutable attribute allocation awaiting expanded-name validation.
///
/// Retaining this owner requires no copy, reserve, normalization or validation.
/// Only a completed validation can publish it as [`OrderedAttributes`].
#[derive(Debug)]
pub struct HtmlAttributeStorage {
    attributes: Arc<Vec<Attribute>>,
}

impl HtmlAttributeStorage {
    /// Adopts the existing shared vector, including its actual spare capacity.
    pub fn from_shared_attributes(attributes: Arc<Vec<Attribute>>) -> Self {
        Self { attributes }
    }

    /// Borrows the retained immutable allocation.
    pub fn shared_attributes(&self) -> &Arc<Vec<Attribute>> {
        &self.attributes
    }

    /// Returns the retained backing without validating or publishing attributes.
    pub fn into_shared_attributes(self) -> Arc<Vec<Attribute>> {
        self.attributes
    }

    /// Begins duplicate-major, first-minor exact expanded-name validation.
    ///
    /// This compares every pair of names, which is quadratic in the attribute count; it suits a
    /// bounded count. [`validate_indexed`](Self::validate_indexed) is linear.
    pub fn validate(self) -> HtmlAttributeValidation {
        let stage = (self.attributes.len() >= 2).then_some(Stage::Pairs);
        HtmlAttributeValidation {
            storage: self,
            duplicate: 1,
            first: 0,
            phase: Phase::Pair,
            failure: None,
            indexed: None,
            stage,
            probe: None,
        }
    }

    /// Begins the same validation through `index`, so that a duplicate is found by probing the
    /// few names that share a hash. The work is linear in the attribute count and the total
    /// length of the names, instead of quadratic.
    ///
    /// The verdict is that of [`validate`](Self::validate): the first attribute, in order,
    /// whose expanded name repeats an earlier one, with the earlier attribute's index. The index
    /// must hold at least [`HtmlAttributeNameIndex::slots_required`] slots for the attribute
    /// count; with less, the cursor compares every pair as `validate` does. Take the index back
    /// with [`HtmlAttributeValidation::take_index`].
    pub fn validate_indexed(self, mut index: HtmlAttributeNameIndex) -> HtmlAttributeValidation {
        let count = self.attributes.len();
        let Some(geometry) = index.geometry_for(count).filter(|_| count >= 2) else {
            let mut validation = self.validate();
            validation.indexed = Some(index);
            return validation;
        };
        index.begin_clear(geometry);
        HtmlAttributeValidation {
            storage: self,
            duplicate: 0,
            first: 0,
            phase: Phase::Pair,
            failure: None,
            indexed: Some(index),
            stage: Some(Stage::Clear),
            probe: None,
        }
    }
}

/// The first duplicate expanded name, or a refused ownership transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlAttributeValidationError {
    /// The original cursor still has comparisons to perform.
    NotReady,
    /// Prefixes and values are excluded from this exact expanded-name identity.
    Duplicate {
        /// The index of the earlier attribute that owns the expanded name.
        first: usize,
        /// The index of the later attribute that repeats it.
        duplicate: usize,
    },
    /// The supplied index could not hold the names. A cursor that is given enough slots never
    /// reports it.
    IndexUnavailable,
}

impl fmt::Display for HtmlAttributeValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReady => formatter.write_str("attribute validation is incomplete"),
            Self::IndexUnavailable => formatter.write_str("attribute name index is unavailable"),
            Self::Duplicate { first, duplicate } => write!(
                formatter,
                "attribute at index {duplicate} duplicates the expanded name at index {first}"
            ),
        }
    }
}

impl Error for HtmlAttributeValidationError {}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Phase {
    Pair,
    NamespaceLength,
    NamespaceUnit { offset: usize, len: usize },
    LocalLength,
    LocalUnit { offset: usize, len: usize },
}

/// The exact next step's fixed work, before its operand read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HtmlAttributeValidationLayout {
    record_visits: u64,
    inspected_bytes: u64,
    record_writes: u64,
    copied_bytes: u64,
}

impl HtmlAttributeValidationLayout {
    /// Charges each nonzero dimension of the step on its own, in the order record visits,
    /// inspected bytes, record writes, copied bytes.
    pub(crate) fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        for change in [
            Change::Work(HtmlWork::RecordVisit, self.record_visits),
            Change::Work(HtmlWork::InspectByte, self.inspected_bytes),
            Change::Work(HtmlWork::RecordWrite, self.record_writes),
            Change::Work(HtmlWork::CopyByte, self.copied_bytes),
        ] {
            if !matches!(change, Change::Work(_, 0)) {
                charge.charge(&[change])?;
            }
        }
        Ok(())
    }
    /// Number of attribute/string header visits.
    pub(crate) fn record_visits(self) -> u64 {
        self.record_visits
    }

    /// Bytes inspected in exact UTF-16 unit comparisons and in hashing.
    pub(crate) fn inspected_bytes(self) -> u64 {
        self.inspected_bytes
    }

    /// Slots of the name index the step writes. Zero without an index.
    #[cfg(test)]
    pub(crate) fn record_writes(self) -> u64 {
        self.record_writes
    }

    /// Bytes of the name index the step copies. Zero without an index.
    #[cfg(test)]
    pub(crate) fn copied_bytes(self) -> u64 {
        self.copied_bytes
    }
}

/// What the next action of a cursor is. Pairwise validation only ever compares.
#[derive(Clone, Copy, Debug)]
enum Stage {
    /// Compare every pair of names; `phase` is the comparison.
    Pairs,
    /// Overwrite the next slots of the index with the empty value.
    Clear,
    /// Read the lengths of the next name to hash.
    HashLength,
    /// Hash the next name, which has `units` code units.
    Hash { units: usize },
    /// Move the probe to its next slot and read it.
    Probe { tag: u32 },
    /// Compare the next name with the attribute found in the probe's slot by the comparison in
    /// `phase`.
    Compare { tag: u32 },
}

/// An exclusive cursor retaining its allocation on interruption or refusal.
#[derive(Debug)]
pub struct HtmlAttributeValidation {
    storage: HtmlAttributeStorage,
    duplicate: usize,
    first: usize,
    phase: Phase,
    failure: Option<HtmlAttributeValidationError>,
    indexed: Option<HtmlAttributeNameIndex>,
    /// The next action's stage, or `None` once every comparison is made.
    stage: Option<Stage>,
    /// The probe of the name being entered in the index.
    probe: Option<HtmlNameIndexProbe>,
}

impl HtmlAttributeValidation {
    /// Describes the next comparison without reading the described operands.
    /// Dropping an action leaves the cursor at that same comparison.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlAttributeValidationAction<'_>>, HtmlAttributeValidationError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let stage = self.stage;
        Ok(stage.map(|stage| HtmlAttributeValidationAction {
            validation: self,
            stage,
        }))
    }

    /// Consumes a successfully completed cursor. A latched failure or an incomplete cursor refuses
    /// the finish and hands the cursor back unchanged.
    /// This transition performs no comparisons, allocation or backing clone. A name index that
    /// was not taken is dropped; [`finish_with_index`](Self::finish_with_index) gives it back.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn finish(
        self,
    ) -> Result<HtmlValidatedAttributes, HtmlFinishFailure<Self, HtmlAttributeValidationError>>
    {
        self.finish_with_index().map(|(validated, _)| validated)
    }

    /// As [`finish`](Self::finish), and the name index too if it was not taken.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn finish_with_index(
        self,
    ) -> Result<
        (HtmlValidatedAttributes, Option<HtmlAttributeNameIndex>),
        HtmlFinishFailure<Self, HtmlAttributeValidationError>,
    > {
        if let Some(error) = self.failure {
            return Err(HtmlFinishFailure::new(self, error));
        }
        if self.stage.is_some() {
            return Err(HtmlFinishFailure::new(
                self,
                HtmlAttributeValidationError::NotReady,
            ));
        }
        Ok((
            HtmlValidatedAttributes {
                storage: self.storage,
            },
            self.indexed,
        ))
    }

    /// Recovers the original allocation without further reads or publication.
    pub fn abort(self) -> HtmlAttributeStorage {
        self.storage
    }

    /// As [`abort`](Self::abort), and the name index too if it was not taken.
    pub fn abort_with_index(self) -> (HtmlAttributeStorage, Option<HtmlAttributeNameIndex>) {
        (self.storage, self.indexed)
    }

    fn next_pair(&mut self) {
        self.first += 1;
        if self.first == self.duplicate {
            self.first = 0;
            self.duplicate += 1;
        }
        self.phase = Phase::Pair;
        if self.duplicate == self.storage.attributes.len() {
            self.stage = None;
        }
    }

    fn duplicate(&mut self) -> Result<(), HtmlAttributeValidationError> {
        let error = HtmlAttributeValidationError::Duplicate {
            first: self.first,
            duplicate: self.duplicate,
        };
        self.failure = Some(error);
        Err(error)
    }

    /// Takes back the name index that `validate_indexed` was given, whatever its contents.
    /// A cursor given none returns `None`. [`abort`](Self::abort) and
    /// [`finish`](Self::finish) drop an index that was not taken; their `_with_index`
    /// variants give it back.
    pub fn take_index(&mut self) -> Option<HtmlAttributeNameIndex> {
        self.indexed.take()
    }

    fn fail(
        &mut self,
        error: HtmlAttributeValidationError,
    ) -> Result<(), HtmlAttributeValidationError> {
        self.failure = Some(error);
        Err(error)
    }

    /// Moves to the attribute after the one just placed in the index.
    fn next_attribute(&mut self) {
        self.duplicate += 1;
        self.stage = (self.duplicate != self.storage.attributes.len()).then_some(Stage::HashLength);
    }

    fn names(&self) -> (&super::QualifiedName, &super::QualifiedName) {
        (
            self.storage.attributes[self.first].name(),
            self.storage.attributes[self.duplicate].name(),
        )
    }
}

/// One non-replayable comparison borrowing its original cursor exclusively.
#[derive(Debug)]
pub struct HtmlAttributeValidationAction<'a> {
    validation: &'a mut HtmlAttributeValidation,
    /// The stage this action performs, taken when the cursor announced it.
    stage: Stage,
}

impl HtmlAttributeValidationAction<'_> {
    /// The work this action admits before it reads its operands.
    fn layout(&self) -> HtmlAttributeValidationLayout {
        let validation = &*self.validation;
        let none = |layout: HtmlAttributeValidationLayout| layout;
        match self.stage {
            Stage::Pairs | Stage::Compare { .. } => {
                let layout = comparison_layout(validation.phase);
                none(HtmlAttributeValidationLayout {
                    record_visits: layout.record_visits,
                    inspected_bytes: layout.inspected_bytes,
                    record_writes: 0,
                    copied_bytes: 0,
                })
            }
            Stage::Clear => {
                let table = validation.indexed.as_ref();
                let slots = table
                    .map_or(0, HtmlAttributeNameIndex::unwritten)
                    .min(CLEAR_CHUNK);
                // The chunk that finishes the table also binds it to its index, which reads
                // every slot once and records the storage it guards.
                let scanned = table.map_or(0, |table| table.scanned_by_clearing(CLEAR_CHUNK));
                let finishing = usize::from(scanned != 0);
                // The first chunk follows the start of the rebuild: a new index, an unbound
                // table and an empty storage.
                let first =
                    usize::from(table.is_some_and(|table| {
                        table.unwritten() == table.size() && table.size() != 0
                    }));
                HtmlAttributeValidationLayout {
                    record_visits: 0,
                    inspected_bytes: (scanned * size_of::<HtmlNameIndexSlot>()) as u64,
                    record_writes: (slots + finishing + 3 * first) as u64,
                    copied_bytes: (slots * size_of::<HtmlNameIndexSlot>()
                        + finishing * size_of::<usize>()
                        + first * (size_of::<HtmlNameIndex>() + size_of::<usize>() * 2))
                        as u64,
                }
            }
            // The attribute and its two name headers.
            Stage::HashLength => HtmlAttributeValidationLayout {
                record_visits: 3,
                inspected_bytes: 0,
                record_writes: 0,
                copied_bytes: 0,
            },
            Stage::Hash { units } => HtmlAttributeValidationLayout {
                record_visits: 0,
                inspected_bytes: 2 * units as u64,
                record_writes: 0,
                copied_bytes: 0,
            },
            // One slot read; the write is admitted for the empty slot it may find.
            Stage::Probe { .. } => HtmlAttributeValidationLayout {
                record_visits: 1,
                inspected_bytes: 0,
                record_writes: 2,
                copied_bytes: (size_of::<HtmlNameIndexSlot>() + size_of::<usize>()) as u64,
            },
        }
    }

    /// Charges this action's work, then performs only its read and advances its cursor. Each
    /// nonzero dimension of the work is its own charge, in the order record visits, inspected
    /// bytes, record writes, copied bytes. A refused charge leaves the cursor at the same action.
    /// A duplicate is latched before returning its first refusal.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlAttributeValidationError>> {
        self.layout()
            .charge(charge)
            .map_err(HtmlChargedError::Stopped)?;
        self.perform().map_err(HtmlChargedError::Faulted)
    }
    /// Performs only this action's read and advances its cursor.
    fn perform(self) -> Result<(), HtmlAttributeValidationError> {
        let validation = self.validation;
        match self.stage {
            Stage::Pairs => {
                let (left, right) = validation.names();
                match advance_comparison(
                    validation.phase,
                    left.namespace().uri(),
                    left.local_name(),
                    right.namespace().uri(),
                    right.local_name(),
                ) {
                    ComparisonStep::Continue(phase) => validation.phase = phase,
                    ComparisonStep::Different => validation.next_pair(),
                    ComparisonStep::Equal => return validation.duplicate(),
                }
                Ok(())
            }
            Stage::Clear => {
                let Some(index) = validation.indexed.as_mut() else {
                    return validation.fail(HtmlAttributeValidationError::IndexUnavailable);
                };
                index.clear_chunk(CLEAR_CHUNK);
                if index.unwritten() == 0 {
                    if index.bind_storage().is_err() {
                        return validation.fail(HtmlAttributeValidationError::IndexUnavailable);
                    }
                    validation.stage = Some(Stage::HashLength);
                }
                Ok(())
            }
            Stage::HashLength => {
                let name = validation.storage.attributes[validation.duplicate].name();
                let units = name
                    .namespace()
                    .uri()
                    .map_or(0, DocumentString::len)
                    .saturating_add(name.local_name().len());
                validation.stage = Some(Stage::Hash { units });
                Ok(())
            }
            Stage::Hash { .. } => {
                let name = validation.storage.attributes[validation.duplicate].name();
                let Some(index) = validation.indexed.as_mut() else {
                    return validation.fail(HtmlAttributeValidationError::IndexUnavailable);
                };
                let hash = index.hash(name);
                let Ok(probe) = index.probe(hash) else {
                    return validation.fail(HtmlAttributeValidationError::IndexUnavailable);
                };
                validation.probe = Some(probe);
                validation.stage = Some(Stage::Probe { tag: tag(hash) });
                Ok(())
            }
            Stage::Probe { tag } => {
                let (Some(index), Some(probe)) =
                    (validation.indexed.as_mut(), validation.probe.as_mut())
                else {
                    return validation.fail(HtmlAttributeValidationError::IndexUnavailable);
                };
                match index.advance(probe) {
                    Ok(Some(HtmlNameIndexStep::Vacant(vacancy))) => {
                        let placed = entry(tag, Occupant::Attribute(validation.duplicate))
                            .is_some_and(|entry| index.insert(vacancy, entry).is_ok());
                        if !placed {
                            return validation.fail(HtmlAttributeValidationError::IndexUnavailable);
                        }
                        validation.probe = None;
                        validation.next_attribute();
                        Ok(())
                    }
                    // Another name's tag: not this name, so not read; the next action advances.
                    Ok(Some(HtmlNameIndexStep::Occupied(held))) => match occupant(held) {
                        (_, held_tag) if held_tag != tag => Ok(()),
                        (Occupant::Attribute(first), _) if first < validation.duplicate => {
                            validation.first = first;
                            validation.phase = Phase::Pair;
                            validation.stage = Some(Stage::Compare { tag });
                            Ok(())
                        }
                        _ => validation.fail(HtmlAttributeValidationError::IndexUnavailable),
                    },
                    Ok(None) | Err(_) => {
                        validation.fail(HtmlAttributeValidationError::IndexUnavailable)
                    }
                }
            }
            Stage::Compare { tag } => {
                let (old, new) = validation.names();
                match advance_comparison(
                    validation.phase,
                    old.namespace().uri(),
                    old.local_name(),
                    new.namespace().uri(),
                    new.local_name(),
                ) {
                    ComparisonStep::Continue(phase) => {
                        validation.phase = phase;
                        Ok(())
                    }
                    ComparisonStep::Different => {
                        validation.phase = Phase::Pair;
                        validation.stage = Some(Stage::Probe { tag });
                        Ok(())
                    }
                    ComparisonStep::Equal => validation.duplicate(),
                }
            }
        }
    }
}

/// A complete expanded-name proof attached to its actual immutable storage.
/// It cannot be constructed from a flag, caller assertion or another allocation.
#[derive(Debug)]
pub struct HtmlValidatedAttributes {
    storage: HtmlAttributeStorage,
}
impl From<OrderedAttributes> for HtmlValidatedAttributes {
    /// Transfers an existing expanded-name proof with its original backing.
    /// No comparison, allocation, clone or validation is performed.
    fn from(value: OrderedAttributes) -> Self {
        Self {
            storage: HtmlAttributeStorage {
                attributes: value.into_shared_attributes(),
            },
        }
    }
}

impl HtmlValidatedAttributes {
    /// Borrows the exact validated order without publishing or copying storage.
    pub fn as_slice(&self) -> &[Attribute] {
        &self.storage.attributes
    }
    /// Publishes exact input order while moving the original shared backing.
    pub fn into_attributes(self) -> OrderedAttributes {
        OrderedAttributes {
            attributes: self.storage.attributes,
        }
    }
}

pub(crate) enum ComparisonStep {
    Continue(Phase),
    Different,
    Equal,
}
pub(crate) fn comparison_layout(phase: Phase) -> HtmlAttributeValidationLayout {
    match phase {
        Phase::Pair | Phase::NamespaceLength | Phase::LocalLength => {
            HtmlAttributeValidationLayout {
                record_visits: 2,
                inspected_bytes: 0,
                record_writes: 0,
                copied_bytes: 0,
            }
        }
        Phase::NamespaceUnit { .. } | Phase::LocalUnit { .. } => HtmlAttributeValidationLayout {
            record_visits: 0,
            inspected_bytes: 4,
            record_writes: 0,
            copied_bytes: 0,
        },
    }
}
/// One step of the comparison of two expanded names. The answer is total: two names are equal
/// exactly when both have the same namespace, or both have none, and the same local name, so a
/// namespace phase for names that disagree on having one, or a unit past either name's end, is a
/// difference, not a state to refuse.
pub(crate) fn advance_comparison(
    phase: Phase,
    left_namespace: Option<&DocumentString>,
    left_local: &DocumentString,
    right_namespace: Option<&DocumentString>,
    right_local: &DocumentString,
) -> ComparisonStep {
    use ComparisonStep::{Continue, Different, Equal};
    match phase {
        Phase::Pair => match (left_namespace, right_namespace) {
            (None, None) => Continue(Phase::LocalLength),
            (Some(_), Some(_)) => Continue(Phase::NamespaceLength),
            _ => Different,
        },
        Phase::NamespaceLength | Phase::LocalLength => {
            let namespace = matches!(phase, Phase::NamespaceLength);
            let (left, right) = if namespace {
                let (Some(left), Some(right)) = (left_namespace, right_namespace) else {
                    return Different;
                };
                (left, right)
            } else {
                (left_local, right_local)
            };
            if left.len() != right.len() {
                Different
            } else if left.is_empty() {
                if namespace {
                    Continue(Phase::LocalLength)
                } else {
                    Equal
                }
            } else if namespace {
                Continue(Phase::NamespaceUnit {
                    offset: 0,
                    len: left.len(),
                })
            } else {
                Continue(Phase::LocalUnit {
                    offset: 0,
                    len: left.len(),
                })
            }
        }
        Phase::NamespaceUnit { offset, len } | Phase::LocalUnit { offset, len } => {
            let namespace = matches!(phase, Phase::NamespaceUnit { .. });
            let (left, right) = if namespace {
                let (Some(left), Some(right)) = (left_namespace, right_namespace) else {
                    return Different;
                };
                (left, right)
            } else {
                (left_local, right_local)
            };
            let (Some(left_unit), Some(right_unit)) = (
                left.code_units().get(offset),
                right.code_units().get(offset),
            ) else {
                return Different;
            };
            if left_unit != right_unit {
                Different
            } else if offset + 1 == len {
                if namespace {
                    Continue(Phase::LocalLength)
                } else {
                    Equal
                }
            } else if namespace {
                Continue(Phase::NamespaceUnit {
                    offset: offset + 1,
                    len,
                })
            } else {
                Continue(Phase::LocalUnit {
                    offset: offset + 1,
                    len,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::values::{Namespace, QualifiedName};

    /// A deterministic generator, so a failing case is replayable.
    struct Random(u64);
    impl Random {
        fn next(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % bound as u64) as usize
        }
    }

    fn attribute(namespace: &str, prefix: Option<&str>, local: &str) -> Attribute {
        Attribute::new(
            QualifiedName::new(
                Namespace::from_uri(namespace),
                prefix.map(DocumentString::from),
                DocumentString::from(local),
            ),
            "v",
        )
    }

    /// Names that collide often: the same letters in both cases, empty names, an empty and a
    /// present namespace, and prefixes that identity ignores.
    fn random_attribute(random: &mut Random) -> Attribute {
        const LOCALS: [&str; 8] = ["a", "A", "b", "ab", "ba", "", "a\u{0}", "\u{ffff}"];
        const NAMESPACES: [&str; 4] = ["", "", "urn:n", "urn:N"];
        const PREFIXES: [Option<&str>; 3] = [None, Some(""), Some("p")];
        attribute(
            NAMESPACES[random.next(NAMESPACES.len())],
            PREFIXES[random.next(PREFIXES.len())],
            LOCALS[random.next(LOCALS.len())],
        )
    }

    #[derive(Default)]
    struct Work {
        visits: u64,
        inspected: u64,
        writes: u64,
        copied: u64,
        steps: u64,
    }

    impl Work {
        fn total(&self) -> u64 {
            self.visits + self.inspected + self.writes + self.copied + self.steps
        }
    }

    fn run(
        mut validation: HtmlAttributeValidation,
    ) -> (Result<(), HtmlAttributeValidationError>, Work) {
        let mut work = Work::default();
        let result = (|| {
            while let Some(action) = validation.next_action()? {
                let layout = action.layout();
                work.visits += layout.record_visits();
                work.inspected += layout.inspected_bytes();
                work.writes += layout.record_writes();
                work.copied += layout.copied_bytes();
                work.steps += 1;
                action.perform()?;
            }
            Ok(())
        })();
        (result, work)
    }

    fn index_for(count: usize) -> HtmlAttributeNameIndex {
        let slots = HtmlAttributeNameIndex::slots_required(count).expect("representable");
        HtmlAttributeNameIndex::new(Vec::<HtmlNameIndexSlot>::with_capacity(slots))
    }

    #[test]
    fn an_indexed_validation_reaches_the_verdict_of_the_pairwise_one_on_random_names() {
        let mut random = Random(0x9e37_79b9_7f4a_7c15);
        let mut duplicates = 0;
        let mut distinct = 0;
        for round in 0..3000 {
            let count = random.next(if round % 10 == 0 { 40 } else { 8 });
            let attributes: Vec<Attribute> =
                (0..count).map(|_| random_attribute(&mut random)).collect();
            let expected = run(HtmlAttributeStorage::from_shared_attributes(Arc::new(
                attributes.clone(),
            ))
            .validate())
            .0;
            let indexed = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes))
                .validate_indexed(index_for(count));
            let got = run(indexed).0;
            assert_eq!(got, expected, "round {round}, {count} attributes");
            match expected {
                Ok(()) => distinct += 1,
                Err(_) => duplicates += 1,
            }
        }
        assert!(
            duplicates > 100 && distinct > 100,
            "{duplicates} {distinct}"
        );
    }

    #[test]
    fn an_indexed_validation_reports_the_first_repeat_and_its_earlier_owner() {
        let attributes = vec![
            attribute("", None, "a"),
            attribute("urn:n", None, "a"),
            attribute("", Some("p"), "b"),
            attribute("", None, "A"),
            attribute("urn:n", Some("q"), "a"),
            attribute("", None, "a"),
        ];
        let validation = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes))
            .validate_indexed(index_for(6));
        assert_eq!(
            run(validation).0,
            Err(HtmlAttributeValidationError::Duplicate {
                first: 1,
                duplicate: 4
            })
        );
    }

    #[test]
    fn an_index_too_small_for_the_attributes_still_validates_every_pair() {
        let attributes: Vec<Attribute> = ["a", "b", "a"]
            .iter()
            .map(|name| attribute("", None, name))
            .collect();
        let validation = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes))
            .validate_indexed(HtmlAttributeNameIndex::new(Vec::new()));
        assert_eq!(
            run(validation).0,
            Err(HtmlAttributeValidationError::Duplicate {
                first: 0,
                duplicate: 2
            })
        );
    }

    #[test]
    fn a_finished_validation_gives_the_index_back() {
        let attributes: Vec<Attribute> = (0..5)
            .map(|i| attribute("", None, &format!("a{i}")))
            .collect();
        let index = index_for(5);
        let capacity = index.capacity();
        let mut validation = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes))
            .validate_indexed(index);
        while let Some(action) = validation.next_action().expect("distinct names") {
            action.perform().expect("distinct names");
        }
        let index = validation.take_index().expect("the cursor held the index");
        assert_eq!(index.capacity(), capacity, "validation never reallocates");
        assert!(validation.take_index().is_none());
        assert!(validation.finish().is_ok());
    }

    #[test]
    fn indexed_validation_work_at_most_doubles_with_the_attribute_count() {
        let work = |count: usize| {
            let attributes: Vec<Attribute> = (0..count)
                .map(|i| attribute("", None, &format!("attribute{i}")))
                .collect();
            let validation = HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes))
                .validate_indexed(index_for(count));
            let (result, work) = run(validation);
            assert_eq!(result, Ok(()));
            work.total()
        };
        let measured: Vec<u64> = [250, 500, 1000, 2000, 4000].map(work).to_vec();
        for pair in measured.windows(2) {
            assert!(
                pair[1] * 10 <= pair[0] * 22,
                "doubling the attributes took work from {} to {}",
                pair[0],
                pair[1]
            );
        }
        // The pairwise validation is the quadratic work this replaces.
        let attributes: Vec<Attribute> = (0..1000)
            .map(|i| attribute("", None, &format!("attribute{i}")))
            .collect();
        let pairwise =
            run(HtmlAttributeStorage::from_shared_attributes(Arc::new(attributes)).validate())
                .1
                .total();
        assert!(
            pairwise > 10 * measured[2],
            "{pairwise} against {}",
            measured[2]
        );
    }

    /// Independent oracle: direct equality of the expanded names. Stepping the comparison to its
    /// end agrees with it for every pair of names that collide often, in at most one step per
    /// namespace unit, local unit and length, and a phase asked of names that do not have what it
    /// reads is a difference.
    #[test]
    fn stepping_a_comparison_agrees_with_equality_of_expanded_names() {
        const LOCALS: [&str; 6] = ["a", "A", "ab", "", "a\u{0}", "\u{ffff}"];
        const NAMESPACES: [Option<&str>; 4] = [None, Some(""), Some("urn:n"), Some("urn:N")];
        for left_namespace in NAMESPACES {
            for right_namespace in NAMESPACES {
                for left_local in LOCALS {
                    for right_local in LOCALS {
                        let namespace = |value: Option<&str>| value.map(DocumentString::from);
                        let (left_ns, right_ns) =
                            (namespace(left_namespace), namespace(right_namespace));
                        let (left_local_name, right_local_name) = (
                            DocumentString::from(left_local),
                            DocumentString::from(right_local),
                        );
                        let mut phase = Phase::Pair;
                        let equal = loop {
                            match advance_comparison(
                                phase,
                                left_ns.as_ref(),
                                &left_local_name,
                                right_ns.as_ref(),
                                &right_local_name,
                            ) {
                                ComparisonStep::Continue(next) => phase = next,
                                ComparisonStep::Different => break false,
                                ComparisonStep::Equal => break true,
                            }
                        };
                        assert_eq!(
                            equal,
                            left_namespace == right_namespace && left_local == right_local,
                            "{left_namespace:?} {left_local:?} against {right_namespace:?} {right_local:?}"
                        );
                    }
                }
            }
        }
        let some = DocumentString::from("urn:n");
        let local = DocumentString::from("a");
        for phase in [
            Phase::NamespaceLength,
            Phase::NamespaceUnit { offset: 0, len: 5 },
        ] {
            assert!(matches!(
                advance_comparison(phase, Some(&some), &local, None, &local),
                ComparisonStep::Different
            ));
        }
        assert!(matches!(
            advance_comparison(
                Phase::LocalUnit { offset: 3, len: 4 },
                None,
                &local,
                None,
                &local
            ),
            ComparisonStep::Different
        ));
    }
}
