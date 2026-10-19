//! Private numerical/CSS servicing for the one canonical Draft.
//!
//! The original scratch tariff is independent from Source's actual ownership
//! admission. Both work domains remain cumulative across reached operations.
use super::source::{checkpoint::Charge, formation::Formation, storage::Payload};
use super::{DomError, DomResource, check_cancel};
use crate::DocumentString;
use std::{mem::size_of, sync::atomic::AtomicBool};
use surgeist_html::HtmlInputColorSpace;
mod color;
mod range;
pub(super) struct RangeInput<'a> {
    pub(super) current: &'a DocumentString,
    pub(super) min: Option<&'a DocumentString>,
    pub(super) max: Option<&'a DocumentString>,
    pub(super) step: Option<&'a DocumentString>,
    pub(super) default_value: Option<&'a DocumentString>,
}
struct OutputBuffer {
    units: Vec<u16>,
    payload: Option<Charge>,
}
pub(super) struct InputValue {
    pub(super) value: DocumentString,
    pub(super) payload: Option<Charge>,
}
// Keep this original child ledger's layout and byte/work tariff unchanged.
pub(super) struct InputProcessing {
    scratch_limit: usize,
    work_limit: u64,
    live: usize,
    peak: usize,
    work: u64,
    stopped: Option<DomError>,
}
#[derive(Clone, Copy)]
enum BankKind {
    Bytes,
    Utf16,
}
impl BankKind {
    fn payload(self, capacity: usize, width: usize) -> Result<Payload, DomError> {
        Ok(Payload {
            bytes: extent(capacity, width)?,
            units: match self {
                Self::Bytes => 0,
                Self::Utf16 => capacity,
            },
        })
    }
}
fn metadata(value: usize) -> Payload {
    Payload {
        bytes: value,
        units: 0,
    }
}
fn output_payload(units: &Vec<u16>) -> Result<Payload, DomError> {
    Ok(Payload {
        bytes: sum(&[
            size_of::<Vec<u16>>(),
            extent(units.capacity(), size_of::<u16>())?,
        ])?,
        units: units.capacity(),
    })
}
fn source_marker(source: Option<&Formation>) -> Payload {
    source.map_or(Payload::default(), Formation::owned_payload)
}
// Only after all objects covered by the released scope have actually dropped.
// A split output Charge is excluded from this lease's remaining ownership.
fn source_cleanup(source: Option<&Formation>, marker: Payload) {
    if let Some(source) = source {
        let owned = source.owned_payload();
        source.release_payload(Payload {
            bytes: owned
                .bytes
                .checked_sub(marker.bytes)
                .expect("Source scope bytes"),
            units: owned
                .units
                .checked_sub(marker.units)
                .expect("Source scope units"),
        });
    }
}
impl InputProcessing {
    pub(super) fn admit_limits(&self, scratch: usize, work: u64) -> Result<(), DomError> {
        super::admit(self.peak, scratch, DomResource::InputProcessingScratch)?;
        if self.work > work {
            return Err(DomError::InputProcessingWorkLimit { limit: work });
        }
        Ok(())
    }
    pub(super) fn update_limits(&mut self, scratch: usize, work: u64) {
        self.scratch_limit = scratch;
        self.work_limit = work;
    }
    pub(super) fn checkpoint(&self) -> Self {
        assert_eq!(
            self.live, 0,
            "source boundaries retain no live numerical registers"
        );
        assert!(
            self.stopped.is_none(),
            "checkpoint precedes the fallible segment"
        );
        Self {
            scratch_limit: self.scratch_limit,
            work_limit: self.work_limit,
            live: 0,
            peak: self.peak,
            work: self.work,
            stopped: None,
        }
    }
    pub(super) fn preserve_spent(&mut self, failed: &Self) {
        assert_eq!(
            failed.live, 0,
            "failed numerical kernels released registers"
        );
        self.work = self.work.max(failed.work);
        self.peak = self.peak.max(failed.peak);
    }
    pub(super) fn new(scratch_bytes: usize, work: u64) -> Self {
        Self {
            scratch_limit: scratch_bytes,
            work_limit: work,
            live: 0,
            peak: 0,
            work: 0,
            stopped: None,
        }
    }
    fn fail(&mut self, error: DomError) -> DomError {
        self.stopped.get_or_insert(error).clone()
    }
    fn poll(&mut self, cancelled: &AtomicBool) -> Result<(), DomError> {
        if let Some(error) = &self.stopped {
            return Err(error.clone());
        }
        check_cancel(cancelled).map_err(|error| self.fail(error))
    }
    fn charge(
        &mut self,
        amount: usize,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.poll(cancel)?;
        let next = self
            .work
            .checked_add(
                u64::try_from(amount).map_err(|_| self.fail(DomError::RepresentationExhausted))?,
            )
            .ok_or_else(|| self.fail(DomError::RepresentationExhausted))?;
        if next > self.work_limit {
            return Err(self.fail(DomError::InputProcessingWorkLimit {
                limit: self.work_limit,
            }));
        }
        self.work = next;
        if let Some(source) = source {
            source.step(cancel).map_err(|error| self.fail(error))?;
        }
        Ok(())
    }
    // A cursor's terminal/request peek is itself a bounded Source read. The
    // original child meter has no additional tariff for that observation.
    fn inspect(&mut self, source: Option<&Formation>, cancel: &AtomicBool) -> Result<(), DomError> {
        if let Some(source) = source {
            self.poll(cancel)?;
            source.step(cancel).map_err(|error| self.fail(error))?;
        }
        Ok(())
    }
    fn acquire(
        &mut self,
        amount: usize,
        payload: Payload,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.poll(cancel)?;
        let next = self
            .live
            .checked_add(amount)
            .ok_or_else(|| self.fail(DomError::RepresentationExhausted))?;
        if next > self.scratch_limit {
            return Err(self.fail(DomError::ResourceLimit {
                resource: DomResource::InputProcessingScratch,
                limit: self.scratch_limit,
            }));
        }
        if let Some(source) = source {
            source
                .retain_payload(payload, cancel)
                .map_err(|error| self.fail(error))?;
        }
        self.live = next;
        self.peak = self.peak.max(next);
        Ok(())
    }
    fn release(&mut self, amount: usize, payload: Payload, source: Option<&Formation>) {
        self.live = self.live.checked_sub(amount).expect("owned scratch charge");
        if let Some(source) = source {
            source.release_payload(payload);
        }
    }
    fn fixed(
        &mut self,
        amount: usize,
        source_bytes: usize,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.charge(sum(&[1, amount])?, source, cancel)?;
        self.acquire(amount, metadata(source_bytes), source, cancel)
    }
    fn reserve<T>(
        &mut self,
        buffer: &mut Vec<T>,
        required: usize,
        kind: BankKind,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if required <= buffer.capacity() {
            return Ok(());
        }
        let header = sum(&[size_of::<Vec<T>>(), size_of::<std::vec::Drain<'_, T>>()])?;
        self.fixed(header, header, source, cancel)?;
        let result = self.grow(buffer, required, kind, source, cancel);
        self.release(header, metadata(header), source);
        result
    }
    fn grow<T>(
        &mut self,
        buffer: &mut Vec<T>,
        required: usize,
        kind: BankKind,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let width = size_of::<T>();
        if width == 0 {
            return Err(self.fail(DomError::InputProcessingProtocol));
        }
        let requested = extent(required, width)?;
        let old_bytes = extent(buffer.capacity(), width)?;
        let old_capacity = buffer.capacity();
        let movement = extent(buffer.len(), width)?;
        self.charge(sum(&[1, requested, movement, movement])?, source, cancel)?;
        #[cfg(test)]
        let source_growth = source_observation::before(old_bytes, source.is_some());
        self.acquire(requested, kind.payload(required, width)?, source, cancel)?;
        let mut next = Vec::new();
        if next.try_reserve_exact(required).is_err() {
            self.release(requested, kind.payload(required, width)?, source);
            return Err(self.fail(DomError::AllocationFailed));
        }
        let actual = extent(next.capacity(), width)?;
        #[cfg(test)]
        if super::scale_metrics::ENABLED && actual != 0 {
            super::scale_metrics::record(super::scale_metrics::Event::StructuralAllocation {
                bytes: actual,
            });
        }
        #[cfg(test)]
        source_observation::grew(source_growth, actual);
        if let Err(error) = self.acquire(
            actual - requested,
            kind.payload(next.capacity() - required, width)?,
            source,
            cancel,
        ) {
            drop(next);
            self.release(requested, kind.payload(required, width)?, source);
            return Err(error);
        }
        if let Err(error) = self.charge(actual - requested, source, cancel) {
            let capacity = next.capacity();
            drop(next);
            self.release(actual, kind.payload(capacity, width)?, source);
            return Err(error);
        }
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        let moved = (|| {
            let mut values = buffer.drain(..);
            loop {
                self.inspect(source, cancel)?;
                let Some(value) = values.next() else {
                    break;
                };
                self.charge(1, source, cancel)?;
                next.push(value);
                #[cfg(test)]
                written.pushed::<T>();
            }
            Ok(())
        })();
        if let Err(error) = moved {
            let capacity = next.capacity();
            drop(next);
            self.release(actual, kind.payload(capacity, width)?, source);
            return Err(error);
        }
        let old = std::mem::replace(buffer, next);
        drop(old);
        self.release(old_bytes, kind.payload(old_capacity, width)?, source);
        Ok(())
    }
    fn output(
        &mut self,
        units: Vec<u16>,
        payload: Option<Charge>,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<InputValue, DomError> {
        if let Err(error) = self.charge(sum(&[1, size_of::<DocumentString>()])?, source, cancel) {
            drop(units);
            drop(payload);
            return Err(error);
        }
        // This exact output Vec header was admitted by its own frame. The
        // move to immutable heap ownership reclassifies that same header.
        let payload = match (payload, source) {
            (Some(payload), _) => Some(payload),
            (None, Some(source)) => {
                let payload = output_payload(&units)?;
                Some(source.split_payload(payload))
            }
            (None, None) => None,
        };
        Ok(InputValue {
            value: DocumentString::from_utf16(units),
            payload,
        })
    }
    fn shared_output(
        &mut self,
        value: &DocumentString,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<InputValue, DomError> {
        self.charge(sum(&[1, size_of::<DocumentString>()])?, source, cancel)?;
        let payload = if let Some(source) = source {
            source.step(cancel).map_err(|error| self.fail(error))?;
            let mut payload = Payload::default();
            payload.string(value)?;
            source
                .retain_payload(payload, cancel)
                .map_err(|error| self.fail(error))?;
            Some(source.split_payload(payload))
        } else {
            None
        };
        Ok(InputValue {
            value: value.clone(),
            payload,
        })
    }
    pub(super) fn range(
        &mut self,
        input: RangeInput<'_>,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<InputValue, DomError> {
        let base = self.live;
        let marker = source_marker(source);
        let result = range::run(self, input, source, cancel);
        self.live = base;
        source_cleanup(source, marker);
        result.map_err(|error| self.fail(error))
    }
    pub(super) fn color(
        &mut self,
        input: &DocumentString,
        alpha: bool,
        space: HtmlInputColorSpace,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<InputValue, DomError> {
        let base = self.live;
        let marker = source_marker(source);
        let result = color::run(self, input, alpha, space, source, cancel);
        self.live = base;
        source_cleanup(source, marker);
        result.map_err(|error| self.fail(error))
    }
}

fn sum(parts: &[usize]) -> Result<usize, DomError> {
    parts.iter().try_fold(0usize, |total, part| {
        total
            .checked_add(*part)
            .ok_or(DomError::RepresentationExhausted)
    })
}

fn extent(count: usize, width: usize) -> Result<usize, DomError> {
    count
        .checked_mul(width)
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(DomError::RepresentationExhausted)
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Commit {
    RangeCell,
    ColorSlot,
    ColorDigit,
    OutputUnit,
    ColorOutputRetained,
}

#[cfg(test)]
thread_local! {
    static CANCEL_AT: std::cell::Cell<Option<Commit>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn committed(point: Commit, cancelled: &AtomicBool) {
    CANCEL_AT.with(|target| {
        if target.get() == Some(point) {
            target.set(None);
            cancelled.store(true, std::sync::atomic::Ordering::Release);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use surgeist_html::HtmlInputColorSpace;

    // Explicit caller-selected fixture allowances; production chooses neither.
    pub(super) fn fixture() -> InputProcessing {
        InputProcessing::new(2_000_000, 2_000_000_000)
    }

    #[test]
    fn zero_allowances_stop_before_processing_and_keep_the_first_error() {
        let cancel = AtomicBool::new(false);
        let mut meter = InputProcessing::new(0, 0);
        let first = DomError::InputProcessingWorkLimit { limit: 0 };
        assert_eq!(
            meter
                .color(
                    &DocumentString::from("red"),
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    None,
                    &cancel
                )
                .map(|output| output.value),
            Err(first.clone())
        );
        assert_eq!(meter.work, 0);
        assert_eq!(meter.live, 0);
        cancel.store(true, Ordering::Release);
        assert_eq!(
            meter
                .range(
                    RangeInput {
                        current: &DocumentString::from(""),
                        min: None,
                        max: None,
                        step: None,
                        default_value: None,
                    },
                    None,
                    &cancel
                )
                .map(|output| output.value),
            Err(first)
        );
        assert_eq!(meter.work, 0);

        let mut meter = InputProcessing::new(0, 2_000_000_000);
        cancel.store(false, Ordering::Release);
        assert_eq!(
            meter
                .color(
                    &DocumentString::from("red"),
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    None,
                    &cancel
                )
                .map(|output| output.value),
            Err(DomError::ResourceLimit {
                resource: DomResource::InputProcessingScratch,
                limit: 0,
            })
        );
        assert_eq!(meter.live, 0);
        assert_eq!(meter.peak, 0);
    }

    #[test]
    fn replacement_growth_one_short_preserves_the_actual_old_buffer() {
        let cancel = AtomicBool::new(false);
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(1).unwrap();
        buffer.push(7u8);
        let old_capacity = buffer.capacity();
        let requested = old_capacity + 1;
        // Actual old capacity + the new Vec/Drain headers + requested replacement
        // capacity coexist before any old byte can move. One short must refuse
        // before allocating/moving the replacement, independent of allocator
        // rounding or a previously observed whole-operation peak.
        let limit =
            old_capacity + size_of::<Vec<u8>>() + size_of::<std::vec::Drain<'_, u8>>() + requested
                - 1;
        let mut meter = InputProcessing::new(limit, 10_000);
        meter
            .acquire(old_capacity, metadata(old_capacity), None, &cancel)
            .unwrap();
        assert_eq!(
            meter.reserve(&mut buffer, requested, BankKind::Bytes, None, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::InputProcessingScratch,
                limit,
            })
        );
        assert_eq!(buffer, [7]);
        assert_eq!(buffer.capacity(), old_capacity);
        assert_eq!(meter.live, old_capacity);
    }

    #[test]
    fn transient_capacity_is_reused_but_batch_processing_work_accumulates() {
        let cancel = AtomicBool::new(false);
        let mut meter = fixture();
        let raw = DocumentString::from("red");
        assert_eq!(
            meter
                .color(&raw, false, HtmlInputColorSpace::LimitedSrgb, None, &cancel)
                .map(|output| output.value)
                .unwrap(),
            DocumentString::from("#ff0000")
        );
        let peak = meter.peak;
        let one_work = meter.work;
        assert_eq!(meter.live, 0);
        meter.scratch_limit = peak;
        assert_eq!(
            meter
                .color(&raw, false, HtmlInputColorSpace::LimitedSrgb, None, &cancel)
                .map(|output| output.value)
                .unwrap(),
            DocumentString::from("#ff0000")
        );
        assert_eq!(meter.live, 0);
        assert_eq!(meter.peak, peak);
        assert_eq!(meter.work, 2 * one_work);
        // Permit the same actual peak, and exactly the already reached work.
        // A new operation must not silently reset the batch's cumulative work.
        meter.scratch_limit = peak;
        meter.work_limit = 2 * one_work;
        assert_eq!(
            meter
                .color(&raw, false, HtmlInputColorSpace::LimitedSrgb, None, &cancel)
                .map(|output| output.value),
            Err(DomError::InputProcessingWorkLimit {
                limit: 2 * one_work,
            })
        );
        assert_eq!(meter.work, 2 * one_work);
        assert_eq!(meter.live, 0);
    }

    #[test]
    fn cancellation_after_real_protocol_commits_cleans_storage_and_stays_first() {
        for point in [
            Commit::RangeCell,
            Commit::ColorSlot,
            Commit::ColorDigit,
            Commit::OutputUnit,
        ] {
            let cancel = AtomicBool::new(false);
            let mut meter = fixture();
            CANCEL_AT.with(|target| target.set(Some(point)));
            let result = if point == Commit::RangeCell {
                meter
                    .range(
                        RangeInput {
                            current: &DocumentString::from("0.1"),
                            min: None,
                            max: None,
                            step: Some(&DocumentString::from("any")),
                            default_value: None,
                        },
                        None,
                        &cancel,
                    )
                    .map(|output| output.value)
            } else {
                meter
                    .color(
                        &DocumentString::from("red"),
                        false,
                        HtmlInputColorSpace::DisplayP3,
                        None,
                        &cancel,
                    )
                    .map(|output| output.value)
            };
            assert_eq!(result, Err(DomError::Cancelled), "{point:?}");
            CANCEL_AT.with(|target| assert_eq!(target.get(), None, "actual {point:?} was reached"));
            assert_eq!(meter.live, 0);
            let stopped_work = meter.work;
            cancel.store(false, Ordering::Release);
            assert_eq!(
                meter
                    .color(
                        &DocumentString::from("garbage"),
                        false,
                        HtmlInputColorSpace::LimitedSrgb,
                        None,
                        &cancel
                    )
                    .map(|output| output.value),
                Err(DomError::Cancelled)
            );
            assert_eq!(meter.work, stopped_work);
            assert_eq!(meter.live, 0);
        }
    }
}

#[cfg(test)]
pub(super) mod source_observation {
    use crate::store::Draft;
    use std::cell::{Cell, RefCell};
    struct Context {
        external: usize,
        ledger: super::super::custody::Ledger,
        limit: usize,
    }
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Growth {
        pub required: usize,
        pub limit: usize,
    }
    pub(in crate::store) struct Binding {
        previous: Option<Context>,
    }
    impl Drop for Binding {
        fn drop(&mut self) {
            CONTEXT.with(|context| *context.borrow_mut() = self.previous.take());
        }
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
        static LAST: Cell<Option<Growth>> = const { Cell::new(None) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(super) fn finish() -> Option<Growth> {
        ACTIVE.set(false);
        LAST.take()
    }
    pub(in crate::store) fn bind(draft: &Draft) -> Binding {
        let next = draft
            .source_limits
            .filter(|_| ACTIVE.get())
            .map(|limits| Context {
                external: draft.source_other_bytes,
                ledger: draft.custody_ledger().clone(),
                limit: limits.retained_bytes,
            });
        let previous = CONTEXT.with(|context| context.replace(next));
        Binding { previous }
    }
    pub(super) fn before(old: usize, registered: bool) -> Option<Growth> {
        CONTEXT.with(|context| {
            context.borrow().as_ref().map(|context| Growth {
                required: context.external
                    + context.ledger.total().bytes()
                    + if registered { 0 } else { old },
                limit: context.limit,
            })
        })
    }
    pub(super) fn grew(before: Option<Growth>, new: usize) {
        if let Some(mut actual) = before {
            actual.required += new;
            if LAST.get().is_none_or(|old| old.required < actual.required) {
                LAST.set(Some(actual));
            }
        }
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;
    use crate::store::{BatchIdentity, Draft};
    use crate::{Dom, DomBatch, DomLimits, DomSourceLimits, Namespace};
    use std::sync::{Arc, atomic::Ordering};

    // A real Source always keeps its canonical owner alive. Its owner-held
    // ledger allocations must remain present after the baseline is measured.
    struct Fixture {
        draft: Draft,
        _dom: Dom,
    }
    impl std::ops::Deref for Fixture {
        type Target = Draft;
        fn deref(&self) -> &Draft {
            &self.draft
        }
    }
    impl std::ops::DerefMut for Fixture {
        fn deref_mut(&mut self) -> &mut Draft {
            &mut self.draft
        }
    }

    fn draft(cancel: &AtomicBool, bytes: usize, units: usize) -> Fixture {
        let dom = Dom::new(
            DomLimits::new(
                16,
                100_000,
                32,
                0,
                0,
                8,
                100_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), cancel).unwrap();
        draft.source_limits = Some(DomSourceLimits::new(
            usize::MAX,
            usize::MAX,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
        let base = draft.source_storage_bytes(cancel).unwrap() + draft.source_other_bytes;
        let base_units = draft.source_storage_units(cancel).unwrap() + draft.source_other_units;
        draft.source_limits = Some(DomSourceLimits::new(
            base + bytes,
            base_units + units,
            100_000_000,
            64,
            4096,
            4096,
            1,
        ));
        Fixture { draft, _dom: dom }
    }

    fn assert_released(draft: &Draft, source: &Formation) {
        assert_eq!(source.owned_payload().bytes, 0);
        assert_eq!(source.owned_payload().units, 0);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn source_outputs_keep_exact_backing_after_all_numerical_scopes_end() {
        let cancel = AtomicBool::new(false);
        let mut padded = Vec::with_capacity(256);
        padded.extend(".5".encode_utf16());
        let current = DocumentString::from_utf16(padded);
        let any = DocumentString::from("any");
        let min = DocumentString::from("0");
        let max = DocumentString::from("10");
        let step = DocumentString::from("3");
        let invalid = DocumentString::from("garbage");
        let red = DocumentString::from("red");
        for case in 0..3 {
            let draft = draft(&cancel, 10_000_000, 10_000_000);
            let source = Formation::new(&draft, &cancel).unwrap().unwrap();
            let mut ordinary = super::tests::fixture();
            let mut metered = super::tests::fixture();
            let run = |meter: &mut InputProcessing, source| match case {
                0 => meter.range(
                    RangeInput {
                        current: &current,
                        min: None,
                        max: None,
                        step: Some(&any),
                        default_value: None,
                    },
                    source,
                    &cancel,
                ),
                1 => meter.range(
                    RangeInput {
                        current: &invalid,
                        min: Some(&min),
                        max: Some(&max),
                        step: Some(&step),
                        default_value: None,
                    },
                    source,
                    &cancel,
                ),
                _ => meter.color(
                    &red,
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    source,
                    &cancel,
                ),
            };
            let plain = run(&mut ordinary, None).unwrap();
            let output = run(&mut metered, Some(&source)).unwrap();
            let expected = [".5", "6", "#ff0000"][case];
            assert_eq!(output.value, DocumentString::from(expected));
            assert_eq!(output.value, plain.value);
            assert_eq!(
                (metered.work, metered.peak, metered.live),
                (ordinary.work, ordinary.peak, 0)
            );
            let retained_units = output.value.shared_code_units().capacity();
            let retained_bytes = size_of::<Vec<u16>>() + retained_units * size_of::<u16>();
            assert_eq!(source.owned_payload().bytes, 0);
            assert_eq!(source.owned_payload().units, 0);
            assert_eq!(
                draft.source_formation_bytes.load(Ordering::Relaxed),
                retained_bytes
            );
            assert_eq!(
                draft.source_temporary_units.load(Ordering::Relaxed),
                retained_units
            );
            if case == 0 {
                assert_eq!(retained_units, 256);
                assert!(Arc::ptr_eq(
                    current.shared_code_units(),
                    output.value.shared_code_units()
                ));
            }
            let InputValue { value, payload } = output;
            drop(value);
            drop(payload);
            assert_released(&draft, &source);
        }
    }

    #[test]
    fn source_utf16_growth_prices_old_new_banks_and_headers_before_copy() {
        let cancel = AtomicBool::new(false);
        for shortage in 0..3 {
            let mut bank = Vec::with_capacity(1);
            bank.push(7u16);
            let old = bank.capacity();
            let new = old + 1;
            let headers = size_of::<Vec<u16>>() + size_of::<std::vec::Drain<'_, u16>>();
            let bytes = size_of::<Vec<u16>>() + old * 2 + headers + new * 2;
            let units = old + new;
            let draft = draft(
                &cancel,
                bytes - usize::from(shortage == 1),
                units - usize::from(shortage == 2),
            );
            let source = Formation::new(&draft, &cancel).unwrap().unwrap();
            let mut meter = super::tests::fixture();
            source.retain_bytes(size_of::<Vec<u16>>(), &cancel).unwrap();
            meter
                .acquire(
                    old * 2,
                    Payload {
                        bytes: old * 2,
                        units: old,
                    },
                    Some(&source),
                    &cancel,
                )
                .unwrap();
            let result = meter.reserve(&mut bank, new, BankKind::Utf16, Some(&source), &cancel);
            if shortage == 0 {
                result.unwrap();
                assert!(bank.capacity() >= new);
                assert_eq!(bank, [7]);
            } else {
                let resource = if shortage == 1 {
                    DomResource::SourceRetainedBytes
                } else {
                    DomResource::SourceRetainedUnits
                };
                let limit = if shortage == 1 {
                    draft.source_limits.unwrap().retained_bytes
                } else {
                    draft.source_limits.unwrap().retained_units
                };
                assert_eq!(result, Err(DomError::ResourceLimit { resource, limit }));
                assert_eq!(bank.capacity(), old);
                assert_eq!(bank, [7]);
            }
            drop(bank);
            source_cleanup(Some(&source), Payload::default());
            assert_released(&draft, &source);
        }
    }

    #[test]
    fn source_cancellation_cleans_each_reached_kernel_and_preserves_first_stop() {
        for point in [
            Commit::RangeCell,
            Commit::ColorSlot,
            Commit::ColorDigit,
            Commit::OutputUnit,
            Commit::ColorOutputRetained,
        ] {
            let cancel = AtomicBool::new(false);
            let draft = draft(&cancel, 10_000_000, 10_000_000);
            let source = Formation::new(&draft, &cancel).unwrap().unwrap();
            let mut meter = super::tests::fixture();
            let _observation = observe_output(&draft);
            CANCEL_AT.set(Some(point));
            let result = if point == Commit::RangeCell {
                meter.range(
                    RangeInput {
                        current: &DocumentString::from("0.1"),
                        min: None,
                        max: None,
                        step: Some(&DocumentString::from("any")),
                        default_value: None,
                    },
                    Some(&source),
                    &cancel,
                )
            } else {
                meter.color(
                    &DocumentString::from("red"),
                    false,
                    HtmlInputColorSpace::DisplayP3,
                    Some(&source),
                    &cancel,
                )
            };
            assert_eq!(result.err(), Some(DomError::Cancelled), "{point:?}");
            assert_eq!(CANCEL_AT.get(), None);
            assert_eq!(OUTPUT_OBSERVED.get(), point == Commit::ColorOutputRetained);
            assert_released(&draft, &source);
            assert_eq!(meter.live, 0);
            let work = meter.work;
            let parent_work = draft.source_work.load(Ordering::Relaxed);
            cancel.store(false, Ordering::Release);
            assert_eq!(
                meter
                    .color(
                        &DocumentString::from("red"),
                        false,
                        HtmlInputColorSpace::LimitedSrgb,
                        Some(&source),
                        &cancel
                    )
                    .err(),
                Some(DomError::Cancelled)
            );
            assert_eq!(meter.work, work);
            assert_eq!(draft.source_work.load(Ordering::Relaxed), parent_work);
            assert_released(&draft, &source);
        }
    }

    struct OutputCounters {
        bytes: Arc<std::sync::atomic::AtomicUsize>,
        units: Arc<std::sync::atomic::AtomicUsize>,
    }
    thread_local! {
        static OUTPUT_COUNTERS: std::cell::RefCell<Option<OutputCounters>> = const { std::cell::RefCell::new(None) };
        static OUTPUT_OBSERVED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    struct OutputObservation;
    impl Drop for OutputObservation {
        fn drop(&mut self) {
            OUTPUT_COUNTERS.with(|counters| counters.borrow_mut().take());
        }
    }
    fn observe_output(draft: &Draft) -> OutputObservation {
        OUTPUT_OBSERVED.set(false);
        OUTPUT_COUNTERS.with(|counters| {
            *counters.borrow_mut() = Some(OutputCounters {
                bytes: draft.source_formation_bytes.clone(),
                units: draft.source_temporary_units.clone(),
            });
        });
        OutputObservation
    }
    // This is the actual serializer boundary, after its scratch reset and
    // before its original fallible child reacquire. Read the shared live debit,
    // rather than infer continuous ownership from a later successful result.
    pub(super) fn serialized_output(
        source: Option<&Formation>,
        units: &Vec<u16>,
        cancel: &AtomicBool,
    ) {
        OUTPUT_COUNTERS.with(|counters| {
            if let Some(counters) = counters.borrow().as_ref() {
                let owned = source.expect("actual Source invocation").owned_payload();
                let retained_units = units.capacity();
                let retained_bytes = size_of::<Vec<u16>>() + retained_units * size_of::<u16>();
                assert_eq!(
                    counters.bytes.load(Ordering::Relaxed),
                    owned.bytes + retained_bytes
                );
                assert_eq!(
                    counters.units.load(Ordering::Relaxed),
                    owned.units + retained_units
                );
                OUTPUT_OBSERVED.set(true);
            }
        });
        committed(Commit::ColorOutputRetained, cancel);
    }

    #[test]
    fn padded_unchanged_output_requires_its_full_retained_extent_before_clone() {
        let cancel = AtomicBool::new(false);
        let mut units = Vec::with_capacity(256);
        units.extend(".5".encode_utf16());
        let current = DocumentString::from_utf16(units);
        let any = DocumentString::from("any");
        for allowance in [256, 255] {
            let draft = draft(&cancel, 10_000_000, allowance);
            let source = Formation::new(&draft, &cancel).unwrap().unwrap();
            let mut meter = super::tests::fixture();
            let result = meter.range(
                RangeInput {
                    current: &current,
                    min: None,
                    max: None,
                    step: Some(&any),
                    default_value: None,
                },
                Some(&source),
                &cancel,
            );
            if allowance == 256 {
                let InputValue { value, payload } = result.unwrap();
                assert!(Arc::ptr_eq(
                    current.shared_code_units(),
                    value.shared_code_units()
                ));
                assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 256);
                drop(value);
                drop(payload);
            } else {
                assert_eq!(
                    result.err(),
                    Some(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedUnits,
                        limit: 255
                    })
                );
                assert_eq!(Arc::strong_count(current.shared_code_units()), 1);
            }
            assert_eq!(meter.live, 0);
            assert!(meter.work > 0);
            assert!(draft.source_work.load(Ordering::Relaxed) > 0);
            assert_released(&draft, &source);
        }
    }

    #[test]
    fn unavailable_color_keeps_typed_error_and_releases_source_scope() {
        let cancel = AtomicBool::new(false);
        let draft = draft(&cancel, 10_000_000, 10_000_000);
        let source = Formation::new(&draft, &cancel).unwrap().unwrap();
        let mut meter = super::tests::fixture();
        assert_eq!(
            meter
                .color(
                    &DocumentString::from("currentcolor"),
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    Some(&source),
                    &cancel
                )
                .err(),
            Some(DomError::InputColorUnavailable(
                surgeist_css::CssColorUnavailable::Palette(
                    surgeist_css::CssSystemColor::CanvasText
                )
            ))
        );
        assert_eq!(meter.live, 0);
        assert!(meter.work > 0);
        assert_released(&draft, &source);
    }

    fn actual_numeric_growth(bytes: usize) -> Option<source_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let limits = DomLimits::new(
            16,
            100_000,
            32,
            0,
            0,
            8,
            100_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_input_processing(2_000_000, 2_000_000_000)
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let input = setup.create_element_ns(&document, Namespace::html(), "input");
        setup.set_attribute(&input, "type", "color");
        let commit = dom.apply(setup, &cancel).unwrap();
        let input = commit.created(&input).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_input_value(input, "rgb(255 0 0 / .5)");
        let limits = DomSourceLimits::new(bytes, 10_000_000, 100_000_000, 64, 4096, 4096, 1);
        let Ok(mut lease) = dom.begin_source_commands(batch, limits, &cancel) else {
            return None;
        };
        source_observation::start();
        let result = lease.advance(&cancel);
        let growth = source_observation::finish();
        if result.is_err() {
            assert_eq!(lease.snapshot().stamp(), before.stamp());
        }
        growth
    }
    #[test]
    fn source_numeric_growth_admits_actual_old_and_new_banks_before_allocation() {
        let high = actual_numeric_growth(64_000_000).expect("actual Color bank growth");
        if let Some(actual) = actual_numeric_growth(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "numeric banks retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }
}
