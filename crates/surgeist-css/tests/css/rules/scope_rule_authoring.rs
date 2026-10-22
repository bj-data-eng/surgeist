//! `@scope` (Cascade 6) roots, limits and scoped rules through the public
//! stylesheet parser and constructors.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn scope_rule(rule: &CssRule) -> &CssScopeRule {
    match rule {
        CssRule::Scope(rule) => rule,
        unexpected => panic!("expected scope rule, got {unexpected:?}"),
    }
}

#[test]
fn scope_rule_parser_accepts_roots_limits_and_scoped_style_rules() {
    let sheet = parse_sheet(
        r#"
            @scope (.card, [data-scope]) to (.stop, .boundary) {
                .title, > .action { color: black; }
                &:hover { color: red; }
                :scope .note { color: blue; }
            }
        "#,
    )
    .expect("clean parse report");

    let scope = scope_rule(&sheet.rules()[0]);
    assert_eq!(scope.root().unwrap().selectors().len(), 2);
    assert_eq!(scope.limit().unwrap().selectors().len(), 2);

    let [
        CssScopedRule::Style(first),
        CssScopedRule::Style(anchored),
        CssScopedRule::Style(scope_pseudo),
    ] = scope.rules().rules()
    else {
        panic!("expected three scoped style rules");
    };
    let selectors = first.selectors().selectors();
    assert!(matches!(selectors[0], CssScopedStyleSelector::Selector(_)));
    assert!(matches!(selectors[1], CssScopedStyleSelector::Relative(_)));
    let CssScopedStyleSelector::Relative(relative) = &selectors[1] else {
        panic!("expected relative selector");
    };
    assert_eq!(relative.combinator(), CssSelectorCombinator::Child);

    let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] =
        anchored.selectors().selectors()
    else {
        panic!("expected authored scope anchor selector");
    };
    assert!(selector.has_scope_anchor());
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Hover]);

    let [CssScopedStyleSelector::Selector(CssSelector::Complex(selector))] =
        scope_pseudo.selectors().selectors()
    else {
        panic!("expected authored :scope complex selector");
    };
    assert_eq!(selector.first().pseudo_classes(), &[CssPseudoClass::Scope]);
}

#[test]
fn scope_rule_parser_accepts_pseudo_elements_in_scoped_style_rules() {
    let sheet = parse_sheet("@scope (.card) { .label::before { color: red; } }")
        .expect("clean parse report");
    let scope = scope_rule(&sheet.rules()[0]);
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("expected scoped style rule");
    };
    let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] =
        style.selectors().selectors()
    else {
        panic!("expected compound scoped style selector");
    };

    assert_eq!(selector.classes(), &["label".to_owned()]);
    assert_pseudo_elements(
        selector.pseudo_elements().unwrap(),
        &[CssPseudoElement::Before],
    );
}

#[test]
fn scope_rule_parser_accepts_limit_only_and_empty_prelude_scope() {
    let sheet = parse_sheet(
        r#"
            @scope to (.stop) { .title { color: black; } }
            @scope { > .item { color: red; } }
        "#,
    )
    .expect("clean parse report");

    let limit_only = scope_rule(&sheet.rules()[0]);
    assert!(limit_only.root().is_none());
    assert_eq!(limit_only.limit().unwrap().selectors().len(), 1);

    let anonymous = scope_rule(&sheet.rules()[1]);
    assert!(anonymous.root().is_none());
    assert!(anonymous.limit().is_none());
    let [CssScopedRule::Style(rule)] = anonymous.rules().rules() else {
        panic!("expected scoped style rule");
    };
    assert!(matches!(
        rule.selectors().selectors(),
        [CssScopedStyleSelector::Relative(_)]
    ));
}

#[test]
fn scope_rule_parser_keeps_nested_scoped_group_rules_scoped() {
    let sheet = parse_sheet(
        r#"
            @scope (.card) {
                @media screen {
                    @container (inline-size > 30rem) {
                        @layer theme {
                            @scope (.inner) {
                                > .label { color: black; }
                            }
                        }
                    }
                }
                @layer reset;
            }
        "#,
    )
    .expect("clean parse report");

    let scope = scope_rule(&sheet.rules()[0]);
    let [
        CssScopedRule::Media(media),
        CssScopedRule::LayerStatement(_),
    ] = scope.rules().rules()
    else {
        panic!("expected scoped media and layer statement");
    };
    let [CssScopedRule::Container(container)] = media.rules().rules() else {
        panic!("expected scoped container");
    };
    let [CssScopedRule::LayerBlock(layer)] = container.rules().rules() else {
        panic!("expected scoped layer block");
    };
    let [CssScopedRule::Scope(nested_scope)] = layer.rules().rules() else {
        panic!("expected nested scope rule");
    };
    let [CssScopedRule::Style(style)] = nested_scope.rules().rules() else {
        panic!("expected scoped relative style rule");
    };
    assert!(matches!(
        style.selectors().selectors(),
        [CssScopedStyleSelector::Relative(_)]
    ));
}

#[test]
fn public_api_exposes_layer_scope_and_scoped_rule_structure() {
    let sheet = parse_sheet(
        r#"
            @layer reset, theme.components;
            @layer theme {
                .button:hover::before { content: "x"; }
                .panel > dialog::backdrop { color: black; }
            }
            @scope (.card, [data-scope]) to (.stop) {
                .title, > .action { color: black; }
                @media screen {
                    @layer components {
                        > .icon::after { content: counter(section); }
                    }
                }
                @layer scoped.order;
            }
        "#,
    )
    .expect("clean parse report");

    let [
        CssRule::LayerStatement(statement),
        CssRule::LayerBlock(layer),
        CssRule::Scope(scope),
    ] = sheet.rules()
    else {
        panic!("expected layer statement, layer block, and scope rule");
    };

    let layer_names = statement.names().names();
    assert_eq!(layer_names.len(), 2);
    assert_eq!(layer_names[0].components(), &["reset".to_owned()]);
    assert_eq!(
        layer_names[1].components(),
        &["theme".to_owned(), "components".to_owned()]
    );
    assert_eq!(layer.name().unwrap().components(), &["theme".to_owned()]);

    let [CssRule::Style(before_rule), CssRule::Style(backdrop_rule)] = layer.rules() else {
        panic!("expected ordinary style rules inside layer block");
    };
    let CssSelector::Compound(before_selector) = before_rule.selectors().selectors()[0].selector()
    else {
        panic!("expected compound pseudo-element selector");
    };
    assert_eq!(before_selector.pseudo_classes(), &[CssPseudoClass::Hover]);
    assert_pseudo_elements(
        before_selector.pseudo_elements().unwrap(),
        &[CssPseudoElement::Before],
    );

    let CssSelector::Complex(backdrop_selector) =
        backdrop_rule.selectors().selectors()[0].selector()
    else {
        panic!("expected complex pseudo-element selector");
    };
    let [backdrop_part] = backdrop_selector.rest() else {
        panic!("expected one complex selector part");
    };
    assert_eq!(backdrop_part.combinator(), CssSelectorCombinator::Child);
    assert_pseudo_elements(
        backdrop_part.selector().pseudo_elements().unwrap(),
        &[CssPseudoElement::Backdrop],
    );

    assert_eq!(scope.root().unwrap().selectors().len(), 2);
    assert_eq!(scope.limit().unwrap().selectors().len(), 1);
    let [
        CssScopedRule::Style(scoped_style),
        CssScopedRule::Media(scoped_media),
        CssScopedRule::LayerStatement(scoped_layer_statement),
    ] = scope.rules().rules()
    else {
        panic!("expected scoped style, scoped media, and scoped layer statement");
    };

    let scoped_selectors = scoped_style.selectors().selectors();
    assert!(matches!(
        scoped_selectors[0],
        CssScopedStyleSelector::Selector(_)
    ));
    let CssScopedStyleSelector::Relative(relative_action) = &scoped_selectors[1] else {
        panic!("expected relative scoped selector");
    };
    assert_eq!(relative_action.combinator(), CssSelectorCombinator::Child);

    assert_eq!(
        scoped_layer_statement.names().names()[0].components(),
        &["scoped".to_owned(), "order".to_owned()]
    );
    let [CssScopedRule::LayerBlock(scoped_layer)] = scoped_media.rules().rules() else {
        panic!("expected scoped layer nested inside scoped media");
    };
    let [CssScopedRule::Style(nested_scoped_style)] = scoped_layer.rules().rules() else {
        panic!("expected scoped style rule inside scoped layer");
    };
    let [CssScopedStyleSelector::Relative(relative_icon)] =
        nested_scoped_style.selectors().selectors()
    else {
        panic!("expected nested scoped-only relative selector");
    };
    assert_eq!(relative_icon.combinator(), CssSelectorCombinator::Child);
    let CssSelector::Compound(icon_selector) = relative_icon.selector() else {
        panic!("expected compound selector after relative combinator");
    };
    assert_eq!(icon_selector.classes(), &["icon".to_owned()]);
    assert_pseudo_elements(
        icon_selector.pseudo_elements().unwrap(),
        &[CssPseudoElement::After],
    );
}

#[test]
fn nested_style_rule_parser_accepts_layer_and_scope_groups() {
    let sheet = parse_sheet(
        r#"
            .card {
                color: black;
                @layer components { & > .title { color: red; } }
                @scope (.title) { > .icon { color: blue; } }
            }
        "#,
    )
    .expect("clean parse report");

    let [CssRule::Style(parent)] = sheet.rules() else {
        panic!("expected authored parent");
    };
    let [layer, scope] = parent.rules() else {
        panic!("expected nested layer and scope");
    };
    let layer = layer_block_rule(layer);
    assert!(matches!(layer.rules(), [CssRule::Style(_)]));
    let scope = scope_rule(scope);
    assert!(matches!(scope.rules().rules(), [CssScopedRule::Style(_)]));
}

#[test]
fn scope_rule_parser_rejects_malformed_scope_syntax() {
    for (css, rejection) in [
        (
            "@scope .card { .title { color: black; } }",
            Rejection::prelude("scope", Some(".")),
        ),
        (
            "@scope (.card) to { .title { color: black; } }",
            Rejection::prelude("scope", Some("to")),
        ),
        (
            "@scope to { .title { color: black; } }",
            Rejection::prelude("scope", Some("to")),
        ),
        (
            "@scope () { .title { color: black; } }",
            Rejection::prelude("scope", Some(")")),
        ),
        (
            "@scope (.card,) { .title { color: black; } }",
            Rejection::prelude("scope", Some(")")),
        ),
        (
            "@scope (.card) to (.stop) extra { .title { color: black; } }",
            Rejection::prelude("scope", Some("extra")),
        ),
        (
            "@scope (.card::before) { .title { color: black; } }",
            Rejection::prelude("scope", Some("before")),
        ),
        (
            "@scope (.card) to (.stop::after) { .title { color: black; } }",
            Rejection::prelude("scope", Some("after")),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn scope_rule_parser_retains_global_font_definition() {
    let sheet =
        parse_sheet("@scope (.card) { @font-face { font-family: Test; src: local(Test); } }")
            .expect("clean parse report");
    let [CssRule::Scope(scope)] = sheet.rules() else {
        panic!("scope")
    };
    let [CssScopedRule::FontFace(font)] = scope.rules().rules() else {
        panic!("font definition")
    };
    assert_eq!(face_value!(font.descriptors(), FontFamily).as_str(), "Test");
    assert!(
        matches!(face_value!(font.descriptors(), Src).sources(), [CssFontFaceSource::Local(name)] if name.as_str() == "Test")
    );
    let with_neighbors = parse_sheet("@scope (.card) { .ok { color: black; } @font-face { font-family: Test; src: local(Test); } } .after { color: blue; }").expect("clean parse report");
    let [CssRule::Scope(scope), CssRule::Style(after)] = with_neighbors.rules() else {
        panic!("scope and later style")
    };
    let [CssScopedRule::Style(before), CssScopedRule::FontFace(font)] = scope.rules().rules()
    else {
        panic!("style and global font definition")
    };
    assert!(
        matches!(before.selectors().selectors(), [CssScopedStyleSelector::Selector(CssSelector::Class(name))] if name == "ok")
    );
    assert!(
        matches!(after.selectors().selectors()[0].selector(), CssSelector::Class(name) if name == "after")
    );
    assert_eq!(face_value!(font.descriptors(), FontFamily).as_str(), "Test");
    assert!(
        matches!(face_value!(font.descriptors(), Src).sources(), [CssFontFaceSource::Local(name)] if name.as_str() == "Test")
    );
}

#[test]
fn relative_selectors_stay_rejected_outside_scoped_blocks() {
    assert_rejection("> .label { color: black; }", Rejection::selector(Some(">")));
    assert_rejection(
        "@media screen { > .label { color: black; } }",
        Rejection::selector(Some(">")),
    );
}
