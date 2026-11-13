use super::*;
use crate::store::source::operation::accounting_tests::commit_actual_empty_sheet;
use crate::{
    DomNodeRetirementLimits, DomShadowRootInit, DomSourceEdit, DomSourceLimits, DomSourceOutput,
    DomSourceProgress, DomSourceRequest, DomStyleAction, DomStyleInlineCspObservation,
    HtmlShadowRootMode,
};
use std::sync::atomic::Ordering;
use surgeist_cssom::CssomEditGuard;
use surgeist_html::{elements::style::HtmlStyleUpdateAction, values::Namespace};
use surgeist_html_parser::HtmlFragmentParseRequest;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};

fn parsed_open_root(
    dom: &mut Dom,
    cancel: &AtomicBool,
) -> (DocumentId, NodeId, NodeId, [NodeId; 2]) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&host, &document, None);
    let commit = dom.apply(batch, cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let capture = dom
        .snapshot()
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(256, 1024, 1024, 65_536, 256, 0, 0, 16_777_216),
            crate::DomHtmlCaptureLimits::new(256, 1_000_000),
            cancel,
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let parsed = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template shadowrootmode=open><p>first</p><div>last</div></template>",
            ),
            context,
            parser_limits(),
            surgeist_html::parsing::HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let installed = dom
        .install_parsed_html_fragment(
            parsed,
            &capture,
            Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
            cancel,
        )
        .unwrap();
    let root = installed
        .snapshot()
        .node(host)
        .unwrap()
        .hosted_shadow_root()
        .unwrap();
    let original = installed.snapshot().shadow_root(root).unwrap();
    let [first, last] = *original.node().children() else {
        panic!("expected both originally parsed children");
    };
    assert!(original.state().declarative());

    (document, host, root, [first, last])
}

#[test]
fn declarative_shadow_reuse_resumes_after_actual_sheet_removal_refusal() {
    let cancel = AtomicBool::new(false);
    let limits = DomSourceLimits::new(16_000_000, 200_000, 10_000_000, 64, 64, 64, 2);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65_536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let (document, host, root, [first, last]) = parsed_open_root(&mut dom, &cancel);

    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let style = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let cssom = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, root, Some(last)),
                limits,
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("connected Style must ask for its policy");
        };
        lease
            .accept_inline_csp(
                &policy,
                &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("admitted connected Style must request a sheet");
        };
        let cssom = commit_actual_empty_sheet(&mut lease, &create, &cancel);
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        cssom
    };
    let init = DomShadowRootInit::new(HtmlShadowRootMode::Open);
    let before = dom.snapshot();
    assert_eq!(before.node(root).unwrap().children(), &[first, style, last]);
    assert!(matches!(
        dom.attach_shadow_synchronously(host, init, limits, None, &cancel),
        Err(crate::DomError::IncompleteSourceOperation)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.pending_source_edit().is_none());
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1_000);
    let mut inbox = dom.node_retirement_inbox(watch_limits, &cancel).unwrap();
    assert!(matches!(
        dom.attach_shadow_with_node_retirement_watch(
            host,
            init,
            (&inbox, 1),
            (limits, watch_limits),
            None,
            &cancel
        ),
        Err(crate::DomError::IncompleteSourceOperation)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.pending_source_edit().is_none());
    assert!(inbox.take().is_none());
    let watch = dom
        .watch_node_retirement(root, &inbox, 1, watch_limits, &cancel)
        .unwrap();
    drop(watch);

    let mut lease = dom
        .begin_source_edit(DomSourceEdit::attach_shadow(host, init), limits, &cancel)
        .unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(remove)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("reuse must reach the existing Style sheet removal");
    };
    assert_eq!(
        remove.action(),
        DomStyleAction::Update(HtmlStyleUpdateAction::RemoveAssociatedSheet)
    );
    assert_eq!(remove.source().element(), style);
    let parked = lease.snapshot();
    assert_eq!(parked.node(first).unwrap().parent(), None);
    assert_eq!(parked.node(style).unwrap().parent(), None);
    assert_eq!(parked.node(root).unwrap().children(), &[last]);
    assert!(parked.shadow_root(root).unwrap().state().declarative());
    cancel.store(true, Ordering::Release);
    assert_eq!(
        lease.advance(&cancel).unwrap_err(),
        crate::DomError::Cancelled
    );
    assert_eq!(lease.snapshot().stamp(), parked.stamp());
    drop(lease);
    assert_eq!(dom.pending_source_edit().unwrap().operation(), operation);
    cancel.store(false, Ordering::Release);

    let mut lease = dom.resume_source_edit(operation, limits, &cancel).unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(retry)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("original CSSOM removal request must survive the refused advance");
    };
    assert_eq!(retry.association(), remove.association());
    assert_eq!(retry.source().element(), style);
    let retry = lease
        .qualify_cssom(&retry, cssom.root(), &cssom.snapshot().unwrap())
        .unwrap();
    let mut edit = cssom
        .batch(CssomEditGuard::from_snapshot(&cssom.snapshot().unwrap()))
        .unwrap();
    edit.remove_sheet(retry.sheet().unwrap().id()).unwrap();
    let publication = cssom.commit(&mut Some(edit)).unwrap();
    lease
        .accept_cssom_publication(&retry, publication, cssom.snapshot().unwrap(), None)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the original attachment must finish after CSSOM removal");
    };
    assert_eq!(done.operation(), operation);
    assert_eq!(done.output(), Some(DomSourceOutput::Node(root)));
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    let after = dom.snapshot();
    assert_eq!(after.node(host).unwrap().hosted_shadow_root(), Some(root));
    assert!(after.node(root).unwrap().children().is_empty());
    assert!(!after.shadow_root(root).unwrap().state().declarative());
    assert_eq!(after.node(last).unwrap().parent(), None);
    assert!(cssom.snapshot().unwrap().sheets().is_empty());
}

#[test]
fn synchronous_reuse_never_exposes_an_unserviceable_fullscreen_handoff() {
    use crate::store::source::operation::fullscreen::native_fixture;

    let cancel = AtomicBool::new(false);
    let limits = DomSourceLimits::new(16_000_000, 200_000, 10_000_000, 64, 64, 64, 2);
    let (mut dom, _source, document, previous_fullscreen) = native_fixture(
        DomLimits::new(
            256,
            65_536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_fullscreen_exits(4)
        .unwrap()
        .with_fullscreen_events(4)
        .unwrap(),
    );
    let (_, host, root, [first, fullscreen]) = parsed_open_root(&mut dom, &cancel);
    let parent = dom
        .snapshot()
        .node(previous_fullscreen)
        .unwrap()
        .parent()
        .unwrap();
    let mut batch = DomBatch::new();
    batch.unfullscreen_element_model(previous_fullscreen);
    batch.adopt(host, document);
    batch.insert_before(host, parent, None);
    batch.fullscreen_element_model(fullscreen);
    dom.apply(batch, &cancel).unwrap();

    let before = dom.snapshot();
    let identity = dom.next_fullscreen_exit;
    let init = DomShadowRootInit::new(HtmlShadowRootMode::Open);
    let watch_limits = DomNodeRetirementLimits::new(16_000_000, 1_000);
    let mut inbox = dom.node_retirement_inbox(watch_limits, &cancel).unwrap();
    for watch in [false, true] {
        let refusal = if watch {
            dom.attach_shadow_with_node_retirement_watch(
                host,
                init,
                (&inbox, 1),
                (limits, watch_limits),
                None,
                &cancel,
            )
            .map(drop)
        } else {
            dom.attach_shadow_synchronously(host, init, limits, None, &cancel)
                .map(drop)
        };
        assert_eq!(
            refusal.unwrap_err(),
            crate::DomError::IncompleteSourceOperation
        );
        assert!(
            dom.pending_source_edit().is_none(),
            "a context-free refusal must release the unpublished Source"
        );
        assert_eq!(dom.next_fullscreen_exit, identity);
        assert!(dom.fullscreen_exits.is_empty());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot().node(root).unwrap().children(),
            &[first, fullscreen]
        );
        assert!(
            dom.snapshot()
                .shadow_root(root)
                .unwrap()
                .state()
                .declarative()
        );
        assert!(inbox.take().is_none());
    }

    let mut lease = dom
        .begin_source_edit(DomSourceEdit::attach_shadow(host, init), limits, &cancel)
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the original ordinary Source path must expose the real fullscreen handoff");
    };
    assert_eq!(request.operation(), lease.operation());
    assert_eq!(lease.snapshot().stamp(), before.stamp());
}
