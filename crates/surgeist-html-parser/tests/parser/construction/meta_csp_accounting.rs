//! Owned native error and temporary custody through the real consuming parser.
//! This closed trusted fixture recognizes two serialized policies; it is not a
//! CSP grammar implementation or an enforcement-engine conformance test.
use crate::parser::support::limits;
use std::{
    convert::Infallible,
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use surgeist_html::*;
use surgeist_html_parser::*;

const FIRST: &str = "default-src 'self'; report-uri /reports; frame-ancestors 'none'; sandbox";
const SECOND: &str = "img-src 'none'";
const ERROR_UNITS: usize = 128;
const SCRATCH_UNITS: usize = 64;
const REJECTED_UNITS: usize = 4096;

#[derive(Debug, Default)]
struct Seen {
    calls: AtomicUsize,
    policies: AtomicUsize,
    policies_dropped: AtomicUsize,
    scratch: AtomicUsize,
    scratch_dropped: AtomicUsize,
    errors: AtomicUsize,
    errors_dropped: AtomicUsize,
    error_pointer: AtomicUsize,
}
fn count(value: &AtomicUsize) -> usize {
    value.load(Ordering::Relaxed)
}
fn allow(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Refused,
    Failed,
    CancelBeforeScratch,
    CancelAfterScratch,
    CancelAtAttestation,
    NativeLimit,
}
#[derive(Debug)]
struct Policy {
    default_self: bool,
    seen: Arc<Seen>,
}
impl Policy {
    fn layout() -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>() + size_of::<Seen>(), 0, 2).unwrap(),
            HtmlCspDisposalWork::try_new(8, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
}
impl HtmlCspPolicyArtifact for Policy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        Self::layout()
    }
}
impl Drop for Policy {
    fn drop(&mut self) {
        self.seen.policies_dropped.fetch_add(1, Ordering::Relaxed);
    }
}
#[derive(Debug)]
struct Scratch {
    _backing: Box<[u16; SCRATCH_UNITS]>,
    seen: Arc<Seen>,
}
impl Scratch {
    fn storage() -> HtmlCspStorage {
        HtmlCspStorage::try_new(
            size_of::<Self>() + size_of::<[u16; SCRATCH_UNITS]>() + size_of::<Seen>(),
            SCRATCH_UNITS,
            3,
        )
        .unwrap()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        self.seen.scratch_dropped.fetch_add(1, Ordering::Relaxed);
    }
}
#[derive(Debug)]
struct BackendError {
    text: Box<[u16]>,
    prior_default_self: bool,
    seen: Arc<Seen>,
}
impl BackendError {
    fn storage(units: usize) -> HtmlCspStorage {
        HtmlCspStorage::try_new(
            size_of::<Self>() + units * size_of::<u16>() + size_of::<Seen>(),
            units,
            3,
        )
        .unwrap()
    }
}
impl Drop for BackendError {
    fn drop(&mut self) {
        self.seen.errors_dropped.fetch_add(1, Ordering::Relaxed);
    }
}
struct Processor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    cancellation: HtmlParseCancellation,
    mode: Mode,
    seen: Arc<Seen>,
}
impl Processor {
    fn new(mode: Mode, cancellation: HtmlParseCancellation, seen: Arc<Seen>) -> Self {
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(17, &mut allow).unwrap();
        Self {
            authority,
            profile,
            cancellation,
            mode,
            seen,
        }
    }
}
impl HtmlMetaCspProcessor for Processor {
    type Error = BackendError;
    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>> {
        request
            .qualification()
            .processor()
            .validate_selected_with(&self.profile, admit)?;
        let prior = request
            .prior_policy()
            .borrow_native_with::<Policy, E>(&self.profile, admit)?;
        let q = request.qualification();
        admit(HtmlCspLayout::processing(
            u64::try_from(q.content().len()).unwrap() * 4 + 16,
            HtmlCspStorage::default(),
            HtmlCspDisposalWork::default(),
            size_of::<Self>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        assert_eq!(q.source(), HtmlMetaCspSource::Meta);
        assert_eq!(q.disposition(), HtmlMetaCspDisposition::Enforce);
        assert_eq!(
            q.removed_directives(),
            [
                HtmlMetaCspRemovedDirective::ReportUri,
                HtmlMetaCspRemovedDirective::FrameAncestors,
                HtmlMetaCspRemovedDirective::Sandbox
            ]
        );
        self.seen.calls.fetch_add(1, Ordering::Relaxed);
        if q.content()
            .code_units()
            .iter()
            .copied()
            .eq(FIRST.encode_utf16())
        {
            assert!(matches!(prior, HtmlCspNativePolicy::InitialEmpty));
            let layout = Policy::layout();
            admit(HtmlCspLayout::processing(
                4,
                layout.storage(),
                layout.disposal(),
                size_of::<Policy>(),
            )?)
            .map_err(HtmlCspError::Interrupted)?;
            self.seen.policies.fetch_add(1, Ordering::Relaxed);
            let artifact = Arc::new(Policy {
                default_self: true,
                seen: self.seen.clone(),
            });
            if self.mode == Mode::CancelAtAttestation {
                self.cancellation.cancel();
            }
            return match self.authority.attest_changed_with(request, artifact, admit) {
                Ok(prepared) => Ok(HtmlMetaCspOutcome::Completed(prepared)),
                Err(rejected) => {
                    drop(rejected.retained);
                    release(layout.storage());
                    Err(rejected.error)
                }
            };
        }
        // Only these exact valid serialized fixture inputs are recognized.
        // The prior artifact contains the complete previously accepted state;
        // the excluded directives never entered that state.
        if !q
            .content()
            .code_units()
            .iter()
            .copied()
            .eq(SECOND.encode_utf16())
        {
            return Ok(HtmlMetaCspOutcome::Unavailable(HtmlCspUnavailable::Profile));
        }
        let HtmlCspNativePolicy::External(prior) = prior else {
            panic!("second operation must consume the real first artifact")
        };
        assert!(prior.default_self);
        if self.mode == Mode::CancelBeforeScratch {
            self.cancellation.cancel();
        }
        let scratch_storage = Scratch::storage();
        admit(HtmlCspLayout::processing(
            (SCRATCH_UNITS as u64) * 2 + 7,
            scratch_storage,
            HtmlCspDisposalWork::try_new(7, size_of::<Scratch>())?,
            size_of::<Scratch>() + size_of::<[u16; SCRATCH_UNITS]>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        self.seen.scratch.fetch_add(1, Ordering::Relaxed);
        let scratch = Scratch {
            _backing: Box::new([0; SCRATCH_UNITS]),
            seen: self.seen.clone(),
        };
        if self.mode == Mode::CancelAfterScratch {
            self.cancellation.cancel();
        }
        let units = if self.mode == Mode::NativeLimit {
            REJECTED_UNITS
        } else {
            ERROR_UNITS
        };
        let error_storage = BackendError::storage(units);
        // Both the backing and its possible last-owner disposal are admitted
        // before allocation. The larger finite plan must be refused by the
        // supplied UTF-16 cap, before its allocation branch is entered.
        let admitted = admit(HtmlCspLayout::processing(
            u64::try_from(units).unwrap() * 2 + 32,
            error_storage,
            HtmlCspDisposalWork::try_new(9, size_of::<BackendError>())?,
            size_of::<BackendError>() + units * size_of::<u16>(),
        )?);
        if let Err(error) = admitted {
            drop(scratch);
            release(scratch_storage);
            return Err(HtmlCspError::Interrupted(error));
        }
        self.seen.errors.fetch_add(1, Ordering::Relaxed);
        let mut text: Box<[u16]> = match units {
            ERROR_UNITS => Box::new([0; ERROR_UNITS]),
            REJECTED_UNITS => Box::new([0; REJECTED_UNITS]),
            _ => unreachable!("closed native allocation plans"),
        };
        text[..q.content().len()].copy_from_slice(q.content().code_units());
        self.seen
            .error_pointer
            .store(text.as_ptr() as usize, Ordering::Relaxed);
        let error = BackendError {
            text,
            prior_default_self: prior.default_self,
            seen: self.seen.clone(),
        };
        // The temporary graph really disposes before its debit is released.
        // Returned native error custody remains live and is not released here.
        drop(scratch);
        release(scratch_storage);
        Ok(if self.mode == Mode::Refused {
            HtmlMetaCspOutcome::Refused(error)
        } else {
            HtmlMetaCspOutcome::Failed(error)
        })
    }
}
fn source() -> HtmlSource {
    HtmlSource::utf8(format!(
        "<!doctype html><head><meta http-equiv=content-security-policy content=\"{FIRST}\"><meta http-equiv=content-security-policy content=\"{SECOND}\"><meta http-equiv=content-security-policy content=\"{FIRST}\"></head>"
    ))
}
fn cleaned(failure: &HtmlParseCspFailure<BackendError>) -> &HtmlParseUsage {
    assert_eq!(failure.parse_failure().cleanup_error(), None);
    failure.parse_failure().after_cleanup().unwrap()
}

#[test]
fn refused_and_failed_keep_actual_error_backing_after_temporary_and_document_cleanup() {
    for (mode, expected) in [
        (Mode::Refused, HtmlParseCspFailureKind::Refused),
        (Mode::Failed, HtmlParseCspFailureKind::Failed),
    ] {
        let request = HtmlDocumentParseRequest::offline(source(), limits());
        let original = request.source().identity().clone();
        let seen = Arc::new(Seen::default());
        let mut processor = Processor::new(mode, request.cancellation().clone(), seen.clone());
        let failure = request.parse_with_meta_csp(&mut processor).unwrap_err();
        assert_eq!(failure.kind(), expected);
        assert_eq!(failure.parse_failure().source().identity(), &original);
        assert_eq!(
            count(&seen.calls),
            2,
            "the following reached operation must not execute after terminal native error"
        );
        assert_eq!(count(&seen.policies), 1);
        assert_eq!(count(&seen.policies_dropped), 1);
        assert_eq!(count(&seen.scratch), 1);
        assert_eq!(count(&seen.scratch_dropped), 1);
        assert_eq!(count(&seen.errors), 1);
        assert_eq!(count(&seen.errors_dropped), 0);
        let error = failure.backend_error().unwrap();
        assert!(error.prior_default_self);
        assert_eq!(error.text.as_ptr() as usize, count(&seen.error_pointer));
        assert!(
            error.text[..SECOND.encode_utf16().count()]
                .iter()
                .copied()
                .eq(SECOND.encode_utf16())
        );
        assert_eq!(
            cleaned(&failure)
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current(),
            ERROR_UNITS as u64
        );
        assert!(
            cleaned(&failure)
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current()
                >= BackendError::storage(ERROR_UNITS).bytes() as u64
        );
        let at_stop = failure.parse_failure().at_stop().unwrap();
        assert!(
            at_stop
                .resource(HtmlParseResource::ManagedUtf16Units)
                .high_water()
                >= (ERROR_UNITS + SCRATCH_UNITS) as u64
        );
        let spent = at_stop.resource(HtmlParseResource::WorkCredits).current();
        assert!(spent > 0);
        assert_eq!(
            cleaned(&failure)
                .resource(HtmlParseResource::WorkCredits)
                .current(),
            spent,
            "actual cleanup uses prepaid closing work after the original stop"
        );
        drop(processor);
        assert_eq!(
            count(&seen.errors_dropped),
            0,
            "borrowed service does not own the returned native error"
        );
        drop(failure);
        assert_eq!(count(&seen.errors_dropped), 1);
        assert_eq!(count(&seen.scratch_dropped), 1);
    }
}

#[test]
fn cancellation_before_and_after_temporary_formation_stops_before_error_allocation() {
    for (mode, scratch) in [
        (Mode::CancelBeforeScratch, 0),
        (Mode::CancelAfterScratch, 1),
    ] {
        let request = HtmlDocumentParseRequest::offline(source(), limits());
        let original = request.source().identity().clone();
        let seen = Arc::new(Seen::default());
        let mut processor = Processor::new(mode, request.cancellation().clone(), seen.clone());
        let failure = request.parse_with_meta_csp(&mut processor).unwrap_err();
        assert_eq!(
            failure.kind(),
            HtmlParseCspFailureKind::Interrupted(HtmlParseStop::Cancelled)
        );
        assert_eq!(
            failure.parse_failure().first_stop(),
            Some(HtmlParseStop::Cancelled)
        );
        assert_eq!(failure.parse_failure().source().identity(), &original);
        assert!(failure.backend_error().is_none());
        assert_eq!(count(&seen.calls), 2);
        assert_eq!(count(&seen.scratch), scratch);
        assert_eq!(count(&seen.scratch_dropped), scratch);
        assert_eq!(count(&seen.errors), 0);
        assert_eq!(count(&seen.policies_dropped), 1);
        assert_eq!(
            cleaned(&failure)
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current(),
            0
        );
        drop(failure);
        assert_eq!(count(&seen.errors_dropped), 0);
    }
}

#[test]
fn interrupted_attestation_disposes_the_prepaid_native_artifact_without_acceptance() {
    let request = HtmlDocumentParseRequest::offline(source(), limits());
    let original = request.source().identity().clone();
    let seen = Arc::new(Seen::default());
    let mut processor = Processor::new(
        Mode::CancelAtAttestation,
        request.cancellation().clone(),
        seen.clone(),
    );
    let failure = request.parse_with_meta_csp(&mut processor).unwrap_err();
    assert_eq!(
        failure.kind(),
        HtmlParseCspFailureKind::Interrupted(HtmlParseStop::Cancelled)
    );
    assert_eq!(failure.parse_failure().source().identity(), &original);
    assert_eq!(count(&seen.calls), 1);
    assert_eq!(count(&seen.policies), 1);
    assert_eq!(count(&seen.policies_dropped), 1);
    assert_eq!(count(&seen.scratch), 0);
    assert_eq!(count(&seen.errors), 0);
    assert!(failure.backend_error().is_none());
    assert_eq!(
        cleaned(&failure)
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current(),
        0
    );
}

#[test]
fn native_error_plan_over_finite_utf16_cap_refuses_before_backing_formation() {
    let request = HtmlDocumentParseRequest::offline(
        source(),
        limits()
            .into_builder()
            .managed_utf16_units((REJECTED_UNITS - 1) as u64)
            .finish()
            .unwrap(),
    );
    assert_eq!(
        request.limits().managed_utf16_units(),
        (REJECTED_UNITS - 1) as u64
    );
    let original = request.source().identity().clone();
    let seen = Arc::new(Seen::default());
    let mut processor = Processor::new(
        Mode::NativeLimit,
        request.cancellation().clone(),
        seen.clone(),
    );
    let failure = request.parse_with_meta_csp(&mut processor).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlParseCspFailureKind::Interrupted(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedUtf16Units,
            ..
        })
    ));
    assert_eq!(failure.parse_failure().source().identity(), &original);
    assert_eq!(count(&seen.calls), 2);
    assert_eq!(count(&seen.scratch), 1);
    assert_eq!(count(&seen.scratch_dropped), 1);
    assert_eq!(count(&seen.errors), 0);
    assert_eq!(count(&seen.policies_dropped), 1);
    assert!(failure.backend_error().is_none());
    assert_eq!(
        cleaned(&failure)
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current(),
        0
    );
}

#[test]
fn preexecution_zero_work_keeps_original_source_without_entering_the_processor() {
    let request = HtmlDocumentParseRequest::offline(
        source(),
        limits().into_builder().work_credits(0).finish().unwrap(),
    );
    let original = request.source().identity().clone();
    let seen = Arc::new(Seen::default());
    let mut processor = Processor::new(Mode::Refused, request.cancellation().clone(), seen.clone());
    let failure = request.parse_with_meta_csp(&mut processor).unwrap_err();
    assert_eq!(failure.kind(), HtmlParseCspFailureKind::Parse);
    assert!(matches!(
        failure.parse_failure().first_stop(),
        Some(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    assert_eq!(failure.parse_failure().source().identity(), &original);
    assert!(failure.backend_error().is_none());
    assert_eq!(count(&seen.calls), 0);
    assert_eq!(count(&seen.policies), 0);
    assert_eq!(count(&seen.scratch), 0);
    assert_eq!(count(&seen.errors), 0);
}
