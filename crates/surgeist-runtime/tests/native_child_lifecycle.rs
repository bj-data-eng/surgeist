//! HTML877d Frame/Iframe post-connection creates a real child navigable before
//! the shared absent-src initial-about:blank ancestor guard returns null.
//! Source completion covers the insertion's mandatory hooks, not queued history.
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
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn session() -> DomSession {
    let dom =
        Dom::new(DomLimits::new(256, 200_000, 1024, 1024, 1024, 32, 4_000_000).unwrap()).unwrap();
    DomSession::new(
        dom,
        DomSessionLimits {
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 32,
                reports: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            details: DomDetailsRuntimeLimits {
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
        },
    )
    .unwrap()
}
fn absent_src_insertion_creates_child(local: &str) {
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        let DomHostedProgress::Created(top) =
            hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
        else {
            panic!("actual initial top-level construction must complete")
        };
        top
    };
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let parent = snapshot.node(html).unwrap().children()[1];
    assert_eq!(
        snapshot
            .node(parent)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &DocumentString::from("body")
    );
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let (closed, container) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        // Native initialization in a genuinely closed associated Document is
        // supported independently of the later hosted insertion hook.
        let closed = batch.create_html_document(top.document(), None);
        let container = batch.create_element_ns(&closed, Namespace::html(), local);
        let commit = dom.apply(batch, &cancel()).unwrap();
        (
            commit.document(&closed).unwrap(),
            commit.created(&container).unwrap().unwrap(),
        )
    };
    let before = session.snapshot();
    let old_node = before.node(container).unwrap();
    assert_eq!(old_node.owner_document(), Some(closed));
    assert_eq!(old_node.parent(), None);
    let old_element = old_node.as_element().unwrap();
    assert_eq!(old_element.child_navigable(), Some(None));
    assert_eq!(old_element.attribute(&DocumentString::from("src")), None);
    assert_eq!(old_element.attribute(&DocumentString::from("srcdoc")), None);
    let mut source = session.source_execution();
    let completion = {
        let mut dom = session.dom();
        let progress = source
            .apply(
                &mut dom,
                DomSourceEdit::insert_before(container, parent, None),
                DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096),
                |_| None,
                &cancel(),
            )
            .expect("absent-src insertion must fulfill the actual child-creation hook");
        let DomSourceDriveProgress::Complete(completion) = progress else {
            panic!("the admitted initial child must finish its mandatory insertion hooks")
        };
        completion
    };
    let current = session.snapshot();
    assert_eq!(completion.commit().changes().to(), current.stamp());
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(current.incomplete_hosted_operation(), None);
    let node = current.node(container).unwrap();
    assert_eq!(node.parent(), Some(parent));
    assert_eq!(node.owner_document(), Some(top.document()));
    assert!(node.is_connected());
    let child = node
        .as_element()
        .unwrap()
        .child_navigable()
        .expect("actual Frame/Iframe family")
        .expect("connection must create its canonical child before attribute processing");
    assert_ne!(child, top.navigable());
    assert!(
        current
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(before.node(container).unwrap().parent(), None);
    assert_eq!(
        before.node(container).unwrap().owner_document(),
        Some(closed)
    );
    assert_eq!(
        before
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

#[test]
fn absent_src_frame_source_insertion_creates_actual_child_and_completes() {
    absent_src_insertion_creates_child("frame");
}

#[test]
fn absent_src_iframe_source_insertion_creates_actual_child_and_completes() {
    absent_src_insertion_creates_child("iframe");
}
