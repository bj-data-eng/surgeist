//! Independent authored contract: Text4 WD20260814 #propdef-line-padding,
//! #typedef-autospace, #typedef-spacing-trim, #propdef-text-spacing,
//! #propdef-hanging-punctuation; Values3/4 §2.2 requires both grouping barriers.
//! The pinned signed Length range, grammar order and literal projections supply
//! expected values. Transparent finite visits are adopted product accounting.
//! Existing public front doors only. New typed constructors/borrowed views,
//! mode presence/effective semantics, primitive-provider and private suppression
//! tests accompany functional implementation. No contextual glyph computation.

use crate::css::support::{
    authored_checked_declaration as checked, authored_parsed_declaration as parsed,
    known_property as property,
};
use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

use crate::css::support::authored_property;
use crate::css::support::check_named_components as checked_components;
use crate::css::support::named_property::invalid_named_value as invalid;
use crate::css::support::named_property::named_fronts as fronts;
use crate::css::support::serialization_limits::LimitLadder;
use authored_property::assert_source;

fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed intrinsic contribution")
    };
    let name = source.known().unwrap().property().canonical_name();
    assert_eq!(values.items().len(), terminals(name).len());
    for (item, name) in values.items().iter().zip(terminals(name)) {
        assert_eq!(item.property(), property(name));
        assert_source(item, source);
    }
    values
}
fn canonical(source: &CssDeclaration, name: &str, text: &str) {
    assert_eq!(
        source.to_specified_css().unwrap(),
        format!("{name}: {text} !important;")
    );
}
fn accepted(name: &str, input: &str, expected: &str) {
    for source in fronts(name, input) {
        let before = source.clone();
        canonical(&source, name, expected);
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        for item in contributions(&source).items() {
            assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                item.property()
            );
            assert!(item.replacement_components().is_none());
        }
        assert_eq!(source, before);
    }
    canonical(&checked(name, expected, true), name, expected);
}
fn permutations(words: &[&str]) -> Vec<String> {
    fn walk(words: &[&str], used: &mut Vec<bool>, prefix: &mut Vec<String>, out: &mut Vec<String>) {
        if prefix.len() == words.len() {
            out.push(prefix.join(" "));
            return;
        }
        for i in 0..words.len() {
            if !used[i] {
                used[i] = true;
                prefix.push(words[i].to_owned());
                walk(words, used, prefix, out);
                prefix.pop();
                used[i] = false;
            }
        }
    }
    let mut out = Vec::new();
    walk(
        words,
        &mut vec![false; words.len()],
        &mut Vec::new(),
        &mut out,
    );
    out
}
#[test]
fn programmatic_components_preserve_origin_identity_and_importance_at_both_checked_fronts() {
    for name in FAMILY {
        let text = ordinary_cases(name)[0];
        let value = if name == "line-padding" {
            CssComponentValue::try_number("0").unwrap()
        } else {
            CssComponentValue::try_ident(text).unwrap()
        };
        let components = CssComponentValues::try_new(vec![value]).unwrap();
        for grammar in [false, true] {
            let source = checked_components(name, components.clone(), grammar).unwrap();
            assert_eq!(source.value_components(), &components);
            assert_eq!(
                source.value_components().items()[0].origin(),
                &CssValueOrigin::Programmatic
            );
            assert!(source.parsed_value().is_none());
            assert_eq!(source.importance(), CssImportance::Important);
            canonical(&source, name, text);
            contributions(&source);
        }
    }
}
#[test]
fn intrinsic_initials_and_inheritance_equal_authored_terminal_values_without_fabricated_sources() {
    for (name, text, inherited) in INITIALS {
        let p = property(name);
        assert_eq!(P::from_name(&name.to_ascii_uppercase()), Some(p));
        let CssPropertyKindRef::Longhand(meta) = p.metadata().unwrap().kind() else {
            panic!("intrinsic longhand")
        };
        assert_eq!(meta.property().known_property(), p);
        assert_eq!(meta.inherited_by_default(), inherited);
        let initial = meta.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("ordinary initial")
        };
        assert_eq!(value.property().known_property(), p);
        // Numeric equality includes origin. Compare the zero initial against
        // an exact programmatic zero; a parsed zero has distinct provenance.
        let declaration = if name == "line-padding" {
            checked_components(
                name,
                CssComponentValues::try_new(vec![CssComponentValue::try_number("0").unwrap()])
                    .unwrap(),
                true,
            )
            .unwrap()
        } else {
            checked(name, text, true)
        };
        let values = contributions(&declaration);
        assert_eq!(values.items()[0].ordinary_value(), Some(value));
    }
}
#[test]
fn normalization_counts_complete_terminals_and_pending_occurrences_cumulatively() {
    for name in FAMILY {
        let css = format!(
            "@media screen{{.a{{{name}:{}!important;{name}:var(--text);{name}:unset!important;color:red}}}}",
            ordinary_cases(name)[0]
        );
        let report = parse_sheet(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let before = report.clone();
        let count = terminals(name).len() * 2 + 2;
        let exact = CssNormalizationLimits::try_new(1, 2, 4, count).unwrap();
        let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|v| match v {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 4);
        for (index, item) in declarations.iter().enumerate() {
            assert_eq!(item.order(), index);
            assert_eq!(
                item.source().known().unwrap().property(),
                if index == 3 { P::Color } else { property(name) }
            );
            assert_eq!(
                item.source().importance(),
                if index == 0 || index == 2 {
                    CssImportance::Important
                } else {
                    CssImportance::Normal
                }
            );
            assert!(
                item.selector_context()
                    .same_context(declarations[0].selector_context())
            );
            assert!(
                item.rule_context()
                    .same_context(declarations[0].rule_context())
            );
            if index > 0 {
                assert!(
                    !item
                        .source()
                        .same_occurrence(declarations[index - 1].source())
                );
            }
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert_eq!(index, 1);
                    assert!(handle.source().same_occurrence(item.source()));
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(
                        values.items().len(),
                        if index == 3 { 1 } else { terminals(name).len() }
                    );
                    for value in values.items() {
                        assert_source(value, item.source());
                        if index == 2 {
                            assert!(matches!(
                                value.value(),
                                CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                            ));
                        } else {
                            assert!(matches!(
                                value.value(),
                                CssContributionValueRef::Ordinary(_)
                            ));
                        }
                    }
                }
                _ => panic!("intrinsic authored lifecycle"),
            }
        }
        for (limits, resource, limit) in [
            (
                CssNormalizationLimits::try_new(1, 2, 4, count - 1).unwrap(),
                CssNormalizationResource::Contributions,
                count - 1,
            ),
            (
                CssNormalizationLimits::try_new(1, 2, 3, count).unwrap(),
                CssNormalizationResource::Declarations,
                3,
            ),
        ] {
            for _ in 0..2 {
                let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
                assert_eq!(
                    error.kind(),
                    &CssNormalizationErrorKind::LimitExceeded { resource, limit }
                );
                assert_eq!(error.declaration_order(), Some(3));
                assert!(
                    error
                        .declaration()
                        .unwrap()
                        .same_occurrence(declarations[3].source())
                );
            }
        }
        assert_eq!(report, before);
        assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
    }
}
fn declaration_limits(name: &str, text: &str, nodes: usize) {
    for source in [parsed(name, text), checked(name, text, true)] {
        let expected = format!("{name}: {text} !important;");
        LimitLadder::new(&expected, nodes + 2, nodes + 2)
            .assert_unchanged(&source, CssDeclaration::to_specified_css_with_limits);
    }
}
#[test]
fn every_emitted_keyword_and_literal_charges_exact_declaration_work_and_utf8_bytes_atomically() {
    for name in FAMILY {
        for text in budget_cases(name) {
            declaration_limits(name, &text, text.split_whitespace().count());
        }
    }
}
fn sheet_limits(input: &str, expected: &str, nodes: usize) {
    let report = parse_sheet(input);
    assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    // Each rule fits separately; exhaustion only comes from cumulative use.
    LimitLadder::new(expected, nodes, nodes).assert_cumulative_sheet(report.syntax(), 1);
}

const FAMILY: [&str; 5] = [
    "line-padding",
    "text-autospace",
    "text-spacing-trim",
    "text-spacing",
    "hanging-punctuation",
];
const TRIM: [&str; 6] = [
    "space-all",
    "normal",
    "space-first",
    "trim-start",
    "trim-both",
    "trim-all",
];
const FLAGS: [&str; 3] = ["ideograph-alpha", "ideograph-numeric", "punctuation"];
const INITIALS: [(&str, &str, bool); 4] = [
    ("line-padding", "0", true),
    ("text-autospace", "normal", true),
    ("text-spacing-trim", "normal", true),
    ("hanging-punctuation", "none", true),
];
fn terminals(name: &str) -> &'static [&'static str] {
    match name {
        "line-padding" => &["line-padding"],
        "text-autospace" => &["text-autospace"],
        "text-spacing-trim" => &["text-spacing-trim"],
        "text-spacing" => &["text-spacing-trim", "text-autospace"],
        "hanging-punctuation" => &["hanging-punctuation"],
        _ => panic!("family"),
    }
}
fn ordinary_cases(name: &str) -> &'static [&'static str] {
    match name {
        "line-padding" => &["0", "-2px", "1.25em"],
        "text-autospace" => &[
            "normal",
            "auto",
            "no-autospace",
            "insert",
            "replace",
            "ideograph-alpha",
            "ideograph-alpha punctuation replace",
        ],
        "text-spacing-trim" => &[
            "normal",
            "space-all",
            "space-first",
            "trim-start",
            "trim-both",
            "trim-all",
            "auto",
        ],
        "text-spacing" => &[
            "none",
            "auto",
            "normal",
            "trim-start",
            "insert",
            "replace",
            "no-autospace",
            "trim-both ideograph-alpha punctuation replace",
        ],
        "hanging-punctuation" => &[
            "none",
            "first",
            "last",
            "force-end",
            "allow-end",
            "first force-end last",
            "first allow-end last",
        ],
        _ => panic!("family"),
    }
}
fn budget_cases(name: &str) -> Vec<String> {
    match name {
        "text-autospace" => autospace_constituents()
            .into_iter()
            .map(|(text, _)| text)
            .chain(["normal".to_owned(), "auto".to_owned()])
            .collect(),
        "text-spacing" => {
            let mut values = vec!["none".to_owned(), "auto".to_owned()];
            values.extend(TRIM.into_iter().map(str::to_owned));
            for (auto, _) in autospace_constituents() {
                values.push(auto.clone());
                values.extend(TRIM.into_iter().map(|trim| format!("{trim} {auto}")));
            }
            values
        }
        "hanging-punctuation" => {
            let mut values = vec!["none".to_owned()];
            for first in [false, true] {
                for end in [None, Some("force-end"), Some("allow-end")] {
                    for last in [false, true] {
                        let text = first
                            .then_some("first")
                            .into_iter()
                            .chain(end)
                            .chain(last.then_some("last"))
                            .collect::<Vec<_>>()
                            .join(" ");
                        if !text.is_empty() {
                            values.push(text);
                        }
                    }
                }
            }
            values
        }
        _ => ordinary_cases(name)
            .iter()
            .map(|text| (*text).to_owned())
            .collect(),
    }
}
fn autospace_constituents() -> Vec<(String, Vec<String>)> {
    let mut cases = vec![("no-autospace".to_owned(), vec!["no-autospace".to_owned()])];
    for mask in 0..8 {
        let flags: Vec<&str> = FLAGS
            .iter()
            .enumerate()
            .filter_map(|(i, s)| (mask & (1 << i) != 0).then_some(*s))
            .collect();
        for mode in [None, Some("insert"), Some("replace")] {
            if flags.is_empty() && mode.is_none() {
                continue;
            }
            let canonical = flags
                .iter()
                .copied()
                .chain(mode)
                .collect::<Vec<_>>()
                .join(" ");
            let mut inputs = Vec::new();
            for ordered in permutations(&flags) {
                if let Some(mode) = mode {
                    if ordered.is_empty() {
                        inputs.push(mode.to_owned());
                    } else {
                        inputs.push(format!("{ordered} {mode}"));
                        inputs.push(format!("{mode} {ordered}"));
                    }
                } else {
                    inputs.push(ordered);
                }
            }
            cases.push((canonical, inputs));
        }
    }
    cases
}
fn projected(source: &CssDeclaration, expected: [&str; 2]) {
    let values = contributions(source);
    for ((item, name), text) in values
        .items()
        .iter()
        .zip(terminals("text-spacing"))
        .zip(expected)
    {
        let direct = contributions(&checked(name, text, true));
        assert_eq!(
            item.ordinary_value(),
            direct.items()[0].ordinary_value(),
            "{name}: {text}"
        );
    }
}
#[test]
fn all_autospace_flag_subsets_orders_and_optional_modes_have_contiguous_group_admission() {
    for text in ["normal", "auto"] {
        accepted("text-autospace", text, text);
    }
    for (expected, inputs) in autospace_constituents() {
        for input in inputs {
            accepted("text-autospace", &input, &expected);
        }
    }
    for mask in 1..8 {
        let flags: Vec<_> = FLAGS
            .iter()
            .enumerate()
            .filter_map(|(i, s)| (mask & (1 << i) != 0).then_some(*s))
            .collect();
        if flags.len() < 2 {
            continue;
        }
        for ordered in permutations(&flags) {
            let ordered: Vec<_> = ordered.split_whitespace().collect();
            for cut in 1..ordered.len() {
                for mode in ["insert", "replace"] {
                    invalid(
                        "text-autospace",
                        &format!(
                            "{} {mode} {}",
                            ordered[..cut].join(" "),
                            ordered[cut..].join(" ")
                        ),
                    );
                }
            }
        }
    }
}
#[test]
fn autospace_duplicate_conflicting_exclusive_and_unknown_roles_reject_atomically() {
    for flag in FLAGS {
        invalid("text-autospace", &format!("{flag} {flag}"));
        invalid("text-autospace", &format!("insert {flag} {flag}"));
    }
    for a in ["insert", "replace"] {
        for b in ["insert", "replace"] {
            invalid("text-autospace", &format!("{a} {b}"));
            invalid("text-autospace", &format!("{a} ideograph-alpha {b}"));
        }
    }
    for branch in ["normal", "auto", "no-autospace"] {
        for other in [
            "normal",
            "auto",
            "no-autospace",
            "insert",
            "replace",
            "ideograph-alpha",
            "ideograph-numeric",
            "punctuation",
        ] {
            invalid("text-autospace", &format!("{branch} {other}"));
            invalid("text-autospace", &format!("{other} {branch}"));
        }
    }
    for text in [
        "",
        "none",
        "1",
        "25%",
        "ideograph-alpha,punctuation",
        "[insert]",
        "\"normal\"",
        "calc(1 + 2)",
    ] {
        invalid("text-autospace", text);
    }
}
#[test]
fn every_trim_longhand_keyword_is_complete_and_duplicates_or_mixtures_reject() {
    for text in TRIM.into_iter().chain(["auto"]) {
        accepted("text-spacing-trim", text, text);
    }
    for a in TRIM.into_iter().chain(["auto"]) {
        for b in TRIM.into_iter().chain(["auto"]) {
            invalid("text-spacing-trim", &format!("{a} {b}"));
        }
    }
    for text in [
        "",
        "none",
        "trim-end",
        "insert",
        "no-autospace",
        "1px",
        "25%",
        "normal,auto",
    ] {
        invalid("text-spacing-trim", text);
    }
}
#[test]
fn shorthand_special_branches_and_each_omitted_member_project_exactly_two_longhands() {
    for (input, expected, projection) in [
        ("none", "none", ["space-all", "no-autospace"]),
        ("auto", "auto", ["auto", "auto"]),
        ("normal", "normal", ["normal", "normal"]),
        ("trim-start", "trim-start", ["trim-start", "normal"]),
        ("insert", "insert", ["normal", "insert"]),
        ("replace", "replace", ["normal", "replace"]),
        ("no-autospace", "no-autospace", ["normal", "no-autospace"]),
        (
            "ideograph-alpha",
            "ideograph-alpha",
            ["normal", "ideograph-alpha"],
        ),
        (
            "normal no-autospace",
            "normal no-autospace",
            ["normal", "no-autospace"],
        ),
    ] {
        accepted("text-spacing", input, expected);
        for source in fronts("text-spacing", input) {
            projected(&source, projection);
        }
    }
    for trim in TRIM {
        for source in fronts("text-spacing", trim) {
            canonical(&source, "text-spacing", trim);
            projected(&source, [trim, "normal"]);
        }
    }
    let CssPropertyKindRef::Shorthand(meta) = property("text-spacing").metadata().unwrap().kind()
    else {
        panic!("two-member shorthand")
    };
    let names: Vec<_> = meta
        .members()
        .iter()
        .map(|p| p.known_property().canonical_name())
        .collect();
    assert_eq!(names, ["text-spacing-trim", "text-autospace"]);
    assert_eq!(meta.settable_members(), meta.members());
    assert!(meta.reset_only_members().is_empty());
    assert!(!meta.is_legacy());
}
#[test]
fn shorthand_all_complete_autospace_and_trim_orders_preserve_both_grouping_levels() {
    for (auto, inputs) in autospace_constituents() {
        for input in &inputs {
            accepted("text-spacing", input, &auto);
            projected(&checked("text-spacing", input, true), ["normal", &auto]);
        }
        for trim in TRIM {
            let expected = format!("{trim} {auto}");
            for input in &inputs {
                for text in [format!("{trim} {input}"), format!("{input} {trim}")] {
                    accepted("text-spacing", &text, &expected);
                    projected(&checked("text-spacing", &text, true), [trim, &auto]);
                }
                let words: Vec<_> = input.split_whitespace().collect();
                for cut in 1..words.len() {
                    invalid(
                        "text-spacing",
                        &format!(
                            "{} {trim} {}",
                            words[..cut].join(" "),
                            words[cut..].join(" ")
                        ),
                    );
                }
            }
        }
    }
    // Inner flag/mode interleaving remains invalid even when trim is outside.
    for trim in TRIM {
        for mode in ["insert", "replace"] {
            invalid(
                "text-spacing",
                &format!("{trim} ideograph-alpha {mode} punctuation"),
            );
            invalid(
                "text-spacing",
                &format!("ideograph-alpha {mode} punctuation {trim}"),
            );
        }
    }
}
#[test]
fn shorthand_never_accepts_longhand_auto_or_normal_as_autospace_constituents() {
    for text in [
        "",
        "normal normal",
        "auto insert",
        "insert auto",
        "trim-start auto",
        "auto trim-start",
        "none insert",
        "insert none",
        "none normal",
        "normal none",
        "auto auto",
        "none none",
        "normal auto",
        "auto normal",
        "no-autospace insert",
        "insert no-autospace",
        "insert replace",
        "ideograph-alpha ideograph-alpha",
        "trim-start trim-both",
        "trim-start normal",
        "normal trim-start",
        "space-all space-first",
        "trim-start,insert",
        "\"normal\"",
        "1px",
    ] {
        invalid("text-spacing", text);
    }
}
#[test]
fn hanging_all_first_last_and_exclusive_end_role_permutations_emit_grammar_order() {
    accepted("hanging-punctuation", "none", "none");
    for first in [false, true] {
        for end in [None, Some("force-end"), Some("allow-end")] {
            for last in [false, true] {
                let mut words = Vec::new();
                if first {
                    words.push("first");
                }
                if let Some(end) = end {
                    words.push(end);
                }
                if last {
                    words.push("last");
                }
                if words.is_empty() {
                    continue;
                }
                let expected = words.join(" ");
                for text in permutations(&words) {
                    accepted("hanging-punctuation", &text, &expected);
                }
            }
        }
    }
    for text in [
        "",
        "normal",
        "auto",
        "first first",
        "last last",
        "force-end force-end",
        "allow-end allow-end",
        "force-end allow-end",
        "allow-end force-end",
        "first force-end last allow-end",
        "none first",
        "last none",
        "none none",
        "first,last",
        "1px",
        "[first]",
        "\"first\"",
    ] {
        invalid("hanging-punctuation", text);
    }
}
#[test]
fn decoded_keywords_comments_and_case_preserve_original_authorship_and_canonical_group_roles() {
    for (name, input, expected) in [
        (
            "text-autospace",
            r"REPLACE/**/\70 unctuation IDEOGRAPH-ALPHA",
            "ideograph-alpha punctuation replace",
        ),
        (
            "text-spacing",
            r"\72 eplace PUNCTUATION IDEOGRAPH-ALPHA/**/TRIM-START",
            "trim-start ideograph-alpha punctuation replace",
        ),
        ("text-spacing-trim", r"\74 rim-both", "trim-both"),
        (
            "hanging-punctuation",
            r"LAST/**/\61 llow-end FIRST",
            "first allow-end last",
        ),
    ] {
        accepted(name, input, expected);
    }
}
#[test]
fn spacing_projection_does_not_reset_white_space_letter_word_or_justify_siblings() {
    let report = parse_style_attribute(
        "white-space:pre;word-spacing:2px;letter-spacing:3px;text-justify:distribute;text-spacing:none!important;color:red",
    );
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 6);
    let source = &report.syntax()[4];
    projected(source, ["space-all", "no-autospace"]);
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "white-space: pre;"
    );
    assert_eq!(
        report.syntax()[1].to_specified_css().unwrap(),
        "word-spacing: 2px;"
    );
    assert_eq!(
        report.syntax()[2].to_specified_css().unwrap(),
        "letter-spacing: 3px;"
    );
    assert_eq!(
        report.syntax()[3].to_specified_css().unwrap(),
        "text-justify: distribute;"
    );
    assert_eq!(
        report.syntax()[5].to_specified_css().unwrap(),
        "color: red;"
    );
}
#[test]
fn sheet_siblings_share_exact_work_and_non_ascii_output_bytes_without_charging_hidden_defaults() {
    // Sheet1 + two class rule structural groups4 + declarations2 each + leaves4/1.
    sheet_limits(
        ".é{text-spacing:replace punctuation ideograph-alpha trim-both!important}.b{line-padding:-2px}",
        ".é { text-spacing: trim-both ideograph-alpha punctuation replace !important; }\n.b { line-padding: -2px; }",
        18,
    );
    // A shorthand none leaf stays one provider visit despite its two projections.
    sheet_limits(
        ".é{text-spacing:none!important}.b{text-autospace:replace}",
        ".é { text-spacing: none !important; }\n.b { text-autospace: replace; }",
        15,
    );
}

#[test]
fn signed_length_provider_keeps_exact_source_and_bare_root_admission_with_atomic_limits() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for (input, expected) in [
        ("0", "0"),
        ("-2PX", "-2px"),
        ("-0.0000005px", "-0.000001px"),
        ("1.25em", "1.25em"),
    ] {
        let text = format!("/*😀*/{input}");
        let components = parse_component_values(&text).unwrap();
        let token = components
            .items()
            .iter()
            .find(|v| {
                matches!(
                    v.view(),
                    CssComponentValueRef::Token(
                        CssValueTokenRef::Number(_) | CssValueTokenRef::Dimension { .. }
                    )
                )
            })
            .unwrap();
        let numeric = CssSpecifiedLength::try_from_component(token.clone()).unwrap();
        assert_eq!(numeric.origin(), token.origin());
        assert_eq!(numeric.literal_component(), Some(token));
        let before = numeric.clone();
        let exact = Limits::new(1, 1, expected.len());
        assert_eq!(
            numeric.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (Limits::new(0, 1, expected.len()), Kind::InputNodeLimit),
            (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(1, 1, expected.len() - 1), Kind::ByteLimit),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    numeric
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(numeric, before);
            }
        }
        assert_eq!(
            numeric.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
    }
    for text in ["1", "-1", "1%"] {
        let calculation =
            CssLengthCalculation::try_from_components(parse_component_values(text).unwrap());
        if let Ok(calculation) = calculation {
            assert!(CssSpecifiedLength::try_from_calculation(calculation).is_err());
        }
    }
}
