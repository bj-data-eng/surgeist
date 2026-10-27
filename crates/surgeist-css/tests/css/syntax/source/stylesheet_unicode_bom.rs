//! Unicode-string input is distinct from a byte stream requiring BOM decoding.
//!
//! CSS Syntax 3 (2026-10-01), sections 3.3, 4.2, 5.3 and 5.4.3, preserves
//! U+FEFF in string input as an identifier code point. Section 5.5.3
//! discards a qualified rule whose prelude reaches EOF without a block.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/
//! That edition's §4.2 excludes U+FFFE from non-ASCII ident code points; the
//! U+FFFE rows still follow the 2021-12-24 §4.2 classification, a deferred
//! tokenizer difference (docs/reference.md#css-syntax-3-edition-differences).
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#non-ascii-code-point

use crate::css::support::authored_syntax::assert_position;
use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::CssRule;
use surgeist_css::selector::CssSelector;

fn assert_incomplete_unicode_rule(source: &str, start: (usize, u32, u32), end: (usize, u32, u32)) {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.syntax().rules().is_empty(), "{source:?}");
    assert!(!report.is_clean(), "{source:?}");
    assert!(
        CssParser::default()
            .parse_sheet(source)
            .into_validation_result()
            .is_err(),
        "{source:?}"
    );
    assert_eq!(report.diagnostics().len(), 1, "{source:?}");
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidQualifiedRule
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_position(diagnostic.error().position(), end.0, end.1, end.2);
    assert_position(diagnostic.span().start(), start.0, start.1, start.2);
    assert_position(diagnostic.span().end(), end.0, end.1, end.2);
}

#[test]
fn leading_unicode_feff_is_an_incomplete_qualified_rule() {
    // Original StyleSheet.json "BOM UTF-16BE" fixture is already Unicode.
    assert_incomplete_unicode_rule("\u{feff}", (0, 0, 0), (3, 0, 1));
}

#[test]
fn at_keyword_after_unicode_feff_remains_in_the_qualified_rule_prelude() {
    // Original "BOM UTF-16BE #2": @a is not a separate top-level at-rule.
    assert_incomplete_unicode_rule("\u{feff}@a;", (0, 0, 0), (6, 0, 4));
}

#[test]
fn repeated_and_nonleading_unicode_marks_preserve_original_coordinates() {
    for (source, start, end) in [
        ("\u{fffe}", (0, 0, 0), (3, 0, 1)),
        ("\u{fffe}@a;", (0, 0, 0), (6, 0, 4)),
        ("\u{feff}\u{feff}", (0, 0, 0), (6, 0, 2)),
        ("\r\n \u{feff}", (3, 1, 1), (6, 1, 2)),
        ("a\u{feff}", (0, 0, 0), (4, 0, 2)),
    ] {
        assert_incomplete_unicode_rule(source, start, end);
    }
}

#[test]
fn unicode_feff_is_retained_in_complete_type_selector_names() {
    for (source, expected_name, next_byte, next_column) in [
        ("\u{feff}a{} b{}", "\u{feff}a", 7, 5),
        ("\u{feff}\u{feff}a{} b{}", "\u{feff}\u{feff}a", 10, 6),
        ("a\u{feff}{} b{}", "a\u{feff}", 7, 5),
        ("\u{fffe}a{} b{}", "\u{fffe}a", 7, 5),
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
        assert_eq!(
            CssParser::default()
                .parse_sheet(source)
                .into_validation_result()
                .unwrap(),
            *report.syntax(),
            "{source:?}"
        );
        assert_eq!(report.syntax().rules().len(), 2, "{source:?}");
        for (rule, name, byte, column) in [
            (&report.syntax().rules()[0], expected_name, 0, 0),
            (&report.syntax().rules()[1], "b", next_byte, next_column),
        ] {
            let CssRule::Style(rule) = rule else {
                panic!("expected a style rule in {source:?}");
            };
            assert_eq!(rule.selectors().selectors().len(), 1);
            assert_eq!(
                rule.selectors().selectors()[0].selector(),
                &CssSelector::Tag(name.to_owned())
            );
            assert_position(rule.position(), byte, 0, column);
        }
    }
}
