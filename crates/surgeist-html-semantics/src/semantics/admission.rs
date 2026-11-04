//! The admission engine shared by the fact families that join external
//! producer facts to a capture: link relationships, media, image dimensions and
//! natural dimensions.
//!
//! Each family admits its incoming vectors under the same three limits (record
//! capacity, retained bytes and validation work), checks the same host revision,
//! and rejects duplicate facts through an index of the facts' subjects, filled at
//! a charge of one unit per fact. They differ in what a fact must satisfy, so
//! the engine owns only the shared accounting and leaves each constructor's
//! checks and their order to the family. Every primitive reports the family's own
//! error, and none charges after it fails.

use super::scan::{ScanError, ScanMeter};
use super::*;

/// Defines the revision key of a producer's facts: the producer's identity, the
/// configuration it ran under and its revision number. Consumers compare whole keys
/// to learn that facts came from the same producer, configuration and revision.
macro_rules! producer_revision {
    ($(#[$meta:meta])* $name:ident, $producer:ty) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name {
            producer: $producer,
            configuration: u64,
            revision: u64,
        }
        impl $name {
            pub fn new(producer: $producer, configuration: u64, revision: u64) -> Self {
                Self {
                    producer,
                    configuration,
                    revision,
                }
            }
            pub fn producer(&self) -> &$producer {
                &self.producer
            }
            pub fn configuration(&self) -> u64 {
                self.configuration
            }
            pub fn revision(&self) -> u64 {
                self.revision
            }
        }
    };
}
pub(super) use producer_revision;

/// Defines a producer-admission limits type: `records` bounds the sum of incoming
/// vector capacities, `bytes` the retained storage and `work` the validation work.
macro_rules! fact_limits {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name {
            records: usize,
            bytes: usize,
            work: usize,
        }
        impl $name {
            pub fn new(records: usize, bytes: usize, work: usize) -> Self {
                Self {
                    records,
                    bytes,
                    work,
                }
            }
            pub fn records(self) -> usize {
                self.records
            }
            pub fn bytes(self) -> usize {
                self.bytes
            }
            pub fn work(self) -> usize {
                self.work
            }
        }
        impl From<$name> for $crate::semantics::admission::FactLimits {
            fn from(limits: $name) -> Self {
                Self::new(limits.records, limits.bytes, limits.work)
            }
        }
    };
}
pub(super) use fact_limits;

/// The three limits of one producer admission.
#[derive(Clone, Copy, Debug)]
pub(super) struct FactLimits {
    pub(super) records: usize,
    pub(super) bytes: usize,
    pub(super) work: usize,
}
impl FactLimits {
    pub(super) fn new(records: usize, bytes: usize, work: usize) -> Self {
        Self {
            records,
            bytes,
            work,
        }
    }
}

/// The refusals every fact family reports, beside its work refusal and capture error.
pub(super) trait FactError: ScanError {
    /// The producer's host revision is not the capture's.
    fn host_revision_mismatch() -> Self;
    /// Incoming record capacity exceeds the record limit.
    fn record_limit() -> Self;
    /// Retained bytes exceed the byte limit.
    fn byte_limit() -> Self;
}

/// The record, byte and work totals of one admission against its limits.
pub(super) struct FactAdmission<E> {
    pub(super) limits: FactLimits,
    pub(super) records: usize,
    pub(super) bytes: usize,
    meter: ScanMeter<E>,
}
impl<E: FactError> FactAdmission<E> {
    /// An admission with nothing admitted yet.
    pub(super) fn new(limits: impl Into<FactLimits>) -> Self {
        Self::resume(limits, 0)
    }
    /// An admission that continues from `work` already admitted, with no records
    /// or bytes of its own.
    pub(super) fn resume(limits: impl Into<FactLimits>, work: usize) -> Self {
        let limits = limits.into();
        Self {
            limits,
            records: 0,
            bytes: 0,
            meter: ScanMeter::new(work, limits.work),
        }
    }
    /// The validation work admitted so far.
    pub(super) fn work_total(&self) -> usize {
        self.meter.work()
    }
    pub(super) fn work(&mut self, amount: usize) -> Result<(), E> {
        self.meter.charge(amount)
    }
    /// Looks up a node, charging 1 before the lookup.
    pub(super) fn node<'a>(
        &mut self,
        capture: &'a HtmlCapture,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, E> {
        self.meter.node(capture, id)
    }
    /// Refuses a producer whose host revision is not the capture's.
    pub(super) fn host(capture: &HtmlCapture, host: &HtmlHostFactsRevision) -> Result<(), E> {
        if host != capture.host_facts_revision() {
            return Err(E::host_revision_mismatch());
        }
        Ok(())
    }
    /// Refuses a record total beyond the record limit.
    pub(super) fn check_records(&self) -> Result<(), E> {
        if self.records > self.limits.records {
            return Err(E::record_limit());
        }
        Ok(())
    }
    /// Refuses a byte total beyond the byte limit.
    pub(super) fn check_bytes(&self) -> Result<(), E> {
        if self.bytes > self.limits.bytes {
            return Err(E::byte_limit());
        }
        Ok(())
    }
    /// Adds retained records without checking the limit; the caller checks at the
    /// point its own order of refusals requires.
    pub(super) fn grow_records(&mut self, records: usize) -> Result<(), E> {
        self.records = self
            .records
            .checked_add(records)
            .ok_or_else(E::representation_exhausted)?;
        Ok(())
    }
    /// Adds retained bytes without checking the limit; the caller checks at the
    /// point its own order of refusals requires.
    pub(super) fn grow_bytes(&mut self, bytes: usize) -> Result<(), E> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or_else(E::representation_exhausted)?;
        Ok(())
    }
    /// Adds retained records, then refuses a total beyond the record limit.
    pub(super) fn add_records(&mut self, records: usize) -> Result<(), E> {
        self.grow_records(records)?;
        self.check_records()
    }
    /// Adds retained bytes, then refuses a total beyond the byte limit.
    pub(super) fn add_bytes(&mut self, bytes: usize) -> Result<(), E> {
        self.grow_bytes(bytes)?;
        self.check_bytes()
    }
    /// Admits one incoming vector: 1 work, then its whole capacity as records and
    /// as bytes. Both totals are added before either limit is checked, the record
    /// limit first.
    pub(super) fn vector<T>(&mut self, vector: &Vec<T>) -> Result<(), E> {
        self.work(1)?;
        self.grow_records(vector.capacity())?;
        self.grow_bytes(
            vector
                .capacity()
                .checked_mul(std::mem::size_of::<T>())
                .ok_or_else(E::representation_exhausted)?,
        )?;
        self.check_records()?;
        self.check_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    enum Refusal {
        Host,
        Records,
        Bytes,
        Work,
        Exhausted,
        Capture,
    }
    impl ScanError for Refusal {
        fn representation_exhausted() -> Self {
            Self::Exhausted
        }
        fn work_exceeded(_: usize, _: usize) -> Self {
            Self::Work
        }
        fn capture(_: HtmlCaptureError) -> Self {
            Self::Capture
        }
    }
    impl FactError for Refusal {
        fn host_revision_mismatch() -> Self {
            Self::Host
        }
        fn record_limit() -> Self {
            Self::Records
        }
        fn byte_limit() -> Self {
            Self::Bytes
        }
    }
    fact_limits!(
        /// Limits of the toy family.
        ToyLimits
    );
    identity!(
        /// The toy family's producer.
        ToyProducer,
        fresh
    );
    producer_revision!(
        /// The toy family's revision key.
        ToyRevision,
        ToyProducer
    );
    fn admission(records: usize, bytes: usize, work: usize) -> FactAdmission<Refusal> {
        FactAdmission::new(ToyLimits::new(records, bytes, work))
    }

    #[test]
    fn limits_type_keeps_its_three_values() {
        let limits = ToyLimits::new(1, 2, 3);
        assert_eq!((limits.records(), limits.bytes(), limits.work()), (1, 2, 3));
        let engine = FactLimits::from(limits);
        assert_eq!((engine.records, engine.bytes, engine.work), (1, 2, 3));
    }

    #[test]
    fn a_vector_charges_one_then_admits_capacity_as_records_and_bytes() {
        let vector: Vec<u32> = Vec::with_capacity(5);
        let mut a = admission(5, 20, 1);
        a.vector(&vector).unwrap();
        assert_eq!((a.records, a.bytes, a.work_total()), (5, 20, 1));
        // The next vector needs work the limit does not allow.
        assert_eq!(a.vector(&vector), Err(Refusal::Work));
    }

    #[test]
    fn a_vector_over_both_limits_reports_records_before_bytes() {
        let vector: Vec<u32> = Vec::with_capacity(5);
        assert_eq!(admission(4, 19, 10).vector(&vector), Err(Refusal::Records));
        assert_eq!(admission(5, 19, 10).vector(&vector), Err(Refusal::Bytes));
    }

    #[test]
    fn a_vector_charges_before_it_admits_anything() {
        let vector: Vec<u32> = Vec::with_capacity(5);
        let mut a = admission(0, 0, 0);
        assert_eq!(a.vector(&vector), Err(Refusal::Work));
        assert_eq!((a.records, a.bytes), (0, 0));
    }

    #[test]
    fn growth_checks_only_when_asked_and_add_checks_at_once() {
        let mut a = admission(2, 8, 0);
        a.grow_records(3).unwrap();
        a.grow_bytes(9).unwrap();
        assert_eq!(a.check_records(), Err(Refusal::Records));
        assert_eq!(a.check_bytes(), Err(Refusal::Bytes));
        let mut b = admission(2, 8, 0);
        assert_eq!(b.add_records(3), Err(Refusal::Records));
        let mut c = admission(2, 8, 0);
        assert_eq!(c.add_bytes(9), Err(Refusal::Bytes));
        let mut d = admission(usize::MAX, usize::MAX, 0);
        d.grow_records(usize::MAX).unwrap();
        assert_eq!(d.add_records(1), Err(Refusal::Exhausted));
        d.grow_bytes(usize::MAX).unwrap();
        assert_eq!(d.add_bytes(1), Err(Refusal::Exhausted));
    }

    #[test]
    fn resume_continues_from_the_work_already_admitted() {
        let mut a = FactAdmission::<Refusal>::resume(ToyLimits::new(0, 0, 10), 7);
        assert_eq!(a.work_total(), 7);
        a.work(3).unwrap();
        assert_eq!(a.work(1), Err(Refusal::Work));
    }

    #[test]
    fn a_revision_key_equals_only_the_same_producer_configuration_and_revision() {
        let producer = ToyProducer::fresh();
        let key = ToyRevision::new(producer.clone(), 7, 9);
        assert_eq!(key, ToyRevision::new(producer.clone(), 7, 9));
        assert_ne!(key, ToyRevision::new(producer.clone(), 8, 9));
        assert_ne!(key, ToyRevision::new(producer.clone(), 7, 10));
        assert_ne!(key, ToyRevision::new(ToyProducer::fresh(), 7, 9));
        assert_eq!(key.producer(), &producer);
        assert_eq!((key.configuration(), key.revision()), (7, 9));
    }
}
