#![forbid(unsafe_code)]
//! Page payloads keep page semantics inside an ordinary group under scope.
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/page.html#page-box
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/
use surgeist_css::*;

fn page_rules(sheet: &CssNormalizedSheet) -> Vec<&CssPageRule> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::Page(page) => Some(page),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn assert_margin(page: &CssPageRule) {
    assert_eq!(page.declarations().len(), 1);
    let declaration = &page.declarations()[0];
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssKnownPropertyValueRef::Margin(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed margin")
    };
    let edges = value.i01_subset().unwrap();
    assert!(matches!(edges.top, CssLength::Px(value) if value.value() == -1.0));
    assert!(matches!(edges.right, CssLength::Percent(value) if value.value() == 2.0));
    assert!(matches!(edges.bottom, CssLength::Auto));
    assert!(
        matches!(edges.left, CssLength::Dimension(value) if value.value() == 3.0 && value.unit() == CssLengthUnit::Cm)
    );
}

#[test]
fn scoped_pages_reuse_payload_without_element_declaration_contributions() {
    for (prelude, selector) in [
        ("", None),
        (":left", Some(CssPageSelector::Left)),
        (":right", Some(CssPageSelector::Right)),
        (":first", Some(CssPageSelector::First)),
    ] {
        let source = format!(
            "@scope(.host){{@media print{{@page {prelude}{{margin:-1px 2% auto 3cm!important}}}}}}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("scope")
        };
        let [CssScopedRule::Media(media)] = scope.rules().rules() else {
            panic!("media")
        };
        let [CssScopedRule::Page(page)] = media.rules().rules() else {
            panic!("typed scoped page")
        };
        assert_eq!(page.selector(), selector);
        assert_eq!(
            page.position().byte_offset().value(),
            source.find("@page").unwrap()
        );
        assert_margin(page);
        let detached = page.clone();
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert!(
            !normalized
                .items()
                .iter()
                .any(|item| matches!(item, CssNormalizedItem::Declaration(_)))
        );
        let pages = page_rules(&normalized);
        assert_eq!(pages.len(), 1);
        assert_eq!(*pages[0], detached);
        assert!(pages[0].declarations()[0].same_occurrence(&page.declarations()[0]));
        drop(report);
        drop(normalized);
        let CssRule::Page(ordinary) = CssRule::Page(detached) else {
            unreachable!()
        };
        assert_margin(&ordinary);
        assert_eq!(ordinary.selector(), selector);
    }
}

#[test]
fn scoped_page_prelude_errors_keep_neighbors_and_valid_page() {
    let source = "@scope{@media all{before{}@page :left;@page :unknown{margin:1px}@page :right{margin-right:2%}after{}}}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropAtRule)
    );
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("media")
    };
    let [
        CssScopedRule::Style(before),
        CssScopedRule::Page(page),
        CssScopedRule::Style(after),
    ] = media.rules().rules()
    else {
        panic!("retained neighbors and valid page")
    };
    assert_eq!(
        before.position().byte_offset().value(),
        source.find("before{}").unwrap()
    );
    assert_eq!(
        after.position().byte_offset().value(),
        source.find("after{}").unwrap()
    );
    assert_eq!(page.selector(), Some(CssPageSelector::Right));
    assert_eq!(
        page.declarations()[0].known().unwrap().property(),
        CssKnownProperty::MarginRight
    );
}

#[test]
fn scoped_page_retains_payload_and_original_positions_at_eof() {
    let source = "@scope(.host){@media print{@page :first{margin:-1px 2% auto 3cm!important";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 3);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("media")
    };
    let [CssScopedRule::Page(page)] = media.rules().rules() else {
        panic!("page")
    };
    assert_margin(page);
    assert_eq!(page.selector(), Some(CssPageSelector::First));
    assert_eq!(
        page.position().byte_offset().value(),
        source.find("@page").unwrap()
    );
    assert_eq!(
        page.declarations()[0]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.find("margin:").unwrap()
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(page_rules(&normalized), vec![page]);
}

#[test]
fn page_leaf_does_not_change_namespace_resolution_of_scoped_neighbors() {
    let source = "@namespace svg 'urn:svg';@scope(svg|root){@media all{svg|before{}@page{margin:1px}bad|lost{}svg|after{}}}";
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one undeclared prefix")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("bad|lost").unwrap()
    );
    let [CssRule::Namespace(_), CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("namespace and scope")
    };
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("media")
    };
    let [
        CssScopedRule::Style(before),
        CssScopedRule::Page(page),
        CssScopedRule::Style(after),
    ] = media.rules().rules()
    else {
        panic!("neighbors and page")
    };
    assert_eq!(page.declarations().len(), 1);
    for (style, local_name) in [(before, "before"), (after, "after")] {
        let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] =
            style.selectors().selectors()
        else {
            panic!("qualified selector")
        };
        let name = selector.type_selector().unwrap();
        assert!(
            matches!(name.namespace(), CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg")
        );
        assert_eq!(name.local_name(), Some(local_name));
    }
    assert_eq!(
        page_rules(&normalize_sheet(report.syntax()).unwrap()).len(),
        1
    );
}

#[test]
fn scoped_page_depth_ceiling_preserves_detached_leaf_and_later_sibling() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            // Scope + 254 media blocks + page is exactly the depth-256 ceiling.
            let prefix = format!("@scope{{{}", "@media all{".repeat(254));
            let source = format!(
                "{prefix}@page:left{{margin:-1px 2% auto 3cm!important}}{}",
                "}".repeat(255)
            );
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let pages = page_rules(&normalized);
            assert_eq!(pages.len(), 1);
            let detached = pages[0].clone();
            assert_margin(&detached);
            let cloned_sheet = report.syntax().clone();
            drop(report);
            drop(normalized);
            drop(cloned_sheet);
            assert_margin(&detached.clone());
            drop(detached);
            let exceeded = format!(
                "{prefix}@media all{{@page{{margin:1px}}}}{}tail{{}}",
                "}".repeat(255)
            );
            let report = parse_sheet(&exceeded);
            let [diagnostic] = report.diagnostics() else {
                panic!("one excess page block")
            };
            assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
            assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
            assert!(matches!(
                report.syntax().rules().last(),
                Some(CssRule::Style(_))
            ));
            assert!(page_rules(&normalize_sheet(report.syntax()).unwrap()).is_empty());
        })
        .unwrap()
        .join()
        .unwrap();
}
