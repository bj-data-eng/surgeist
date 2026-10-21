//! Exact original Meta progress after the genuine two-invocation Style prefix.
use super::*;
use surgeist_html_parser::{
    HtmlParseAssociationOccurrenceId, HtmlParseAssociationOccurrenceKind as K,
    HtmlParsePendingAssociationStep as S,
};
fn pair(
    source: &str,
) -> (
    Dom,
    DocumentId,
    DomParsedHtmlAssociationId,
    DomStyleCssomRoot,
    Vec<HtmlParseAssociationOccurrenceId>,
) {
    let report = parse(source);
    let mut cursor = report.pending_associations();
    let mut ids = Vec::new();
    loop {
        match cursor.advance() {
            S::Occurrence(r) if r.kind() == K::ChangePreferred => ids.push(r.id()),
            S::Complete => break,
            _ => (),
        }
    }
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, report, env);
    decision::ready(&mut dom, id, envelope());
    let binding_q = qualifier(&dom, id);
    // The actual initial-empty policy permits each inline text. No external
    // blocking decision is fabricated to reach the no-sheet branch.
    let mut input = Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .unwrap(),
        true,
    ));
    let cap = create::root(document);
    {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        h.accept_inline_policy(&mut input, &flag()).unwrap();
        let mut prep = h.prepare_create_sheet(&cap, &flag()).unwrap();
        create::stage(&mut prep);
        let (_, dead) = prep.commit(&flag()).unwrap().into_parts();
        drop(dead);
        h.exclude_script_bookkeeping(
            &binding_q,
            surgeist_html::HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag(),
        )
        .unwrap();
        drop(
            h.process_rendering(
                &binding_q,
                surgeist_dom::DomParsedHtmlStyleNotificationHandoff::Retain,
                &flag(),
            )
            .unwrap(),
        );
        let (_, dead) = h
            .prepare_successor(Some(&cap), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(dead);
    }
    let q = qualifier(&dom, id);
    let mut input = Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .unwrap(),
        true,
    ));
    {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        h.accept_inline_policy(&mut input, &flag()).unwrap();
        let mut prep = h.prepare_create_sheet(&cap, &flag()).unwrap();
        create::stage(&mut prep);
        let (_, dead) = prep.commit(&flag()).unwrap().into_parts();
        drop(dead);
        h.exclude_script_bookkeeping(
            &q,
            surgeist_html::HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag(),
        )
        .unwrap();
        drop(
            h.process_rendering(
                &q,
                surgeist_dom::DomParsedHtmlStyleNotificationHandoff::Retain,
                &flag(),
            )
            .unwrap(),
        );
    }
    (dom, document, id, cap, ids)
}
#[test]
fn meta_occurrence_exact_order_noop_commit_and_separate_synchronous_end() {
    let (mut dom, document, id, cap, ids) = pair(
        "<!doctype html><style title=A media=''>a{color:red}</style><meta http-equiv=default-style content='B𝄞'><meta http-equiv=default-style content=A><meta http-equiv=default-style content=A>",
    );
    let stamp = dom.snapshot().stamp();
    let q = qualifier(&dom, id);
    {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            h.meta_occurrence_document(&ids[1], &flag()),
            Err(DomParsedHtmlMetaOccurrenceError::WrongOccurrence)
        ));
        assert!(h.complete_synchronous_execution(&flag()).is_err());
    }
    for (index, expected) in ["B𝄞", "A", "A"].iter().enumerate() {
        let before = cap.snapshot().unwrap().revision().clone();
        let completion = {
            let mut h = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            assert_eq!(
                h.meta_occurrence_document(&ids[index], &flag()).unwrap(),
                document
            );
            let prep = h
                .prepare_meta_occurrence(&ids[index], &cap, &flag())
                .unwrap();
            assert_eq!(
                prep.occurrence().content().code_units(),
                expected.encode_utf16().collect::<Vec<_>>()
            );
            let published = prep
                .commit(
                    DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
                    &flag(),
                )
                .unwrap();
            assert!(matches!(cap.snapshot(), Err(DomStyleCssomRootError::Busy)));
            let (done, dead) = published.into_parts();
            drop(dead);
            done
        };
        if index == 0 {
            let hold = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            assert!(
                matches!(
                    hold.meta_occurrence_document(q.occurrence(), &flag()),
                    Err(DomParsedHtmlMetaOccurrenceError::WrongOccurrence)
                ),
                "a completed Style is not a consumed Meta occurrence"
            );
        }
        assert_eq!(completion.occurrence().id(), &ids[index]);
        assert_eq!(completion.stamp(), stamp);
        assert!(
            matches!(completion.cssom().publication(),surgeist_cssom::CssomPublication::Unchanged{revision} if revision==&before)
        );
        assert_eq!(qualifier(&dom, id), q);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert!(dom.parsed_html_synchronous_end(id).unwrap().is_none());
    }
    let mut h = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        h.prepare_meta_occurrence(&ids[0], &cap, &flag()),
        Err(DomParsedHtmlMetaOccurrenceError::AlreadyConsumed)
    ));
    drop(h.complete_synchronous_execution(&flag()).unwrap());
    drop(h);
    assert!(dom.parsed_html_synchronous_end(id).unwrap().is_some());
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        id
    );
}
#[test]
fn meta_occurrence_cancel_drop_and_foreign_cap_preserve_exact_next_row() {
    let (mut dom, document, id, cap, ids) = pair(
        "<!doctype html><style title=A media=''>a{color:red}</style><meta http-equiv=default-style content=B>",
    );
    let q = qualifier(&dom, id);
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let foreign = create::root(document);
    let mut h = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        h.prepare_meta_occurrence(&ids[0], &foreign, &flag()),
        Err(DomParsedHtmlMetaOccurrenceError::MismatchedOwner { .. })
    ));
    let prep = h.prepare_meta_occurrence(&ids[0], &cap, &flag()).unwrap();
    assert!(matches!(
        prep.commit(
            DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
            &AtomicBool::new(true)
        ),
        Err(DomParsedHtmlMetaOccurrenceError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    let prep = h.prepare_meta_occurrence(&ids[0], &cap, &flag()).unwrap();
    drop(prep);
    let spent = h.spent_work();
    drop(h);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(qualifier(&dom, id), q);
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    let mut h = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(h.spent_work() >= spent);
    let (done, dead) = h
        .prepare_meta_occurrence(&ids[0], &cap, &flag())
        .unwrap()
        .commit(
            DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
            &flag(),
        )
        .unwrap()
        .into_parts();
    drop(dead);
    assert_eq!(done.occurrence().id(), &ids[0]);
}
