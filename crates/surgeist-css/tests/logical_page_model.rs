#![forbid(unsafe_code)]
//! Functional evidence for new page classifications and page specificity APIs.
//! Logical 1 WD20251204 §3 equates logical and physical side specificity;
//! CSS2 §13.2.2 orders unqualified pages below side pages below first pages.

use surgeist_css::*;

#[test]
fn parsed_page_classifications_retain_their_symbolic_identity_in_each_mode() {
    for (pseudo, canonical, selector) in [
        ("left", "left", CssPagePseudo::Left),
        ("right", "right", CssPagePseudo::Right),
        ("first", "first", CssPagePseudo::First),
        ("RECTO", "recto", CssPagePseudo::Recto),
        (r"\76 erso", "verso", CssPagePseudo::Verso),
    ] {
        for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
            let context = CssParserContext::new(mode);
            let source = format!("@page :{pseudo}{{margin-left:2cm!important}}");
            let report = context.parse_rule(&source, &CssNamespaceContext::default());
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            let Some(CssRule::Page(page)) = report.syntax() else {
                panic!("one typed page rule")
            };
            assert_eq!(page_pseudo(page), Some(selector));
            assert_eq!(page_specificity(page), selector.specificity());
            assert_eq!(
                page.declarations().properties()[0].importance(),
                CssImportance::Important
            );
            let canonical = format!("@page :{canonical} {{ margin-left: 2cm !important; }}");
            assert_eq!(
                report
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .to_specified_css()
                    .unwrap(),
                canonical
            );
            let reparsed = context.parse_sheet(&canonical);
            assert!(reparsed.is_clean());
            let [CssRule::Page(reparsed)] = reparsed.syntax().rules() else {
                panic!("one reparsed page")
            };
            assert_eq!(page_pseudo(reparsed), Some(selector));
            assert_eq!(page_specificity(reparsed), selector.specificity());
        }
    }
    assert_ne!(CssPagePseudo::Recto, CssPagePseudo::Left);
    assert_ne!(CssPagePseudo::Recto, CssPagePseudo::Right);
    assert_ne!(CssPagePseudo::Verso, CssPagePseudo::Left);
    assert_ne!(CssPagePseudo::Verso, CssPagePseudo::Right);
    assert_ne!(CssPagePseudo::Recto, CssPagePseudo::Verso);
}

#[test]
fn page_specificity_compares_default_side_and_first_without_mapping_or_importance() {
    let report = parse_sheet(concat!(
        "@page{margin-left:1px!important}",
        "@page :left{margin-left:2px}",
        "@page :right{margin-left:3px!important}",
        "@page :recto{margin-left:4px}",
        "@page :verso{margin-left:5px!important}",
        "@page :first{margin-left:6px}",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Page(default),
        CssRule::Page(left),
        CssRule::Page(right),
        CssRule::Page(recto),
        CssRule::Page(verso),
        CssRule::Page(first),
    ] = report.syntax().rules()
    else {
        panic!("default, four sides, and first page")
    };
    assert_eq!(page_pseudo(default), None);
    assert_eq!(page_specificity(default), CssPageSpecificity::new(0, 0, 0));
    assert_eq!(page_specificity(first), CssPageSpecificity::new(0, 1, 0));
    for page in [left, right, recto, verso] {
        assert_eq!(page_specificity(page), CssPageSpecificity::new(0, 0, 1));
        assert_eq!(
            page_specificity(page),
            page_pseudo(page).unwrap().specificity()
        );
        assert!(page_specificity(default) < page_specificity(page));
        assert!(page_specificity(page) < page_specificity(first));
    }
    assert_eq!(
        page_specificity(recto).cmp(&page_specificity(verso)),
        std::cmp::Ordering::Equal
    );
    assert_eq!(page_specificity(left), page_specificity(right));
    assert_eq!(page_specificity(recto), page_specificity(left));
    assert_eq!(page_specificity(verso), page_specificity(right));
    assert!(page_specificity(default) < page_specificity(first));
    assert_eq!(CssPagePseudo::First.specificity(), page_specificity(first));
}

#[test]
fn normalization_preserves_typed_page_classification_and_specificity_inside_groups() {
    let source = concat!(
        "@scope (.root){@media print{",
        "@page :recto{margin-top:1px}",
        "@page :verso{margin-top:2px!important}",
        "@page :first{margin-top:3px}",
        "@page{margin-top:4px}",
        "}}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let pages: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::Page(page) => Some(page),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(pages.len(), 4);
    for (page, selector, specificity, importance, marker) in [
        (
            pages[0],
            Some(CssPagePseudo::Recto),
            CssPageSpecificity::new(0, 0, 1),
            CssImportance::Normal,
            "@page :recto",
        ),
        (
            pages[1],
            Some(CssPagePseudo::Verso),
            CssPageSpecificity::new(0, 0, 1),
            CssImportance::Important,
            "@page :verso",
        ),
        (
            pages[2],
            Some(CssPagePseudo::First),
            CssPageSpecificity::new(0, 1, 0),
            CssImportance::Normal,
            "@page :first",
        ),
        (
            pages[3],
            None,
            CssPageSpecificity::new(0, 0, 0),
            CssImportance::Normal,
            "@page{",
        ),
    ] {
        assert_eq!(page_pseudo(page), selector);
        assert_eq!(page_specificity(page), specificity);
        assert_eq!(page.declarations().properties()[0].importance(), importance);
        assert_eq!(
            page.position().byte_offset().value(),
            source.find(marker).unwrap()
        );
    }
    assert!(
        normalized
            .items()
            .iter()
            .all(|item| !matches!(item, CssNormalizedItem::Declaration(_)))
    );
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@scope (.root) { @media print { @page :recto { margin-top: 1px; } @page :verso { margin-top: 2px !important; } @page :first { margin-top: 3px; } @page { margin-top: 4px; } } }"
    );
}

fn page_pseudo(page: &surgeist_css::CssPageRule) -> Option<surgeist_css::CssPagePseudo> {
    page.selectors()
        .selectors()
        .first()
        .and_then(|s| s.pseudos().first().copied())
}

fn page_specificity(page: &surgeist_css::CssPageRule) -> CssPageSpecificity {
    page.selectors()
        .selectors()
        .first()
        .map(|s| s.specificity())
        .unwrap_or(CssPageSpecificity::new(0, 0, 0))
}
