//! `@layer` (Cascade 5) statement and block forms through the public stylesheet
//! parser and constructors.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn layer_statement_rule(rule: &CssRule) -> &CssLayerStatementRule {
    match rule {
        CssRule::LayerStatement(rule) => rule,
        unexpected => panic!("expected layer statement rule, got {unexpected:?}"),
    }
}

#[test]
fn layer_rule_parser_accepts_statement_and_block_forms() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"
            @layer reset, theme.components;
            @layer theme { .button { color: black; } }
            @layer { .utility { color: red; } }
        "#,
        )
        .expect("clean parse report");

    assert_eq!(sheet.rules().len(), 3);
    let statement = layer_statement_rule(&sheet.rules()[0]);
    assert_eq!(
        statement.names().names(),
        &[
            CssLayerName::try_new(["reset"]).unwrap(),
            CssLayerName::try_new(["theme", "components"]).unwrap(),
        ]
    );

    let named = layer_block_rule(&sheet.rules()[1]);
    assert_eq!(
        named.name(),
        Some(&CssLayerName::try_new(["theme"]).unwrap())
    );
    assert!(matches!(named.rules(), [CssRule::Style(_)]));

    let anonymous = layer_block_rule(&sheet.rules()[2]);
    assert_eq!(anonymous.name(), None);
    assert!(matches!(anonymous.rules(), [CssRule::Style(_)]));
}

#[test]
fn layer_rule_parser_accepts_nested_group_rule_blocks() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"
            @media screen {
                @layer theme.components {
                    @container (inline-size > 30rem) {
                        @layer utilities;
                        .button { color: black; }
                    }
                }
            }
        "#,
        )
        .expect("clean parse report");

    let media = media_rule(&sheet.rules()[0]);
    let [CssRule::LayerBlock(layer)] = media.rules() else {
        panic!("expected layer block inside media");
    };
    assert_eq!(
        layer.name(),
        Some(&CssLayerName::try_new(["theme", "components"]).unwrap())
    );
    let [CssRule::Container(container)] = layer.rules() else {
        panic!("expected container inside layer");
    };
    assert!(matches!(
        container.rules(),
        [CssRule::LayerStatement(_), CssRule::Style(_)]
    ));
}

#[test]
fn layer_rule_parser_rejects_malformed_layer_syntax() {
    for (css, rejection) in [
        (
            "@layer;",
            Rejection::new(CssErrorCode::UnexpectedToken, None, Some(";")),
        ),
        ("@layer , theme;", Rejection::prelude("layer", Some(","))),
        ("@layer theme,;", Rejection::prelude("layer", Some(","))),
        (
            "@layer theme..components;",
            Rejection::prelude("layer", Some(".")),
        ),
        (
            "@layer theme, { .x { color: black; } }",
            Rejection::prelude("layer", None),
        ),
        (
            "@layer theme.components extra { .x { color: black; } }",
            Rejection::prelude("layer", Some("extra")),
        ),
        (
            "@layer initial;",
            Rejection::prelude("layer", Some("initial")),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}
