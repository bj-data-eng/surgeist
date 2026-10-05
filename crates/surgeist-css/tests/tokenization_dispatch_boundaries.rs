#![forbid(unsafe_code)]
//! CSS Syntax 3 CRD (2021-12-24), §§3.3, 4.3.1, 4.3.3 and 4.3.9:
//! a hyphen followed by backslash-newline does not start an identifier. The
//! at-keyword and dimension dispatch branches must leave that hyphen for the
//! next token. Split components retain their genuine authored byte ranges and
//! count separately toward admission limits. Literal NUL is preprocessed to
//! U+FFFD before identifier lookahead.
//! <https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/>

use std::ops::Range;

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef,
    CssHashFlag, CssRule, CssSupportsTestItem, CssValueOrigin, CssValueTokenRef,
    parse_component_values, parse_component_values_with_limits, parse_sheet, parse_style_attribute,
};

fn assert_span(origin: &CssValueOrigin, source: &str, expected: Range<usize>) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("each authored token owns its original parsed range");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), expected.start);
    assert_eq!(origin.span().end().byte_offset().value(), expected.end);
}

fn assert_delimiter(component: &CssComponentValue, expected: char) {
    assert!(
        matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)) if actual == expected),
        "expected delimiter {expected:?}: {:?}",
        component.view()
    );
}

fn assert_split(
    components: &[CssComponentValue],
    source: &str,
    start: usize,
    prefix: &str,
    newline: &str,
) {
    let [first, hyphen, backslash, whitespace] = components else {
        panic!("four independently consumed tokens: {source:?}: {components:?}");
    };
    if prefix == "@" {
        assert_delimiter(first, '@');
    } else {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = first.view() else {
            panic!(
                "the numeric prefix is a number, not a dimension: {:?}",
                first.view()
            );
        };
        assert_eq!(number.representation(), prefix);
    }
    assert_delimiter(hyphen, '-');
    assert_delimiter(backslash, '\\');
    assert!(
        matches!(whitespace.view(), CssComponentValueRef::Token(CssValueTokenRef::Whitespace(actual)) if actual == newline)
    );
    let prefix_end = start + prefix.len();
    assert_span(first.origin(), source, start..prefix_end);
    assert_span(hyphen.origin(), source, prefix_end..prefix_end + 1);
    assert_span(backslash.origin(), source, prefix_end + 1..prefix_end + 2);
    assert_span(
        whitespace.origin(),
        source,
        prefix_end + 2..prefix_end + 2 + newline.len(),
    );
}

#[test]
fn invalid_at_keyword_escape_splits_delimiters_with_original_spans() {
    for newline in ["\n", "\r", "\r\n", "\u{c}"] {
        let source = format!("@-\\{newline}");
        let values = parse_component_values(&source).expect("delimiter sequence is representable");
        assert_eq!(values.component_count(), 4);
        assert_split(values.items(), &source, 0, "@", newline);
    }
}

#[test]
fn invalid_dimension_escape_splits_exact_signed_and_exponent_numbers() {
    for number in ["1", "+1", "-1", ".5", "-.5", "1e2", "-1.5e+2"] {
        for newline in ["\n", "\r", "\r\n", "\u{c}"] {
            let source = format!("{number}-\\{newline}");
            let values =
                parse_component_values(&source).expect("number and delimiters are representable");
            assert_eq!(values.component_count(), 4);
            assert_split(values.items(), &source, 0, number, newline);
        }
    }
}

#[test]
fn function_children_split_dispatch_tokens_and_keep_original_utf16_positions() {
    let source = "😀 f(@-\\\r\n)";
    let values = parse_component_values(source).expect("complete function");
    assert_eq!(values.component_count(), 7);
    assert_eq!(values.nesting_depth(), 1);
    let CssComponentValueRef::Function(function) = values.items()[2].view() else {
        panic!("the final component is f()");
    };
    assert_eq!(function.name(), "f");
    assert_split(function.values().items(), source, 7, "@", "\r\n");
    let CssValueOrigin::Parsed(at) = function.values().items()[0].origin() else {
        panic!("original @ origin");
    };
    assert_eq!(at.span().start().line().value(), 0);
    assert_eq!(at.span().start().column().value(), 5);
    assert_eq!(at.span().end().column().value(), 6);
    let CssValueOrigin::Parsed(newline) = function.values().items()[3].origin() else {
        panic!("original CRLF origin");
    };
    assert_eq!(newline.span().start().column().value(), 8);
    assert_eq!(newline.span().end().line().value(), 1);
    assert_eq!(newline.span().end().column().value(), 0);
}

#[test]
fn nested_blocks_split_dimension_dispatch_without_spending_extra_depth() {
    let source = "[f(-1.5e+2-\\\n)]";
    let values = parse_component_values(source).expect("complete nested blocks");
    assert_eq!(values.component_count(), 6);
    assert_eq!(values.nesting_depth(), 2);
    let CssComponentValueRef::Block(block) = values.items()[0].view() else {
        panic!("square block");
    };
    let CssComponentValueRef::Function(function) = block.values().items()[0].view() else {
        panic!("nested f()");
    };
    assert_split(function.values().items(), source, 3, "-1.5e+2", "\n");
}

#[test]
fn split_at_keyword_components_count_separately_toward_the_limit() {
    let source = "@-\\\n";
    let failure = parse_component_values_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 3, 100).unwrap(),
    )
    .expect_err("the fourth whitespace token exceeds a three-component budget");
    assert_eq!(failure.kind(), CssComponentValueErrorKind::ComponentLimit);
    assert_span(failure.origin(), source, 3..4);
    let admitted = parse_component_values_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 4, 100).unwrap(),
    )
    .unwrap();
    assert_eq!(admitted.component_count(), 4);
}

#[test]
fn split_dimension_components_count_separately_toward_the_limit() {
    let source = "1e2-\\\r\n";
    let failure = parse_component_values_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 3, 100).unwrap(),
    )
    .expect_err("the fourth whitespace token exceeds a three-component budget");
    assert_eq!(failure.kind(), CssComponentValueErrorKind::ComponentLimit);
    assert_span(failure.origin(), source, 5..7);
    let admitted = parse_component_values_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 4, 100).unwrap(),
    )
    .unwrap();
    assert_eq!(admitted.component_count(), 4);
}

#[test]
fn a_one_component_limit_reports_the_first_split_hyphen_range() {
    for (source, hyphen) in [("@-\\\n", 1..2), ("1e2-\\\n", 3..4)] {
        let failure = parse_component_values_with_limits(
            source,
            CssComponentValueLimits::try_new(0, 1, 100).unwrap(),
        )
        .unwrap_err();
        assert_eq!(failure.kind(), CssComponentValueErrorKind::ComponentLimit);
        assert_span(failure.origin(), source, hyphen);
    }
}

#[test]
fn authored_declaration_components_preserve_both_split_dispatch_branches() {
    for prefix in ["@", "1e2"] {
        let source = format!("--x:{prefix}-\\\n;--after:kept");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2);
        let declaration = &report.syntax()[0];
        assert_eq!(declaration.custom().unwrap().name().as_str(), "--x");
        assert_split(
            declaration.value_components().items(),
            &source,
            4,
            prefix,
            "\n",
        );
        assert_eq!(
            report.syntax()[1].custom().unwrap().name().as_str(),
            "--after"
        );
    }
}

#[test]
fn recovering_named_test_collector_preserves_split_value_prefixes() {
    for prefix in ["@", "1e2"] {
        let source = format!("@supports-condition --probe{{mystery:{prefix}-\\\n;}}");
        let report = parse_sheet(&source);
        let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
            panic!("retained named test: {report:?}");
        };
        let [CssSupportsTestItem::Declarations(run)] = rule.body().items() else {
            panic!("one generic declaration run");
        };
        let [declaration] = run.declarations() else {
            panic!("one generic declaration candidate");
        };
        assert_eq!(declaration.property(), "mystery");
        let value = declaration.value_components();
        assert!(value.len() >= 3);
        if prefix == "@" {
            assert_delimiter(&value[0], '@');
        } else {
            assert!(
                matches!(value[0].view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == prefix)
            );
        }
        assert_delimiter(&value[1], '-');
        assert_delimiter(&value[2], '\\');
        let start = "@supports-condition --probe{mystery:".len();
        assert_span(value[0].origin(), &source, start..start + prefix.len());
        assert_span(
            value[1].origin(),
            &source,
            start + prefix.len()..start + prefix.len() + 1,
        );
        assert_span(
            value[2].origin(),
            &source,
            start + prefix.len() + 1..start + prefix.len() + 2,
        );
    }
}

#[test]
fn eof_escapes_still_start_at_keywords_and_dimension_units() {
    for (source, unit) in [("@-\\", "-\u{fffd}"), ("1-\\", "-\u{fffd}")] {
        let values = parse_component_values(source).unwrap();
        let [component] = values.items() else {
            panic!("EOF escape keeps one token");
        };
        match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::AtKeyword(actual))
                if source.starts_with('@') =>
            {
                assert_eq!(actual, unit)
            }
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number,
                unit: actual,
            }) if source.starts_with('1') => {
                assert_eq!(number.representation(), "1");
                assert_eq!(actual, unit);
            }
            actual => panic!("valid EOF escape lookahead: {actual:?}"),
        }
        assert_span(component.origin(), source, 0..source.len());
    }
}

#[test]
fn escaped_hyphen_names_are_not_split_after_decoding() {
    for source in [r"@\2d ", r"@\00002d "] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(matches!(
            values.items()[0].view(),
            CssComponentValueRef::Token(CssValueTokenRef::AtKeyword("-"))
        ));
        assert_span(values.items()[0].origin(), source, 0..source.len());
    }
    for source in [r"1\2d ", r"1\00002d "] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(
            matches!(values.items()[0].view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit: "-" }) if number.representation() == "1")
        );
        assert_span(values.items()[0].origin(), source, 0..source.len());
    }
}

#[test]
fn nul_preprocessing_preserves_identifier_start_dispatch() {
    for (source, decoded) in [("@\0", "\u{fffd}"), ("@-\0", "-\u{fffd}")] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(
            matches!(values.items()[0].view(), CssComponentValueRef::Token(CssValueTokenRef::AtKeyword(actual)) if actual == decoded)
        );
        assert_span(values.items()[0].origin(), source, 0..source.len());
    }
    for (source, decoded) in [("1\0", "\u{fffd}"), ("1-\0", "-\u{fffd}")] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(
            matches!(values.items()[0].view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) if number.representation() == "1" && unit == decoded)
        );
        assert_span(values.items()[0].origin(), source, 0..source.len());
    }
    for (source, decoded) in [("#\0", "\u{fffd}"), ("#-\0", "-\u{fffd}")] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(
            matches!(values.items()[0].view(), CssComponentValueRef::Token(CssValueTokenRef::Hash { value, flag: CssHashFlag::Id }) if value == decoded)
        );
        assert_span(values.items()[0].origin(), source, 0..source.len());
    }
}

#[test]
fn normal_hyphen_names_keep_at_keyword_and_dimension_identity() {
    for (source, expected) in [("@--", "--"), ("@-name", "-name"), (r"@-\61 ", "-a")] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(matches!(
            values.items()[0].view(),
            CssComponentValueRef::Token(CssValueTokenRef::AtKeyword(actual)) if actual == expected
        ));
    }
    for (source, expected) in [("1--", "--"), ("1-name", "-name"), (r"1-\61 ", "-a")] {
        let values = parse_component_values(source).unwrap();
        assert_eq!(values.component_count(), 1);
        assert!(matches!(
            values.items()[0].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                if number.representation() == "1" && unit == expected
        ));
    }
}
