//! External historical decisions use the actual stored request, never native Style captures.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, environment, installation, parsing, report};
use std::{convert::Infallible, mem::size_of, sync::Arc};
use surgeist_dom::*;
use surgeist_html::*;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseReport, HtmlSource};

fn install_into(
    session: &mut DomSession,
    value: Arc<HtmlParseReport>,
) -> DomParsedHtmlInstallation {
    let environment = environment(session.dom().raw());
    session
        .install_parsed_html_document(
            session.snapshot().stamp(),
            value,
            environment,
            installation(),
            &cancel(),
        )
        .unwrap()
}
fn installed(value: Arc<HtmlParseReport>) -> (DomSession, DomParsedHtmlInstallation) {
    let (mut session, _) = bound_dispatch_session();
    let installation = install_into(&mut session, value);
    (session, installation)
}
fn allow(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}
fn empty_policy() -> HtmlCspAssociation {
    HtmlCspOwner::new_with(&mut allow)
        .unwrap()
        .initial_document_with(&mut allow)
        .unwrap()
}

/// A fixed external backend fixture, not a CSP parser. Its actual immutable
/// artifact supplies the decision; Meta receipts are used only to construct a
/// genuine artifact association, never as an inline decision receipt.
#[derive(Debug)]
struct NativePolicy {
    allowed: bool,
}
impl HtmlCspPolicyArtifact for NativePolicy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>(), 0, 1).unwrap(),
            HtmlCspDisposalWork::try_new(1, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
}
fn native_policy(allowed: bool) -> (HtmlCspAssociation, HtmlCspProcessorProfile) {
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut policy = owner.initial_document_with(&mut allow).unwrap();
    let mut backend = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    let profile = backend.profile_with(1, &mut allow).unwrap();
    let request = owner
        .request_meta_insertion_with(&policy, &profile, "fixture policy".into(), &mut allow)
        .unwrap();
    let native = Arc::new(NativePolicy { allowed });
    let prepared = backend
        .attest_changed_with(&request, native, &mut allow)
        .unwrap();
    let completion = owner
        .prepare_publication_with(&mut policy, request, prepared, &mut allow)
        .unwrap()
        .publish();
    drop(completion);
    (policy, profile)
}
fn request_ready(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    policy: HtmlCspAssociation,
    limits: DomParsedHtmlAssociationLimits,
) {
    let runtime = session.style_execution();
    assert_eq!(
        runtime
            .prepare_parsed_style(&mut session.dom(), id, limits, &cancel())
            .unwrap(),
        DomParsedHtmlStylePhase::AwaitingInitialPolicy
    );
    let qualification = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let mut input = Some(policy);
    runtime
        .bind_parsed_style_initial_policy(
            &mut session.dom(),
            &qualification,
            &mut input,
            limits,
            &cancel(),
        )
        .unwrap();
    assert!(input.is_none());
    assert_eq!(
        runtime
            .prepare_parsed_style(&mut session.dom(), id, limits, &cancel())
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
}
fn observe(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
    supplier: &mut impl FnMut(
        &DomParsedHtmlStyleInlinePolicyRequest,
    ) -> Option<DomParsedHtmlStyleInlinePolicyObservation>,
) -> Option<DomParsedHtmlStyleInlinePolicyObservation> {
    let view = session.parsed_html_style_prefix(id).unwrap();
    supplier(view.inline_policy_request().unwrap())
}
fn accept(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    input: &mut Option<DomParsedHtmlStyleInlinePolicyObservation>,
    limits: DomParsedHtmlAssociationLimits,
) -> Result<DomParsedHtmlStylePhase, DomStyleParsedPrefixError> {
    session.style_execution().accept_parsed_style_inline_policy(
        &mut session.dom(),
        id,
        input,
        limits,
        &cancel(),
    )
}
fn still_pending(session: &DomSession, id: DomParsedHtmlAssociationId, stamp: DomStamp) {
    assert_eq!(session.snapshot().stamp(), stamp);
    assert!(
        session
            .pending_parsed_html_associations()
            .any(|entry| entry.id() == id)
    );
    let view = session.parsed_html_style_prefix(id).unwrap();
    let document = view.initial_policy_request().unwrap().document();
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .is_none()
    );
}

#[test]
fn explicit_empty_policy_allows_the_exact_original_request_once_without_creating_a_sheet() {
    let mut units = "<!doctype html><style nonce='N&amp;"
        .encode_utf16()
        .collect::<Vec<_>>();
    units.push(0xd800);
    units.extend("'>a{color:red}</style>".encode_utf16());
    let value = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(units)),
            parsing(),
        )
        .parse()
        .unwrap(),
    );
    let (mut session, installation) = installed(value);
    let id = installation.pending_association().unwrap();
    let stamp = session.snapshot().stamp();
    let policy = empty_policy();
    let policy_id = policy.policy().id().clone();
    let generation = policy.generation().clone();
    request_ready(&mut session, id, policy, envelope());
    let mut calls = 0;
    let mut duplicate = None;
    let mut input = observe(&session, id, &mut |request| {
        calls += 1;
        assert_eq!(request.qualification().contribution(), id);
        assert_eq!(request.policy().policy().id(), &policy_id);
        assert_eq!(request.policy().generation(), &generation);
        assert!(request.policy().policy().profile().is_none());
        assert!(
            request.text().is_empty(),
            "insertion precedes original text consumption"
        );
        assert_eq!(
            request.nonce(),
            Some(&DocumentString::from_utf16([0x4e, 0x26, 0xd800]))
        );
        assert_eq!(request.action(), HtmlStyleUpdateAction::CheckInlineCsp);
        duplicate = Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ));
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert_eq!(
        accept(&mut session, id, &mut input, envelope()).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(input.is_none());
    assert_eq!(calls, 1);
    assert!(
        matches!(accept(&mut session, id, &mut duplicate, envelope()),
        Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::DecisionAlreadyAccepted {contribution})) if contribution == id)
    );
    assert!(duplicate.is_some());
    assert_eq!(
        session
            .style_execution()
            .prepare_parsed_style(&mut session.dom(), id, envelope(), &cancel(),)
            .unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(
        session
            .parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .is_none()
    );
    still_pending(&session, id, stamp);
}

#[test]
fn an_actual_external_native_policy_can_block_only_the_selected_original_occurrence() {
    let (policy, profile) = native_policy(false);
    let policy_id = policy.policy().id().clone();
    let revision = policy.policy().revision();
    let (mut session, installation) = installed(report(
        "<!doctype html><style nonce=blocked>x{color:red}</style><style>later{color:blue}</style>",
    ));
    let id = installation.pending_association().unwrap();
    let stamp = session.snapshot().stamp();
    request_ready(&mut session, id, policy, envelope());
    let mut calls = 0;
    let mut input = observe(&session, id, &mut |request| {
        calls += 1;
        assert_eq!(request.policy().policy().id(), &policy_id);
        assert_eq!(request.policy().policy().revision(), revision);
        assert_eq!(request.policy().policy().profile(), Some(&profile));
        assert_eq!(request.nonce(), Some(&DocumentString::from("blocked")));
        let HtmlCspNativePolicy::External(native) = request
            .policy()
            .policy()
            .borrow_native_with::<NativePolicy, _>(&profile, &mut allow)
            .unwrap()
        else {
            panic!("explicit external artifact, not InitialEmpty")
        };
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request,
            native.allowed,
        ))
    });
    assert!(!input.as_ref().unwrap().allowed());
    assert_eq!(
        accept(&mut session, id, &mut input, envelope()).unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert!(input.is_none());
    assert_eq!(calls, 1);
    assert_eq!(
        session
            .style_execution()
            .prepare_parsed_style(&mut session.dom(), id, envelope(), &cancel(),)
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert!(
        session
            .parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .is_none()
    );
    still_pending(&session, id, stamp);
}

#[test]
fn an_unavailable_supplier_is_a_missing_observation_without_an_implicit_allow() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>x{color:red}</style>"));
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, empty_policy(), envelope());
    let stamp = session.snapshot().stamp();
    let mut input = observe(&session, id, &mut |_| None);
    assert!(matches!(
        accept(&mut session, id, &mut input, envelope()),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::MissingObservation
        ))
    ));
    assert!(input.is_none());
    let discarded = observe(&session, id, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    drop(discarded);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    still_pending(&session, id, stamp);
}

#[test]
fn wrong_target_and_foreign_destination_keep_the_original_decision_for_its_actual_request() {
    let value = report("<!doctype html><style>x{color:red}</style>");
    let (mut session, first) = installed(value.clone());
    let second = install_into(&mut session, value.clone());
    let (mut foreign, third) = installed(value);
    let a = first.pending_association().unwrap();
    let b = second.pending_association().unwrap();
    let c = third.pending_association().unwrap();
    request_ready(&mut session, a, empty_policy(), envelope());
    request_ready(&mut session, b, empty_policy(), envelope());
    request_ready(&mut foreign, c, empty_policy(), envelope());
    let mut input = observe(&session, a, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    let qualification = input.as_ref().unwrap().qualification().clone();
    assert!(matches!(
        accept(&mut session, b, &mut input, envelope()),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::WrongRequest
        ))
    ));
    assert_eq!(input.as_ref().unwrap().qualification(), &qualification);
    assert!(matches!(
        accept(&mut foreign, a, &mut input, envelope()),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::ForeignOwner
            ))
        ))
    ));
    assert_eq!(input.as_ref().unwrap().qualification(), &qualification);
    assert!(matches!(
        accept(&mut foreign, c, &mut input, envelope()),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::WrongRequest
        ))
    ));
    assert_eq!(input.as_ref().unwrap().qualification(), &qualification);
    assert_eq!(
        session.parsed_html_style_prefix(b).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    assert_eq!(
        foreign.parsed_html_style_prefix(c).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    assert_eq!(
        accept(&mut session, a, &mut input, envelope()).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let mut independent = observe(&foreign, c, &mut |request| {
        assert_eq!(
            request.qualification().occurrence(),
            qualification.occurrence()
        );
        assert_eq!(request.qualification().slot(), qualification.slot());
        assert_ne!(
            request.qualification().contribution(),
            qualification.contribution()
        );
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert_eq!(
        accept(&mut foreign, c, &mut independent, envelope()).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(independent.is_none());
    assert_eq!(
        session.parsed_html_style_prefix(a).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert_eq!(
        session.parsed_html_style_prefix(b).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
}

#[test]
fn cancel_and_late_output_refusal_retry_the_same_external_decision_without_evaluating_again() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>x{color:red}</style>"));
    let id = installation.pending_association().unwrap();
    let large = envelope();
    let one = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        1,
    )
    .unwrap();
    request_ready(&mut session, id, empty_policy(), one);
    let stamp = session.snapshot().stamp();
    let mut calls = 0;
    let mut input = observe(&session, id, &mut |request| {
        calls += 1;
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    let qualification = input.as_ref().unwrap().qualification().clone();
    assert!(matches!(
        session.style_execution().accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut input,
            one,
            &AtomicBool::new(true)
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    assert_eq!(input.as_ref().unwrap().qualification(), &qualification);
    assert!(matches!(
        accept(&mut session, id, &mut input, one),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 1
                }
            ))
        ))
    ));
    assert_eq!(input.as_ref().unwrap().qualification(), &qualification);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    still_pending(&session, id, stamp);
    let two = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        2,
    )
    .unwrap();
    assert_eq!(
        accept(&mut session, id, &mut input, two).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(input.is_none());
    assert_eq!(calls, 1);
    still_pending(&session, id, stamp);
}

#[test]
fn original_work_exhaustion_does_not_reset_or_consume_a_previously_evaluated_decision() {
    let (mut session, installation) =
        installed(report("<!doctype html><style>x{color:red}</style>"));
    let id = installation.pending_association().unwrap();
    let large = envelope();
    let limited = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        4096,
        large.outputs(),
    )
    .unwrap();
    request_ready(&mut session, id, empty_policy(), limited);
    let mut calls = 0;
    let mut input = observe(&session, id, &mut |request| {
        calls += 1;
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    let mut exhausted = false;
    for _ in 0..4096 {
        match accept(&mut session, id, &mut None, limited) {
            Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::MissingObservation)) => {}
            Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 4096,
                }),
            )))
            | Err(DomStyleParsedPrefixError::Runtime(DomStyleRuntimeError::Dom(
                DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 4096,
                },
            ))) => {
                exhausted = true;
                break;
            }
            other => panic!("unexpected refusal before actual original work exhaustion: {other:?}"),
        }
    }
    assert!(
        exhausted,
        "repeated missing inputs spend original work rather than resetting it"
    );
    assert!(matches!(
        accept(&mut session, id, &mut input, limited),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 4096
                }
            ))
        )) | Err(DomStyleParsedPrefixError::Runtime(
            DomStyleRuntimeError::Dom(DomError::WorkLimit {
                resource: DomResource::ParsedAssociationProcessingWork,
                limit: 4096
            })
        ))
    ));
    assert!(input.is_some());
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let raised = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        8192,
        large.outputs(),
    )
    .unwrap();
    assert_eq!(
        accept(&mut session, id, &mut input, raised).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert_eq!(calls, 1);
}

#[test]
fn source_session_and_forgotten_read_custody_refuse_without_losing_the_evaluated_input() {
    let value = report("<!doctype html><style>x{color:red}</style>");
    let weak = Arc::downgrade(&value);
    let (mut session, document) = bound_dispatch_session();
    let parent = super::super::meta::container(&session, document);
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    let created = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();
    let installation = install_into(&mut session, value);
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, empty_policy(), envelope());
    let mut input = observe(&session, id, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    let (mut other, _) = bound_dispatch_session();
    assert!(matches!(
        session.style_execution().accept_parsed_style_inline_policy(
            &mut other.dom(),
            id,
            &mut input,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(input.is_some());
    let pending = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(dialog),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(event),
        ..
    } = pending
    else {
        panic!("actual BeforeToggle tail")
    };
    assert!(matches!(accept(&mut session,id,&mut input,envelope()),
        Err(DomStyleParsedPrefixError::Runtime(DomStyleRuntimeError::SourcePending(operation))) if operation==event.operation()));
    assert!(input.is_some());
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        event.operation()
    );
    drop(event);
    drop(installation);
    drop(created);
    drop(session);
    assert!(
        weak.upgrade().is_none(),
        "fixed decision stamp retains no report or policy graph"
    );
    assert_eq!(input.as_ref().unwrap().qualification().contribution(), id);

    let value = report("<!doctype html><style>x{color:red}</style>");
    let weak = Arc::downgrade(&value);
    let (mut session, installation) = installed(value);
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, empty_policy(), envelope());
    let mut input = observe(&session, id, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    std::mem::forget(session.parsed_html_style_prefix(id).unwrap());
    assert!(matches!(accept(&mut session,id,&mut input,envelope()),
        Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::Busy {contribution})) if contribution==id));
    assert!(input.is_some());
    drop(installation);
    drop(session);
    assert!(
        weak.upgrade().is_none(),
        "forgotten borrowed view owns no graph"
    );
}
