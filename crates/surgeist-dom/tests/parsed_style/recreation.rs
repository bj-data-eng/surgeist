//! Two genuine original invocations; known-null removal is not publication.
use super::*;
use surgeist_html::*;

// An external native backend fixture allows precisely this known text. It is
// not a CSP parser, and decisions are obtained from the actual request inputs.
#[derive(Debug)]
struct ExactText;
impl HtmlCspPolicyArtifact for ExactText {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>(), 0, 1).unwrap(),
            HtmlCspDisposalWork::try_new(1, 0).unwrap(),
        )
        .unwrap()
    }
}
fn policy() -> HtmlCspAssociation {
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut current = owner.initial_document_with(&mut allow).unwrap();
    let mut backend = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    let profile = backend.profile_with(1, &mut allow).unwrap();
    let request = owner
        .request_meta_insertion_with(
            &current,
            &profile,
            "fixture exact inline text".into(),
            &mut allow,
        )
        .unwrap();
    let attested = backend
        .attest_changed_with(&request, Arc::new(ExactText), &mut allow)
        .unwrap();
    drop(
        owner
            .prepare_publication_with(&mut current, request, attested, &mut allow)
            .unwrap()
            .publish(),
    );
    current
}
fn observed(
    dom: &Dom,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    let view = dom.parsed_html_style_prefix(id).unwrap();
    let request = view.inline_policy_request().unwrap();
    let profile = request.policy().policy().profile().unwrap();
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let native = request
        .policy()
        .policy()
        .borrow_native_with::<ExactText, _>(profile, &mut allow)
        .unwrap();
    assert!(matches!(native, HtmlCspNativePolicy::External(_)));
    let allowed = request.text().code_units() == "a{color:red}".encode_utf16().collect::<Vec<_>>();
    DomParsedHtmlStyleInlinePolicyObservation::evaluated(request, allowed)
}
fn blocked(
    source: &str,
    limits: DomParsedHtmlAssociationLimits,
) -> (
    Dom,
    DocumentId,
    DomParsedHtmlAssociationId,
    DomParsedHtmlStyleInitialPolicyRequest,
    DomParsedHtmlStyleInlinePolicyObservation,
) {
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, parse(source), env);
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        hold.advance(&flag()).unwrap();
    }
    let q = qualifier(&dom, id);
    {
        let mut supplied = Some(policy());
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        hold.bind_initial_policy(&q, &mut supplied, &flag())
            .unwrap();
        hold.advance(&flag()).unwrap();
    }
    let old = observed(&dom, id);
    let mut supplied = Some(observed(&dom, id));
    assert!(!supplied.as_ref().unwrap().allowed());
    assert_eq!(
        dom.begin_parsed_html_style(id, limits, &flag())
            .unwrap()
            .accept_inline_policy(&mut supplied, &flag())
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    (dom, document, id, q, old)
}
fn no_sheet(
    dom: &mut Dom,
    id: DomParsedHtmlAssociationId,
    limits: DomParsedHtmlAssociationLimits,
) -> DomParsedHtmlStyleSuccessorCompletion {
    let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    assert_eq!(hold.successor_owner_document(&flag()).unwrap(), None);
    let published = hold
        .prepare_successor(None, &flag())
        .unwrap()
        .commit(&flag())
        .unwrap();
    let (completion, retirement) = published.into_parts();
    assert!(matches!(
        completion.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    drop(retirement);
    completion
}
#[test]
fn blocked_first_needs_no_creation_owner_and_later_exact_text_can_create_and_render() {
    let (mut dom, document, id, old_q, old) = blocked(
        "<!doctype html><head><style title=A media='' nonce=n>a{color:red}</style></head><body>final",
        envelope(),
    );
    let stamp = dom.snapshot().stamp();
    let completion = no_sheet(&mut dom, id, envelope());
    assert_eq!(
        dom.snapshot().stamp(),
        stamp,
        "known-null progress fabricated no commit"
    );
    assert_ne!(completion.qualification(), &old_q);
    let mut old_input = Some(old);
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut old_input, &flag()),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        assert!(old_input.is_some());
    }
    let mut input = Some(observed(&dom, id));
    assert!(input.as_ref().unwrap().allowed());
    let cap = create::root(document);
    let sheet = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.accept_inline_policy(&mut input, &flag()).unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        assert_eq!(
            prep.request().text().code_units(),
            "a{color:red}".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(prep.request().title().unwrap().code_units(), [65]);
        create::stage(&mut prep);
        let (created, retired) = prep.commit(&flag()).unwrap().into_parts();
        drop(retired);
        created.sheet().clone()
    };
    assert!(
        !cap.snapshot()
            .unwrap()
            .sheet_rules(sheet.id())
            .unwrap()
            .is_empty()
    );
    let q = completion.qualification();
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.exclude_script_bookkeeping(
            q,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag(),
        )
        .unwrap();
        assert!(matches!(
            hold.process_rendering(q, &flag()).unwrap(),
            DomParsedHtmlStyleRenderingProgress::Complete(_)
        ));
        assert!(matches!(
            hold.prepare_notification(&old_q, &sheet, "owner", &cap, &flag()),
            Err(DomParsedHtmlStyleNotificationError::Style(
                DomParsedHtmlStyleError::WrongPhase {
                    phase: DomParsedHtmlStylePhase::InlinePolicyBlocked
                }
            ))
        ));
    }
    assert_eq!(
        dom.pending_parsed_html_association(document).unwrap(),
        Some(id)
    );
}
#[test]
fn blocked_successor_remains_a_real_early_return_without_a_creation_owner() {
    let (mut dom, document, id, _q, _old) =
        blocked("<!doctype html><style>b{color:blue}</style>", envelope());
    let before = dom.snapshot().stamp();
    no_sheet(&mut dom, id, envelope());
    let mut input = Some(observed(&dom, id));
    assert!(!input.as_ref().unwrap().allowed());
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert_eq!(
        hold.accept_inline_policy(&mut input, &flag()).unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert!(matches!(
        hold.create_sheet_document(&flag()),
        Err(DomParsedHtmlStyleError::WrongPhase {
            phase: DomParsedHtmlStylePhase::InlinePolicyBlocked
        })
    ));
    drop(hold);
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.pending_parsed_html_association(document).unwrap(),
        Some(id)
    );
}
#[test]
fn known_null_refuses_unneeded_owner_and_cancellation_without_changing_its_phase() {
    let (mut dom, document, id, q, _) =
        blocked("<!doctype html><style>a{color:red}</style>", envelope());
    let cap = create::root(document);
    let before = dom.snapshot().stamp();
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.prepare_successor(Some(&cap), &flag()),
            Err(DomParsedHtmlStyleSuccessorError::Style(
                DomParsedHtmlStyleError::WrongRequest
            ))
        ));
        assert!(
            hold.prepare_successor(None, &AtomicBool::new(true))
                .is_err()
        );
    }
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .initial_policy_request(),
        Some(&q)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    no_sheet(&mut dom, id, envelope());
    assert!(cap.snapshot().unwrap().sheets().is_empty());
}
#[test]
fn known_null_output_refusal_retains_original_controller_and_raise_retries_without_publication() {
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 2).unwrap();
    let (mut dom, _document, id, q, _) =
        blocked("<!doctype html><style>a{color:red}</style>", limits);
    let before = dom.snapshot().stamp();
    let spent = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        let initial = hold.spent_work();
        assert!(matches!(
            hold.prepare_successor(None, &flag())
                .unwrap()
                .commit(&flag()),
            Err(DomParsedHtmlStyleSuccessorError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::ParsedAssociationProcessingOutputs,
                        limit: 2
                    }
                ))
            ))
        ));
        assert!(hold.spent_work() > initial);
        hold.spent_work()
    };
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .initial_policy_request(),
        Some(&q)
    );
    {
        let hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(hold.spent_work() >= spent);
    }
    no_sheet(&mut dom, id, envelope());
    assert_eq!(dom.snapshot().stamp(), before);
}
#[test]
fn created_predecessor_requires_its_real_owner_instead_of_using_missing_as_null() {
    let mut value = super::successor::rendered();
    let foreign = create::root(value.document);
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert_eq!(
        hold.successor_owner_document(&flag()).unwrap(),
        Some(value.document)
    );
    assert!(
        matches!(hold.prepare_successor(None,&flag()),Err(DomParsedHtmlStyleSuccessorError::MissingOwner { root }) if root==value.document.node())
    );
    assert!(
        matches!(hold.prepare_successor(Some(&foreign),&flag()),Err(DomParsedHtmlStyleSuccessorError::MismatchedOwner { root }) if root==value.document.node())
    );
}
#[test]
fn created_first_recreates_distinct_association_and_old_notification_uses_old_sheet_proof() {
    let mut value = super::successor::rendered();
    let old = value.association.clone();
    let completion = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        let (completed, retired) = hold
            .prepare_successor(Some(&value.root), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retired);
        completed
    };
    assert!(matches!(
        completion.removal(),
        DomParsedHtmlStyleSuccessorRemoval::Removed { .. }
    ));
    let mut decision = {
        let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let created = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        hold.accept_inline_policy(&mut decision, &flag()).unwrap();
        let mut prep = hold.prepare_create_sheet(&value.root, &flag()).unwrap();
        create::stage(&mut prep);
        let (created, retired) = prep.commit(&flag()).unwrap().into_parts();
        drop(retired);
        created
    };
    assert_ne!(created.sheet().id(), old.id());
    assert_ne!(created.sheet().association(), old.association());
    let snapshot = value.root.snapshot().unwrap();
    assert_eq!(
        snapshot.sheets(),
        std::slice::from_ref(created.sheet().id())
    );
    assert!(snapshot.sheet(old.id()).unwrap().inputs().owner.is_none());
    assert!(
        !snapshot
            .sheet_rules(created.sheet().id())
            .unwrap()
            .is_empty()
    );
    drop(snapshot);
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    hold.exclude_script_bookkeeping(
        completion.qualification(),
        HtmlStyleExecutionProfile::NoExecutableJavaScript,
        &flag(),
    )
    .unwrap();
    assert!(matches!(
        hold.process_rendering(completion.qualification(), &flag())
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    assert!(matches!(
        hold.prepare_notification(&value.q, created.sheet(), "owner", &value.root, &flag()),
        Err(DomParsedHtmlStyleNotificationError::MismatchedAssociation { .. })
    ));
    assert!(
        hold.prepare_notification(&value.q, &old, "owner", &value.root, &flag())
            .unwrap()
            .owner_association_removed()
    );
}
#[test]
fn blocked_pending_adoption_refusal_preserves_real_destination_for_null_successor() {
    let (mut dom, document, id, q, _) =
        blocked("<!doctype html><style>a{color:red}</style>", envelope());
    let element = dom
        .pending_parsed_html_associations()
        .find(|row| row.id() == id)
        .unwrap()
        .node_for(q.subject())
        .unwrap();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let output = dom.apply(batch, &flag()).unwrap();
    let other = output.document(&other).unwrap();
    let before = dom.snapshot().stamp();
    let mut adopt = DomBatch::new();
    adopt.adopt(element, other.node());
    assert!(
        matches!(dom.apply(adopt,&flag()),Err(DomError::ParsedAssociationPending { document:actual,pending }) if actual==document && pending==id)
    );
    assert_eq!(dom.snapshot().stamp(), before);
    let completion = no_sheet(&mut dom, id, envelope());
    assert_eq!(completion.qualification().document(), document);
    assert_eq!(dom.snapshot().stamp(), before);
}
