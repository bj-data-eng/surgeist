//! CSS-owned property graphs retained under the original parser controller.
use super::*;
use crate::parser::buffer::Utf16Buffer;
use std::cell::{Cell, RefCell};
use surgeist_css::component::CssComponentValueLimits;
use surgeist_css::source::{
    CssUtf16InputLimits, CssUtf16PropertyValueLimits, CssUtf16ValueAdmission as Admission,
    CssUtf16ValueStorage as Storage,
};
use surgeist_css::values::color_adjust::{CssColorScheme, CssColorSchemeParseReport};
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

/// One admitted immutable provider occurrence. This never enters the string pool.
struct ProviderReceipt {
    owner: ControlIdentity,
    bytes: u64,
    units: u64,
}
impl ProviderReceipt {
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        c.release_retained_provider(&self.owner, self.bytes, self.units, 0)
    }
}

pub(crate) struct CssSchemeValue {
    report: Arc<CssColorSchemeParseReport>,
    receipt: ProviderReceipt,
}
impl CssSchemeValue {
    pub(crate) fn report(&self) -> &Arc<CssColorSchemeParseReport> {
        &self.report
    }
    pub(crate) fn require(&self, c: &mut ParseControl) -> Result<(), ParseStop> {
        c.require_identity(&self.receipt.owner)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let usage = self.report.retained_storage();
        if self.receipt.bytes != usage.bytes() as u64
            || self.receipt.units != usage.utf16_units() as u64
            || c.usage
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                < self.receipt.bytes
            || c.usage
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current
                < self.receipt.units
        {
            return Err(c.fail_invariant(Invariant::CapacityMismatch));
        }
        Ok(())
    }
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let Self { report, receipt } = self;
        drop(report);
        receipt.release(c)
    }
}

/// A state the CSS parser cannot produce, forced into the next scheme parse.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SchemeFault {
    /// Every release callback finds the state already in use.
    ReentrantRelease,
    /// The storage admitted differs from the storage the report retains.
    LiveDrift,
}
/// The controller and live-storage cell the CSS parser's admission and release
/// callbacks act on. The two callbacks are separate closures, so they share
/// this state through one cell. A callback that finds the state in use, or a
/// release the ledger cannot honour, cannot reach the controller to latch the
/// stop; it records it for [`Self::finish`].
struct SchemeCallbacks<'c> {
    #[cfg(test)]
    fault: Option<SchemeFault>,
    control: RefCell<&'c mut ParseControl>,
    live: Cell<(u64, u64)>,
    owner: ControlIdentity,
    reentered: Cell<bool>,
    release_error: Cell<Option<ParseStop>>,
}
/// What the callbacks leave behind once the parser has returned.
struct Finished<'c> {
    control: &'c mut ParseControl,
    /// The storage still admitted for the parser's retained graph.
    live: (u64, u64),
    owner: ControlIdentity,
    /// The first refusal of a callback, already latched on `control`.
    refusal: Option<ParseStop>,
}
impl<'c> SchemeCallbacks<'c> {
    fn new(control: &'c mut ParseControl) -> Self {
        let owner = control.identity();
        Self {
            #[cfg(test)]
            fault: control.css_scheme_fault.take(),
            control: RefCell::new(control),
            live: Cell::new((0, 0)),
            owner,
            reentered: Cell::new(false),
            release_error: Cell::new(None),
        }
    }
    /// The stop handed to a parser that entered a callback while another still
    /// held the controller. The controller is out of reach there, so `finish`
    /// latches it once the parser has returned.
    fn refuse_reentry(&self) -> ParseStop {
        self.reentered.set(true);
        ParseStop::InternalInvariant(Invariant::ReentrantProviderCallback)
    }
    fn admit(&self, event: Admission) -> Result<(), ParseStop> {
        let live = if let Admission::Live(next) = event {
            Some((next.bytes() as u64, next.utf16_units() as u64))
        } else {
            None
        };
        self.admit_live(live)
    }
    /// One admission event: its step, and for a live-storage observation the
    /// growth over the storage already admitted.
    fn admit_live(&self, next: Option<(u64, u64)>) -> Result<(), ParseStop> {
        let Ok(mut c) = self.control.try_borrow_mut() else {
            return Err(self.refuse_reentry());
        };
        c.charge(WorkCategory::StateStep, 1)?;
        if let Some(next) = next {
            let old = self.live.get();
            let bytes = next
                .0
                .checked_sub(old.0)
                .ok_or_else(|| c.fail_invariant(Invariant::ReleaseUnderflow))?;
            let units = next
                .1
                .checked_sub(old.1)
                .ok_or_else(|| c.fail_invariant(Invariant::ReleaseUnderflow))?;
            c.acquire_retained_provider(bytes, units, 0)?;
            self.live.set(next);
        }
        Ok(())
    }
    fn release(&self, remaining: Storage) {
        self.release_to((remaining.bytes() as u64, remaining.utf16_units() as u64));
    }
    /// Credits the storage the parser dropped, leaving `next` live. The parser
    /// has no way to take a refusal back, so it is kept for `finish`; the
    /// ledger is left as it was.
    fn release_to(&self, next: (u64, u64)) {
        #[cfg(test)]
        if self.fault == Some(SchemeFault::ReentrantRelease) {
            self.refuse_reentry();
            return;
        }
        let Ok(mut c) = self.control.try_borrow_mut() else {
            self.refuse_reentry();
            return;
        };
        let old = self.live.get();
        let released = match old.0.checked_sub(next.0).zip(old.1.checked_sub(next.1)) {
            Some((bytes, units)) => c.release_retained_provider(&self.owner, bytes, units, 0),
            None => c.fail(ParseStop::InternalInvariant(Invariant::ReleaseUnderflow)),
        };
        match released {
            Ok(()) => self.live.set(next),
            Err(stop) => {
                self.release_error
                    .set(self.release_error.get().or(Some(stop)));
            }
        }
    }
    /// Hands the controller back once the parser has returned, latching a
    /// reentry the callbacks could not.
    fn finish(self) -> Finished<'c> {
        let reentered = self.reentered.get();
        let release_error = self.release_error.get();
        let live = self.live.get();
        let control = self.control.into_inner();
        #[cfg(test)]
        let live = {
            let mut live = live;
            if self.fault == Some(SchemeFault::LiveDrift)
                && control.acquire_retained_provider(1, 0, 0).is_ok()
            {
                live.0 += 1;
            }
            live
        };
        let refusal = if reentered {
            Some(control.fail_invariant(Invariant::ReentrantProviderCallback))
        } else {
            release_error
        };
        Finished {
            control,
            live,
            owner: self.owner,
            refusal,
        }
    }
}

impl ParseControl {
    fn remaining_css(&mut self, resource: HtmlParseResource) -> Result<usize, ParseStop> {
        self.checkpoint()?;
        let remaining = self
            .limits
            .copy(resource)
            .checked_sub(self.usage.resource(resource).current)
            .ok_or_else(|| self.fail_invariant(Invariant::ReleaseUnderflow))?;
        Ok(usize::try_from(remaining).unwrap_or(usize::MAX))
    }
    fn css_scheme_limits(
        &mut self,
    ) -> Result<(CssUtf16PropertyValueLimits, CssComponentValueLimits), ParseStop> {
        let bytes = self
            .remaining_css(HtmlParseResource::ManagedCapacityBytes)?
            .min(self.remaining_css(HtmlParseResource::PeakTransientBytes)?);
        let units = self.remaining_css(HtmlParseResource::ManagedUtf16Units)?;
        let work = self.remaining_css(HtmlParseResource::WorkCredits)?;
        let entries = self.remaining_css(HtmlParseResource::AuxiliaryEntries)?;
        // AtomicLayoutBytes observes a maximum, rather than a consumed extent.
        let scratch = usize::try_from(self.limits.copy(HtmlParseResource::AtomicLayoutBytes))
            .unwrap_or(usize::MAX);
        // CSS caps nesting at 256 levels; a depth within it is always accepted.
        const DEPTH_CEILING: u32 = 256;
        let depth = u32::try_from(entries.min(DEPTH_CEILING as usize)).unwrap_or(DEPTH_CEILING);
        let Some(syntax) = CssComponentValueLimits::try_new(depth, entries, bytes) else {
            return Err(self.fail_invariant(Invariant::CapacityMismatch));
        };
        let limits = CssUtf16PropertyValueLimits {
            input: CssUtf16InputLimits {
                max_input_units: units,
                max_output_bytes: bytes,
                max_map_entries: entries,
                max_checkpoints: entries,
                max_scratch_bytes: scratch,
                max_retained_bytes: bytes,
                max_work: work,
            },
            max_work: work,
            max_live_bytes: bytes,
            max_retained_bytes: bytes,
        };
        Ok((limits, syntax))
    }
    /// Copy through the existing bounded buffer, then move its unique ticket
    /// into provider custody. This cannot alias a registered construction string.
    fn css_scheme_input(
        &mut self,
        raw: &[u16],
    ) -> Result<(Arc<Vec<u16>>, ProviderReceipt), ParseStop> {
        let mut buffer = Utf16Buffer::new(self)?;
        if let Err(stop) = buffer.extend_units(raw, self) {
            buffer.release(self)?;
            return Err(stop);
        }
        let frozen = buffer.freeze(self)?;
        let prepared = (|| {
            self.require_document_string(frozen.allocation(), frozen.text())?;
            let units = frozen.text().shared_code_units().capacity() as u64;
            let bytes = units
                .checked_mul(size_of::<u16>() as u64)
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>() as u64))
                .ok_or_else(|| self.stop_representation(HtmlParseResource::ManagedCapacityBytes))?;
            self.charge(WorkCategory::CopyByte, size_of::<Arc<Vec<u16>>>() as u64)?;
            // Last fallible operation; the existing backing remains admitted.
            self.acquire(HtmlParseResource::ManagedUtf16Units, units)?;
            Ok((bytes, units))
        })();
        let (bytes, units) = match prepared {
            Ok(value) => value,
            Err(stop) => {
                frozen.release(self)?;
                return Err(stop);
            }
        };
        let (text, allocation) = frozen.into_parts();
        let input = text.shared_code_units().clone();
        drop(text);
        if let Err(stop) = self.commit_prepared_release(&[(allocation.resource, allocation.units)])
        {
            // The ticket was qualified above, so this refusal needs a ledger
            // that no longer matches it. Withdraw the units acquired for the
            // receipt; the stop is latched.
            drop(allocation);
            drop(input);
            let _ = self.commit_prepared_release(&[(HtmlParseResource::ManagedUtf16Units, units)]);
            return Err(stop);
        }
        drop(allocation); // The exact original debit moves into this receipt.
        Ok((
            input,
            ProviderReceipt {
                owner: self.identity(),
                bytes,
                units,
            },
        ))
    }
    /// Original-controller property ingress. Scratch and selected output share
    /// the same cumulative work and actual provider-capacity ledger.
    pub(crate) fn parse_meta_color_scheme(
        &mut self,
        raw: &[u16],
        mode: crate::HtmlDocumentMode,
    ) -> Result<CssSchemeValue, ParseStop> {
        let (input, input_receipt) = self.css_scheme_input(raw)?;
        let result = (|| {
            let (limits, syntax) = self.css_scheme_limits()?;
            let callbacks = SchemeCallbacks::new(&mut *self);
            let context = CssParserContext::new(if mode == crate::HtmlDocumentMode::Quirks {
                CssParserMode::Quirks
            } else {
                CssParserMode::Standards
            });
            let result = CssColorScheme::parse_utf16(
                &CssParser::new(context, syntax),
                input.clone(),
                limits,
                |event| callbacks.admit(event),
                |remaining| callbacks.release(remaining),
            );
            let Finished {
                control: c,
                live,
                owner: original,
                refusal,
            } = callbacks.finish();
            if let Some(stop) = refusal {
                // A callback was refused while the parser ran. Drop its
                // result and credit what is still admitted for it; the stop
                // is latched, so a refusal here changes nothing.
                drop(result);
                let _ = c.release_retained_provider(&original, live.0, live.1, 0);
                return Err(stop);
            }
            let report = match result {
                Ok(report) => report,
                Err(error) => {
                    let refusal = match error.into_interruption() {
                        Ok(stop) => return Err(stop),
                        Err(refusal) => refusal,
                    };
                    if let Some(resource) = refusal.exhausted_limit() {
                        return c.fail(ParseStop::CssPropertyLimit { resource });
                    }
                    if refusal.is_allocation_failure() {
                        return c.fail(ParseStop::CssPropertyAllocationFailed);
                    }
                    // A capacity overflow.
                    return c.representation(HtmlParseResource::ManagedCapacityBytes);
                }
            };
            let usage = report.retained_storage();
            if live != (usage.bytes() as u64, usage.utf16_units() as u64) {
                // The parser returns exactly the storage it kept admitted.
                drop(report);
                let stop = c.fail_invariant(Invariant::CapacityMismatch);
                let _ = c.release_retained_provider(&original, live.0, live.1, 0);
                return Err(stop);
            }
            // Admit the heap report body while the inline report is still live.
            let header = size_of::<CssColorSchemeParseReport>() as u64;
            if let Err(stop) = c
                .charge(WorkCategory::CopyByte, header)
                .and_then(|()| c.acquire_retained_provider(header, 0, 0))
            {
                drop(report);
                // The stop is latched; a refused credit here changes nothing.
                let _ = c.release_retained_provider(
                    &original,
                    usage.bytes() as u64,
                    usage.utf16_units() as u64,
                    0,
                );
                return Err(stop);
            }
            let report = Arc::new(report);
            if let Err(stop) = c.release_retained_provider(&original, header, 0, 0) {
                drop(report);
                let _ = c.release_retained_provider(
                    &original,
                    usage.bytes() as u64,
                    usage.utf16_units() as u64,
                    0,
                );
                return Err(stop);
            }
            Ok(CssSchemeValue {
                report,
                receipt: ProviderReceipt {
                    owner: original,
                    bytes: usage.bytes() as u64,
                    units: usage.utf16_units() as u64,
                },
            })
        })();
        drop(input);
        match (result, input_receipt.release(self)) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(stop), _) => Err(stop),
            (Ok(value), Err(stop)) => {
                // The stop is latched; the value's own refusal would add nothing.
                let _ = value.release(self);
                Err(stop)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> crate::HtmlParseLimits {
        crate::parser::tree_builder::tests::limits(1000)
    }
    fn empty(c: &ParseControl) {
        for resource in [
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::PeakTransientBytes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::AuxiliaryEntries,
        ] {
            assert_eq!(c.usage().resource(resource).current(), 0, "{resource:?}");
        }
    }
    #[test]
    fn selected_css_receipt_is_the_only_retained_graph_and_drop_refunds_exact_original_owner() {
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        let value = c
            .parse_meta_color_scheme(
                &"only dark".encode_utf16().collect::<Vec<_>>(),
                crate::HtmlDocumentMode::NoQuirks,
            )
            .unwrap();
        value.require(&mut c).unwrap();
        let usage = value.report().retained_storage();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current(),
            usage.bytes() as u64
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current(),
            usage.utf16_units() as u64
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current(),
            0
        );
        value.release(&mut c).unwrap();
        empty(&c);
        assert!(c.usage().resource(HtmlParseResource::WorkCredits).current() > 0);
    }
    #[test]
    fn one_short_real_css_peak_refuses_without_leaking_input_scratch_or_report() {
        let raw: Vec<_> = "only dark".encode_utf16().collect();
        let mut baseline = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        let value = baseline
            .parse_meta_color_scheme(&raw, crate::HtmlDocumentMode::NoQuirks)
            .unwrap();
        let peak = baseline.usage().actual_capacity_high_water();
        value.release(&mut baseline).unwrap();
        empty(&baseline);
        let bounded = limits()
            .into_builder()
            .managed_capacity_bytes(peak - 1)
            .peak_transient_bytes(peak - 1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(bounded, crate::HtmlParseCancellation::new());
        let error = match c.parse_meta_color_scheme(&raw, crate::HtmlDocumentMode::NoQuirks) {
            Err(error) => error,
            Ok(value) => {
                value.release(&mut c).unwrap();
                panic!("one short of the real peak");
            }
        };
        assert!(matches!(
            error,
            ParseStop::LimitExceeded { .. } | ParseStop::CssPropertyLimit { .. }
        ));
        assert_eq!(c.stop(), Some(error));
        empty(&c);
        assert!(c.usage().resource(HtmlParseResource::WorkCredits).current() > 0);
    }
    #[test]
    fn retained_report_disposal_after_cancellation_does_not_poll_or_replace_first_stop() {
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        let value = c
            .parse_meta_color_scheme(
                &"dark".encode_utf16().collect::<Vec<_>>(),
                crate::HtmlDocumentMode::Quirks,
            )
            .unwrap();
        c.cancellation.cancel();
        assert_eq!(c.checkpoint(), Err(ParseStop::Cancelled));
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current();
        value.release(&mut c).unwrap();
        empty(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current(),
            work
        );
    }
    #[test]
    fn a_callback_entered_while_another_runs_stops_the_parse_without_panicking() {
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        let stop = ParseStop::InternalInvariant(Invariant::ReentrantProviderCallback);
        let callbacks = SchemeCallbacks::new(&mut c);
        {
            // The parser cannot reach this through the closures it is given:
            // neither closure calls out while it holds the controller. The
            // guard stands in for foreign code still running inside one.
            let _running = callbacks.control.borrow_mut();
            assert_eq!(callbacks.admit_live(Some((8, 2))), Err(stop));
            callbacks.release_to((0, 0));
        }
        let finished = callbacks.finish();
        assert_eq!(finished.refusal, Some(stop));
        assert_eq!(finished.control.stop(), Some(stop));
        assert_eq!(finished.live, (0, 0));
        empty(finished.control);
    }
    #[test]
    fn a_release_larger_than_the_live_storage_stops_the_parse_without_panicking() {
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        let callbacks = SchemeCallbacks::new(&mut c);
        callbacks.admit_live(Some((10, 4))).unwrap();
        // The parser reports more remaining storage than it ever admitted.
        callbacks.release_to((12, 4));
        let Finished {
            control: c,
            live,
            owner,
            refusal,
        } = callbacks.finish();
        let underflow = ParseStop::InternalInvariant(Invariant::ReleaseUnderflow);
        assert_eq!(refusal, Some(underflow));
        assert_eq!(c.stop(), Some(underflow));
        assert_eq!(live, (10, 4), "the ledger is left as it was");
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current(),
            10,
            "nothing is credited for the impossible release"
        );
        c.release_retained_provider(&owner, live.0, live.1, 0)
            .unwrap();
        empty(c);
    }
    // Characterization of existing behaviour after B07; not RED against an
    // earlier commit. A test hook forces each state the CSS parser cannot make.
    #[test]
    fn characterization_a_refused_callback_during_a_scheme_parse_latches_and_credits_everything() {
        // The skipped release leaves more live than the parser later reports,
        // so its next admission underflows first; `finish` keeps that first
        // stop in place of the reentry it also records.
        let underflow = ParseStop::InternalInvariant(Invariant::ReleaseUnderflow);
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        c.css_scheme_fault = Some(SchemeFault::ReentrantRelease);
        let raw: Vec<_> = "only dark".encode_utf16().collect();
        let error = match c.parse_meta_color_scheme(&raw, crate::HtmlDocumentMode::NoQuirks) {
            Err(error) => error,
            Ok(value) => {
                value.release(&mut c).unwrap();
                panic!("the refused release was not reported");
            }
        };
        assert_eq!(error, underflow);
        assert_eq!(c.stop(), Some(underflow));
        empty(&c);
    }
    #[test]
    fn characterization_storage_admitted_beyond_the_report_is_a_capacity_mismatch_and_credited() {
        let mismatch = ParseStop::InternalInvariant(Invariant::CapacityMismatch);
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        c.css_scheme_fault = Some(SchemeFault::LiveDrift);
        let raw: Vec<_> = "only dark".encode_utf16().collect();
        let error = match c.parse_meta_color_scheme(&raw, crate::HtmlDocumentMode::NoQuirks) {
            Err(error) => error,
            Ok(value) => {
                value.release(&mut c).unwrap();
                panic!("the drifted admission was not reported");
            }
        };
        assert_eq!(error, mismatch);
        assert_eq!(c.stop(), Some(mismatch));
        empty(&c);
    }
}
