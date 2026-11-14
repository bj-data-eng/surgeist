//! Producer-private caps for admissions delegated to an external host.
//!
//! A pending admission owns one prevalidated candidate. Actual buffer releases
//! update both accepted custody and that candidate before host acceptance, so
//! promotion requires only infallible cell assignments.

use std::{cell::Cell, fmt};

use surgeist_ledger::{Change, Dimension, Limits, RefusalReason};

use super::{Resource, ResourceTable, Work, WorkTable};

/// A binding-local cap or custody refusal, without a shared Ledger scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalRefusal {
    change: Change<Work, Resource>,
    current: u64,
    reason: RefusalReason,
}

impl LocalRefusal {
    /// The requested change in binding-owned units.
    pub fn change(self) -> Change<Work, Resource> {
        self.change
    }

    /// Accepted usage against which this change was checked.
    pub fn current(self) -> u64 {
        self.current
    }

    pub fn reason(self) -> RefusalReason {
        self.reason
    }
}

impl fmt::Display for LocalRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "conversion change {:?} refused at {}: {:?}",
            self.change, self.current, self.reason
        )
    }
}

impl std::error::Error for LocalRefusal {}

/// Why a host-delegated conversion admission did not begin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionError {
    Refused(LocalRefusal),
    /// Another admission or host callback is already active on this account.
    Reentrant,
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(refusal) => refusal.fmt(f),
            Self::Reentrant => f.write_str("conversion admission reentered its host boundary"),
        }
    }
}

impl std::error::Error for AdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Refused(refusal) => Some(refusal),
            Self::Reentrant => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LocalUsage {
    work: WorkTable<u64>,
    live: ResourceTable<u64>,
    peak: ResourceTable<u64>,
}

impl LocalUsage {
    fn zero() -> Self {
        Self {
            work: WorkTable::splat(0),
            live: ResourceTable::splat(0),
            peak: ResourceTable::splat(0),
        }
    }

    pub(super) fn work(self, dimension: Work) -> u64 {
        *Work::get(&self.work, dimension)
    }

    pub(super) fn live(self) -> u64 {
        self.live.BufferBytes
    }

    pub(super) fn peak(self) -> u64 {
        self.peak.BufferBytes
    }
}

pub(super) struct LocalState {
    limits: Limits<Work, Resource>,
    accepted: Cell<LocalUsage>,
    staged: Cell<LocalUsage>,
    pending: Cell<bool>,
    in_host: Cell<bool>,
}

impl LocalState {
    pub(super) fn new(limits: Limits<Work, Resource>) -> Self {
        Self {
            limits,
            accepted: Cell::new(LocalUsage::zero()),
            staged: Cell::new(LocalUsage::zero()),
            pending: Cell::new(false),
            in_host: Cell::new(false),
        }
    }

    pub(super) fn check_admission(&self) -> Result<(), AdmissionError> {
        if self.pending.get() || self.in_host.get() {
            Err(AdmissionError::Reentrant)
        } else {
            Ok(())
        }
    }

    pub(super) fn stage_work(
        &self,
        dimension: Work,
        amount: u64,
    ) -> Result<Pending<'_>, AdmissionError> {
        self.check_admission()?;
        let mut candidate = self.accepted.get();
        let current = candidate.work(dimension);
        let next = increase(current, amount, self.limits.work(dimension)).map_err(|reason| {
            AdmissionError::Refused(LocalRefusal {
                change: Change::Work(dimension, amount),
                current,
                reason,
            })
        })?;
        *Work::get_mut(&mut candidate.work, dimension) = next;
        Ok(self.stage(candidate))
    }

    pub(super) fn stage_buffer(&self, bytes: u64) -> Result<Pending<'_>, AdmissionError> {
        self.check_admission()?;
        let mut candidate = self.accepted.get();
        let current = candidate.live();
        let next = increase(current, bytes, self.limits.live(Resource::BufferBytes)).map_err(
            |reason| {
                AdmissionError::Refused(LocalRefusal {
                    change: Change::Acquire(Resource::BufferBytes, bytes),
                    current,
                    reason,
                })
            },
        )?;
        candidate.live.BufferBytes = next;
        candidate.peak.BufferBytes = candidate.peak().max(next);
        Ok(self.stage(candidate))
    }

    fn stage(&self, candidate: LocalUsage) -> Pending<'_> {
        self.staged.set(candidate);
        self.pending.set(true);
        Pending { state: self }
    }

    pub(super) fn release(&self, bytes: u64) -> Result<(), LocalRefusal> {
        let mut accepted = self.accepted.get();
        let refusal = LocalRefusal {
            change: Change::Release(Resource::BufferBytes, bytes),
            current: accepted.live(),
            reason: RefusalReason::Underflow,
        };
        accepted.live.BufferBytes = accepted.live().checked_sub(bytes).ok_or(refusal)?;

        // Pending acquisitions never give existing buffers speculative custody.
        // Validate both subtractions before mutating either accepted snapshot.
        let staged = if self.pending.get() {
            let mut candidate = self.staged.get();
            candidate.live.BufferBytes = candidate.live().checked_sub(bytes).ok_or(refusal)?;
            Some(candidate)
        } else {
            None
        };
        self.accepted.set(accepted);
        if let Some(candidate) = staged {
            self.staged.set(candidate);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn usage(&self) -> LocalUsage {
        self.accepted.get()
    }

    pub(super) fn enter_host(&self) -> HostBoundary<'_> {
        HostBoundary {
            state: self,
            previous: self.in_host.replace(true),
        }
    }
}

fn increase(current: u64, amount: u64, limit: u64) -> Result<u64, RefusalReason> {
    let next = current.checked_add(amount).ok_or(RefusalReason::Overflow)?;
    if next > limit {
        Err(RefusalReason::Limit { limit })
    } else {
        Ok(next)
    }
}

/// The sole live candidate; dropping it preserves only actual accepted usage.
pub(super) struct Pending<'a> {
    state: &'a LocalState,
}

impl Pending<'_> {
    pub(super) fn commit(self) {
        self.state.accepted.set(self.state.staged.get());
        self.state.pending.set(false);
    }
}

impl Drop for Pending<'_> {
    fn drop(&mut self) {
        self.state.pending.set(false);
    }
}

pub(super) struct HostBoundary<'a> {
    state: &'a LocalState,
    previous: bool,
}

impl Drop for HostBoundary<'_> {
    fn drop(&mut self) {
        self.state.in_host.set(self.previous);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn limited(work: u64, buffers: u64) -> LocalState {
        LocalState::new(Limits::new(
            WorkTable::splat(work),
            ResourceTable::splat(buffers),
        ))
    }

    fn historical_buffer() -> Result<LocalState, Box<dyn std::error::Error>> {
        let state = limited(u64::MAX, u64::MAX);
        state.stage_buffer(100)?.commit();
        state.release(40)?;
        Ok(state)
    }

    #[test]
    fn work_limits_are_inclusive_cumulative_and_dimension_specific() -> TestResult {
        let state = limited(7, 1);
        state.stage_work(Work::Value, 3)?.commit();
        state.stage_work(Work::Value, 4)?.commit();
        state.stage_work(Work::CodeUnit, 7)?.commit();
        let refused = state.stage_work(Work::Value, 1).map(|_| ());
        assert_eq!(
            refused,
            Err(AdmissionError::Refused(LocalRefusal {
                change: Change::Work(Work::Value, 1),
                current: 7,
                reason: RefusalReason::Limit { limit: 7 },
            }))
        );
        for dimension in Work::ALL {
            let expected = match dimension {
                Work::Value | Work::CodeUnit => 7,
                _ => 0,
            };
            assert_eq!(state.usage().work(*dimension), expected);
        }
        assert_eq!(state.usage().live(), 0);
        assert_eq!(state.usage().peak(), 0);
        Ok(())
    }

    #[test]
    fn work_overflow_precedes_cap_refusal_and_is_atomic() -> TestResult {
        let state = limited(u64::MAX - 1, 0);
        state.stage_work(Work::Value, u64::MAX - 1)?.commit();
        let before = state.usage();
        assert!(matches!(
            state.stage_work(Work::Value, 2),
            Err(AdmissionError::Refused(LocalRefusal {
                current,
                reason: RefusalReason::Overflow,
                ..
            })) if current == u64::MAX - 1
        ));
        assert_eq!(state.usage(), before);
        state.stage_work(Work::Argument, 1)?.commit();
        assert_eq!(state.usage().work(Work::Argument), 1);
        Ok(())
    }

    #[test]
    fn buffer_limits_bound_live_usage_and_preserve_peak_on_release() -> TestResult {
        let state = limited(0, 100);
        state.stage_buffer(100)?.commit();
        state.release(40)?;
        state.stage_buffer(40)?.commit();
        let refused = state.stage_buffer(1).map(|_| ());
        assert_eq!(
            refused,
            Err(AdmissionError::Refused(LocalRefusal {
                change: Change::Acquire(Resource::BufferBytes, 1),
                current: 100,
                reason: RefusalReason::Limit { limit: 100 },
            }))
        );
        assert_eq!(state.usage().live(), 100);
        state.release(100)?;
        assert_eq!(state.usage().live(), 0);
        assert_eq!(state.usage().peak(), 100);
        assert_eq!(state.usage().work(Work::Value), 0);
        Ok(())
    }

    #[test]
    fn buffer_overflow_precedes_cap_refusal_and_is_atomic() -> TestResult {
        let state = limited(0, u64::MAX - 1);
        state.stage_buffer(u64::MAX - 1)?.commit();
        let before = state.usage();
        assert!(matches!(
            state.stage_buffer(2),
            Err(AdmissionError::Refused(LocalRefusal {
                current,
                reason: RefusalReason::Overflow,
                ..
            })) if current == u64::MAX - 1
        ));
        assert_eq!(state.usage(), before);
        Ok(())
    }

    #[test]
    fn rejected_host_admission_discards_work_and_buffer_candidates() -> TestResult {
        let state = limited(10, 10);
        let before = state.usage();
        let work = state.stage_work(Work::Value, 5)?;
        {
            let _host = state.enter_host();
            assert_eq!(state.usage(), before);
        }
        drop(work);
        let buffer = state.stage_buffer(8)?;
        {
            let _host = state.enter_host();
            assert_eq!(state.usage(), before);
        }
        drop(buffer);
        assert_eq!(state.usage(), before);
        state.stage_work(Work::Value, 10)?.commit();
        state.stage_buffer(10)?.commit();
        assert_eq!(state.usage().work(Work::Value), 10);
        assert_eq!(state.usage().live(), 10);
        Ok(())
    }

    #[test]
    fn host_acceptance_promotes_prevalidated_usage_infallibly() -> TestResult {
        let state = limited(5, 10);
        let candidate = state.stage_work(Work::Value, 5)?;
        {
            let _host = state.enter_host();
            assert_eq!(state.usage().work(Work::Value), 0);
        }
        candidate.commit();
        assert_eq!(state.usage().work(Work::Value), 5);
        let candidate = state.stage_buffer(10)?;
        {
            let _host = state.enter_host();
            assert_eq!(state.usage().live(), 0);
        }
        candidate.commit();
        assert_eq!(state.usage().live(), 10);
        assert_eq!(state.usage().peak(), 10);
        Ok(())
    }

    #[test]
    fn accepted_acquisition_keeps_actual_release_and_candidate_peak() -> TestResult {
        let state = historical_buffer()?;
        let candidate = state.stage_buffer(50)?;
        {
            let _host = state.enter_host();
            state.release(20)?;
            assert_eq!(state.usage().live(), 40);
            assert_eq!(state.usage().peak(), 100);
        }
        candidate.commit();
        assert_eq!(state.usage().live(), 90);
        assert_eq!(state.usage().peak(), 110);
        Ok(())
    }

    #[test]
    fn rejected_acquisition_keeps_actual_release_and_accepted_peak() -> TestResult {
        let state = historical_buffer()?;
        let candidate = state.stage_buffer(50)?;
        {
            let _host = state.enter_host();
            state.release(20)?;
        }
        drop(candidate);
        assert_eq!(state.usage().live(), 40);
        assert_eq!(state.usage().peak(), 100);
        state.stage_buffer(60)?.commit();
        assert_eq!(state.usage().live(), 100);
        assert_eq!(state.usage().peak(), 100);
        Ok(())
    }

    #[test]
    fn work_promotion_keeps_actual_release_and_historical_peak() -> TestResult {
        let state = historical_buffer()?;
        let candidate = state.stage_work(Work::Value, 3)?;
        state.release(60)?;
        candidate.commit();
        assert_eq!(state.usage().work(Work::Value), 3);
        assert_eq!(state.usage().live(), 0);
        assert_eq!(state.usage().peak(), 100);
        Ok(())
    }

    #[test]
    fn release_underflow_cannot_spend_speculative_custody() -> TestResult {
        let state = historical_buffer()?;
        let candidate = state.stage_buffer(50)?;
        let before = state.usage();
        assert_eq!(
            state.release(61),
            Err(LocalRefusal {
                change: Change::Release(Resource::BufferBytes, 61),
                current: 60,
                reason: RefusalReason::Underflow,
            })
        );
        assert_eq!(state.usage(), before);
        candidate.commit();
        assert_eq!(state.usage().live(), 110);
        assert_eq!(state.usage().peak(), 110);
        Ok(())
    }

    #[test]
    fn refusal_after_real_release_keeps_accepted_custody() -> TestResult {
        let state = limited(0, 100);
        state.stage_buffer(100)?.commit();
        state.release(20)?;
        assert!(matches!(
            state.stage_buffer(21),
            Err(AdmissionError::Refused(_))
        ));
        assert_eq!(state.usage().live(), 80);
        assert_eq!(state.usage().peak(), 100);
        state.stage_buffer(20)?.commit();
        assert_eq!(state.usage().live(), 100);
        Ok(())
    }

    #[test]
    fn pending_admission_blocks_reentry_and_drop_reopens_it() -> TestResult {
        let state = limited(10, 10);
        let candidate = state.stage_buffer(5)?;
        assert_eq!(
            state.stage_work(Work::Value, 0).map(|_| ()),
            Err(AdmissionError::Reentrant)
        );
        assert_eq!(
            state.stage_buffer(0).map(|_| ()),
            Err(AdmissionError::Reentrant)
        );
        drop(candidate);
        state.stage_work(Work::Value, 1)?.commit();
        assert_eq!(state.usage().work(Work::Value), 1);
        Ok(())
    }

    #[test]
    fn phase_and_release_host_callbacks_block_admission_without_a_candidate() -> TestResult {
        let state = limited(10, 10);
        state.stage_buffer(10)?.commit();
        for released in [0, 5] {
            let host = state.enter_host();
            state.release(released)?;
            assert_eq!(
                state.stage_work(Work::Value, 1).map(|_| ()),
                Err(AdmissionError::Reentrant)
            );
            assert_eq!(
                state.stage_buffer(1).map(|_| ()),
                Err(AdmissionError::Reentrant)
            );
            assert_eq!(state.usage().peak(), 10);
            drop(host);
        }
        assert_eq!(state.usage().live(), 5);
        state.stage_work(Work::Value, 1)?.commit();
        Ok(())
    }

    #[test]
    fn nested_host_guards_restore_the_enclosing_gate() -> TestResult {
        let state = limited(10, 10);
        state.stage_buffer(10)?.commit();
        let outer = state.enter_host();
        {
            let _inner = state.enter_host();
            state.release(5)?;
        }
        assert_eq!(
            state.stage_buffer(1).map(|_| ()),
            Err(AdmissionError::Reentrant)
        );
        drop(outer);
        state.stage_buffer(5)?.commit();
        assert_eq!(state.usage().live(), 10);
        Ok(())
    }

    #[test]
    fn unwinding_cancels_candidate_and_restores_host_gate() -> TestResult {
        struct Unwind;
        impl Drop for Unwind {
            fn drop(&mut self) {
                std::panic::resume_unwind(Box::new(()));
            }
        }

        let state = historical_buffer()?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> TestResult {
            let _candidate = state.stage_buffer(50)?;
            let _host = state.enter_host();
            state.release(20)?;
            let _unwind = Unwind;
            Ok(())
        }));
        assert!(result.is_err());
        assert_eq!(state.usage().live(), 40);
        assert_eq!(state.usage().peak(), 100);
        state.stage_buffer(60)?.commit();
        assert_eq!(state.usage().live(), 100);
        Ok(())
    }

    #[test]
    fn zero_changes_are_neutral_and_refusal_getters_preserve_evidence() -> TestResult {
        let state = limited(0, 0);
        state.stage_work(Work::Value, 0)?.commit();
        state.stage_buffer(0)?.commit();
        state.release(0)?;
        assert_eq!(state.usage(), LocalUsage::zero());
        let refusal = state.release(1).err();
        assert!(matches!(refusal, Some(value) if
            value.change() == Change::Release(Resource::BufferBytes, 1)
                && value.current() == 0
                && value.reason() == RefusalReason::Underflow));
        assert_eq!(state.usage(), LocalUsage::zero());
        Ok(())
    }
}
