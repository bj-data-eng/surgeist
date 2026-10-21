//! Actual original native graph custody, not a policy-byte estimate from source.
use super::*;

#[derive(Debug)]
struct OriginalPolicy {
    content: DocumentString,
    padding: [u8; 8192],
    minimum_text_units: usize,
}
impl HtmlCspPolicyArtifact for OriginalPolicy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        let extent = self.content.shared_code_units().capacity();
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(
                size_of::<Self>() + size_of::<Vec<u16>>() + extent * 2,
                extent,
                2,
            )
            .unwrap(),
            HtmlCspDisposalWork::try_new(16, 2048).unwrap(),
        )
        .unwrap()
    }
}
struct Processor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
    calls: usize,
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
        let value = OriginalPolicy {
            content: request.qualification().content().clone(),
            padding: [53; 8192],
            minimum_text_units: 1,
        };
        let layout = value.retained_layout();
        admit(HtmlCspLayout::processing(
            8,
            layout.storage(),
            layout.disposal(),
            size_of::<Self>(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        self.calls += 1;
        match self
            .authority
            .attest_changed_with(request, Arc::new(value), admit)
        {
            Ok(value) => Ok(HtmlMetaCspOutcome::Completed(value)),
            Err(rejected) => {
                drop(rejected.retained);
                release(layout.storage());
                Err(rejected.error)
            }
        }
    }
}
fn report() -> (Arc<HtmlParseReport>, Processor, HtmlCspAssociation) {
    let mut processor = Processor::new();
    let source = format!(
        "<!doctype html><head><meta http-equiv=content-security-policy content='{}'><style nonce=n>a{{color:red}}</style>",
        "x".repeat(4096)
    );
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), parser_limits())
            .parse_with_meta_csp(&mut processor)
            .unwrap(),
    );
    let mut rows = report.pending_associations();
    let original = loop {
        match rows.advance() {
            Step::Occurrence(row) if row.requires_processing() && row.style().is_some() => {
                break row.context().csp_association().unwrap().clone();
            }
            Step::Complete => panic!("actual known Style"),
            _ => (),
        }
    };
    (report, processor, original)
}
#[test]
fn known_graph_actual_padding_and_utf16_admission_refuse_then_raise_same_controller() {
    let (report, processor, original) = report();
    let layout = original.retained_layout().unwrap();
    let policy_bytes = layout.storage().bytes() - size_of::<HtmlCspAssociation>();
    assert!(policy_bytes >= 8192 + 4096 * 2);
    assert!(layout.storage().utf16_units() >= 4096);
    for by_units in [false, true] {
        let (mut dom, env) = setup();
        let (_, id) = install(&mut dom, report.clone(), env);
        let before = dom.pending_associations.usage;
        let ledger = dom.custody_registry.ledger().clone();
        let baseline = ledger.total();
        let bytes = if by_units {
            envelope().retained_bytes()
        } else {
            size_of::<DomParsedHtmlStyleRuntimeHold<'_>>()
                + size_of::<Graph>()
                + policy_bytes
                + layout.disposal().atomic_bytes()
                - 1
        };
        let units = if by_units {
            layout.storage().utf16_units() - 1
        } else {
            envelope().retained_units()
        };
        let limits =
            DomParsedHtmlAssociationLimits::new(bytes, units, envelope().work(), 1024).unwrap();
        let spent = {
            let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
            let error = hold.advance(&flag()).unwrap_err();
            assert!(matches!(error, DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit { resource, .. })
            ) if resource == if by_units { DomResource::ParsedAssociationProcessingRetainedUnits } else { DomResource::ParsedAssociationProcessingRetainedBytes }));
            hold.spent_work()
        };
        assert_eq!(ledger.total(), baseline);
        assert_eq!(dom.pending_associations.usage, before);
        assert_eq!(
            dom.parsed_html_style_prefix(id).unwrap().phase(),
            DomParsedHtmlStylePhase::Unstarted
        );
        let record = dom.pending_associations.controlled_entry(id).unwrap();
        assert_eq!(
            record.value().control.persistent_usage(),
            custody::Usage::ZERO
        );
        {
            let mut hold = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            assert!(hold.spent_work() > spent);
            assert_eq!(
                hold.advance(&flag()).unwrap(),
                DomParsedHtmlStylePhase::InlinePolicyPending
            );
        }
        // Exact actual first text/nonce extents plus full original graph, not
        // a source-only receipt or size_of<Arc> for the native policy.
        let expected = custody::Usage::new(
            size_of::<Graph>() + policy_bytes + 2 * size_of::<Vec<u16>>() + 2,
            layout.storage().utf16_units() + 1,
        );
        assert_eq!(record.value().control.persistent_usage(), expected);
        assert_eq!(
            dom.pending_associations.usage,
            before.checked_add(expected).unwrap()
        );
        let view = dom.parsed_html_style_prefix(id).unwrap();
        assert!(view.initial_policy_request().is_none());
        let request = view.inline_policy_request().unwrap();
        assert_eq!(request.policy().generation(), original.generation());
        assert_eq!(request.policy().policy(), original.policy());
        let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
        let HtmlCspNativePolicy::External(native) = request
            .policy()
            .policy()
            .borrow_native_with::<OriginalPolicy, _>(&processor.profile, &mut allow)
            .unwrap()
        else {
            panic!("actual native policy")
        };
        assert_eq!(native.padding, [53; 8192]);
        assert_eq!(native.content.len(), 4096);
    }
    assert_eq!(processor.calls, 1);
}
#[test]
fn known_successor_overlap_and_cancel_retire_old_policy_only_after_real_transition() {
    let (report, processor, original) = report();
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, report, env);
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .advance(&flag())
        .unwrap();
    let old_q = qualifier(&dom, id);
    let mut decision = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        let request = view.inline_policy_request().unwrap();
        // This fixture's supplied policy permits only nonempty exact text;
        // authentic first insertion is therefore blocked, with no sheet owner.
        let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
        let HtmlCspNativePolicy::External(native) = request
            .policy()
            .policy()
            .borrow_native_with::<OriginalPolicy, _>(&processor.profile, &mut allow)
            .unwrap()
        else {
            panic!("actual blocking fixture policy")
        };
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request,
            request.text().len() >= native.minimum_text_units,
        ))
    };
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut decision, &flag())
        .unwrap();
    let record = dom.pending_associations.controlled_entry(id).unwrap();
    let persistent = record.value().control.persistent_usage();
    let ledger = dom.custody_registry.ledger().clone();
    let baseline = ledger.total();
    let policy_bytes =
        original.retained_layout().unwrap().storage().bytes() - size_of::<HtmlCspAssociation>();
    let cancel = flag();
    let spent = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &cancel)
            .unwrap();
        let prep = hold.prepare_successor(None, &cancel).unwrap();
        assert!(ledger.total().bytes() >= baseline.bytes() + policy_bytes);
        assert_ne!(prep.qualification().occurrence(), old_q.occurrence());
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            prep.commit(&cancel),
            Err(DomParsedHtmlStyleSuccessorError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::Cancelled
                ))
            ))
        ));
        hold.spent_work()
    };
    assert_eq!(ledger.total(), baseline);
    assert_eq!(record.value().control.persistent_usage(), persistent);
    assert_eq!(qualifier(&dom, id), old_q);
    cancel.store(false, Ordering::Release);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &cancel)
        .unwrap();
    assert!(hold.spent_work() > spent);
    let (completion, retired) = hold
        .prepare_successor(None, &cancel)
        .unwrap()
        .commit(&cancel)
        .unwrap()
        .into_parts();
    assert!(matches!(
        completion.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    drop(hold);
    let with_retirement = ledger.total();
    drop(retired);
    assert!(with_retirement.bytes() >= ledger.total().bytes() + policy_bytes);
    let view = dom.parsed_html_style_prefix(id).unwrap();
    let request = view.inline_policy_request().unwrap();
    assert_eq!(request.text(), &DocumentString::from("a{color:red}"));
    assert_eq!(request.policy().document(), original.document());
    assert_eq!(request.policy().generation(), original.generation());
    assert_eq!(request.policy().policy(), original.policy());
    assert_eq!(request.qualification(), completion.qualification());
    assert_eq!(processor.calls, 1);
}
