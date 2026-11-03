//! Reporting many diagnostics of one source does work in proportion to the
//! source and the diagnostics, not to their product (#1563).
//!
//! The work measure is a counter, not time: the bytes of source text that
//! position resolution walked. A diagnostic found by a walk from the start of
//! the source adds the bytes before it, so `count` diagnostics of a source of
//! `length` bytes would add about `count * length / 2`. With the checkpoints of
//! the source snapshot, one diagnostic resolves its few positions from the
//! checkpoint before each, and the source is walked once in all.

use crate::source::position::WALKED_BYTES;

/// The bytes position resolution walked while `parse` ran, and what it returned.
fn walked<T>(parse: impl FnOnce() -> T) -> (usize, T) {
    WALKED_BYTES.with(|walked| walked.set(0));
    let result = parse();
    (WALKED_BYTES.with(std::cell::Cell::get), result)
}

/// The most that `count` diagnostics of a source of `length` bytes may cost:
/// each position the parse locates (a token, a diagnostic) is found from the
/// checkpoint before it, which is fewer than 64 bytes back, and the
/// source holds fewer positions than bytes.
fn bound(length: usize, count: usize) -> usize {
    80 * length + 512 * count
}

/// A parse of `count` copies of `unit`, which each report a diagnostic, takes
/// work within the bound at every size, and grows with the count about as the
/// source does.
fn assert_linear<T>(unit: &str, diagnostics: impl Fn(&T) -> usize, parse: impl Fn(&str) -> T) {
    let mut previous = None;
    for count in [100_usize, 200, 400] {
        let source = unit.repeat(count);
        let (work, result) = walked(|| parse(&source));
        assert!(
            diagnostics(&result) >= count,
            "{unit:?} x {count}: {} diagnostics",
            diagnostics(&result)
        );
        assert!(
            work <= bound(source.len(), count),
            "{unit:?} x {count}: walked {work} bytes of {}",
            source.len()
        );
        if let Some(before) = previous {
            // Doubling the count roughly doubles the work; a walk from the
            // start for each diagnostic would multiply it by four.
            assert!(
                work <= 3 * before,
                "{unit:?} x {count}: {work} bytes after {before}"
            );
        }
        previous = Some(work);
    }
}

fn parser() -> crate::CssParser {
    crate::CssParser::default()
}

#[test]
fn failing_declarations_of_a_style_rule_resolve_in_linear_work() {
    for declaration in ["unknown-property: 1;\n", "width: red;\n", "width: ;\n"] {
        assert_linear(
            declaration,
            |report: &crate::CssParseReport<_>| report.diagnostics().len(),
            |source| parser().parse_sheet(format!("a {{\n{source}}}")),
        );
    }
}

#[test]
fn failing_declarations_of_a_block_resolve_in_linear_work() {
    assert_linear(
        "unknown-property: 1;\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_block_contents(source),
    );
}

#[test]
fn failing_declarations_of_a_keyframe_block_resolve_in_linear_work() {
    assert_linear(
        "unknown-property: 1;\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| {
            crate::rule::keyframes::CssParsedKeyframeRule::parse(
                &parser(),
                format!("from {{\n{source}}}"),
            )
        },
    );
}

#[test]
fn failing_declarations_of_a_page_margin_block_resolve_in_linear_work() {
    assert_linear(
        "unknown-property: 1;\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| {
            crate::rule::page::CssMarginRule::parse(&parser(), format!("@top-left {{\n{source}}}"))
        },
    );
}

#[test]
fn failing_rules_of_a_sheet_resolve_in_linear_work() {
    for rule in [
        "@unknown-rule;\n",
        "a { b: c }\n",
        "1 { color: red }\n",
        "@import;\n",
    ] {
        assert_linear(
            rule,
            |report: &crate::CssParseReport<_>| report.diagnostics().len(),
            |source| parser().parse_sheet(source),
        );
    }
}

#[test]
fn failing_rules_of_a_nested_list_resolve_in_linear_work() {
    assert_linear(
        "@unknown-rule;\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(format!("@media screen {{\n{source}}}")),
    );
    assert_linear(
        "& b { c: d }\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(format!("a {{\n{source}}}")),
    );
}

#[test]
fn failing_members_of_a_selector_list_resolve_in_linear_work() {
    assert_linear(
        ":is(1), a, ",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(format!("{source} b {{ c: d }}")),
    );
}

#[test]
fn failing_members_of_a_source_list_resolve_in_linear_work() {
    assert_linear(
        "bogus(x), ",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(format!("@font-face {{ src: {source} url(y) }}")),
    );
}

#[test]
fn refused_blocks_of_statement_at_rules_resolve_in_linear_work() {
    assert_linear(
        "@import 'a' {}\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(source),
    );
    assert_linear(
        "@charset 'a' {}\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(format!("a {{\n{source}}}")),
    );
}

#[test]
fn block_at_rules_without_a_block_resolve_in_linear_work() {
    for unit in ["@page;\n", "@counter-style c;\n", "@media x;\n"] {
        assert_linear(
            unit,
            |report: &crate::CssParseReport<_>| report.diagnostics().len(),
            |source| parser().parse_sheet(source),
        );
    }
}

#[test]
fn rules_of_a_chain_that_are_out_of_place_resolve_in_linear_work() {
    assert_linear(
        "@else {}\n",
        |report: &crate::CssParseReport<_>| report.diagnostics().len(),
        |source| parser().parse_sheet(source),
    );
}

#[test]
fn a_rule_out_of_place_in_a_chain_reads_the_snapshot_of_the_sheet() {
    // Rejecting an `@else` locates it among the bytes of the sheet. It reads
    // the checkpoints of the snapshot the parse holds, so it walks about the
    // bytes between two checkpoints, not the whole sheet again.
    for count in [200_usize, 400, 800] {
        let padding = "a{x:y}\n".repeat(count);
        let (without, _) = walked(|| parser().parse_sheet(&padding));
        let (with, report) = walked(|| parser().parse_sheet(format!("{padding}@else {{}}")));
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action()
                    == crate::diagnostic::CssRecoveryAction::DropAtRule),
            "the else is dropped"
        );
        assert!(
            with <= without + 512,
            "x {count}: walked {with} bytes with the else and {without} without it"
        );
    }
}
