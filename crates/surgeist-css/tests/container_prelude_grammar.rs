#![forbid(unsafe_code)]
//! Conditional Rules 5 section 5.4 admits a nonempty comma-separated list of
//! entries, each containing a name, a query, or both. Entries remain independent.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
//! Values 4 section 4.2 defines case-sensitive custom identifiers and exclusions.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#custom-idents
//! CSS Syntax section 2.1 decodes identifier escapes before grammar admission.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#escaping
use surgeist_css::{
    CssContainerName, CssErrorCode, CssRecoveryAction, CssRule, CssScopedRule, parse_sheet,
    validate_sheet,
};

fn assert_accepted(prelude: &str) {
    let source =
        format!(".before {{}} @container {prelude} {{ .first {{}} .second {{}} }} .after {{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {report:?}");
    assert!(validate_sheet(&source).is_ok(), "{source}");
    let [
        CssRule::Style(before),
        CssRule::Container(container),
        CssRule::Style(after),
    ] = report.syntax().rules()
    else {
        panic!("one container between its surviving siblings: {source}: {report:?}");
    };
    assert_eq!(before.position().byte_offset().value(), 0);
    assert_eq!(
        after.position().byte_offset().value(),
        source.find(".after").unwrap()
    );
    let [CssRule::Style(first), CssRule::Style(second)] = container.rules() else {
        panic!("one intact body, without duplication: {source}: {report:?}");
    };
    assert_eq!(
        first.position().byte_offset().value(),
        source.find(".first").unwrap()
    );
    assert_eq!(
        second.position().byte_offset().value(),
        source.find(".second").unwrap()
    );
}

fn assert_rejected(prelude: &str) {
    let source = format!(".before {{}} @container {prelude} {{ .discarded {{}} }} .after {{}}");
    let report = parse_sheet(&source);
    assert!(validate_sheet(&source).is_err(), "{source}");
    let [CssRule::Style(before), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("discard the whole invalid rule and preserve siblings: {source}: {report:?}");
    };
    assert_eq!(before.position().byte_offset().value(), 0);
    assert_eq!(
        after.position().byte_offset().value(),
        source.find(".after").unwrap()
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one prelude diagnostic: {source}: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
}

#[test]
fn name_only_entries_retain_the_container_body() {
    for prelude in ["sidebar", "sidebar, toolbar", "sidebar, sidebar"] {
        assert_accepted(prelude);
    }
}

#[test]
fn comma_entries_accept_independently_named_and_unnamed_queries() {
    for prelude in [
        "card (width > 1px), style(--theme: dark)",
        "(width > 1px), card (height > 2px)",
        "card (width > 1px), card (height > 2px)",
        "sidebar, (width > 1px), toolbar style(--theme)",
        "card (width > 1px), scroll-state(stuck: top)",
        "card /* name */ (width > 1px) /* left */, /* right */ toolbar",
    ] {
        assert_accepted(prelude);
    }
}

#[test]
fn function_spellings_are_valid_literal_container_names() {
    for name in ["style", "STYLE", "scroll-state", "Scroll-State"] {
        let checked = CssContainerName::try_new(name).expect("an unreserved custom identifier");
        assert_eq!(checked.as_str(), name);
        assert_accepted(name);
        let source = format!("@container {name} (width > 1px) {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Container(rule)] = report.syntax().rules() else {
            panic!("one named container: {source}: {report:?}");
        };
        assert_eq!(rule.name().map(CssContainerName::as_str), Some(name));
    }
}

#[test]
fn function_tokens_remain_unnamed_queries() {
    for query in [
        "style(--theme)",
        "STYLE(--theme)",
        "scroll-state(stuck: top)",
    ] {
        let source = format!("@container {query} {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Container(rule)] = report.syntax().rules() else {
            panic!("one unnamed query: {source}: {report:?}");
        };
        assert!(rule.name().is_none(), "a Function token is not a name");
    }
}

#[test]
fn escaped_names_keep_the_decoded_identifier_without_retokenizing() {
    for (authored, decoded) in [
        (r"\31 pane", "1pane"),
        (r"a\ b", "a b"),
        (r"a\,b", "a,b"),
        (r"a\5c b", r"a\b"),
        (r"s\74 yle", "style"),
    ] {
        let source = format!("@container {authored} (width > 1px) {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Container(rule)] = report.syntax().rules() else {
            panic!("one named query: {source}: {report:?}");
        };
        assert_eq!(rule.name().map(CssContainerName::as_str), Some(decoded));
    }
}

#[test]
fn container_names_retain_case_sensitive_identity() {
    for name in ["Card", "card", "--card", "revert-rule"] {
        let source = format!("@container {name} (width > 1px) {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Container(rule)] = report.syntax().rules() else {
            panic!("one named query: {source}: {report:?}");
        };
        assert_eq!(rule.name().map(CssContainerName::as_str), Some(name));
    }
    assert_ne!(
        CssContainerName::try_new("Card"),
        CssContainerName::try_new("card")
    );
}

#[test]
fn default_is_reserved_after_escape_decoding_and_case_folding() {
    for name in ["default", "DeFaUlT", r"d\65 fault", r"\64 efault"] {
        assert_rejected(&format!("{name} (width > 1px)"));
        assert_rejected(name);
    }
}

#[test]
fn checked_literal_names_exclude_default_in_every_ascii_case() {
    for name in ["default", "DEFAULT", "DeFaUlT"] {
        assert_eq!(CssContainerName::try_new(name), None, "{name}");
    }
}

#[test]
fn other_reserved_names_reject_as_names_with_or_without_queries() {
    for name in [
        "none",
        "and",
        "or",
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "NoNe",
        r"n\6f ne",
        r"\69 nitial",
    ] {
        assert_rejected(name);
        assert_rejected(&format!("{name} (width > 1px)"));
    }
    // `not` before an operand is valid query syntax, but never a name-only entry.
    assert_rejected("not");
    assert_rejected("NOT");
    assert_rejected(r"n\6f t");
}

#[test]
fn empty_entries_and_trailing_junk_reject_the_entire_list() {
    for prelude in [
        "",
        "/* empty */",
        ",",
        ", card",
        "card,",
        "card,,sidebar",
        "(width > 1px), , (height > 2px)",
        "(width > 1px), !",
        "card sidebar",
        "card (width > 1px) tail",
        "\"card\"",
        "card, \"sidebar\"",
    ] {
        assert_rejected(prelude);
    }
}

#[test]
fn nested_commas_remain_inside_their_query_operands() {
    for prelude in [
        "style(--tokens: a,b)",
        "style(--tokens: \"a,b\")",
        "future(a,b)",
        "future(inner(a,b), c)",
    ] {
        assert_accepted(prelude);
    }
}

#[test]
fn top_level_commas_do_not_split_nested_or_escaped_commas() {
    for prelude in [
        "style(--tokens: a,b), sidebar",
        "style(--tokens: \"a,b\"), future(c,d)",
        "future(inner(a,b), c), (width > 1px)",
        r"a\,b (width > 1px), sidebar",
    ] {
        assert_accepted(prelude);
    }
}

#[test]
fn name_only_and_list_entries_survive_scoped_rule_parsing() {
    for prelude in ["sidebar", "card (width > 1px), toolbar"] {
        let source = format!(
            "@scope (.host) {{ .before {{}} @container {prelude} {{ .child {{}} }} .after {{}} }}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(validate_sheet(&source).is_ok(), "{source}");
        let [CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("one scope: {source}: {report:?}");
        };
        let [
            CssScopedRule::Style(before),
            CssScopedRule::Container(container),
            CssScopedRule::Style(after),
        ] = scope.rules().rules()
        else {
            panic!("one scoped container between siblings: {source}: {report:?}");
        };
        assert_eq!(
            before.position().byte_offset().value(),
            source.find(".before").unwrap()
        );
        assert_eq!(
            after.position().byte_offset().value(),
            source.find(".after").unwrap()
        );
        let [CssScopedRule::Style(child)] = container.rules().rules() else {
            panic!("one retained scoped child: {source}: {report:?}");
        };
        assert_eq!(
            child.position().byte_offset().value(),
            source.find(".child").unwrap()
        );
    }
}

#[test]
fn name_only_and_list_entries_survive_nested_style_rule_parsing() {
    for prelude in ["sidebar", "card (width > 1px), toolbar"] {
        let source =
            format!(".host {{ .before {{}} @container {prelude} {{ .child {{}} }} .after {{}} }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(validate_sheet(&source).is_ok(), "{source}");
        let [CssRule::Style(host)] = report.syntax().rules() else {
            panic!("one parent style rule: {source}: {report:?}");
        };
        let [
            CssRule::Style(before),
            CssRule::Container(container),
            CssRule::Style(after),
        ] = host.rules()
        else {
            panic!("one nested container between siblings: {source}: {report:?}");
        };
        assert_eq!(
            before.position().byte_offset().value(),
            source.find(".before").unwrap()
        );
        assert_eq!(
            after.position().byte_offset().value(),
            source.find(".after").unwrap()
        );
        let [CssRule::Style(child)] = container.rules() else {
            panic!("one retained nested child: {source}: {report:?}");
        };
        assert_eq!(
            child.position().byte_offset().value(),
            source.find(".child").unwrap()
        );
    }
}

#[test]
fn invalid_list_drops_only_its_scoped_container_rule() {
    let source =
        "@scope (.host) { .before {} @container (width > 1px), ! { .discarded {} } .after {} }";
    let report = parse_sheet(source);
    assert!(validate_sheet(source).is_err());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("one surviving scope: {report:?}");
    };
    let [CssScopedRule::Style(before), CssScopedRule::Style(after)] = scope.rules().rules() else {
        panic!("only the scoped siblings survive: {report:?}");
    };
    assert_eq!(
        before.position().byte_offset().value(),
        source.find(".before").unwrap()
    );
    assert_eq!(
        after.position().byte_offset().value(),
        source.find(".after").unwrap()
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one prelude diagnostic: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
}

#[test]
fn invalid_list_drops_only_its_nested_container_rule() {
    let source = ".host { .before {} @container (width > 1px), ! { .discarded {} } .after {} }";
    let report = parse_sheet(source);
    assert!(validate_sheet(source).is_err());
    let [CssRule::Style(host)] = report.syntax().rules() else {
        panic!("one surviving parent: {report:?}");
    };
    let [CssRule::Style(before), CssRule::Style(after)] = host.rules() else {
        panic!("only the nested siblings survive: {report:?}");
    };
    assert_eq!(
        before.position().byte_offset().value(),
        source.find(".before").unwrap()
    );
    assert_eq!(
        after.position().byte_offset().value(),
        source.find(".after").unwrap()
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one prelude diagnostic: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
}

#[test]
fn existing_boolean_query_grammar_remains_distinct_from_comma_entries() {
    for prelude in [
        "not (width > 1px)",
        "(width > 1px) and (height > 2px)",
        "(width > 1px) or (height > 2px)",
        "((width > 1px) or (height > 2px)) and style(--theme)",
    ] {
        assert_accepted(prelude);
    }
    for prelude in [
        "not not (width > 1px)",
        "not (width > 1px) and (height > 2px)",
        "(width > 1px) and not (height > 2px)",
        "(width > 1px) and (height > 2px) or (inline-size > 3px)",
    ] {
        assert_rejected(prelude);
    }
}
