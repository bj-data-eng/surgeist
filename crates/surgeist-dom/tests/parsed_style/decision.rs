//! One actual external observation, never policy evaluation by DOM.
use super::*;
use surgeist_html::*;

// A bounded external backend fixture, not a CSP parser. Its native immutable
// artifact owns the fixture directive; Meta attestation only constructs policy
// custody and is never used as an inline decision receipt.
#[derive(Debug)]
struct DenyInlineStyle;
impl HtmlCspPolicyArtifact for DenyInlineStyle {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>(), 0, 1).unwrap(),
            HtmlCspDisposalWork::try_new(1, 0).unwrap(),
        )
        .unwrap()
    }
}
fn blocking_policy() -> HtmlCspAssociation {
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut current = owner.initial_document_with(&mut allow).unwrap();
    let mut backend = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    let profile = backend.profile_with(1, &mut allow).unwrap();
    let request = owner
        .request_meta_insertion_with(
            &current,
            &profile,
            "fixture deny inline style".into(),
            &mut allow,
        )
        .unwrap();
    let prepared = backend
        .attest_changed_with(&request, Arc::new(DenyInlineStyle), &mut allow)
        .unwrap();
    drop(
        owner
            .prepare_publication_with(&mut current, request, prepared, &mut allow)
            .unwrap()
            .publish(),
    );
    current
}

fn ready(dom: &mut Dom, id: DomParsedHtmlAssociationId, limits: DomParsedHtmlAssociationLimits) {
    ready_with_policy(dom, id, limits, empty_policy());
}
fn ready_with_policy(
    dom: &mut Dom,
    id: DomParsedHtmlAssociationId,
    limits: DomParsedHtmlAssociationLimits,
    policy: HtmlCspAssociation,
) {
    let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    hold.advance(&flag()).unwrap();
    drop(hold);
    let q = qualifier(dom, id);
    let mut policy = Some(policy);
    let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    hold.bind_initial_policy(&q, &mut policy, &flag()).unwrap();
    assert_eq!(
        hold.advance(&flag()).unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
}
fn observed(
    dom: &Dom,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    let view = dom.parsed_html_style_prefix(id).unwrap();
    let request = view.inline_policy_request().unwrap();
    assert!(request.text().is_empty());
    assert_eq!(
        request.nonce().unwrap().code_units(),
        &"exact nonce".encode_utf16().collect::<Vec<_>>()
    );
    let allowed = match request.policy().policy().profile() {
        None => true, // Genuine constructor-empty state.
        Some(profile) => {
            let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
            match request
                .policy()
                .policy()
                .borrow_native_with::<DenyInlineStyle, _>(profile, &mut allow)
                .unwrap()
            {
                HtmlCspNativePolicy::InitialEmpty => panic!("external fixture profile"),
                HtmlCspNativePolicy::External(_) => false,
            }
        }
    };
    DomParsedHtmlStyleInlinePolicyObservation::evaluated(request, allowed)
}
fn fixture() -> (Dom, DocumentId, DomParsedHtmlAssociationId) {
    let (mut dom, env) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><style nonce='exact nonce'>later authored text</style>"),
        env,
    );
    (dom, document, id)
}

#[test]
fn allow_and_block_advance_the_selected_cursor_once_without_consuming_the_contribution() {
    for (allowed, expected) in [
        (true, DomParsedHtmlStylePhase::CreateSheetPending),
        (false, DomParsedHtmlStylePhase::InlinePolicyBlocked),
    ] {
        let (mut dom, document, id) = fixture();
        ready_with_policy(
            &mut dom,
            id,
            envelope(),
            if allowed {
                empty_policy()
            } else {
                blocking_policy()
            },
        );
        let stamp = dom.snapshot().stamp();
        let mut input = Some(observed(&dom, id));
        let mut repeated = Some(observed(&dom, id));
        assert_eq!(input.as_ref().unwrap().qualification().contribution(), id);
        assert_eq!(input.as_ref().unwrap().allowed(), allowed);
        {
            let mut hold = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            assert_eq!(
                hold.accept_inline_policy(&mut input, &flag()).unwrap(),
                expected
            );
            assert!(input.is_none());
            assert_eq!(hold.advance(&flag()).unwrap(), expected);
            assert!(
                matches!(hold.accept_inline_policy(&mut repeated, &flag()), Err(DomParsedHtmlStyleError::DecisionAlreadyAccepted { contribution }) if contribution == id)
            );
        }
        assert!(repeated.is_some());
        let view = dom.parsed_html_style_prefix(id).unwrap();
        assert_eq!(view.phase(), expected);
        assert!(view.inline_policy_request().is_none());
        drop(view);
        assert_eq!(
            dom.pending_parsed_html_association(document).unwrap(),
            Some(id)
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
    }
}

#[test]
fn independent_destination_and_missing_observation_do_not_consume_the_actual_envelope() {
    let (mut dom, env) = setup();
    let report = parse("<!doctype html><style nonce='exact nonce'>later</style>");
    let (_, a) = install(&mut dom, report.clone(), env.clone());
    let (_, b) = install(&mut dom, report, env);
    ready(&mut dom, a, envelope());
    ready(&mut dom, b, envelope());
    let mut input = Some(observed(&dom, a));
    {
        let mut hold = dom.begin_parsed_html_style(b, envelope(), &flag()).unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut input, &flag()),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        assert!(matches!(
            hold.accept_inline_policy(&mut None, &flag()),
            Err(DomParsedHtmlStyleError::MissingObservation)
        ));
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::InlinePolicyPending
        );
    }
    assert_eq!(input.as_ref().unwrap().qualification().contribution(), a);
    dom.begin_parsed_html_style(a, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut input, &flag())
        .unwrap();
    assert!(input.is_none());
}

#[test]
fn cancellation_preserves_the_same_observation_and_request_for_retry() {
    let (mut dom, _, id) = fixture();
    ready_with_policy(&mut dom, id, envelope(), blocking_policy());
    let mut input = Some(observed(&dom, id));
    let cancel = AtomicBool::new(true);
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut input, &cancel),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ));
    }
    assert!(input.is_some());
    assert!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .is_some()
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert_eq!(
        hold.accept_inline_policy(&mut input, &flag()).unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
}

#[test]
fn output_refusal_preserves_decision_and_binding_until_raised_retry() {
    let (mut dom, _, id) = fixture();
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1).unwrap();
    ready(&mut dom, id, limits);
    let mut evaluations = 0;
    let mut input = Some({
        evaluations += 1;
        observed(&dom, id)
    });
    let spent;
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut input, &flag()),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 1
                })
            ))
        ));
        spent = hold.spent_work();
    }
    assert!(input.is_some());
    assert!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .initial_policy_bound()
    );
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let raised = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 2).unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() > spent);
    assert_eq!(
        hold.accept_inline_policy(&mut input, &flag()).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert_eq!(evaluations, 1);
}

#[test]
fn forgotten_read_guard_keeps_busy_custody_and_the_unconsumed_decision() {
    let (mut dom, _, id) = fixture();
    ready(&mut dom, id, envelope());
    let input = Some(observed(&dom, id));
    let view = dom.parsed_html_style_prefix(id).unwrap();
    std::mem::forget(view);
    assert!(
        matches!(dom.begin_parsed_html_style(id, envelope(), &flag()), Err(DomParsedHtmlStyleError::Busy { contribution }) if contribution == id)
    );
    assert!(input.is_some());
}
