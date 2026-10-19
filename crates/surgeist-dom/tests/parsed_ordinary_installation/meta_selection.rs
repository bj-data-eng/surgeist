//! Parser-owned intrinsic Meta selections transfer without native processing.
use super::*;
use surgeist_color::Color;
use surgeist_css::{CssColorSchemeKeyword, CssKnownPropertyValueRef, CssValueOrigin};

fn transfer_owner() -> (Dom, DomParsedDocumentEnvironment) {
    let (mut dom, environment) = setup();
    // Admit the retained selected CSS report only. Native Meta work, theme
    // scratch and CSS parsing limits remain unconfigured: installation must
    // transfer the completed producer state, not replay native Meta hooks.
    dom.set_limits(
        dom.limits()
            .with_meta_color_scheme_retained_bytes(1_000_000)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dom.limits().meta_theme_color_scratch_bytes(), 0);
    assert!(dom.limits().meta_color_scheme_limits().is_none());
    (dom, environment)
}

#[test]
fn completed_parser_theme_color_skips_invalid_candidate_and_transfers_literal_rgba() {
    // Pinned Meta selection walks candidates in tree order and skips a failed
    // CSS color parse. #3699 expands each nibble, including alpha, by 17.
    let markup = concat!(
        "<!doctype html><head>",
        "<meta name=theme-color content=not-a-color>",
        "<meta name=theme-color content=' #3699 '>",
        "</head>"
    );
    let report = parse(HtmlSource::utf8(markup));
    let source = report.source().identity().clone();
    let (mut dom, environment) = transfer_owner();
    let before = dom.snapshot();
    let installed = dom
        .install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment,
            limits(),
            &AtomicBool::new(false),
        )
        .expect("completed intrinsic theme-color selection must be transferable");
    let snapshot = installed.snapshot();
    assert_eq!(snapshot.revision().get(), before.revision().get() + 1);
    assert_eq!(installed.changes().from(), before.stamp());
    assert_eq!(installed.changes().to(), snapshot.stamp());
    assert_eq!(
        snapshot
            .document(installed.document())
            .unwrap()
            .metadata()
            .meta_theme_color(),
        &DomMetaThemeColorState::Known(Some(Color::from_srgba8([51, 102, 153, 153])))
    );
    let head = element(&installed, "head");
    let candidates = snapshot.node(head).unwrap().children();
    assert_eq!(candidates.len(), 2);
    for (id, content) in candidates.iter().zip(["not-a-color", " #3699 "]) {
        let node = snapshot.node(*id).unwrap();
        let name = node.as_element().unwrap().name();
        assert_eq!(name.namespace_uri(), Namespace::html().uri());
        assert_eq!(name.local_name(), &DocumentString::from("meta"));
        assert_eq!(node.attribute(&"name".into()), Some(&"theme-color".into()));
        assert_eq!(node.attribute(&"content".into()), Some(&content.into()));
    }
    assert!(std::ptr::eq(installed.report(), report.as_ref()));
    assert_eq!(report.source().identity(), &source);
    assert_eq!(report.source().as_utf8(), Some(markup));
    assert_eq!(dom.limits().meta_theme_color_scratch_bytes(), 0);
    assert!(dom.limits().meta_color_scheme_limits().is_none());
}

#[test]
fn completed_parser_color_scheme_skips_invalid_syntax_and_transfers_authored_dark() {
    // A lone reserved `only` has no scheme and is invalid; an arbitrary unknown
    // identifier would be a valid custom scheme and would not test fallback.
    let markup = concat!(
        "<!doctype html><head>",
        "<meta name=color-scheme content=only>",
        "<meta name=color-scheme content=dark>",
        "</head>"
    );
    let report = parse(HtmlSource::utf8(markup));
    let source = report.source().identity().clone();
    let (mut dom, environment) = transfer_owner();
    let before = dom.snapshot();
    let installed = dom
        .install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment,
            limits(),
            &AtomicBool::new(false),
        )
        .expect("completed intrinsic color-scheme selection must be transferable");
    let snapshot = installed.snapshot();
    assert_eq!(snapshot.revision().get(), before.revision().get() + 1);
    assert_eq!(installed.changes().from(), before.stamp());
    assert_eq!(installed.changes().to(), snapshot.stamp());
    let DomMetaColorSchemeState::Known(Some(selected)) = snapshot
        .document(installed.document())
        .unwrap()
        .metadata()
        .meta_color_scheme()
    else {
        panic!("actual completed CSS color-scheme selection")
    };
    let css_source = selected.source().source();
    assert_eq!(css_source.as_str(), "dark");
    assert_eq!(
        css_source.original_utf16().unwrap().as_slice(),
        [100, 97, 114, 107]
    );
    let declaration = selected.result().as_ref().unwrap();
    let CssKnownPropertyValueRef::ColorScheme(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("the retained authored declaration is color-scheme")
    };
    assert_eq!(value.value().schemes(), &[CssColorSchemeKeyword::Dark]);
    assert!(!value.value().only());
    for component in declaration.value_components().items() {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("the actual authored component retains its parsed origin")
        };
        assert!(origin.source().same_snapshot(css_source));
    }
    let head = element(&installed, "head");
    let candidates = snapshot.node(head).unwrap().children();
    assert_eq!(candidates.len(), 2);
    for (id, content) in candidates.iter().zip(["only", "dark"]) {
        let node = snapshot.node(*id).unwrap();
        let name = node.as_element().unwrap().name();
        assert_eq!(name.namespace_uri(), Namespace::html().uri());
        assert_eq!(name.local_name(), &DocumentString::from("meta"));
        assert_eq!(node.attribute(&"name".into()), Some(&"color-scheme".into()));
        assert_eq!(node.attribute(&"content".into()), Some(&content.into()));
    }
    assert!(std::ptr::eq(installed.report(), report.as_ref()));
    assert_eq!(report.source().identity(), &source);
    assert_eq!(report.source().as_utf8(), Some(markup));
    assert_eq!(dom.limits().meta_theme_color_scratch_bytes(), 0);
    assert!(dom.limits().meta_color_scheme_limits().is_none());
}
