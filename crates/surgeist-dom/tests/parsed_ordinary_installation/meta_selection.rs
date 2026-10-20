//! Parser-owned intrinsic Meta selections transfer without native processing.
use super::*;
use surgeist_color::Color;
use surgeist_css::{
    CssColorSchemeKeyword, CssColorSchemeParseReport, CssComponentValueLimits,
    CssKnownPropertyValueRef, CssUtf16InputLimits, CssUtf16PropertyValueLimits, CssValueOrigin,
};
use surgeist_html_parser::HtmlParseMetadataStep;

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

fn producer_scheme(report: &HtmlParseReport) -> Arc<CssColorSchemeParseReport> {
    let root = report.document().unwrap().root().id();
    let mut metadata = report.metadata();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(meta) = row.document_meta()
                    && meta.document().id() == root
                {
                    return meta.color_scheme().unwrap().clone();
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => panic!("actual root Meta selection"),
        }
    }
}

fn selected_scheme(
    snapshot: &DomSnapshot,
    document: DocumentId,
) -> Option<&CssColorSchemeParseReport> {
    match snapshot
        .document(document)
        .unwrap()
        .metadata()
        .meta_color_scheme()
    {
        DomMetaColorSchemeState::Known(value) => value,
        DomMetaColorSchemeState::Unavailable => panic!("completed intrinsic scheme selection"),
    }
}

fn assert_scheme_keyword(report: &CssColorSchemeParseReport, keyword: CssColorSchemeKeyword) {
    let CssKnownPropertyValueRef::ColorScheme(value) = report
        .result()
        .as_ref()
        .unwrap()
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("authored color-scheme declaration")
    };
    assert_eq!(value.value().schemes(), &[keyword]);
    assert!(!value.value().only());
}

#[test]
fn selected_scheme_aggregate_one_short_and_cancellation_preserve_same_report_for_exact_retry() {
    let first = parse(HtmlSource::utf8(
        "<!doctype html><meta name=color-scheme content=light>",
    ));
    let second_markup = "<!doctype html><meta name=color-scheme content=dark>";
    let second = parse(HtmlSource::utf8(second_markup));
    let first_css = producer_scheme(&first);
    let second_css = producer_scheme(&second);
    assert_scheme_keyword(&first_css, CssColorSchemeKeyword::Light);
    assert_scheme_keyword(&second_css, CssColorSchemeKeyword::Dark);
    // This is the CSS producer's actual retained output receipt, not a quota
    // inferred from fresh installations or the destination's current usage.
    let first_bytes = first_css.retained_storage().bytes();
    let second_bytes = second_css.retained_storage().bytes();
    assert!(first_bytes > 0 && second_bytes > 0);
    let exact = first_bytes.checked_add(second_bytes).unwrap();
    let (mut dom, environment) = transfer_owner();
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            first.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let first_document = installed.document();
    assert!(std::ptr::eq(
        selected_scheme(installed.snapshot(), first_document).unwrap(),
        first_css.as_ref()
    ));
    assert_eq!(dom.usage().meta_color_scheme_retained_bytes(), first_bytes);
    let before = dom.snapshot();
    let usage = dom.usage();
    let first_children = before
        .node(first_document.node())
        .unwrap()
        .children()
        .to_vec();
    let original_source = second.source().identity().clone();
    dom.set_limits(
        dom.limits()
            .with_meta_color_scheme_retained_bytes(exact - 1)
            .unwrap(),
    )
    .unwrap();
    let error = dom
        .install_parsed_html_document(
            before.stamp(),
            second.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert!(
        matches!(error, DomParsedHtmlInstallError::Target(DomError::ResourceLimit {
        resource: DomResource::MetaColorSchemeRetainedBytes, limit, ..
    }) if limit == exact - 1)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot()
            .node(first_document.node())
            .unwrap()
            .children(),
        first_children
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    assert!(std::ptr::eq(
        selected_scheme(&before, first_document).unwrap(),
        first_css.as_ref()
    ));
    assert_eq!(second.source().identity(), &original_source);
    assert_eq!(second.source().as_utf8(), Some(second_markup));

    dom.set_limits(
        dom.limits()
            .with_meta_color_scheme_retained_bytes(exact)
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            second.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(true),
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );

    let retried = dom
        .install_parsed_html_document(
            before.stamp(),
            second.clone(),
            environment,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_ne!(retried.document(), first_document);
    assert_eq!(
        retried.snapshot().revision().get(),
        before.revision().get() + 1
    );
    assert_eq!(retried.changes().from(), before.stamp());
    assert_eq!(retried.changes().to(), retried.snapshot().stamp());
    assert!(std::ptr::eq(retried.report(), second.as_ref()));
    assert!(std::ptr::eq(
        selected_scheme(retried.snapshot(), retried.document()).unwrap(),
        second_css.as_ref()
    ));
    assert_eq!(dom.usage().meta_color_scheme_retained_bytes(), exact);
    assert_scheme_keyword(
        selected_scheme(&before, first_document).unwrap(),
        CssColorSchemeKeyword::Light,
    );
    assert_eq!(
        second_css
            .source()
            .source()
            .original_utf16()
            .unwrap()
            .as_slice(),
        [100, 97, 114, 107]
    );
    assert_eq!(second.source().identity(), &original_source);
    assert_eq!(second.source().as_utf8(), Some(second_markup));
}

#[test]
fn installed_selections_survive_report_release_and_native_reselection_keeps_old_css_source() {
    let report = parse(HtmlSource::utf8(concat!(
        "<!doctype html><head>",
        "<meta name=theme-color content=#3699>",
        "<meta name=color-scheme content=dark></head>"
    )));
    let weak_report = Arc::downgrade(&report);
    let (mut dom, environment) = transfer_owner();
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report.clone(),
            environment,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    let document = installed.document();
    let old = installed.snapshot().clone();
    let head = element(&installed, "head");
    let theme = old.node(head).unwrap().children()[0];
    let scheme = old.node(head).unwrap().children()[1];
    drop(installed);
    drop(report);
    assert!(weak_report.upgrade().is_none());
    let old_css = selected_scheme(&old, document).unwrap();
    assert_scheme_keyword(old_css, CssColorSchemeKeyword::Dark);
    assert_eq!(old_css.source().source().as_str(), "dark");

    // A later ordinary edit uses the native controller with explicitly priced
    // work, scratch and CSS input; installation above supplied none of them.
    dom.set_limits(
        dom.limits()
            .with_meta_processing_work(100_000_000)
            .with_meta_theme_color_scratch_bytes(2 * 1024 * 1024)
            .unwrap()
            .with_meta_color_scheme_limits(CssUtf16PropertyValueLimits {
                input: CssUtf16InputLimits::default(),
                syntax: CssComponentValueLimits::default(),
                max_work: 1_000_000,
                max_live_bytes: 2 * 1024 * 1024,
                max_retained_bytes: 2 * 1024 * 1024,
            })
            .with_meta_color_scheme_retained_bytes(8 * 1024 * 1024)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(theme, "content", "#102030");
    batch.set_attribute(scheme, "content", "light");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert_eq!(current.revision().get(), old.revision().get() + 1);
    assert_eq!(commit.changes().from(), old.stamp());
    assert_eq!(commit.changes().to(), current.stamp());
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentMeta(document))
    );
    assert_eq!(
        current
            .document(document)
            .unwrap()
            .metadata()
            .meta_theme_color(),
        &DomMetaThemeColorState::Known(Some(Color::from_srgba8([16, 32, 48, 255])))
    );
    let new_css = selected_scheme(&current, document).unwrap();
    assert_scheme_keyword(new_css, CssColorSchemeKeyword::Light);
    assert_eq!(new_css.source().source().as_str(), "light");
    assert!(
        !new_css
            .source()
            .source()
            .same_snapshot(old_css.source().source())
    );
    assert_eq!(
        current.node(theme).unwrap().attribute(&"content".into()),
        Some(&"#102030".into())
    );
    assert_eq!(
        current.node(scheme).unwrap().attribute(&"content".into()),
        Some(&"light".into())
    );
    assert_eq!(
        old.document(document)
            .unwrap()
            .metadata()
            .meta_theme_color(),
        &DomMetaThemeColorState::Known(Some(Color::from_srgba8([51, 102, 153, 153])))
    );
    assert_eq!(
        old.node(theme).unwrap().attribute(&"content".into()),
        Some(&"#3699".into())
    );
    assert_eq!(
        old.node(scheme).unwrap().attribute(&"content".into()),
        Some(&"dark".into())
    );
    assert_scheme_keyword(old_css, CssColorSchemeKeyword::Dark);
    assert_eq!(
        old_css
            .source()
            .source()
            .original_utf16()
            .unwrap()
            .as_slice(),
        [100, 97, 114, 107]
    );
}

#[test]
fn template_metas_leave_both_documents_known_absent_and_never_override_outer_selection() {
    for outer in [false, true] {
        let prefix = if outer {
            "<meta name=theme-color content=#3699><meta name=color-scheme content=light>"
        } else {
            ""
        };
        let markup = format!(
            "<!doctype html><head>{prefix}<template><meta name=theme-color content=#ff0000><meta name=color-scheme content=dark></template></head>"
        );
        let report = parse(HtmlSource::utf8(markup.clone()));
        let (mut dom, environment) = transfer_owner();
        let installed = dom
            .install_parsed_html_document(
                dom.snapshot().stamp(),
                report.clone(),
                environment,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let snapshot = installed.snapshot();
        let template = element(&installed, "template");
        let contents = snapshot
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap();
        let contents = snapshot.node(contents).unwrap();
        let inert = contents.owner_document().unwrap();
        assert_ne!(inert, installed.document());
        assert_eq!(contents.fragment_host(), Some(template));
        assert_eq!(
            snapshot.document(inert).unwrap().template_role(),
            DomTemplateDocumentRole::InertTemplate
        );
        assert_eq!(
            snapshot
                .document(installed.document())
                .unwrap()
                .associated_inert_template_document(),
            Some(inert)
        );
        assert_eq!(
            snapshot
                .document(inert)
                .unwrap()
                .metadata()
                .meta_theme_color(),
            &DomMetaThemeColorState::Known(None)
        );
        assert!(selected_scheme(snapshot, inert).is_none());
        let outer_meta = snapshot.document(installed.document()).unwrap().metadata();
        if outer {
            assert_eq!(
                outer_meta.meta_theme_color(),
                &DomMetaThemeColorState::Known(Some(Color::from_srgba8([51, 102, 153, 153])))
            );
            assert_scheme_keyword(
                selected_scheme(snapshot, installed.document()).unwrap(),
                CssColorSchemeKeyword::Light,
            );
        } else {
            assert_eq!(
                outer_meta.meta_theme_color(),
                &DomMetaThemeColorState::Known(None)
            );
            assert!(selected_scheme(snapshot, installed.document()).is_none());
        }
        let inner = contents.children();
        assert_eq!(inner.len(), 2);
        for (node, name, content) in [
            (inner[0], "theme-color", "#ff0000"),
            (inner[1], "color-scheme", "dark"),
        ] {
            let node = snapshot.node(node).unwrap();
            assert_eq!(node.owner_document().unwrap(), inert);
            assert_eq!(node.attribute(&"name".into()), Some(&name.into()));
            assert_eq!(node.attribute(&"content".into()), Some(&content.into()));
        }
        assert!(std::ptr::eq(installed.report(), report.as_ref()));
        assert_eq!(report.source().as_utf8(), Some(markup.as_str()));
    }
}
