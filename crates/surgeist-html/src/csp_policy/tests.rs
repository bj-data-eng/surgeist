//! Controlled external adapter tests. This finite fixture protocol is not a CSP
//! grammar or a claim about an absent production enforcement engine.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn allow(_: HtmlCspLayout) -> Result<(), Stop> {
    Ok(())
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stop {
    Cancelled,
    Work,
    Storage,
    Atomic,
}

#[derive(Debug)]
struct Budget {
    work: u64,
    spent: u64,
    storage: usize,
    atomic: usize,
    cancelled: bool,
    first: Option<Stop>,
    layouts: Vec<HtmlCspLayout>,
}
impl Budget {
    fn unlimited() -> Self {
        Self {
            work: u64::MAX,
            spent: 0,
            storage: usize::MAX,
            atomic: usize::MAX,
            cancelled: false,
            first: None,
            layouts: Vec::new(),
        }
    }
    fn admit(&mut self, layout: HtmlCspLayout) -> Result<(), Stop> {
        if let Some(stop) = self.first {
            return Err(stop);
        }
        let charge = layout
            .work_credits()
            .checked_add(layout.closing_work().work_credits())
            .unwrap();
        let stop = if self.cancelled {
            Some(Stop::Cancelled)
        } else if charge > self.work {
            Some(Stop::Work)
        } else if layout.peak().bytes() > self.storage {
            Some(Stop::Storage)
        } else if layout.atomic_bytes() > self.atomic {
            Some(Stop::Atomic)
        } else {
            None
        };
        if let Some(stop) = stop {
            self.first = Some(stop);
            return Err(stop);
        }
        self.work -= charge;
        self.spent += charge;
        self.layouts.push(layout);
        Ok(())
    }
}

#[derive(Debug, Default)]
struct Observation {
    metadata: AtomicUsize,
    drops: AtomicUsize,
    last_value: AtomicUsize,
}
#[derive(Debug)]
struct NativePolicy {
    values: [u16; 4],
    len: usize,
    observation: Arc<Observation>,
}
impl NativePolicy {
    fn layout() -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>() + size_of::<Observation>(), 0, 2).unwrap(),
            HtmlCspDisposalWork::try_new(7, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
}
impl HtmlCspPolicyArtifact for NativePolicy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        self.observation.metadata.fetch_add(1, Ordering::Relaxed);
        Self::layout()
    }
}
impl Drop for NativePolicy {
    fn drop(&mut self) {
        self.observation.last_value.store(
            usize::from(self.values[self.len.saturating_sub(1)]),
            Ordering::Relaxed,
        );
        self.observation.drops.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct OtherNative;
impl HtmlCspPolicyArtifact for OtherNative {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
enum NativeError {
    Capacity,
    ServiceClosed,
}

struct Processor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    observation: Arc<Observation>,
    calls: usize,
    service_open: bool,
}
impl Processor {
    fn new() -> Self {
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(17, &mut allow).unwrap();
        Self {
            authority,
            profile,
            observation: Arc::default(),
            calls: 0,
            service_open: true,
        }
    }
}
impl HtmlMetaCspProcessor for Processor {
    type Error = NativeError;
    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>> {
        if !self.service_open {
            return Ok(HtmlMetaCspOutcome::Unavailable(
                HtmlCspUnavailable::NativeOwner,
            ));
        }
        request
            .qualification()
            .processor()
            .validate_selected_with(&self.profile, admit)?;
        let prior = request
            .prior_policy()
            .borrow_native_with::<NativePolicy, E>(&self.profile, admit)?;
        let q = request.qualification();
        let units = u64::try_from(q.content().len())
            .map_err(|_| HtmlCspProtocolError::RepresentationExhausted)?;
        admit(HtmlCspLayout::processing(
            units + 12,
            HtmlCspStorage::default(),
            HtmlCspDisposalWork::default(),
            size_of::<[u16; 4]>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        assert_eq!(q.source(), HtmlMetaCspSource::Meta);
        assert_eq!(q.disposition(), HtmlMetaCspDisposition::Enforce);
        assert_eq!(q.trigger(), HtmlMetaCspTrigger::Insertion);
        assert_eq!(
            q.removed_directives(),
            [
                HtmlMetaCspRemovedDirective::ReportUri,
                HtmlMetaCspRemovedDirective::FrameAncestors,
                HtmlMetaCspRemovedDirective::Sandbox
            ]
        );
        let (mut values, len) = match prior {
            HtmlCspNativePolicy::InitialEmpty => ([0; 4], 0),
            HtmlCspNativePolicy::External(prior) => (prior.values, prior.len),
        };
        self.calls += 1;
        // Explicit controlled fixture inputs, not a CSP parser. The actual
        // retained prior native array determines every complete result.
        if q.content().code_units() == [b'=' as u16] {
            return self
                .authority
                .attest_unchanged_with(request, admit)
                .map(HtmlMetaCspOutcome::Completed);
        }
        if q.content().code_units() == [b'!' as u16] {
            return Ok(HtmlMetaCspOutcome::Refused(NativeError::Capacity));
        }
        if q.content().code_units() == [b'~' as u16] {
            return Ok(HtmlMetaCspOutcome::Cancelled);
        }
        if len == values.len() {
            return Ok(HtmlMetaCspOutcome::Refused(NativeError::Capacity));
        }
        values[len] = match q.content().code_units() {
            [49] => 101,
            [50] => 202,
            [51] => 303,
            _ => return Ok(HtmlMetaCspOutcome::Failed(NativeError::ServiceClosed)),
        };
        let native_layout = NativePolicy::layout();
        admit(HtmlCspLayout::processing(
            3,
            native_layout.storage(),
            native_layout.disposal(),
            size_of::<NativePolicy>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        let artifact = Arc::new(NativePolicy {
            values,
            len: len + 1,
            observation: self.observation.clone(),
        });
        match self.authority.attest_changed_with(request, artifact, admit) {
            Ok(prepared) => Ok(HtmlMetaCspOutcome::Completed(prepared)),
            Err(rejected) => {
                drop(rejected.retained);
                release(native_layout.storage());
                Err(rejected.error)
            }
        }
    }
}

fn pair() -> (HtmlCspOwner, HtmlCspAssociation) {
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let association = owner.initial_document_with(&mut allow).unwrap();
    (owner, association)
}
fn request(
    owner: &mut HtmlCspOwner,
    current: &HtmlCspAssociation,
    profile: &HtmlCspProcessorProfile,
    text: &str,
) -> HtmlMetaCspRequest {
    owner
        .request_meta_insertion_with(current, profile, DocumentString::from(text), &mut allow)
        .unwrap()
}
fn prepared(processor: &mut Processor, request: &HtmlMetaCspRequest) -> HtmlMetaCspPrepared {
    match processor.prepare(request, &mut allow, &mut |_| {}).unwrap() {
        HtmlMetaCspOutcome::Completed(prepared) => prepared,
        other => panic!("expected actual fixture processing: {other:?}"),
    }
}
fn publish(
    owner: &mut HtmlCspOwner,
    current: &mut HtmlCspAssociation,
    request: HtmlMetaCspRequest,
    prepared: HtmlMetaCspPrepared,
) -> HtmlMetaCspCompletion {
    owner
        .prepare_publication_with(current, request, prepared, &mut allow)
        .unwrap()
        .publish()
}
fn native<'a>(
    state: &'a HtmlCspPolicyState,
    profile: &HtmlCspProcessorProfile,
) -> &'a NativePolicy {
    match state
        .borrow_native_with::<NativePolicy, _>(profile, &mut allow)
        .unwrap()
    {
        HtmlCspNativePolicy::External(native) => native,
        HtmlCspNativePolicy::InitialEmpty => panic!("expected native artifact"),
    }
}
fn protocol<T: fmt::Debug>(result: Result<T, HtmlCspError<Stop>>, expected: HtmlCspProtocolError) {
    assert!(matches!(result, Err(HtmlCspError::Protocol(actual)) if actual == expected));
}

#[test]
fn exact_utf16_and_fixed_meta_contract_retain_original_backing() {
    let (mut owner, association) = pair();
    let p = Processor::new();
    let content = DocumentString::from_utf16([0xd800, 0, 13, 10, 0xdfff]);
    let backing = content.shared_code_units().clone();
    let request = owner
        .request_meta_insertion_with(&association, p.profile(), content, &mut allow)
        .unwrap();
    assert!(Arc::ptr_eq(
        &backing,
        request.qualification().content().shared_code_units()
    ));
    assert_eq!(
        request.qualification().content().code_units(),
        [0xd800, 0, 13, 10, 0xdfff]
    );
    assert_eq!(request.prior_policy(), association.policy());
    assert!(matches!(
        association
            .policy()
            .borrow_native_with::<NativePolicy, _>(p.profile(), &mut allow)
            .unwrap(),
        HtmlCspNativePolicy::InitialEmpty
    ));
}

#[test]
fn changed_processing_keeps_old_snapshot_and_returns_actual_retired_association() {
    let (mut owner, mut association) = pair();
    let snapshot = association.clone();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let occurrence = r.qualification().occurrence().clone();
    let value = prepared(&mut p, &r);
    let policy = value.policy().clone();
    let completion = publish(&mut owner, &mut association, r, value);
    assert_eq!(completion.retired().policy(), snapshot.policy());
    assert_eq!(completion.retired().generation(), snapshot.generation());
    assert_eq!(completion.receipt().policy(), &policy);
    assert_eq!(association.policy(), &policy);
    assert_eq!(association.latest().unwrap().occurrence(), &occurrence);
    assert_ne!(association.generation(), snapshot.generation());
    assert_ne!(association.policy(), snapshot.policy());
    assert_eq!(
        &native(association.policy(), p.profile()).values[..1],
        [101]
    );
    assert!(snapshot.policy().profile().is_none());
}

#[test]
fn nochange_completes_once_and_a_second_old_basis_result_is_stale() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let first = request(&mut owner, &association, p.profile(), "=");
    let other = request(&mut owner, &association, p.profile(), "=");
    assert_ne!(
        first.qualification().occurrence(),
        other.qualification().occurrence()
    );
    let first_value = prepared(&mut p, &first);
    let other_value = prepared(&mut p, &other);
    let original = association.policy().clone();
    let generation = association.generation().clone();
    let done = publish(&mut owner, &mut association, first, first_value);
    assert_eq!(association.policy(), &original);
    assert_ne!(association.generation(), &generation);
    assert_eq!(done.receipt().policy(), &original);
    let rejection = owner
        .prepare_publication_with(&mut association, other, other_value, &mut allow)
        .unwrap_err();
    assert!(matches!(
        rejection.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::StaleAssociation)
    ));
    assert_eq!(rejection.retained.0.prior_policy(), &original);
    assert_eq!(p.calls, 2);
}

#[test]
fn wrong_owner_document_request_and_actual_prior_are_rejected_without_replacement() {
    let (mut owner, mut association) = pair();
    let (mut foreign, _) = pair();
    let mut p = Processor::new();
    let raw = DocumentString::from("1");
    let error = foreign
        .request_meta_insertion_with(&association, p.profile(), raw, &mut allow)
        .unwrap_err();
    assert!(matches!(
        error.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::ForeignOwner)
    ));
    assert_eq!(error.retained.code_units(), [49]);
    let mut other_doc = owner.initial_document_with(&mut allow).unwrap();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    let rejected = owner
        .prepare_publication_with(&mut other_doc, r, value, &mut allow)
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::ForeignDocument)
    ));
    let (r, value) = rejected.retained;
    let other_r = request(&mut owner, &association, p.profile(), "2");
    let rejected = owner
        .prepare_publication_with(&mut association, other_r, value, &mut allow)
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::WrongRequest)
    ));
    let (_, value) = rejected.retained;
    let error = foreign
        .prepare_publication_with(&mut association, r, value, &mut allow)
        .unwrap_err();
    assert!(matches!(
        error.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::ForeignOwner)
    ));
    assert!(association.policy().profile().is_none());
}

#[test]
fn profile_precedes_native_type_and_same_numeric_version_is_foreign() {
    let (mut owner, mut association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    let value = prepared(&mut p, &r);
    drop(publish(&mut owner, &mut association, r, value));
    let foreign = Processor::new();
    assert_eq!(
        p.profile().semantic_version(),
        foreign.profile().semantic_version()
    );
    protocol(
        association
            .policy()
            .borrow_native_with::<OtherNative, _>(foreign.profile(), &mut allow),
        HtmlCspProtocolError::WrongProcessor,
    );
    let changed_profile = p.authority.profile_with(18, &mut allow).unwrap();
    protocol(
        association
            .policy()
            .borrow_native_with::<OtherNative, _>(&changed_profile, &mut allow),
        HtmlCspProtocolError::WrongProfile,
    );
    protocol(
        association
            .policy()
            .borrow_native_with::<OtherNative, _>(p.profile(), &mut allow),
        HtmlCspProtocolError::WrongNativeArtifactType,
    );
    let rejected = owner
        .request_meta_insertion_with(
            &association,
            &changed_profile,
            DocumentString::from("2"),
            &mut allow,
        )
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::WrongProfile)
    ));
    assert_eq!(native(association.policy(), p.profile()).len, 1);
    assert_eq!(p.calls, 1, "borrowing performs no native processing");
}

#[test]
fn two_policies_after_report_only_transfer_use_actual_retained_native_predecessors() {
    let (mut parser, mut report) = pair();
    let mut p = Processor::new();
    let r = request(&mut parser, &report, p.profile(), "1");
    let value = prepared(&mut p, &r);
    drop(publish(&mut parser, &mut report, r, value));
    let original_document = report.document().clone();
    let original_policy = report.policy().id().clone();
    let before_installation = p.calls;
    // `report` is now the only strong policy custodian: processor owns no
    // native value, typed mirror or lookup index, and all inputs were dropped.
    let (mut dom, _) = pair();
    let mut installed = dom
        .associate_document_with(&report, HtmlCspAssociationCause::Installation, &mut allow)
        .unwrap();
    assert_ne!(installed.document(), &original_document);
    assert_eq!(installed.policy().id(), &original_policy);
    assert_eq!(installed.origin().original_document(), &original_document);
    assert_eq!(
        p.calls, before_installation,
        "installation did not invoke the processor"
    );
    drop(report);
    drop(parser);
    let old_snapshot = installed.clone();
    let r = request(&mut dom, &installed, p.profile(), "2");
    let value = prepared(&mut p, &r);
    drop(publish(&mut dom, &mut installed, r, value));
    assert_eq!(
        &native(installed.policy(), p.profile()).values[..2],
        [101, 202]
    );
    let r = request(&mut dom, &installed, p.profile(), "3");
    let value = prepared(&mut p, &r);
    drop(publish(&mut dom, &mut installed, r, value));
    assert_eq!(
        &native(installed.policy(), p.profile()).values[..3],
        [101, 202, 303]
    );
    assert_eq!(
        &native(old_snapshot.policy(), p.profile()).values[..1],
        [101]
    );
    assert_eq!(p.calls, 3, "installation did not invoke the processor");
    assert_eq!(
        p.observation.metadata.load(Ordering::Relaxed),
        3,
        "descriptors cached once per actual artifact"
    );
    let stale_request = request(&mut dom, &old_snapshot, p.profile(), "=");
    let stale_result = prepared(&mut p, &stale_request);
    let rejected = dom
        .prepare_publication_with(&mut installed, stale_request, stale_result, &mut allow)
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::StaleAssociation)
    ));
    assert_eq!(
        &native(installed.policy(), p.profile()).values[..3],
        [101, 202, 303]
    );
    let inherited = dom
        .associate_document_with(&installed, HtmlCspAssociationCause::Inheritance, &mut allow)
        .unwrap();
    assert_ne!(inherited.document(), installed.document());
    assert_eq!(inherited.policy(), installed.policy());
    assert!(matches!(
        inherited.origin().transferred_from(),
        Some((_, _, HtmlCspAssociationCause::Inheritance))
    ));
}

#[test]
fn service_unavailability_refusal_failure_and_cancellation_are_not_empty_success() {
    let (mut owner, association) = pair();
    let mut p = Processor::new();
    let r = request(&mut owner, &association, p.profile(), "1");
    p.service_open = false;
    assert!(matches!(
        p.prepare(&r, &mut allow, &mut |_| {}).unwrap(),
        HtmlMetaCspOutcome::Unavailable(HtmlCspUnavailable::NativeOwner)
    ));
    p.service_open = true;
    let mut cancelled = Budget::unlimited();
    cancelled.cancelled = true;
    assert!(matches!(
        p.prepare(&r, &mut |layout| cancelled.admit(layout), &mut |_| {}),
        Err(HtmlCspError::Interrupted(Stop::Cancelled))
    ));
    assert_eq!(p.calls, 0);
    let failure_r = request(
        &mut owner,
        &association,
        p.profile(),
        "unsupported fixture input",
    );
    assert!(matches!(
        p.prepare(&failure_r, &mut allow, &mut |_| {}).unwrap(),
        HtmlMetaCspOutcome::Failed(NativeError::ServiceClosed)
    ));
    let refusal = request(&mut owner, &association, p.profile(), "!");
    let cancellation = request(&mut owner, &association, p.profile(), "~");
    assert!(matches!(
        p.prepare(&refusal, &mut allow, &mut |_| {}).unwrap(),
        HtmlMetaCspOutcome::Refused(NativeError::Capacity)
    ));
    assert!(matches!(
        p.prepare(&cancellation, &mut allow, &mut |_| {}).unwrap(),
        HtmlMetaCspOutcome::Cancelled
    ));
    assert!(association.policy().profile().is_none());
}

#[test]
fn equal_generation_private_branches_still_require_the_exact_prior_artifact() {
    let (mut owner, original) = pair();
    let mut a = original.clone();
    let mut b = original;
    let mut p = Processor::new();
    let r = request(&mut owner, &a, p.profile(), "1");
    let value = prepared(&mut p, &r);
    drop(publish(&mut owner, &mut a, r, value));
    let r = request(&mut owner, &b, p.profile(), "2");
    let value = prepared(&mut p, &r);
    drop(publish(&mut owner, &mut b, r, value));
    assert_eq!(a.generation(), b.generation());
    let r = request(&mut owner, &a, p.profile(), "=");
    let value = prepared(&mut p, &r);
    let rejected = owner
        .prepare_publication_with(&mut b, r, value, &mut allow)
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        HtmlCspError::Protocol(HtmlCspProtocolError::PriorPolicyMismatch)
    ));
    assert_eq!(&native(b.policy(), p.profile()).values[..1], [202]);
}

mod admission;
mod disposal;
