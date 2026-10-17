//! Pinned HTML's environment-independent empty media match for Meta.
#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(32, 4096, 64, 32, 16, 8, 262_144)
        .unwrap()
        .with_meta_processing_work(100_000_000)
}

fn assert_media_matches(media: &str) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let meta = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "name", "theme-color");
    batch.set_attribute(&meta, "content", "#123abc");
    batch.set_attribute(&meta, "media", media);
    batch.insert_before(&meta, &document, None);
    let before = dom.snapshot();
    // HTML's matches-the-environment definition returns true for an empty
    // or ASCII-whitespace-only string before any environment observation.
    let result = dom.apply(batch, &AtomicBool::new(false));
    assert!(
        result.is_ok(),
        "fact-independent media {media:?} must complete: {result:?}"
    );
    let commit = result.unwrap();
    let document = commit.document(&document).unwrap();
    let meta = commit.created(&meta).unwrap().unwrap();
    let current = dom.snapshot();
    assert_ne!(current.stamp(), before.stamp());
    assert_eq!(current.node(meta).unwrap().parent(), Some(document.node()));
    assert_eq!(
        current.node(meta).unwrap().attribute(&"media".into()),
        Some(&DocumentString::from(media))
    );
    assert_eq!(current.incomplete_source_operation(), None);
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn empty_media_matches_without_environment_facts() {
    assert_media_matches("");
}

#[test]
fn ascii_whitespace_media_matches_without_environment_facts() {
    assert_media_matches(" \t\r\n\u{000c}");
}
