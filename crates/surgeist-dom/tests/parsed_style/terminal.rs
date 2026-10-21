//! Explicit whole settlement uses genuine execution and affine lower authority.
use super::*;
use surgeist_cssom::CssomEditGuard;
use surgeist_html::HtmlStyleExecutionProfile;
#[test]
fn blocked_invocations_require_actual_end_and_publish_only_real_style_readiness() {
    let (mut dom, document, id, q, _) =
        recreation::blocked("<!doctype html><style>rejected</style>", envelope());
    let node = dom
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == id)
        .unwrap()
        .node_for(q.subject())
        .unwrap();
    assert!(
        matches!(dom.begin_parsed_html_style(id,envelope(),&flag()).unwrap().complete_contribution(&flag()),Err(DomParsedHtmlTerminalError::SynchronousEndRequired{contribution}) if contribution==id)
    );
    recreation::no_sheet(&mut dom, id, envelope());
    let mut observation = Some(recreation::observed(&dom, id));
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    let end = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag())
        .unwrap();
    let old = dom.snapshot();
    let (mut foreign, _) = setup();
    assert!(matches!(
        foreign.begin_parsed_html_style(id, envelope(), &flag()),
        Err(DomParsedHtmlStyleError::Admission(
            DomParsedHtmlAssociationAdmissionError::Dom(DomError::ForeignOwner)
        ))
    ));
    let done = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_contribution(&flag())
        .unwrap();
    assert_eq!(done.contribution(), id);
    assert_eq!(done.dom().changes().from(), old.stamp());
    assert_eq!(done.dom().changes().to(), dom.snapshot().stamp());
    assert_eq!(
        done.dom().changes().changes(),
        &[DomChange::ElementState(node)]
    );
    assert!(
        dom.pending_parsed_html_association(document)
            .unwrap()
            .is_none()
    );
    let current = dom.snapshot();
    let style = current.node(node).unwrap().as_style().unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::Established
    );
    assert!(style.sheet().unwrap().is_none());
    assert!(!style.is_render_blocking().unwrap());
    assert!(matches!(
        old.node(node).unwrap().as_style().unwrap().sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    assert!(
        matches!(dom.begin_parsed_html_style(id,envelope(),&flag()),Err(DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::NotPending{contribution})) if contribution==id)
    );
    assert_eq!(end.contribution(), id);
    drop(dom);
    assert_eq!(done.dom().changes().to(), current.stamp());
}
fn rendered_pair(
    text: &str,
) -> (
    rendering::Ready,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
) {
    let mut value = rendering::ready(
        &format!("<!doctype html><style media=''>{text}</style>"),
        envelope(),
    );
    assert!(matches!(
        value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap()
            .process_rendering(
                &value.q,
                DomParsedHtmlStyleNotificationHandoff::Retain,
                &flag()
            )
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    let (next, retired) = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap()
        .prepare_successor(Some(&value.root), &flag())
        .unwrap()
        .commit(&flag())
        .unwrap()
        .into_parts();
    let q = next.qualification().clone();
    drop(retired);
    drop(next);
    let mut observation = {
        let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    hold.accept_inline_policy(&mut observation, &flag())
        .unwrap();
    let mut prep = hold.prepare_create_sheet(&value.root, &flag()).unwrap();
    create::stage(&mut prep);
    let expected = if text.starts_with("@import") {
        DomParsedHtmlStyleResources::ImportsUnresolved
    } else {
        DomParsedHtmlStyleResources::NoImports
    };
    assert_eq!(prep.resources(), Some(expected));
    let (created, retired) = prep.commit(&flag()).unwrap().into_parts();
    assert_eq!(created.resources(), expected);
    let sheet = created.sheet().clone();
    drop(retired);
    drop(created);
    hold.exclude_script_bookkeeping(
        &q,
        HtmlStyleExecutionProfile::NoExecutableJavaScript,
        &flag(),
    )
    .unwrap();
    assert!(matches!(
        hold.process_rendering(&q, DomParsedHtmlStyleNotificationHandoff::Retain, &flag())
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    hold.complete_synchronous_execution(&flag()).unwrap();
    drop(hold);
    (value, q, sheet)
}
#[test]
fn retained_and_explicitly_abandoned_rendered_rights_never_become_whole_completion() {
    let (mut value, q, _) = rendered_pair("a{}");
    let before = value.dom.snapshot().stamp();
    for abandon in [false, true] {
        if abandon {
            for qualification in [&value.q, &q] {
                let mut continuation = None;
                value
                    .dom
                    .begin_parsed_html_style(value.id, envelope(), &flag())
                    .unwrap()
                    .transfer_notification(qualification, &mut continuation, &flag())
                    .unwrap();
                drop(continuation);
            }
        }
        assert!(
            matches!(value.dom.begin_parsed_html_style(value.id,envelope(),&flag()).unwrap().complete_contribution(&flag()),Err(DomParsedHtmlTerminalError::NotificationsUnsettled{remaining,..}) if remaining.get()==2)
        );
        assert_eq!(value.dom.snapshot().stamp(), before);
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
fn actual_import_classification_stays_unresolved_after_both_real_owner_removed_settlements() {
    let (mut value, q, sheet) = rendered_pair("@import 'unresolved.css';");
    let snapshot = value.root.snapshot().unwrap();
    let mut batch = Some(
        value
            .root
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .unwrap(),
    );
    assert!(batch.as_mut().unwrap().remove_sheet(sheet.id()).unwrap());
    value.root.commit(&mut batch).unwrap();
    for qualification in [&value.q, &q] {
        let mut continuation = None;
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        hold.transfer_notification(qualification, &mut continuation, &flag())
            .unwrap();
        let prep = hold
            .prepare_notification(&mut continuation, "owner", &value.root, &flag())
            .unwrap();
        assert!(prep.owner_association_removed());
        let mut published = prep.commit(&flag()).unwrap();
        assert!(matches!(
            published.take_disposition(),
            Some(DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved { .. })
        ));
    }
    assert!(
        matches!(value.dom.begin_parsed_html_style(value.id,envelope(),&flag()).unwrap().complete_contribution(&flag()),Err(DomParsedHtmlTerminalError::CriticalResourcesUnresolved{contribution}) if contribution==value.id)
    );
    assert_eq!(
        value
            .dom
            .pending_parsed_html_association(value.document)
            .unwrap(),
        Some(value.id)
    );
}
