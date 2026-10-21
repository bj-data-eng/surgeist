//! Authentic original CSP artifacts transfer; DOM never invokes the processor.
use super::*;
use surgeist_html::*;
use surgeist_html_parser::HtmlParseMetadataStep;

#[derive(Debug)]
struct RecordedPolicy {
    text: DocumentString,
    padding: [u8; 1024],
}
impl HtmlCspPolicyArtifact for RecordedPolicy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        let units = self.text.shared_code_units().capacity();
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(
                size_of::<Self>() + size_of::<Vec<u16>>() + units * 2,
                units,
                2,
            )
            .unwrap(),
            HtmlCspDisposalWork::try_new(7, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
}
struct Processor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    calls: usize,
    unchanged: bool,
    unchanged_on_call: Option<usize>,
    last: Option<std::sync::Weak<RecordedPolicy>>,
}
impl Processor {
    fn new() -> Self {
        let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(1, &mut allow).unwrap();
        Self {
            authority,
            profile,
            calls: 0,
            unchanged: false,
            unchanged_on_call: None,
            last: None,
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
        if self.unchanged || self.unchanged_on_call == Some(self.calls) {
            self.calls += 1;
            return self
                .authority
                .attest_unchanged_with(request, admit)
                .map(HtmlMetaCspOutcome::Completed);
        }
        let value = RecordedPolicy {
            text: request.qualification().content().clone(),
            padding: [37; 1024],
        };
        let layout = value.retained_layout();
        admit(HtmlCspLayout::processing(
            9,
            layout.storage(),
            layout.disposal(),
            size_of::<Self>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        let native = Arc::new(value);
        self.last = Some(Arc::downgrade(&native));
        self.calls += 1;
        match self.authority.attest_changed_with(request, native, admit) {
            Ok(prepared) => Ok(HtmlMetaCspOutcome::Completed(prepared)),
            Err(rejected) => {
                drop(rejected.retained);
                release(layout.storage());
                Err(rejected.error)
            }
        }
    }
}
fn processed(text: &str, processor: &mut Processor) -> Arc<HtmlParseReport> {
    Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), parse_limits())
            .parse_with_meta_csp(processor)
            .unwrap(),
    )
}
fn original(report: &HtmlParseReport) -> &HtmlCspAssociation {
    let root = report.document().unwrap().root().id();
    let mut rows = report.metadata();
    loop {
        match rows.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(meta) = row.document_meta()
                    && meta.document().id() == root
                {
                    return meta.csp_association().unwrap();
                }
            }
            HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => panic!("actual Document policy"),
        }
    }
}
fn associated(snapshot: &DomSnapshot, document: DocumentId) -> &HtmlCspAssociation {
    match snapshot
        .document(document)
        .unwrap()
        .metadata()
        .policy_container()
        .csp()
    {
        DomDocumentCspState::Associated(value) => value,
        _ => panic!("qualified original transfer"),
    }
}
#[test]
fn constructor_and_unqualified_report_have_distinct_policy_knowledge() {
    let (mut dom, env) = setup();
    let mut b = DomBatch::new();
    let created = b.create_document();
    let document = dom
        .apply(b, &AtomicBool::new(false))
        .unwrap()
        .document(&created)
        .unwrap();
    assert!(matches!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .csp(),
        DomDocumentCspState::ConstructorInitialEmpty
    ));
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            parse(HtmlSource::utf8("<!doctype html><p>unqualified")),
            env,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(matches!(
        installed
            .snapshot()
            .document(installed.document())
            .unwrap()
            .metadata()
            .policy_container()
            .csp(),
        DomDocumentCspState::Unavailable
    ));
}
#[test]
fn exact_original_artifact_transfers_twice_without_processing_and_survives_report_drop() {
    let mut service = Processor::new();
    let report = processed(
        "<!doctype html><meta http-equiv=content-security-policy content='first'><meta http-equiv=content-security-policy content='last'><p>x",
        &mut service,
    );
    assert_eq!(service.calls, 2);
    let source = original(&report).clone();
    let weak = service.last.as_ref().unwrap().clone();
    let (mut dom, env) = setup();
    let one = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report.clone(),
            env.clone(),
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let two = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report.clone(),
            env,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let first = associated(one.snapshot(), one.document());
    let second = associated(two.snapshot(), two.document());
    assert_ne!(first.document(), source.document());
    assert_ne!(first.document(), second.document());
    assert_eq!(first.policy(), source.policy());
    assert_eq!(second.policy(), source.policy());
    assert_eq!(
        first.latest().unwrap().content(),
        &DocumentString::from("last")
    );
    assert_eq!(
        first.origin().transferred_from(),
        Some((
            source.document(),
            source.generation(),
            HtmlCspAssociationCause::Installation
        ))
    );
    let snapshot = one.snapshot().clone();
    let document = one.document();
    drop((one, two, report, source, service));
    let policy = associated(&snapshot, document).policy();
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let HtmlCspNativePolicy::External(native) = policy
        .borrow_native_with::<RecordedPolicy, _>(policy.profile().unwrap(), &mut allow)
        .unwrap()
    else {
        panic!("actual native artifact")
    };
    assert_eq!(native.text, DocumentString::from("last"));
    assert_eq!(native.padding, [37; 1024]);
    assert!(weak.upgrade().is_some());
    drop(dom);
    assert!(
        weak.upgrade().is_some(),
        "immutable snapshot owns actual graph"
    );
    drop(snapshot);
    assert!(weak.upgrade().is_none());
}
#[test]
fn qualified_original_style_uses_its_policy_without_offline_binding_or_final_policy_substitution() {
    let mut service = Processor::new();
    let report = processed(
        "<!doctype html><head><meta http-equiv=content-security-policy content='policy A'><style nonce='original-nonce'>body{color:red}</style><meta http-equiv=content-security-policy content='policy B'>",
        &mut service,
    );
    assert_eq!(service.calls, 2);
    let policy_b = original(&report).clone();
    let mut records = report.pending_associations();
    let policy_a = loop {
        match records.advance() {
            surgeist_html_parser::HtmlParsePendingAssociationStep::Occurrence(row)
                if row.requires_processing() && row.style().is_some() =>
            {
                break row.context().csp_association().unwrap().clone();
            }
            surgeist_html_parser::HtmlParsePendingAssociationStep::Complete => {
                panic!("actual original Style occurrence")
            }
            _ => (),
        }
    };
    assert_ne!(policy_a.policy().id(), policy_b.policy().id());
    assert_eq!(
        policy_a.latest().unwrap().content(),
        &DocumentString::from("policy A")
    );
    assert_eq!(
        policy_b.latest().unwrap().content(),
        &DocumentString::from("policy B")
    );
    let (mut dom, env) = setup();
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            env,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let before = dom.snapshot().stamp();
    let cancel = AtomicBool::new(false);
    assert_eq!(
        dom.parsed_html_dispatch_cursor()
            .advance(&cancel)
            .unwrap()
            .unwrap()
            .state(),
        DomParsedHtmlExecutionState::Unstarted,
    );
    let processing =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap();
    let mut hold = dom
        .begin_parsed_html_style(id, processing, &cancel)
        .unwrap();
    assert_eq!(
        hold.advance(&cancel)
            .expect("qualified original policy must reach its actual inline request"),
        DomParsedHtmlStylePhase::InlinePolicyPending,
    );
    drop(hold);
    {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        assert!(
            view.initial_policy_request().is_none(),
            "known original policy supplies no offline binding request or slot"
        );
        let request = view.inline_policy_request().unwrap();
        assert_eq!(request.action(), HtmlStyleUpdateAction::CheckInlineCsp);
        assert!(
            request.text().is_empty(),
            "first insertion precedes parser text"
        );
        assert_eq!(
            request.nonce(),
            Some(&DocumentString::from("original-nonce"))
        );
        assert_eq!(request.policy().document(), policy_a.document());
        assert_eq!(request.policy().generation(), policy_a.generation());
        assert_eq!(request.policy().policy(), policy_a.policy());
        let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
        let policy = request.policy().policy();
        let HtmlCspNativePolicy::External(native) = policy
            .borrow_native_with::<RecordedPolicy, _>(&service.profile, &mut allow)
            .unwrap()
        else {
            panic!("original processor artifact A")
        };
        assert_eq!(native.text, DocumentString::from("policy A"));
        assert_eq!(native.padding, [37; 1024]);
        assert_eq!(request.qualification().contribution(), id);
        assert_eq!(request.qualification().document(), document);
    }
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.stamp(), before);
    assert_eq!(associated(&snapshot, document).policy(), policy_b.policy());
    assert_eq!(
        dom.parsed_html_dispatch_cursor()
            .advance(&cancel)
            .unwrap()
            .unwrap()
            .state(),
        DomParsedHtmlExecutionState::Unstarted,
        "request preparation does not execute the Style"
    );
    assert_eq!(service.calls, 2);
    assert!(dom.pending_parsed_html_associations().next().is_some());
}

#[test]
fn unchanged_original_policy_keeps_artifact_a_at_new_generation_for_style_before_final_b() {
    let mut processor = Processor::new();
    // The real processor fixture selects an authenticated unchanged result for
    // its second request; the other requests create their actual native artifacts.
    processor.unchanged_on_call = Some(1);
    let report = processed(
        "<!doctype html><head><meta http-equiv=content-security-policy content='policy A'><meta http-equiv=content-security-policy content='unchanged'><style nonce=n>a{color:red}</style><meta http-equiv=content-security-policy content='policy B'>",
        &mut processor,
    );
    assert_eq!(processor.calls, 3);
    let policy_b = original(&report).clone();
    let mut rows = report.pending_associations();
    let policy_a = loop {
        match rows.advance() {
            surgeist_html_parser::HtmlParsePendingAssociationStep::Occurrence(row)
                if row.requires_processing() && row.style().is_some() =>
            {
                break row.context().csp_association().unwrap().clone();
            }
            surgeist_html_parser::HtmlParsePendingAssociationStep::Complete => {
                panic!("actual original Style")
            }
            _ => (),
        }
    };
    let unchanged = policy_a.latest().unwrap();
    assert_eq!(unchanged.content(), &DocumentString::from("unchanged"));
    assert_eq!(policy_a.policy().id(), unchanged.prior_policy());
    assert_eq!(policy_a.policy().revision(), unchanged.prior_revision());
    assert_ne!(policy_a.generation(), unchanged.generation());
    assert_eq!(policy_a.document(), unchanged.document());
    assert_ne!(policy_a.policy().id(), policy_b.policy().id());
    assert_eq!(
        policy_b.latest().unwrap().content(),
        &DocumentString::from("policy B")
    );
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let HtmlCspNativePolicy::External(native) = policy_a
        .policy()
        .borrow_native_with::<RecordedPolicy, _>(&processor.profile, &mut allow)
        .unwrap()
    else {
        panic!("retained native artifact A")
    };
    assert_eq!(native.text, DocumentString::from("policy A"));
    assert_eq!(native.padding, [37; 1024]);
    let (mut dom, environment) = setup();
    let cancel = AtomicBool::new(false);
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            environment,
            limits(),
            &cancel,
        )
        .unwrap();
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let before = dom.snapshot().stamp();
    let processing =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap();
    assert_eq!(
        dom.begin_parsed_html_style(id, processing, &cancel)
            .unwrap()
            .advance(&cancel)
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let view = dom.parsed_html_style_prefix(id).unwrap();
    assert!(view.initial_policy_request().is_none());
    assert!(view.policy_ready());
    let request = view.inline_policy_request().unwrap();
    assert_eq!(request.policy().document(), policy_a.document());
    assert_eq!(request.policy().generation(), policy_a.generation());
    assert_eq!(request.policy().policy(), policy_a.policy());
    assert_eq!(
        request.policy().latest().unwrap().occurrence(),
        unchanged.occurrence()
    );
    assert!(request.text().is_empty());
    assert_eq!(request.nonce(), Some(&DocumentString::from("n")));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        associated(&dom.snapshot(), document).policy(),
        policy_b.policy()
    );
    assert_eq!(
        dom.parsed_html_dispatch_cursor()
            .advance(&cancel)
            .unwrap()
            .unwrap()
            .state(),
        DomParsedHtmlExecutionState::Unstarted
    );
    assert_eq!(
        processor.calls, 3,
        "no processing replay during original Style intake"
    );
}

#[test]
fn known_constructor_policy_has_no_binding_slot_and_rejects_foreign_binding_and_decision() {
    let cancel = AtomicBool::new(false);
    let mut processor = Processor::new();
    let report = processed("<!doctype html><style>body{}</style>", &mut processor);
    let original = original(&report).clone();
    assert!(original.policy().profile().is_none());
    assert!(original.latest().is_none());
    let (mut dom, environment) = setup();
    let mut ids = Vec::new();
    for _ in 0..2 {
        let installed = dom
            .install_parsed_html_document(
                dom.snapshot().stamp(),
                report.clone(),
                environment.clone(),
                limits(),
                &cancel,
            )
            .unwrap();
        ids.push(installed.pending_association().unwrap());
    }
    let processing =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap();
    for id in &ids {
        assert_eq!(
            dom.begin_parsed_html_style(*id, processing, &cancel)
                .unwrap()
                .advance(&cancel)
                .unwrap(),
            DomParsedHtmlStylePhase::InlinePolicyPending
        );
        let view = dom.parsed_html_style_prefix(*id).unwrap();
        assert!(view.initial_policy_request().is_none());
        assert!(view.policy_ready());
        assert_eq!(
            view.inline_policy_request().unwrap().policy().policy(),
            original.policy()
        );
    }
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            parse(HtmlSource::utf8("<!doctype html><style>offline</style>")),
            environment,
            limits(),
            &cancel,
        )
        .unwrap();
    let offline = installed.pending_association().unwrap();
    dom.begin_parsed_html_style(offline, processing, &cancel)
        .unwrap()
        .advance(&cancel)
        .unwrap();
    let request = dom
        .parsed_html_style_prefix(offline)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut input = Some(owner.initial_document_with(&mut allow).unwrap());
    let supplied = input.as_ref().unwrap().document().clone();
    let before = dom.snapshot().stamp();
    assert!(matches!(
        dom.begin_parsed_html_style(ids[0], processing, &cancel)
            .unwrap()
            .bind_initial_policy(&request, &mut input, &cancel),
        Err(DomParsedHtmlStyleError::WrongRequest)
    ));
    assert_eq!(input.as_ref().unwrap().document(), &supplied);
    let mut decision = {
        let view = dom.parsed_html_style_prefix(ids[1]).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    assert!(matches!(
        dom.begin_parsed_html_style(ids[0], processing, &cancel)
            .unwrap()
            .accept_inline_policy(&mut decision, &cancel),
        Err(DomParsedHtmlStyleError::WrongRequest)
    ));
    assert!(decision.is_some());
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.parsed_html_dispatch_cursor()
            .advance(&cancel)
            .unwrap()
            .unwrap()
            .state(),
        DomParsedHtmlExecutionState::Unstarted
    );
    assert_eq!(processor.calls, 0);
}

#[test]
fn base_facts_keep_their_original_policy_pair_and_native_rebinding_is_explicit() {
    use surgeist_html_semantics::*;
    for mode in 0..3 {
        let (mut dom, env) = setup();
        dom.set_limits(dom.limits().with_base_processing(256, 10_000_000).unwrap())
            .unwrap();
        let revision = env.url().revision().clone();
        let first = "https://example.test/first/";
        let second = "https://example.test/second/";
        let facts = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(env.url().clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(env.encoding().clone()),
            )
            .unwrap(),
            [first, second]
                .into_iter()
                .map(|raw| {
                    HtmlBaseParseObservation::new(
                        raw.into(),
                        env.url().clone(),
                        env.encoding().clone(),
                        revision.clone(),
                        HtmlBaseParseOutcome::Parsed {
                            record: HtmlBaseUrlRecord::observed(
                                revision.clone(),
                                raw.into(),
                                HtmlBaseUrlScheme::Other,
                                false,
                                None,
                            )
                            .unwrap(),
                            base_allowed: HtmlBaseAllowed::Allowed,
                        },
                    )
                    .unwrap()
                })
                .collect(),
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        let mut processor = Processor::new();
        processor.unchanged = mode == 1;
        let text = if mode == 0 {
            "<!doctype html><head><base href='https://example.test/first/'>"
        } else {
            "<!doctype html><head><base href='https://example.test/first/'><meta http-equiv=content-security-policy content='fixture'>"
        };
        let report = Arc::new(
            HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), parse_limits())
                .with_initial_policy_base_facts(facts.clone())
                .unwrap()
                .parse_with_meta_csp(&mut processor)
                .unwrap(),
        );
        if mode == 1 {
            assert_eq!(
                original(&report).latest().unwrap().content(),
                &DocumentString::from("fixture")
            );
        }
        let installed = dom
            .install_parsed_html_document(
                dom.snapshot().stamp(),
                report,
                env,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let document = installed.document();
        let base = element(&installed, "base");
        let before = dom.snapshot();
        assert_eq!(
            before
                .node(base)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .unwrap()
                .value()
                .serialization(),
            &DocumentString::from(first)
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(base, "href", second);
        if mode == 2 {
            assert_eq!(
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::BaseFacts(DomBaseFactsError::PolicyMismatch)
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            let mut current = DomBatch::new();
            current.bind_base_facts(dom.snapshot().document(document).unwrap().stamp(), &facts);
            current.set_attribute(base, "href", second);
            dom.apply(current, &AtomicBool::new(false)).unwrap();
        } else {
            // Both an initial policy and an authenticated unchanged result keep
            // the policy identity; association generation alone is irrelevant.
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
        }
        assert_eq!(
            dom.snapshot()
                .node(base)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .unwrap()
                .value()
                .serialization(),
            &DocumentString::from(second)
        );
        assert_eq!(
            before
                .node(base)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .unwrap()
                .value()
                .serialization(),
            &DocumentString::from(first)
        );
        assert_eq!(processor.calls, usize::from(mode != 0));
    }
}

#[test]
fn transferred_policy_does_not_enable_native_csp_processing_or_lose_snapshot_history() {
    let mut service = Processor::new();
    let report = processed(
        "<!doctype html><head><meta http-equiv=content-security-policy content='original'>",
        &mut service,
    );
    let (mut dom, env) = setup();
    dom.set_limits(
        dom.limits()
            .with_meta_processing_work(1_000_000)
            .with_meta_theme_color_scratch_bytes(262144)
            .unwrap(),
    )
    .unwrap();
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            env,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let document = installed.document();
    let meta = element(&installed, "meta");
    let head = element(&installed, "head");
    let old = dom.snapshot();
    let identity = associated(&old, document).document().clone();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(meta, head, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::Meta(DomMetaError::ContentSecurityPolicyUnsupported)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(associated(&dom.snapshot(), document).document(), &identity);
    assert_eq!(
        associated(&old, document).latest().unwrap().content(),
        &DocumentString::from("original")
    );
    assert_eq!(service.calls, 1);
}

#[test]
fn occurrence_policy_history_is_not_replaced_by_final_document_policy() {
    use surgeist_html_parser::{
        HtmlParseAssociationOccurrenceKind, HtmlParsePendingAssociationStep,
    };
    let mut processor = Processor::new();
    let report = processed(
        "<!doctype html><head><style>first</style><meta http-equiv=content-security-policy content='A'><style>second</style><meta http-equiv=content-security-policy content='B'><style>third</style>",
        &mut processor,
    );
    let mut policies = Vec::new();
    let mut reader = report.pending_associations();
    loop {
        match reader.advance() {
            HtmlParsePendingAssociationStep::Occurrence(row) => {
                if matches!(
                    row.kind(),
                    HtmlParseAssociationOccurrenceKind::StyleUpdate(_)
                ) {
                    let policy = row
                        .context()
                        .csp_association()
                        .expect("actual occurrence-time policy");
                    assert_eq!(
                        row.context().requirement(
                            surgeist_html_parser::HtmlParseAssociationContextRole::InitialPolicy
                        ),
                        surgeist_html_parser::HtmlParseAssociationContextRequirement::Known
                    );
                    assert!(row.context().slot(surgeist_html_parser::HtmlParseAssociationContextRole::InitialPolicy).is_none());
                    if policies.last() != Some(policy.policy().id()) {
                        policies.push(policy.policy().id().clone());
                    }
                }
            }
            HtmlParsePendingAssociationStep::Complete => break,
            _ => (),
        }
    }
    assert_eq!(
        policies.len(),
        3,
        "P0, PA and PB remain distinct original inputs"
    );
    assert_eq!(policies.last(), Some(original(&report).policy().id()));
    assert_eq!(processor.calls, 2);
    let (mut dom, env) = setup();
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            env,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(
        associated(installed.snapshot(), installed.document())
            .policy()
            .id(),
        &policies[2]
    );
    let id = installed.pending_association().unwrap();
    let cancel = AtomicBool::new(false);
    let mut hold = dom
        .begin_parsed_html_style(
            id,
            DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap(),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        hold.advance(&cancel).unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    drop(hold);
    assert_eq!(
        dom.parsed_html_dispatch_cursor()
            .advance(&cancel)
            .unwrap()
            .unwrap()
            .state(),
        DomParsedHtmlExecutionState::Unstarted
    );
    let view = dom.parsed_html_style_prefix(id).unwrap();
    assert!(view.initial_policy_request().is_none());
    let request = view.inline_policy_request().unwrap();
    assert_eq!(request.policy().policy().id(), &policies[0]);
    assert!(request.policy().latest().is_none());
    assert!(request.text().is_empty());
    assert_eq!(processor.calls, 2);
}

#[test]
fn untouched_inert_document_keeps_its_own_explicit_constructor_association() {
    let mut service = Processor::new();
    let report = processed(
        "<!doctype html><template><div>inert</div><template><p>nested</template></template>",
        &mut service,
    );
    let (mut dom, env) = setup();
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            env,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let mut identities = Vec::new();
    for binding in installed.bindings() {
        let node = installed.snapshot().node(binding.node()).unwrap();
        if let Some(document) = node.as_document() {
            let DomDocumentCspState::Associated(policy) =
                document.metadata().policy_container().csp()
            else {
                panic!("every actual fresh Document is qualified")
            };
            assert!(policy.policy().profile().is_none());
            assert!(policy.latest().is_none());
            assert!(!identities.contains(policy.document()));
            identities.push(policy.document().clone());
        }
    }
    assert_eq!(
        identities.len(),
        2,
        "ordinary and shared inert template owner"
    );
    assert_eq!(service.calls, 0);
}
