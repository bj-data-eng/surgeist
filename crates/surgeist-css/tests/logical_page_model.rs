#![forbid(unsafe_code)]
//! Functional evidence for new page classifications and page specificity APIs.
//! Logical 1 WD20251204 §3 equates logical and physical side specificity;
//! CSS2 §13.2.2 orders unqualified pages below side pages below first pages.

use surgeist_css::*;

#[test]
fn parsed_page_classifications_retain_their_symbolic_identity_in_each_mode() {
    for (pseudo, canonical, selector) in [
        ("left", "left", CssPageSelector::Left),
        ("right", "right", CssPageSelector::Right),
        ("first", "first", CssPageSelector::First),
        ("RECTO", "recto", CssPageSelector::Recto),
        (r"\76 erso", "verso", CssPageSelector::Verso),
    ] {
        for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
            let context = CssParserContext::new(mode);
            let source = format!("@page :{pseudo}{{margin-left:2cm!important}}");
            let report = context.parse_rule(&source, &CssNamespaceContext::default());
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            let Some(CssRule::Page(page)) = report.syntax() else {
                panic!("one typed page rule")
            };
            assert_eq!(page.selector(), Some(selector));
            assert_eq!(page.specificity(), selector.specificity());
            assert_eq!(
                page.declarations()[0].importance(),
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
            assert_eq!(reparsed.selector(), Some(selector));
            assert_eq!(reparsed.specificity(), selector.specificity());
        }
    }
    assert_ne!(CssPageSelector::Recto, CssPageSelector::Left);
    assert_ne!(CssPageSelector::Recto, CssPageSelector::Right);
    assert_ne!(CssPageSelector::Verso, CssPageSelector::Left);
    assert_ne!(CssPageSelector::Verso, CssPageSelector::Right);
    assert_ne!(CssPageSelector::Recto, CssPageSelector::Verso);
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
    assert_eq!(default.selector(), None);
    assert_eq!(default.specificity(), CssPageSpecificity::Unqualified);
    assert_eq!(first.specificity(), CssPageSpecificity::First);
    for page in [left, right, recto, verso] {
        assert_eq!(page.specificity(), CssPageSpecificity::Side);
        assert_eq!(page.specificity(), page.selector().unwrap().specificity());
        assert!(default.specificity() < page.specificity());
        assert!(page.specificity() < first.specificity());
    }
    assert_eq!(
        recto.specificity().cmp(&verso.specificity()),
        std::cmp::Ordering::Equal
    );
    assert_eq!(left.specificity(), right.specificity());
    assert_eq!(recto.specificity(), left.specificity());
    assert_eq!(verso.specificity(), right.specificity());
    assert!(default.specificity() < first.specificity());
    assert_eq!(CssPageSelector::First.specificity(), first.specificity());
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
            Some(CssPageSelector::Recto),
            CssPageSpecificity::Side,
            CssImportance::Normal,
            "@page :recto",
        ),
        (
            pages[1],
            Some(CssPageSelector::Verso),
            CssPageSpecificity::Side,
            CssImportance::Important,
            "@page :verso",
        ),
        (
            pages[2],
            Some(CssPageSelector::First),
            CssPageSpecificity::First,
            CssImportance::Normal,
            "@page :first",
        ),
        (
            pages[3],
            None,
            CssPageSpecificity::Unqualified,
            CssImportance::Normal,
            "@page{",
        ),
    ] {
        assert_eq!(page.selector(), selector);
        assert_eq!(page.specificity(), specificity);
        assert_eq!(page.declarations()[0].importance(), importance);
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
