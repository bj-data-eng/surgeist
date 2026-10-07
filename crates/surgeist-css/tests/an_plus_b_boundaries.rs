#![forbid(unsafe_code)]
//! Independent CSS Syntax 3 CRD 2021-12-24 §§6, 6.2 and 10.1 expectations.
//! Coefficients and canonical strings below are derived from those clauses.
//! The index oracle interprets the public symbolic pair; it is not a production
//! selector matcher. Endpoint cases require only representable final i32 pairs.
//! <https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/>

use surgeist_css::{
    CssComponentValueRef, CssNamespaceContext, CssNthAnPlusB, CssNthPattern, CssNumericTokenKind,
    CssPseudoClass, CssPseudoSelectorListItem, CssRecoveryAction, CssRule, CssSelector,
    CssStyleSelector, CssValueTokenRef, parse_component_values, parse_selector, parse_sheet,
};

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().as_ref().expect("admitted selector").clone()
}

fn pattern(selector: &CssSelector) -> CssNthPattern {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::NthChild(value))
        | CssSelector::PseudoClass(CssPseudoClass::NthLastChild(value)) => value.pattern(),
        CssSelector::PseudoClass(CssPseudoClass::NthOfType(value))
        | CssSelector::PseudoClass(CssPseudoClass::NthLastOfType(value)) => *value,
        _ => panic!("one typed nth pseudo-class: {selector:?}"),
    }
}

fn coefficients(pattern: CssNthPattern) -> (i32, i32) {
    match pattern {
        CssNthPattern::Odd => (2, 1),
        CssNthPattern::Even => (2, 0),
        CssNthPattern::Integer(b) => (0, b),
        CssNthPattern::AnPlusB(value) => (value.a(), value.b()),
        _ => panic!("unexpected nth pattern: {pattern:?}"),
    }
}

fn assert_pattern(argument: &str, expected: (i32, i32), canonical: &str) {
    let source = format!(":nth-child({argument})");
    let selector = parsed(&source);
    assert_eq!(coefficients(pattern(&selector)), expected, "{source}");
    let before = selector.clone();
    let output = selector.to_specified_css().unwrap();
    assert_eq!(output, format!(":nth-child({canonical})"), "{source}");
    assert_eq!(
        selector, before,
        "serialization preserves the authored model"
    );
    assert_eq!(
        coefficients(pattern(&parsed(&output))),
        expected,
        "{source}"
    );
}

fn assert_rejected(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    assert!(!report.is_clean(), "{source}: {report:?}");
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::RejectInput }),
        "{source}: {report:?}"
    );
}

#[test]
fn each_token_production_maps_to_literal_signed_coefficients_and_canonical_text() {
    // §6.2 production rows, including optional leading '+' on n-ident forms.
    for (argument, a, b, canonical) in [
        ("odd", 2, 1, "2n+1"),
        ("EVEN", 2, 0, "2n"),
        ("5", 0, 5, "5"),
        ("+5", 0, 5, "5"),
        ("-5", 0, -5, "-5"),
        ("2n", 2, 0, "2n"),
        ("+2n", 2, 0, "2n"),
        ("-2N", -2, 0, "-2n"),
        ("n", 1, 0, "n"),
        ("+N", 1, 0, "n"),
        ("-n", -1, 0, "-n"),
        ("2n-2", 2, -2, "2n-2"),
        ("-2n-2", -2, -2, "-2n-2"),
        ("n-2", 1, -2, "n-2"),
        ("+n-2", 1, -2, "n-2"),
        ("-n-2", -1, -2, "-n-2"),
        ("2n +2", 2, 2, "2n+2"),
        ("2n -2", 2, -2, "2n-2"),
        ("n +2", 1, 2, "n+2"),
        ("n -2", 1, -2, "n-2"),
        ("+n -2", 1, -2, "n-2"),
        ("-n +2", -1, 2, "-n+2"),
        ("2n- 2", 2, -2, "2n-2"),
        ("n- 2", 1, -2, "n-2"),
        ("+n- 2", 1, -2, "n-2"),
        ("-n- 2", -1, -2, "-n-2"),
        ("2n + 2", 2, 2, "2n+2"),
        ("2n - 2", 2, -2, "2n-2"),
        ("n + 2", 1, 2, "n+2"),
        ("+n - 2", 1, -2, "n-2"),
        ("-n - 2", -1, -2, "-n-2"),
        ("2N-2", 2, -2, "2n-2"),
        (r"2\6e -2", 2, -2, "2n-2"),
        (r"\6e -2", 1, -2, "n-2"),
        (r"-\4e -2", -1, -2, "-n-2"),
        (r"n-\32 ", 1, -2, "n-2"),
    ] {
        assert_pattern(argument, (a, b), canonical);
    }
}

#[test]
fn only_plus_before_n_ident_requires_no_intervening_whitespace() {
    for argument in ["+n", "+/**/n", "+/**//**/N"] {
        assert_pattern(argument, (1, 0), "n");
    }
    for argument in ["+ n", "+\tn", "+\nn", "+/**/ n", "+ /**/n"] {
        assert_rejected(&format!(":nth-child({argument})"));
    }
    // All other token boundaries may contain whitespace or comments.
    for (argument, a, b, canonical) in [
        (" 2n \t+\r\n 3 ", 2, 3, "2n+3"),
        ("2n/**/+/**/3", 2, 3, "2n+3"),
        ("n- \t3", 1, -3, "n-3"),
        ("-n \t-\n 3", -1, -3, "-n-3"),
        ("+n- /**/3", 1, -3, "n-3"),
    ] {
        assert_pattern(argument, (a, b), canonical);
    }
    // Whitespace cannot synthesize a signed number/dimension token.
    for argument in ["+ 2n", "+ 2", "- 2", "2 n"] {
        assert_rejected(&format!(":nth-child({argument})"));
    }
}

#[test]
fn lexical_integer_flag_and_source_sign_remain_distinct_from_numeric_value() {
    for (source, expected_kind, expected_sign, expected_unit) in [
        ("2n", CssNumericTokenKind::Integer, false, Some("n")),
        ("+2n", CssNumericTokenKind::Integer, true, Some("n")),
        ("-2n", CssNumericTokenKind::Integer, true, Some("n")),
        ("2.0n", CssNumericTokenKind::Number, false, Some("n")),
        ("2e0n", CssNumericTokenKind::Number, false, Some("n")),
        ("2", CssNumericTokenKind::Integer, false, None),
        ("+2", CssNumericTokenKind::Integer, true, None),
        ("-2", CssNumericTokenKind::Integer, true, None),
        ("2.0", CssNumericTokenKind::Number, false, None),
        ("2e0", CssNumericTokenKind::Number, false, None),
    ] {
        let components = parse_component_values(source).unwrap();
        let [component] = components.items() else {
            panic!("one lexical token: {source}");
        };
        let (numeric, unit) = match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => (number, None),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                (number, Some(unit))
            }
            value => panic!("number or dimension: {source}: {value:?}"),
        };
        assert_eq!(numeric.kind(), expected_kind, "{source}");
        assert_eq!(numeric.has_sign(), expected_sign, "{source}");
        assert_eq!(unit, expected_unit, "{source}");
        let expected_repr = source.strip_suffix('n').unwrap_or(source);
        assert_eq!(numeric.representation(), expected_repr, "{source}");
    }
    assert_pattern("n +2", (1, 2), "n+2");
    assert_pattern("n + 2", (1, 2), "n+2");
    for argument in ["2.0n", "2e0n", "2.0", "2e0", "n 2", "n + +2"] {
        assert_rejected(&format!(":nth-child({argument})"));
    }
}

// Independent §6 interpretation over the parsed public pair. For each positive
// index, solve index = A*n+B and require integral n >= 0; i64 avoids i32 overflow.
fn positive_indexes((a, b): (i32, i32), last: i32) -> Vec<i32> {
    let a = i64::from(a);
    let b = i64::from(b);
    (1..=last)
        .filter(|index| {
            let difference = i64::from(*index) - b;
            if a == 0 {
                difference == 0
            } else {
                difference % a == 0 && difference / a >= 0
            }
        })
        .collect()
}

#[test]
fn public_pairs_interpret_only_positive_one_based_indexes_for_nonnegative_n() {
    let cases: &[(&str, (i32, i32), &[i32])] = &[
        ("odd", (2, 1), &[1, 3, 5, 7, 9, 11]),
        ("even", (2, 0), &[2, 4, 6, 8, 10, 12]),
        ("0n+0", (0, 0), &[]),
        ("0n+5", (0, 5), &[5]),
        ("-3", (0, -3), &[]),
        ("-n+3", (-1, 3), &[1, 2, 3]),
        ("-4n+10", (-4, 10), &[2, 6, 10]),
        ("2n-1", (2, -1), &[1, 3, 5, 7, 9, 11]),
        ("3n+2", (3, 2), &[2, 5, 8, 11]),
        ("-n", (-1, 0), &[]),
    ];
    for (argument, expected_pair, expected_indexes) in cases {
        let selector = parsed(&format!(":nth-child({argument})"));
        let pair = coefficients(pattern(&selector));
        assert_eq!(pair, *expected_pair, "{argument}");
        assert_eq!(
            positive_indexes(pair, 12).as_slice(),
            *expected_indexes,
            "{argument}"
        );
    }
}

#[test]
fn typed_coefficients_cover_every_canonical_output_branch_and_i32_endpoint() {
    for (a, b, canonical) in [
        (0, 0, "0"),
        (0, 2, "2"),
        (0, -2, "-2"),
        (1, 0, "n"),
        (1, 2, "n+2"),
        (1, -2, "n-2"),
        (-1, 0, "-n"),
        (-1, 2, "-n+2"),
        (-1, -2, "-n-2"),
        (2, 0, "2n"),
        (2, 2, "2n+2"),
        (2, -2, "2n-2"),
        (-2, 0, "-2n"),
        (-2, 2, "-2n+2"),
        (-2, -2, "-2n-2"),
        (0, i32::MIN, "-2147483648"),
        (0, i32::MAX, "2147483647"),
        (1, i32::MIN, "n-2147483648"),
        (-1, i32::MAX, "-n+2147483647"),
        (i32::MIN, 0, "-2147483648n"),
        (i32::MAX, 0, "2147483647n"),
        (i32::MIN, i32::MIN, "-2147483648n-2147483648"),
        (i32::MAX, i32::MAX, "2147483647n+2147483647"),
    ] {
        let selector = CssSelector::PseudoClass(CssPseudoClass::NthOfType(CssNthPattern::AnPlusB(
            CssNthAnPlusB::new(a, b),
        )));
        let before = selector.clone();
        let output = selector.to_specified_css().unwrap();
        assert_eq!(output, format!(":nth-of-type({canonical})"));
        assert_eq!(selector, before);
        assert_eq!(coefficients(pattern(&parsed(&output))), (a, b));
    }
}

#[test]
fn explicit_minus_and_signless_offset_retain_the_representable_i32_lower_bound() {
    // Candidate probe from §6.2: negate the full signless value 2147483648.
    // Final B is representable. Neither source inspection nor this draft is RED.
    for (argument, a, canonical) in [
        ("n - 2147483648", 1, "n-2147483648"),
        ("n- 2147483648", 1, "n-2147483648"),
        ("+n - 2147483648", 1, "n-2147483648"),
        ("+n- 2147483648", 1, "n-2147483648"),
        ("-n - 2147483648", -1, "-n-2147483648"),
        ("2n - 2147483648", 2, "2n-2147483648"),
        ("2n- 2147483648", 2, "2n-2147483648"),
    ] {
        assert_pattern(argument, (a, i32::MIN), canonical);
    }
    // Adjacent in-range controls distinguish offset interpretation from syntax.
    assert_pattern("n - 2147483647", (1, -2147483647), "n-2147483647");
    assert_pattern("n + 2147483647", (1, i32::MAX), "n+2147483647");
}

#[test]
fn signed_integer_offsets_keep_the_same_representable_i32_lower_bound() {
    for (argument, a, canonical) in [
        ("n -2147483648", 1, "n-2147483648"),
        ("+n -2147483648", 1, "n-2147483648"),
        ("-n -2147483648", -1, "-n-2147483648"),
        ("2n -2147483648", 2, "2n-2147483648"),
    ] {
        assert_pattern(argument, (a, i32::MIN), canonical);
    }
}

#[test]
fn embedded_negative_offsets_keep_the_same_representable_i32_lower_bound() {
    for (argument, a, canonical) in [
        ("n-2147483648", 1, "n-2147483648"),
        ("+n-2147483648", 1, "n-2147483648"),
        ("-n-2147483648", -1, "-n-2147483648"),
        ("2n-2147483648", 2, "2n-2147483648"),
    ] {
        assert_pattern(argument, (a, i32::MIN), canonical);
    }
}

#[test]
fn invalid_token_alternatives_and_trailing_argument_garbage_are_rejected_atomically() {
    for argument in [
        "",
        "n 2",
        "n + +2",
        "n - -2",
        "n + -2",
        "n - +2",
        "n- +2",
        "n- -2",
        "n + 2.0",
        "n + 2e0",
        "2.0n-2",
        "2e0n-2",
        "n--2",
        "n-²",
        "n +",
        "n-",
        "odd + 2",
        "even 2",
        "n 2px",
        "n + 2%",
        "n + 2 garbage",
        "n, 2",
        r"n \+2",
        r"n + \32 ",
    ] {
        assert_rejected(&format!(":nth-child({argument})"));
    }
    for source in [
        ":nth-of-type(n of .a)",
        ":nth-last-of-type(n of .a)",
        ":nth-child(n of)",
        ":nth-last-child(n of)",
    ] {
        assert_rejected(source);
    }
    // A valid retry after the rejected inputs retains the accepted grammar.
    assert_pattern("-n + 3", (-1, 3), "-n+3");
}

#[test]
fn all_nth_wrappers_keep_coefficients_and_only_child_wrappers_admit_of_lists() {
    for name in [
        "nth-child",
        "nth-last-child",
        "nth-of-type",
        "nth-last-of-type",
    ] {
        let selector = parsed(&format!(":{name}(-2n + 3)"));
        assert_eq!(coefficients(pattern(&selector)), (-2, 3));
        assert_eq!(
            selector.to_specified_css().unwrap(),
            format!(":{name}(-2n+3)")
        );
    }
    for name in ["nth-child", "nth-last-child"] {
        let selector = parsed(&format!(":{name}(odd of .a,#b)"));
        let CssSelector::PseudoClass(pseudo) = &selector else {
            panic!("nth child pseudo-class");
        };
        let child = match pseudo {
            CssPseudoClass::NthChild(value) | CssPseudoClass::NthLastChild(value) => value,
            _ => panic!("child-indexed wrapper"),
        };
        assert_eq!(coefficients(child.pattern()), (2, 1));
        let [
            CssPseudoSelectorListItem::Selector(first),
            CssPseudoSelectorListItem::Selector(second),
        ] = child.selector_list().expect("of-list").items()
        else {
            panic!("two preserved selector members");
        };
        assert_eq!(first, &CssSelector::Class("a".into()));
        assert_eq!(second, &CssSelector::Key("b".into()));
        assert_eq!(
            selector.to_specified_css().unwrap(),
            format!(":{name}(2n+1 of .a, #b)")
        );
    }
}

#[test]
fn invalid_nth_outer_rule_drops_locally_and_preserves_a_later_sibling() {
    for argument in ["+ n", "n + -2", "2.0n", "n + 2 garbage"] {
        let source = format!(":nth-child({argument}){{}} .after{{}}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropQualifiedRule }),
            "{source}: {report:?}"
        );
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("only the later style rule survives: {report:?}");
        };
        let [CssStyleSelector::Selector(selector)] = rule.selectors().selectors() else {
            panic!("one retained later selector");
        };
        assert_eq!(selector.to_specified_css().unwrap(), ".after");
    }
}
