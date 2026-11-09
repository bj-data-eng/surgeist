//! Explicit caller-supplied processing for original Meta CSP transitions.
use crate::construction::SessionError;
use crate::parser::control::{HtmlParseInvariant, HtmlParseStop, ParseControl};
use crate::parser::tree_builder::parse::ReturnableCspStorage;
use crate::{HtmlDocumentParseRequest, HtmlParseFailure, HtmlParseReport};
use std::{
    cell::{Cell, RefCell},
    mem::size_of,
};
use surgeist_html::security::csp::{
    HtmlCspError, HtmlCspLayout, HtmlCspProcessorProfile, HtmlCspProtocolError, HtmlCspStorage,
    HtmlCspUnavailable, HtmlMetaCspOutcome, HtmlMetaCspPrepared, HtmlMetaCspProcessor,
    HtmlMetaCspRequest,
};

/// Original external failure; no owned native error is erased or stringified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseCspFailureKind {
    Refused,
    Failed,
    Cancelled,
    Protocol(HtmlCspProtocolError),
    Interrupted(HtmlParseStop),
    Parse,
}
/// A terminal consuming parse, its native error and original accounting owner.
/// Backend error custody cannot be extracted separately from this envelope.
pub struct HtmlParseCspFailure<E> {
    pub(crate) parse: HtmlParseFailure,
    pub(crate) native: Option<E>,
    pub(crate) kind: HtmlParseCspFailureKind,
    pub(crate) storage: ReturnableCspStorage,
}
impl<E> HtmlParseCspFailure<E> {
    /// Builds the envelope. Whether the failure can take the provider's storage
    /// back is settled here, while `cleanup_error` is still readable through
    /// the envelope, rather than when the envelope is dropped.
    pub(crate) fn new(
        mut parse: HtmlParseFailure,
        native: Option<E>,
        kind: HtmlParseCspFailureKind,
        storage: Storage,
    ) -> Self {
        let storage = parse.claim_csp_storage(storage);
        Self {
            parse,
            native,
            kind,
            storage,
        }
    }
    pub fn parse_failure(&self) -> &HtmlParseFailure {
        &self.parse
    }
    pub fn kind(&self) -> HtmlParseCspFailureKind {
        self.kind
    }
    pub fn backend_error(&self) -> Option<&E> {
        self.native.as_ref()
    }
}
impl<E> std::fmt::Debug for HtmlParseCspFailure<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlParseCspFailure")
            .field("kind", &self.kind)
            .field("parse", &self.parse)
            .field("backend_error_retained", &self.native.is_some())
            .finish()
    }
}
impl<E> std::fmt::Display for HtmlParseCspFailure<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTML Meta CSP parse failed: {:?}", self.kind)
    }
}
impl<E: std::error::Error + 'static> std::error::Error for HtmlParseCspFailure<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.backend_error() {
            Some(error) => Some(error),
            None => Some(self.parse_failure()),
        }
    }
}
impl<E> Drop for HtmlParseCspFailure<E> {
    fn drop(&mut self) {
        drop(self.native.take());
        // `new` claimed this storage only after the failure's controller was
        // seen to hold it, and nothing reaches that controller through the
        // envelope afterwards, so the credit is accepted. Were it not, the
        // refusal would be latched on a controller dropped with the failure.
        let _ = self.parse.return_csp_storage(self.storage);
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Storage {
    pub bytes: usize,
    pub units: usize,
    pub entries: usize,
}
impl Storage {
    pub const ZERO: Self = Self {
        bytes: 0,
        units: 0,
        entries: 0,
    };
    pub fn from(value: HtmlCspStorage) -> Self {
        Self {
            bytes: value.bytes(),
            units: value.utf16_units(),
            entries: value.entries(),
        }
    }
    pub fn add(self, b: Self) -> Option<Self> {
        Some(Self {
            bytes: self.bytes.checked_add(b.bytes)?,
            units: self.units.checked_add(b.units)?,
            entries: self.entries.checked_add(b.entries)?,
        })
    }
    pub fn sub(self, b: Self) -> Option<Self> {
        Some(Self {
            bytes: self.bytes.checked_sub(b.bytes)?,
            units: self.units.checked_sub(b.units)?,
            entries: self.entries.checked_sub(b.entries)?,
        })
    }
    fn max(self, b: Self) -> Self {
        Self {
            bytes: self.bytes.max(b.bytes),
            units: self.units.max(b.units),
            entries: self.entries.max(b.entries),
        }
    }
    fn saturating_sub(self, b: Self) -> Self {
        Self {
            bytes: self.bytes.saturating_sub(b.bytes),
            units: self.units.saturating_sub(b.units),
            entries: self.entries.saturating_sub(b.entries),
        }
    }
    /// The extra storage to admit for a call and the part of it that overlaps
    /// the retained result. Admission rises to the largest of the live,
    /// peak and retained aggregates in every dimension, so the overlap is
    /// that rise minus the retained aggregate and neither step can underflow.
    fn overlap(live: Self, peak: Self, next: Self) -> (Self, Self) {
        let admitted = live.max(peak).max(next);
        (admitted.saturating_sub(live), admitted.saturating_sub(next))
    }
    pub fn acquire(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        c.acquire_retained_provider(self.bytes as u64, self.units as u64, self.entries as u64)
    }
    /// Whether `c` holds this much admitted provider storage to credit.
    pub fn fits(self, c: &mut ParseControl) -> bool {
        c.can_release_retained_provider(self.bytes as u64, self.units as u64, self.entries as u64)
    }
    pub fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        let owner = c.identity();
        c.release_retained_provider(
            &owner,
            self.bytes as u64,
            self.units as u64,
            self.entries as u64,
        )
    }
}
/// Complete protocol-owned aggregate. Transferred occurrences already have a
/// receipt; only new storage and closing work are introduced.
#[derive(Default)]
pub(crate) struct Account {
    pub live: Storage,
    overlap: Storage,
    pub last: Storage,
    /// Whether the processor admitted or released through this account since
    /// the retained graph last set `live`. Until then the account differs from
    /// the graph only by the parser's own accounted calls.
    processor_touched: bool,
}
/// Who asks the account to admit storage. The account is shared by the
/// processor, through its admission callback, and by the parser's own calls to
/// the policy owner, and an account that cannot cover a call is a different
/// failure for each: the processor broke the accounting contract of the service
/// it was given, or the parser's own accounting disagrees with what it charged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Caller {
    Parser,
    Processor,
}
impl Account {
    /// Sets `live` to the storage the retained graph actually holds.
    pub fn rebaseline(&mut self, live: Storage) {
        self.live = live;
        self.processor_touched = false;
    }
    pub fn settle(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.overlap.release(c)?;
        self.overlap = Storage::ZERO;
        Ok(())
    }
    pub fn admit(
        &mut self,
        c: &mut ParseControl,
        a: HtmlCspLayout,
        caller: Caller,
    ) -> Result<(), HtmlParseStop> {
        self.processor_touched |= caller == Caller::Processor;
        self.settle(c)?;
        a.charge_work(c).map_err(|error| match error {
            surgeist_html::accounting::HtmlChargedError::Stopped(stop) => stop,
            surgeist_html::accounting::HtmlChargedError::Faulted(_) => c.charge_overflow_stop(),
        })?;
        c.admit_atomic_layout(a.atomic_bytes())?;
        let base = self
            .live
            .sub(Storage::from(a.transferred().storage()))
            .ok_or_else(|| match (caller, self.processor_touched) {
                // The account differs from the graph only by what the
                // processor did, once it has acted, whoever finds the gap.
                (Caller::Parser, false) => c.fail_invariant(HtmlParseInvariant::CapacityMismatch),
                (Caller::Parser, true) | (Caller::Processor, _) => c.csp_processing_stop(),
            })?;
        let peak = base
            .add(Storage::from(a.peak()))
            .ok_or_else(|| c.stop_representation(crate::HtmlParseResource::ManagedCapacityBytes))?;
        let next = base
            .add(Storage::from(a.retained().storage()))
            .ok_or_else(|| c.stop_representation(crate::HtmlParseResource::ManagedCapacityBytes))?;
        // Temporarily admit the declared overlap before the native action.
        let (extra, overlap) = Storage::overlap(self.live, peak, next);
        extra.acquire(c)?;
        self.overlap = overlap;
        self.live = next;
        self.last = Storage::from(a.retained().storage());
        Ok(())
    }
    /// Only the processor's release callback releases through the account, so
    /// storage it never held is the processor's accounting failing.
    pub fn release(
        &mut self,
        c: &mut ParseControl,
        v: HtmlCspStorage,
    ) -> Result<(), HtmlParseStop> {
        self.processor_touched = true;
        self.settle(c)?;
        let n = Storage::from(v);
        self.live = self.live.sub(n).ok_or_else(|| c.csp_processing_stop())?;
        n.release(c)?;
        Ok(())
    }
}
pub(crate) enum Outcome {
    Completed(HtmlMetaCspPrepared),
    Unavailable(HtmlCspUnavailable),
    Terminal,
}
pub(crate) trait Service {
    fn profile(&self) -> &HtmlCspProcessorProfile;
    fn call(
        &mut self,
        c: &mut ParseControl,
        account: &mut Account,
        request: &HtmlMetaCspRequest,
    ) -> Result<Outcome, SessionError>;
}
/// The controller and aggregate the processor's admission and release
/// callbacks act on. The two callbacks are separate closures, so they share
/// this state through one cell.
struct Shared<'c, 'a> {
    state: RefCell<(&'c mut ParseControl, &'a mut Account)>,
    release_error: Cell<Option<HtmlParseStop>>,
    reentered: Cell<bool>,
}
impl<'c, 'a> Shared<'c, 'a> {
    fn new(c: &'c mut ParseControl, account: &'a mut Account) -> Self {
        Self {
            state: RefCell::new((c, account)),
            release_error: Cell::new(None),
            reentered: Cell::new(false),
        }
    }
    /// The stop handed back to a processor that entered a callback while
    /// another still held the state. The controller is out of reach there, so
    /// `finish` latches it as the first stop once the processor has returned.
    fn refuse_reentry(&self) -> HtmlParseStop {
        self.reentered.set(true);
        HtmlParseStop::InternalInvariant(HtmlParseInvariant::ReentrantProviderCallback)
    }
    fn admit(&self, layout: HtmlCspLayout) -> Result<(), HtmlParseStop> {
        let Ok(mut state) = self.state.try_borrow_mut() else {
            return Err(self.refuse_reentry());
        };
        let (c, account) = &mut *state;
        account.admit(c, layout, Caller::Processor)
    }
    fn release(&self, storage: HtmlCspStorage) {
        let Ok(mut state) = self.state.try_borrow_mut() else {
            self.refuse_reentry();
            return;
        };
        let (c, account) = &mut *state;
        if let Err(e) = account.release(c, storage) {
            self.release_error.set(Some(e));
        }
    }
    /// Hands the controller and aggregate back once the processor has returned.
    fn finish(self) -> Result<(&'c mut ParseControl, &'a mut Account), SessionError> {
        let reentered = self.reentered.get();
        let release_error = self.release_error.get();
        let (c, account) = self.state.into_inner();
        if reentered {
            return Err(c
                .fail_invariant(HtmlParseInvariant::ReentrantProviderCallback)
                .into());
        }
        match release_error {
            Some(e) => Err(e.into()),
            None => Ok((c, account)),
        }
    }
}
/// A processor that returned without a usable result while the account holds
/// storage it admitted, or that released more than it held, broke the
/// accounting contract of the service it was given: its processing failed.
/// Every such violation ends the parse as a failed processing, wherever it is
/// found: when the processor releases (`Account::release`), when it asks for a
/// transfer its account cannot cover (`Account::admit`), when it returns, and
/// when the retained graph is reconciled against its account afterwards.
pub(crate) fn accounting_violation(c: &mut ParseControl) -> SessionError {
    c.csp_processing_stop();
    SessionError::CspService
}
struct Adapter<'a, P: HtmlMetaCspProcessor> {
    processor: &'a mut P,
    error: Option<P::Error>,
    kind: HtmlParseCspFailureKind,
    storage: Storage,
}
impl<P: HtmlMetaCspProcessor> Service for Adapter<'_, P> {
    fn profile(&self) -> &HtmlCspProcessorProfile {
        self.processor.profile()
    }
    fn call(
        &mut self,
        c: &mut ParseControl,
        account: &mut Account,
        request: &HtmlMetaCspRequest,
    ) -> Result<Outcome, SessionError> {
        let before = account.live;
        let shared = Shared::new(c, account);
        let result = self
            .processor
            .prepare(request, &mut |a| shared.admit(a), &mut |s| {
                shared.release(s)
            });
        let (c, account) = shared.finish()?;
        account.settle(c)?;
        let refused = matches!(&result, Ok(HtmlMetaCspOutcome::Refused(_)));
        match result {
            Ok(HtmlMetaCspOutcome::Completed(p)) => Ok(Outcome::Completed(p)),
            Ok(HtmlMetaCspOutcome::Refused(e)) | Ok(HtmlMetaCspOutcome::Failed(e)) => {
                // The provider's remaining aggregate travels with the real E.
                self.storage = account
                    .live
                    .sub(before)
                    .ok_or_else(|| accounting_violation(c))?;
                account.live = before;
                self.error = Some(e);
                self.kind = if refused {
                    HtmlParseCspFailureKind::Refused
                } else {
                    HtmlParseCspFailureKind::Failed
                };
                // Set below through the outcome discriminant before moving E.
                c.csp_processing_stop();
                Ok(Outcome::Terminal)
            }
            Ok(HtmlMetaCspOutcome::Unavailable(u)) => {
                if account.live != before {
                    return Err(accounting_violation(c));
                }
                Ok(Outcome::Unavailable(u))
            }
            Ok(HtmlMetaCspOutcome::Cancelled) => {
                // Cancellation is what ended the processing, whatever storage
                // the processor still holds.
                self.kind = HtmlParseCspFailureKind::Cancelled;
                Err(c.cancelled_stop().into())
            }
            Err(HtmlCspError::Protocol(p)) => {
                self.kind = HtmlParseCspFailureKind::Protocol(p);
                c.csp_protocol_stop(p);
                Err(SessionError::CspProtocol(p))
            }
            Err(HtmlCspError::Interrupted(e)) => {
                self.kind = HtmlParseCspFailureKind::Interrupted(e);
                Err(e.into())
            }
        }
    }
}
#[expect(
    clippy::result_large_err,
    reason = "returns the actual original request, controller and native error custody without allocating after a stop"
)]
pub(crate) fn parse<P: HtmlMetaCspProcessor>(
    request: HtmlDocumentParseRequest,
    processor: &mut P,
) -> Result<HtmlParseReport, HtmlParseCspFailure<P::Error>> {
    let mut adapter = Adapter {
        processor,
        error: None,
        kind: HtmlParseCspFailureKind::Parse,
        storage: Storage::ZERO,
    };
    let fixed =
        size_of::<Option<P::Error>>().saturating_add(size_of::<HtmlParseCspFailure<P::Error>>());
    match crate::parser::tree_builder::parse::document(request, Some(&mut adapter), fixed) {
        Ok(report) => Ok(report),
        Err(parse) => {
            let kind = match parse.reason() {
                crate::HtmlParseFailureReason::CspProtocol(p) => {
                    HtmlParseCspFailureKind::Protocol(p)
                }
                _ => adapter.kind,
            };
            Err(HtmlParseCspFailure::new(
                parse,
                adapter.error,
                kind,
                adapter.storage,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlParseCancellation;
    use surgeist_html::security::csp::{HtmlCspDisposalWork, HtmlCspLayout, HtmlCspStorage};

    fn limits() -> crate::HtmlParseLimits {
        crate::HtmlParseLimits::builder()
            .source_units(1_000)
            .buffered_utf16_units(1_000)
            .token_attributes(1_000)
            .auxiliary_entries(1_000)
            .active_formatting_entries(1_000)
            .managed_nodes(1_000)
            .managed_edges(1_000)
            .managed_attributes(1_000)
            .managed_utf16_units(1_000)
            .source_map_entries(1_000)
            .source_map_payload_units(1_000)
            .diagnostics(1_000)
            .diagnostic_payload_units(1_000)
            .context_effects(1_000)
            .open_elements(1_000)
            .tree_depth(1_000)
            .managed_capacity_bytes(16_000)
            .peak_transient_bytes(32_000)
            .atomic_layout_bytes(16_000)
            .work_credits(100_000)
            .finish()
            .unwrap()
    }
    fn control() -> ParseControl {
        ParseControl::new(limits(), HtmlParseCancellation::new())
    }
    fn layout() -> HtmlCspLayout {
        HtmlCspLayout::processing(
            3,
            HtmlCspStorage::default(),
            HtmlCspDisposalWork::default(),
            8,
        )
        .unwrap()
    }

    fn storage(bytes: usize, units: usize, entries: usize) -> Storage {
        Storage {
            bytes,
            units,
            entries,
        }
    }

    #[test]
    fn overlap_admits_the_largest_aggregate_per_dimension_and_keeps_the_excess_over_the_result() {
        // Hand-computed per dimension: admitted = max(live, peak, next).
        // bytes: 30, units: 9, entries: 2.
        let (extra, overlap) =
            Storage::overlap(storage(10, 5, 2), storage(30, 4, 2), storage(20, 9, 1));
        assert_eq!(extra, storage(20, 4, 0));
        assert_eq!(overlap, storage(10, 0, 1));
        // A result larger than the peak is admitted and leaves no overlap.
        let (extra, overlap) =
            Storage::overlap(storage(1, 1, 1), storage(2, 2, 2), storage(7, 7, 7));
        assert_eq!(extra, storage(6, 6, 6));
        assert_eq!(overlap, Storage::ZERO);
        // Nothing beyond the live aggregate is admitted when it already covers both.
        let (extra, overlap) =
            Storage::overlap(storage(9, 9, 9), storage(3, 3, 3), storage(4, 4, 4));
        assert_eq!(extra, Storage::ZERO);
        assert_eq!(overlap, storage(5, 5, 5));
    }

    #[test]
    fn a_callback_entered_while_another_runs_stops_the_parse_without_panicking() {
        let mut c = control();
        let mut account = Account::default();
        let before = c.usage().clone();
        let shared = Shared::new(&mut c, &mut account);
        {
            // A processor cannot reach this through the closures it is given:
            // neither closure calls out while it holds the state. The guard
            // stands in for foreign code still running inside one callback.
            let _running = shared.state.borrow_mut();
            assert_eq!(
                shared.admit(layout()),
                Err(HtmlParseStop::InternalInvariant(
                    HtmlParseInvariant::ReentrantProviderCallback
                ))
            );
            shared.release(HtmlCspStorage::default());
        }
        let stop = HtmlParseStop::InternalInvariant(HtmlParseInvariant::ReentrantProviderCallback);
        assert!(matches!(shared.finish(), Err(SessionError::Stop(s)) if s == stop));
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(c.usage(), before);
        assert_eq!(account.live, Storage::ZERO);
        assert_eq!(account.last, Storage::ZERO);
    }

    #[test]
    fn callbacks_run_in_turn_without_reporting_reentry() {
        let mut c = control();
        let mut account = Account::default();
        let shared = Shared::new(&mut c, &mut account);
        assert_eq!(shared.admit(layout()), Ok(()));
        shared.release(HtmlCspStorage::default());
        assert!(shared.finish().is_ok());
        assert_eq!(c.stop(), None);
    }

    #[test]
    fn storage_a_failure_cannot_take_back_is_reported_before_the_envelope_is_dropped() {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8("<p>x"), limits());
        request.cancellation().cancel();
        let parse = request.parse().unwrap_err();
        assert_eq!(parse.cleanup_error(), None);
        let failure = HtmlParseCspFailure::<()>::new(
            parse,
            None,
            HtmlParseCspFailureKind::Cancelled,
            Storage {
                bytes: 8,
                units: 2,
                entries: 1,
            },
        );
        // The caller can read why the storage was not released, and the
        // envelope holds nothing for its drop to hand back.
        assert_eq!(
            failure.parse_failure().cleanup_error(),
            Some(crate::HtmlParseEngineCondition::Ownership)
        );
        assert_eq!(failure.storage, ReturnableCspStorage::NONE);
        assert_eq!(
            failure.parse_failure().reason(),
            crate::HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled)
        );
        drop(failure);
    }

    #[test]
    fn no_storage_leaves_a_failure_without_a_cleanup_report() {
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8("<p>x"), limits());
        request.cancellation().cancel();
        let parse = request.parse().unwrap_err();
        let failure = HtmlParseCspFailure::<()>::new(
            parse,
            None,
            HtmlParseCspFailureKind::Cancelled,
            Storage::ZERO,
        );
        assert_eq!(failure.parse_failure().cleanup_error(), None);
        assert_eq!(failure.storage, ReturnableCspStorage::NONE);
    }

    #[test]
    fn storage_the_controller_never_admitted_is_reported_and_dropping_the_envelope_is_safe() {
        let tight = crate::parser::tree_builder::tests::limits(100)
            .into_builder()
            .managed_nodes(4)
            .finish()
            .unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8("<p>x</p>"), tight);
        let parse = request.parse().unwrap_err();
        let retained = parse.retained_usage();
        let failure = HtmlParseCspFailure::<()>::new(
            parse,
            None,
            HtmlParseCspFailureKind::Cancelled,
            Storage {
                bytes: 8,
                units: 2,
                entries: 1,
            },
        );
        // The caller can read why the storage was not released, and the
        // envelope holds nothing for its drop to hand back.
        assert_eq!(
            failure.parse_failure().cleanup_error(),
            Some(crate::HtmlParseEngineCondition::Cleanup)
        );
        assert_eq!(failure.storage, ReturnableCspStorage::NONE);
        assert_eq!(failure.parse_failure().retained_usage(), retained);
        drop(failure);
    }
}
