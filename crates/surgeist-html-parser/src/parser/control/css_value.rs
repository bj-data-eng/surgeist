//! CSS-owned property graphs retained under the original parser controller.
use super::*;
use crate::parser::buffer::Utf16Buffer;
use std::cell::{Cell, RefCell};
use surgeist_css::{
    CssColorScheme, CssColorSchemeParseReport, CssComponentValueLimits, CssParser,
    CssParserContext, CssParserMode, CssUtf16InputLimits, CssUtf16PropertyValueLimits,
    CssUtf16ValueAdmission,
};

/// One admitted immutable provider occurrence. This never enters the string pool.
struct ProviderReceipt {
    owner: ControlIdentity,
    bytes: u64,
    units: u64,
}
impl ProviderReceipt {
    fn release(self, c: &mut ParseControl) {
        c.release_retained_provider(&self.owner, self.bytes, self.units, 0);
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
    pub(crate) fn release(self, c: &mut ParseControl) {
        let Self { report, receipt } = self;
        drop(report);
        receipt.release(c);
    }
}

impl ParseControl {
    fn remaining_css(&mut self, resource: HtmlParseResource) -> Result<usize, ParseStop> {
        self.checkpoint()?;
        let remaining = self.limits[resource as usize]
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
        let scratch = usize::try_from(self.limits[HtmlParseResource::AtomicLayoutBytes as usize])
            .unwrap_or(usize::MAX);
        let syntax = CssComponentValueLimits::try_new(
            u32::try_from(entries.min(256)).expect("bounded CSS depth"),
            entries,
            bytes,
        )
        .expect("CSS's fixed maximum depth is preserved");
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
                .ok_or_else(|| {
                    self.representation::<()>(HtmlParseResource::ManagedCapacityBytes)
                        .unwrap_err()
                })?;
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
        self.commit_prepared_release(&[(allocation.resource, allocation.units)]);
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
            let original = self.identity();
            let live = Cell::new((0_u64, 0_u64));
            let control = RefCell::new(&mut *self);
            let context = CssParserContext::new(if mode == crate::HtmlDocumentMode::Quirks {
                CssParserMode::Quirks
            } else {
                CssParserMode::Standards
            });
            let result = CssColorScheme::parse_utf16(
                &CssParser::new(context, syntax),
                input.clone(),
                limits,
                |event| {
                    let mut c = control.borrow_mut();
                    c.charge(WorkCategory::StateStep, 1)?;
                    if let CssUtf16ValueAdmission::Live(next) = event {
                        let next = (next.bytes() as u64, next.utf16_units() as u64);
                        let old = live.get();
                        let bytes = next
                            .0
                            .checked_sub(old.0)
                            .ok_or_else(|| c.fail_invariant(Invariant::ReleaseUnderflow))?;
                        let units = next
                            .1
                            .checked_sub(old.1)
                            .ok_or_else(|| c.fail_invariant(Invariant::ReleaseUnderflow))?;
                        c.acquire_retained_provider(bytes, units, 0)?;
                        live.set(next);
                    }
                    Ok(())
                },
                |remaining| {
                    let next = (remaining.bytes() as u64, remaining.utf16_units() as u64);
                    let old = live.replace(next);
                    control.borrow_mut().release_retained_provider(
                        &original,
                        old.0
                            .checked_sub(next.0)
                            .expect("CSS releases actual bytes"),
                        old.1
                            .checked_sub(next.1)
                            .expect("CSS releases actual units"),
                        0,
                    );
                },
            );
            let c = control.into_inner();
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
            assert_eq!(
                live.get(),
                (usage.bytes() as u64, usage.utf16_units() as u64)
            );
            // Admit the heap report body while the inline report is still live.
            let header = size_of::<CssColorSchemeParseReport>() as u64;
            if let Err(stop) = c
                .charge(WorkCategory::CopyByte, header)
                .and_then(|()| c.acquire_retained_provider(header, 0, 0))
            {
                drop(report);
                c.release_retained_provider(
                    &original,
                    usage.bytes() as u64,
                    usage.utf16_units() as u64,
                    0,
                );
                return Err(stop);
            }
            let report = Arc::new(report);
            c.release_retained_provider(&original, header, 0, 0);
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
        input_receipt.release(self);
        result
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
        value.release(&mut c);
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
        value.release(&mut baseline);
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
                value.release(&mut c);
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
        value.release(&mut c);
        empty(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current(),
            work
        );
    }
}
