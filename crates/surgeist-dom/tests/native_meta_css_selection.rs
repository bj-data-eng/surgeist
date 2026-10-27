//! Pinned HTML877d ordered Meta selection through ordinary canonical edits.
#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;
use surgeist_color::Color;
use surgeist_css::component::CssValueOrigin;
use surgeist_css::declaration::CssGlobalKeyword;
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::source::{CssUtf16InputLimits, CssUtf16PropertyValueLimits};
use surgeist_css::values::color_adjust::{CssColorSchemeKeyword, CssColorSchemeParseReport};
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(
        128,
        131_072,
        512,
        128,
        64,
        16,
        1_048_576,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_meta_processing_work(100_000_000)
    .with_meta_theme_color_scratch_bytes(2 * 1024 * 1024)
    .unwrap()
    .with_meta_color_scheme_limits(CssUtf16PropertyValueLimits {
        input: CssUtf16InputLimits::default(),
        max_work: 1_000_000,
        max_live_bytes: 2 * 1024 * 1024,
        max_retained_bytes: 2 * 1024 * 1024,
    })
    .with_meta_color_scheme_retained_bytes(8 * 1024 * 1024)
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn fixture() -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let head = batch.create_element_ns(&document, Namespace::html(), "head");
    batch.insert_before(&head, &document, None);
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&head).unwrap().unwrap(),
    )
}

fn theme(snapshot: &DomSnapshot, document: DocumentId) -> Option<Color> {
    match snapshot
        .document(document)
        .unwrap()
        .metadata()
        .meta_theme_color()
    {
        DomMetaThemeColorState::Known(value) => *value,
        DomMetaThemeColorState::Unavailable => panic!("native selection is established"),
    }
}

fn scheme(snapshot: &DomSnapshot, document: DocumentId) -> Option<&CssColorSchemeParseReport> {
    match snapshot
        .document(document)
        .unwrap()
        .metadata()
        .meta_color_scheme()
    {
        DomMetaColorSchemeState::Known(value) => value,
        DomMetaColorSchemeState::Unavailable => panic!("native selection is established"),
    }
}

#[test]
fn theme_selects_first_valid_color_and_retains_alpha_across_winner_changes() {
    let (mut dom, document, head) = fixture();
    let initial = dom.snapshot();
    assert_eq!(theme(&initial, document), None);
    let mut batch = DomBatch::new();
    let invalid = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&invalid, "name", "theme-color");
    batch.set_attribute(&invalid, "content", "not-a-color");
    batch.insert_before(&invalid, head, None);
    let first = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&first, "name", "ThEmE-CoLoR");
    batch.set_attribute(&first, "content", " \t#3699\r\n");
    batch.insert_before(&first, head, None);
    let second = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&second, "name", "theme-color");
    batch.set_attribute(&second, "content", "#102030");
    batch.insert_before(&second, head, None);
    let commit = apply(&mut dom, batch);
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let selected = dom.snapshot();
    assert_eq!(
        theme(&selected, document),
        Some(Color::from_srgba8([51, 102, 153, 153]))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentMeta(document))
    );

    let mut batch = DomBatch::new();
    batch.set_attribute(first, "name", "description");
    apply(&mut dom, batch);
    assert_eq!(
        theme(&dom.snapshot(), document),
        Some(Color::from_srgba8([16, 32, 48, 255]))
    );
    let mut batch = DomBatch::new();
    batch.detach(second);
    apply(&mut dom, batch);
    assert_eq!(theme(&dom.snapshot(), document), None);
    assert_eq!(
        theme(&selected, document),
        Some(Color::from_srgba8([51, 102, 153, 153]))
    );
    assert_eq!(theme(&initial, document), None);
}

#[test]
fn scheme_ignores_media_and_preserves_original_components_and_symbolic_keywords() {
    let (mut dom, document, head) = fixture();
    let initial = dom.snapshot();
    assert!(scheme(&initial, document).is_none());
    let mut batch = DomBatch::new();
    let invalid = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&invalid, "name", "color-scheme");
    batch.set_attribute(&invalid, "content", "only");
    batch.insert_before(&invalid, head, None);
    let valid = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&valid, "name", "COLOR-SCHEME");
    batch.set_attribute(&valid, "content", "\r\nonly light dark");
    batch.set_attribute(&valid, "media", "(unavailable-feature: 17)");
    batch.insert_before(&valid, head, None);
    let commit = apply(&mut dom, batch);
    let valid = commit.created(&valid).unwrap().unwrap();
    let selected = dom.snapshot();
    let report = scheme(&selected, document).unwrap();
    let source = report.source().source();
    assert_eq!(
        source.original_utf16().unwrap().as_slice(),
        "\r\nonly light dark".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(source.as_str(), "\nonly light dark");
    assert_eq!(source.original_utf16_range(0..1), Some(0..2));
    let declaration = report.result().as_ref().unwrap();
    let CssKnownPropertyValueRef::ColorScheme(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("checked color-scheme value");
    };
    assert_eq!(
        value.value().schemes(),
        &[CssColorSchemeKeyword::Light, CssColorSchemeKeyword::Dark]
    );
    assert!(value.value().only());
    for item in declaration.value_components().items() {
        let CssValueOrigin::Parsed(origin) = item.origin() else {
            panic!("original parsed component");
        };
        assert!(origin.source().same_snapshot(source));
    }

    let mut batch = DomBatch::new();
    batch.set_attribute(valid, "content", "inherit");
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(
        scheme(&current, document)
            .unwrap()
            .result()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .global(),
        Some(CssGlobalKeyword::Inherit)
    );
    assert_eq!(
        source.original_utf16().unwrap().as_slice(),
        "\r\nonly light dark".encode_utf16().collect::<Vec<_>>()
    );
    assert!(scheme(&initial, document).is_none());
}

#[test]
fn undecidable_earlier_theme_blocks_later_literal_and_rolls_back_the_entire_batch() {
    let (mut dom, document, head) = fixture();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(head, "data-before-refusal", "must-not-publish");
    let first = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&first, "name", "theme-color");
    batch.set_attribute(&first, "content", "#010203");
    batch.set_attribute(&first, "media", "(width > 1px)");
    batch.insert_before(&first, head, None);
    let later = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&later, "name", "theme-color");
    batch.set_attribute(&later, "content", "#ffffff");
    batch.insert_before(&later, head, None);
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)),
        Err(DomError::Meta(DomMetaError::ThemeColorMediaUnavailable))
    ));
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        after
            .node(head)
            .unwrap()
            .attribute(&"data-before-refusal".into()),
        None
    );
    assert!(after.node(head).unwrap().children().is_empty());
    assert_eq!(theme(&after, document), None);
}

#[test]
fn absent_content_and_invalid_empty_values_leave_both_native_selections_known_empty() {
    let (mut dom, document, head) = fixture();
    let mut batch = DomBatch::new();
    for name in ["theme-color", "color-scheme"] {
        for content in [None, Some(""), Some(" \t\r\n ")] {
            let meta = batch.create_element_ns(document, Namespace::html(), "meta");
            batch.set_attribute(&meta, "name", name);
            if let Some(content) = content {
                batch.set_attribute(&meta, "content", content);
            }
            batch.insert_before(&meta, head, None);
        }
    }
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(theme(&snapshot, document), None);
    assert!(scheme(&snapshot, document).is_none());
    assert_eq!(snapshot.incomplete_source_operation(), None);
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn contextual_theme_is_unavailable_rather_than_invalid_css_or_a_later_literal() {
    let (mut dom, document, head) = fixture();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let contextual = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&contextual, "name", "theme-color");
    batch.set_attribute(&contextual, "content", "currentColor");
    batch.insert_before(&contextual, head, None);
    let literal = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&literal, "name", "theme-color");
    batch.set_attribute(&literal, "content", "red");
    batch.insert_before(&literal, head, None);
    // CSS Color 4 §4.5 uses initial properties without a context Element;
    // currentColor therefore reaches the unsupplied CanvasText palette fact.
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)),
        Err(DomError::Meta(DomMetaError::ThemeColorUnavailable(
            surgeist_css::color::CssColorUnavailable::Palette(
                surgeist_css::color::CssSystemColor::CanvasText
            )
        )))
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.snapshot().node(head).unwrap().children().is_empty());
    assert_eq!(theme(&dom.snapshot(), document), None);
}

#[test]
fn later_reached_csp_failure_discards_an_earlier_private_theme_change() {
    let (mut dom, document, head) = fixture();
    let mut batch = DomBatch::new();
    let theme_meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&theme_meta, "name", "theme-color");
    batch.set_attribute(&theme_meta, "content", "#0000ff");
    batch.insert_before(&theme_meta, head, None);
    let commit = apply(&mut dom, batch);
    let theme_meta = commit.created(&theme_meta).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(theme_meta, "content", "#00ff00");
    let csp = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&csp, "http-equiv", "content-security-policy");
    batch.set_attribute(&csp, "content", "default-src 'none'");
    batch.insert_before(&csp, head, None);
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)),
        Err(DomError::Meta(
            DomMetaError::ContentSecurityPolicyUnsupported
        ))
    ));
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        theme(&after, document),
        Some(Color::from_srgba8([0, 0, 255, 255]))
    );
    assert_eq!(
        after.node(theme_meta).unwrap().attribute(&"content".into()),
        Some(&"#0000ff".into())
    );
    assert_eq!(after.node(head).unwrap().children(), &[theme_meta]);
}

#[test]
fn equal_scheme_recomputations_retain_backing_but_changed_original_units_replace_it() {
    let (mut dom, document, head) = fixture();
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "name", "color-scheme");
    batch.set_attribute(&meta, "content", "\r\nlight");
    batch.insert_before(&meta, head, None);
    let commit = apply(&mut dom, batch);
    let meta = commit.created(&meta).unwrap().unwrap();
    let before = dom.snapshot();
    let original = scheme(&before, document).unwrap();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "\r\nlight");
    let commit = apply(&mut dom, batch);
    let equal = dom.snapshot();
    assert_eq!(equal.stamp(), before.stamp());
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.usage(), usage);
    assert!(
        scheme(&equal, document)
            .unwrap()
            .source()
            .source()
            .same_snapshot(original.source().source())
    );
    assert!(
        scheme(&equal, document)
            .unwrap()
            .result()
            .as_ref()
            .unwrap()
            .same_occurrence(original.result().as_ref().unwrap())
    );

    // Another edit can publish while the recomputed selected CSS keeps its owner.
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "\r\nlight");
    batch.set_attribute(head, "data-selection-control", "changed");
    let commit = apply(&mut dom, batch);
    let published = dom.snapshot();
    assert_ne!(published.stamp(), equal.stamp());
    assert!(
        !commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentMeta(document))
    );
    assert!(
        scheme(&published, document)
            .unwrap()
            .source()
            .source()
            .same_snapshot(original.source().source())
    );

    // Normalized text alone must not erase a change in original UTF16 provenance.
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "\nlight");
    let commit = apply(&mut dom, batch);
    let changed = dom.snapshot();
    let replacement = scheme(&changed, document).unwrap();
    assert_eq!(
        replacement.source().source().as_str(),
        original.source().source().as_str()
    );
    assert!(
        !replacement
            .source()
            .source()
            .same_snapshot(original.source().source())
    );
    assert_eq!(
        replacement
            .source()
            .source()
            .original_utf16()
            .unwrap()
            .as_slice(),
        "\nlight".encode_utf16().collect::<Vec<_>>()
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentMeta(document))
    );
}

#[test]
fn adoption_recomputes_source_and_destination_from_their_actual_ordinary_trees() {
    let (mut dom, source, head) = fixture();
    let mut batch = DomBatch::new();
    let meta = batch.create_element_ns(source, Namespace::html(), "meta");
    batch.set_attribute(&meta, "name", "theme-color");
    batch.set_attribute(&meta, "content", "#ff0000");
    batch.insert_before(&meta, head, None);
    let destination = batch.create_document();
    let destination_head = batch.create_element_ns(&destination, Namespace::html(), "head");
    batch.insert_before(&destination_head, &destination, None);
    let other = batch.create_element_ns(&destination, Namespace::html(), "meta");
    batch.set_attribute(&other, "name", "theme-color");
    batch.set_attribute(&other, "content", "#0000ff");
    batch.insert_before(&other, &destination_head, None);
    let commit = apply(&mut dom, batch);
    let meta = commit.created(&meta).unwrap().unwrap();
    let destination = commit.document(&destination).unwrap();
    let destination_head = commit.created(&destination_head).unwrap().unwrap();
    let other = commit.created(&other).unwrap().unwrap();
    let before = dom.snapshot();
    assert_eq!(
        theme(&before, source),
        Some(Color::from_srgba8([255, 0, 0, 255]))
    );
    assert_eq!(
        theme(&before, destination),
        Some(Color::from_srgba8([0, 0, 255, 255]))
    );

    let mut batch = DomBatch::new();
    batch.adopt(meta, destination);
    batch.insert_before(meta, destination_head, None);
    apply(&mut dom, batch);
    let adopted = dom.snapshot();
    assert_eq!(theme(&adopted, source), None);
    assert_eq!(
        theme(&adopted, destination),
        Some(Color::from_srgba8([0, 0, 255, 255]))
    );
    assert_eq!(
        adopted.node(meta).unwrap().owner_document(),
        Some(destination)
    );
    assert_eq!(
        theme(&before, source),
        Some(Color::from_srgba8([255, 0, 0, 255]))
    );

    let mut batch = DomBatch::new();
    batch.insert_before(meta, destination_head, Some(other.into()));
    apply(&mut dom, batch);
    assert_eq!(
        theme(&dom.snapshot(), destination),
        Some(Color::from_srgba8([255, 0, 0, 255]))
    );
}

#[test]
fn aggregate_selected_scheme_limit_covers_both_documents_and_atomic_growth() {
    let (mut dom, first_document, first_head) = fixture();
    let mut batch = DomBatch::new();
    let first = batch.create_element_ns(first_document, Namespace::html(), "meta");
    batch.set_attribute(&first, "name", "color-scheme");
    batch.set_attribute(&first, "content", "light");
    batch.insert_before(&first, first_head, None);
    let second_document = batch.create_document();
    let second = batch.create_element_ns(&second_document, Namespace::html(), "meta");
    batch.set_attribute(&second, "name", "color-scheme");
    batch.set_attribute(&second, "content", "dark");
    batch.insert_before(&second, &second_document, None);
    let commit = apply(&mut dom, batch);
    let first = commit.created(&first).unwrap().unwrap();
    let second_document = commit.document(&second_document).unwrap();
    let before = dom.snapshot();
    let expected = scheme(&before, first_document)
        .unwrap()
        .retained_storage()
        .bytes()
        + scheme(&before, second_document)
            .unwrap()
            .retained_storage()
            .bytes();
    assert!(expected > 0);
    assert_eq!(dom.usage().meta_color_scheme_retained_bytes(), expected);
    let original_limits = dom.limits();
    assert!(matches!(
        dom.set_limits(
            original_limits
                .with_meta_color_scheme_retained_bytes(expected - 1)
                .unwrap()
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::MetaColorSchemeRetainedBytes,
            ..
        })
    ));
    assert_eq!(dom.limits(), original_limits);
    dom.set_limits(
        original_limits
            .with_meta_color_scheme_retained_bytes(expected)
            .unwrap(),
    )
    .unwrap();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(
        first,
        "content",
        "FutureTheme FutureThemeAgain FutureThemeThird",
    );
    assert!(matches!(
        dom.apply(batch, &AtomicBool::new(false)),
        Err(DomError::ResourceLimit {
            resource: DomResource::MetaColorSchemeRetainedBytes,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot()
            .node(first)
            .unwrap()
            .attribute(&"content".into()),
        Some(&"light".into())
    );
}
