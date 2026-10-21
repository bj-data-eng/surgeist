//! Genuine reports and joined first-sheet publication, not final-text replay.
use super::*;
use surgeist_css::{CssParserContext, CssUtf16InputLimits};
use surgeist_cssom::*;

fn ready(source: &str) -> (Dom, DocumentId, DomParsedHtmlAssociationId) {
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, parse(source), env);
    decision::ready(&mut dom, id, envelope());
    let mut observation = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    (dom, document, id)
}
fn root(document: DocumentId) -> DomStyleCssomRoot {
    DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    )
}
fn stage(prep: &mut DomParsedHtmlStyleSheetPreparation<'_, '_, '_>) {
    let parser = CssParserContext::default();
    let sheet = parser
        .parse_sheet_utf16_with_limits(
            prep.request().text().shared_code_units().clone(),
            CssUtf16InputLimits::default(),
            || prep.step(&flag()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            prep.request()
                .media()
                .map_or_else(Arc::default, |s| s.shared_code_units().clone()),
            CssUtf16InputLimits::default(),
            || prep.step(&flag()),
        )
        .unwrap();
    // Concrete caller-owned labels/holders are declared before formation.
    prep.admit_runtime(
        DomParsedHtmlAssociationRuntimeUsage::new(1024, 0, 0).unwrap(),
        &flag(),
    )
    .unwrap();
    let mut input = Some(DomParsedHtmlStyleSheetInput::new(
        sheet,
        media,
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "original".into(),
            revision: 1,
        },
        "owner".into(),
        false,
    ));
    prep.stage_sheet(&mut input, &flag()).unwrap();
    assert!(input.is_none());
}
#[test]
fn first_empty_sheet_preserves_original_attributes_and_pending_lifecycle() {
    let (mut dom, document, id) =
        ready("<!doctype html><style title='A&amp;B' media=screen nonce=n>p{color:red}</style>");
    let cap = root(document);
    let before = dom.snapshot().stamp();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert_eq!(hold.create_sheet_document(&flag()).unwrap(), document);
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    assert!(prep.request().text().is_empty());
    assert_eq!(
        prep.request().title().unwrap().code_units(),
        DocumentString::from("A&B").code_units()
    );
    assert_eq!(
        prep.request().media().unwrap().code_units(),
        DocumentString::from("screen").code_units()
    );
    assert_eq!(prep.request().parser_document(), Some(document));
    let element = prep.request().element();
    let creation = prep.request().qualification().clone();
    stage(&mut prep);
    let published = prep.commit(&flag()).unwrap();
    assert!(matches!(cap.snapshot(), Err(DomStyleCssomRootError::Busy)));
    assert_eq!(published.completion().contribution(), id);
    assert_eq!(
        published.completion().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    let (completion, retirement) = published.into_parts();
    drop(hold);
    let actual = cap.snapshot().unwrap();
    assert_eq!(actual.sheets(), &[completion.sheet().id().clone()]);
    assert_eq!(completion.sheet().revision(), actual.revision());
    assert!(
        actual
            .sheet_rules(completion.sheet().id())
            .unwrap()
            .is_empty()
    );
    assert_ne!(dom.snapshot().stamp(), before);
    assert_eq!(completion.stamp(), dom.snapshot().stamp());
    assert_eq!(completion.sheet().association().element(), element);
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .initial_policy_request(),
        Some(&creation)
    );
    let snapshot = dom.snapshot();
    let style = snapshot.node(element).unwrap().as_style().unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    assert!(matches!(
        style.sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    assert_eq!(
        dom.pending_parsed_html_association(document).unwrap(),
        Some(id)
    );
    drop(retirement);
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        hold.prepare_create_sheet(&cap, &flag()),
        Err(DomParsedHtmlStyleSheetError::Style(
            DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::ReadScriptBlockingPending
            }
        ))
    ));
}
#[test]
fn dropping_or_canceling_prepared_creation_publishes_nothing_and_retry_uses_same_request() {
    let (mut dom, document, id) = ready("<!doctype html><style title=A>later</style>");
    let cap = root(document);
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    for cancel in [false, true] {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        stage(&mut prep);
        if cancel {
            assert!(matches!(
                prep.commit(&AtomicBool::new(true)),
                Err(DomParsedHtmlStyleSheetError::Style(
                    DomParsedHtmlStyleError::Admission(
                        DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
                    )
                ))
            ));
        } else {
            drop(prep);
        }
        drop(hold);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(cap.snapshot().unwrap().revision(), &revision);
        assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
        assert_eq!(
            dom.parsed_html_style_prefix(id).unwrap().phase(),
            DomParsedHtmlStylePhase::CreateSheetPending
        );
    }
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    stage(&mut prep);
    let (completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
    assert_eq!(completion.contribution(), id);
    drop(retirement);
}
#[test]
fn genuine_recovered_media_diagnostics_keep_the_original_source_after_retirement() {
    let (mut dom, document, id) = ready("<!doctype html><style media='screen and ('>later</style>");
    let cap = root(document);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    stage(&mut prep);
    let (completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
    drop(retirement);
    drop(hold);
    drop(dom);
    drop(cap);
    assert!(!completion.media_diagnostics().is_empty());
    for diagnostic in completion.media_diagnostics() {
        assert_eq!(
            diagnostic
                .source()
                .unwrap()
                .original_utf16()
                .unwrap()
                .as_slice(),
            DocumentString::from("screen and (").code_units()
        );
    }
}
#[test]
fn foreign_capability_and_mismatched_original_input_are_refused_before_consumption() {
    let (mut dom, document, id) = ready("<!doctype html><style media=screen>later</style>");
    let (other, otherdoc, _) = ready("<!doctype html><style>different</style>");
    let wrong = root(otherdoc);
    let cap = root(document);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(hold.prepare_create_sheet(&wrong,&flag()),Err(DomParsedHtmlStyleSheetError::MismatchedOwner {root}) if root==document.node())
    );
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    let parser = CssParserContext::default();
    let report = parser
        .parse_sheet_utf16_with_limits(Arc::new(vec![b'x' as u16]), Default::default(), || {
            prep.step(&flag())
        })
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            Arc::new("screen".encode_utf16().collect()),
            Default::default(),
            || prep.step(&flag()),
        )
        .unwrap();
    let mut input = Some(DomParsedHtmlStyleSheetInput::new(
        report,
        media,
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "wrong".into(),
            revision: 1,
        },
        "owner".into(),
        false,
    ));
    assert!(matches!(
        prep.stage_sheet(&mut input, &flag()),
        Err(DomParsedHtmlStyleSheetError::Style(
            DomParsedHtmlStyleError::InvalidSheetInput
        ))
    ));
    assert!(input.is_some());
    drop(other);
}
