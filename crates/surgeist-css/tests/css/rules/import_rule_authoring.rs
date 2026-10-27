//! `@import` (Cascade 5) targets, layers, conditions and placement through the
//! public stylesheet parser and constructors.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::query::CssConditionKind;
use surgeist_css::query::media::{CssMediaLeaf, CssMediaQuery, CssMediaType};
use surgeist_css::query::supports::CssSupportsLeaf;
use surgeist_css::rule::{
    CssImportLayer, CssImportRule, CssImportString, CssImportTarget, CssImportUrl, CssLayerName,
    CssRule,
};

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn import_rule(rule: &CssRule) -> &CssImportRule {
    match rule {
        CssRule::Import(rule) => rule,
        unexpected => panic!("expected import rule, got {unexpected:?}"),
    }
}

#[test]
fn import_layer_name_rejects_empty_components() {
    assert!(CssLayerName::try_new(["theme"]).is_some());
    assert!(CssLayerName::try_new(["theme", "components"]).is_some());
    assert!(CssLayerName::try_new(Vec::<String>::new()).is_none());
    assert!(CssLayerName::try_new([""]).is_none());
    assert!(CssLayerName::try_new(["theme", ""]).is_none());
    assert!(CssLayerName::try_new(["theme", "a\0b"]).is_none());
}

#[test]
fn import_layer_name_preserves_decoded_identifier_components() {
    // Constructors receive decoded identifier values. Their authored spelling
    // escapes punctuation, whitespace and an initial digit into one identifier.
    for (component, spelling) in [
        ("theme components", r"theme\ components"),
        ("theme.components", r"theme\.components"),
        ("theme;", r"theme\;"),
        ("1theme", r"\31 theme"),
        (" \t\n ", r"\ \9 \a \ "),
    ] {
        let name = CssLayerName::try_new(["outer", component]).unwrap();
        assert_eq!(name.components(), &["outer", component]);
        assert_eq!(
            name.serialize_specified(Default::default()).unwrap(),
            format!("outer.{spelling}")
        );
    }
}

#[test]
fn import_layer_name_rejects_reserved_components() {
    assert!(CssLayerName::try_new(["inherit"]).is_none());
    assert!(CssLayerName::try_new(["theme", "initial"]).is_none());
    assert!(CssLayerName::try_new(["theme", "unset"]).is_none());
    assert!(CssLayerName::try_new(["theme", "revert"]).is_none());
    assert!(CssLayerName::try_new(["theme", "revert-layer"]).is_none());
}

#[test]
fn import_target_constructors_preserve_decoded_values() {
    for value in ["", " \t\n "] {
        assert_eq!(
            CssImportUrl::new(surgeist_css::values::common::CssUrl::new(value)).as_str(),
            value
        );
        assert_eq!(CssImportString::new(value).as_str(), value);
    }
    assert_eq!(
        CssImportUrl::new(surgeist_css::values::common::CssUrl::new("theme.css")).as_str(),
        "theme.css"
    );
    assert_eq!(CssImportString::new("theme.css").as_str(), "theme.css");
}

#[test]
fn import_rule_accessors_expose_authored_structure() {
    let target = CssImportTarget::Url(CssImportUrl::new(
        surgeist_css::values::common::CssUrl::new("theme.css"),
    ));
    let layer = CssImportLayer::Named(CssLayerName::try_new(["theme", "components"]).unwrap());
    let location = source_position(3, 7);
    let sheet = CssParser::default()
        .parse_sheet("\n\n\n       @import url(theme.css) layer(theme.components) screen;")
        .expect("clean parse report");
    let [CssRule::Import(rule)] = sheet.rules() else {
        panic!("expected parsed import")
    };

    assert_eq!(rule.target(), &target);
    assert_eq!(rule.layer(), Some(&layer));
    assert_eq!(rule.supports(), None);
    let [CssMediaQuery::Typed(media)] = rule.media().expect("media clause").queries() else {
        panic!("expected one typed media query")
    };
    assert_eq!(media.modifier(), None);
    assert_eq!(media.media_type(), CssMediaType::Screen);
    assert_eq!(media.condition(), None);
    assert_eq!(rule.position(), Some(location));
    let owned_rule = rule.clone();
    assert_eq!(
        CssRule::Import(owned_rule.clone()),
        CssRule::Import(owned_rule)
    );
}

#[test]
fn import_rule_parser_accepts_targets_layers_and_media() {
    let sheet = CssParser::default()
        .parse_sheet(r#"@import "theme.css";"#)
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.target(),
        &CssImportTarget::String(CssImportString::new("theme.css"))
    );
    assert_eq!(rule.layer(), None);
    assert_eq!(rule.media(), None);

    let sheet = CssParser::default()
        .parse_sheet(r#"@import url("layout.css");"#)
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    assert_eq!(
        import_rule(rule).target(),
        &CssImportTarget::Url(CssImportUrl::new(
            surgeist_css::values::common::CssUrl::new("layout.css")
        ))
    );

    let sheet = CssParser::default()
        .parse_sheet("@import url(tokens.css) layer;")
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.target(),
        &CssImportTarget::Url(CssImportUrl::new(
            surgeist_css::values::common::CssUrl::new("tokens.css")
        ))
    );
    assert_eq!(rule.layer(), Some(&CssImportLayer::Anonymous));

    let sheet = CssParser::default()
        .parse_sheet(r#"@import url("components.css") layer(components.buttons);"#)
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.layer(),
        Some(&CssImportLayer::Named(
            CssLayerName::try_new(["components", "buttons"]).unwrap()
        ))
    );

    let sheet = CssParser::default()
        .parse_sheet(r#"@import url("print.css") print;"#)
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let [CssMediaQuery::Typed(print_query)] = import_rule(rule).media().unwrap().queries() else {
        panic!("expected typed print media query");
    };
    assert_eq!(print_query.media_type(), CssMediaType::Print);

    let sheet = CssParser::default()
        .parse_sheet(r#"@import url("wide.css") screen and (min-width: 900px);"#)
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let [CssMediaQuery::Typed(query)] = import_rule(rule).media().unwrap().queries() else {
        panic!("expected typed media query");
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert!(query.condition().is_some());

    let sheet = CssParser::default()
        .parse_sheet(
            r#"@import url("components.css") layer(components) screen and (min-width: 900px);"#,
        )
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.layer(),
        Some(&CssImportLayer::Named(
            CssLayerName::try_new(["components"]).unwrap()
        ))
    );
    assert!(rule.media().is_some());
}

#[test]
fn import_rule_parser_allows_imports_before_style_rules() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"
            @import "theme.css";
            @import url("components.css") layer(components) screen;
            .panel { color: black; }
        "#,
        )
        .expect("clean parse report");

    assert!(matches!(
        sheet.rules(),
        [CssRule::Import(_), CssRule::Import(_), CssRule::Style(_)]
    ));
}

#[test]
fn import_rule_parser_rejects_late_nested_unsupported_and_malformed_imports() {
    for (css, rejection) in [
        (
            r#".panel { color: black; } @import "late.css";"#,
            Rejection::placement("import"),
        ),
        (
            r#"@media screen { @import "nested.css"; }"#,
            Rejection::placement("import"),
        ),
        (
            r#"@scope { @import "nested.css"; }"#,
            Rejection::placement("import"),
        ),
        (
            r#"@import url("theme.css") screen layer(components);"#,
            Rejection::new(CssErrorCode::InvalidMediaQuery, None, Some("layer(")),
        ),
        ("@import;", Rejection::prelude("import", None)),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn import_rule_parser_retains_repeated_supports_as_opaque_media() {
    let sheet = CssParser::default()
        .parse_sheet(r#"@import url("theme.css") supports(display: grid) supports(color: red);"#)
        .expect("clean parse report");
    let [CssRule::Import(rule)] = sheet.rules() else {
        panic!("expected retained import")
    };
    assert_eq!(rule.layer(), None);
    let CssConditionKind::Leaf(CssSupportsLeaf::Declaration(declaration)) = rule
        .supports()
        .expect("first supports clause")
        .condition()
        .kind()
    else {
        panic!("expected declaration supports clause")
    };
    assert_eq!(declaration.authored(), Some("display: grid"));
    let [CssMediaQuery::Condition(condition)] = rule.media().expect("media suffix").queries()
    else {
        panic!("expected one media condition")
    };
    let CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(enclosed)) = condition.kind() else {
        panic!("expected repeated supports function as media")
    };
    assert_eq!(enclosed.authored(), Some("supports(color: red)"));
}
