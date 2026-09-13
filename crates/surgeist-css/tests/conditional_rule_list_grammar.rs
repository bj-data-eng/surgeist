#![forbid(unsafe_code)]
//! Conditional Rules 3 section 3 permits page rules in conditional groups.
//! Syntax 3 section 5.4.1 consumes a rule list: a semicolon starts a qualified
//! rule prelude rather than being discarded as in a style declaration body.
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#contents-of
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-list-of-rules
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#syntax
use surgeist_css::{
    CssErrorCode, CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssLength,
    CssPageSelector, CssRecoveryAction, CssRule, CssSelector, parse_sheet,
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
    assert_eq!(page.selector(), None);
    assert!(page.declarations().is_empty());
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
    assert_eq!(page.selector(), Some(CssPageSelector::Left));
    let [declaration] = page.declarations().as_slice() else {
        panic!("one margin")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::Margin);
    let CssKnownPropertyValueRef::Margin(margin) = known.property_value().unwrap() else {
        panic!("typed page margin")
    };
    let edges = margin.i01_subset().unwrap();
    for edge in [&edges.top, &edges.right, &edges.bottom, &edges.left] {
        assert!(matches!(edge, CssLength::Px(value) if value.value() == 1.0));
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
    assert_eq!(page.selector(), None);
    assert!(page.declarations().is_empty());
    let actions: Vec<_> = report.diagnostics().iter().map(|d| d.action()).collect();
    assert_eq!(actions, [CssRecoveryAction::DropDeclaration]);
    assert!(!report.is_clean());
}

fn assert_semicolon_recovery(source: &str, expected_tags: &[&str], discarded: &[&str]) {
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one media: {report:?}")
    };
    assert_eq!(
        media.rules().len(),
        expected_tags.len(),
        "{source}: {report:?}"
    );
    for (rule, tag) in media.rules().iter().zip(expected_tags) {
        assert_tag(rule, tag);
    }
    assert!(!report.is_clean(), "{source}");
    assert_eq!(
        report.diagnostics().len(),
        discarded.len(),
        "{source}: {report:?}"
    );
    for (diagnostic, unit) in report.diagnostics().iter().zip(discarded) {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropQualifiedRule,
            "{source}"
        );
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
fn original_semicolon_prefixed_qualified_rule_is_discarded() {
    assert_semicolon_recovery("@media (max-width:1000px){a{};b{}}", &["a"], &[";b{}"]);
}
#[test]
fn original_semicolon_prefixed_at_keyword_belongs_to_discarded_qualified_rule() {
    assert_semicolon_recovery("@media (max-width:1000px){a{};@b{}}", &["a"], &[";@b{}"]);
}
#[test]
fn rule_list_recovery_preserves_following_siblings_and_diagnostic_order() {
    assert_semicolon_recovery(
        "@media all{a{};b{}c{};@b{}d{}}",
        &["a", "c", "d"],
        &[";b{}", ";@b{}"],
    );
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
    assert_eq!(page.selector(), None);
    assert!(page.declarations().is_empty());
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
    assert_eq!(page.selector(), Some(CssPageSelector::Right));
    assert!(page.declarations().is_empty());
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
    let [declaration] = page.declarations().as_slice() else {
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
fn semicolon_recovery_span_preserves_comments_unicode_and_balanced_blocks() {
    assert_semicolon_recovery(
        "/*😀*/\n@media all{a{};/*😀*/b{--x:'}';}c{}}",
        &["a", "c"],
        &[";/*😀*/b{--x:'}';}"],
    );
}

#[test]
fn semicolon_in_supports_rule_list_discards_next_qualified_rule() {
    let source = "@supports (display:block){a{};b{}c{}}";
    let report = parse_sheet(source);
    let [CssRule::Supports(supports)] = report.syntax().rules() else {
        panic!("one supports: {report:?}")
    };
    let [a, c] = supports.rules() else {
        panic!("two surviving siblings: {supports:?}")
    };
    assert_tag(a, "a");
    assert_tag(c, "c");
    let [diagnostic] = report.diagnostics() else {
        panic!("one recovery: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find(';').unwrap()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find("c{}").unwrap()
    );
    assert!(report.into_validation_result().is_err());
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
fn repeated_semicolons_and_missing_block_remain_qualified_rule_recovery_units() {
    assert_semicolon_recovery("@media all{a{};;b{}c{}}", &["a", "c"], &[";;b{}"]);
    assert_semicolon_recovery("@media all{a{};b}", &["a"], &[";b"]);
}

#[test]
fn nested_rule_lists_do_not_discard_top_level_only_html_comment_tokens() {
    assert_semicolon_recovery("@media all{a{}<!--b{}c{}}", &["a", "c"], &["<!--b{}"]);
    assert_semicolon_recovery("@media all{a{}-->b{}c{}}", &["a", "c"], &["-->b{}"]);
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
fn layer_and_container_rule_lists_retain_pages_and_recover_semicolon_preludes() {
    // These grouping blocks use rule-list grammar when there is no style ancestor.
    for prefix in ["@layer print", "@container (width > 1px)"] {
        let source = format!("{prefix}{{a{{}}@page :first{{}};b{{}}c{{}}}}");
        let report = parse_sheet(&source);
        let [group] = report.syntax().rules() else {
            panic!("one group: {report:?}")
        };
        let rules = match group {
            CssRule::LayerBlock(layer) => layer.rules(),
            CssRule::Container(container) => container.rules(),
            _ => panic!("expected layer or container: {group:?}"),
        };
        let [a, CssRule::Page(page), c] = rules else {
            panic!("ordered style/page/style: {rules:?}")
        };
        assert_tag(a, "a");
        assert_tag(c, "c");
        assert_eq!(page.selector(), Some(CssPageSelector::First));
        let [diagnostic] = report.diagnostics() else {
            panic!("one semicolon recovery: {report:?}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find(';').unwrap()
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find("c{}").unwrap()
        );
        assert!(report.into_validation_result().is_err());
    }
}
