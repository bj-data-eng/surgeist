//! Conditional Rules 3 section 3 permits page rules in conditional groups.
//! Syntax 3 (2026 CRD) §5.5.2 consumes every at-rule block with §5.5.4
//! "consume a block", whose §5.5.5 "consume a block's contents" discards a
//! `<semicolon-token>` and tries a declaration before a qualified rule that has
//! `<semicolon-token>` as its stop token. A top-level conditional group's
//! `<rule-list>` grammar rejects every declaration, so that attempt is
//! reconsumed as a qualified rule which ends, invalid, at its `;`.
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#contents-of
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-at-rule
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-qualified-rule
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#syntax
use crate::css::support::rules::page_pseudo;
use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssErrorCode, CssImportance, CssKnownProperty,
    CssKnownPropertyValueRef, CssMarginValue, CssPagePseudo, CssRecoveryAction, CssRule,
    CssSelector, CssValueTokenRef, parse_sheet, validate_sheet,
};

fn assert_tag(rule: &CssRule, expected: &str) {
    let CssRule::Style(style) = rule else {
        panic!("expected style rule {expected}: {rule:?}")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Tag(expected.into()));
}

#[test]
fn empty_page_rule_is_valid_inside_media_rule_list() {
    let report = parse_sheet("@media all{@page{}}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media rule")
    };
    let [CssRule::Page(page)] = media.rules() else {
        panic!("one page child: {media:?}")
    };
    assert_eq!(page_pseudo(page), None);
    assert!(page.declarations().properties().is_empty());
}

#[test]
fn nested_page_retains_selector_typed_margin_and_importance() {
    let report = parse_sheet("@media all{a{}@page :left{margin: 1px !important}b{}}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media rule")
    };
    let [a, CssRule::Page(page), b] = media.rules() else {
        panic!("ordered style/page/style: {media:?}")
    };
    assert_tag(a, "a");
    assert_tag(b, "b");
    assert_eq!(page_pseudo(page), Some(CssPagePseudo::Left));
    let [declaration] = page.declarations().properties().as_slice() else {
        panic!("one margin")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::Margin);
    let CssKnownPropertyValueRef::Margin(margin) = known.property_value().unwrap() else {
        panic!("typed page margin")
    };
    for edge in margin.value().assigned_values() {
        let CssMarginValue::LengthPercentage(value) = edge else {
            panic!("exact page margin length")
        };
        let Some(CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })) =
            value.literal_component().map(CssComponentValue::view)
        else {
            panic!("exact pixel token")
        };
        assert_eq!((number.representation(), unit), ("1", "px"));
    }
}

#[test]
fn original_nested_page_recovers_its_declaration_without_dropping_page() {
    let report = parse_sheet("@media test {\n    @page {\n        p: v;\n    }\n}");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("media")
    };
    let [CssRule::Page(page)] = media.rules() else {
        panic!("retained page")
    };
    assert_eq!(page_pseudo(page), None);
    assert!(page.declarations().properties().is_empty());
    let actions: Vec<_> = report.diagnostics().iter().map(|d| d.action()).collect();
    assert_eq!(actions, [CssRecoveryAction::DropDeclaration]);
    assert!(!report.is_clean());
}

fn group_children(rule: &CssRule) -> &[CssRule] {
    match rule {
        CssRule::Media(media) => media.rules(),
        CssRule::Supports(supports) => supports.rules(),
        CssRule::Container(container) => container.rules(),
        CssRule::LayerBlock(layer) => layer.rules(),
        _ => panic!("expected a conditional or layer group: {rule:?}"),
    }
}

/// Asserts one top-level group whose children are the expected style rules,
/// and whose ordered diagnostics are exactly the dropped `(unit, action)`s.
fn assert_block_contents(
    source: &str,
    expected_tags: &[&str],
    dropped: &[(&str, CssRecoveryAction)],
) {
    let report = parse_sheet(source);
    let [group] = report.syntax().rules() else {
        panic!("one group: {source}: {report:?}")
    };
    let children = group_children(group);
    assert_eq!(children.len(), expected_tags.len(), "{source}: {report:?}");
    for (rule, tag) in children.iter().zip(expected_tags) {
        assert_tag(rule, tag);
    }
    assert_eq!(
        report.diagnostics().len(),
        dropped.len(),
        "{source}: {report:?}"
    );
    let mut searched = 0;
    for (diagnostic, (unit, action)) in report.diagnostics().iter().zip(dropped) {
        assert_eq!(diagnostic.action(), *action, "{source}: {unit}");
        let start = searched + source[searched..].find(unit).unwrap();
        searched = start + unit.len();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            start,
            "{source}: {unit}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len(),
            "{source}: {unit}"
        );
    }
    assert_eq!(report.is_clean(), dropped.is_empty(), "{source}");
    assert_eq!(
        report.into_validation_result().is_ok(),
        dropped.is_empty(),
        "{source}"
    );
}

// CSSTree tolerant "correct fallback on error in at-rule block/1": the `;` is
// discarded by consume a block's contents, so `b{}` is an ordinary sibling.
#[test]
fn semicolon_between_media_rules_is_discarded_and_keeps_the_next_rule() {
    assert_block_contents("@media (max-width:1000px){a{};b{}}", &["a", "b"], &[]);
}

// CSSTree tolerant "/2": after the discarded `;`, `@b{}` is consumed as its own
// nested at-rule; it is unknown, so only that at-rule is dropped.
#[test]
fn semicolon_before_an_at_keyword_leaves_an_independent_at_rule() {
    assert_block_contents(
        "@media (max-width:1000px){a{};@b{}}",
        &["a"],
        &[("@b{}", CssRecoveryAction::DropAtRule)],
    );
}

#[test]
fn block_contents_recovery_preserves_following_siblings_and_diagnostic_order() {
    assert_block_contents(
        "@media all{a{};b{}c{};@b{}d{}}",
        &["a", "b", "c", "d"],
        &[("@b{}", CssRecoveryAction::DropAtRule)],
    );
}

#[test]
fn semicolons_before_between_and_after_rules_are_discarded_in_every_group() {
    // Each group's block is consumed by consume a block's contents.
    for prefix in [
        "@media screen",
        "@supports (display:block)",
        "@container (width > 1px)",
        "@layer base",
    ] {
        for (body, tags) in [
            ("; lost {}", &["lost"][..]),
            (";;a{}", &["a"]),
            ("a{};b{}", &["a", "b"]),
            ("a{} ; ; b{}", &["a", "b"]),
            ("a{};", &["a"]),
            ("a{};;", &["a"]),
            (";", &[]),
        ] {
            assert_block_contents(&format!("{prefix} {{ {body} }}"), tags, &[]);
        }
    }
}

#[test]
fn declarations_in_a_top_level_media_rule_list_are_dropped_before_a_rule() {
    // `color: red` is not valid in @media's <rule-list>, so it is reconsumed as
    // a qualified rule that reaches its `;` stop token and returns nothing.
    assert_block_contents(
        "@media screen { color: red; a {} }",
        &["a"],
        &[("color: red", CssRecoveryAction::DropQualifiedRule)],
    );
    // CSSTree tolerant "correct fallback on error in at-rule block/0".
    let source = "@media (max-width:1000px){top:0;left:0;width:100%;}*{color:red}";
    let report = parse_sheet(source);
    let [media, CssRule::Style(after)] = report.syntax().rules() else {
        panic!("media and following style: {report:?}")
    };
    assert!(group_children(media).is_empty(), "{report:?}");
    assert_eq!(after.declarations().len(), 1);
    let units: Vec<_> = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
            &source[diagnostic.span().start().byte_offset().value()
                ..diagnostic.span().end().byte_offset().value()]
        })
        .collect();
    assert_eq!(units, ["top:0", "left:0", "width:100%"]);
}

#[test]
fn a_qualified_rule_ending_at_a_semicolon_drops_only_its_prelude() {
    assert_block_contents(
        "@media all{a{};b;c{}}",
        &["a", "c"],
        &[("b", CssRecoveryAction::DropQualifiedRule)],
    );
    assert_block_contents(
        "@supports (display:block){x y;z{}}",
        &["z"],
        &[("x y", CssRecoveryAction::DropQualifiedRule)],
    );
}

#[test]
fn custom_property_shaped_rule_consumes_the_remnants_of_a_bad_declaration() {
    // Nested consume a qualified rule meets `{` after `--x:`, so it consumes
    // the remnants of a bad declaration up to the next `;` or block end.
    assert_block_contents(
        "@media all{--x:y{}lost{};kept{}}",
        &["kept"],
        &[("--x:y{}lost{}", CssRecoveryAction::DropQualifiedRule)],
    );
    // At the stylesheet top level the same prelude only consumes its block.
    let report = parse_sheet("--x:y{}.kept{}");
    let [CssRule::Style(kept)] = report.syntax().rules() else {
        panic!("top-level custom-property-shaped rule drops only its block: {report:?}")
    };
    assert_eq!(
        kept.selectors().selectors()[0].selector(),
        &CssSelector::Class("kept".into())
    );
}

#[test]
fn top_level_custom_property_shaped_rule_is_dropped_even_as_a_valid_selector() {
    // Syntax 3 (2026-10-01) §5.5.3 checks the first two non-whitespace prelude
    // values before any selector grammar: `--foo:hover` and `--foo :hover`
    // would be valid selectors, but a top-level rule consumes its block and
    // returns nothing.
    for dropped in ["--foo:hover{color:red}", "--foo :hover{color:red}"] {
        let source = format!("{dropped}.kept{{color:blue}}");
        let report = parse_sheet(&source);
        let [CssRule::Style(kept)] = report.syntax().rules() else {
            panic!("{source}: {report:?}")
        };
        assert_eq!(
            kept.selectors().selectors()[0].selector(),
            &CssSelector::Class("kept".into())
        );
        assert_eq!(kept.declarations().len(), 1);
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: {report:?}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), dropped.len());
        assert!(validate_sheet(&source).is_err());
    }
    // An ordinary ident prelude remains a style rule.
    let report = parse_sheet("foo:hover{color:red}");
    assert!(report.is_clean(), "{report:?}");
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
}
#[test]
fn style_body_semicolons_remain_valid_declaration_separators() {
    let report = parse_sheet("a{;;margin:1px;;;}");
    assert!(report.is_clean(), "{report:?}");
    let [rule] = report.syntax().rules() else {
        panic!("one style")
    };
    assert_tag(rule, "a");
    let CssRule::Style(style) = rule else {
        unreachable!()
    };
    let [declaration] = style.declarations().as_slice() else {
        panic!("one declaration")
    };
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Margin
    );
}

#[test]
fn original_page_between_style_children_preserves_all_rule_and_recovery_order() {
    let source = "x {\n    p:v;\n}\n\n@media test {\n    a {\n        p:v;\n    }\n\n    /* comment */\n\n    @page {\n        p: v;\n    }\n\n    b {\n        p:v;\n    }\n}\n\ny {\n    p:v;\n}";
    let report = parse_sheet(source);
    let [x, CssRule::Media(media), y] = report.syntax().rules() else {
        panic!("outer style/media/style: {report:?}")
    };
    assert_tag(x, "x");
    assert_tag(y, "y");
    let [a, CssRule::Page(page), b] = media.rules() else {
        panic!("inner style/page/style: {media:?}")
    };
    assert_tag(a, "a");
    assert_tag(b, "b");
    assert_eq!(page_pseudo(page), None);
    assert!(page.declarations().properties().is_empty());
    assert_eq!(report.diagnostics().len(), 5);
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    }
    // Each failed p declaration belongs to its original owning rule, in order.
    let positions: Vec<_> = report
        .diagnostics()
        .iter()
        .map(|d| d.span().start().byte_offset().value())
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!report.is_clean());
}

#[test]
fn page_is_retained_through_nested_media_and_supports_rule_lists() {
    let report = parse_sheet("@media print{@supports (display:block){@page :right{}}}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media: {report:?}")
    };
    let [CssRule::Supports(supports)] = media.rules() else {
        panic!("one supports: {media:?}")
    };
    let [CssRule::Page(page)] = supports.rules() else {
        panic!("one page: {supports:?}")
    };
    assert_eq!(page_pseudo(page), Some(CssPagePseudo::Right));
    assert!(page.declarations().properties().is_empty());
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn invalid_page_prelude_and_import_drop_only_their_rules_inside_media() {
    let source = "@media print{a{}@page :unknown{}b{}@import 'late.css';c{}}";
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media: {report:?}")
    };
    let [a, b, c] = media.rules() else {
        panic!("three surviving siblings: {media:?}")
    };
    assert_tag(a, "a");
    assert_tag(b, "b");
    assert_tag(c, "c");
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, (unit, code)) in report.diagnostics().iter().zip([
        ("@page :unknown{}", CssErrorCode::InvalidAtRulePrelude),
        ("@import 'late.css';", CssErrorCode::InvalidAtRulePlacement),
    ]) {
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(diagnostic.error().code(), code);
        let start = source.find(unit).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
    }
    assert!(report.into_validation_result().is_err());
}

#[test]
fn nested_page_and_declaration_preserve_authored_unicode_coordinates() {
    let source = "/*😀*/\n@media print{/*😀*/@page :left{/*😀*/margin:1px;}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media: {report:?}")
    };
    let [CssRule::Page(page)] = media.rules() else {
        panic!("one page: {media:?}")
    };
    let [declaration] = page.declarations().properties().as_slice() else {
        panic!("one declaration: {page:?}")
    };
    for (position, token) in [
        (page.position(), "@page"),
        (declaration.position().unwrap(), "margin"),
    ] {
        let offset = source.find(token).unwrap();
        assert_eq!(position.byte_offset().value(), offset);
        assert_eq!(position.line().value(), 1);
        assert_eq!(
            position.column().value() as usize,
            source[source.find('\n').unwrap() + 1..offset]
                .encode_utf16()
                .count()
        );
    }
}

#[test]
fn discarded_semicolon_keeps_comments_unicode_and_balanced_block_siblings() {
    let source = "/*😀*/\n@media all{a{};/*😀*/b{--x:'}';}c{}}";
    assert_block_contents(source, &["a", "b", "c"], &[]);
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media: {report:?}")
    };
    let CssRule::Style(b) = &media.rules()[1] else {
        panic!("b style: {media:?}")
    };
    let offset = source.find("b{").unwrap();
    assert_eq!(b.position().byte_offset().value(), offset);
    assert_eq!(b.position().line().value(), 1);
    assert_eq!(
        b.position().column().value() as usize,
        source[source.find('\n').unwrap() + 1..offset]
            .encode_utf16()
            .count()
    );
    let [declaration] = b.declarations().as_slice() else {
        panic!("one custom declaration: {b:?}")
    };
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        "'}'"
    );
}

#[test]
fn semicolon_in_supports_block_contents_keeps_every_sibling() {
    assert_block_contents(
        "@supports (display:block){a{};b{}c{}}",
        &["a", "b", "c"],
        &[],
    );
}

#[test]
fn style_nested_conditional_body_keeps_semicolon_declaration_separators() {
    let report = parse_sheet("a{@media all{;;margin:1px;;;}}");
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Style(style)] = report.syntax().rules() else {
        panic!("one style: {report:?}")
    };
    let [CssRule::Media(media)] = style.rules() else {
        panic!("one media: {style:?}")
    };
    let [CssRule::NestedDeclarations(declarations)] = media.rules() else {
        panic!("one declaration run: {media:?}")
    };
    let [declaration] = declarations.declarations().as_slice() else {
        panic!("one declaration: {declarations:?}")
    };
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Margin
    );
}

#[test]
fn repeated_semicolons_are_discarded_and_a_missing_block_drops_only_its_prelude() {
    assert_block_contents("@media all{a{};;b{}c{}}", &["a", "b", "c"], &[]);
    // Nested consume a qualified rule returns nothing at the block's `}`.
    assert_block_contents(
        "@media all{a{};b}",
        &["a"],
        &[("b", CssRecoveryAction::DropQualifiedRule)],
    );
}

#[test]
fn nested_block_contents_do_not_discard_top_level_only_html_comment_tokens() {
    // Only consume a stylesheet's contents discards CDO/CDC; in a block they
    // begin a qualified rule after the failed declaration attempt.
    assert_block_contents(
        "@media all{a{}<!--b{}c{}}",
        &["a", "c"],
        &[("<!--b{}", CssRecoveryAction::DropQualifiedRule)],
    );
    assert_block_contents(
        "@media all{a{}-->b{}c{}}",
        &["a", "c"],
        &[("-->b{}", CssRecoveryAction::DropQualifiedRule)],
    );
}

#[test]
fn nested_charset_is_recovered_without_losing_style_siblings() {
    let source = "@media all{a{}@charset \"utf-8\";b{}}";
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media: {report:?}")
    };
    let [a, b] = media.rules() else {
        panic!("two siblings: {media:?}")
    };
    assert_tag(a, "a");
    assert_tag(b, "b");
    let [diagnostic] = report.diagnostics() else {
        panic!("one charset recovery: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("@charset").unwrap()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find("b{}").unwrap()
    );
    assert!(report.into_validation_result().is_err());
}

#[test]
fn layer_and_container_block_contents_retain_pages_and_discard_semicolons() {
    // These grouping blocks use rule-list grammar when there is no style
    // ancestor, after consume a block's contents has discarded the `;`.
    for prefix in ["@layer print", "@container (width > 1px)"] {
        let source = format!("{prefix}{{a{{}}@page :first{{}};b{{}}c{{}}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{report:?}");
        let [group] = report.syntax().rules() else {
            panic!("one group: {report:?}")
        };
        let [a, CssRule::Page(page), b, c] = group_children(group) else {
            panic!("ordered style/page/style/style: {group:?}")
        };
        assert_tag(a, "a");
        assert_tag(b, "b");
        assert_tag(c, "c");
        assert_eq!(page_pseudo(page), Some(CssPagePseudo::First));
        assert!(report.into_validation_result().is_ok());
    }
}
