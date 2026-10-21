//! Original CreateSheet joins the actual producer and sole add-sheet state.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, report};
use super::style_decision::{empty_policy, installed, observe, request_ready};
use surgeist_css::CssUtf16InputError;
use surgeist_dom::*;

pub(super) fn ready(
    value: &str,
    limits: DomParsedHtmlAssociationLimits,
) -> (DomSession, DomParsedHtmlInstallation) {
    let (mut session, installation) = installed(report(value));
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, empty_policy(), limits);
    let mut observation = observe(&session, id, &mut |request| {
        assert!(
            request.text().is_empty(),
            "first insertion precedes parser text"
        );
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut observation,
                limits,
                &cancel()
            )
            .unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    (session, installation)
}
pub(super) fn bind(
    session: &mut DomSession,
    document: DocumentId,
    limits: surgeist_cssom::CssomLimits,
) {
    session
        .style_execution()
        .bind_root(
            &session.dom(),
            "original-root".into(),
            &mut Some(DomStyleCssomRoot::new(
                document.node(),
                CssomStore::new(limits, Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
}
#[expect(
    clippy::result_large_err,
    reason = "exact refusal assertions retain the actual root and producer error without allocation"
)]
pub(super) fn create(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    limits: DomParsedHtmlAssociationLimits,
) -> Result<DomStyleParsedSheetOutcome, DomStyleParsedSheetError> {
    session
        .style_execution()
        .create_parsed_style_sheet(&mut session.dom(), id, limits, &cancel())
}
pub(super) fn pending(session: &DomSession, id: DomParsedHtmlAssociationId) {
    assert!(
        session
            .pending_parsed_html_associations()
            .any(|entry| entry.id() == id)
    );
}
fn unchanged(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    document: DocumentId,
    stamp: DomStamp,
    revision: &surgeist_cssom::CssomRevision,
) {
    assert!(session.state.borrow().completions.is_empty());
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let snapshot = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.revision(), revision);
    assert!(snapshot.sheets().is_empty());
    let names = session
        .style_execution()
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert!(names.preferred_name().is_empty());
    assert_eq!(names.last_selected_name(), None);
    pending(session, id);
}

#[test]
fn original_empty_sheet_joins_exact_title_media_preference_and_stops_before_script_blocking() {
    let (mut session, installation) = ready(
        "<!doctype html><style title='A&amp;B' media='screen' nonce='original'>p{color:red}</style>",
        envelope(),
    );
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(&mut session, document, Default::default());
    let before = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    let result = create(&mut session, id, envelope()).unwrap();
    assert_eq!(result.completion().contribution(), id);
    assert_eq!(
        result.completion().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    let sheet = result.completion().sheet();
    assert_eq!(sheet.root(), document.node());
    let actual = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(actual.sheets(), &[sheet.id().clone()]);
    assert_ne!(actual.revision(), before.revision());
    assert_eq!(sheet.revision(), actual.revision());
    let css = actual.sheet(sheet.id()).unwrap();
    assert!(
        actual.sheet_rules(sheet.id()).unwrap().is_empty(),
        "original first text, not final p rule"
    );
    assert_eq!(
        css.inputs().title.as_slice(),
        DocumentString::from("A&B").code_units()
    );
    assert!(!css.inputs().disabled);
    assert!(!css.inputs().constructed);
    assert!(!css.inputs().alternate);
    assert!(css.inputs().origin_clean);
    assert!(css.inputs().owner.is_some());
    assert_eq!(
        actual.media(css.media()).unwrap(),
        &CssParserContext::default()
            .parse_media_query_list("screen")
            .into_parts()
            .0
    );
    assert_eq!(
        result.observation().preferred_name(),
        &DocumentString::from("A&B")
    );
    assert_eq!(result.observation().last_selected_name(), None);
    assert_eq!(result.observation().cssom_revision(), actual.revision());
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    let node = session.snapshot();
    let style = node
        .node(sheet.association().element())
        .unwrap()
        .as_style()
        .unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    assert!(matches!(
        style.sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    pending(&session, id);
    let revision = actual.revision().clone();
    assert!(create(&mut session, id, envelope()).is_err());
    assert_eq!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .revision(),
        &revision
    );
}

#[test]
fn missing_owner_and_early_cancellation_leave_original_action_retriable() {
    let (mut session, installation) =
        ready("<!doctype html><style title=A>x{}</style>", envelope());
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    let stamp = session.snapshot().stamp();
    assert!(
        matches!(create(&mut session,id,envelope()),Err(DomStyleParsedSheetError::Runtime(DomStyleRuntimeError::SheetSetRootUnavailable{root})) if root==document.node())
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .is_none()
    );
    bind(&mut session, document, Default::default());
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    assert!(matches!(
        session.style_execution().create_parsed_style_sheet(
            &mut session.dom(),
            id,
            envelope(),
            &AtomicBool::new(true)
        ),
        Err(DomStyleParsedSheetError::Sheet(
            DomParsedHtmlStyleSheetError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ))
    ));
    unchanged(&mut session, id, document, stamp, &revision);
    drop(create(&mut session, id, envelope()).unwrap());
}

#[test]
fn css_input_refusal_after_empty_body_preserves_producer_names_and_cursor_for_retry() {
    let (mut session, installation) = ready(
        "<!doctype html><style title=A media='screen'>x{}</style>",
        envelope(),
    );
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(&mut session, document, Default::default());
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    let mut runtime = session.style_execution();
    let mut small = limits();
    small.css_input.max_input_units = 0;
    runtime
        .set_limits(&session.dom(), small, &cancel())
        .unwrap();
    assert!(matches!(
        create(&mut session, id, envelope()),
        Err(DomStyleParsedSheetError::Css(CssUtf16InputError::Limit(
            "original UTF16 units"
        )))
    ));
    unchanged(&mut session, id, document, stamp, &revision);
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    drop(create(&mut session, id, envelope()).unwrap());
}

#[test]
fn actual_cssom_capacity_refusal_cannot_publish_a_partial_association_or_preference() {
    let (mut session, installation) =
        ready("<!doctype html><style title=A>x{}</style>", envelope());
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(
        &mut session,
        document,
        surgeist_cssom::CssomLimits {
            max_objects: 1,
            ..Default::default()
        },
    );
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    assert!(
        matches!(create(&mut session,id,envelope()),Err(DomStyleParsedSheetError::Sheet(DomParsedHtmlStyleSheetError::Cssom{root,error:surgeist_cssom::CssomError::Limit { resource:"objects", maximum:1 }})) if root==document.node())
    );
    unchanged(&mut session, id, document, stamp, &revision);
}

#[test]
fn foreign_session_and_forgotten_canonical_request_loan_refuse_before_creation() {
    let (mut session, installation) =
        ready("<!doctype html><style title=A>x{}</style>", envelope());
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(&mut session, document, Default::default());
    let (mut foreign, _) = bound_dispatch_session();
    assert!(matches!(
        session.style_execution().create_parsed_style_sheet(
            &mut foreign.dom(),
            id,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedSheetError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    std::mem::forget(session.parsed_html_style_prefix(id).unwrap());
    assert!(
        matches!(create(&mut session,id,envelope()),Err(DomStyleParsedSheetError::Sheet(DomParsedHtmlStyleSheetError::Style(DomParsedHtmlStyleError::Busy{contribution}))) if contribution==id)
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .revision(),
        &revision
    );
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
}

#[test]
fn late_output_refusal_keeps_all_three_owners_for_a_raised_same_action_retry() {
    let large = envelope();
    let limited = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        4,
    )
    .unwrap();
    let (mut session, installation) =
        ready("<!doctype html><style title=chosen>x{}</style>", limited);
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(&mut session, document, Default::default());
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    assert!(matches!(
        create(&mut session, id, limited),
        Err(DomStyleParsedSheetError::Sheet(
            DomParsedHtmlStyleSheetError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 4
                })
            ))
        ))
    ));
    unchanged(&mut session, id, document, stamp, &revision);
    let result = create(&mut session, id, large).unwrap();
    assert_eq!(
        result.observation().preferred_name(),
        &DocumentString::from("chosen")
    );
    assert_eq!(
        result.completion().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    pending(&session, id);
}

#[test]
fn original_poll_work_exhausts_across_real_parse_refusals_then_raised_retry_succeeds() {
    let large = envelope();
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        4096,
        large.outputs(),
    )
    .unwrap();
    let (mut session, installation) = ready(
        "<!doctype html><style title=chosen media=screen>x{}</style>",
        finite,
    );
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(&mut session, document, Default::default());
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    let mut runtime = session.style_execution();
    let mut small = limits();
    small.css_input.max_input_units = 0;
    runtime
        .set_limits(&session.dom(), small, &cancel())
        .unwrap();
    let mut attempts = 0;
    loop {
        attempts += 1;
        match create(&mut session, id, finite) {
            Err(DomStyleParsedSheetError::Css(CssUtf16InputError::Limit(
                "original UTF16 units",
            ))) => {}
            Err(DomStyleParsedSheetError::Css(CssUtf16InputError::Interrupted(
                DomStyleRuntimeError::Dom(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 4096,
                }),
            )))
            | Err(DomStyleParsedSheetError::Runtime(DomStyleRuntimeError::Dom(
                DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 4096,
                },
            )))
            | Err(DomStyleParsedSheetError::Sheet(DomParsedHtmlStyleSheetError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::WorkLimit {
                        resource: DomResource::ParsedAssociationProcessingWork,
                        limit: 4096,
                    },
                )),
            ))) => break,
            other => panic!("unexpected original attempt refusal: {other:?}"),
        }
        assert!(
            attempts < 4096,
            "finite original work must not reset per attempt"
        );
    }
    assert!(attempts > 1);
    unchanged(&mut session, id, document, stamp, &revision);
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    let raised = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        8192,
        large.outputs(),
    )
    .unwrap();
    drop(create(&mut session, id, raised).unwrap());
}

#[test]
fn recovered_original_media_diagnostics_keep_authentic_source_after_completion_drop() {
    let (mut session, installation) = ready(
        "<!doctype html><style media='screen and' title='Ω'>x{}</style>",
        envelope(),
    );
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    bind(&mut session, document, Default::default());
    let result = create(&mut session, id, envelope()).unwrap();
    assert!(!result.completion().media_diagnostics().is_empty());
    let diagnostic = &result.completion().media_diagnostics()[0];
    let source = diagnostic.source().unwrap();
    assert_eq!(
        source.original_utf16().unwrap().as_slice(),
        DocumentString::from("screen and").code_units()
    );
    let retained = source.clone();
    let sheet = result.completion().sheet().clone();
    drop(result);
    drop(installation);
    assert_eq!(
        retained.original_utf16().unwrap().as_slice(),
        DocumentString::from("screen and").code_units()
    );
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .sheet(sheet.id())
            .is_ok()
    );
    pending(&session, id);
}

#[test]
fn actual_source_tail_refuses_original_creation_without_replacing_either_operation() {
    let (mut session, document) = bound_dispatch_session();
    let parent = super::super::meta::container(&session, document);
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    let created = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();
    let installation = super::style_decision::install_into(
        &mut session,
        report("<!doctype html><style title=A>x{}</style>"),
    );
    let id = installation.pending_association().unwrap();
    let parsed_document = installation.document();
    request_ready(&mut session, id, empty_policy(), envelope());
    let mut observation = observe(&session, id, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut observation,
            envelope(),
            &cancel(),
        )
        .unwrap();
    bind(&mut session, parsed_document, Default::default());
    let progress = session
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
    } = progress
    else {
        panic!("genuine BeforeToggle event")
    };
    let revision = session
        .style_execution()
        .snapshot(parsed_document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    assert!(
        matches!(create(&mut session,id,envelope()),Err(DomStyleParsedSheetError::Runtime(DomStyleRuntimeError::SourcePending(operation))) if operation==event.operation())
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        event.operation()
    );
    assert_eq!(
        session
            .style_execution()
            .snapshot(parsed_document.node())
            .unwrap()
            .unwrap()
            .revision(),
        &revision
    );
    assert!(
        session
            .style_execution()
            .snapshot(parsed_document.node())
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    pending(&session, id);
}

#[test]
fn original_quirks_mode_and_raw_utf16_title_are_preserved_by_the_real_producer() {
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource};
    let mut units = "<style title='A".encode_utf16().collect::<Vec<_>>();
    units.push(0xd800);
    units.extend("' media=screen nonce='original'>p{width:10}</style>".encode_utf16());
    let value = std::sync::Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(units)),
            super::consumption::parsing(),
        )
        .parse()
        .unwrap(),
    );
    let (mut session, installation) = installed(value);
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    request_ready(&mut session, id, empty_policy(), envelope());
    let mut input = observe(&session, id, &mut |request| {
        assert_eq!(request.nonce(), Some(&DocumentString::from("original")));
        assert!(request.text().is_empty());
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    bind(&mut session, document, Default::default());
    let outcome = create(&mut session, id, envelope()).unwrap();
    let snapshot = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    let sheet = snapshot.sheet(outcome.completion().sheet().id()).unwrap();
    assert_eq!(sheet.inputs().title.as_slice(), &[0x41, 0xd800]);
    assert_eq!(
        outcome.observation().preferred_name().code_units(),
        &[0x41, 0xd800]
    );
    assert!(snapshot.context().inputs.iter().any(|input|matches!(input.data,CssomInputData::ParserMode(context) if context.mode()==CssParserMode::Quirks)));
    assert!(
        snapshot
            .sheet_rules(outcome.completion().sheet().id())
            .unwrap()
            .is_empty()
    );
    pending(&session, id);
}

#[test]
fn an_explicitly_blocked_original_decision_cannot_create_a_sheet() {
    let (policy, _) = super::style_decision::native_policy(false);
    let (mut session, installation) =
        installed(report("<!doctype html><style title=blocked>x{}</style>"));
    let id = installation.pending_association().unwrap();
    let document = installation.document();
    request_ready(&mut session, id, policy, envelope());
    let mut input = observe(&session, id, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, false,
        ))
    });
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut input,
                envelope(),
                &cancel()
            )
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    bind(&mut session, document, Default::default());
    let before = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert!(matches!(
        create(&mut session, id, envelope()),
        Err(DomStyleParsedSheetError::Sheet(
            DomParsedHtmlStyleSheetError::Style(DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::InlinePolicyBlocked
            })
        ))
    ));
    let after = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(before.revision(), after.revision());
    assert!(after.sheets().is_empty());
    pending(&session, id);
}
