//! Actual original creation then explicit no-engine disposition, never a bool.
use super::*;
use surgeist_cssom::{CssomRevision, CssomSheetId};
use surgeist_html::HtmlStyleExecutionProfile;

struct Created {
    dom: Dom,
    document: DocumentId,
    id: DomParsedHtmlAssociationId,
    qualification: DomParsedHtmlStyleInitialPolicyRequest,
    root: DomStyleCssomRoot,
    sheet: CssomSheetId,
    revision: CssomRevision,
}
fn created(source: &str, limits: DomParsedHtmlAssociationLimits) -> Created {
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, parse(source), env);
    decision::ready(&mut dom, id, limits);
    let qualification = qualifier(&dom, id);
    let mut observation = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        assert_eq!(
            view.script_bookkeeping_exclusion(),
            None,
            "selecting the profile does not accept an exclusion"
        );
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    dom.begin_parsed_html_style(id, limits, &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    let root = create::root(document);
    let (sheet, revision) = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        let mut prep = hold.prepare_create_sheet(&root, &flag()).unwrap();
        create::stage(&mut prep);
        let (completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
        let values = (
            completion.sheet().id().clone(),
            completion.sheet().revision().clone(),
        );
        drop(retirement);
        values
    };
    Created {
        dom,
        document,
        id,
        qualification,
        root,
        sheet,
        revision,
    }
}
fn unchanged(value: &Created, stamp: DomStamp) {
    assert_eq!(value.dom.snapshot().stamp(), stamp);
    let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
    assert_eq!(
        view.phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    assert_eq!(view.script_bookkeeping_exclusion(), None);
    assert_eq!(view.initial_policy_request(), Some(&value.qualification));
    drop(view);
    assert_eq!(
        value
            .dom
            .pending_parsed_html_association(value.document)
            .unwrap(),
        Some(value.id)
    );
    let snapshot = value.root.snapshot().unwrap();
    assert_eq!(snapshot.revision(), &value.revision);
    assert_eq!(snapshot.sheets(), std::slice::from_ref(&value.sheet));
}
#[test]
fn original_no_engine_exclusion_reaches_the_real_render_wait_without_recreating_or_acknowledging() {
    for media in ["screen", "all"] {
        let mut value = created(
            &format!("<!doctype html><style title=A media='{media}' nonce=n>later</style>"),
            envelope(),
        );
        let stamp = value.dom.snapshot().stamp();
        unchanged(&value, stamp);
        {
            let mut hold = value
                .dom
                .begin_parsed_html_style(value.id, envelope(), &flag())
                .unwrap();
            assert_eq!(
                hold.exclude_script_bookkeeping(
                    &value.qualification,
                    HtmlStyleExecutionProfile::NoExecutableJavaScript,
                    &flag()
                )
                .unwrap(),
                DomParsedHtmlStylePhase::ReadRenderBlockingPending
            );
        }
        let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
        assert_eq!(
            view.phase(),
            DomParsedHtmlStylePhase::ReadRenderBlockingPending
        );
        assert_eq!(
            view.script_bookkeeping_exclusion(),
            Some(HtmlStyleExecutionProfile::NoExecutableJavaScript)
        );
        assert_eq!(view.initial_policy_request(), Some(&value.qualification));
        drop(view);
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::ReadRenderBlockingPending
        );
        assert!(matches!(
            hold.exclude_script_bookkeeping(
                &value.qualification,
                HtmlStyleExecutionProfile::NoExecutableJavaScript,
                &flag()
            ),
            Err(DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::ReadRenderBlockingPending
            })
        ));
        drop(hold);
        assert_eq!(value.dom.snapshot().stamp(), stamp);
        let snapshot = value.root.snapshot().unwrap();
        assert_eq!(snapshot.revision(), &value.revision);
        assert_eq!(snapshot.sheets(), std::slice::from_ref(&value.sheet));
        assert_eq!(
            value
                .dom
                .pending_parsed_html_association(value.document)
                .unwrap(),
            Some(value.id)
        );
    }
}
#[test]
fn wrong_profile_foreign_occurrence_and_cancellation_preserve_the_original_wait() {
    let mut value = created(
        "<!doctype html><style media=screen>later</style>",
        envelope(),
    );
    let foreign = created(
        "<!doctype html><style media=screen>later</style>",
        envelope(),
    );
    let stamp = value.dom.snapshot().stamp();
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.exclude_script_bookkeeping(
                &value.qualification,
                HtmlStyleExecutionProfile::SourcePredicates,
                &flag()
            ),
            Err(DomParsedHtmlStyleError::WrongExecutionProfile {
                profile: HtmlStyleExecutionProfile::SourcePredicates
            })
        ));
        assert!(matches!(
            hold.exclude_script_bookkeeping(
                &foreign.qualification,
                HtmlStyleExecutionProfile::NoExecutableJavaScript,
                &flag()
            ),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        let spent = hold.spent_work();
        assert!(matches!(
            hold.exclude_script_bookkeeping(
                &value.qualification,
                HtmlStyleExecutionProfile::NoExecutableJavaScript,
                &AtomicBool::new(true)
            ),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ));
        assert_eq!(hold.spent_work(), spent);
    }
    unchanged(&value, stamp);
}
#[test]
fn output_refusal_and_drop_keep_the_real_cursor_for_one_raised_retry() {
    let low = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work(),
        4,
    )
    .unwrap();
    let mut value = created("<!doctype html><style media=screen>later</style>", low);
    let stamp = value.dom.snapshot().stamp();
    let spent;
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, low, &flag())
            .unwrap();
        spent = hold.spent_work();
        assert!(matches!(
            hold.exclude_script_bookkeeping(
                &value.qualification,
                HtmlStyleExecutionProfile::NoExecutableJavaScript,
                &flag()
            ),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 4
                })
            ))
        ));
        assert!(hold.spent_work() > spent);
    }
    unchanged(&value, stamp);
    let raised = DomParsedHtmlAssociationLimits::new(
        low.retained_bytes(),
        low.retained_units(),
        low.work(),
        5,
    )
    .unwrap();
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, raised, &flag())
        .unwrap();
    assert!(hold.spent_work() > spent);
    assert_eq!(
        hold.exclude_script_bookkeeping(
            &value.qualification,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag()
        )
        .unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    assert!(matches!(
        hold.exclude_script_bookkeeping(
            &value.qualification,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag()
        ),
        Err(DomParsedHtmlStyleError::WrongPhase {
            phase: DomParsedHtmlStylePhase::ReadRenderBlockingPending
        })
    ));
    drop(hold);
    assert_eq!(value.root.snapshot().unwrap().revision(), &value.revision);
}
#[test]
fn original_work_frontier_refuses_without_reset_then_resumes_the_same_phase() {
    let mut value = created(
        "<!doctype html><style media=screen>later</style>",
        envelope(),
    );
    let stamp = value.dom.snapshot().stamp();
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        while hold.limits().work() - hold.spent_work() > 1 {
            hold.step(&flag()).unwrap();
        }
        assert!(matches!(
            hold.exclude_script_bookkeeping(
                &value.qualification,
                HtmlStyleExecutionProfile::NoExecutableJavaScript,
                &flag()
            ),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    ..
                })
            ))
        ));
        assert_eq!(hold.spent_work(), hold.limits().work());
    }
    unchanged(&value, stamp);
    let raised = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work() + 100,
        envelope().outputs(),
    )
    .unwrap();
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, raised, &flag())
        .unwrap();
    assert!(hold.spent_work() >= envelope().work());
    assert_eq!(
        hold.exclude_script_bookkeeping(
            &value.qualification,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag()
        )
        .unwrap(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
}
#[test]
fn excluded_reply_cannot_run_before_real_creation() {
    let (mut dom, _document, id) = create::ready("<!doctype html><style>later</style>");
    let qualification = qualifier(&dom, id);
    let stamp = dom.snapshot().stamp();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        hold.exclude_script_bookkeeping(
            &qualification,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag()
        ),
        Err(DomParsedHtmlStyleError::WrongPhase {
            phase: DomParsedHtmlStylePhase::CreateSheetPending
        })
    ));
    drop(hold);
    let view = dom.parsed_html_style_prefix(id).unwrap();
    assert_eq!(view.script_bookkeeping_exclusion(), None);
    assert_eq!(dom.snapshot().stamp(), stamp);
}
