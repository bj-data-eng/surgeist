//! Actual inline Source return boundaries through the durable Runtime adapter.
use super::*;
use crate::{DomSession, DomSessionLimits, DomStyleRuntimeLimits};
use surgeist_dom::{
    Dom, DomBatch, DomHostedLimits, DomHostedRecordLimits, DomInlineStyleInputs,
    DomInlineStyleLimits, DomInlineStyleProfile, DomInlineStyleRequestLimits,
    DomInlineStyleSourceContext, DomLimits, Namespace,
};

fn session() -> DomSession {
    let dom = Dom::new(
        DomLimits::new(
            32,
            4096,
            64,
            0,
            0,
            8,
            100_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    DomSession::new(
        dom,
        DomSessionLimits {
            source_events: crate::dom_dialog::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 8,
                reports: 8,
                storage_bytes: 1_000_000,
                work: 100_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: crate::DomElementTaskRuntimeLimits {
                tasks: 8,
                storage_bytes: 1_000_000,
                work: 100_000,
            },
            hosted: crate::DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    1_000_000,
                    1_000_000,
                    100_000,
                ),
                tasks: 8,
                storage_bytes: 1_000_000,
                scratch_bytes: 1_000_000,
                work: 100_000,
            },
        },
    )
    .unwrap()
}
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 16_000_000, 128, 1024, 1024, 4)
}
fn requests() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: surgeist_css::CssUtf16InputLimits::default(),
        declaration: surgeist_cssom::CssomDeclarationRequestLimits::default(),
    }
}
fn bind(
    session: &mut DomSession,
    element: surgeist_dom::NodeId,
    document: surgeist_dom::DocumentId,
) -> (
    surgeist_cssom::CssStyleProperties,
    DomInlineStyleSourceContext,
) {
    let cancel = AtomicBool::new(false);
    session
        .dom()
        .raw_mut()
        .configure_inline_styles(
            DomInlineStyleLimits::new(
                surgeist_cssom::CssomLimits::default(),
                8,
                64_000_000,
                8_000_000,
                16_000_000,
            )
            .unwrap(),
            &cancel,
        )
        .unwrap();
    let version = surgeist_cssom::CssomInputVersion {
        role: surgeist_cssom::CssomInputRole::Support,
        identity: "runtime-source".into(),
        revision: 1,
    };
    let parser = surgeist_css::CssParserContext::default();
    let input = DomInlineStyleInputs::new(
        session.snapshot().document(document).unwrap().stamp(),
        parser,
        version.clone(),
        surgeist_cssom::CssomDeclarationSupport::default(),
    )
    .unwrap();
    let properties = session
        .dom()
        .raw_mut()
        .inline_style(
            element,
            &input,
            requests(),
            |_, _| panic!("empty initial text"),
            &cancel,
        )
        .unwrap();
    let context = DomInlineStyleSourceContext::new(
        session.snapshot().stamp(),
        vec![
            DomInlineStyleProfile::new(
                element,
                document,
                parser,
                version,
                surgeist_cssom::CssomDeclarationSupport::default(),
            )
            .unwrap(),
        ],
    )
    .unwrap();
    (properties, context)
}
#[test]
fn inline_setter_returns_original_complete_with_receipt_and_releases_source_owner() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::html(), "div");
    let made = session.dom().apply(batch, &cancel).unwrap();
    let document = made.document(&document).unwrap();
    let element = made.created(&element).unwrap().unwrap();
    let (properties, context) = bind(&mut session, element, document);
    let result = session
        .source_execution()
        .apply_with_inline_styles(
            &mut session.dom(),
            DomSourceInlineStyleEdit::new(
                DomSourceEdit::set_attribute(element, "style", ";"),
                context,
                requests(),
            ),
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| panic!("no Style service"),
                url: None,
            },
            |_, _| panic!("no declaration candidates"),
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::InlineSource {
        source: DomSourceProgress::Complete(done),
        publication,
    } = result
    else {
        panic!("real completion and paired publication")
    };
    assert_eq!(done.commit().changes().to(), publication.dom().to());
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"style".into()),
        Some(&";".into())
    );
    assert_eq!(
        session
            .dom()
            .raw_mut()
            .capture_inline_style(element)
            .unwrap()
            .unwrap()
            .properties()
            .handle(),
        &properties
    );
}
#[test]
fn inline_request_boundary_returns_before_policy_and_resume_services_same_requests() {
    let cancel = AtomicBool::new(false);
    let mut session = session();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let container = batch.create_element_ns(&document, Namespace::html(), "div");
    let first = batch.create_element_ns(&document, Namespace::html(), "style");
    let second = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.insert_before(&first, &container, None);
    batch.insert_before(&second, &container, None);
    let made = session.dom().apply(batch, &cancel).unwrap();
    let document = made.document(&document).unwrap();
    let container = made.created(&container).unwrap().unwrap();
    let first = made.created(&first).unwrap().unwrap();
    let (_, context) = bind(&mut session, container, document);
    let mut calls = 0;
    let mut policy = |source: &DomStyleSource| {
        calls += 1;
        Some(DomStyleInlineCspObservation::evaluated(
            source,
            std::sync::Arc::new(vec!["style-src 'none'".into()]),
            1,
            false,
        ))
    };
    let result = session
        .source_execution()
        .apply_with_inline_styles(
            &mut session.dom(),
            DomSourceInlineStyleEdit::new(
                DomSourceEdit::insert_before(container, document.node(), None),
                context,
                requests(),
            ),
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            |_, _| panic!("context change never reparses"),
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::InlineSource {
        source: DomSourceProgress::Request(DomSourceRequest::Style(request)),
        publication,
    } = result
    else {
        panic!("unserviced original request and receipt")
    };
    assert_eq!(request.source().element(), first);
    let operation = request.source().operation();
    assert_eq!(publication.dom().to(), session.snapshot().stamp());
    assert_eq!(calls, 0);
    let mut policy = |source: &DomStyleSource| {
        calls += 1;
        Some(DomStyleInlineCspObservation::evaluated(
            source,
            std::sync::Arc::new(vec!["style-src 'none'".into()]),
            1,
            false,
        ))
    };
    let result = session
        .source_execution()
        .resume_with_inline_styles(
            &mut session.dom(),
            operation,
            limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            |_, _| panic!("tail never reparses"),
            &cancel,
        )
        .unwrap();
    assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    assert_eq!(calls, 2);
    assert!(session.dom().pending_source_edit().is_none());
}
