//! Private numerical/CSS servicing for the one canonical Draft.
//!
//! Scratch is peak logical simultaneous adapter storage, not RSS. Work is
//! cumulative across every reached service in a batch. Neither allowance resets
//! per control, attribute or nested operation. HTML owns all language decisions.

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

pub(super) struct InputProcessing {
    scratch_limit: usize,
    work_limit: u64,
    live: usize,
    peak: usize,
    work: u64,
    stopped: Option<DomError>,
}

impl InputProcessing {
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

    /// Admit the described work; callers include a step credit at each reached
    /// protocol/fixed transition plus its observations, writes and copies.
    fn charge(&mut self, bytes: usize, cancelled: &AtomicBool) -> Result<(), DomError> {
        self.poll(cancelled)?;
        let next = self
            .work
            .checked_add(
                u64::try_from(bytes).map_err(|_| self.fail(DomError::RepresentationExhausted))?,
            )
            .ok_or_else(|| self.fail(DomError::RepresentationExhausted))?;
        if next > self.work_limit {
            return Err(self.fail(DomError::InputProcessingWorkLimit {
                limit: self.work_limit,
            }));
        }
        self.work = next;
        Ok(())
    }

    fn acquire(&mut self, bytes: usize, cancelled: &AtomicBool) -> Result<(), DomError> {
        self.poll(cancelled)?;
        let next = self
            .live
            .checked_add(bytes)
            .ok_or_else(|| self.fail(DomError::RepresentationExhausted))?;
        if next > self.scratch_limit {
            return Err(self.fail(DomError::ResourceLimit {
                resource: DomResource::InputProcessingScratch,
                limit: self.scratch_limit,
            }));
        }
        self.live = next;
        self.peak = self.peak.max(next);
        Ok(())
    }

    /// Called only after the exact described owned storage has been dropped.
    /// Cleanup neither polls nor spends useful work after a first failure.
    fn release(&mut self, bytes: usize) {
        self.live = self.live.checked_sub(bytes).expect("owned scratch charge");
    }

    fn fixed(&mut self, bytes: usize, cancelled: &AtomicBool) -> Result<(), DomError> {
        self.charge(sum(&[1, bytes])?, cancelled)?;
        self.acquire(bytes, cancelled)
    }

    /// Replacement growth admits old+new capacity simultaneously. It charges
    /// initialized prefix reads/writes before moving them, uses a fallible exact
    /// reservation and checks the actual returned capacity before installation.
    fn reserve<T>(
        &mut self,
        buffer: &mut Vec<T>,
        required: usize,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if required <= buffer.capacity() {
            return Ok(());
        }
        let header = sum(&[size_of::<Vec<T>>(), size_of::<std::vec::Drain<'_, T>>()])?;
        self.fixed(header, cancelled)?;
        let result = self.grow(buffer, required, cancelled);
        self.release(header);
        result
    }

    fn grow<T>(
        &mut self,
        buffer: &mut Vec<T>,
        required: usize,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        let width = size_of::<T>();
        if width == 0 {
            return Err(self.fail(DomError::InputProcessingProtocol));
        }
        let requested = extent(required, width)?;
        let old_bytes = extent(buffer.capacity(), width)?;
        let movement = extent(buffer.len(), width)?;
        self.charge(sum(&[1, requested, movement, movement])?, cancelled)?;
        self.acquire(requested, cancelled)?;
        let mut next = Vec::new();
        if next.try_reserve_exact(required).is_err() {
            self.release(requested);
            return Err(self.fail(DomError::AllocationFailed));
        }
        let actual = extent(next.capacity(), width)?;
        if let Err(error) = self.acquire(actual - requested, cancelled) {
            drop(next);
            self.release(requested);
            return Err(error);
        }
        if let Err(error) = self.charge(actual - requested, cancelled) {
            drop(next);
            self.release(actual);
            return Err(error);
        }
        let moved = (|| {
            for value in buffer.drain(..) {
                self.charge(1, cancelled)?;
                next.push(value);
            }
            Ok(())
        })();
        if let Err(error) = moved {
            drop(next);
            self.release(actual);
            return Err(error);
        }
        let old = std::mem::replace(buffer, next);
        drop(old);
        self.release(old_bytes);
        Ok(())
    }

    fn output(
        &mut self,
        units: Vec<u16>,
        cancelled: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        self.charge(sum(&[1, size_of::<DocumentString>()])?, cancelled)?;
        Ok(DocumentString::from_utf16(units))
    }

    pub(super) fn range(
        &mut self,
        input: RangeInput<'_>,
        cancelled: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        let base = self.live;
        let result = range::run(self, input, cancelled);
        // run has dropped all transient banks; returned immutable output has
        // transferred to the caller's retained-value admission domain.
        self.live = base;
        result.map_err(|error| self.fail(error))
    }

    pub(super) fn color(
        &mut self,
        input: &DocumentString,
        alpha: bool,
        space: HtmlInputColorSpace,
        cancelled: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        let base = self.live;
        let result = color::run(self, input, alpha, space, cancelled);
        self.live = base;
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
            meter.color(
                &DocumentString::from("red"),
                false,
                HtmlInputColorSpace::LimitedSrgb,
                &cancel
            ),
            Err(first.clone())
        );
        assert_eq!(meter.work, 0);
        assert_eq!(meter.live, 0);
        cancel.store(true, Ordering::Release);
        assert_eq!(
            meter.range(
                RangeInput {
                    current: &DocumentString::from(""),
                    min: None,
                    max: None,
                    step: None,
                    default_value: None,
                },
                &cancel,
            ),
            Err(first)
        );
        assert_eq!(meter.work, 0);

        let mut meter = InputProcessing::new(0, 2_000_000_000);
        cancel.store(false, Ordering::Release);
        assert_eq!(
            meter.color(
                &DocumentString::from("red"),
                false,
                HtmlInputColorSpace::LimitedSrgb,
                &cancel
            ),
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
        meter.acquire(old_capacity, &cancel).unwrap();
        assert_eq!(
            meter.reserve(&mut buffer, requested, &cancel),
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
                .color(&raw, false, HtmlInputColorSpace::LimitedSrgb, &cancel)
                .unwrap(),
            DocumentString::from("#ff0000")
        );
        let peak = meter.peak;
        let one_work = meter.work;
        assert_eq!(meter.live, 0);
        meter.scratch_limit = peak;
        assert_eq!(
            meter
                .color(&raw, false, HtmlInputColorSpace::LimitedSrgb, &cancel)
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
            meter.color(&raw, false, HtmlInputColorSpace::LimitedSrgb, &cancel),
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
                meter.range(
                    RangeInput {
                        current: &DocumentString::from("0.1"),
                        min: None,
                        max: None,
                        step: Some(&DocumentString::from("any")),
                        default_value: None,
                    },
                    &cancel,
                )
            } else {
                meter.color(
                    &DocumentString::from("red"),
                    false,
                    HtmlInputColorSpace::DisplayP3,
                    &cancel,
                )
            };
            assert_eq!(result, Err(DomError::Cancelled), "{point:?}");
            CANCEL_AT.with(|target| assert_eq!(target.get(), None, "actual {point:?} was reached"));
            assert_eq!(meter.live, 0);
            let stopped_work = meter.work;
            cancel.store(false, Ordering::Release);
            assert_eq!(
                meter.color(
                    &DocumentString::from("garbage"),
                    false,
                    HtmlInputColorSpace::LimitedSrgb,
                    &cancel
                ),
                Err(DomError::Cancelled)
            );
            assert_eq!(meter.work, stopped_work);
            assert_eq!(meter.live, 0);
        }
    }
}
