//! `@keyframes` (CSS Animations 1) authored structure, constructors and strict
//! parsing through the public stylesheet parser.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::property::{CssKnownProperty, CssKnownPropertyValueRef};
use surgeist_css::rule::CssRule;
use surgeist_css::rule::keyframes::{
    CssKeyframePercent, CssKeyframeSelector, CssKeyframesIdent, CssKeyframesName, CssKeyframesRule,
    CssKeyframesString,
};
use surgeist_css::values::animation::CssAnimationName;
use surgeist_css::values::common::CssCustomIdent;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn keyframes_rule(rule: &CssRule) -> &CssKeyframesRule {
    match rule {
        CssRule::Keyframes(rule) => rule,
        unexpected => panic!("expected keyframes rule, got {unexpected:?}"),
    }
}

#[test]
fn keyframes_rule_parser_accepts_strict_blocks() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"@keyframes fade {
            from { opacity: 0; transform: translateX(0px); }
            50% { opacity: 0.5; }
            to { opacity: 1; transform: translateX(10px); }
        }"#,
        )
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one keyframes rule");
    };
    let rule = keyframes_rule(rule);

    assert_eq!(
        rule.name(),
        &CssKeyframesName::Ident(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap()
        )
    );
    assert_eq!(rule.blocks().len(), 3);
    assert_eq!(
        rule.blocks()[0].selectors().selectors(),
        &[CssKeyframeSelector::From]
    );
    assert_eq!(
        rule.blocks()[1].selectors().selectors(),
        &[CssKeyframeSelector::Percent(
            CssKeyframePercent::try_new(50.0).unwrap()
        )]
    );
    assert_eq!(
        rule.blocks()[0].declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::Opacity)
    );
}

#[test]
fn keyframes_rule_parser_accepts_string_names_and_selector_lists() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"@keyframes "fade in" {
            0%, 100% { opacity: 1; }
        }
        .panel { animation-name: "fade in"; animation: "fade in" 120ms ease; }"#,
        )
        .expect("clean parse report");
    let [keyframes, style] = sheet.rules() else {
        panic!("expected keyframes and style rules");
    };

    assert_eq!(
        keyframes_rule(keyframes).name(),
        &CssKeyframesName::String(CssKeyframesString::new("fade in"))
    );
    assert_eq!(
        keyframes_rule(keyframes).blocks()[0]
            .selectors()
            .selectors(),
        &[
            CssKeyframeSelector::Percent(CssKeyframePercent::try_new(0.0).unwrap()),
            CssKeyframeSelector::Percent(CssKeyframePercent::try_new(100.0).unwrap()),
        ]
    );

    let declarations = style_rule(style).declarations();
    assert_eq!(
        declarations[0].known().map(|known| known.property()),
        Some(CssKnownProperty::AnimationName)
    );
    assert_eq!(
        declarations[1].known().map(|known| known.property()),
        Some(CssKnownProperty::Animation)
    );

    let names = property_payload!(declarations[0], AnimationName, value);
    assert_eq!(
        names.names(),
        &[CssAnimationName::String(CssKeyframesString::new("fade in"))]
    );

    let animations = property_payload!(declarations[1], Animation, value);
    assert_eq!(
        animations.values()[0].name(),
        Some(&CssAnimationName::String(CssKeyframesString::new(
            "fade in"
        )))
    );
}

#[test]
fn keyframes_rule_parser_accepts_keyframes_inside_conditional_groups() {
    let sheet = CssParser::default().parse_sheet(r#"@media screen {
            @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
        }
        @container sidebar (inline-size > 30rem) {
            @keyframes slide { 0% { transform: translateX(0px); } 100% { transform: translateX(10px); } }
        }"#)
    .expect("clean parse report");

    let [media, container] = sheet.rules() else {
        panic!("expected media and container rules");
    };
    let [media_keyframes] = media_rule(media).rules() else {
        panic!("expected keyframes inside media");
    };
    let [container_keyframes] = container_rule(container).rules() else {
        panic!("expected keyframes inside container");
    };

    assert_eq!(
        keyframes_rule(media_keyframes).name(),
        &CssKeyframesName::Ident(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap()
        )
    );
    assert_eq!(
        keyframes_rule(container_keyframes).name(),
        &CssKeyframesName::Ident(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new("slide").unwrap()).unwrap()
        )
    );
}

#[test]
fn keyframes_rule_parser_rejects_invalid_names_selectors_and_placements() {
    for (css, rejection) in [
        ("@keyframes fade;", Rejection::body("keyframes", None)),
        (
            "@keyframes { from { opacity: 0; } }",
            Rejection::prelude("keyframes", None),
        ),
        (
            "@keyframes none { from { opacity: 0; } }",
            Rejection::prelude("keyframes", Some("none")),
        ),
        (
            "@keyframes fade { from { .nested { opacity: 0; } } }",
            Rejection::new(CssErrorCode::InvalidQualifiedRule, None, Some(".")),
        ),
        (
            "@keyframes fade { from { @media screen { opacity: 0; } } }",
            Rejection::placement("media"),
        ),
        (
            ".panel { @keyframes fade { from { opacity: 0; } } }",
            Rejection::placement("keyframes"),
        ),
    ] {
        assert_rejection(css, rejection);
    }
    for selector in ["0", "-1%", "101%"] {
        assert_drops_invalid_keyframe_block(
            &format!("@keyframes fade {{ {selector} {{ opacity: 0; }} }}"),
            selector,
        );
    }
}
