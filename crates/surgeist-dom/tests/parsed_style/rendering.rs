//! Original rendering observes the insertion frontier, not final-tree predicates.
use super::*;
use surgeist_cssom::{CssomRevision, CssomSheetId};
use surgeist_html::HtmlStyleExecutionProfile;

pub(super) struct Ready {
    pub(super) dom: Dom,
    pub(super) document: DocumentId,
    pub(super) id: DomParsedHtmlAssociationId,
    pub(super) q: DomParsedHtmlStyleInitialPolicyRequest,
    pub(super) element: NodeId,
    pub(super) root: DomStyleCssomRoot,
    pub(super) sheet: CssomSheetId,
    pub(super) revision: CssomRevision,
    pub(super) association: DomStyleSheet,
}
pub(super) fn ready(source: &str, limits: DomParsedHtmlAssociationLimits) -> Ready {
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, parse(source), env);
    decision::ready(&mut dom, id, limits);
    let q = qualifier(&dom, id);
    let mut observation = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let root = create::root(document);
    let (element, sheet, revision, association) = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        hold.accept_inline_policy(&mut observation, &flag())
            .unwrap();
        let mut prep = hold.prepare_create_sheet(&root, &flag()).unwrap();
        let element = prep.request().element();
        create::stage(&mut prep);
        let (completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
        let values = (
            element,
            completion.sheet().id().clone(),
            completion.sheet().revision().clone(),
            completion.sheet().clone(),
        );
        drop(retirement);
        values
    };
    dom.begin_parsed_html_style(id, limits, &flag())
        .unwrap()
        .exclude_script_bookkeeping(
            &q,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag(),
        )
        .unwrap();
    Ready {
        dom,
        document,
        id,
        q,
        element,
        root,
        sheet,
        revision,
        association,
    }
}
fn unchanged_sheet_and_pending(value: &Ready) {
    let sheet = value.root.snapshot().unwrap();
    assert_eq!(sheet.revision(), &value.revision);
    assert_eq!(sheet.sheets(), std::slice::from_ref(&value.sheet));
    assert_eq!(
        value
            .dom
            .pending_parsed_html_association(value.document)
            .unwrap(),
        Some(value.id)
    );
    assert!(matches!(
        value
            .dom
            .snapshot()
            .node(value.element)
            .unwrap()
            .as_style()
            .unwrap()
            .is_render_blocking(),
        Err(DomError::StyleProcessingUnavailable)
    ));
}
#[test]
fn absent_empty_and_ascii_whitespace_media_use_original_head_body_gate_and_real_changes() {
    for attr in ["", "media=''", "media=' \t\r\n\u{c}'"] {
        for in_body in [false, true] {
            let source = if in_body {
                format!("<!doctype html><body><style {attr}>later</style>")
            } else {
                format!("<!doctype html><head><style {attr}>later</style></head><body>final body")
            };
            let mut value = ready(&source, envelope());
            let final_tree = value.dom.snapshot();
            assert!(final_tree.nodes().any(|id| {
                let node = final_tree.node(id).unwrap();
                node.owner_document() == Some(value.document)
                    && node.as_element().is_some_and(|element| {
                        element.name().local_name() == &DocumentString::from("body")
                    })
            }));
            drop(final_tree);
            let before = value.dom.snapshot().stamp();
            let progress = value
                .dom
                .begin_parsed_html_style(value.id, envelope(), &flag())
                .unwrap()
                .process_rendering(&value.q, &flag())
                .unwrap();
            let DomParsedHtmlStyleRenderingProgress::Complete(done) = progress else {
                panic!("unconditional media must finish this invocation")
            };
            assert_eq!(done.qualification(), &value.q);
            assert_eq!(done.element(), value.element);
            assert_eq!(done.phase(), DomParsedHtmlStylePhase::RenderingComplete);
            assert_eq!(done.render_blocking_member(), !in_body);
            assert_eq!(done.dom().changes().from(), before);
            assert_eq!(done.dom().changes().to(), value.dom.snapshot().stamp());
            assert_eq!(done.dom().changes().changes().iter().any(|change|matches!(change,DomChange::DocumentRenderBlocking(id) if *id==value.document)),!in_body);
            assert_eq!(done.dom().changes().changes().is_empty(), in_body);
            let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
            assert_eq!(view.phase(), DomParsedHtmlStylePhase::RenderingComplete);
            assert_eq!(
                view.script_bookkeeping_exclusion(),
                Some(HtmlStyleExecutionProfile::NoExecutableJavaScript)
            );
            drop(view);
            unchanged_sheet_and_pending(&value);
            let mut hold = value
                .dom
                .begin_parsed_html_style(value.id, envelope(), &flag())
                .unwrap();
            assert!(matches!(
                hold.process_rendering(&value.q, &flag()),
                Err(DomParsedHtmlStyleError::WrongPhase {
                    phase: DomParsedHtmlStylePhase::RenderingComplete
                })
            ));
            assert_eq!(
                hold.advance(&flag()).unwrap(),
                DomParsedHtmlStylePhase::RenderingComplete
            );
        }
    }
}
#[test]
fn nontrivial_media_keeps_original_cursor_pending_without_guessing_a_match() {
    for media in ["screen", "all", "\u{a0}"] {
        let mut value = ready(
            &format!("<!doctype html><style media='{media}'>later</style>"),
            envelope(),
        );
        let before = value.dom.snapshot().stamp();
        for _ in 0..2 {
            assert!(matches!(
                value
                    .dom
                    .begin_parsed_html_style(value.id, envelope(), &flag())
                    .unwrap()
                    .process_rendering(&value.q, &flag())
                    .unwrap(),
                DomParsedHtmlStyleRenderingProgress::Pending(
                    DomParsedHtmlStyleRenderingRequirement::MediaEnvironment
                )
            ));
            assert_eq!(value.dom.snapshot().stamp(), before);
            assert_eq!(
                value
                    .dom
                    .parsed_html_style_prefix(value.id)
                    .unwrap()
                    .phase(),
                DomParsedHtmlStylePhase::ReadRenderBlockingPending
            );
        }
        unchanged_sheet_and_pending(&value);
    }
}
#[test]
fn cancellation_wrong_qualification_and_final_output_refusal_preserve_same_original_retry() {
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 6).unwrap();
    let mut value = ready("<!doctype html><style>later</style>", limits);
    let other = ready("<!doctype html><style>other</style>", envelope());
    let stamp = value.dom.snapshot().stamp();
    let spent = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, limits, &flag())
            .unwrap();
        assert!(matches!(
            hold.process_rendering(&other.q, &flag()),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        assert!(matches!(
            hold.process_rendering(&value.q, &AtomicBool::new(true)),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ));
        assert!(matches!(
            hold.process_rendering(&value.q, &flag()),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 6
                })
            ))
        ));
        hold.spent_work()
    };
    assert_eq!(value.dom.snapshot().stamp(), stamp);
    assert_eq!(
        value
            .dom
            .parsed_html_style_prefix(value.id)
            .unwrap()
            .phase(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    unchanged_sheet_and_pending(&value);
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert!(hold.spent_work() > spent);
    assert!(matches!(
        hold.process_rendering(&value.q, &flag()).unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
}

#[test]
fn actual_pending_adoption_guard_preserves_live_destination_and_original_qualification() {
    let mut value = ready("<!doctype html><style>later</style>", envelope());
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let committed = value.dom.apply(batch, &flag()).unwrap();
    let other = committed.document(&other).unwrap();
    let before = value.dom.snapshot().stamp();
    let mut adopt = DomBatch::new();
    adopt.adopt(value.element, other.node());
    assert!(
        matches!(value.dom.apply(adopt,&flag()),Err(DomError::ParsedAssociationPending{document,pending}) if document==value.document && pending==value.id)
    );
    assert_eq!(value.dom.snapshot().stamp(), before);
    let result = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap()
        .process_rendering(&value.q, &flag())
        .unwrap();
    let DomParsedHtmlStyleRenderingProgress::Complete(done) = result else {
        panic!("original destination remains eligible")
    };
    assert_eq!(done.qualification().document(), value.document);
    assert!(done.render_blocking_member());
    unchanged_sheet_and_pending(&value);
}
