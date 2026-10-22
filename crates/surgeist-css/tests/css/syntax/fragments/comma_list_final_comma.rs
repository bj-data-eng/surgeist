//! Selected Syntax 3 (2026-10-01) §5.4.10 "parse a comma-separated list of
//! component values" appends a group only while its input is not empty, then
//! discards the comma. A comma followed by no token ends the list without an
//! empty group; whitespace is a token, comments are not (§4.3.1). The
//! consumers here are Media Queries 4 §3 `<media-query-list>`, Selectors 4
//! forgiving selector lists and Fonts 4 `<font-src-list>`, through §5.4.2
//! "parse a list" where named.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#parse-comma-separated-list-of-component-values
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#parse-comma-list
//! https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/#mq-syntax
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#forgiving-selector
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
use crate::css::support::authored_syntax::parsed_origin;
use surgeist_css::{
    CssCustomMediaBody, CssCustomMediaConstructionError, CssCustomMediaRule,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue, CssImportConstructionError,
    CssImportRule, CssMediaConstructionError, CssMediaQuery, CssMediaQueryList,
    CssNamespaceContext, CssRecoveryAction, CssRule, parse_component_values,
    parse_media_query_list, parse_selector, parse_sheet,
};

fn never_count(list: &CssMediaQueryList) -> usize {
    list.queries()
        .iter()
        .filter(|query| matches!(query, CssMediaQuery::Never(_)))
        .count()
}

#[test]
fn a_final_comma_without_a_following_token_adds_no_media_query() {
    for source in ["screen,", "screen,/**/", "screen,print,"] {
        let report = parse_media_query_list(source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert_eq!(never_count(report.syntax()), 0, "{source:?}");
        let expected = if source.contains("print") { 2 } else { 1 };
        assert_eq!(report.syntax().queries().len(), expected, "{source:?}");
    }
    // Whitespace after the comma is a token and so forms a malformed member,
    // as does every empty member between commas or before the first comma.
    for (source, members, never) in [
        ("screen, ", 2, 1),
        ("screen,,", 2, 1),
        (",", 1, 1),
        (",screen", 2, 1),
    ] {
        let report = parse_media_query_list(source);
        assert_eq!(report.syntax().queries().len(), members, "{source:?}");
        assert_eq!(never_count(report.syntax()), never, "{source:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| d.action() == CssRecoveryAction::ReplaceMediaQueryWithNever),
            "{source:?}: {report:?}"
        );
        assert_eq!(report.diagnostics().len(), never, "{source:?}");
    }
}

#[test]
fn rule_preludes_share_the_final_comma_list_boundary() {
    let report = parse_sheet("@media screen,{a{color:red}}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("{report:?}")
    };
    assert_eq!(media.query().queries().len(), 1);
    assert_eq!(media.rules().len(), 1);

    let report = parse_sheet("@import 'x.css' screen,;");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("{report:?}")
    };
    assert_eq!(import.media().unwrap().queries().len(), 1);

    let report = parse_sheet("@custom-media --narrow screen,;");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::CustomMedia(custom)] = report.syntax().rules() else {
        panic!("{report:?}")
    };
    let CssCustomMediaBody::Media(list) = custom.body() else {
        panic!("{custom:?}")
    };
    assert_eq!(list.queries().len(), 1);
}

#[test]
fn forgiving_selector_lists_add_no_failure_for_a_final_comma() {
    let namespaces = CssNamespaceContext::default();
    for source in [":is(a,)", ":where(a,/**/)"] {
        let report = parse_selector(source, &namespaces);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert!(report.syntax().is_some());
    }
    let report = parse_selector(":is(a, )", &namespaces);
    assert!(!report.is_clean(), "{report:?}");
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropSelectorListItem)
    );
}

#[test]
fn font_source_lists_add_no_failure_for_a_final_comma() {
    let report = parse_sheet("@font-face{font-family:x;src:url(a),}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("{report:?}")
    };
    let src = face.descriptors().effective(Kind::Src).unwrap();
    let surgeist_css::CssAuthoredFontFaceDescriptorValue::Ordinary(
        CssFontFaceDescriptorValue::Src(sources),
    ) = src.value()
    else {
        panic!("{src:?}")
    };
    assert_eq!(sources.sources().len(), 1);

    let report = parse_sheet("@font-face{font-family:x;src:url(a), }");
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::DropFontSourceListItem]
    );
}

#[test]
fn checked_media_lists_share_the_final_comma_boundary() {
    let namespaces = CssNamespaceContext::default();
    let import = |source: &str| {
        CssImportRule::try_from_components(parse_component_values(source).unwrap(), &namespaces)
    };
    for source in ["@import 'x.css' screen,;", "@import 'x.css' screen,/**/;"] {
        let rule = import(source).unwrap();
        assert_eq!(rule.media().unwrap().queries().len(), 1, "{source:?}");
    }
    // An empty member names no token; its rejection carries the empty
    // member's position (its whitespace, or the second comma ending it), and
    // both owners locate it alike.
    for (source, responsible) in [
        ("@import 'x.css' screen, ;", Some(23)),
        ("@import 'x.css' screen,,;", Some(23)),
    ] {
        let Err(CssImportConstructionError::Media(error)) = import(source) else {
            panic!("{source:?}: {:?}", import(source))
        };
        assert_invalid_member(&error, source, responsible);
    }

    let custom = |source: &str| {
        CssCustomMediaRule::try_from_components(parse_component_values(source).unwrap())
    };
    let rule = custom("--narrow screen,").unwrap();
    let CssCustomMediaBody::Media(list) = rule.body() else {
        panic!("{rule:?}")
    };
    assert_eq!(list.queries().len(), 1);
    for source in ["--narrow screen, ", "--narrow screen,,"] {
        let Err(CssCustomMediaConstructionError::Media(error)) = custom(source) else {
            panic!("{source:?}: {:?}", custom(source))
        };
        assert_invalid_member(&error, source, Some(16));
    }
}

#[track_caller]
fn assert_invalid_member(
    error: &CssMediaConstructionError,
    source: &str,
    responsible: Option<usize>,
) {
    assert!(
        matches!(error, CssMediaConstructionError::InvalidQueryGrammar { .. }),
        "{source:?}: {error:?}"
    );
    let origin = parsed_origin(error.origin());
    assert_eq!(origin.source().as_str(), source);
    if let Some(responsible) = responsible {
        assert_eq!(
            origin.span().start().byte_offset().value(),
            responsible,
            "{source:?}"
        );
    }
}
