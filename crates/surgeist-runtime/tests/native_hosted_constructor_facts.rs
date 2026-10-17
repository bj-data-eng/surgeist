//! Actual no-opener hosted construction retains the HTML877d opener defaults
//! and source-initial Base/UTF-8 facts, independently of later lifecycle state.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_html::HtmlOpenerPolicyValue;
use surgeist_html_semantics::HtmlAboutBaseUrl;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn session(policy: DomSessionLimits) -> DomSession {
    let dom =
        Dom::new(DomLimits::new(256, 200_000, 1024, 1024, 1024, 32, 4_000_000).unwrap()).unwrap();
    DomSession::new(dom, policy).unwrap()
}
fn create(session: &mut DomSession) -> (HostedRuntime, DomHostedTopLevel) {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    // Initial no-opener creation reaches no URL parsing service.
    let top = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
    (hosted, top)
}

#[test]
fn actual_initial_policy_and_url_encoding_identities_survive_reads_and_destruction() {
    let mut session = session(limits());
    let (mut hosted, top) = create(&mut session);
    let initial = session.snapshot();
    let document = initial.document(top.document()).unwrap();
    let opener = document
        .opener_policy()
        .expect("actual hosted opener-policy record");
    assert_eq!(opener.value(), HtmlOpenerPolicyValue::UnsafeNone);
    assert_eq!(opener.reporting_endpoint(), None);
    assert_eq!(
        opener.report_only_value(),
        HtmlOpenerPolicyValue::UnsafeNone
    );
    assert_eq!(opener.report_only_reporting_endpoint(), None);
    let facts = document
        .base_document_facts()
        .expect("actual constructor-qualified facts");
    let url = facts.document_url().unwrap();
    let encoding = facts.encoding().unwrap();
    assert_eq!(url.serialization(), &DocumentString::from("about:blank"));
    assert!(url.matches_about_blank());
    assert!(matches!(
        facts.about_base_url(),
        HtmlAboutBaseUrl::KnownNull
    ));
    assert!(!facts.iframe_srcdoc());
    assert_eq!(
        document.metadata().encoding(),
        &DocumentString::from("UTF-8")
    );
    assert_eq!(document.document_base_record().unwrap(), url);
    let settings = {
        let dom = session.dom();
        let settings = hosted.settings(&dom, top.global(), &cancel()).unwrap();
        assert_eq!(settings.creation_url(), url);
        assert_eq!(settings.top_level_creation_url(), url);
        assert_eq!(
            settings
                .document()
                .base_document_facts()
                .unwrap()
                .encoding(),
            Some(encoding)
        );
        let repeated = hosted.settings(&dom, top.global(), &cancel()).unwrap();
        assert_eq!(repeated.creation_url(), settings.creation_url());
        assert_eq!(
            repeated.top_level_creation_url(),
            settings.top_level_creation_url()
        );
        settings
    };
    {
        let mut dom = session.dom();
        let root = initial.node(top.document().node()).unwrap().children()[0];
        let mut batch = DomBatch::new();
        batch.set_attribute(root, "data-current", "changed revision");
        dom.apply(batch, &cancel()).unwrap();
        let current = hosted.settings(&dom, top.global(), &cancel()).unwrap();
        assert_eq!(current.creation_url(), url);
        assert_eq!(current.top_level_creation_url(), url);
        assert_eq!(
            current.document().base_document_facts().unwrap().encoding(),
            Some(encoding)
        );
        assert_eq!(current.document().document_base_record().unwrap(), url);
        let queued = hosted
            .destroy_top_level(&mut dom, top.navigable(), &cancel())
            .unwrap();
        assert_eq!(queued.host(), top.host());
        assert_eq!(queued.navigable(), top.navigable());
        assert_eq!(queued.traversable(), top.navigable());
        assert!(
            matches!(pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        );
        assert!(hosted.settings(&dom, top.global(), &cancel()).is_err());
    }
    assert!(
        initial
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        document.base_document_facts().unwrap().document_url(),
        Some(url)
    );
    assert_eq!(
        document.base_document_facts().unwrap().encoding(),
        Some(encoding)
    );
    assert_eq!(document.opener_policy().unwrap(), opener);
    assert_eq!(settings.creation_url(), url);
    assert_eq!(settings.top_level_creation_url(), url);
    assert_eq!(
        settings
            .document()
            .base_document_facts()
            .unwrap()
            .encoding(),
        Some(encoding)
    );
}

#[test]
fn fresh_closed_document_creators_do_not_inherit_hosted_policy_or_base_qualification() {
    let mut session = session(limits());
    let (_, top) = create(&mut session);
    let (xml, created_xml, html, template) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let xml = batch.create_document();
        let created_xml =
            batch.create_xml_document(top.document(), Namespace::none(), "root", None);
        let html = batch.create_html_document(top.document(), None);
        let template = batch.create_element_ns(top.document(), Namespace::html(), "template");
        let commit = dom.apply(batch, &cancel()).unwrap();
        (
            commit.document(&xml).unwrap(),
            commit.document(&created_xml).unwrap(),
            commit.document(&html).unwrap(),
            commit.created(&template).unwrap().unwrap(),
        )
    };
    let snapshot = session.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    for id in [xml, created_xml, html, inert] {
        let document = snapshot.document(id).unwrap();
        assert_eq!(document.opener_policy(), None);
        assert!(document.base_document_facts().is_none());
        assert!(
            !document
                .metadata()
                .document_environment()
                .has_ever_had_navigable()
        );
    }
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .opener_policy()
            .is_some()
    );
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .base_document_facts()
            .is_some()
    );
}

#[test]
fn entry_capacity_refusal_and_cancelled_retry_retain_original_constructor_facts() {
    let mut policy = limits();
    policy.hosted.canonical = DomHostedLimits::new(
        DomHostedRecordLimits::new(32, 32, 32, 0, 32, 32, 32),
        16_000_000,
        16_000_000,
        1_000_000,
    );
    let mut session = session(policy);
    let mut hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        assert!(matches!(
            hosted.create_top_level(&mut dom, host, &cancel()),
            Err(DomHostedRuntimeError::Dom(DomHostedError::Limit {
                resource: DomHostedResource::Entries,
                limit: 0,
            }))
        ));
    }
    // Entry issuance follows actual Policy, Execution, make-active and loading.
    // Observe the already published original Document, never a fabricated ID.
    let parked = session.snapshot();
    let operation = parked
        .incomplete_hosted_operation()
        .expect("original entry phase retained");
    let documents = parked
        .nodes()
        .filter_map(|id| parked.node(id).unwrap().as_document().map(|d| d.id()))
        .collect::<Vec<_>>();
    assert_eq!(documents.len(), 1);
    let document_id = documents[0];
    let document = parked.document(document_id).unwrap();
    let facts = document.base_document_facts().unwrap();
    let url = facts.document_url().unwrap().clone();
    let encoding = facts.encoding().unwrap().clone();
    let revision = facts.revision().clone();
    let opener = document.opener_policy().unwrap().clone();
    assert_eq!(url.serialization(), &DocumentString::from("about:blank"));
    assert!(matches!(
        facts.about_base_url(),
        HtmlAboutBaseUrl::KnownNull
    ));
    assert!(!facts.iframe_srcdoc());
    {
        let mut dom = session.dom();
        assert!(matches!(
            hosted.resume_top_level_creation(&mut dom, &AtomicBool::new(true)),
            Err(DomHostedRuntimeError::Cancelled)
        ));
        assert_eq!(dom.snapshot().stamp(), parked.stamp());
        assert_eq!(
            dom.snapshot().incomplete_hosted_operation(),
            Some(operation)
        );
        hosted
            .set_limits(&mut dom, limits().hosted, &cancel())
            .unwrap();
    }
    let top = {
        let mut dom = session.dom();
        hosted
            .resume_top_level_creation(&mut dom, &cancel())
            .unwrap()
    };
    assert_eq!(top.document(), document_id);
    let current = session.snapshot();
    assert_eq!(current.incomplete_hosted_operation(), None);
    let current_document = current.document(top.document()).unwrap();
    let current_facts = current_document.base_document_facts().unwrap();
    assert_eq!(current_facts.document_url(), Some(&url));
    assert_eq!(current_facts.encoding(), Some(&encoding));
    assert_eq!(current_facts.revision(), &revision);
    assert_eq!(current_document.document_base_record().unwrap(), &url);
    assert_eq!(current_document.opener_policy(), Some(&opener));
    assert_eq!(
        document.base_document_facts().unwrap().document_url(),
        Some(&url)
    );
    assert_eq!(
        document.base_document_facts().unwrap().encoding(),
        Some(&encoding)
    );
    assert_eq!(document.opener_policy(), Some(&opener));
    let dom = session.dom();
    let settings = hosted.settings(&dom, top.global(), &cancel()).unwrap();
    let again = hosted.settings(&dom, top.global(), &cancel()).unwrap();
    for observed in [&settings, &again] {
        assert_eq!(observed.document().id(), document_id);
        assert_eq!(observed.creation_url(), &url);
        assert_eq!(observed.top_level_creation_url(), &url);
        assert_eq!(
            observed
                .document()
                .base_document_facts()
                .unwrap()
                .encoding(),
            Some(&encoding)
        );
    }
}

fn pump_destruction(
    hosted: &mut HostedRuntime,
    dom: &mut DomEventDom<'_>,
    queued: DomHostedDestructionQueued,
) -> Result<DomHostedTaskProgress, DomHostedRuntimeError> {
    let mut last = DomHostedTaskProgress::Idle;
    for _ in 0..256 {
        last = hosted.run_next(dom, queued.traversable(), &cancel())?;
        if matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            break;
        }
    }
    Ok(last)
}
