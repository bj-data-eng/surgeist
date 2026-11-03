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

/// The components compared by lookups by path while `work` ran, and what it
/// returned.
fn compared<T>(work: impl FnOnce() -> T) -> (usize, T) {
    crate::component::values::PATH_COMPARISONS.with(|compared| compared.set(0));
    let result = work();
    (
        crate::component::values::PATH_COMPARISONS.with(std::cell::Cell::get),
        result,
    )
}

/// Constructing from `prefix`, `count` copies of `unit` and `suffix` as
/// components, which `construct` accepts, locates positions and looks components
/// up by path in proportion to the components, not to their product.
fn assert_construction_linear(
    prefix: &str,
    unit: &str,
    suffix: &str,
    construct: impl Fn(crate::component::CssComponentValues) -> bool,
) {
    let mut previous: Option<(usize, usize)> = None;
    for count in [100_usize, 200, 400] {
        let text = format!("{prefix}{}{suffix}", unit.repeat(count));
        let values = crate::component::CssComponentValues::parse(&parser(), &text).unwrap();
        let (walked_bytes, (path_comparisons, accepted)) =
            walked(|| compared(|| construct(values)));
        assert!(accepted, "{unit:?} x {count} is constructed");
        assert!(
            walked_bytes <= bound(text.len(), count),
            "{unit:?} x {count}: walked {walked_bytes} bytes of {}",
            text.len()
        );
        // A lookup compares about as many paths as the binary logarithm of
        // the components, and the construction looks up a few per unit.
        assert!(
            path_comparisons <= 16 * count * (usize::BITS - count.leading_zeros()) as usize,
            "{unit:?} x {count}: compared {path_comparisons} paths"
        );
        if let Some((before_bytes, before_comparisons)) = previous {
            // Doubling the count roughly doubles the work; a scan or a walk
            // from the start for each lookup would multiply it by four.
            assert!(
                walked_bytes <= 3 * before_bytes,
                "{unit:?} x {count}: {walked_bytes} bytes after {before_bytes}"
            );
            assert!(
                path_comparisons <= 3 * before_comparisons,
                "{unit:?} x {count}: {path_comparisons} comparisons after {before_comparisons}"
            );
        }
        previous = Some((walked_bytes, path_comparisons));
    }
}

#[test]
fn media_queries_of_many_features_construct_in_linear_work() {
    let limits = crate::component::CssComponentValueLimits::default();
    for (unit, suffix) in [
        ("(width: 1px) and ", "(height)"),
        ("(width: 1px) or ", "(height)"),
        ("(100px <= width <= 200px) and ", "(height)"),
    ] {
        assert_construction_linear("", unit, suffix, |values| {
            crate::query::media::CssMediaQuery::try_from_components(values, limits).is_ok()
        });
        assert_construction_linear("", unit, suffix, |values| {
            crate::query::CssCondition::<crate::query::media::CssMediaLeaf>::try_from_components(
                values, limits,
            )
            .is_ok()
        });
    }
}

#[test]
fn custom_media_preludes_of_many_features_construct_in_linear_work() {
    let limits = crate::component::CssComponentValueLimits::default();
    assert_construction_linear("--c ", "(width: 1px) and ", "(height)", |values| {
        crate::rule::CssCustomMediaRule::try_from_components(values, limits).is_ok()
    });
}

#[test]
fn font_face_source_lists_of_many_members_construct_in_linear_work() {
    let limits = crate::component::CssComponentValueLimits::default();
    let kind = crate::rule::font_face::CssFontFaceDescriptorKind::from_css_name("src").unwrap();
    assert_construction_linear("", "url(a) format(woff), ", "local(x)", |values| {
        crate::rule::font_face::CssAuthoredFontFaceDescriptorValue::try_from_components(
            kind, values, limits,
        )
        .is_ok()
    });
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

/// A parse of the sheet `text_of(count)` for each count does searches in
/// proportion to the count, as `read` counts them: within `per_item` for each
/// item and each step of a binary search, and about doubling when the count
/// does. A search over everything kept so far for each item multiplies the work
/// by four.
fn assert_searches_linear(
    text_of: impl Fn(usize) -> String,
    parse: impl Fn(&str) -> usize,
    reset: fn(),
    read: fn() -> usize,
    per_item: usize,
    diagnostics: impl Fn(usize) -> usize,
) {
    let mut previous = None;
    for count in [200_usize, 400, 800] {
        let text = text_of(count);
        reset();
        let found = parse(&text);
        let work = read();
        assert!(
            found >= diagnostics(count),
            "x {count}: {found} diagnostics"
        );
        assert!(
            work <= per_item * count * (usize::BITS - count.leading_zeros()) as usize,
            "x {count}: {work} searches"
        );
        if let Some(before) = previous {
            assert!(
                work <= 3 * before,
                "x {count}: {work} searches after {before}"
            );
        }
        previous = Some(work);
    }
}

fn sheet_diagnostics(text: &str) -> usize {
    parser().parse_sheet(text).diagnostics().len()
}

fn reset_ranges_examined() {
    super::conditional_chains::RANGES_EXAMINED.with(|examined| examined.set(0));
}

fn ranges_examined() -> usize {
    super::conditional_chains::RANGES_EXAMINED.with(std::cell::Cell::get)
}

fn reset_namespace_probes() {
    super::NAMESPACE_PROBES.with(|probes| probes.set(0));
}

fn namespace_probes() -> usize {
    super::NAMESPACE_PROBES.with(std::cell::Cell::get)
}

#[test]
fn diagnostics_among_many_rejected_rules_are_filtered_in_linear_work() {
    for unit in ["a{x:y}\n@else {}\n", "@media x{@else {} a{x:y}}\n"] {
        assert_searches_linear(
            |count| unit.repeat(count),
            sheet_diagnostics,
            reset_ranges_examined,
            ranges_examined,
            4,
            |count| count,
        );
    }
}

#[test]
fn many_distinct_namespace_prefixes_are_bound_and_read_in_linear_work() {
    assert_searches_linear(
        |count| {
            let declarations: String = (0..count)
                .map(|i| format!("@namespace p{i} url(x{i});\n"))
                .collect();
            let uses: String = (0..count).map(|i| format!("p{i}|a {{}}\n")).collect();
            declarations + &uses
        },
        sheet_diagnostics,
        reset_namespace_probes,
        namespace_probes,
        8,
        |_| 0,
    );
}

#[test]
fn a_namespace_prefix_declared_again_is_found_among_many_in_linear_work() {
    assert_searches_linear(
        |count| {
            (0..2 * count)
                .map(|i| format!("@namespace p{} url(x{i});\n", i % count))
                .collect()
        },
        sheet_diagnostics,
        reset_namespace_probes,
        namespace_probes,
        8,
        |count| count,
    );
}

#[test]
fn many_distinct_namespace_rules_are_checked_as_a_sheet_in_linear_work() {
    assert_searches_linear(
        |count| {
            (0..count)
                .map(|i| format!("@namespace p{i} url(x{i});\n"))
                .collect()
        },
        |text| {
            let rules = parser().parse_sheet(text).syntax().rules().to_vec();
            // Parsing binds every prefix; only the construction check is measured.
            reset_namespace_probes();
            crate::rule::CssSheet::try_from_rules(rules).map_or(0, |_| 1)
        },
        || (),
        namespace_probes,
        8,
        |_| 1,
    );
}
