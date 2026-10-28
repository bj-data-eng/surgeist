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
use crate::rule::CssRule;
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

// CHARACTERIZATION of current behaviour that contradicts Syntax 3 (2026 CRD)
// §5.5.6 step 8, which returns nothing for a non-custom declaration whose
// value holds a `{}`-block beside other values (WPT `var-with-blocks.html`
// agrees). It is due to change when every declaration path follows steps 6 to
// 8; the expectations below then change with it.
#[test]
fn characterization_a_declaration_value_runs_to_its_semicolon_when_a_substitution_admits_the_block()
{
    // The value of `color` holds a substitution; the `{}`-block does not end
    // the declaration, which runs to the `;`, and the declaration after the
    // semicolon is separate. Step 8 would drop `color: var(--a) {x}` as a
    // qualified rule and read `opacity: 1;` as a declaration of its own.
    let source = "{ color: var(--a) {x} opacity: 1; width: 1px }";
    let report = fragments::parse_keyframe_declaration_block_with_context(
        source,
        CssParserContext::default(),
    );
    let fragment = report.syntax().as_ref().expect("a declaration block");
    assert_eq!(fragment.body().len(), 2, "{report:?}");
    assert!(report.is_clean(), "{report:?}");
}

// CHARACTERIZATION, as above: step 8 refuses `x: y { z }` before any grammar
// sees it, so its error would be the unexpected `{`, not the unknown property.
#[test]
fn characterization_a_declaration_whose_block_is_refused_reports_its_grammars_verdict_first() {
    // The unknown property is the first thing the declaration grammar says of
    // `x: y { z }`, before the value.
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
    // CHARACTERIZATION (contradicts §5.5.6 step 8, due to change with it): the
    // substitution admits the block, so the first declaration runs to its `;`.
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

fn importance_and_codes(body: &str) -> (Vec<crate::declaration::CssImportance>, Vec<CssErrorCode>) {
    let source = format!("@top-left {{ {body} }}");
    let report = CssMarginRule::parse(&CssParser::default(), &source);
    let rule = report.syntax().as_ref().expect("a margin rule");
    (
        rule.declarations()
            .properties()
            .iter()
            .map(|declaration| declaration.importance())
            .collect(),
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect(),
    )
}

#[test]
fn the_importance_annotation_ignores_whitespace_comments_and_the_case_of_important() {
    use crate::declaration::CssImportance::{Important, Normal};
    // Syntax 3 (2026 CRD) §5.5.6 step 6: a value whose last two non-whitespace
    // tokens are `!` and an ASCII case-insensitive `important` is important,
    // and a comment is not a token.
    for (body, expected) in [
        ("margin-top: 1px", Normal),
        ("margin-top: 1px !important", Important),
        ("margin-top: 1px!important", Important),
        ("margin-top: 1px ! important", Important),
        ("margin-top: 1px /*c*/ ! /*d*/ important /*e*/", Important),
        ("margin-top: 1px !IMPORTANT", Important),
        ("margin-top: 1px !ImPoRtAnT\n", Important),
        ("--x: a !important", Important),
        ("--x: a ! important ", Important),
    ] {
        assert_eq!(
            importance_and_codes(body),
            (vec![expected], vec![]),
            "{body:?}"
        );
    }
}

// CHARACTERIZATION of current behaviour that departs from Syntax 3 (2026 CRD)
// §5.5.6 steps 6 and 7: the first `!` of a value starts the annotation, and
// anything but `important` after it is an invalid annotation. By the steps, a
// `!` that is not the second to last token is an ordinary delimiter in the
// value, and the declaration is judged by its property's grammar (for a custom
// property it is accepted whole). Due to change with step 12.
#[test]
fn characterization_a_bang_that_does_not_end_the_value_is_an_invalid_annotation() {
    for body in [
        "margin-top: 1px ! 2px",
        "margin-top: 1px !important x",
        "margin-top: 1px !important !important",
        "margin-top: 1px !",
        "margin-top: 1px !importantx",
        "--x: a ! b",
        "--x: a !important b",
    ] {
        assert_eq!(
            importance_and_codes(body),
            (vec![], vec![CssErrorCode::InvalidDeclarationAnnotation]),
            "{body:?}"
        );
    }
}

#[test]
fn a_keyframe_declaration_admits_no_importance() {
    // CSS Animations 1 section 4.2: a declaration marked `!important` in a
    // keyframe is ignored.
    for body in [
        "color: red !important",
        "color: red ! important",
        "--x: a !important",
    ] {
        let source = format!("{{ {body}; opacity: 1 }}");
        let report = fragments::parse_keyframe_declaration_block_with_context(
            &source,
            CssParserContext::default(),
        );
        assert_eq!(
            report.syntax().as_ref().unwrap().body().len(),
            1,
            "{body:?}"
        );
        assert_eq!(
            dropped(&report)
                .iter()
                .map(|unit| (unit.0, unit.1))
                .collect::<Vec<_>>(),
            [(
                CssRecoveryAction::DropDeclaration,
                CssErrorCode::InvalidDeclarationAnnotation
            )],
            "{body:?}"
        );
    }
}

#[test]
fn error_locations_count_columns_in_utf16_units_and_offsets_in_bytes() {
    // Every character before the error is counted once in a column per UTF-16
    // unit (`é` one, `😀` two) and in the byte offset per UTF-8 byte (two and
    // four).
    let source = "@top-left { content: \"é😀\"; bogus: 1 }";
    let report = CssMarginRule::parse(&CssParser::default(), source);
    let [unknown] = report.diagnostics() else {
        panic!("one diagnostic: {report:?}")
    };
    assert_eq!(unknown.error().code(), CssErrorCode::UnknownProperty);
    let position = unknown.error().position();
    assert_eq!(
        (position.byte_offset().value(), position.column().value()),
        (source.find("bogus").unwrap(), 28)
    );
    // CHARACTERIZATION (§5.5.6 step 6 would not call this an annotation): the
    // annotation error is located at the `!`.
    let source = "@top-left { content: \"😀\"; margin-top: 1px ! x }";
    let report = CssMarginRule::parse(&CssParser::default(), source);
    let [annotation] = report.diagnostics() else {
        panic!("one diagnostic: {report:?}")
    };
    assert_eq!(
        annotation.error().code(),
        CssErrorCode::InvalidDeclarationAnnotation
    );
    let position = annotation.error().position();
    assert_eq!(
        (position.byte_offset().value(), position.column().value()),
        (source.find('!').unwrap(), 43)
    );
}

/// What a nest of one opener, `depth` deep, in one position of a block makes of
/// an entry point whose block the native rule driver enters: the entry point,
/// the position of the nest (`last` after a declaration, `followed` by another
/// declaration, `beside` a declaration on each side), the opener, the depth,
/// whether the input has syntax, how many declarations it keeps (the sheet
/// entry counts a keyframe declaration as 1000 and a margin declaration as 1),
/// and each diagnostic as (code, action, span start, span end).
type CeilingRow = (
    &'static str,
    &'static str,
    &'static str,
    usize,
    bool,
    usize,
    &'static [(CssErrorCode, CssRecoveryAction, usize, usize)],
);

// The structural scans of the recovery state judge the ceiling of 256 levels,
// counted from the whole source, one unit at a time: a unit nested past it is
// dropped alone (and the diagnostic spans only that unit), and the units
// around it, before and after, are kept, at 255, 256, 257 and 1000 levels, for
// every opener, and whether the block is entered through a keyframes rule, a
// detached keyframe, a page rule or a sheet. The scope of a block therefore
// has no ceiling of its own that could refuse all of it.
#[test]
fn a_nest_at_the_depth_ceiling_drops_only_its_unit_in_every_block_the_native_driver_enters() {
    use CssErrorCode::{
        InvalidAtRuleBody, InvalidQualifiedRule, NestingLimit, UnexpectedEnd, UnexpectedToken,
    };
    use CssRecoveryAction::{DropAtRule, DropQualifiedRule, StopAtNestingLimit};
    const TABLE: &[CeilingRow] = &[
        (
            "kf-body",
            "last",
            "brace",
            255,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 21, 531)],
        ),
        (
            "kf-body",
            "last",
            "brace",
            256,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 21, 533)],
        ),
        (
            "kf-body",
            "last",
            "brace",
            257,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 21, 535)],
        ),
        (
            "kf-body",
            "last",
            "brace",
            1000,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 21, 2021)],
        ),
        (
            "kf-body",
            "followed",
            "brace",
            256,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 21, 533)],
        ),
        (
            "kf-body",
            "followed",
            "brace",
            1000,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 21, 2021)],
        ),
        (
            "kf-body",
            "beside",
            "brace",
            256,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 26, 538)],
        ),
        (
            "kf-body",
            "beside",
            "brace",
            257,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 26, 540)],
        ),
        (
            "kf-body",
            "last",
            "paren",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 532)],
        ),
        (
            "kf-body",
            "last",
            "paren",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 534)],
        ),
        (
            "kf-body",
            "last",
            "paren",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 536)],
        ),
        (
            "kf-body",
            "last",
            "paren",
            1000,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 2022)],
        ),
        (
            "kf-body",
            "followed",
            "paren",
            256,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 534)],
        ),
        (
            "kf-body",
            "followed",
            "paren",
            1000,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 2022)],
        ),
        (
            "kf-body",
            "beside",
            "paren",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 26, 550)],
        ),
        (
            "kf-body",
            "beside",
            "paren",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 26, 552)],
        ),
        (
            "kf-body",
            "last",
            "square",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 532)],
        ),
        (
            "kf-body",
            "last",
            "square",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 534)],
        ),
        (
            "kf-body",
            "last",
            "square",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 536)],
        ),
        (
            "kf-body",
            "last",
            "square",
            1000,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 2022)],
        ),
        (
            "kf-body",
            "followed",
            "square",
            256,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 534)],
        ),
        (
            "kf-body",
            "followed",
            "square",
            1000,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 2022)],
        ),
        (
            "kf-body",
            "beside",
            "square",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 26, 550)],
        ),
        (
            "kf-body",
            "beside",
            "square",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 26, 552)],
        ),
        (
            "kf-body",
            "last",
            "function",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 787)],
        ),
        (
            "kf-body",
            "last",
            "function",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 790)],
        ),
        (
            "kf-body",
            "last",
            "function",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 793)],
        ),
        (
            "kf-body",
            "last",
            "function",
            1000,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 3022)],
        ),
        (
            "kf-body",
            "followed",
            "function",
            256,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 790)],
        ),
        (
            "kf-body",
            "followed",
            "function",
            1000,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 21, 3022)],
        ),
        (
            "kf-body",
            "beside",
            "function",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 26, 806)],
        ),
        (
            "kf-body",
            "beside",
            "function",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 26, 809)],
        ),
        (
            "kf-rule",
            "last",
            "brace",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 529)],
        ),
        (
            "kf-rule",
            "last",
            "brace",
            256,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 19, 531)],
        ),
        (
            "kf-rule",
            "last",
            "brace",
            257,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 19, 533)],
        ),
        (
            "kf-rule",
            "last",
            "brace",
            1000,
            true,
            1,
            &[(NestingLimit, StopAtNestingLimit, 19, 2019)],
        ),
        (
            "kf-rule",
            "followed",
            "brace",
            256,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 19, 531)],
        ),
        (
            "kf-rule",
            "followed",
            "brace",
            1000,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 19, 2019)],
        ),
        (
            "kf-rule",
            "beside",
            "brace",
            256,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 24, 536)],
        ),
        (
            "kf-rule",
            "beside",
            "brace",
            257,
            true,
            2,
            &[(NestingLimit, StopAtNestingLimit, 24, 538)],
        ),
        (
            "kf-rule",
            "last",
            "paren",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 530)],
        ),
        (
            "kf-rule",
            "last",
            "paren",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 532)],
        ),
        (
            "kf-rule",
            "last",
            "paren",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 534)],
        ),
        (
            "kf-rule",
            "last",
            "paren",
            1000,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 2020)],
        ),
        (
            "kf-rule",
            "followed",
            "paren",
            256,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 532)],
        ),
        (
            "kf-rule",
            "followed",
            "paren",
            1000,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 2020)],
        ),
        (
            "kf-rule",
            "beside",
            "paren",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 24, 548)],
        ),
        (
            "kf-rule",
            "beside",
            "paren",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 24, 550)],
        ),
        (
            "kf-rule",
            "last",
            "square",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 530)],
        ),
        (
            "kf-rule",
            "last",
            "square",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 532)],
        ),
        (
            "kf-rule",
            "last",
            "square",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 534)],
        ),
        (
            "kf-rule",
            "last",
            "square",
            1000,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 2020)],
        ),
        (
            "kf-rule",
            "followed",
            "square",
            256,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 532)],
        ),
        (
            "kf-rule",
            "followed",
            "square",
            1000,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 2020)],
        ),
        (
            "kf-rule",
            "beside",
            "square",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 24, 548)],
        ),
        (
            "kf-rule",
            "beside",
            "square",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 24, 550)],
        ),
        (
            "kf-rule",
            "last",
            "function",
            255,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 785)],
        ),
        (
            "kf-rule",
            "last",
            "function",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 788)],
        ),
        (
            "kf-rule",
            "last",
            "function",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 791)],
        ),
        (
            "kf-rule",
            "last",
            "function",
            1000,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 3020)],
        ),
        (
            "kf-rule",
            "followed",
            "function",
            256,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 788)],
        ),
        (
            "kf-rule",
            "followed",
            "function",
            1000,
            true,
            2,
            &[(InvalidQualifiedRule, DropQualifiedRule, 19, 3020)],
        ),
        (
            "kf-rule",
            "beside",
            "function",
            256,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 24, 804)],
        ),
        (
            "kf-rule",
            "beside",
            "function",
            257,
            true,
            1,
            &[(InvalidQualifiedRule, DropQualifiedRule, 24, 807)],
        ),
        (
            "page-block",
            "last",
            "brace",
            255,
            true,
            1,
            &[(InvalidAtRuleBody, DropAtRule, 26, 27)],
        ),
        (
            "page-block",
            "last",
            "brace",
            256,
            true,
            1,
            &[(InvalidAtRuleBody, DropAtRule, 26, 27)],
        ),
        (
            "page-block",
            "last",
            "brace",
            257,
            true,
            1,
            &[(InvalidAtRuleBody, DropAtRule, 26, 27)],
        ),
        (
            "page-block",
            "last",
            "brace",
            1000,
            true,
            1,
            &[(InvalidAtRuleBody, DropAtRule, 26, 27)],
        ),
        (
            "page-block",
            "followed",
            "brace",
            256,
            true,
            2,
            &[(InvalidAtRuleBody, DropAtRule, 26, 27)],
        ),
        (
            "page-block",
            "followed",
            "brace",
            1000,
            true,
            2,
            &[(InvalidAtRuleBody, DropAtRule, 26, 27)],
        ),
        (
            "page-block",
            "beside",
            "brace",
            256,
            true,
            2,
            &[(InvalidAtRuleBody, DropAtRule, 31, 32)],
        ),
        (
            "page-block",
            "beside",
            "brace",
            257,
            true,
            2,
            &[(InvalidAtRuleBody, DropAtRule, 31, 32)],
        ),
        (
            "page-block",
            "last",
            "paren",
            255,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 537)],
        ),
        (
            "page-block",
            "last",
            "paren",
            256,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 539)],
        ),
        (
            "page-block",
            "last",
            "paren",
            257,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 541)],
        ),
        (
            "page-block",
            "last",
            "paren",
            1000,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 2027)],
        ),
        (
            "page-block",
            "followed",
            "paren",
            256,
            true,
            2,
            &[(UnexpectedToken, DropAtRule, 26, 539)],
        ),
        (
            "page-block",
            "followed",
            "paren",
            1000,
            true,
            2,
            &[(UnexpectedToken, DropAtRule, 26, 2027)],
        ),
        (
            "page-block",
            "beside",
            "paren",
            256,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 31, 555)],
        ),
        (
            "page-block",
            "beside",
            "paren",
            257,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 31, 557)],
        ),
        (
            "page-block",
            "last",
            "square",
            255,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 537)],
        ),
        (
            "page-block",
            "last",
            "square",
            256,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 539)],
        ),
        (
            "page-block",
            "last",
            "square",
            257,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 541)],
        ),
        (
            "page-block",
            "last",
            "square",
            1000,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 2027)],
        ),
        (
            "page-block",
            "followed",
            "square",
            256,
            true,
            2,
            &[(UnexpectedToken, DropAtRule, 26, 539)],
        ),
        (
            "page-block",
            "followed",
            "square",
            1000,
            true,
            2,
            &[(UnexpectedToken, DropAtRule, 26, 2027)],
        ),
        (
            "page-block",
            "beside",
            "square",
            256,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 31, 555)],
        ),
        (
            "page-block",
            "beside",
            "square",
            257,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 31, 557)],
        ),
        (
            "page-block",
            "last",
            "function",
            255,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 792)],
        ),
        (
            "page-block",
            "last",
            "function",
            256,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 795)],
        ),
        (
            "page-block",
            "last",
            "function",
            257,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 798)],
        ),
        (
            "page-block",
            "last",
            "function",
            1000,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 26, 3027)],
        ),
        (
            "page-block",
            "followed",
            "function",
            256,
            true,
            2,
            &[(UnexpectedToken, DropAtRule, 26, 795)],
        ),
        (
            "page-block",
            "followed",
            "function",
            1000,
            true,
            2,
            &[(UnexpectedToken, DropAtRule, 26, 3027)],
        ),
        (
            "page-block",
            "beside",
            "function",
            256,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 31, 811)],
        ),
        (
            "page-block",
            "beside",
            "function",
            257,
            true,
            1,
            &[(UnexpectedEnd, DropAtRule, 31, 814)],
        ),
        (
            "sheet",
            "last",
            "brace",
            255,
            true,
            1001,
            &[
                (NestingLimit, StopAtNestingLimit, 34, 544),
                (InvalidAtRuleBody, DropAtRule, 581, 582),
            ],
        ),
        (
            "sheet",
            "last",
            "brace",
            256,
            true,
            1001,
            &[
                (NestingLimit, StopAtNestingLimit, 34, 546),
                (InvalidAtRuleBody, DropAtRule, 583, 584),
            ],
        ),
        (
            "sheet",
            "last",
            "brace",
            257,
            true,
            1001,
            &[
                (NestingLimit, StopAtNestingLimit, 34, 548),
                (InvalidAtRuleBody, DropAtRule, 585, 586),
            ],
        ),
        (
            "sheet",
            "last",
            "brace",
            1000,
            true,
            1001,
            &[
                (NestingLimit, StopAtNestingLimit, 34, 2034),
                (InvalidAtRuleBody, DropAtRule, 2071, 2072),
            ],
        ),
        (
            "sheet",
            "followed",
            "brace",
            256,
            true,
            2002,
            &[
                (NestingLimit, StopAtNestingLimit, 34, 546),
                (InvalidAtRuleBody, DropAtRule, 596, 597),
            ],
        ),
        (
            "sheet",
            "followed",
            "brace",
            1000,
            true,
            2002,
            &[
                (NestingLimit, StopAtNestingLimit, 34, 2034),
                (InvalidAtRuleBody, DropAtRule, 2084, 2085),
            ],
        ),
        (
            "sheet",
            "beside",
            "brace",
            256,
            true,
            2002,
            &[
                (NestingLimit, StopAtNestingLimit, 39, 551),
                (InvalidAtRuleBody, DropAtRule, 604, 605),
            ],
        ),
        (
            "sheet",
            "beside",
            "brace",
            257,
            true,
            2002,
            &[
                (NestingLimit, StopAtNestingLimit, 39, 553),
                (InvalidAtRuleBody, DropAtRule, 606, 607),
            ],
        ),
        (
            "sheet",
            "last",
            "paren",
            255,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 545),
                (UnexpectedEnd, DropAtRule, 581, 1092),
            ],
        ),
        (
            "sheet",
            "last",
            "paren",
            256,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 547),
                (UnexpectedEnd, DropAtRule, 583, 1096),
            ],
        ),
        (
            "sheet",
            "last",
            "paren",
            257,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 549),
                (UnexpectedEnd, DropAtRule, 585, 1100),
            ],
        ),
        (
            "sheet",
            "last",
            "paren",
            1000,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 2035),
                (UnexpectedEnd, DropAtRule, 2071, 4072),
            ],
        ),
        (
            "sheet",
            "followed",
            "paren",
            256,
            true,
            2002,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 547),
                (UnexpectedToken, DropAtRule, 596, 1109),
            ],
        ),
        (
            "sheet",
            "followed",
            "paren",
            1000,
            true,
            2002,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 2035),
                (UnexpectedToken, DropAtRule, 2084, 4085),
            ],
        ),
        (
            "sheet",
            "beside",
            "paren",
            256,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 39, 563),
                (UnexpectedEnd, DropAtRule, 604, 1128),
            ],
        ),
        (
            "sheet",
            "beside",
            "paren",
            257,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 39, 565),
                (UnexpectedEnd, DropAtRule, 606, 1132),
            ],
        ),
        (
            "sheet",
            "last",
            "square",
            255,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 545),
                (UnexpectedEnd, DropAtRule, 581, 1092),
            ],
        ),
        (
            "sheet",
            "last",
            "square",
            256,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 547),
                (UnexpectedEnd, DropAtRule, 583, 1096),
            ],
        ),
        (
            "sheet",
            "last",
            "square",
            257,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 549),
                (UnexpectedEnd, DropAtRule, 585, 1100),
            ],
        ),
        (
            "sheet",
            "last",
            "square",
            1000,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 2035),
                (UnexpectedEnd, DropAtRule, 2071, 4072),
            ],
        ),
        (
            "sheet",
            "followed",
            "square",
            256,
            true,
            2002,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 547),
                (UnexpectedToken, DropAtRule, 596, 1109),
            ],
        ),
        (
            "sheet",
            "followed",
            "square",
            1000,
            true,
            2002,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 2035),
                (UnexpectedToken, DropAtRule, 2084, 4085),
            ],
        ),
        (
            "sheet",
            "beside",
            "square",
            256,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 39, 563),
                (UnexpectedEnd, DropAtRule, 604, 1128),
            ],
        ),
        (
            "sheet",
            "beside",
            "square",
            257,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 39, 565),
                (UnexpectedEnd, DropAtRule, 606, 1132),
            ],
        ),
        (
            "sheet",
            "last",
            "function",
            255,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 800),
                (UnexpectedEnd, DropAtRule, 836, 1602),
            ],
        ),
        (
            "sheet",
            "last",
            "function",
            256,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 803),
                (UnexpectedEnd, DropAtRule, 839, 1608),
            ],
        ),
        (
            "sheet",
            "last",
            "function",
            257,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 806),
                (UnexpectedEnd, DropAtRule, 842, 1614),
            ],
        ),
        (
            "sheet",
            "last",
            "function",
            1000,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 3035),
                (UnexpectedEnd, DropAtRule, 3071, 6072),
            ],
        ),
        (
            "sheet",
            "followed",
            "function",
            256,
            true,
            2002,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 803),
                (UnexpectedToken, DropAtRule, 852, 1621),
            ],
        ),
        (
            "sheet",
            "followed",
            "function",
            1000,
            true,
            2002,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 34, 3035),
                (UnexpectedToken, DropAtRule, 3084, 6085),
            ],
        ),
        (
            "sheet",
            "beside",
            "function",
            256,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 39, 819),
                (UnexpectedEnd, DropAtRule, 860, 1640),
            ],
        ),
        (
            "sheet",
            "beside",
            "function",
            257,
            true,
            1001,
            &[
                (InvalidQualifiedRule, DropQualifiedRule, 39, 822),
                (UnexpectedEnd, DropAtRule, 863, 1646),
            ],
        ),
    ];
    let parser = CssParser::default();
    let context = CssParserContext::default();
    for (entry, shape, opener, depth, has_syntax, kept, expected) in TABLE {
        let (open, close) = match *opener {
            "brace" => ("{", "}"),
            "paren" => ("(", ")"),
            "square" => ("[", "]"),
            _ => ("f(", ")"),
        };
        let nest = format!("{}{}", open.repeat(*depth), close.repeat(*depth));
        let body = match *shape {
            "last" => format!("color: red; {nest}"),
            "followed" => format!("color: red; {nest}; color: blue"),
            _ => format!("margin-top: 1px; {nest} color: red"),
        };
        let (has_syntax_now, kept_now, diagnostics) = match *entry {
            "kf-body" => {
                let source = format!("{{ from {{ {body} }} }}");
                let report = fragments::parse_keyframes_block_with_context(&source, context);
                let kept = report.syntax().as_ref().map_or(0, |fragment| {
                    fragment
                        .body()
                        .iter()
                        .map(|block| block.declarations().len())
                        .sum()
                });
                (report.syntax().is_some(), kept, dropped_with_spans(&report))
            }
            "kf-rule" => {
                let source = format!("from {{ {body} }}");
                let report = CssParsedKeyframeRule::parse(&parser, &source);
                let kept = report
                    .syntax()
                    .as_ref()
                    .map_or(0, |rule| rule.block().declarations().len());
                (report.syntax().is_some(), kept, dropped_with_spans(&report))
            }
            "page-block" => {
                let source = format!("{{ @top-left {{ {body} }} }}");
                let report = fragments::parse_page_block_with_context(&source, context);
                let kept = report.syntax().as_ref().map_or(0, |fragment| {
                    fragment
                        .body()
                        .margin_rules()
                        .iter()
                        .map(|margin| margin.declarations().properties().len())
                        .sum()
                });
                (report.syntax().is_some(), kept, dropped_with_spans(&report))
            }
            _ => {
                let source = format!(
                    "@keyframes k {{ from {{ {body} }} }} @page {{ @top-left {{ {body} }} }}"
                );
                let report = parser.parse_sheet(&source);
                let mut kept = 0;
                for rule in report.syntax().rules() {
                    match rule {
                        CssRule::Keyframes(rule) => {
                            kept += 1000
                                * rule
                                    .blocks()
                                    .iter()
                                    .map(|block| block.declarations().len())
                                    .sum::<usize>();
                        }
                        CssRule::Page(rule) => {
                            kept += rule
                                .margin_rules()
                                .iter()
                                .map(|margin| margin.declarations().properties().len())
                                .sum::<usize>();
                        }
                        _ => {}
                    }
                }
                (true, kept, dropped_with_spans(&report))
            }
        };
        assert_eq!(
            (has_syntax_now, kept_now, diagnostics),
            (*has_syntax, *kept, expected.to_vec()),
            "{entry} {shape} {opener} {depth}"
        );
    }
}

fn dropped_with_spans<T>(
    report: &CssParseReport<T>,
) -> Vec<(CssErrorCode, CssRecoveryAction, usize, usize)> {
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let (start, end) = start(diagnostic.span());
            (diagnostic.error().code(), diagnostic.action(), start, end)
        })
        .collect()
}
