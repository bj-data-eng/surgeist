//! Consuming private ordinary construction ownership. This seal proves the
//! admitted immutable native/source transfer, not complete lexical D1 or public
//! parse success. Public promotion must consume the real complete associations
//! proof supplied by the sole evidence owner.

use super::{StandaloneError, terminal::FrozenStandaloneParts};
use crate::{HtmlDocumentParseRequest, request::DocumentRequestParts};

#[derive(Clone, Copy)]
enum InitializationPhase {
    Fresh,
    Rejected(super::entry::EntryRejection),
    Executing,
    Stopped,
    Transferred,
    Consumed,
}

/// Fixed source-owning bootstrap carrier. Its request and failure slot exist
/// before the first admission; no source-owning envelope is allocated after a
/// pre-source stop. The control/Arc bootstrap remains the existing documented
/// provider limitation. The native-only seal and full lexical D1 remain separate.
pub(crate) struct OrdinaryInitializationOwner {
    request: Option<HtmlDocumentParseRequest>,
    failure: Option<super::entry::StoppedInitialization>,
    phase: InitializationPhase,
}
impl OrdinaryInitializationOwner {
    pub(crate) fn new(request: HtmlDocumentParseRequest) -> Self {
        let request = request.share_associated_inputs();
        let phase = super::entry::EntryRejection::for_request(&request)
            .map_or(InitializationPhase::Fresh, InitializationPhase::Rejected);
        Self {
            request: Some(request),
            failure: None,
            phase,
        }
    }
    pub(crate) fn source(&self) -> Option<&crate::HtmlSource> {
        self.request.as_ref().map(|request| request.source())
    }
    pub(crate) fn request(&self) -> Option<&HtmlDocumentParseRequest> {
        self.request.as_ref()
    }
    /// The attempt retains every real borrowed prefix on refusal. It must be
    /// explicitly transferred or cleaned before dropping; no infallible Drop
    /// cleanup is implied. Only this consumer admits its actual fixed carrier.
    #[cfg(test)]
    pub(crate) fn begin(&mut self) -> Result<InitializationAttempt<'_>, StandaloneError> {
        self.begin_for(super::parse::InitializationTarget::Native, false)
    }
    pub(super) fn begin_report(&mut self) -> Result<InitializationAttempt<'_>, StandaloneError> {
        self.begin_for(
            super::parse::InitializationTarget::Report,
            #[cfg(test)]
            false,
        )
    }
    #[cfg(test)]
    pub(crate) fn begin_progressive(
        &mut self,
    ) -> Result<InitializationAttempt<'_>, StandaloneError> {
        self.begin_for(super::parse::InitializationTarget::Native, true)
    }
    fn begin_for(
        &mut self,
        target: super::parse::InitializationTarget,
        #[cfg(test)] progressive: bool,
    ) -> Result<InitializationAttempt<'_>, StandaloneError> {
        match self.phase {
            InitializationPhase::Rejected(reason) => return Err(reason.error()),
            InitializationPhase::Fresh => {}
            _ => return Err(StandaloneError::Consumed),
        }
        let request = self.request.as_ref().ok_or(StandaloneError::Consumed)?;
        self.phase = InitializationPhase::Executing;
        Ok(InitializationAttempt {
            entry: {
                #[cfg(test)]
                let entry = if progressive {
                    super::entry::OrdinaryDocumentEntry::owned_progressive(request)
                } else {
                    super::entry::OrdinaryDocumentEntry::owned(request, target)
                };
                #[cfg(not(test))]
                let entry = super::entry::OrdinaryDocumentEntry::owned(request, target);
                entry
            },
            request,
            failure: &mut self.failure,
            phase: &mut self.phase,
        })
    }
    pub(crate) fn failure(&self) -> Option<super::entry::InitializationFailure> {
        match self.phase {
            InitializationPhase::Rejected(reason) => {
                Some(super::entry::InitializationFailure::Rejected(reason))
            }
            InitializationPhase::Stopped => Some(super::entry::InitializationFailure::Stopped(
                self.failure
                    .as_ref()
                    .expect("retained initialization failure")
                    .stop(),
            )),
            _ => None,
        }
    }
    /// None means no stopped owner in this carrier, including pre-execution
    /// rejection or a transferred driver. Some(false) is a real pre-admission
    /// stop. An admitted empty source is Some(true).
    pub(crate) fn source_admitted(&self) -> Option<bool> {
        self.failure
            .as_ref()
            .map(|failure| failure.source_admitted())
    }
    pub(crate) fn at_stop(&self) -> Option<&crate::parser::control::ParseUsage> {
        self.failure.as_ref().map(|failure| failure.at_stop())
    }
    pub(crate) fn after_cleanup(&self) -> Option<&crate::parser::control::ParseUsage> {
        self.failure.as_ref().map(|failure| failure.after_cleanup())
    }
    pub(crate) fn retained_usage(&self) -> Option<&crate::parser::control::ParseUsage> {
        self.failure
            .as_ref()
            .map(|failure| failure.retained_usage())
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.failure
            .as_ref()
            .and_then(|failure| failure.diagnostics())
    }
    #[cfg(test)]
    pub(crate) fn release_failure(&mut self) -> Result<(), StandaloneError> {
        match self.phase {
            InitializationPhase::Rejected(_) => Ok(()),
            InitializationPhase::Stopped => self
                .failure
                .as_mut()
                .expect("actual stopped owner")
                .release(),
            _ => Err(StandaloneError::Consumed),
        }
    }
    /// A ready actual driver no longer borrows the carrier once its execution
    /// scope ends. Move the same original request to the existing final seal.
    pub(crate) fn take_request(&mut self) -> Result<HtmlDocumentParseRequest, StandaloneError> {
        match self.phase {
            InitializationPhase::Fresh | InitializationPhase::Transferred => {}
            _ => return Err(StandaloneError::Consumed),
        }
        let request = self.request.take().ok_or(StandaloneError::Consumed)?;
        self.phase = InitializationPhase::Consumed;
        Ok(request)
    }
}

pub(crate) struct InitializationAttempt<'request> {
    entry: super::entry::OrdinaryDocumentEntry<'request>,
    request: &'request HtmlDocumentParseRequest,
    failure: &'request mut Option<super::entry::StoppedInitialization>,
    phase: &'request mut InitializationPhase,
}
impl<'request> InitializationAttempt<'request> {
    #[cfg(test)]
    pub(crate) fn failure(&self) -> Option<super::entry::InitializationFailure> {
        self.entry.failure()
    }
    #[cfg(test)]
    pub(crate) fn at_stop(&self) -> Option<&crate::parser::control::ParseUsage> {
        self.entry.at_stop()
    }
    pub(crate) fn take_driver(
        &mut self,
    ) -> Result<super::StandaloneDocumentDriver<'request>, StandaloneError> {
        let driver = self.entry.take_driver()?;
        *self.phase = InitializationPhase::Transferred;
        Ok(driver)
    }
    pub(crate) fn retain_failure(&mut self) -> Result<(), StandaloneError> {
        self.entry.retain_failure(self.request, self.failure)?;
        *self.phase = InitializationPhase::Stopped;
        Ok(())
    }
}

/// Three carrier fields, four attempt fields and its three phase transitions,
/// plus the cleared original request slot;
/// plus the exact entry transfer slots. All are fixed bootstrap storage, not
/// variable source/native copies. Running admission follows source precedence;
/// an earlier stop cannot pretend this layout was admitted.
pub(super) const fn initialization_bootstrap_layout() -> (usize, usize) {
    const {
        use std::mem::size_of;
        let (bytes, writes) = super::entry::initialization_transfer_layout();
        (
            bytes
                + size_of::<OrdinaryInitializationOwner>()
                + size_of::<InitializationAttempt>()
                + 3 * size_of::<InitializationPhase>()
                + size_of::<Option<HtmlDocumentParseRequest>>(),
            writes + 3 + 4 + 3 + 1,
        )
    }
}
/// One cancellation identity and two source identity/basis comparisons, plus
/// six fixed phase/slot/stop/optional-binding visits, precede the stopped moves.
pub(super) const fn initialization_qualification_bytes() -> usize {
    const {
        2 * std::mem::size_of::<crate::HtmlParseCancellation>()
            + 4 * std::mem::size_of::<crate::HtmlSourceId>()
            + 4 * std::mem::size_of::<crate::HtmlSourceCoordinates>()
    }
}

/// Exact physical transfer layouts are prepaid by the initialization owner.
/// Four cleared owning options, five detached fields, four request fields and
/// two joined fields: every terminal move is fixed and needs no new admission.
pub(super) const fn terminal_transfer_layout() -> (usize, usize) {
    const {
        use std::mem::size_of;
        (
            size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<Option<super::terminal::FailedStandaloneDocument>>()
                + size_of::<super::terminal::DetachedStandaloneFailure>()
                + size_of::<Option<HtmlDocumentParseRequest>>()
                + size_of::<Option<super::terminal::DetachedStandaloneFailure>>()
                + size_of::<DocumentRequestParts>()
                + size_of::<OwnedOrdinaryFailure>(),
            4 + 5 + 4 + 2,
        )
    }
}
/// The terminal join reads two actual Arc identities and coordinate tags, with
/// two fixed offline-context predicates. Admission precedes the first pull.
pub(super) const fn terminal_qualification_bytes() -> usize {
    const {
        2 * std::mem::size_of::<crate::HtmlParseCancellation>()
            + 2 * std::mem::size_of::<crate::HtmlSourceId>()
            + 2 * std::mem::size_of::<crate::HtmlSourceCoordinates>()
    }
}

pub(crate) struct SealedOrdinaryConstruction {
    pub(super) request: DocumentRequestParts,
    pub(super) parts: FrozenStandaloneParts,
}
impl SealedOrdinaryConstruction {
    /// Both original owning slots survive every admission/qualification error.
    /// No fallible operation follows the two already-admitted moves.
    pub(crate) fn take(
        request: &mut Option<HtmlDocumentParseRequest>,
        parts: &mut Option<FrozenStandaloneParts>,
    ) -> Result<Self, StandaloneError> {
        let original = request.as_ref().ok_or(StandaloneError::Consumed)?;
        parts
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .prepare_seal(original)?;
        Ok(Self {
            request: request
                .take()
                .expect("qualified original request")
                .into_parts(),
            parts: parts.take().expect("admitted immutable construction"),
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn limits(&self) -> &crate::HtmlParseLimits {
        self.request.limits()
    }
    #[cfg(test)]
    pub(crate) fn is_offline(&self) -> bool {
        self.request.is_offline()
    }
    #[cfg(test)]
    pub(crate) fn document(&self) -> Option<&crate::HtmlDocument> {
        self.parts.immutable_document()
    }
    /// Same fixed committed store, with no controller borrow or new admission.
    /// Its native-only publication basis does not certify complete lexical D1.
    #[cfg(test)]
    pub(crate) fn evidence(
        &self,
    ) -> Result<&crate::construction::evidence::ParseEvidence, StandaloneError> {
        self.parts.immutable_evidence()
    }
    #[cfg(test)]
    pub(crate) fn final_usage(&self) -> &crate::parser::control::ParseUsage {
        self.parts.final_usage()
    }
    pub(crate) fn retained_usage(&self) -> &crate::parser::control::ParseUsage {
        self.parts.retained_usage()
    }
    /// Actual fallible disposal remains explicit. A live borrowed/owned native
    /// alias can refuse disposal; that error never discards the retained owner.
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        self.parts.release_sealed()
    }
}

/// Cleaned terminal parse evidence, joined to the original owned request. There
/// is deliberately no tree/effect/native lender on this stopped result.
pub(crate) struct OwnedOrdinaryFailure {
    request: DocumentRequestParts,
    failure: super::terminal::DetachedStandaloneFailure,
}
impl OwnedOrdinaryFailure {
    pub(crate) fn release_csp_storage(&mut self, storage: crate::meta_csp::Storage) {
        self.failure.release_csp_storage(storage)
    }
    pub(super) fn original_request(&self) -> &DocumentRequestParts {
        &self.request
    }
    pub(crate) fn take(
        request: &mut Option<HtmlDocumentParseRequest>,
        failure: &mut Option<super::terminal::DetachedStandaloneFailure>,
    ) -> Result<Self, StandaloneError> {
        let original = request.as_ref().ok_or(StandaloneError::Consumed)?;
        let failed = failure.as_ref().ok_or(StandaloneError::Consumed)?;
        if original.capture().is_some()
            || original.document().is_some()
            || !failed.owns_request(original.cancellation())
            || &failed.binding.0 != original.source().identity()
            || failed.binding.1 != original.source().coordinates()
        {
            return Err(StandaloneError::Consumed);
        }
        // Fixed envelope moves were admitted before the first source pull.
        Ok(Self {
            request: request
                .take()
                .expect("actual original request")
                .into_parts(),
            failure: failure.take().expect("qualified cleaned failure"),
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn stop(&self) -> crate::parser::control::ParseStop {
        self.failure.stop()
    }
    pub(crate) fn at_stop(&self) -> &crate::parser::control::ParseUsage {
        self.failure.at_stop()
    }
    pub(crate) fn after_cleanup(&self) -> &crate::parser::control::ParseUsage {
        self.failure.after_cleanup()
    }
    pub(crate) fn retained_usage(&self) -> &crate::parser::control::ParseUsage {
        self.failure.retained_usage()
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.failure.diagnostics()
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        self.failure.release()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tree_builder::{StandaloneDocumentDriver, head::HeadStep};
    use crate::{HtmlParseResource, HtmlSource};

    fn frozen(request: &HtmlDocumentParseRequest) -> FrozenStandaloneParts {
        let mut driver = Some(super::super::entry::initialized_driver(request));
        for _ in 0..4096 {
            if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
                let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
                completed.prepare_native_associations().unwrap();
                return completed.freeze_parts().unwrap();
            }
        }
        panic!("actual ordinary EOF was not reached");
    }
    fn request() -> HtmlDocumentParseRequest {
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A&amp;B"),
            super::super::tests::limits(100),
        )
    }
    fn work_limits(work: u64) -> crate::HtmlParseLimits {
        crate::HtmlParseLimits::builder()
            .source_units(100_000)
            .buffered_utf16_units(100_000)
            .token_attributes(10_000)
            .auxiliary_entries(1_000_000)
            .managed_nodes(100_000)
            .managed_edges(100_000)
            .managed_attributes(100_000)
            .managed_utf16_units(100_000)
            .source_map_entries(100_000)
            .source_map_payload_units(100_000)
            .diagnostics(100_000)
            .diagnostic_payload_units(100_000)
            .context_effects(100_000)
            .open_elements(100)
            .tree_depth(100_000)
            .managed_capacity_bytes(100_000_000)
            .peak_transient_bytes(100_000_000)
            .atomic_layout_bytes(100_000_000)
            .work_credits(work)
            .finish()
            .unwrap()
    }

    fn initialized_failure(request: HtmlDocumentParseRequest) -> OrdinaryInitializationOwner {
        let mut owner = OrdinaryInitializationOwner::new(request);
        {
            let mut attempt = owner.begin().unwrap();
            assert!(attempt.failure().is_some());
            assert!(attempt.take_driver().is_err());
            attempt.retain_failure().unwrap();
            assert!(matches!(
                attempt.retain_failure(),
                Err(StandaloneError::Consumed)
            ));
        }
        owner
    }
    fn no_retained_storage(usage: &crate::parser::control::ParseUsage) {
        for resource in crate::settings::RESOURCES {
            if resource.storage() && resource != HtmlParseResource::ManagedNodes {
                assert_eq!(usage.resource(resource).current, 0, "{resource:?}");
            }
        }
        assert_eq!(usage.actual_capacity_bytes, 0);
    }

    #[test]
    fn owned_cancelled_initialization_retains_exact_source_without_admitted_binding() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(crate::DocumentString::from_utf16(vec![
                0xfeff, 0xd800, 0x41, 0xdc00,
            ])),
            work_limits(0),
        );
        let source_id = request.source().identity().clone();
        request.cancellation().cancel();
        let mut owner = initialized_failure(request);
        assert_eq!(owner.source().unwrap().identity(), &source_id);
        assert_eq!(
            owner.source().unwrap().as_utf16().unwrap().code_units(),
            &[0xfeff, 0xd800, 0x41, 0xdc00]
        );
        assert_eq!(
            owner.failure(),
            Some(super::super::entry::InitializationFailure::Stopped(
                crate::parser::control::ParseStop::Cancelled
            ))
        );
        assert_eq!(owner.source_admitted(), Some(false));
        let first = owner.at_stop().unwrap().clone();
        assert_eq!(first.resource(HtmlParseResource::SourceUnits).current, 0);
        assert_eq!(first.resource(HtmlParseResource::WorkCredits).current, 0);
        assert_eq!(owner.after_cleanup(), Some(&first));
        assert!(owner.diagnostics().is_none());
        owner.release_failure().unwrap();
        assert_eq!(owner.retained_usage(), Some(&first));
        assert!(matches!(owner.begin(), Err(StandaloneError::Consumed)));
    }

    #[test]
    fn owned_zero_work_stops_before_source_and_never_counts_denied_record_visit() {
        let mut owner = initialized_failure(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("x"),
            work_limits(0),
        ));
        assert_eq!(
            owner.failure(),
            Some(super::super::entry::InitializationFailure::Stopped(
                crate::parser::control::ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: 0,
                    current: 0,
                    requested_delta: 1,
                }
            ))
        );
        assert_eq!(owner.source_admitted(), Some(false));
        let first = owner.at_stop().unwrap().clone();
        assert_eq!(
            first.work(crate::parser::control::WorkCategory::RecordVisit),
            0
        );
        assert_eq!(first.largest_atomic_layout, 0);
        assert!(matches!(
            owner.take_request(),
            Err(StandaloneError::Consumed)
        ));
        assert_eq!(owner.source().unwrap().as_utf8(), Some("x"));
        assert_eq!(owner.at_stop(), Some(&first));
        owner.request().unwrap().cancellation().cancel();
        owner.release_failure().unwrap();
        assert_eq!(owner.at_stop(), Some(&first));
        assert_eq!(owner.after_cleanup(), Some(&first));
        assert_eq!(owner.retained_usage(), Some(&first));
    }

    #[test]
    fn owned_zero_source_cap_precedes_zero_layout_but_empty_source_is_admitted() {
        let limits = work_limits(1_000_000)
            .into_builder()
            .source_units(0)
            .atomic_layout_bytes(0)
            .finish()
            .unwrap();
        for text in ["x", ""] {
            let mut owner = initialized_failure(HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(text),
                limits,
            ));
            let first = owner.at_stop().unwrap().clone();
            assert_eq!(
                first.work(crate::parser::control::WorkCategory::RecordVisit),
                1
            );
            assert_eq!(
                first.work(crate::parser::control::WorkCategory::CopyByte),
                0
            );
            assert_eq!(first.resource(HtmlParseResource::SourceUnits).current, 0);
            assert_eq!(owner.source_admitted(), Some(text.is_empty()));
            let expected = if text.is_empty() {
                crate::parser::control::ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AtomicLayoutBytes,
                    limit: 0,
                    current: 0,
                    requested_delta: initialization_bootstrap_layout().0 as u64,
                }
            } else {
                crate::parser::control::ParseStop::LimitExceeded {
                    resource: HtmlParseResource::SourceUnits,
                    limit: 0,
                    current: 0,
                    requested_delta: 1,
                }
            };
            assert_eq!(
                owner.failure(),
                Some(super::super::entry::InitializationFailure::Stopped(
                    expected
                ))
            );
            owner.release_failure().unwrap();
            assert_eq!(owner.at_stop(), Some(&first));
            no_retained_storage(owner.retained_usage().unwrap());
        }
    }

    #[test]
    fn owned_bootstrap_copy_and_write_cuts_preserve_actual_admitted_prefix() {
        let (bytes, writes) = initialization_bootstrap_layout();
        for (cap, copied, denied) in [
            (1, 0, bytes as u64),
            (1 + bytes as u64, bytes as u64, writes as u64),
        ] {
            let mut owner = initialized_failure(HtmlDocumentParseRequest::offline(
                HtmlSource::utf8("x"),
                work_limits(cap),
            ));
            let first = owner.at_stop().unwrap().clone();
            assert_eq!(
                owner.failure(),
                Some(super::super::entry::InitializationFailure::Stopped(
                    crate::parser::control::ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        limit: cap,
                        current: 1 + copied,
                        requested_delta: denied,
                    }
                ))
            );
            assert_eq!(owner.source_admitted(), Some(true));
            assert_eq!(first.resource(HtmlParseResource::SourceUnits).current, 1);
            assert_eq!(
                first.work(crate::parser::control::WorkCategory::CopyByte),
                copied
            );
            assert_eq!(
                first.work(crate::parser::control::WorkCategory::RecordWrite),
                0
            );
            assert_eq!(first.largest_atomic_layout, bytes as u64);
            owner.release_failure().unwrap();
            assert_eq!(owner.at_stop(), Some(&first));
            assert_eq!(owner.after_cleanup(), Some(&first));
            no_retained_storage(owner.retained_usage().unwrap());
        }
    }

    #[test]
    fn owned_nested_initialization_cleanup_keeps_canonical_capacity_observation() {
        // The Document retains its child-ticket and append-field index entries
        // before the next metadata slot requests one additional entry.
        let limits = work_limits(1_000_000)
            .into_builder()
            .auxiliary_entries(2)
            .finish()
            .unwrap();
        let mut owner = initialized_failure(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(""),
            limits,
        ));
        assert_eq!(
            owner.failure(),
            Some(super::super::entry::InitializationFailure::Stopped(
                crate::parser::control::ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    limit: 2,
                    current: 2,
                    requested_delta: 1,
                }
            ))
        );
        let first = owner.at_stop().unwrap().clone();
        assert!(first.actual_capacity_bytes > 0);
        assert_eq!(first.resource(HtmlParseResource::ManagedNodes).current, 1);
        assert_eq!(owner.source_admitted(), Some(true));
        assert!(owner.diagnostics().is_none());
        no_retained_storage(owner.retained_usage().unwrap());
        assert_eq!(
            owner
                .retained_usage()
                .unwrap()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            1
        );
        owner.release_failure().unwrap();
        assert_eq!(owner.at_stop(), Some(&first));
        assert_eq!(
            owner.after_cleanup().unwrap().actual_capacity_high_water,
            first.actual_capacity_high_water
        );
    }

    #[test]
    fn owned_ready_entry_cancelled_before_take_keeps_initialized_empty_diagnostics() {
        let request =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), work_limits(1_000_000));
        let cancel = request.cancellation().clone();
        let mut owner = OrdinaryInitializationOwner::new(request);
        let first;
        {
            let mut attempt = owner.begin().unwrap();
            assert!(attempt.failure().is_none());
            cancel.cancel();
            assert!(matches!(
                attempt.take_driver(),
                Err(StandaloneError::Stop(
                    crate::parser::control::ParseStop::Cancelled
                ))
            ));
            first = attempt.at_stop().unwrap().clone();
            attempt.retain_failure().unwrap();
        }
        assert_eq!(owner.source_admitted(), Some(true));
        assert_eq!(owner.diagnostics().unwrap().len(), 0);
        // The initialized empty store retains its actual zero-sized issuer
        // ticket, not an invented nonzero Vec capacity or Arc-header estimate.
        no_retained_storage(owner.retained_usage().unwrap());
        let after = owner.after_cleanup().unwrap().clone();
        assert_eq!(owner.at_stop(), Some(&first));
        owner.release_failure().unwrap();
        assert!(owner.diagnostics().is_none());
        no_retained_storage(owner.retained_usage().unwrap());
        assert_eq!(owner.at_stop(), Some(&first));
        assert_eq!(owner.after_cleanup(), Some(&after));
    }

    #[test]
    fn owned_initialization_foreign_and_same_source_fresh_requests_refuse_without_losing_prefix() {
        let mut owner = OrdinaryInitializationOwner::new(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("x"),
            work_limits(0),
        ));
        let fresh = HtmlDocumentParseRequest::offline(
            owner.source().unwrap().clone(),
            work_limits(1_000_000),
        );
        let foreign = HtmlDocumentParseRequest::offline(HtmlSource::utf8("x"), work_limits(0));
        {
            let mut attempt = owner.begin().unwrap();
            let first = attempt.at_stop().unwrap().clone();
            for wrong in [&fresh, &foreign] {
                let mut refused = None;
                assert!(matches!(
                    attempt.entry.retain_failure(wrong, &mut refused),
                    Err(StandaloneError::Consumed)
                ));
                assert!(refused.is_none());
                assert_eq!(attempt.at_stop(), Some(&first));
            }
            attempt.retain_failure().unwrap();
        }
        assert_eq!(owner.source_admitted(), Some(false));
        owner.release_failure().unwrap();
    }

    #[test]
    fn captured_owned_request_is_rejected_without_control_or_usage_and_preserves_context() {
        let mut builder = crate::HtmlCaptureBuilder::new(
            crate::HtmlCaptureLimits::new(10, 10, 0, 0, 10, 0, 0, 16_777_216),
            crate::HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                crate::HtmlDocumentMode::Quirks,
                crate::HtmlScriptingStatus::Disabled,
                crate::HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let capture = builder.finish().unwrap();
        let document = capture
            .document_id(&capture.subject_id(&document).unwrap())
            .unwrap();
        let request = HtmlDocumentParseRequest::captured(
            HtmlSource::utf8("<p>x"),
            capture,
            document,
            work_limits(0),
        )
        .unwrap();
        let source_id = request.source().identity().clone();
        let capture_id = request.capture().unwrap().identity().clone();
        let document_id = request.document().unwrap().clone();
        let mut owner = OrdinaryInitializationOwner::new(request);
        assert_eq!(
            owner.failure(),
            Some(super::super::entry::InitializationFailure::Rejected(
                super::super::entry::EntryRejection::DocumentInitialization
            ))
        );
        assert!(matches!(
            owner.begin(),
            Err(StandaloneError::UnsupportedDocumentInitialization)
        ));
        assert_eq!(owner.source().unwrap().identity(), &source_id);
        assert_eq!(owner.source().unwrap().as_utf8(), Some("<p>x"));
        assert_eq!(
            owner.request().unwrap().capture().unwrap().identity(),
            &capture_id
        );
        assert_eq!(owner.request().unwrap().document(), Some(&document_id));
        assert_eq!(owner.source_admitted(), None);
        assert!(owner.at_stop().is_none());
        assert!(owner.after_cleanup().is_none());
        assert!(owner.retained_usage().is_none());
        assert!(owner.diagnostics().is_none());
        owner.release_failure().unwrap();
        assert!(owner.take_request().is_err());
        assert_eq!(owner.source().unwrap().identity(), &source_id);
        assert_eq!(
            owner.request().unwrap().capture().unwrap().identity(),
            &capture_id
        );
    }

    #[test]
    fn actual_owned_ready_driver_preserves_original_request_for_native_seal() {
        let request = request();
        let source_id = request.source().identity().clone();
        let mut owner = OrdinaryInitializationOwner::new(request);
        let parts = {
            let mut attempt = owner.begin().unwrap();
            assert!(matches!(
                attempt.retain_failure(),
                Err(StandaloneError::Consumed)
            ));
            let mut driver = Some(attempt.take_driver().unwrap());
            assert!(matches!(
                attempt.take_driver(),
                Err(StandaloneError::Consumed)
            ));
            for _ in 0..4096 {
                if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
                    break;
                }
            }
            let mut completed = StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
            completed.prepare_native_associations().unwrap();
            completed.freeze_parts().unwrap()
        };
        let original = owner.take_request().unwrap();
        assert!(owner.source().is_none());
        assert!(matches!(
            owner.take_request(),
            Err(StandaloneError::Consumed)
        ));
        let mut original = Some(original);
        let mut parts = Some(parts);
        let mut sealed = SealedOrdinaryConstruction::take(&mut original, &mut parts).unwrap();
        assert_eq!(sealed.source().identity(), &source_id);
        assert_eq!(sealed.source().as_utf8(), Some("<!doctype html><p>A&amp;B"));
        assert!(sealed.document().is_some());
        sealed.release().unwrap();
    }

    #[test]
    fn owned_initialization_work_cuts_keep_real_prefix_tickets_and_first_stop() {
        for cap in (0..80_000).step_by(313) {
            let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), work_limits(cap));
            let mut owner = OrdinaryInitializationOwner::new(request);
            let expected;
            {
                let mut attempt = owner.begin().unwrap();
                match attempt.take_driver() {
                    Ok(mut driver) => {
                        if driver.release().is_err() {
                            driver.release().unwrap();
                        }
                        no_retained_storage(driver.control.usage());
                        expected = None;
                    }
                    Err(_) => {
                        let first = attempt.at_stop().unwrap().clone();
                        assert!(matches!(
                            attempt.failure(),
                            Some(super::super::entry::InitializationFailure::Stopped(
                                crate::parser::control::ParseStop::LimitExceeded {
                                    resource: HtmlParseResource::WorkCredits,
                                    ..
                                }
                            ))
                        ));
                        attempt.retain_failure().unwrap();
                        assert_eq!(attempt.failure(), None);
                        assert_eq!(
                            owner_failure_stop(&first),
                            first.resource(HtmlParseResource::WorkCredits).current
                        );
                        expected = Some(first);
                    }
                }
            }
            if let Some(first) = expected {
                assert_eq!(owner.at_stop(), Some(&first));
                let after = owner.after_cleanup().unwrap().clone();
                owner.request().unwrap().cancellation().cancel();
                owner.release_failure().unwrap();
                assert_eq!(owner.at_stop(), Some(&first));
                assert_eq!(owner.after_cleanup(), Some(&after));
                no_retained_storage(owner.retained_usage().unwrap());
            }
        }
    }
    fn owner_failure_stop(first: &crate::parser::control::ParseUsage) -> u64 {
        use crate::parser::control::WorkCategory;
        [
            WorkCategory::InputUnitRead,
            WorkCategory::InspectByte,
            WorkCategory::CopyByte,
            WorkCategory::StateStep,
            WorkCategory::RecordVisit,
            WorkCategory::RecordWrite,
            WorkCategory::ReserveByte,
            WorkCategory::ProviderCall,
        ]
        .into_iter()
        .map(|category| first.work(category))
        .sum()
    }
    #[test]
    fn actual_seal_write_admission_cut_preserves_both_slots_and_terminal_cleanup() {
        let original = request();
        let mut measured = frozen(&original);
        let before = measured
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let bytes = std::mem::size_of::<SealedOrdinaryConstruction>()
            + std::mem::size_of::<Option<FrozenStandaloneParts>>()
            + std::mem::size_of::<Option<HtmlDocumentParseRequest>>()
            + std::mem::size_of::<DocumentRequestParts>()
            + std::mem::size_of::<crate::parser::control::ParseUsage>();
        measured.release().unwrap();
        let mut request = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A&amp;B"),
            work_limits(before + bytes as u64 + 1 + 5 + 9 - 1),
        ));
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        assert_eq!(
            parts
                .as_ref()
                .unwrap()
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before
        );
        assert!(matches!(
            SealedOrdinaryConstruction::take(&mut request, &mut parts),
            Err(StandaloneError::Stop(
                crate::parser::control::ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta: 9,
                    ..
                }
            ))
        ));
        assert!(request.is_some() && parts.is_some());
        let at_stop = parts.as_ref().unwrap().at_stop().unwrap().clone();
        parts.as_mut().unwrap().release().unwrap();
        assert_eq!(parts.as_ref().unwrap().at_stop(), Some(&at_stop));
        assert_eq!(
            parts
                .as_ref()
                .unwrap()
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
    #[test]
    fn seal_moves_original_source_and_same_frozen_tree_without_readback_work() {
        let mut request = Some(request());
        let identity = request.as_ref().unwrap().source().identity().clone();
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let document = parts.as_mut().unwrap().document().unwrap().clone();
        let root = document.root().id();
        let mut sealed = SealedOrdinaryConstruction::take(&mut request, &mut parts).unwrap();
        assert!(request.is_none() && parts.is_none());
        assert_eq!(sealed.source().identity(), &identity);
        assert_eq!(sealed.document().unwrap().root().id(), root);
        assert!(sealed.is_offline());
        assert_eq!(sealed.limits(), &super::super::tests::limits(100));
        let tree = sealed.document().unwrap();
        let element = |parent: crate::HtmlNode<'_>, name: &str| {
            parent.children().find_map(|id| {
                let node = tree.node(&id).unwrap();
                matches!(node.kind(), crate::HtmlNodeKind::Element(e) if e.name().local_name().code_units() == name.encode_utf16().collect::<Vec<_>>()).then_some(node)
            }).expect("literal native element")
        };
        let html = element(tree.root(), "html");
        let body = element(html, "body");
        let p = element(body, "p");
        let text = tree.node(&p.children().next().unwrap()).unwrap();
        assert!(
            matches!(text.kind(), crate::HtmlNodeKind::Text(value) if value.code_units() == [65, 38, 66])
        );
        let before = sealed.retained_usage().clone();
        let evidence = sealed.evidence().unwrap() as *const _;
        for _ in 0..3 {
            assert_eq!(sealed.evidence().unwrap() as *const _, evidence);
            assert_eq!(sealed.document().unwrap().root().id(), root);
        }
        assert_eq!(sealed.retained_usage(), &before);
        drop(document);
        sealed.release().unwrap();
        assert!(sealed.document().is_none());
        assert!(sealed.evidence().is_err());
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        sealed.release().unwrap();
    }
    #[test]
    fn later_cancellation_does_not_invalidate_immutable_native_or_evidence_lenders() {
        let mut request = Some(request());
        let cancellation = request.as_ref().unwrap().cancellation().clone();
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let mut sealed = SealedOrdinaryConstruction::take(&mut request, &mut parts).unwrap();
        let final_usage = sealed.final_usage().clone();
        let retained = sealed.retained_usage().clone();
        let evidence = sealed.evidence().unwrap() as *const _;
        cancellation.cancel();
        assert!(sealed.request.cancellation().is_cancelled());
        assert!(sealed.document().is_some());
        assert_eq!(sealed.evidence().unwrap() as *const _, evidence);
        assert_eq!(sealed.final_usage(), &final_usage);
        assert_eq!(sealed.retained_usage(), &retained);
        sealed.release().unwrap();
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
    #[test]
    fn real_alias_refusal_retains_sealed_owner_and_retry_releases_same_storage() {
        let mut request = Some(request());
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let mut sealed = SealedOrdinaryConstruction::take(&mut request, &mut parts).unwrap();
        let alias = sealed.document().unwrap().clone();
        let identity = alias.root().id();
        let evidence = sealed.evidence().unwrap() as *const _;
        let final_usage = sealed.final_usage().clone();
        assert!(sealed.release().is_err());
        assert_eq!(sealed.document().unwrap().root().id(), identity);
        assert_eq!(sealed.evidence().unwrap() as *const _, evidence);
        assert_eq!(sealed.final_usage(), &final_usage);
        assert!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                > 0
        );
        drop(alias);
        sealed.release().unwrap();
        assert!(sealed.document().is_none());
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }
    #[test]
    fn foreign_equal_text_request_and_final_cancellation_keep_both_original_slots() {
        let original = request();
        let mut parts = Some(frozen(&original));
        let mut foreign = Some(request());
        assert!(SealedOrdinaryConstruction::take(&mut foreign, &mut parts).is_err());
        assert!(foreign.is_some() && parts.is_some());
        parts.as_mut().unwrap().release().unwrap();
        let mut actual = Some(request());
        let mut parts = Some(frozen(actual.as_ref().unwrap()));
        actual.as_ref().unwrap().cancellation().cancel();
        assert!(SealedOrdinaryConstruction::take(&mut actual, &mut parts).is_err());
        assert!(actual.is_some() && parts.is_some());
        parts.as_mut().unwrap().release().unwrap();
    }
    #[test]
    fn stopped_frozen_alias_refusal_preserves_binding_and_retry_joins_original_owned_source() {
        let mut request = Some(request());
        let source = request.as_ref().unwrap().source().identity().clone();
        let mut parts = Some(frozen(request.as_ref().unwrap()));
        let alias = parts.as_mut().unwrap().document().unwrap().clone();
        request.as_ref().unwrap().cancellation().cancel();
        assert!(SealedOrdinaryConstruction::take(&mut request, &mut parts).is_err());
        let first = parts.as_ref().unwrap().at_stop().unwrap().clone();
        assert!(FrozenStandaloneParts::detach_failure(&mut parts).is_err());
        assert!(parts.is_some() && request.is_some());
        assert_eq!(parts.as_ref().unwrap().source_binding().0, &source);
        assert_eq!(parts.as_ref().unwrap().at_stop(), Some(&first));
        drop(alias);
        let mut failure = Some(FrozenStandaloneParts::detach_failure(&mut parts).unwrap());
        assert!(parts.is_none());
        let mut owned = OwnedOrdinaryFailure::take(&mut request, &mut failure).unwrap();
        assert_eq!(owned.source().identity(), &source);
        assert_eq!(owned.at_stop(), &first);
        assert_eq!(
            owned
                .retained_usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            first.resource(HtmlParseResource::ManagedNodes).current
        );
        owned.release().unwrap();
        assert_eq!(owned.at_stop(), &first);
        assert_eq!(
            owned
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    fn stopped(
        request: &HtmlDocumentParseRequest,
    ) -> super::super::terminal::DetachedStandaloneFailure {
        let mut driver = Some(super::super::entry::initialized_driver(request));
        for _ in 0..256 {
            if driver
                .as_ref()
                .unwrap()
                .control
                .usage()
                .resource(HtmlParseResource::Diagnostics)
                .current
                > 0
            {
                break;
            }
            driver.as_mut().unwrap().step().unwrap();
        }
        assert!(
            driver
                .as_ref()
                .unwrap()
                .control
                .usage()
                .resource(HtmlParseResource::Diagnostics)
                .current
                > 0
        );
        request.cancellation().cancel();
        assert!(driver.as_mut().unwrap().step().is_err());
        let mut failed = Some(StandaloneDocumentDriver::take_failed(&mut driver).unwrap());
        assert!(driver.is_none());
        let detached =
            super::super::terminal::FailedStandaloneDocument::detach(&mut failed).unwrap();
        assert!(failed.is_none());
        assert!(super::super::terminal::FailedStandaloneDocument::detach(&mut failed).is_err());
        detached
    }
    #[test]
    fn cleaned_terminal_transfer_owns_original_source_and_keeps_exact_first_stop_and_retention() {
        let mut request = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let identity = request.as_ref().unwrap().source().identity().clone();
        let mut failure = Some(stopped(request.as_ref().unwrap()));
        let at_stop = failure.as_ref().unwrap().at_stop().clone();
        let cleaned = failure.as_ref().unwrap().after_cleanup().clone();
        let retained = failure.as_ref().unwrap().retained_usage().clone();
        let records = failure.as_ref().unwrap().diagnostics().unwrap().len();
        assert!(records > 0);
        let mut owned = OwnedOrdinaryFailure::take(&mut request, &mut failure).unwrap();
        assert!(request.is_none() && failure.is_none());
        assert_eq!(owned.source().identity(), &identity);
        assert_eq!(owned.stop(), crate::parser::control::ParseStop::Cancelled);
        assert_eq!(owned.at_stop(), &at_stop);
        assert_eq!(owned.after_cleanup(), &cleaned);
        assert_eq!(owned.retained_usage(), &retained);
        assert_eq!(owned.diagnostics().unwrap().len(), records);
        owned.release().unwrap();
        assert_eq!(owned.at_stop(), &at_stop);
        assert_eq!(owned.after_cleanup(), &cleaned);
        assert_eq!(
            owned
                .retained_usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        owned.release().unwrap();
    }
    #[test]
    fn foreign_terminal_join_refuses_without_losing_original_request_or_failure_and_retry_succeeds()
    {
        let mut request = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let mut failure = Some(stopped(request.as_ref().unwrap()));
        let mut foreign = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let before = failure.as_ref().unwrap().retained_usage().clone();
        assert!(OwnedOrdinaryFailure::take(&mut foreign, &mut failure).is_err());
        assert!(foreign.is_some() && failure.is_some());
        assert_eq!(failure.as_ref().unwrap().retained_usage(), &before);
        let mut owned = OwnedOrdinaryFailure::take(&mut request, &mut failure).unwrap();
        owned.release().unwrap();
    }
    #[test]
    fn source_binding_move_is_cleanup_only_and_once_only_without_changing_first_stop() {
        let request = request();
        let mut control = crate::parser::control::ParseControl::new(
            *request.limits(),
            request.cancellation().clone(),
        );
        control.admit_source(request.source()).unwrap();
        assert!(control.take_stopped_source_binding().is_err());
        assert!(control.stop().is_none());
        control.require_source(request.source()).unwrap();
        request.cancellation().cancel();
        assert!(control.checkpoint().is_err());
        let before = control.usage().clone();
        let first = control.first_stop_usage().unwrap().clone();
        let binding = control.take_stopped_source_binding().unwrap();
        assert_eq!(&binding.0, request.source().identity());
        assert_eq!(binding.1, request.source().coordinates());
        assert!(control.take_stopped_source_binding().is_err());
        assert_eq!(control.usage(), &before);
        assert_eq!(control.first_stop_usage(), Some(&first));
    }
    #[test]
    fn same_source_fresh_request_cannot_join_sealed_runtime_or_frozen_owners() {
        let mut original = Some(request());
        let mut fresh = Some(HtmlDocumentParseRequest::offline(
            original.as_ref().unwrap().source().clone(),
            work_limits(99_999_999),
        ));
        let mut parts = Some(frozen(original.as_ref().unwrap()));
        assert!(SealedOrdinaryConstruction::take(&mut fresh, &mut parts).is_err());
        assert!(fresh.is_some() && parts.is_some());
        let mut sealed = SealedOrdinaryConstruction::take(&mut original, &mut parts).unwrap();
        sealed.release().unwrap();

        let mut original = Some(HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        ));
        let mut fresh = Some(HtmlDocumentParseRequest::offline(
            original.as_ref().unwrap().source().clone(),
            work_limits(99_999_999),
        ));
        let mut failure = Some(stopped(original.as_ref().unwrap()));
        assert!(OwnedOrdinaryFailure::take(&mut fresh, &mut failure).is_err());
        assert!(fresh.is_some() && failure.is_some());
        let mut owned = OwnedOrdinaryFailure::take(&mut original, &mut failure).unwrap();
        owned.release().unwrap();

        let mut original = Some(request());
        let mut fresh = Some(HtmlDocumentParseRequest::offline(
            original.as_ref().unwrap().source().clone(),
            work_limits(99_999_999),
        ));
        let mut parts = Some(frozen(original.as_ref().unwrap()));
        original.as_ref().unwrap().cancellation().cancel();
        assert!(SealedOrdinaryConstruction::take(&mut original, &mut parts).is_err());
        let mut failure = Some(FrozenStandaloneParts::detach_failure(&mut parts).unwrap());
        assert!(OwnedOrdinaryFailure::take(&mut fresh, &mut failure).is_err());
        assert!(fresh.is_some() && failure.is_some());
        let mut owned = OwnedOrdinaryFailure::take(&mut original, &mut failure).unwrap();
        owned.release().unwrap();
    }
}
