#![forbid(unsafe_code)]
//! Text4 2026-08-14: mode/style 1530-1538/1744-1752, text-wrap 1910-1932;
//! collapse/trim 803-811/912-920, white-space 653-702, word-break 1986-1994.
//! The source determines membership, keyword sets, inheritance and initials.
//! Selected Surgeist policy preserves aliases/omission and emits mode/style,
//! collapse/mode/trim and before/after/inner; white-space's source order is n/a.
//! Each emitted keyword costs one input/projection node; carriers are transparent.
//! Declarations add two nodes. Normalization charges 2/3/1 terminal members.
//!
//! Missing constituent recognition is tested at runtime, not with future symbols.
//! Finite keyword payloads are compared with independently authored longhands
//! via the existing owned-value equality boundary. New typed constructors,
//! borrowed variants, primitive methods and private output-suppression checks
//! receive functional tests with implementation; no helper is invented here.

#[path = "common/authored_property.rs"]
mod authored_property;
use authored_property::{ParserFront, assert_source, checked, invalid, parsed};

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const WRAP: [&str; 2] = ["text-wrap-mode", "text-wrap-style"];
const SPACE: [&str; 3] = ["white-space-collapse", "text-wrap-mode", "white-space-trim"];
const LONGHANDS: [(&str, &str, bool); 5] = [
    ("text-wrap-mode", "wrap", true),
    ("text-wrap-style", "auto", true),
    ("white-space-collapse", "collapse", true),
    ("white-space-trim", "none", false),
    ("word-break", "normal", true),
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn property(name: &str) -> P {
    let property = P::from_name(name).unwrap_or_else(|| panic!("selected property {name}"));
    assert_eq!(property.canonical_name(), name);
    assert_eq!(
        CssPropertyGrammar::from_name(name),
        Some(property.grammar())
    );
    assert_eq!(P::from_name(&name.to_ascii_uppercase()), Some(property));
    property
}
const FRONTS: [ParserFront; 3] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
];
fn fronts(property: P, value: &str) -> [CssDeclaration; 3] {
    FRONTS.map(|front| front.parse(property, value))
}
fn accepted(property: P, value: &str, expected: &str) {
    for front in FRONTS {
        let source = front.valid(property, value, expected);
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
    }
    checked(property, expected, false);
    checked(property, expected, true);
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("terminal contributions")
    };
    values
}
fn names(values: &CssLonghandContributions) -> Vec<&'static str> {
    values
        .items()
        .iter()
        .map(|item| item.property().canonical_name())
        .collect()
}
fn assert_ordinary(values: &CssLonghandContributions, source: &CssDeclaration) {
    for item in values.items() {
        assert_source(item, source);
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            item.property()
        );
    }
}
fn assert_payloads(values: &CssLonghandContributions, expected: &[(&str, &str)]) {
    assert_eq!(
        names(values),
        expected.iter().map(|(name, _)| *name).collect::<Vec<_>>()
    );
    for (item, (name, value)) in values.items().iter().zip(expected) {
        let explicit = completed(&checked(property(name), value, true));
        let [expected_item] = explicit.items() else {
            panic!("one explicit terminal")
        };
        assert_eq!(
            item.ordinary_value().expect("ordinary projection"),
            expected_item
                .ordinary_value()
                .expect("ordinary explicit longhand"),
            "{name}:{value}"
        );
    }
}
fn permutations(values: &[&str]) -> Vec<String> {
    if values.is_empty() {
        return vec![String::new()];
    }
    let mut result = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let rest: Vec<_> = values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| (i != index).then_some(*v))
            .collect();
        for suffix in permutations(&rest) {
            result.push(if suffix.is_empty() {
                (*value).into()
            } else {
                format!("{value} {suffix}")
            });
        }
    }
    result
}

#[test]
fn text_wrap_accepts_each_mode_style_pair_in_both_orders_and_omitted_roles() {
    for mode in ["wrap", "nowrap"] {
        accepted(P::TextWrap, mode, mode);
        for style in [
            "auto",
            "balance",
            "stable",
            "pretty",
            "avoid-short-last-line",
        ] {
            let expected = format!("{mode} {style}");
            accepted(P::TextWrap, &expected, &expected);
            accepted(P::TextWrap, &format!("{style} {mode}"), &expected);
        }
    }
    for style in [
        "auto",
        "balance",
        "stable",
        "pretty",
        "avoid-short-last-line",
    ] {
        accepted(P::TextWrap, style, style);
    }
    accepted(P::TextWrap, r"n\6f wrap b\61 lance", "nowrap balance");
}
#[test]
fn white_space_accepts_role_permutations_without_inserting_serialized_defaults() {
    for css in permutations(&["preserve", "nowrap", "discard-after discard-before"]) {
        accepted(
            P::WhiteSpace,
            &css,
            "preserve nowrap discard-before discard-after",
        );
    }
    for collapse in [
        "collapse",
        "discard",
        "preserve",
        "preserve-breaks",
        "preserve-spaces",
        "break-spaces",
    ] {
        accepted(P::WhiteSpace, collapse, collapse);
    }
    for mode in ["wrap", "nowrap"] {
        accepted(P::WhiteSpace, mode, mode);
    }
    for trim in [
        "none",
        "discard-before",
        "discard-after",
        "discard-inner",
        "discard-before discard-after",
        "discard-before discard-inner",
        "discard-after discard-inner",
        "discard-before discard-after discard-inner",
    ] {
        accepted(P::WhiteSpace, trim, trim);
    }
    for css in permutations(&["collapse", "nowrap", "none"]) {
        accepted(P::WhiteSpace, &css, "collapse nowrap none");
    }
    accepted(
        P::WhiteSpace,
        "PRESERVE NOWRAP DISCARD-AFTER discard-before",
        "preserve nowrap discard-before discard-after",
    );
}
#[test]
fn all_six_word_break_keywords_are_admitted_and_emitted_literally() {
    for keyword in [
        "normal",
        "break-all",
        "keep-all",
        "manual",
        "auto-phrase",
        "break-word",
    ] {
        accepted(P::WordBreak, keyword, keyword);
        accepted(P::WordBreak, &keyword.to_ascii_uppercase(), keyword);
    }
    accepted(P::WordBreak, r"m\61 nual", "manual");
}
#[test]
fn registered_text_negative_grammars_reject_duplicates_conflicts_and_foreign_roles() {
    for css in [
        "wrap nowrap",
        "balance pretty",
        "auto auto",
        "nowrap nowrap",
        "balance balance",
        "wrap balance auto",
        "avoid-orphans",
        "none",
        "inherit balance",
        "balance, pretty",
    ] {
        invalid(P::TextWrap, css);
    }
    for css in [
        "pre nowrap",
        "normal preserve",
        "pre-line discard-before",
        "collapse preserve",
        "wrap nowrap",
        "none discard-inner",
        "discard-before discard-before",
        "collapse collapse",
        "balance",
        "stable",
        "auto",
        "inherit collapse",
        "preserve, nowrap",
    ] {
        invalid(P::WhiteSpace, css);
    }
    for css in [
        "manual normal",
        "auto-phrase auto-phrase",
        "normal, break-all",
        "anywhere",
        "none",
        "inherit manual",
    ] {
        invalid(P::WordBreak, css);
    }
}
#[test]
fn white_space_trim_group_cannot_interleave_mode_or_collapse_constituents() {
    // Values4 quoted-property grouping is nonassociative: a trim set is one
    // contiguous constituent even when its internally unordered flags differ.
    for css in [
        "discard-before nowrap discard-after",
        "discard-inner preserve discard-after",
        "discard-before collapse discard-inner",
        "discard-after wrap discard-inner",
    ] {
        invalid(P::WhiteSpace, css);
    }
}

#[test]
fn existing_singleton_wrap_white_space_and_word_break_provider_controls_remain_supported() {
    for css in ["wrap", "nowrap", "balance", "pretty", "stable"] {
        accepted(P::TextWrap, css, css);
    }
    for css in [
        "normal",
        "nowrap",
        "pre",
        "pre-wrap",
        "pre-line",
        "break-spaces",
    ] {
        accepted(P::WhiteSpace, css, css);
    }
    for css in ["normal", "break-all", "keep-all", "break-word"] {
        accepted(P::WordBreak, css, css);
    }
}

#[test]
fn text_wrap_mode_runtime_recognition_drives_both_real_checked_fronts() {
    let p = property("text-wrap-mode");
    for css in ["wrap", "nowrap"] {
        accepted(p, css, css);
    }
    for css in ["auto", "balance", "wrap nowrap", "none"] {
        invalid(p, css);
    }
}
#[test]
fn text_wrap_style_runtime_recognition_drives_all_selected_keywords() {
    let p = property("text-wrap-style");
    for css in [
        "auto",
        "balance",
        "stable",
        "pretty",
        "avoid-short-last-line",
    ] {
        accepted(p, css, css);
    }
    for css in ["nowrap", "avoid-orphans", "pretty stable", "none"] {
        invalid(p, css);
    }
}
#[test]
fn white_space_collapse_runtime_recognition_includes_discard_without_execution() {
    let p = property("white-space-collapse");
    for css in [
        "collapse",
        "discard",
        "preserve",
        "preserve-breaks",
        "preserve-spaces",
        "break-spaces",
    ] {
        accepted(p, css, css);
    }
    for css in ["none", "pre", "wrap", "preserve collapse"] {
        invalid(p, css);
    }
}
#[test]
fn white_space_trim_runtime_recognition_accepts_sets_and_canonicalizes_flag_order() {
    let p = property("white-space-trim");
    accepted(p, "none", "none");
    for flags in [
        vec!["discard-before"],
        vec!["discard-after"],
        vec!["discard-inner"],
        vec!["discard-before", "discard-after"],
        vec!["discard-before", "discard-inner"],
        vec!["discard-after", "discard-inner"],
        vec!["discard-before", "discard-after", "discard-inner"],
    ] {
        let expected = flags.join(" ");
        for css in permutations(&flags) {
            accepted(p, &css, &expected);
        }
    }
    for css in [
        "none discard-before",
        "discard-before none",
        "discard-after discard-after",
        "collapse",
        "auto",
    ] {
        invalid(p, css);
    }
}

fn longhand_metadata(name: &str, initial_css: &str, inherited: bool) {
    let p = property(name);
    let metadata = p.metadata().expect("intrinsic metadata");
    assert_eq!(metadata.grammar(), p.grammar());
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert_eq!(longhand.property().known_property(), p);
    assert_eq!(longhand.inherited_by_default(), inherited);
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), p);
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary intrinsic initial")
    };
    let explicit = completed(&checked(p, initial_css, true));
    assert_eq!(Some(initial), explicit.items()[0].ordinary_value());
}
macro_rules! metadata_test {
    ($test:ident, $name:literal, $initial:literal, $inherited:literal) => {
        #[test]
        fn $test() {
            longhand_metadata($name, $initial, $inherited);
        }
    };
}
metadata_test!(
    mode_metadata_is_inherited_with_wrap_initial,
    "text-wrap-mode",
    "wrap",
    true
);
metadata_test!(
    style_metadata_is_inherited_with_auto_initial,
    "text-wrap-style",
    "auto",
    true
);
metadata_test!(
    collapse_metadata_is_inherited_with_collapse_initial,
    "white-space-collapse",
    "collapse",
    true
);
metadata_test!(
    trim_metadata_is_noninherited_with_none_initial,
    "white-space-trim",
    "none",
    false
);
metadata_test!(
    word_break_metadata_is_inherited_with_normal_initial,
    "word-break",
    "normal",
    true
);

fn shorthand_metadata(p: P, expected: &[&str]) {
    let metadata = p.metadata().expect("shorthand metadata");
    assert_eq!(metadata.grammar(), p.grammar());
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("shorthand")
    };
    let names = |members: &[CssLonghandProperty]| {
        members
            .iter()
            .map(|p| p.known_property().canonical_name())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(shorthand.members()), expected);
    assert_eq!(names(shorthand.settable_members()), expected);
    assert!(shorthand.reset_only_members().is_empty());
    assert!(!shorthand.is_legacy());
}
#[test]
fn text_wrap_metadata_has_exactly_mode_style_and_no_reset_only_members() {
    shorthand_metadata(P::TextWrap, &WRAP);
}
#[test]
fn white_space_metadata_has_exactly_collapse_mode_trim_and_no_style_spill() {
    shorthand_metadata(P::WhiteSpace, &SPACE);
}

#[test]
fn text_wrap_projects_sparse_defaults_and_explicit_roles_with_original_occurrences() {
    for (css, mode, style) in [
        ("wrap", "wrap", "auto"),
        ("nowrap", "nowrap", "auto"),
        ("balance", "wrap", "balance"),
        ("auto", "wrap", "auto"),
        ("stable nowrap", "nowrap", "stable"),
        ("avoid-short-last-line", "wrap", "avoid-short-last-line"),
    ] {
        for source in fronts(P::TextWrap, css) {
            let values = completed(&source);
            assert_payloads(
                &values,
                &[("text-wrap-mode", mode), ("text-wrap-style", style)],
            );
            assert_ordinary(&values, &source);
            assert!(
                values
                    .items()
                    .iter()
                    .all(|item| item.replacement_components().is_none())
            );
        }
    }
}
#[test]
fn four_white_space_special_forms_project_independent_source_defined_values() {
    for (css, collapse, mode) in [
        ("normal", "collapse", "wrap"),
        ("pre", "preserve", "nowrap"),
        ("pre-wrap", "preserve", "wrap"),
        ("pre-line", "preserve-breaks", "wrap"),
    ] {
        for source in fronts(P::WhiteSpace, css) {
            let values = completed(&source);
            assert_payloads(
                &values,
                &[
                    ("white-space-collapse", collapse),
                    ("text-wrap-mode", mode),
                    ("white-space-trim", "none"),
                ],
            );
            assert_ordinary(&values, &source);
            assert!(
                values
                    .items()
                    .iter()
                    .all(|item| item.replacement_components().is_none())
            );
        }
    }
}
#[test]
fn white_space_component_omissions_reset_only_its_three_members() {
    for (css, collapse, mode, trim) in [
        ("nowrap", "collapse", "nowrap", "none"),
        ("break-spaces", "break-spaces", "wrap", "none"),
        ("discard-inner", "collapse", "wrap", "discard-inner"),
        (
            "preserve-spaces discard-after",
            "preserve-spaces",
            "wrap",
            "discard-after",
        ),
        (
            "discard nowrap discard-before",
            "discard",
            "nowrap",
            "discard-before",
        ),
    ] {
        for source in fronts(P::WhiteSpace, css) {
            let values = completed(&source);
            assert_payloads(
                &values,
                &[
                    ("white-space-collapse", collapse),
                    ("text-wrap-mode", mode),
                    ("white-space-trim", trim),
                ],
            );
            assert_ordinary(&values, &source);
        }
    }
}
#[test]
fn all_five_longhands_expand_once_without_replacing_components_or_occurrences() {
    for (name, css, _) in LONGHANDS {
        let p = property(name);
        let sources = fronts(p, css);
        assert!(!sources[0].same_occurrence(&sources[1]));
        for source in &sources {
            let values = completed(source);
            assert_eq!(names(&values), [name]);
            assert_ordinary(&values, source);
            assert!(values.items()[0].replacement_components().is_none());
        }
        assert_eq!(
            completed(&sources[0]).items()[0].ordinary_value(),
            completed(&sources[1]).items()[0].ordinary_value()
        );
    }
}
#[test]
fn break_word_stays_one_word_break_terminal_without_overflow_wrap_reset() {
    let source = parsed(P::WordBreak, "break-word");
    let values = completed(&source);
    assert_payloads(&values, &[("word-break", "break-word")]);
    assert_ordinary(&values, &source);
}

#[test]
fn five_css_wide_keywords_propagate_to_exact_shorthand_members() {
    for (p, expected) in [
        (P::TextWrap, WRAP.as_slice()),
        (P::WhiteSpace, SPACE.as_slice()),
        (P::WordBreak, ["word-break"].as_slice()),
    ] {
        for (css, keyword) in GLOBALS {
            for source in fronts(p, css) {
                let values = completed(&source);
                assert_eq!(names(&values), expected);
                for item in values.items() {
                    assert_source(item, &source);
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    assert!(item.ordinary_value().is_none());
                    assert!(item.replacement_components().is_none());
                }
            }
        }
    }
}
#[test]
fn five_css_wide_keywords_propagate_to_each_constituent_longhand() {
    for (name, _, _) in LONGHANDS {
        let p = property(name);
        for (css, keyword) in GLOBALS {
            for source in fronts(p, css) {
                let values = completed(&source);
                assert_eq!(names(&values), [name]);
                assert_eq!(
                    values.items()[0].value(),
                    CssContributionValueRef::Global(keyword)
                );
                assert_source(&values.items()[0], &source);
            }
        }
    }
}

fn pending_contract(p: P, value: &str, expected: &[(&str, &str)], invalid_values: &[&str]) {
    for pending in ["var(--text)", "env(text)", "attr(data-text)"] {
        for source in fronts(p, pending) {
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("one unresolved occurrence")
            };
            assert!(handle.source().same_occurrence(&source));
            let replacement = parse_component_values(value).unwrap();
            let original_replacement = replacement.clone();
            for bad in invalid_values {
                let components = parse_component_values(bad).unwrap();
                let expected_error = parse_property_value_for_grammar(
                    p.grammar(),
                    components.clone(),
                    CssImportance::Normal,
                )
                .unwrap_err();
                let error = handle.reenter(components).unwrap_err();
                if *bad == "var(--again)/*" {
                    // Residual detection precedes replacement grammar, even
                    // when direct checked admission rejects original closure.
                    assert_eq!(error.kind(), &CssExpansionErrorKind::ResidualSubstitution);
                } else {
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("original grammar failure")
                    };
                    assert_eq!(actual, &expected_error);
                }
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("valid retry")
                };
                assert_payloads(&values, expected);
            }
            for residual in ["var(--again)", "env(other)", "attr(data-other)"] {
                assert_eq!(
                    handle
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("retry after residual")
                };
                assert_payloads(&values, expected);
            }
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("complete replacement")
                };
                assert_payloads(&values, expected);
                assert_ordinary(&values, &source);
                for item in values.items() {
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    for (actual, original) in item
                        .replacement_components()
                        .unwrap()
                        .items()
                        .iter()
                        .zip(replacement.items())
                    {
                        assert_eq!(actual.origin(), original.origin());
                        if let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) =
                            (actual.origin(), original.origin())
                        {
                            assert!(a.source().same_snapshot(b.source()));
                        }
                    }
                }
            }
            for (css, keyword) in GLOBALS {
                let replacement = parse_component_values(css).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("global replacement")
                };
                assert_eq!(
                    names(&values),
                    expected.iter().map(|(name, _)| *name).collect::<Vec<_>>()
                );
                for item in values.items() {
                    assert_source(item, &source);
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
            assert_eq!(replacement, original_replacement);
            assert_eq!(source, before);
        }
    }
}
#[test]
fn text_wrap_pending_reentry_is_atomic_and_reusable_after_grammar_recovery_and_residual_failures() {
    pending_contract(
        P::TextWrap,
        "balance nowrap",
        &[("text-wrap-mode", "nowrap"), ("text-wrap-style", "balance")],
        &[
            "wrap nowrap",
            "auto balance",
            "balance/*",
            "initial/*",
            "var(--again)/*",
            "balance!important",
            "balance;color:red",
        ],
    );
}
#[test]
fn white_space_pending_reentry_retains_original_grammar_and_three_member_defaults() {
    pending_contract(
        P::WhiteSpace,
        "pre",
        &[
            ("white-space-collapse", "preserve"),
            ("text-wrap-mode", "nowrap"),
            ("white-space-trim", "none"),
        ],
        &[
            "pre nowrap",
            "none discard-before",
            "pre/*",
            "initial/*",
            "var(--again)/*",
            "pre!important",
            "pre;color:red",
        ],
    );
}
#[test]
fn word_break_pending_reentry_keeps_break_word_literal_and_never_resets_overflow_wrap() {
    pending_contract(
        P::WordBreak,
        "break-word",
        &[("word-break", "break-word")],
        &[
            "break-all normal",
            "normal/*",
            "initial/*",
            "var(--again)/*",
            "normal!important",
            "normal;color:red",
        ],
    );
}
#[test]
fn constituent_pending_reentry_uses_each_registered_original_grammar() {
    for (name, value, _) in LONGHANDS {
        let p = property(name);
        let recovered = format!("{value}/*");
        pending_contract(
            p,
            value,
            &[(name, value)],
            &[
                "not-a-selected-value",
                &recovered,
                "initial/*",
                "var(--again)/*",
            ],
        );
    }
}

fn strict_closure(p: P, ordinary: &str) {
    let mut failures = Vec::new();
    for text in [
        format!("{ordinary}/*"),
        format!("{ordinary}/*unfinished"),
        "initial/*".into(),
        "var(--text)/*".into(),
    ] {
        let components = parse_component_values(&text).unwrap();
        let before = components.clone();
        let serialized = components.serialize().unwrap();
        let origin = (0..serialized.as_css().len())
            .find_map(|offset| match serialized.origin_at(offset) {
                Some(CssSerializedOrigin::Token(
                    origin @ CssValueOrigin::ImplicitClosure { .. },
                )) => Some(origin.clone()),
                _ => None,
            })
            .expect("original implicit closure mapping");
        let CssValueOrigin::ImplicitClosure { at, .. } = &origin else {
            unreachable!()
        };
        assert_eq!(at.source().as_str(), text);
        assert_eq!(at.span().start().byte_offset().value(), text.len());
        assert_eq!(at.span().start(), at.span().end());
        for (front, result) in [
            (
                "property",
                parse_property_value(
                    CssPropertyNameRef::Known(p),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
            (
                "grammar",
                parse_property_value_for_grammar(
                    p.grammar(),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
        ] {
            match result {
                Err(error)
                    if matches!(
                        error.kind(),
                        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                    ) && error.origin() == &CssSerializedOrigin::End(Some(origin.clone())) => {}
                actual => failures.push(format!(
                    "{front}/{text}: expected original implicit EOF, actual {actual:?}"
                )),
            }
        }
        assert_eq!(components, before);
    }
    assert!(
        failures.is_empty(),
        "{}:\n{}",
        p.canonical_name(),
        failures.join("\n")
    );
}
#[test]
fn text_wrap_checked_fronts_reject_original_implicit_comment_closure() {
    strict_closure(P::TextWrap, "balance");
}
#[test]
fn white_space_checked_fronts_reject_original_implicit_comment_closure() {
    strict_closure(P::WhiteSpace, "pre");
}
#[test]
fn word_break_checked_fronts_reject_original_implicit_comment_closure() {
    strict_closure(P::WordBreak, "normal");
}
#[test]
fn constituent_checked_fronts_reject_original_implicit_comment_closure() {
    for (name, value, _) in LONGHANDS {
        strict_closure(property(name), value);
    }
}
#[test]
fn complete_text_comments_globals_and_pending_values_remain_checked_controls() {
    for (p, value) in [
        (P::TextWrap, "balance"),
        (P::WhiteSpace, "pre"),
        (P::WordBreak, "normal"),
    ] {
        for css in [
            format!("{value}/**/"),
            format!("{value}/*unfinished*/"),
            "initial/**/".into(),
            "var(--text)/**/".into(),
        ] {
            checked(p, &css, false);
            checked(p, &css, true);
        }
    }
}

#[test]
fn normalization_preserves_duplicate_order_contexts_and_two_three_one_terminal_work() {
    let report = parse_sheet(
        "@media screen{.a{text-wrap:balance!important;white-space:pre;word-break:break-word;text-wrap:var(--t);white-space:unset;color:red}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    // 2 + 3 + 1 ordinary, one pending occurrence, 3 global, 1 Color = 11.
    let exact = CssNormalizationLimits::try_new(1, 2, 6, 11).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    for (index, (item, expected)) in declarations
        .iter()
        .zip([
            P::TextWrap,
            P::WhiteSpace,
            P::WordBreak,
            P::TextWrap,
            P::WhiteSpace,
            P::Color,
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), expected);
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        if index == 3 {
            let CssExpansion::Pending(handle) = item.expansion() else {
                panic!("one pending occurrence")
            };
            assert!(handle.source().same_occurrence(item.source()));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("ordered terminals")
            };
            let expected: &[&str] = match index {
                0 => &WRAP,
                1 | 4 => &SPACE,
                2 => &["word-break"],
                5 => &["color"],
                _ => unreachable!(),
            };
            assert_eq!(names(values), expected);
            for terminal in values.items() {
                assert_source(terminal, item.source());
            }
        }
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(1, 2, 6, 10).unwrap(),
            CssNormalizationResource::Contributions,
            10,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 5, 11).unwrap(),
            CssNormalizationResource::Declarations,
            5,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit }
            );
            assert_eq!(error.declaration_order(), Some(5));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[5].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn browser_recovery_normalization_retains_diagnostics_and_source_without_forging_checked_input() {
    for (p, value, expected) in [
        (P::TextWrap, "balance", WRAP.as_slice()),
        (P::WhiteSpace, "pre", SPACE.as_slice()),
        (P::WordBreak, "normal", ["word-break"].as_slice()),
    ] {
        let css = format!(".a{{{}:{value}/*", p.canonical_name());
        let report = parse_sheet(&css);
        assert!(!report.is_clean());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, expected.len()).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|v| match v {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        let [item] = declarations.as_slice() else {
            panic!("one recovered occurrence")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("recovered ordinary terminals")
        };
        assert_eq!(names(values), expected);
        assert_ordinary(values, item.source());
        assert!(
            parse_property_value_for_grammar(
                p.grammar(),
                item.source().value_components().clone(),
                CssImportance::Normal
            )
            .is_err()
        );
        let error = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, expected.len() - 1).unwrap(),
        )
        .unwrap_err();
        assert!(error.declaration().unwrap().same_occurrence(item.source()));
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}

fn typed_output(
    source: &CssDeclaration,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TextWrap(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::WhiteSpace(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::WordBreak(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        _ => panic!("existing typed provider"),
    }
}
fn provider_limits(p: P, css: &str, expected: &str, work: usize) {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let declaration_css = format!("{}: {expected} !important;", p.canonical_name());
    for source in fronts(p, css) {
        let before = source.clone();
        let exact = Limits::new(work, work, expected.len());
        assert_eq!(typed_output(&source, exact).unwrap(), expected);
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(
                    work + 2,
                    work + 2,
                    declaration_css.len()
                ))
                .unwrap(),
            declaration_css
        );
        for (value_limits, declaration_limits, kind) in [
            (
                Limits::new(work - 1, work, expected.len()),
                Limits::new(work + 1, work + 2, declaration_css.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(work, work - 1, expected.len()),
                Limits::new(work + 2, work + 1, declaration_css.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(work, work, expected.len() - 1),
                Limits::new(work + 2, work + 2, declaration_css.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    typed_output(&source, value_limits).unwrap_err().kind(),
                    kind
                );
                assert_eq!(
                    source
                        .to_specified_css_with_limits(declaration_limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(typed_output(&source, exact).unwrap(), expected);
        assert_eq!(source.to_specified_css().unwrap(), declaration_css);
    }
}
#[test]
fn text_wrap_two_keywords_have_exact_two_node_provider_work_and_atomic_limits() {
    provider_limits(P::TextWrap, "balance nowrap", "nowrap balance", 2);
}
#[test]
fn white_space_five_keywords_share_exact_work_and_canonical_byte_limits() {
    provider_limits(
        P::WhiteSpace,
        "discard-inner discard-after discard-before nowrap preserve",
        "preserve nowrap discard-before discard-after discard-inner",
        5,
    );
}
#[test]
fn represented_single_keyword_provider_costs_remain_one_node_controls() {
    provider_limits(P::TextWrap, "balance", "balance", 1);
    for css in ["normal", "pre", "nowrap", "break-spaces"] {
        provider_limits(P::WhiteSpace, css, css, 1);
    }
    provider_limits(P::WordBreak, "break-word", "break-word", 1);
}
#[test]
fn new_word_break_keywords_keep_single_keyword_work_and_exact_bytes() {
    provider_limits(P::WordBreak, "manual", "manual", 1);
    provider_limits(P::WordBreak, "auto-phrase", "auto-phrase", 1);
}
#[test]
fn sibling_text_rules_share_final_sheet_bytes_and_atomic_reusable_output() {
    use CssSpecifiedRuleSerializationErrorKind as RuleKind;
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let report =
        parse_sheet(".a{text-wrap:balance nowrap!important}.b{white-space:discard-after preserve}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected =
        ".a { text-wrap: nowrap balance !important; }\n.b { white-space: preserve discard-after; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(error.kind(), RuleKind::Resource(Kind::ByteLimit));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn independent_color_and_overflow_wrap_controls_keep_metadata_and_lifecycle() {
    let CssPropertyKindRef::Longhand(color) = P::Color.metadata().unwrap().kind() else {
        panic!("color longhand")
    };
    assert!(color.inherited_by_default());
    let initial = color.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("symbolic color initial")
    };
    let CssLonghandValueRef::Color(color) = initial.view() else {
        panic!("existing color borrowed view")
    };
    assert_eq!(color.system(), Some(CssSystemColor::CanvasText));
    longhand_metadata("overflow-wrap", "normal", true);
    for (p, css) in [(P::Color, "currentcolor"), (P::OverflowWrap, "anywhere")] {
        for source in fronts(p, css) {
            let values = completed(&source);
            assert_eq!(names(&values), [p.canonical_name()]);
            assert_ordinary(&values, &source);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{}: {css} !important;", p.canonical_name())
            );
        }
    }
}
