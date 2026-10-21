//! Real original Style preparation, without policy decisions or sheet creation.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, environment, installation, parsing, report};
use std::sync::Arc;
use surgeist_dom::*;
use surgeist_html::{HtmlCspAssociation, HtmlCspOwner};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseReport, HtmlSource};

fn install(value: Arc<HtmlParseReport>) -> (DomSession, DomParsedHtmlInstallation) {
    let (mut session, _) = bound_dispatch_session();
    let environment = environment(session.dom().raw());
    let installed = session
        .install_parsed_html_document(
            session.snapshot().stamp(),
            value,
            environment,
            installation(),
            &cancel(),
        )
        .unwrap();
    (session, installed)
}
fn empty_policy() -> HtmlCspAssociation {
    let mut admit = |_| Ok::<_, std::convert::Infallible>(());
    HtmlCspOwner::new_with(&mut admit)
        .unwrap()
        .initial_document_with(&mut admit)
        .unwrap()
}
fn prepare(session: &mut DomSession, id: DomParsedHtmlAssociationId) -> DomParsedHtmlStylePhase {
    session
        .style_execution()
        .prepare_parsed_style(&mut session.dom(), id, envelope(), &cancel())
        .unwrap()
}
fn binding_request(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInitialPolicyRequest {
    session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone()
}

#[test]
fn exact_original_text_nonce_and_explicit_policy_reach_only_the_inline_request() {
    let mut input = "<!doctype html><style nonce='N&amp;"
        .encode_utf16()
        .collect::<Vec<_>>();
    input.push(0xd800);
    input.extend("'>a{content:'".encode_utf16());
    input.push(0xd800);
    input.extend("';}\r\n</style>".encode_utf16());
    let value = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(input)),
            parsing(),
        )
        .parse()
        .unwrap(),
    );
    let mut cursor = value.pending_associations();
    let original = loop {
        if let surgeist_html_parser::HtmlParsePendingAssociationStep::Occurrence(row) =
            cursor.advance()
            && row.requires_processing()
        {
            assert!(matches!(
                row.kind(),
                surgeist_html_parser::HtmlParseAssociationOccurrenceKind::StyleUpdate(_)
            ));
            break (
                row.id(),
                row.subject().id(),
                row.context().epoch(),
                row.context()
                    .slot(surgeist_html_parser::HtmlParseAssociationContextRole::InitialPolicy)
                    .unwrap(),
                match row.document() {
                    surgeist_html_parser::HtmlParseMetadataDocument::Owned(node) => {
                        DomParsedHtmlHistoricalDocument::Constructed(node.id())
                    }
                    surgeist_html_parser::HtmlParseMetadataDocument::Captured(_) => {
                        panic!("ordinary original Document")
                    }
                },
            );
        }
    };
    let (mut session, installed) = install(value);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let stamp = session.snapshot().stamp();
    assert_eq!(
        prepare(&mut session, id),
        DomParsedHtmlStylePhase::AwaitingInitialPolicy
    );
    let request = binding_request(&session, id);
    assert_eq!(request.qualification().contribution(), id);
    assert_eq!(request.qualification().document(), document);
    assert_eq!(request.qualification().occurrence(), &original.0);
    assert_eq!(request.qualification().subject(), &original.1);
    assert_eq!(request.qualification().context_epoch(), &original.2);
    assert_eq!(request.slot(), &original.3);
    assert_eq!(request.qualification().original_document(), &original.4);
    assert_eq!(
        prepare(&mut session, id),
        DomParsedHtmlStylePhase::AwaitingInitialPolicy
    );
    assert_eq!(binding_request(&session, id), request);
    let policy = empty_policy();
    let policy_id = policy.policy().id().clone();
    let revision = policy.policy().revision();
    let generation = policy.generation().clone();
    let mut input = Some(policy);
    session
        .style_execution()
        .bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert!(input.is_none());
    assert_eq!(
        prepare(&mut session, id),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let expected_text = DocumentString::from_utf16(
        "a{content:'"
            .encode_utf16()
            .chain([0xd800])
            .chain("';}\n".encode_utf16())
            .collect::<Vec<_>>(),
    );
    let expected_nonce = DocumentString::from_utf16(vec![0x4e, 0x26, 0xd800]);
    {
        let snapshot = installed.snapshot();
        let style = installed.node_for(&original.1).unwrap();
        let mut final_units = Vec::new();
        for child in snapshot.node(style).unwrap().children() {
            if let DomNodeKind::Text(text) = snapshot.node(*child).unwrap().kind() {
                final_units.extend_from_slice(text.code_units());
            }
        }
        assert_eq!(final_units, expected_text.code_units());
    }
    {
        let view = session.parsed_html_style_prefix(id).unwrap();
        let inline = view.inline_policy_request().unwrap();
        assert_eq!(inline.qualification(), request.qualification());
        assert!(
            inline.text().is_empty(),
            "first insertion precedes original text consumption"
        );
        assert_eq!(inline.nonce(), Some(&expected_nonce));
        assert_eq!(
            inline.action(),
            surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp
        );
        assert!(inline.state().created_by_document_parser());
        assert!(!inline.state().on_parser_stack());
        assert_eq!(inline.policy().policy().id(), &policy_id);
        assert_eq!(inline.policy().policy().revision(), revision);
        assert_eq!(inline.policy().generation(), &generation);
    }
    assert_eq!(
        prepare(&mut session, id),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        session
            .pending_parsed_html_associations()
            .next()
            .unwrap()
            .id(),
        id
    );
}

#[test]
fn independent_installations_bind_distinct_explicit_policies_without_rewriting_original_slots() {
    let value = report("<!doctype html><style>p {color:red}</style>");
    let (mut a, a_install) = install(value.clone());
    let (mut b, b_install) = install(value);
    let a_id = a_install.pending_association().unwrap();
    let b_id = b_install.pending_association().unwrap();
    assert_ne!(a_id, b_id);
    prepare(&mut a, a_id);
    prepare(&mut b, b_id);
    let a_request = binding_request(&a, a_id);
    let b_request = binding_request(&b, b_id);
    assert_eq!(
        a_request.qualification().occurrence(),
        b_request.qualification().occurrence()
    );
    assert_eq!(a_request.slot(), b_request.slot());
    let a_policy = empty_policy();
    let b_policy = empty_policy();
    let a_policy_id = a_policy.policy().id().clone();
    let b_policy_id = b_policy.policy().id().clone();
    assert_ne!(a_policy_id, b_policy_id);
    a.style_execution()
        .bind_parsed_style_initial_policy(
            &mut a.dom(),
            &a_request,
            &mut Some(a_policy),
            envelope(),
            &cancel(),
        )
        .unwrap();
    b.style_execution()
        .bind_parsed_style_initial_policy(
            &mut b.dom(),
            &b_request,
            &mut Some(b_policy),
            envelope(),
            &cancel(),
        )
        .unwrap();
    prepare(&mut a, a_id);
    prepare(&mut b, b_id);
    assert_eq!(
        a.parsed_html_style_prefix(a_id)
            .unwrap()
            .inline_policy_request()
            .unwrap()
            .policy()
            .policy()
            .id(),
        &a_policy_id
    );
    assert_eq!(
        b.parsed_html_style_prefix(b_id)
            .unwrap()
            .inline_policy_request()
            .unwrap()
            .policy()
            .policy()
            .id(),
        &b_policy_id
    );
}

#[test]
fn accepted_binding_survives_cancel_and_refuses_replacement_without_losing_input() {
    let (mut session, installed) = install(report("<!doctype html><style>p {color:red}</style>"));
    let id = installed.pending_association().unwrap();
    prepare(&mut session, id);
    let request = binding_request(&session, id);
    let mut missing = None;
    assert!(matches!(
        session.style_execution().bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut missing,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::MissingPolicy
        ))
    ));
    let policy = empty_policy();
    let policy_id = policy.policy().id().clone();
    session
        .style_execution()
        .bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut Some(policy),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let mut replacement = Some(empty_policy());
    let replacement_id = replacement.as_ref().unwrap().policy().id().clone();
    assert!(matches!(
        session.style_execution().bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut replacement,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::AlreadyBound
        ))
    ));
    assert_eq!(replacement.as_ref().unwrap().policy().id(), &replacement_id);
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        session.style_execution().prepare_parsed_style(
            &mut session.dom(),
            id,
            envelope(),
            &cancelled
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    assert_eq!(
        prepare(&mut session, id),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let maximum = envelope();
    let reduced = DomParsedHtmlAssociationLimits::new(
        maximum.retained_bytes(),
        maximum.retained_units(),
        maximum.work() - 1,
        maximum.outputs(),
    )
    .unwrap();
    assert!(matches!(
        session.style_execution().prepare_parsed_style(&mut session.dom(), id, reduced, &cancel()),
        Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::Admission(
            DomParsedHtmlAssociationAdmissionError::LimitReduction { contribution }
        ))) if contribution == id
    ));
    assert_eq!(
        session
            .parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .unwrap()
            .policy()
            .policy()
            .id(),
        &policy_id
    );
}

#[test]
fn late_request_admission_keeps_accepted_policy_and_forgetting_a_view_keeps_canonical_custody() {
    let value = report("<!doctype html><style>p{color:red}</style>");
    let weak = Arc::downgrade(&value);
    let (mut session, installed) = install(value);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let large = envelope();
    let no_outputs = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        0,
    )
    .unwrap();
    assert_eq!(
        session
            .style_execution()
            .prepare_parsed_style(&mut session.dom(), id, no_outputs, &cancel())
            .unwrap(),
        DomParsedHtmlStylePhase::AwaitingInitialPolicy
    );
    let request = binding_request(&session, id);
    let policy = empty_policy();
    let policy_id = policy.policy().id().clone();
    session
        .style_execution()
        .bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut Some(policy),
            no_outputs,
            &cancel(),
        )
        .unwrap();
    let stamp = session.snapshot().stamp();
    assert!(matches!(
        session.style_execution().prepare_parsed_style(
            &mut session.dom(),
            id,
            no_outputs,
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 0
                }
            ))
        ))
    ));
    let mut replacement = Some(empty_policy());
    assert!(matches!(
        session.style_execution().bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut replacement,
            no_outputs,
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::AlreadyBound
        ))
    ));
    assert!(replacement.is_some());
    assert_eq!(session.snapshot().stamp(), stamp);
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        prepare(&mut session, id),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    {
        let view = session.parsed_html_style_prefix(id).unwrap();
        assert_eq!(
            view.inline_policy_request().unwrap().policy().policy().id(),
            &policy_id
        );
    }
    drop(installed);
    let view = session.parsed_html_style_prefix(id).unwrap();
    std::mem::forget(view);
    assert!(
        matches!(session.style_execution().prepare_parsed_style(&mut session.dom(), id, large, &cancel()), Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::Busy {contribution})) if contribution == id)
    );
    drop(session);
    assert!(
        weak.upgrade().is_none(),
        "forgotten borrowed view owns no report graph"
    );
}

#[test]
fn an_earlier_real_meta_refuses_but_proved_ignored_style_does_not_hide_the_first_update() {
    let (mut blocked, installed) = install(report(
        "<!doctype html><meta http-equiv=default-style content=A><style>p {color:red}</style>",
    ));
    let id = installed.pending_association().unwrap();
    let stamp = blocked.snapshot().stamp();
    assert!(matches!(
        blocked.style_execution().prepare_parsed_style(&mut blocked.dom(), id, envelope(), &cancel()),
        Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::UnsupportedProfile {contribution})) if contribution == id
    ));
    assert_eq!(blocked.snapshot().stamp(), stamp);
    assert_eq!(
        blocked
            .pending_parsed_html_associations()
            .next()
            .unwrap()
            .id(),
        id
    );

    let (mut session, installed) = install(report(
        "<!doctype html><style type=text/plain nonce=ignored>ignored</style><style nonce=chosen>chosen {color:blue}</style>",
    ));
    let id = installed.pending_association().unwrap();
    prepare(&mut session, id);
    let request = binding_request(&session, id);
    session
        .style_execution()
        .bind_parsed_style_initial_policy(
            &mut session.dom(),
            &request,
            &mut Some(empty_policy()),
            envelope(),
            &cancel(),
        )
        .unwrap();
    prepare(&mut session, id);
    let view = session.parsed_html_style_prefix(id).unwrap();
    assert!(view.inline_policy_request().unwrap().text().is_empty());
    assert_eq!(
        view.inline_policy_request().unwrap().nonce(),
        Some(&DocumentString::from("chosen"))
    );
}

#[test]
fn real_source_custody_and_foreign_session_refuse_without_taking_policy_input() {
    let (mut session, document) = bound_dispatch_session();
    let parent = super::super::meta::container(&session, document);
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    let created = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();
    let environment = environment(session.dom().raw());
    let installed = session
        .install_parsed_html_document(
            session.snapshot().stamp(),
            report("<!doctype html><style>p{color:red}</style>"),
            environment,
            installation(),
            &cancel(),
        )
        .unwrap();
    let id = installed.pending_association().unwrap();
    prepare(&mut session, id);
    let request = binding_request(&session, id);
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
    let mut input = Some(empty_policy());
    let policy_id = input.as_ref().unwrap().policy().id().clone();
    assert!(
        matches!(session.style_execution().bind_parsed_style_initial_policy(&mut session.dom(), &request, &mut input, envelope(), &cancel()), Err(DomStyleParsedPrefixError::Runtime(DomStyleRuntimeError::SourcePending(operation))) if operation == event.operation())
    );
    assert_eq!(input.as_ref().unwrap().policy().id(), &policy_id);
    let (mut other, _) = bound_dispatch_session();
    assert!(matches!(
        session
            .style_execution()
            .prepare_parsed_style(&mut other.dom(), id, envelope(), &cancel()),
        Err(DomStyleParsedPrefixError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        other.style_execution().bind_parsed_style_initial_policy(
            &mut other.dom(),
            &request,
            &mut input,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::ForeignOwner
            ))
        ))
    ));
    assert_eq!(input.as_ref().unwrap().policy().id(), &policy_id);
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(binding_request(&session, id), request);
}

#[test]
fn captured_policy_authority_is_not_replaced_by_the_current_closed_document() {
    use surgeist_html_parser::{HtmlFragmentParseRequest, HtmlParseAssociationContextRequirement};
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};
    let (mut session, document) = bound_dispatch_session();
    let snapshot = session.snapshot();
    let document = snapshot.node(document).unwrap().as_document().unwrap().id();
    let host = snapshot.node(document.node()).unwrap().children()[0];
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let value = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template shadowrootmode=open><style>p{color:red}</style></template>",
            ),
            context,
            parsing(),
            surgeist_html::HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let mut cursor = value.pending_associations();
    loop {
        if let surgeist_html_parser::HtmlParsePendingAssociationStep::Occurrence(row) =
            cursor.advance()
            && row.requires_processing()
        {
            assert_eq!(
                row.context().requirement(
                    surgeist_html_parser::HtmlParseAssociationContextRole::InitialPolicy
                ),
                HtmlParseAssociationContextRequirement::DeferredCaptured
            );
            break;
        }
    }
    let installed = session
        .install_parsed_html_fragment(value, &capture, installation(), &cancel())
        .unwrap();
    let id = installed.pending_association().unwrap();
    let stamp = session.snapshot().stamp();
    assert!(
        matches!(session.style_execution().prepare_parsed_style(&mut session.dom(), id, envelope(), &cancel()), Err(DomStyleParsedPrefixError::Style(DomParsedHtmlStyleError::UnsupportedProfile {contribution})) if contribution == id)
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session
            .pending_parsed_html_associations()
            .next()
            .unwrap()
            .id(),
        id
    );
    let view = session.parsed_html_style_prefix(id).unwrap();
    assert!(view.initial_policy_request().is_none());
    assert!(view.inline_policy_request().is_none());
}

#[test]
fn original_intrinsic_csp_gap_refuses_structural_installation_without_a_style_prefix() {
    let (mut session, _) = bound_dispatch_session();
    let environment = environment(session.dom().raw());
    let stamp = session.snapshot().stamp();
    let usage = session.dom().usage();
    let value = report(
        "<!doctype html><meta http-equiv=Content-Security-Policy content=\"style-src 'none'\"><style>p{color:red}</style>",
    );
    let error = match session.install_parsed_html_document(
        stamp,
        value,
        environment,
        installation(),
        &cancel(),
    ) {
        Err(error) => error,
        Ok(_) => panic!("unavailable original policy cannot publish structure"),
    };
    assert!(
        matches!(
            error,
            DomStyleParsedInstallError::Installation(
                DomParsedHtmlInstallError::AssociationCaptureUnavailable {
                    reason: surgeist_html_parser::HtmlParseAssociationMissingKind::PolicyHistory,
                    ..
                }
            )
        ),
        "{error:?}"
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(session.dom().usage(), usage);
    assert!(session.pending_parsed_html_associations().next().is_none());
}
