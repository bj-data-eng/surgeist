#![forbid(unsafe_code)]
//! Dated CSS Syntax 3 (2021-12-24) section 4.3.1's fallback consumes one
//! delimiter code point. Thus ~ =, | =, ^ =, $ = and * = are pairs of Delims,
//! even when their authored code points are adjacent. The public component
//! contract preserves consumed spelling, origins and exact-one construction.
//! Selectors 4's attribute matcher semantics remain a separate grammar.
//! <https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-token>

use std::ops::Range;

use surgeist_css::{
    CssAttributeCaseSensitivity, CssAttributeMatcher, CssBlockKind, CssComponentValue,
    CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssErrorCode, CssNamespaceContext, CssParsedOrigin, CssRecoveryAction, CssSelector,
    CssSerializedOrigin, CssTokenKind, CssValueOrigin, CssValueTokenRef, ErrorKind,
    parse_component_values, parse_component_values_with_limits, parse_selector,
};

const OPERATORS: [(char, &str); 5] = [
    ('~', "~="),
    ('|', "|="),
    ('^', "^="),
    ('$', "$="),
    ('*', "*="),
];

fn parsed(origin: &CssValueOrigin) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("authored token keeps its actual parsed origin")
    };
    origin
}

fn assert_span(origin: &CssValueOrigin, source: &str, range: Range<usize>) {
    let origin = parsed(origin);
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

fn assert_delim(component: &CssComponentValue, expected: char, source: &str, start: usize) {
    assert!(
        matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)) if actual == expected),
        "{source:?}: expected delimiter {expected:?}, got {:?}",
        component.view()
    );
    assert_span(component.origin(), source, start..start + 1);
    assert_eq!(&source[start..start + 1], expected.to_string());
    let singleton = CssComponentValues::try_new(vec![component.clone()]).unwrap();
    assert_eq!(
        singleton.serialize().unwrap().as_css(),
        expected.to_string()
    );
}

fn assert_pair(items: &[CssComponentValue], prefix: char, source: &str, start: usize) {
    let [first, equals] = items else {
        panic!("{source:?}: exactly two independently consumed delimiter components")
    };
    assert_delim(first, prefix, source, start);
    assert_delim(equals, '=', source, start + 1);
    assert!(
        parsed(first.origin())
            .source()
            .same_snapshot(parsed(equals.origin()).source())
    );
}

fn source_pair_and_nested_order(prefix: char, operator: &str) {
    let values = parse_component_values(operator).unwrap();
    assert_pair(values.items(), prefix, operator, 0);
    assert_eq!(values.component_count(), 2);
    assert_eq!(values.nesting_depth(), 0);

    let source = format!("[f({operator})]tail");
    let nested = parse_component_values(&source).unwrap();
    let [group, tail] = nested.items() else {
        panic!("nested group then original sibling")
    };
    let CssComponentValueRef::Block(block) = group.view() else {
        panic!("square group")
    };
    assert_eq!(block.kind(), CssBlockKind::SquareBracket);
    let [function] = block.values().items() else {
        panic!("one nested function")
    };
    let CssComponentValueRef::Function(function) = function.view() else {
        panic!("f function")
    };
    assert_eq!(function.name(), "f");
    assert_pair(function.values().items(), prefix, &source, 3);
    assert_span(function.closing_origin(), &source, 5..6);
    assert_span(block.closing_origin(), &source, 6..7);
    assert!(matches!(
        tail.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("tail"))
    ));
    assert_span(tail.origin(), &source, 7..11);
    assert_eq!(nested.component_count(), 5);
    assert_eq!(nested.nesting_depth(), 2);

    let repeated = format!("{operator}=");
    let repeated_values = parse_component_values(&repeated).unwrap();
    let [first, second, third] = repeated_values.items() else {
        panic!("three ordered delimiters")
    };
    assert_delim(first, prefix, &repeated, 0);
    assert_delim(second, '=', &repeated, 1);
    assert_delim(third, '=', &repeated, 2);

    let following = format!("😀{operator}x");
    let following_values = parse_component_values(&following).unwrap();
    let [emoji, first, second, last] = following_values.items() else {
        panic!("ident, delimiter pair, ident")
    };
    assert!(matches!(
        emoji.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("😀"))
    ));
    assert_span(emoji.origin(), &following, 0..4);
    assert_pair(&[first.clone(), second.clone()], prefix, &following, 4);
    assert_eq!(parsed(first.origin()).span().start().column().value(), 2);
    assert_eq!(parsed(second.origin()).span().start().column().value(), 3);
    assert!(matches!(
        last.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("x"))
    ));
    assert_span(last.origin(), &following, 6..7);
}

macro_rules! source_operator_test {
    ($name:ident, $prefix:literal, $operator:literal) => {
        #[test]
        fn $name() {
            source_pair_and_nested_order($prefix, $operator);
        }
    };
}

source_operator_test!(adjacent_tilde_equals_keeps_two_delimiters, '~', "~=");
source_operator_test!(adjacent_bar_equals_keeps_two_delimiters, '|', "|=");
source_operator_test!(adjacent_caret_equals_keeps_two_delimiters, '^', "^=");
source_operator_test!(adjacent_dollar_equals_keeps_two_delimiters, '$', "$=");
source_operator_test!(adjacent_asterisk_equals_keeps_two_delimiters, '*', "*=");

#[test]
fn exact_one_token_constructor_rejects_operator_pairs_but_admits_each_delimiter() {
    for (prefix, operator) in OPERATORS {
        let failure = CssComponentValue::try_token(operator).unwrap_err();
        assert_eq!(
            failure.kind(),
            CssComponentValueErrorKind::InvalidToken,
            "{operator}"
        );
        assert_eq!(failure.origin(), &CssValueOrigin::Programmatic);
        for character in [prefix, '='] {
            let token = CssComponentValue::try_token(&character.to_string()).unwrap();
            assert!(
                matches!(token.view(), CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)) if actual == character)
            );
            assert_eq!(token.origin(), &CssValueOrigin::Programmatic);
        }
    }
}

#[test]
fn component_and_input_byte_limits_distinguish_second_delimiter_from_whole_input() {
    for (prefix, operator) in OPERATORS {
        let exact = CssComponentValueLimits::try_new(0, 2, 2).unwrap();
        let admitted = parse_component_values_with_limits(operator, exact).unwrap();
        assert_pair(admitted.items(), prefix, operator, 0);
        let failure = parse_component_values_with_limits(
            operator,
            CssComponentValueLimits::try_new(0, 1, 2).unwrap(),
        )
        .unwrap_err();
        assert_eq!(failure.kind(), CssComponentValueErrorKind::ComponentLimit);
        assert_span(failure.origin(), operator, 1..2);
        let failure = parse_component_values_with_limits(
            operator,
            CssComponentValueLimits::try_new(0, 2, 1).unwrap(),
        )
        .unwrap_err();
        assert_eq!(failure.kind(), CssComponentValueErrorKind::ByteLimit);
        assert_eq!(
            failure.origin(),
            &CssValueOrigin::UnretainedInput { byte_length: 2 }
        );
        assert_pair(
            parse_component_values_with_limits(operator, exact)
                .unwrap()
                .items(),
            prefix,
            operator,
            0,
        );
    }
}

fn assert_pair_output(values: &CssComponentValues, operator: &str) {
    let before = values.clone();
    let output = values.serialize_with_limit(2).unwrap();
    assert_eq!(output.as_css(), operator);
    assert_eq!(output.segments().len(), 2);
    for (index, (segment, component)) in output.segments().iter().zip(values.items()).enumerate() {
        assert_eq!(segment.byte_range(), index..index + 1);
        let CssSerializedOrigin::Token(origin) = segment.origin() else {
            panic!("actual delimiter bytes, without a generated separator")
        };
        assert_eq!(origin, component.origin());
        if let CssValueOrigin::Parsed(expected) = component.origin() {
            assert!(parsed(origin).source().same_snapshot(expected.source()));
        }
        assert_eq!(output.origin_at(index), Some(segment.origin()));
    }
    let CssSerializedOrigin::End(Some(end)) = output.origin_at(2).unwrap() else {
        panic!("EOF anchored to the second delimiter")
    };
    assert_eq!(end, values.items()[1].origin());
    let failure = values.serialize_with_limit(1).unwrap_err();
    assert_eq!(failure.kind(), CssComponentValueErrorKind::ByteLimit);
    assert_eq!(failure.origin(), values.items()[1].origin());
    assert_eq!(
        values, &before,
        "rejected output cannot mutate admitted components"
    );
    assert_eq!(values.serialize_with_limit(2).unwrap().as_css(), operator);
}

#[test]
fn programmatic_delimiter_pairs_serialize_without_obsolete_operator_separators() {
    for (prefix, operator) in OPERATORS {
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_token(&prefix.to_string()).unwrap(),
            CssComponentValue::try_token("=").unwrap(),
        ])
        .unwrap();
        assert_eq!(values.component_count(), 2);
        assert_pair_output(&values, operator);
    }
}

#[test]
fn source_delimiter_pairs_serialize_two_bytes_with_their_original_per_item_origins() {
    for (prefix, operator) in OPERATORS {
        let values = parse_component_values(operator).unwrap();
        assert_pair(values.items(), prefix, operator, 0);
        assert_pair_output(&values, operator);
    }
}

#[test]
fn genuine_identifier_number_and_comment_opening_hazards_still_insert_separators() {
    for (first, second, expected) in [
        (
            CssComponentValue::try_ident("a").unwrap(),
            CssComponentValue::try_ident("b").unwrap(),
            "a/**/b",
        ),
        (
            CssComponentValue::try_number("10").unwrap(),
            CssComponentValue::try_ident("px").unwrap(),
            "10/**/px",
        ),
        (
            CssComponentValue::try_token("/").unwrap(),
            CssComponentValue::try_token("*").unwrap(),
            concat!("/", "/**/", "*"),
        ),
    ] {
        let values = CssComponentValues::try_new(vec![first, second]).unwrap();
        let output = values.serialize().unwrap();
        assert_eq!(output.as_css(), expected);
        let separators: Vec<_> = output
            .segments()
            .iter()
            .filter_map(|segment| match segment.origin() {
                CssSerializedOrigin::Separator { before, after } => Some((segment, before, after)),
                _ => None,
            })
            .collect();
        let [(segment, before, after)] = separators.as_slice() else {
            panic!("one genuine boundary separator")
        };
        assert_eq!(&output.as_css()[segment.byte_range()], "/**/");
        assert_eq!(*before, values.items()[0].origin());
        assert_eq!(*after, values.items()[1].origin());
    }
}

#[test]
fn rejected_operator_selector_identifies_its_first_actual_delimiter() {
    for (prefix, operator) in OPERATORS {
        let report = parse_selector(operator, &CssNamespaceContext::default());
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
        let [diagnostic] = report.diagnostics() else {
            panic!("one whole-input selector rejection")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), 2);
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("selector-domain failure")
        };
        let token = detail
            .encountered()
            .expect("actual first delimiter, rather than missing input");
        assert_eq!(token.kind(), CssTokenKind::Delim);
        assert_eq!(token.authored(), prefix.to_string());
        assert_eq!(token.authored().len(), 1);
    }
}

#[test]
fn attribute_matcher_semantics_accept_adjacent_or_commented_parts_and_reject_whitespace() {
    let context = CssNamespaceContext::default();
    for (prefix, operator, matcher) in [
        ('~', "~=", CssAttributeMatcher::Includes("MiX".into())),
        ('|', "|=", CssAttributeMatcher::DashMatch("MiX".into())),
        ('^', "^=", CssAttributeMatcher::Prefix("MiX".into())),
        ('$', "$=", CssAttributeMatcher::Suffix("MiX".into())),
        ('*', "*=", CssAttributeMatcher::Substring("MiX".into())),
    ] {
        for source in [
            format!("[a{operator}\"MiX\" i]"),
            format!("[a{prefix}/**/=\"MiX\" i]"),
        ] {
            let report = parse_selector(&source, &context);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            let Some(CssSelector::Compound(compound)) = report.syntax() else {
                panic!("one attribute compound")
            };
            let [attribute] = compound.attributes() else {
                panic!("one retained semantic matcher")
            };
            assert_eq!(attribute.name().as_str(), "a");
            assert_eq!(attribute.matcher(), &matcher);
            assert_eq!(
                attribute.case_sensitivity(),
                CssAttributeCaseSensitivity::AsciiCaseInsensitive
            );
        }
        let source = format!("[a{prefix} =\"MiX\"]");
        let report = parse_selector(&source, &context);
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean());
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected whitespace-separated matcher")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    }
}
