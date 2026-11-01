//! Controlled request-derived native artifacts; this fixture is not a CSP engine.
use crate::parser::support::limits;
use std::{convert::Infallible, mem::size_of, sync::Arc};
use surgeist_html::*;
use surgeist_html_parser::*;

#[derive(Debug)]
struct Native {
    values: [u8; 8],
    len: usize,
}
impl HtmlCspPolicyArtifact for Native {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        native_layout()
    }
}
fn native_layout() -> HtmlCspArtifactLayout {
    HtmlCspArtifactLayout::try_new(
        HtmlCspStorage::try_new(size_of::<Native>(), 0, 1).unwrap(),
        HtmlCspDisposalWork::try_new(8, size_of::<Native>()).unwrap(),
    )
    .unwrap()
}
fn allow(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}
struct Processor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    calls: usize,
}
impl Processor {
    fn new() -> Self {
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(1, &mut allow).unwrap();
        Self {
            authority,
            profile,
            calls: 0,
        }
    }
}
impl HtmlMetaCspProcessor for Processor {
    type Error = ();
    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<()>, HtmlCspError<E>> {
        self.profile
            .validate_selected_with(request.qualification().processor(), admit)?;
        self.calls += 1;
        let prior = request
            .prior_policy()
            .borrow_native_with::<Native, E>(&self.profile, admit)?;
        let (mut values, mut len) = match prior {
            HtmlCspNativePolicy::InitialEmpty => ([0; 8], 0),
            HtmlCspNativePolicy::External(n) => (n.values, n.len),
        };
        let q = request.qualification();
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
        let mut unchanged = true;
        for directive in q.content().code_units().split(|u| *u == 59) {
            for _ in directive {
                admit(HtmlCspLayout::processing(
                    1,
                    Default::default(),
                    Default::default(),
                    0,
                )?)
                .map_err(HtmlCspError::Interrupted)?;
            }
            // Fixed controlled vocabulary with actual removed-directive branches.
            if directive.starts_with(&[114, 101, 112, 111, 114, 116, 45, 117, 114, 105])
                || directive.starts_with(&[
                    102, 114, 97, 109, 101, 45, 97, 110, 99, 101, 115, 116, 111, 114, 115,
                ])
                || directive.starts_with(&[115, 97, 110, 100, 98, 111, 120])
            {
                continue;
            }
            match directive {
                [61] => (),
                [33] => return Ok(HtmlMetaCspOutcome::Refused(())),
                [63] => return Ok(HtmlMetaCspOutcome::Failed(())),
                [126] => return Ok(HtmlMetaCspOutcome::Cancelled),
                [117] => {
                    return Ok(HtmlMetaCspOutcome::Unavailable(
                        HtmlCspUnavailable::NativeOwner,
                    ));
                }
                [v @ 49..=57] if len < values.len() => {
                    values[len] = (*v - 48) as u8;
                    len += 1;
                    unchanged = false;
                }
                _ => return Ok(HtmlMetaCspOutcome::Failed(())),
            }
        }
        if unchanged {
            return self
                .authority
                .attest_unchanged_with(request, admit)
                .map(HtmlMetaCspOutcome::Completed);
        }
        let layout = native_layout();
        admit(HtmlCspLayout::processing(
            4,
            layout.storage(),
            layout.disposal(),
            size_of::<Native>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        let native = Arc::new(Native { values, len });
        match self.authority.attest_changed_with(request, native, admit) {
            Ok(value) => Ok(HtmlMetaCspOutcome::Completed(value)),
            Err(rejected) => {
                drop(rejected.retained);
                release(layout.storage());
                Err(rejected.error)
            }
        }
    }
}
fn rows(report: &HtmlParseReport) -> Vec<HtmlParseDocumentMeta<'_>> {
    let mut rows = Vec::new();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(r) => {
                if let Some(meta) = r.document_meta() {
                    rows.push(meta)
                }
            }
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            _ => (),
        }
    }
    rows
}
fn parse(source: &str, processor: &mut Processor) -> HtmlParseReport {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
        .parse_with_meta_csp(processor)
        .unwrap();
    assert_eq!(
        report
            .document()
            .unwrap()
            .root()
            .document_focused_area_designation(),
        Some(HtmlFocusedAreaDesignation::KnownNull)
    );
    report
}
#[test]
fn reached_policies_use_exact_prior_artifacts_and_original_style_epochs() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><style>a{}</style><meta http-equiv=content-security-policy content='1;report-uri ignored;frame-ancestors ignored;sandbox ignored'><style>b{}</style><meta http-equiv=content-security-policy content='='><meta http-equiv=content-security-policy content='2'><style>c{}</style>",
        &mut p,
    );
    assert_eq!(p.calls, 3);
    let meta = rows(&report).remove(0);
    assert!(matches!(
        meta.disposition(HtmlParseMetaDomain::Csp),
        HtmlParseMetaDisposition::Complete
    ));
    let policy = meta.csp_association().unwrap();
    let native = policy
        .policy()
        .borrow_native_with::<Native, Infallible>(&p.profile, &mut allow)
        .unwrap();
    let HtmlCspNativePolicy::External(native) = native else {
        panic!("actual processed policy")
    };
    assert_eq!(&native.values[..native.len], &[1, 2]);
    let mut cursor = report.pending_associations();
    let mut policies = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Occurrence(r) if r.style().is_some() => {
                assert_eq!(
                    r.context()
                        .requirement(HtmlParseAssociationContextRole::InitialPolicy),
                    HtmlParseAssociationContextRequirement::Known
                );
                assert!(
                    r.context()
                        .slot(HtmlParseAssociationContextRole::InitialPolicy)
                        .is_none()
                );
                let p = r.context().csp_association().unwrap();
                if policies.last() != Some(p.policy().id()) {
                    policies.push(p.policy().id().clone());
                }
            }
            HtmlParsePendingAssociationStep::Complete => break,
            _ => (),
        }
    }
    assert_eq!(policies.len(), 3);
    assert_eq!(policies.last(), Some(policy.policy().id()));
}
#[test]
fn ignored_body_template_and_empty_meta_never_call_processor() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content=''><template><meta http-equiv=content-security-policy content='1'></template><body><meta http-equiv=content-security-policy content='2'>",
        &mut p,
    );
    assert_eq!(p.calls, 0);
    assert!(rows(&report)[0].csp_association().is_some());
}
#[test]
fn default_parse_does_not_claim_initial_policy_or_processed_csp() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><meta http-equiv=content-security-policy content='1'>"),
        limits(),
    )
    .parse()
    .unwrap();
    let meta = rows(&report).remove(0);
    assert!(meta.csp_association().is_none());
    assert!(matches!(
        meta.disposition(HtmlParseMetaDomain::Csp),
        HtmlParseMetaDisposition::Unavailable(_)
    ));
}
#[test]
fn no_change_completes_its_occurrence_without_changing_policy_identity() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content='='>",
        &mut p,
    );
    let meta = rows(&report).remove(0);
    let association = meta.csp_association().unwrap();
    assert_eq!(
        association.policy().id(),
        association.latest().unwrap().prior_policy()
    );
    assert_eq!(p.calls, 1);
}
#[test]
fn consuming_terminal_outcomes_preserve_typed_external_classification() {
    for (text, kind) in [
        ("!", HtmlParseCspFailureKind::Refused),
        ("?", HtmlParseCspFailureKind::Failed),
        ("~", HtmlParseCspFailureKind::Cancelled),
    ] {
        let source =
            format!("<!doctype html><meta http-equiv=content-security-policy content='{text}'>");
        let mut p = Processor::new();
        let error = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .parse_with_meta_csp(&mut p)
            .unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(
            error.parse_failure().first_stop(),
            Some(if text == "~" {
                HtmlParseStop::Cancelled
            } else {
                HtmlParseStop::CspProcessing
            })
        );
        assert_eq!(error.backend_error().is_some(), text != "~");
        assert_eq!(p.calls, 1);
        assert!(error.parse_failure().cleanup_error().is_none());
    }
}
#[test]
fn unavailable_transition_blocks_later_processing_on_unknown_prior() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content='u'><meta http-equiv=content-security-policy content='1'>",
        &mut p,
    );
    assert_eq!(p.calls, 1);
    assert!(matches!(
        rows(&report)[0].disposition(HtmlParseMetaDomain::Csp),
        HtmlParseMetaDisposition::Unavailable(_)
    ));
}

fn base_facts() -> surgeist_html_semantics::HtmlParseBaseFacts {
    use surgeist_html_semantics::*;
    let revision = crate::parser::support::revision();
    let fallback = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let observation = HtmlBaseParseObservation::new(
        "chosen".into(),
        fallback.clone(),
        encoding.clone(),
        revision.clone(),
        HtmlBaseParseOutcome::Failure,
    )
    .unwrap();
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision,
            Some(fallback),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding),
        )
        .unwrap(),
        vec![observation],
        HtmlBaseFactsLimits::new(4, 1000, 10000),
    )
    .unwrap()
}
#[test]
fn initial_base_basis_is_historical_unchanged_policy_matches_changed_policy_does_not() {
    for content in ["=", "1"] {
        let mut p = Processor::new();
        let report=HtmlDocumentParseRequest::offline(HtmlSource::utf8(format!("<!doctype html><base href=chosen><meta http-equiv=content-security-policy content='{content}'>")),limits()).with_initial_policy_base_facts(base_facts()).unwrap().parse_with_meta_csp(&mut p).unwrap();
        let meta = rows(&report).remove(0);
        let basis = meta.base_policy_association().unwrap();
        let current = meta.csp_association().unwrap();
        assert_eq!(basis.policy().id() == current.policy().id(), content == "=");
        assert!(basis.latest().is_none());
        assert!(current.latest().is_some());
    }
}
#[test]
fn initial_base_bank_is_used_only_when_explicitly_paired_and_still_current() {
    for (content, qualify, replace, success) in [
        ("=", true, false, true),
        ("1", true, false, false),
        ("=", false, false, false),
        ("=", true, true, false),
    ] {
        let mut p = Processor::new();
        let source = HtmlSource::utf8(format!(
            "<!doctype html><meta http-equiv=content-security-policy content='{content}'><base href=chosen>"
        ));
        let request = HtmlDocumentParseRequest::offline(source, limits());
        let mut request = if qualify {
            request
                .with_initial_policy_base_facts(base_facts())
                .unwrap()
        } else {
            request.with_base_facts(base_facts()).unwrap()
        };
        if replace {
            request = request.with_base_facts(base_facts()).unwrap();
        }
        let result = request.parse_with_meta_csp(&mut p);
        assert_eq!(result.is_ok(), success, "{content}/{qualify}/{replace}");
        if let Err(failure) = result {
            assert_eq!(failure.parse_failure().cleanup_error(), None);
            assert!(matches!(
                failure.parse_failure().first_stop(),
                Some(HtmlParseStop::BaseFacts(_))
            ));
        }
    }
}
#[test]
fn unavailable_policy_lenders_cannot_present_last_completed_policy_as_current() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content=1><style>a{}</style><meta http-equiv=content-security-policy content=u><style>b{}</style>",
        &mut p,
    );
    let meta = rows(&report).remove(0);
    assert!(meta.csp_association().is_none());
    let mut cursor = report.pending_associations();
    let (mut known, mut missing) = (0, 0);
    loop {
        match cursor.advance() {
            HtmlParsePendingAssociationStep::Occurrence(row) if row.style().is_some() => {
                let context = row.context();
                match context.requirement(HtmlParseAssociationContextRole::InitialPolicy) {
                    HtmlParseAssociationContextRequirement::Known => {
                        assert!(context.csp_association().is_some());
                        known += 1;
                    }
                    HtmlParseAssociationContextRequirement::Unavailable(_) => {
                        assert!(context.csp_association().is_none());
                        assert!(
                            context
                                .slot(HtmlParseAssociationContextRole::InitialPolicy)
                                .is_none()
                        );
                        missing += 1;
                    }
                    other => panic!("unexpected policy {other:?}"),
                }
            }
            HtmlParsePendingAssociationStep::Complete => break,
            _ => (),
        }
    }
    assert!(known > 0 && missing > 0);
}
#[test]
fn independent_documents_and_wrong_processor_results_preserve_authentic_identity() {
    let mut p = Processor::new();
    let a = parse(
        "<!doctype html><meta http-equiv=content-security-policy content=1>",
        &mut p,
    );
    let b = parse(
        "<!doctype html><meta http-equiv=content-security-policy content=1>",
        &mut p,
    );
    let am = rows(&a).remove(0);
    let bm = rows(&b).remove(0);
    let ac = am.csp_association().unwrap();
    let bc = bm.csp_association().unwrap();
    assert_ne!(ac.document(), bc.document());
    assert_ne!(ac.policy().id(), bc.policy().id());
    let other = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    p.profile = other.profile_with(2, &mut allow).unwrap();
    let failure = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><meta http-equiv=content-security-policy content=1>"),
        limits(),
    )
    .parse_with_meta_csp(&mut p)
    .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlParseCspFailureKind::Protocol(_)
    ));
    assert!(matches!(
        failure.parse_failure().first_stop(),
        Some(HtmlParseStop::CspProtocol(_))
    ));
    assert_eq!(failure.parse_failure().cleanup_error(), None);
}
#[test]
fn non_power_of_two_original_content_owns_its_real_utf16_capacity() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content='1;report-uri xyz'>",
        &mut p,
    );
    assert_eq!(p.calls, 1);
    let meta = rows(&report).remove(0);
    let text = meta.csp_association().unwrap().latest().unwrap().content();
    assert_eq!(
        text.code_units(),
        "1;report-uri xyz".encode_utf16().collect::<Vec<_>>()
    );
    assert!(
        report.retained_construction_utf16_units() >= text.shared_code_units().capacity() as u64
    );
}

#[test]
fn after_head_delegation_preserves_the_borrowed_service_but_body_meta_is_ignored() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><head></head><meta http-equiv=content-security-policy content=1><body><div><meta http-equiv=content-security-policy content=2></div>",
        &mut p,
    );
    assert_eq!(p.calls, 1);
    let meta = rows(&report).remove(0);
    let HtmlCspNativePolicy::External(native) = meta
        .csp_association()
        .unwrap()
        .policy()
        .borrow_native_with::<Native, Infallible>(&p.profile, &mut allow)
        .unwrap()
    else {
        panic!("processed delegated Meta")
    };
    assert_eq!(&native.values[..native.len], &[1]);
}

#[test]
fn finite_original_storage_and_work_boundaries_release_partial_csp_state() {
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::WorkCredits,
    ] {
        for cap in [0, 128, 1024, 4096, 16384, 65536] {
            let builder = limits().into_builder();
            let limits = match resource {
                HtmlParseResource::ManagedCapacityBytes => builder.managed_capacity_bytes(cap),
                HtmlParseResource::WorkCredits => builder.work_credits(cap),
                _ => unreachable!(),
            }
            .finish()
            .unwrap();
            let mut p = Processor::new();
            let result=HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><meta http-equiv=content-security-policy content=1><style>a{}</style>"),limits).parse_with_meta_csp(&mut p);
            if let Err(failure) = result {
                assert_eq!(
                    failure.parse_failure().cleanup_error(),
                    None,
                    "{resource:?}/{cap}"
                );
                if let Some(usage) = failure.parse_failure().after_cleanup() {
                    for retained in [
                        HtmlParseResource::ManagedCapacityBytes,
                        HtmlParseResource::ManagedUtf16Units,
                        HtmlParseResource::BufferedUtf16Units,
                        HtmlParseResource::AuxiliaryEntries,
                    ] {
                        assert_eq!(
                            usage.resource(retained).current(),
                            0,
                            "{resource:?}/{cap}/{retained:?}"
                        );
                    }
                }
            }
        }
    }
}
struct ExactUtf16Processor(Processor);
impl HtmlMetaCspProcessor for ExactUtf16Processor {
    type Error = ();
    fn profile(&self) -> &HtmlCspProcessorProfile {
        self.0.profile()
    }
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        _: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<()>, HtmlCspError<E>> {
        self.profile()
            .validate_selected_with(request.qualification().processor(), admit)?;
        admit(HtmlCspLayout::processing(
            3,
            Default::default(),
            Default::default(),
            0,
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        assert_eq!(
            request.qualification().content().code_units(),
            [0xD800, 32, 0xDC00]
        );
        self.0.calls += 1;
        Ok(HtmlMetaCspOutcome::Refused(()))
    }
}
#[test]
fn processor_receives_exact_non_scalar_original_utf16_attribute_units() {
    let mut units = "<!doctype html><meta http-equiv=content-security-policy content='"
        .encode_utf16()
        .collect::<Vec<_>>();
    units.extend([0xD800, 32, 0xDC00]);
    units.extend("'>".encode_utf16());
    let mut p = ExactUtf16Processor(Processor::new());
    let failure = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(),
    )
    .parse_with_meta_csp(&mut p)
    .unwrap_err();
    assert_eq!(p.0.calls, 1);
    assert_eq!(failure.kind(), HtmlParseCspFailureKind::Refused);
    assert_eq!(failure.parse_failure().cleanup_error(), None);
}

#[test]
fn separately_constructed_template_documents_have_distinct_known_initial_policy() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content=1><template><style>a{}</style><meta http-equiv=content-security-policy content=2></template>",
        &mut p,
    );
    assert_eq!(p.calls, 1);
    let documents = rows(&report);
    assert!(documents.len() > 1);
    let root = documents[0].csp_association().unwrap();
    for document in &documents[1..] {
        if let Some(association) = document.csp_association() {
            assert_ne!(association.document(), root.document());
            assert!(association.latest().is_none());
            assert!(matches!(
                association
                    .policy()
                    .borrow_native_with::<Native, Infallible>(&p.profile, &mut allow)
                    .unwrap(),
                HtmlCspNativePolicy::InitialEmpty
            ));
        }
    }
}

#[test]
fn template_without_style_or_link_retains_its_fresh_constructor_policy() {
    let mut p = Processor::new();
    let report = parse(
        "<!doctype html><meta http-equiv=content-security-policy content=1><template><span>inert</span></template>",
        &mut p,
    );
    let documents = rows(&report);
    assert_eq!(p.calls, 1);
    assert_eq!(documents.len(), 2);
    let root = documents[0].csp_association().unwrap();
    let inert = documents[1].csp_association().unwrap();
    assert_ne!(inert.document(), root.document());
    assert!(inert.latest().is_none());
    assert!(matches!(
        inert
            .policy()
            .borrow_native_with::<Native, Infallible>(&p.profile, &mut allow)
            .unwrap(),
        HtmlCspNativePolicy::InitialEmpty
    ));
}
