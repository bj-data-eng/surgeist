//! Layout work attribution. Layout owns the dimensions and their units; `surgeist-ledger` owns
//! scopes, reports and profiling.
//!
//! A layout call adds the volumes of its main algorithmic steps to a [`Meter`], a handle on
//! plain session-local counts. A charge site receives the handle through the input it already
//! takes, so no state is ambient. Counting makes no ledger call. A host that supplies a
//! [`LayoutHostScope`] receives the totals once, in one batch inside one phase that spans the
//! call; a call without a host makes no ledger call at all.

use std::borrow::Cow;
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
#[cfg(test)]
use std::{cell::RefCell, collections::BTreeMap};
use surgeist_ledger::{
    Change, Dimension, Error, NoDimension, PhaseObservation, Scope, SharedOperation, dimensions,
};

dimensions! {
    /// Work a layout call counts. Each dimension is one defined algorithm action, not CPU time.
    pub(crate) enum LayoutWork => LayoutWorkTable, LayoutWorkFields {
        /// One search for the best line from one start, in Text line fitting.
        LineSearch,
        /// One fitter run over one window of break opportunities.
        SequenceRun,
        /// One break opportunity visited by a line search, including those skipped because they
        /// end before the line start.
        OpportunityVisit,
        /// One break opportunity evaluated as a candidate line end.
        Candidate,
        /// One participant projected to a supplied-line descriptor.
        InlineProjection,
        /// One participant inspected while locating hard line ends.
        InlineBoundaryScan,
        /// One call into Text's leading-edge query, made once per next-line selection.
        InlineLeadingQuery,
        /// One supplied unit passed to the leading-edge query.
        InlineLeadingQueryUnit,
        /// One participant scanned while choosing a soft line end.
        InlineCandidateScan,
        /// One avoidance-class subset comparison.
        WrappingSubsetTest,
        /// One receipt offered to a preference frontier.
        WrappingOffer,
        /// One encounter event visited while replaying float placement.
        FloatReplayEvent,
        /// One float ledger entry evaluated by a band query.
        FloatLedgerEntry,
    }
}

type HostLedger = SharedOperation<LayoutWork, NoDimension>;

/// The number of work dimensions, which sizes the count table.
const WORK_DIMENSIONS: usize = <LayoutWork as Dimension>::ALL.len();

/// The scope of a host's operation that receives the totals of a layout call.
///
/// Cloning the value shares the operation.
#[derive(Clone)]
pub(crate) struct LayoutHostScope {
    ledger: HostLedger,
    scope: Scope,
}

impl LayoutHostScope {
    /// The scope `scope` of `ledger`, an operation of the layout schema that the host owns.
    #[cfg(test)]
    pub(crate) fn new(ledger: HostLedger, scope: Scope) -> Self {
        Self { ledger, scope }
    }
}

impl fmt::Debug for LayoutHostScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayoutHostScope")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

/// The window of break opportunities one balance group fits over.
pub(crate) type GroupWindow = (usize, usize);

/// Add-only counts, one cell per work dimension in declaration order.
struct Counts {
    work: [Cell<u64>; WORK_DIMENSIONS],
    /// The same counts by balance group, for tests that bound each group.
    #[cfg(test)]
    groups: RefCell<BTreeMap<GroupWindow, [u64; WORK_DIMENSIONS]>>,
}

impl Counts {
    fn new() -> Self {
        Self {
            work: [const { Cell::new(0) }; WORK_DIMENSIONS],
            #[cfg(test)]
            groups: RefCell::new(BTreeMap::new()),
        }
    }
}

/// The handle on the counts of one layout call. Clones add to the same counts.
///
/// Meters compare equal whatever they count: the counts observe a computation and are not part
/// of the identity of the input that carries them.
///
/// The counts are plain cells behind an `Rc`, so a meter and every session that holds one stay
/// on the thread of their layout call. No layout entry is bound to `Send`.
#[derive(Clone)]
pub(crate) struct Meter {
    counts: Rc<Counts>,
    #[cfg(test)]
    window: Option<GroupWindow>,
}

impl Meter {
    /// A meter on fresh, zero counts.
    pub(crate) fn new() -> Self {
        Self {
            counts: Rc::new(Counts::new()),
            #[cfg(test)]
            window: None,
        }
    }

    /// Adds `amount` units of `work`.
    pub(crate) fn charge(&self, work: LayoutWork, amount: usize) {
        let amount = u64::try_from(amount).unwrap_or(u64::MAX);
        if let Some(cell) = self.counts.work.get(work as usize) {
            cell.set(cell.get().saturating_add(amount));
        }
        #[cfg(test)]
        if let Some(window) = self.window {
            self.counts.charge_group(window, work, amount);
        }
    }

    /// Counts units of `work` locally and adds the total once when dropped, including on an
    /// early return, so a hot loop makes one add instead of one per unit.
    pub(crate) fn tally(&self, work: LayoutWork) -> Tally<'_> {
        Tally {
            meter: self,
            work,
            count: 0,
        }
    }

    /// The meter that receives the work of one balance group. In a test build the group also
    /// keeps its own counts, read through [`Meter::group_work`]; otherwise the meter is returned
    /// unchanged, so a call site is the same in both.
    #[cfg(not(test))]
    pub(crate) fn group(&self, _window: GroupWindow) -> Cow<'_, Self> {
        Cow::Borrowed(self)
    }

    #[cfg(test)]
    pub(crate) fn group(&self, window: GroupWindow) -> Cow<'_, Self> {
        Cow::Owned(Self {
            counts: Rc::clone(&self.counts),
            window: Some(window),
        })
    }

    /// Reports the non-zero totals to `host` in one batch.
    fn report(&self, host: &LayoutHostScope) -> Result<(), Error<LayoutWork, NoDimension>> {
        let mut batch = [Change::Work(LayoutWork::LineSearch, 0); WORK_DIMENSIONS];
        let mut len = 0;
        for (dimension, cell) in <LayoutWork as Dimension>::ALL
            .iter()
            .zip(self.counts.work.iter())
        {
            let total = cell.get();
            if total == 0 {
                continue;
            }
            if let Some(slot) = batch.get_mut(len) {
                *slot = Change::Work(*dimension, total);
                len += 1;
            }
        }
        host.ledger
            .apply_batch(host.scope, batch.get(..len).unwrap_or_default())
    }
}

/// One host phase around a layout call, so the host sees its inclusive time. It ends on return
/// and on unwinding.
struct HostPhase<'a> {
    host: &'a LayoutHostScope,
    observation: Option<PhaseObservation>,
}

impl<'a> HostPhase<'a> {
    /// Begins the phase. A host whose scope the ledger refuses sees no phase and the call
    /// proceeds unobserved, because accounting has no effect on geometry or errors.
    fn begin(host: &'a LayoutHostScope) -> Self {
        Self {
            host,
            observation: host.ledger.begin_phase(host.scope).ok(),
        }
    }
}

impl Drop for HostPhase<'_> {
    fn drop(&mut self) {
        if let Some(observation) = self.observation.take() {
            // The observation began on this operation, so ending it cannot find a foreign one.
            let _ = self.host.ledger.end_phase(observation);
        }
    }
}

/// Runs one layout call, which counts into the meter it receives.
///
/// With no host the call makes no ledger call. With a host, the call runs inside one host phase
/// and its totals reach the host's scope in one batch when it returns. The report is attribution
/// only and never changes a layout result: a refusal of the batch is the host's own cap, visible
/// in its own scope, and is ignored, as a refused phase is. The call's result is returned
/// unchanged.
pub(crate) fn with_host<T>(host: Option<&LayoutHostScope>, call: impl FnOnce(Meter) -> T) -> T {
    let meter = Meter::new();
    let Some(host) = host else {
        return call(meter);
    };
    let _phase = HostPhase::begin(host);
    let result = call(meter.clone());
    let _ = meter.report(host);
    result
}

impl fmt::Debug for Meter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Meter")
    }
}

impl PartialEq for Meter {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Units of one dimension counted by a loop and added when it ends.
pub(crate) struct Tally<'a> {
    meter: &'a Meter,
    work: LayoutWork,
    count: usize,
}

impl Tally<'_> {
    /// Counts one unit.
    pub(crate) fn one(&mut self) {
        self.add(1);
    }

    /// Counts `amount` units.
    pub(crate) fn add(&mut self, amount: usize) {
        self.count = self.count.saturating_add(amount);
    }
}

impl Drop for Tally<'_> {
    fn drop(&mut self) {
        self.meter.charge(self.work, self.count);
    }
}

#[cfg(test)]
impl Counts {
    fn charge_group(&self, window: GroupWindow, work: LayoutWork, amount: u64) {
        if let Ok(mut groups) = self.groups.try_borrow_mut()
            && let Some(cell) = groups.entry(window).or_default().get_mut(work as usize)
        {
            *cell = cell.saturating_add(amount);
        }
    }
}

#[cfg(test)]
impl Meter {
    /// A meter on fresh counts, read through [`Meter::work`].
    pub(crate) fn for_test() -> Self {
        Self::new()
    }

    /// The units of `work` counted so far.
    pub(crate) fn work(&self, work: LayoutWork) -> u64 {
        self.counts.work.get(work as usize).map_or(0, Cell::get)
    }

    /// The windows that have counted a group, in order.
    pub(crate) fn group_windows(&self) -> Vec<GroupWindow> {
        self.counts.groups.borrow().keys().copied().collect()
    }

    /// The units of `work` counted for the group of `window`.
    pub(crate) fn group_work(&self, window: GroupWindow, work: LayoutWork) -> u64 {
        self.counts
            .groups
            .borrow()
            .get(&window)
            .and_then(|counts| counts.get(work as usize))
            .copied()
            .unwrap_or(0)
    }
}

/// A host for tests: an operation of the layout schema and its root scope, whose usage tests
/// read after a layout call reported into it.
#[cfg(test)]
pub(crate) struct TestHost {
    ledger: HostLedger,
    scope: Scope,
}

#[cfg(test)]
impl TestHost {
    pub(crate) fn new() -> Self {
        let ledger = surgeist_ledger::Operation::with_profile(
            "layout test host",
            surgeist_ledger::Limits::unlimited(),
            1,
            surgeist_ledger::ProfileMode::Summary { timing: false },
        )
        .expect("a one-scope test operation is created")
        .share();
        let scope = ledger.root();
        Self { ledger, scope }
    }

    /// The scope a layout call reports into.
    pub(crate) fn scope(&self) -> LayoutHostScope {
        LayoutHostScope::new(self.ledger.clone(), self.scope)
    }

    /// The units of `work` the host's scope has received.
    pub(crate) fn work(&self, work: LayoutWork) -> u64 {
        self.ledger
            .usage(self.scope)
            .expect("the host scope of a live operation has usage")
            .work(work)
    }

    /// The phases the host's scope has seen end, one per reported call.
    pub(crate) fn phases(&self) -> u64 {
        self.ledger
            .snapshot()
            .ok()
            .flatten()
            .map_or(0, |report| report.scopes[0].phases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_dimensions_index_their_count_cells_in_declaration_order() {
        for (index, dimension) in <LayoutWork as Dimension>::ALL.iter().enumerate() {
            assert_eq!(*dimension as usize, index);
        }
        assert_eq!(WORK_DIMENSIONS, <LayoutWork as Dimension>::ALL.len());
    }

    #[test]
    fn charges_add_to_the_count_of_their_dimension() {
        let meter = Meter::for_test();
        meter.charge(LayoutWork::LineSearch, 2);
        meter.charge(LayoutWork::LineSearch, 3);
        meter.charge(LayoutWork::Candidate, 7);
        assert_eq!(meter.work(LayoutWork::LineSearch), 5);
        assert_eq!(meter.work(LayoutWork::Candidate), 7);
        assert_eq!(meter.work(LayoutWork::SequenceRun), 0);
    }

    #[test]
    fn clones_add_to_the_same_counts() {
        let meter = Meter::for_test();
        let clone = meter.clone();
        clone.charge(LayoutWork::WrappingOffer, 4);
        assert_eq!(meter.work(LayoutWork::WrappingOffer), 4);
    }

    #[test]
    fn separate_meters_keep_separate_counts() {
        let first = Meter::for_test();
        let second = Meter::for_test();
        first.charge(LayoutWork::FloatLedgerEntry, 9);
        assert_eq!(second.work(LayoutWork::FloatLedgerEntry), 0);
    }

    #[test]
    fn a_count_that_would_overflow_saturates_without_panicking() {
        let meter = Meter::for_test();
        meter.charge(LayoutWork::Candidate, usize::MAX);
        meter.charge(LayoutWork::Candidate, usize::MAX);
        meter.charge(LayoutWork::Candidate, usize::MAX);
        let expected = u64::try_from(usize::MAX).unwrap_or(u64::MAX);
        assert!(meter.work(LayoutWork::Candidate) >= expected);
        meter.charge(LayoutWork::Candidate, 1);
    }

    #[test]
    fn a_tally_adds_its_total_once_when_dropped() {
        let meter = Meter::for_test();
        {
            let mut tally = meter.tally(LayoutWork::InlineCandidateScan);
            tally.one();
            tally.one();
            tally.add(3);
            assert_eq!(meter.work(LayoutWork::InlineCandidateScan), 0);
        }
        assert_eq!(meter.work(LayoutWork::InlineCandidateScan), 5);
    }

    #[test]
    fn a_tally_adds_when_its_loop_returns_early() {
        fn scan(meter: &Meter, items: &[u32], stop: u32) -> Option<u32> {
            let mut tally = meter.tally(LayoutWork::OpportunityVisit);
            for &item in items {
                tally.one();
                if item == stop {
                    return Some(item);
                }
            }
            None
        }
        let meter = Meter::for_test();
        assert_eq!(scan(&meter, &[1, 2, 3, 4], 3), Some(3));
        assert_eq!(meter.work(LayoutWork::OpportunityVisit), 3);
    }

    #[test]
    fn a_tally_adds_when_its_loop_unwinds() {
        let meter = Meter::for_test();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut tally = meter.tally(LayoutWork::SequenceRun);
            tally.one();
            panic!("intentional unwind");
        }));
        assert!(result.is_err());
        assert_eq!(meter.work(LayoutWork::SequenceRun), 1);
    }

    #[test]
    fn meters_compare_equal_whatever_they_count() {
        let first = Meter::for_test();
        let second = Meter::for_test();
        first.charge(LayoutWork::LineSearch, 1);
        assert_eq!(first, second);
    }

    #[test]
    fn a_group_meter_counts_its_window_and_the_call() {
        let meter = Meter::for_test();
        meter.group((0, 4)).charge(LayoutWork::SequenceRun, 2);
        meter.group((0, 4)).charge(LayoutWork::SequenceRun, 1);
        meter.group((4, 9)).charge(LayoutWork::SequenceRun, 5);
        meter.charge(LayoutWork::SequenceRun, 7);
        assert_eq!(meter.group_windows(), [(0, 4), (4, 9)]);
        assert_eq!(meter.group_work((0, 4), LayoutWork::SequenceRun), 3);
        assert_eq!(meter.group_work((4, 9), LayoutWork::SequenceRun), 5);
        assert_eq!(meter.work(LayoutWork::SequenceRun), 15);
        assert_eq!(meter.group_work((0, 4), LayoutWork::Candidate), 0);
    }

    #[test]
    fn a_call_without_a_host_counts_locally_and_returns_its_result() {
        let result = with_host(None, |meter| {
            meter.charge(LayoutWork::LineSearch, 3);
            meter.work(LayoutWork::LineSearch)
        });
        assert_eq!(result, 3);
    }

    #[test]
    fn a_call_with_a_host_reports_its_totals_once_inside_one_phase() {
        let host = TestHost::new();
        let result = with_host(Some(&host.scope()), |meter| {
            meter.charge(LayoutWork::LineSearch, 2);
            meter.charge(LayoutWork::LineSearch, 3);
            meter.charge(LayoutWork::Candidate, 7);
            "done"
        });
        assert_eq!(result, "done");
        assert_eq!(host.work(LayoutWork::LineSearch), 5);
        assert_eq!(host.work(LayoutWork::Candidate), 7);
        assert_eq!(host.work(LayoutWork::SequenceRun), 0);
        assert_eq!(host.phases(), 1);
    }

    #[test]
    fn a_call_that_unwinds_ends_its_host_phase_and_reports_nothing() {
        let host = TestHost::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_host(Some(&host.scope()), |meter| {
                meter.charge(LayoutWork::LineSearch, 3);
                panic!("intentional unwind");
            })
        }));
        assert!(result.is_err());
        assert_eq!(host.phases(), 1);
        assert_eq!(host.work(LayoutWork::LineSearch), 0);
    }

    #[test]
    fn successive_calls_accumulate_in_the_host_scope() {
        let host = TestHost::new();
        for amount in [1, 2, 4] {
            with_host(Some(&host.scope()), |meter| {
                meter.charge(LayoutWork::WrappingSubsetTest, amount);
            });
        }
        assert_eq!(host.work(LayoutWork::WrappingSubsetTest), 7);
        assert_eq!(host.phases(), 3);
    }

    #[test]
    fn nested_calls_report_separately_into_the_host_scope() {
        let host = TestHost::new();
        with_host(Some(&host.scope()), |outer| {
            outer.charge(LayoutWork::LineSearch, 1);
            with_host(Some(&host.scope()), |inner| {
                inner.charge(LayoutWork::LineSearch, 10);
            });
            assert_eq!(outer.work(LayoutWork::LineSearch), 1);
        });
        assert_eq!(host.work(LayoutWork::LineSearch), 11);
    }

    #[test]
    fn a_host_scope_refused_by_a_limit_leaves_the_result_unchanged() {
        let ledger = surgeist_ledger::Operation::<LayoutWork, NoDimension>::new(
            "capped host",
            surgeist_ledger::Limits::new(
                LayoutWorkTable::new(LayoutWorkFields {
                    LineSearch: 1,
                    ..LayoutWorkFields::splat(u64::MAX)
                }),
                surgeist_ledger::NoTable::EMPTY,
            ),
            1,
        )
        .unwrap()
        .share();
        let scope = ledger.root();
        let host = LayoutHostScope::new(ledger.clone(), scope);
        let result = with_host(Some(&host), |meter| {
            meter.charge(LayoutWork::LineSearch, 5);
            meter.charge(LayoutWork::Candidate, 2);
            "unchanged"
        });
        assert_eq!(result, "unchanged");
        // The batch is atomic: the refused total leaves every dimension as it was.
        assert_eq!(ledger.usage(scope).unwrap().work(LayoutWork::LineSearch), 0);
        assert_eq!(ledger.usage(scope).unwrap().work(LayoutWork::Candidate), 0);
    }
}
