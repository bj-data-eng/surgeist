#![forbid(unsafe_code)]
// Install as a cfg(test) child of parser; never a public fallback test hook.
//! The grammars of a keyframe's declarations and of a margin box, which the
//! parser crate's block driver runs, entered from the native callbacks that
//! still hand them a scope and from the fronts that build a token tree
//! directly. The expected values are read off the literals: Syntax 3 (2026
//! CRD) §5.5.5 for the units, Paged Media 3 Appendix A and CSS Animations 1
//! §4 for what a block admits, and the byte offsets of the sources.
use super::fragments;
use crate::diagnostic::{CssErrorCode, CssRecoveryAction};
use crate::rule::keyframes::CssParsedKeyframeRule;
use crate::rule::page::CssMarginRule;
use crate::{CssParseReport, CssParser, CssParserContext};

fn start(report_span: crate::source::CssSourceSpan) -> (usize, usize) {
    (
        report_span.start().byte_offset().value(),
        report_span.end().byte_offset().value(),
    )
}

/// The byte range of the first occurrence of `text` in `source`.
fn range_of(source: &str, text: &str) -> (usize, usize) {
    let at = source.find(text).expect("the literal contains the text");
    (at, at + text.len())
}

fn dropped<T>(
    report: &CssParseReport<T>,
) -> Vec<(CssRecoveryAction, CssErrorCode, (usize, usize))> {
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.action(),
                diagnostic.error().code(),
                start(diagnostic.span()),
            )
        })
        .collect()
}

#[test]
fn a_keyframe_block_entered_by_the_native_rule_driver_drops_rules_and_keeps_declarations() {
    let source = "{ from { color: red; @media x {} opacity: 1 } }";
    let report = fragments::parse_keyframes_block_with_context(source, CssParserContext::default());
    let blocks = report.syntax().as_ref().expect("a keyframes block").body();
    let [block] = blocks.as_slice() else {
        panic!("one keyframe block: {report:?}")
    };
    assert_eq!(block.declarations().len(), 2, "{report:?}");
    // CSS Animations 1 section 4: a keyframe's block is a declaration list, so
    // the at-rule is invalid there and is dropped as one unit, block included.
    assert_eq!(
        dropped(&report),
        [(
            CssRecoveryAction::DropAtRule,
            CssErrorCode::InvalidAtRulePlacement,
            range_of(source, "@media x {}")
        )]
    );
}

#[test]
fn a_keyframe_declaration_block_is_one_block_and_nothing_else() {
    let context = CssParserContext::default();
    let source = "  { color: red; opacity: 1 }  ";
    let report = fragments::parse_keyframe_declaration_block_with_context(source, context);
    let fragment = report.syntax().as_ref().expect("a declaration block");
    assert_eq!(fragment.body().len(), 2);
    assert_eq!(
        start(fragment.origin().span()),
        range_of(source, "{ color: red; opacity: 1 }")
    );
    assert!(report.is_clean());
    for rejected in [
        "color: red",
        "",
        "{ color: red } color: red",
        "{ color: red } {}",
    ] {
        let report = fragments::parse_keyframe_declaration_block_with_context(rejected, context);
        assert!(report.syntax().is_none(), "{rejected:?}: {report:?}");
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.action())
                .collect::<Vec<_>>(),
            [CssRecoveryAction::RejectInput],
            "{rejected:?}"
        );
    }
}

#[test]
fn a_declaration_value_runs_to_its_semicolon_when_a_substitution_admits_the_block() {
    // The value of `color` holds a substitution, which admits any tokens (CSS
    // Variables 1 section 3), so the `{}`-block does not end the declaration:
    // it runs to the `;`, as the native block parser read it, and the
    // declaration after the semicolon is separate.
    let source = "{ color: var(--a) {x} opacity: 1; width: 1px }";
    let report = fragments::parse_keyframe_declaration_block_with_context(
        source,
        CssParserContext::default(),
    );
    let fragment = report.syntax().as_ref().expect("a declaration block");
    assert_eq!(fragment.body().len(), 2, "{report:?}");
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn a_declaration_whose_block_is_refused_reports_its_grammars_verdict_first() {
    // An unknown property is the first thing the declaration grammar says of
    // `x: y { z }`, before the value, which §5.5.6 step 8 would refuse.
    let source = "{ x: y { z } ; color: red }";
    let report = fragments::parse_keyframe_declaration_block_with_context(
        source,
        CssParserContext::default(),
    );
    assert_eq!(report.syntax().as_ref().unwrap().body().len(), 1);
    assert_eq!(
        dropped(&report),
        [(
            CssRecoveryAction::DropDeclaration,
            CssErrorCode::UnknownProperty,
            range_of(source, "x: y { z }")
        )]
    );
}

#[test]
fn a_detached_margin_rule_admits_its_declarations_and_reports_what_it_drops() {
    let source = "@top-left { margin: var(--a) {x} 1px; margin-top: 1px; bogus: 1; @media x {} }";
    let report = CssMarginRule::parse(&CssParser::default(), source);
    let rule = report.syntax().as_ref().expect("a margin rule");
    // The substitution admits the block, so the first declaration runs to its `;`.
    assert_eq!(rule.declarations().properties().len(), 2, "{report:?}");
    assert_eq!(
        dropped(&report),
        [
            (
                CssRecoveryAction::DropDeclaration,
                CssErrorCode::UnknownProperty,
                range_of(source, "bogus: 1;")
            ),
            (
                CssRecoveryAction::DropAtRule,
                CssErrorCode::InvalidAtRuleBody,
                range_of(source, "@media x {}")
            ),
        ]
    );
}

#[test]
fn a_rejected_prelude_and_a_rejected_block_drop_different_extents() {
    // A margin box's name takes no prelude: `@top-right x {}` is rejected before
    // its block is entered, and the unit stops after the block's `{`. An at-rule
    // that no margin box declares is rejected for its block, which was entered
    // and is dropped with it.
    let source = "@top-left { @top-right x {} margin-top: 1px; @media x {} }";
    let report = CssMarginRule::parse(&CssParser::default(), source);
    let [prelude, block] = dropped(&report)[..] else {
        panic!("two dropped units: {report:?}")
    };
    // The span of the first stops after the `{` of the block it never entered.
    assert_eq!(
        (prelude.0, prelude.1, prelude.2.1),
        (
            CssRecoveryAction::DropAtRule,
            CssErrorCode::UnexpectedToken,
            range_of(source, "x {").1
        )
    );
    assert_eq!(
        block,
        (
            CssRecoveryAction::DropAtRule,
            CssErrorCode::InvalidAtRuleBody,
            range_of(source, "@media x {}")
        )
    );
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .declarations()
            .properties()
            .len(),
        1
    );
}

#[test]
fn a_rule_that_is_no_margin_box_is_rejected_with_its_own_reason() {
    for (source, code) in [
        ("@page {}", CssErrorCode::InvalidAtRulePlacement),
        ("@unknown {}", CssErrorCode::UnknownAtRule),
        ("@top-left", CssErrorCode::UnexpectedEnd),
        ("@top-left;", CssErrorCode::UnexpectedToken),
        ("{}", CssErrorCode::InvalidQualifiedRule),
    ] {
        let report = CssMarginRule::parse(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        // The rejection is the last diagnostic; a rule cut short by the end of
        // its input is also reported as the unterminated rule it is.
        assert_eq!(
            report
                .diagnostics()
                .last()
                .map(|diagnostic| diagnostic.error().code()),
            Some(code),
            "{source}"
        );
    }
}

#[test]
fn a_detached_keyframe_rule_reaches_the_declaration_grammar_through_the_native_rule() {
    let source = "from { color: red; bogus: 1; opacity: 1 }";
    let report = CssParsedKeyframeRule::parse(&CssParser::default(), source);
    let rule = report.syntax().as_ref().expect("a keyframe rule");
    assert_eq!(rule.block().declarations().len(), 2, "{report:?}");
    assert_eq!(
        dropped(&report),
        [(
            CssRecoveryAction::DropDeclaration,
            CssErrorCode::UnknownProperty,
            range_of(source, "bogus: 1;")
        )]
    );
}
