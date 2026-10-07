#![forbid(unsafe_code)]
//! Authored component construction through public source fronts.
//! Syntax 3 CRD20211224 §§5.3.9–11, 5.4.7–9 and source-entry contracts.
use surgeist_css::{
    CssBlockKind, CssComponentValue, CssComponentValueErrorKind, CssComponentValueLimits,
    CssComponentValueRef, CssErrorCode, CssParsedOrigin, CssRecoveryAction, CssRecoveryDiagnostic,
    CssSourceSpan, CssValueOrigin, CssValueTokenRef, ErrorKind,
    parse_comma_separated_component_values, parse_comma_separated_component_values_with_limits,
    parse_component_value, parse_component_value_with_limits,
};

fn parsed(origin: &CssValueOrigin) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed occurrence")
    };
    origin
}
fn span(span: CssSourceSpan, expected: std::ops::Range<usize>) {
    assert_eq!(span.start().byte_offset().value(), expected.start);
    assert_eq!(span.end().byte_offset().value(), expected.end);
}
fn origin(origin: &CssValueOrigin, source: &str, range: std::ops::Range<usize>) {
    let occurrence = parsed(origin);
    assert_eq!(occurrence.source().as_str(), source);
    span(occurrence.span(), range);
}
fn lexical(
    diagnostic: &CssRecoveryDiagnostic,
    source: &str,
    kind: CssComponentValueErrorKind,
    responsible: std::ops::Range<usize>,
    unit: std::ops::Range<usize>,
) {
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidComponentValue
    );
    let ErrorKind::InvalidComponentValue(detail) = diagnostic.error().kind() else {
        panic!("typed component failure")
    };
    assert_eq!(detail.kind(), kind);
    origin(detail.origin(), source, responsible.clone());
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible.start
    );
    span(diagnostic.span(), unit);
}
fn ident(component: &CssComponentValue, expected: &str) {
    assert!(
        matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(actual)) if actual == expected)
    );
}

#[test]
fn singular_entry_selects_complete_token_after_outer_trivia_with_original_occurrence() {
    let source = " \t/**/ \\61 /*tail*/ ";
    let report = parse_component_value(source);
    assert!(report.is_clean(), "{report:?}");
    let component = report.syntax().as_ref().unwrap();
    ident(component, "a");
    origin(component.origin(), source, 7..11);
    assert!(report.into_validation_result().is_ok());
}

#[test]
fn singular_entry_returns_one_complete_function_or_each_matching_block_kind() {
    for (source, kind) in [
        (" /**/ [a,b] ", CssBlockKind::SquareBracket),
        (" /**/ (a,b) ", CssBlockKind::Parenthesis),
        (" /**/ {a,b} ", CssBlockKind::CurlyBracket),
    ] {
        let report = parse_component_value(source);
        assert!(report.is_clean(), "{report:?}");
        let component = report.syntax().as_ref().unwrap();
        let CssComponentValueRef::Block(block) = component.view() else {
            panic!("whole block")
        };
        assert_eq!(block.kind(), kind);
        assert_eq!(block.values().items().len(), 3);
        assert!(matches!(
            block.values().items()[1].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Comma)
        ));
        origin(component.origin(), source, 6..7);
        origin(block.closing_origin(), source, 10..11);
        assert!(
            parsed(component.origin())
                .source()
                .same_snapshot(parsed(block.closing_origin()).source())
        );
    }
    let source = " /*c*/ f([a,b],{c,d}) ";
    let report = parse_component_value(source);
    assert!(report.is_clean(), "{report:?}");
    let component = report.syntax().as_ref().unwrap();
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("whole function")
    };
    assert_eq!(function.name(), "f");
    assert_eq!(function.values().items().len(), 3);
    assert!(matches!(
        function.values().items()[0].view(),
        CssComponentValueRef::Block(_)
    ));
    assert!(matches!(
        function.values().items()[2].view(),
        CssComponentValueRef::Block(_)
    ));
    origin(component.origin(), source, 7..9);
    origin(function.closing_origin(), source, 20..21);
}

#[test]
fn singular_empty_or_trivia_only_input_rejects_at_real_eof() {
    for source in ["", " \t\n", " /**/ "] {
        let report = parse_component_value(source);
        assert!(report.syntax().is_none());
        let [diagnostic] = report.diagnostics() else {
            panic!("one missing-component fault")
        };
        lexical(
            diagnostic,
            source,
            CssComponentValueErrorKind::EmptyInput,
            source.len()..source.len(),
            0..source.len(),
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn singular_extra_component_rejects_the_whole_fragment_at_first_extra_component() {
    for (source, extra) in [(" a /**/ f(x) ", 8..10), ("[x] {y}", 4..5), ("a,b", 1..2)] {
        let report = parse_component_value(source);
        assert!(
            report.syntax().is_none(),
            "no partial first component: {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one trailing-input fault")
        };
        lexical(
            diagnostic,
            source,
            CssComponentValueErrorKind::TrailingInput,
            extra,
            0..source.len(),
        );
    }
}

#[test]
fn singular_checked_admission_rejects_raw_bad_tokens_and_unmatched_closers() {
    for (source, kind, responsible) in [
        ("url(a b)", CssComponentValueErrorKind::BadUrl, 0..8),
        ("\"bad\n", CssComponentValueErrorKind::BadString, 0..4),
        (
            "]",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
            0..1,
        ),
        (
            ")",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
            0..1,
        ),
        (
            "}",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
            0..1,
        ),
        (
            "f([)])",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
            3..4,
        ),
    ] {
        let report = parse_component_value(source);
        assert!(report.syntax().is_none());
        let [diagnostic] = report.diagnostics() else {
            panic!("original lexical rejection")
        };
        lexical(diagnostic, source, kind, responsible, 0..source.len());
    }
}

#[test]
fn singular_implicit_function_and_all_block_ends_retain_syntax_but_reject_clean_validation() {
    for source in ["f(a", "[a", "(a", "{a"] {
        let report = parse_component_value(source);
        let component = report.syntax().as_ref().unwrap();
        let closing = match component.view() {
            CssComponentValueRef::Function(f) => f.closing_origin(),
            CssComponentValueRef::Block(b) => b.closing_origin(),
            _ => panic!("returned implicitly closed group"),
        };
        let CssValueOrigin::ImplicitClosure { opening, at } = closing else {
            panic!("actual missing-end identity")
        };
        assert!(
            opening
                .source()
                .same_snapshot(parsed(component.origin()).source())
        );
        assert!(at.source().same_snapshot(opening.source()));
        span(at.span(), source.len()..source.len());
        let [diagnostic] = report.diagnostics() else {
            panic!("retained EOF fault")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        span(diagnostic.span(), source.len()..source.len());
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn raw_comma_entry_preserves_empty_leading_double_and_final_slots() {
    for (source, lengths) in [
        ("", vec![0]),
        (",", vec![0, 0]),
        ("a,,", vec![1, 0, 0]),
        (",a,,b,", vec![0, 1, 0, 1, 0]),
    ] {
        let report = parse_comma_separated_component_values(source);
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(
            report
                .syntax()
                .iter()
                .map(|slot| slot.as_ref().unwrap().items().len())
                .collect::<Vec<_>>(),
            lengths
        );
        assert!(report.into_validation_result().is_ok());
    }
}

#[test]
fn raw_comma_entry_retains_whitespace_comments_and_one_trivia_only_slot() {
    let source = " \t/**/ ";
    let report = parse_comma_separated_component_values(source);
    assert!(report.is_clean());
    let [Some(values)] = report.syntax().as_slice() else {
        panic!("raw trivia source has one slot")
    };
    assert_eq!(values.items().len(), 3);
    assert!(matches!(
        values.items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" \t"))
    ));
    assert!(matches!(
        values.items()[1].view(),
        CssComponentValueRef::Comment("")
    ));
    assert!(matches!(
        values.items()[2].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" "))
    ));
    for (component, range) in values.items().iter().zip([0..2, 2..6, 6..7]) {
        origin(component.origin(), source, range);
    }
}

#[test]
fn raw_comma_entry_keeps_all_nested_commas_in_their_function_or_block() {
    let source = "a,f(x,y),[x,y],(x,y),{x,y},,";
    let report = parse_comma_separated_component_values(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 7);
    let first = report.syntax()[0].as_ref().unwrap();
    ident(&first.items()[0], "a");
    let original = parsed(first.items()[0].origin()).source();
    for (index, opening_range) in [(1, 2..4), (2, 9..10), (3, 15..16), (4, 21..22)] {
        let values = report.syntax()[index].as_ref().unwrap();
        let [component] = values.items() else {
            panic!("nested comma cannot divide a root group")
        };
        origin(component.origin(), source, opening_range);
        assert!(parsed(component.origin()).source().same_snapshot(original));
        let contents = match component.view() {
            CssComponentValueRef::Function(f) => f.values(),
            CssComponentValueRef::Block(b) => b.values(),
            _ => panic!("one enclosure"),
        };
        assert_eq!(contents.items().len(), 3);
        assert!(matches!(
            contents.items()[1].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Comma)
        ));
    }
    assert!(report.syntax()[5].as_ref().unwrap().items().is_empty());
    assert!(report.syntax()[6].as_ref().unwrap().items().is_empty());
}

#[test]
fn raw_comma_member_failure_preserves_later_slots_and_original_responsible_span() {
    let source = "a, url(a b) ,z";
    let report = parse_comma_separated_component_values(source);
    let [Some(first), None, Some(last)] = report.syntax().as_slice() else {
        panic!("local bad-URL failure occupies only its original slot")
    };
    ident(&first.items()[0], "a");
    ident(&last.items()[0], "z");
    origin(last.items()[0].origin(), source, 13..14);
    assert!(
        parsed(first.items()[0].origin())
            .source()
            .same_snapshot(parsed(last.items()[0].origin()).source())
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one bad member diagnostic")
    };
    lexical(
        diagnostic,
        source,
        CssComponentValueErrorKind::BadUrl,
        3..11,
        2..12,
    );
    assert!(report.into_validation_result().is_err());
    let source = "] ,a,\"bad\n,z";
    let report = parse_comma_separated_component_values(source);
    let [None, Some(a), None, Some(z)] = report.syntax().as_slice() else {
        panic!("both malformed members retain independent failures")
    };
    ident(&a.items()[0], "a");
    ident(&z.items()[0], "z");
    let [closer, bad_string] = report.diagnostics() else {
        panic!("source-ordered lexical diagnostics")
    };
    lexical(
        closer,
        source,
        CssComponentValueErrorKind::UnmatchedClosingDelimiter,
        0..1,
        0..2,
    );
    lexical(
        bad_string,
        source,
        CssComponentValueErrorKind::BadString,
        5..9,
        5..10,
    );
}

#[test]
fn raw_comma_entry_retains_implicit_group_and_does_not_split_its_unclosed_child_comma() {
    let source = "a,f(x,y";
    let report = parse_comma_separated_component_values(source);
    let [Some(first), Some(last)] = report.syntax().as_slice() else {
        panic!("child comma remains inside the last function")
    };
    ident(&first.items()[0], "a");
    let [component] = last.items() else {
        panic!("whole implicit function")
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("function retained")
    };
    assert_eq!(function.values().items().len(), 3);
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("implicit group fault retained")
    };
    assert!(at.source().same_snapshot(opening.source()));
    span(at.span(), 7..7);
    let [diagnostic] = report.diagnostics() else {
        panic!("one retained implicit-end fault")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    span(diagnostic.span(), 7..7);
    assert!(report.into_validation_result().is_err());
}

#[test]
fn explicit_limits_charge_whole_original_input_and_fail_atomically_before_slot_promotion() {
    for (source, limits, expected) in [
        (
            "a,b",
            CssComponentValueLimits::try_new(0, 2, 100).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            "a,b",
            CssComponentValueLimits::try_new(0, 100, 2).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
        (
            "a,f(x)",
            CssComponentValueLimits::try_new(0, 100, 100).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
    ] {
        let report = parse_comma_separated_component_values_with_limits(source, limits);
        assert!(
            report.syntax().is_empty(),
            "whole-input resource failure admits no partial first slot: {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one atomic resource failure")
        };
        let ErrorKind::InvalidComponentValue(detail) = diagnostic.error().kind() else {
            panic!("typed original resource error")
        };
        assert_eq!(detail.kind(), expected);
        if expected == CssComponentValueErrorKind::ByteLimit {
            assert_eq!(
                detail.origin(),
                &CssValueOrigin::UnretainedInput {
                    byte_length: source.len()
                }
            );
            assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
        } else {
            let responsible = match (source, expected) {
                ("a,b", CssComponentValueErrorKind::ComponentLimit)
                | (" a ", CssComponentValueErrorKind::ComponentLimit) => 2..3,
                ("a,f(x)", CssComponentValueErrorKind::NestingLimit) => 2..4,
                ("f(x)", CssComponentValueErrorKind::NestingLimit) => 0..2,
                _ => panic!("literal selected limit stimulus"),
            };
            origin(detail.origin(), source, responsible.clone());
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                responsible.start
            );
        }
        assert_eq!(
            diagnostic.action(),
            if expected == CssComponentValueErrorKind::NestingLimit {
                CssRecoveryAction::StopAtNestingLimit
            } else {
                CssRecoveryAction::RejectInput
            }
        );
        span(diagnostic.span(), 0..source.len());
        assert!(report.into_validation_result().is_err());
    }
    for (source, limits, expected) in [
        (
            " a ",
            CssComponentValueLimits::try_new(0, 2, 100).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            " a ",
            CssComponentValueLimits::try_new(0, 100, 2).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
        (
            "f(x)",
            CssComponentValueLimits::try_new(0, 100, 100).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
    ] {
        let report = parse_component_value_with_limits(source, limits);
        assert!(
            report.syntax().is_none(),
            "outer trivia and groups remain charged: {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one resource failure")
        };
        let ErrorKind::InvalidComponentValue(detail) = diagnostic.error().kind() else {
            panic!("typed resource error")
        };
        assert_eq!(detail.kind(), expected);
        if expected == CssComponentValueErrorKind::ByteLimit {
            assert_eq!(
                detail.origin(),
                &CssValueOrigin::UnretainedInput {
                    byte_length: source.len()
                }
            );
            assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
        } else {
            let responsible = match (source, expected) {
                ("a,b", CssComponentValueErrorKind::ComponentLimit)
                | (" a ", CssComponentValueErrorKind::ComponentLimit) => 2..3,
                ("a,f(x)", CssComponentValueErrorKind::NestingLimit) => 2..4,
                ("f(x)", CssComponentValueErrorKind::NestingLimit) => 0..2,
                _ => panic!("literal selected limit stimulus"),
            };
            origin(detail.origin(), source, responsible.clone());
            assert_eq!(
                diagnostic.error().position().byte_offset().value(),
                responsible.start
            );
        }
        assert_eq!(
            diagnostic.action(),
            if expected == CssComponentValueErrorKind::NestingLimit {
                CssRecoveryAction::StopAtNestingLimit
            } else {
                CssRecoveryAction::RejectInput
            }
        );
        span(diagnostic.span(), 0..source.len());
    }
    let limits = CssComponentValueLimits::try_new(1, 4, 6).unwrap();
    let source = "a,f(x)";
    let report = parse_comma_separated_component_values_with_limits(source, limits);
    assert!(
        report.is_clean(),
        "exact aggregate bounds admit a leaf, separator, function and its child: {report:?}"
    );
    assert_eq!(report.syntax().len(), 2);
}

#[test]
fn generated_implicit_closers_and_root_separators_share_one_aggregate_byte_budget() {
    let source = "a,f(g(";
    // Original bytes fit six; selected spelling is a,f(g()), eight bytes.
    // The last segment's six generated bytes fit alone but not with a,.
    let alone = parse_component_value_with_limits(
        "f(g(",
        CssComponentValueLimits::try_new(2, 4, 6).unwrap(),
    );
    assert!(alone.syntax().is_some());
    assert_eq!(alone.diagnostics().len(), 2);
    for limit in [6, 7] {
        let report = parse_comma_separated_component_values_with_limits(
            source,
            CssComponentValueLimits::try_new(2, 4, limit).unwrap(),
        );
        assert!(
            report.syntax().is_empty(),
            "aggregate overflow rejects prior valid slots: {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("atomic generated-byte resource failure")
        };
        let ErrorKind::InvalidComponentValue(detail) = diagnostic.error().kind() else {
            panic!("typed canonical byte bound")
        };
        assert_eq!(detail.kind(), CssComponentValueErrorKind::ByteLimit);
        let CssValueOrigin::ImplicitClosure { opening, at } = detail.origin() else {
            panic!("the actual first over-budget generated closer")
        };
        assert_eq!(opening.source().as_str(), source);
        assert!(opening.source().same_snapshot(at.source()));
        let responsible = if limit == 6 { 4..6 } else { 2..4 };
        span(opening.span(), responsible.clone());
        span(at.span(), 6..6);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible.start
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        span(diagnostic.span(), 0..6);
        assert!(report.into_validation_result().is_err());
    }
    let report = parse_comma_separated_component_values_with_limits(
        source,
        CssComponentValueLimits::try_new(2, 4, 8).unwrap(),
    );
    let [Some(first), Some(last)] = report.syntax().as_slice() else {
        panic!("exact aggregate generated spelling bound admits both lists")
    };
    ident(&first.items()[0], "a");
    let CssComponentValueRef::Function(function) = last.items()[0].view() else {
        panic!("outer function")
    };
    assert_eq!(function.name(), "f");
    let CssComponentValueRef::Function(inner) = function.values().items()[0].view() else {
        panic!("nested function")
    };
    assert_eq!(inner.name(), "g");
    assert!(inner.values().items().is_empty());
    assert_eq!(report.diagnostics().len(), 2);
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        span(diagnostic.span(), 6..6);
    }
    assert!(report.into_validation_result().is_err());
}

#[test]
fn default_and_explicit_depth_limits_preserve_the_public_256_level_ceiling() {
    assert!(
        CssComponentValueLimits::try_new(257, 258, 772).is_none(),
        "explicit limits cannot raise the structural ceiling"
    );
    let source = format!("{}x{}", "f(".repeat(256), ")".repeat(256));
    let limits = CssComponentValueLimits::try_new(256, 257, 769).unwrap();
    for report in [
        parse_component_value(&source),
        parse_component_value_with_limits(&source, limits),
    ] {
        assert!(
            report.is_clean(),
            "the 256th group is within the documented ceiling: {report:?}"
        );
        assert!(report.syntax().is_some());
    }
    let source = format!("{}x{}", "f(".repeat(257), ")".repeat(257));
    let limits = CssComponentValueLimits::try_new(256, 258, 772).unwrap();
    for report in [
        parse_component_value(&source),
        parse_component_value_with_limits(&source, limits),
    ] {
        assert!(report.syntax().is_none());
        let [diagnostic] = report.diagnostics() else {
            panic!("one global depth admission failure")
        };
        let ErrorKind::InvalidComponentValue(detail) = diagnostic.error().kind() else {
            panic!("typed depth failure")
        };
        assert_eq!(detail.kind(), CssComponentValueErrorKind::NestingLimit);
        origin(detail.origin(), &source, 512..514);
        assert_eq!(diagnostic.error().position().byte_offset().value(), 512);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        span(diagnostic.span(), 0..772);
    }
}
