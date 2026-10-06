#![forbid(unsafe_code)]
//! Existing-callable Content 3 WD 2025-12-04 authored contract.
//! Selected string-set table/local prose remains narrow despite #named-strings
//! contradiction; the source record retains #680 and the bounded disposition.
//! Published source basis: cfffe14a4e5371a1b09e9a0a37b56dd74cf712e0.
//! No missing-symbol/constructor RED or execution evidence is claimed.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};
const NAMES: [&str; 6] = [
    "content",
    "quotes",
    "string-set",
    "bookmark-level",
    "bookmark-label",
    "bookmark-state",
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn property(name: &str) -> CssKnownProperty {
    let property = CssKnownProperty::from_name(name).unwrap_or_else(|| {
        panic!("selected authored property {name} must accept public discovery")
    });
    assert_eq!(property.canonical_name(), name);
    assert_eq!(
        CssKnownProperty::from_name(&name.to_ascii_uppercase()),
        Some(property)
    );
    assert_eq!(
        CssPropertyGrammar::from_name(name).unwrap(),
        property.grammar()
    );
    property
}

fn declaration(name: &str, input: &str, front: usize) -> CssDeclaration {
    let p = property(name);
    let source = match front {
        0 => {
            let css = format!("/*😀*/{name}:{input}!important");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
            let [source] = report.syntax().as_slice() else {
                panic!("one authored occurrence")
            };
            assert!(source.position().is_some());
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            source.clone()
        }
        1 | 2 => {
            let components = parse_component_values(input).unwrap();
            let before = components.clone();
            let source = checked(p, components.clone(), front == 2).unwrap();
            assert_eq!(components, before);
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_value().is_none());
            source
        }
        3 | 4 => {
            let report = if front == 3 {
                parse_property_value_text(
                    input,
                    CssPropertyNameRef::Known(p),
                    CssImportance::Important,
                )
            } else {
                parse_property_value_text_for_grammar(input, p.grammar(), CssImportance::Important)
            };
            assert!(
                report.is_clean(),
                "{name}:{input}: {:?}",
                report.diagnostics()
            );
            let source = report
                .syntax()
                .as_ref()
                .expect("retained direct value")
                .clone();
            assert!(source.position().is_none());
            assert_eq!(source.parsed_value().unwrap().source().as_str(), input);
            source
        }
        _ => unreachable!(),
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

fn checked(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    }
}

fn accept(name: &str, cases: &[(&str, &str)]) {
    for &(input, expected) in cases {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let before = source.clone();
            let output = format!("{name}: {expected} !important;");
            assert_eq!(
                source.to_specified_css().unwrap(),
                output,
                "{input} front {front}"
            );
            assert_eq!(source, before);
            let reparse = parse_style_attribute(&output);
            assert!(reparse.is_clean(), "{output}: {:?}", reparse.diagnostics());
            let [reparsed] = reparse.syntax().as_slice() else {
                panic!("one reentered occurrence")
            };
            assert_eq!(reparsed.known().unwrap().property(), property(name));
            assert_eq!(reparsed.importance(), CssImportance::Important);
            assert_eq!(reparsed.to_specified_css().unwrap(), output);
        }
    }
}

fn invalid(name: &str, values: &[&str]) {
    let p = property(name);
    for value in values {
        let middle = format!("{name}:{value};");
        let css = format!("/*😀*/color:red!important;{middle}color:blue");
        let report = parse_style_attribute(&css);
        let [before, after] = report.syntax().as_slice() else {
            panic!("only adjacent colors survive {css}")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(before.importance(), CssImportance::Important);
        assert_eq!(after.importance(), CssImportance::Normal);
        let [diagnostic] = report.diagnostics() else {
            panic!("one atomic diagnostic {css}")
        };
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("actual property grammar diagnostic")
        };
        assert_eq!(detail.property(), p);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = css.find(&middle).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + middle.len()
        );
        assert_eq!(
            diagnostic.span().start().column().value() as usize,
            css[..start].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let components = parse_component_values(value).unwrap();
        let original = components.clone();
        for grammar in [false, true] {
            assert!(
                checked(p, components.clone(), grammar).is_err(),
                "{name}:{value}"
            );
        }
        assert_eq!(components, original);
    }
}

#[test]
fn string_sets_keep_comma_entry_and_string_leaf_order_without_concatenating_authored_children() {
    accept(
        "string-set",
        &[
            ("NONE", "none"),
            (
                "Chapter 'A' \"B\", Author \"C\"",
                "Chapter \"A\" \"B\", Author \"C\"",
            ),
            (
                "Chapter \"A\", Chapter \"B\"",
                "Chapter \"A\", Chapter \"B\"",
            ),
            ("chapter \"\" \"\"", "chapter \"\" \"\""),
            (r"chapter\ name 'text'", r#"chapter\ name "text""#),
            (r"\31 chapter 'text'", r#"\31 chapter "text""#),
            ("none \"named\"", "none \"named\""),
            ("auto \"A\", open \"B\"", "auto \"A\", open \"B\""),
        ],
    );
}

#[test]
fn string_sets_reject_missing_leaves_empty_entries_and_reserved_decoded_names() {
    invalid(
        "string-set",
        &[
            "chapter",
            "\"text\"",
            "chapter \"a\",",
            ",chapter \"a\"",
            "chapter \"a\",,author \"b\"",
            "chapter \"a\" author \"b\"",
            "initial \"a\"",
            "inherit \"a\"",
            "unset \"a\"",
            "revert \"a\"",
            "revert-layer \"a\"",
            "default \"a\"",
            "DEFAULT \"a\"",
            r#"\64 efault "a""#,
            r#"\69 nherit "a""#,
            "chapter 1",
            "chapter url(\"#x\")",
            "none none",
        ],
    );
}

#[test]
fn string_set_selected_table_does_not_silently_adopt_conflicting_content_like_prose() {
    // The exact normative discrepancy is retained in the source basis. Valid
    // attr() is pending substitution, so it is not an ordinary rejection probe.
    invalid(
        "string-set",
        &[
            "chapter contents",
            "chapter content()",
            "chapter counter(chapter)",
            "chapter counters(chapter,\".\")",
            "chapter string(other)",
            "chapter leader(dotted)",
            "chapter target-text(\"#x\")",
            "chapter open-quote",
            "chapter \"a\" / \"spoken\"",
        ],
    );
}

#[test]
fn bookmark_level_keeps_exact_unbounded_positive_integer_literals_and_real_math() {
    accept(
        "bookmark-level",
        &[
            ("NONE", "none"),
            ("+0001", "1"),
            (
                "99999999999999999999999999999999999999999999999999",
                "99999999999999999999999999999999999999999999999999",
            ),
            ("calc(1 + 2)", "calc(3)"),
            ("calc(0)", "calc(0)"),
            ("calc(1 - 2)", "calc(-1)"),
        ],
    );
    let source = declaration("bookmark-level", "+0001", 0);
    assert_eq!(
        source.value_components().serialize().unwrap().as_css(),
        "+0001"
    );
    let before = source.clone();
    assert_eq!(
        source.to_specified_css().unwrap(),
        "bookmark-level: 1 !important;"
    );
    assert_eq!(source, before);
}

#[test]
fn bookmark_level_rejects_zero_negative_noninteger_token_spellings_and_foreign_roots() {
    invalid(
        "bookmark-level",
        &[
            "0",
            "+0",
            "-0",
            "-0000",
            "-1",
            "1.0",
            "1e2",
            "1%",
            "1px",
            "1 2",
            "none 1",
            "open",
            "calc(1px)",
            "inherit 1",
        ],
    );
}

#[test]
fn bookmark_labels_reuse_full_nonempty_content_list_without_property_only_branches() {
    accept(
        "bookmark-label",
        &[
            ("''", "\"\""),
            ("content(text)", "content()"),
            (
                "content() content(before) content(after) content(first-letter) content(marker)",
                "content() content(before) content(after) content(first-letter) content(marker)",
            ),
            ("contents contents", "contents contents"),
            (
                "open-quote close-quote no-open-quote no-close-quote",
                "open-quote close-quote no-open-quote no-close-quote",
            ),
            (
                "counter(chapter,DECIMAL) counters(chapter,'.',decimal)",
                "counter(chapter) counters(chapter, \".\")",
            ),
            ("counter(chapter,MyStyle)", "counter(chapter, MyStyle)"),
            (
                "counter(chapter,symbols(symbolic '*'))",
                "counter(chapter, symbols(\"*\"))",
            ),
            (
                "leader(dotted) leader(solid) leader(space)",
                "leader(\".\") leader(\"_\") leader(\" \")",
            ),
            (
                "string(Chapter,first) string(Chapter,start) string(Chapter,last) string(Chapter,first-except)",
                "string(Chapter) string(Chapter, start) string(Chapter, last) string(Chapter, first-except)",
            ),
            (
                "target-counter('#x',chapter,decimal)",
                "target-counter(\"#x\", chapter, decimal)",
            ),
            (
                "target-counters('#x',chapter,'.')",
                "target-counters(\"#x\", chapter, \".\")",
            ),
            (
                "target-text('#x',content) target-text('#x')",
                "target-text(\"#x\", content) target-text(\"#x\")",
            ),
            ("url('#icon')", "url(\"#icon\")"),
            ("src('#icon') 'caption'", "src(\"#icon\") \"caption\""),
            (
                "linear-gradient(red,blue) 'caption'",
                "linear-gradient(red, blue) \"caption\"",
            ),
            ("light-dark(none,none)", "light-dark(none, none)"),
        ],
    );
}

#[test]
fn bookmark_labels_reject_content_keywords_slash_alternatives_and_malformed_functions() {
    invalid(
        "bookmark-label",
        &[
            "normal",
            "none",
            "none \"caption\"",
            "\"caption\" / \"spoken\"",
            ",",
            "1",
            "content(chapter)",
            "string(chapter,middle)",
            "counter(none)",
            "counter(default)",
            "target-counter(\"#x\")",
            "target-text(\"#x\",marker)",
            "counter(chapter,symbols(numeric \"0\"))",
            "leader()",
            "leader(dashed)",
            "url(\"#x\"),url(\"#y\")",
            "inherit contents",
        ],
    );
}

#[test]
fn bookmark_state_is_an_exclusive_case_insensitive_keyword() {
    accept(
        "bookmark-state",
        &[("OPEN", "open"), (r"c\6c osed", "closed")],
    );
    invalid(
        "bookmark-state",
        &[
            "none",
            "auto",
            "0",
            "open closed",
            "closed \"x\"",
            "open,closed",
            "inherit open",
        ],
    );
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary longhand contribution")
    };
    items
}

fn member(
    items: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    let [item] = items.items() else {
        panic!("one terminal; Content family has no shorthand")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.replacement_components(), replacement);
}

fn metadata(name: &str, inherited: bool) {
    let p = property(name);
    let meta = p.metadata().expect("Content intrinsic metadata");
    assert_eq!(meta.grammar(), p.grammar());
    let CssPropertyKindRef::Longhand(longhand) = meta.kind() else {
        panic!("one intrinsic longhand")
    };
    assert_eq!(longhand.property().known_property(), p);
    assert_eq!(longhand.inherited_by_default(), inherited);
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("fixed symbolic initial, no user-agent requirement")
    };
    assert_eq!(initial.property().known_property(), p);
    // Independent none/content(text)/open payload assertions are functional
    // new-variant tests with implementation, not inferred by comparing production.
}

fn ordinary(name: &str, value: &str) {
    for front in 0..5 {
        let source = declaration(name, value, front);
        let before = source.clone();
        let items = completed(&source);
        member(&items, &source, None);
        assert_eq!(
            items.items()[0]
                .ordinary_value()
                .unwrap()
                .property()
                .known_property(),
            property(name)
        );
        assert_eq!(source, before);
    }
}

fn globals(name: &str) {
    for (input, keyword) in GLOBALS {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let items = completed(&source);
            member(&items, &source, None);
            assert_eq!(
                items.items()[0].value(),
                CssContributionValueRef::Global(keyword)
            );
            assert!(items.items()[0].ordinary_value().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {input} !important;")
            );
        }
    }
}

fn pending(name: &str, valid: &str, invalid: &str) {
    let p = property(name);
    for authored in [
        "bogus var(--generated)",
        "bogus env(generated)",
        "bogus attr(data-generated)",
    ] {
        for front in 0..5 {
            let source = declaration(name, authored, front);
            let original = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole pending source")
            };
            assert!(handle.source().same_occurrence(&source));
            for _ in 0..2 {
                for bad in [invalid, "inherit bogus", "bogus"] {
                    let components = parse_component_values(bad).unwrap();
                    let direct = checked(p, components.clone(), false).unwrap_err();
                    let error = handle.reenter(components).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("typed strict replacement error")
                    };
                    assert_eq!(actual.kind(), direct.kind());
                    assert_eq!(actual.origin(), direct.origin());
                }
                for residual in [
                    "bogus var(--again)",
                    "[f(env(generated))]",
                    "f(attr(data-generated))",
                    r"f(v\61 r(--again))",
                ] {
                    assert_eq!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        &CssExpansionErrorKind::ResidualSubstitution
                    );
                }
                let replacement = parse_component_values(&format!("/*😀*/{valid}")).unwrap();
                let before = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one resolved terminal")
                };
                member(&values, &source, Some(&replacement));
                assert!(values.items()[0].ordinary_value().is_some());
                assert_eq!(replacement, before);
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("one global terminal")
                    };
                    member(&values, &source, Some(&replacement));
                    assert_eq!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(keyword)
                    );
                }
            }
            assert_eq!(source, original);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {authored} !important;")
            );
        }
    }
}

macro_rules! lifecycle {
    ($module:ident, $name:literal, $good:literal, $bad:literal, $inherited:literal) => {
        mod $module {
            use super::*;
            #[test]
            fn selected_metadata_has_fixed_initial_and_correct_inheritance() {
                metadata($name, $inherited);
            }
            #[test]
            fn ordinary_expansion_keeps_occurrence_and_grammar() {
                ordinary($name, $good);
            }
            #[test]
            fn every_css_wide_keyword_remains_symbolic() {
                globals($name);
            }
            #[test]
            fn failed_and_residual_reentry_is_atomic_reusable_and_retains_replacement() {
                pending($name, $good, $bad);
            }
        }
    };
}
lifecycle!(
    content,
    "content",
    "counter(chapter)",
    "normal contents",
    false
);
lifecycle!(quotes, "quotes", "\"«\" \"»\"", "\"one\"", true);
lifecycle!(
    strings,
    "string-set",
    "Chapter \"A\" \"B\", Author \"C\"",
    "Chapter counter(chapter)",
    false
);
lifecycle!(level, "bookmark-level", "+0001", "0", false);
lifecycle!(
    label,
    "bookmark-label",
    "content(text) counter(chapter)",
    "normal",
    false
);
lifecycle!(state, "bookmark-state", "closed", "none", false);
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    let output = components.serialize().unwrap();
    (0..output.as_css().len())
        .find_map(|offset| match output.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original fixture has implicit closure")
}

fn strict_original(name: &str, ordinary: &str, unclosed: &str) {
    let p = property(name);
    let inputs = [
        format!("/*😀*/{unclosed}"),
        format!("{ordinary}/*unfinished"),
        "inherit/*unfinished".into(),
        "var(--generated".into(),
        "env(shape".into(),
        "attr(data-generated".into(),
        "var(--generated)/*unfinished".into(),
    ];
    for input in inputs {
        let components = parse_component_values(&input).unwrap();
        let original = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            let error = checked(p, components.clone(), grammar).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ),
                "{input}: {error:?}"
            );
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(components, original);
    }
}

#[test]
fn all_six_content_properties_reject_original_unclosed_comments_functions_and_pending_roots() {
    for (name, ordinary, open) in [
        ("content", "counter(chapter)", "counter(chapter"),
        ("quotes", "\"«\" \"»\"", "var(--pairs"),
        ("string-set", "Chapter \"A\"", "Chapter var(--text"),
        ("bookmark-level", "1", "calc(1 + 2"),
        ("bookmark-label", "content(text)", "content(text"),
        ("bookmark-state", "open", "var(--state"),
    ] {
        strict_original(name, ordinary, open);
    }
}

#[test]
fn reentry_rejects_original_closure_after_residual_scan_and_remains_reusable_for_all_six() {
    for (name, valid, open) in [
        ("content", "counter(chapter)", "counter(chapter"),
        ("quotes", "\"a\" \"b\"", "\"a\" \"b\"/*unfinished"),
        ("string-set", "Chapter \"A\"", "Chapter \"A\"/*unfinished"),
        ("bookmark-level", "1", "calc(1 + 2"),
        ("bookmark-label", "content(text)", "content(text"),
        ("bookmark-state", "open", "open/*unfinished"),
    ] {
        let source = declaration(name, "var(--generated)", 0);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = parse_component_values(open).unwrap();
        let origin = implicit_origin(&replacement);
        let error = handle.reenter(replacement).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("closure failure")
        };
        assert!(matches!(
            actual.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ));
        assert_eq!(actual.origin(), &CssSerializedOrigin::End(Some(origin)));
        assert_eq!(
            handle
                .reenter(parse_component_values("future(var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        assert!(
            handle
                .reenter(parse_component_values(valid).unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
}

#[test]
fn typed_attribute_functions_keep_the_whole_occurrence_pending_until_full_replacement() {
    for (name, input, valid) in [
        (
            "content",
            "\"caption\" / attr(data-alt)",
            "\"caption\" / \"spoken\"",
        ),
        ("quotes", "attr(data-pairs)", "\"a\" \"b\""),
        (
            "string-set",
            "Chapter attr(data-title)",
            "Chapter \"Title\"",
        ),
        ("bookmark-level", "attr(data-level <integer>, 1)", "1"),
        (
            "bookmark-label",
            "content(text) attr(data-label)",
            "content(text) \"Title\"",
        ),
        ("bookmark-state", "attr(data-state)", "open"),
    ] {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let original = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("valid attr is whole pending")
            };
            assert_eq!(
                handle
                    .reenter(parse_component_values(input).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            assert!(
                handle
                    .reenter(parse_component_values(valid).unwrap())
                    .is_ok()
            );
            assert_eq!(source, original);
        }
    }
}

#[test]
fn normalized_six_property_composition_keeps_context_order_and_cumulative_contributions() {
    let input = ".a{content:counter(chapter)!important;quotes:match-parent;string-set:Chapter \"A\", Author \"B\";bookmark-level:1;bookmark-label:var(--label);bookmark-state:closed!important}";
    let report = parse_sheet(input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style rule")
    };
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 6, 6).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    for (order, (item, source)) in declarations
        .iter()
        .zip(style.declarations().iter())
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                let [value] = values.items() else {
                    panic!("one terminal")
                };
                assert!(value.source().same_occurrence(source));
                assert_eq!(value.property(), property(NAMES[order]));
            }
            CssExpansion::Pending(handle) if order == 4 => {
                assert!(handle.source().same_occurrence(source))
            }
            other => panic!("one authored occurrence, got {other:?}"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(0, 1, 5, 6).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 6, 5).unwrap(),
            CssNormalizationResource::Contributions,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit: 5 }
        );
        assert_eq!(error.declaration_order(), Some(5));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(&style.declarations()[5])
        );
        assert!(error.rule_context().is_some());
        assert_eq!(report, before);
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn universal_reset_includes_all_six_authored_content_properties_symbolically() {
    let source = declaration("all", "initial", 0);
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("all")
    };
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property(name))));
    }
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Direction)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::UnicodeBidi)));
}

fn budget(name: &str, input: &str, value: &str, count: usize) {
    let output = format!("{name}: {value} !important;");
    for front in 0..5 {
        let source = declaration(name, input, front);
        let original = source.clone();
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(count, count, output.len()))
                .unwrap(),
            output
        );
        for (limits, kind) in [
            (L::new(count - 1, count, output.len()), K::InputNodeLimit),
            (
                L::new(count, count - 1, output.len()),
                K::ProjectionNodeLimit,
            ),
            (L::new(count, count, output.len() - 1), K::ByteLimit),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, original);
            }
        }
        assert_eq!(source.to_specified_css().unwrap(), output);
    }
}

#[test]
fn simple_keyword_and_integer_declarations_have_one_shared_leaf_budget() {
    for (name, input, expected) in [
        ("content", "none", "none"),
        ("quotes", "auto", "auto"),
        ("string-set", "none", "none"),
        ("bookmark-level", "+0001", "1"),
        ("bookmark-state", "CLOSED", "closed"),
    ] {
        budget(name, input, expected, 3);
    }
}

#[test]
fn ordered_string_set_entries_and_strings_share_one_cumulative_budget() {
    // Outer list1 + two entry aggregates2 + names2 + strings3 + declaration2.
    budget(
        "string-set",
        "Chapter 'A' 'B', Author 'C'",
        "Chapter \"A\" \"B\", Author \"C\"",
        10,
    );
}

#[test]
fn bookmark_label_uses_content_list_aggregate_and_charges_explicit_omitted_defaults() {
    // List1 + two string leaves2 + declaration2 =5.
    budget("bookmark-label", "'é' '😀'", "\"é\" \"😀\"", 5);
    // List1 + function1 + explicitly authored text default1 + declaration2 =5.
    budget("bookmark-label", "content(text)", "content()", 5);
    budget("bookmark-label", "content()", "content()", 4);
    // List1 + counter function1 + name1 + explicit decimal1 + declaration2 =6.
    budget(
        "bookmark-label",
        "counter(chapter,decimal)",
        "counter(chapter)",
        6,
    );
    budget("bookmark-label", "counter(chapter)", "counter(chapter)", 5);
}

#[test]
fn nested_counter_image_and_numeric_children_in_labels_share_atomic_byte_and_work_limits() {
    let input = "counter(chapter,symbols(linear-gradient(red calc(1px + 2%),blue) url(\"#star\" policy(flag))))";
    let value = "counter(chapter, symbols(linear-gradient(red calc(2% + 1px), blue) url(\"#star\" policy(flag))))";
    for name in ["content", "bookmark-label"] {
        let source = declaration(name, input, 0);
        let original = source.clone();
        let output = format!("{name}: {value} !important;");
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(128, 256, output.len()))
                .unwrap(),
            output
        );
        for (limits, kind) in [
            (L::new(128, 256, output.len() - 1), K::ByteLimit),
            (L::new(0, 256, output.len()), K::InputNodeLimit),
            (L::new(128, 0, output.len()), K::ProjectionNodeLimit),
        ] {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(source, original);
        }
    }
}

#[test]
fn accepted_content_provider_control_preserves_order_defaults_and_symbolic_targets() {
    accept(
        "content",
        &[
            (
                "counter(chapter,DECIMAL) string(Chapter,first) content(text)",
                "counter(chapter) string(Chapter) content()",
            ),
            (
                "leader(dotted) target-text('#x',content)",
                "leader(\".\") target-text(\"#x\", content)",
            ),
            (
                "'caption' / counters(chapter,'.',decimal)",
                "\"caption\" / counters(chapter, \".\")",
            ),
        ],
    );
}

#[test]
fn accepted_quotes_and_positive_integer_provider_controls_distinguish_authored_phases() {
    accept(
        "quotes",
        &[("MATCH-PARENT", "match-parent"), ("'«' '»'", "\"«\" \"»\"")],
    );
    accept(
        "column-count",
        &[("+0001", "1"), ("calc(1 - 2)", "calc(-1)")],
    );
    invalid("column-count", &["0", "-1", "1.0", "1e2"]);
}

#[test]
fn string_and_label_escaping_preserves_decoded_control_and_unicode_children() {
    accept(
        "string-set",
        &[(r#"Chapter "x\a y" "\1 é""#, r#"Chapter "x\a y" "\1 é""#)],
    );
    accept(
        "bookmark-label",
        &[(r#""a\"b" "c\\d""#, r#""a\"b" "c\\d""#)],
    );
    accept("quotes", &[(r#""x\a y" "\1 é""#, r#""x\a y" "\1 é""#)]);
}

#[test]
fn full_image_depth_error_survives_content_list_retry_and_pending_reentry() {
    // Raw unquoted URL is a token; typed URL is an image function at depth1.
    // 256 LightDark wrappers therefore fit raw components but exceed the typed
    // subtree ceiling. The accepted Image owner defines this resource contract.
    let mut text = "url(a.svg)".to_owned();
    for _ in 0..256 {
        text = format!("light-dark({text}, none)");
    }
    let components = parse_component_values(&text).expect("raw structural ceiling");
    assert_eq!(components.nesting_depth(), 256);
    let original = components.clone();
    for name in ["content", "bookmark-label"] {
        let p = property(name);
        for grammar in [false, true] {
            let error = checked(p, components.clone(), grammar).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
                ),
                "{error:?}"
            );
        }
        let source = declaration(name, "var(--image)", 0);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending image consumer")
        };
        for _ in 0..2 {
            let direct = checked(p, components.clone(), false).unwrap_err();
            let error = handle.reenter(components.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("typed resource failure")
            };
            assert_eq!(error.kind(), direct.kind());
            assert_eq!(error.origin(), direct.origin());
        }
        assert!(
            handle
                .reenter(parse_component_values("content(text)").unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
    assert_eq!(components, original);
}
