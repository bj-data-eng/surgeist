#![forbid(unsafe_code)]
//! Authored hyphenation contract through existing public front doors.
//!
//! Text4 WD20260814 tables 2734–2957 and clauses 2839–2847, 2908–2914;
//! Text3 CRD20260814 hyphens; Values3/4 §2.2 grouping and whole globals;
//! Values4 WD20240312 §5.1/5.2 and §10.12 deferred integer ranges.
//! Selected product policy: six inherited ordinary longhands, singleton initials,
//! no resets; chars preserves authored arity, signed zone, exact nonnegative
//! integer literals versus deferred function math; transparent leaf pricing.
//! Finite/string leaves cost1/1, numeric children retain their existing prices;
//! declaration/name add2, whitespace and quoted escapes count final UTF-8 bytes.
//! No future enum/borrowed variants, constructors, helpers or missing-API stubs.
//! Generic retained components expose string provenance; new typed string origin,
//! effective chars triples and bare-calculation constructor admission are tested
//! functionally alongside implementation.
//! Used hyphenation, language dictionaries, UA auto counts and rounding are excluded.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const FAMILY: [&str; 6] = [
    "hyphens",
    "hyphenate-character",
    "hyphenate-limit-zone",
    "hyphenate-limit-chars",
    "hyphenate-limit-lines",
    "hyphenate-limit-last",
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn property(name: &str) -> P {
    P::from_name(name).unwrap_or_else(|| panic!("authored property unavailable: {name}"))
}
fn initial(name: &str) -> &'static str {
    match name {
        "hyphens" => "manual",
        "hyphenate-character" | "hyphenate-limit-chars" => "auto",
        "hyphenate-limit-zone" => "0",
        "hyphenate-limit-lines" => "no-limit",
        "hyphenate-limit-last" => "none",
        _ => panic!("selected family"),
    }
}
fn positives(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "hyphens" => &[
            ("none", "none"),
            ("manual", "manual"),
            ("auto", "auto"),
            (r"\6d anual", "manual"),
            ("AUTO/**/", "auto"),
        ],
        "hyphenate-character" => &[
            ("auto", "auto"),
            ("AUTO", "auto"),
            (r#""""#, r#""""#),
            (r#"" ""#, r#"" ""#),
            (r#""😀᐀""#, r#""😀᐀""#),
            (r#""\2010 ""#, r#""‐""#),
            (r#""inherit""#, r#""inherit""#),
            (r#"'a"b'"#, r#""a\"b""#),
            (r#""a\\b""#, r#""a\\b""#),
        ],
        "hyphenate-limit-zone" => &[
            ("0", "0"),
            ("-0", "0"),
            ("-2PX", "-2px"),
            ("-1e-999px", "0px"),
            ("-25%", "-25%"),
            ("25%", "25%"),
            (".5em", "0.5em"),
            (r"2\70 x", "2px"),
            ("calc(-2px - 3%)", "calc(-3% - 2px)"),
            ("calc(1px + 2px)", "calc(3px)"),
            ("calc(2em + 1px)", "calc(2em + 1px)"),
        ],
        "hyphenate-limit-chars" => &[
            ("auto", "auto"),
            ("0", "0"),
            ("-0", "0"),
            ("+0002", "2"),
            ("8 auto", "8 auto"),
            ("8 2", "8 2"),
            ("8 2 auto", "8 2 auto"),
            ("8 auto 3", "8 auto 3"),
            ("auto 0 -0", "auto 0 0"),
            ("auto auto auto", "auto auto auto"),
            ("calc(-1)", "calc(-1)"),
            ("calc(1.5) auto", "calc(1.5) auto"),
            ("calc(1 - 3) 2 calc(3 / 2)", "calc(-2) 2 calc(1.5)"),
        ],
        "hyphenate-limit-lines" => &[
            ("no-limit", "no-limit"),
            ("NO-LIMIT", "no-limit"),
            ("0", "0"),
            ("-0", "0"),
            ("+0002", "2"),
            ("42", "42"),
            ("calc(-1)", "calc(-1)"),
            ("calc(1.5)", "calc(1.5)"),
            ("calc(1 - 3)", "calc(-2)"),
        ],
        "hyphenate-limit-last" => &[
            ("none", "none"),
            ("always", "always"),
            ("column", "column"),
            ("page", "page"),
            ("spread", "spread"),
            (r"\63 olumn", "column"),
            ("SPREAD/**/", "spread"),
        ],
        _ => panic!("selected family"),
    }
}
fn negatives(name: &str) -> &'static [&'static str] {
    match name {
        "hyphens" => &[
            "",
            "normal",
            "none auto",
            "manual manual",
            "manual,auto",
            "1",
            "\"auto\"",
            "[auto]",
        ],
        "hyphenate-character" => &[
            "",
            "none",
            "manual",
            "1",
            "1px",
            "[auto]",
            "auto auto",
            "\"a\" \"b\"",
            "auto \"x\"",
            "url(x)",
        ],
        "hyphenate-limit-zone" => &[
            "",
            "auto",
            "1",
            "1s",
            "1fr",
            "1px 2px",
            "1px,2px",
            "\"1px\"",
            "[1px]",
            "calc(1 + 1px)",
            "calc(1px + 1s)",
            "sin(1px)",
        ],
        "hyphenate-limit-chars" => &[
            "",
            "none",
            "no-limit",
            "-1",
            "-0001",
            "1.0",
            "1e0",
            "-0.0",
            "-0e0",
            "1px",
            "1%",
            "1,2",
            "1 2 3 4",
            "auto auto auto auto",
            "8 -1",
            "8 2 -1",
            "calc(1px)",
            "calc(1%)",
            "calc(1 + 1px)",
            "[2]",
            "\"2\"",
        ],
        "hyphenate-limit-lines" => &[
            "",
            "auto",
            "none",
            "-1",
            "-0001",
            "1.0",
            "1e0",
            "-0.0",
            "-0e0",
            "1px",
            "1%",
            "1 2",
            "no-limit 1",
            "calc(1px)",
            "calc(1%)",
            "calc(1 + 1px)",
            "[2]",
            "\"2\"",
        ],
        "hyphenate-limit-last" => &[
            "",
            "auto",
            "normal",
            "never",
            "none page",
            "page spread",
            "page page",
            "page,column",
            "1",
            "\"none\"",
            "[none]",
        ],
        _ => panic!("selected family"),
    }
}
fn parsed(name: &str, text: &str) -> CssDeclaration {
    let p = property(name);
    let css = format!("/*😀*/{}:{text}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    source.clone()
}
fn checked_components(
    name: &str,
    components: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let p = property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components,
            CssImportance::Important,
        )
    }
}
fn checked(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let before = components.clone();
    let source = checked_components(name, components.clone(), grammar).unwrap();
    assert_eq!(source.value_components(), &components);
    assert_eq!(components, before);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(source.known().unwrap().grammar(), property(name).grammar());
    source
}
fn text_front(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let p = property(name);
    let report = if grammar {
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important)
    } else {
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important)
    };
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let source = report.syntax().as_ref().unwrap().clone();
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
    source
}
fn fronts(name: &str, text: &str) -> [CssDeclaration; 5] {
    [
        parsed(name, text),
        checked(name, text, false),
        checked(name, text, true),
        text_front(name, text, false),
        text_front(name, text, true),
    ]
}
fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete longhand contribution")
    };
    assert_eq!(values.items().len(), 1);
    assert_eq!(
        values.items()[0].property(),
        source.known().unwrap().property()
    );
    assert_source(&values.items()[0], source);
    values
}
fn accepted(name: &str, text: &str, canonical: &str) {
    for source in fronts(name, text) {
        let before = source.clone();
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {canonical} !important;")
        );
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        let values = completed(&source);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            property(name)
        );
        assert!(item.replacement_components().is_none());
        assert_eq!(source, before);
    }
    assert_eq!(
        checked(name, canonical, true).to_specified_css().unwrap(),
        format!("{name}: {canonical} !important;")
    );
}
fn invalid(name: &str, text: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{text};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|v| v.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic grammar failure")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("property-specific diagnostic")
    };
    assert_eq!(detail.property(), p);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(text).unwrap();
    let before = components.clone();
    for grammar in [false, true] {
        assert!(matches!(
            checked_components(name, components.clone(), grammar)
                .unwrap_err()
                .kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    for report in [
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(components, before);
}
fn intrinsic(name: &str) {
    let p = property(name);
    assert_eq!(P::from_name(&name.to_ascii_uppercase()), Some(p));
    assert_eq!(CssPropertyGrammar::from_name(name), Some(p.grammar()));
    let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
        panic!("one terminal, no resets")
    };
    assert_eq!(metadata.property().known_property(), p);
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    assert_eq!(initial_value.property().known_property(), p);
    let CssInitialValueRef::Value(value) = initial_value.view() else {
        panic!("ordinary intrinsic initial")
    };
    // Both payloads have programmatic origins. Do not compare a separately
    // parsed numeric child's span against this programmatic metadata value.
    let component = if name == "hyphenate-limit-zone" {
        CssComponentValue::try_number("0").unwrap()
    } else {
        CssComponentValue::try_ident(initial(name)).unwrap()
    };
    let source = checked_components(
        name,
        CssComponentValues::try_new(vec![component]).unwrap(),
        false,
    )
    .unwrap();
    assert_eq!(
        source.to_specified_css().unwrap(),
        format!("{name}: {} !important;", initial(name))
    );
    assert_eq!(completed(&source).items()[0].ordinary_value(), Some(value));
}
fn ordinary(name: &str) {
    for &(text, canonical) in positives(name) {
        accepted(name, text, canonical);
    }
}
fn invalid_grammar(name: &str) {
    for text in negatives(name) {
        invalid(name, text);
    }
    for (text, _) in GLOBALS {
        invalid(name, &format!("{text} {}", initial(name)));
        invalid(name, &format!("{} {text}", initial(name)));
    }
}
fn globals(name: &str) {
    for (text, keyword) in GLOBALS {
        for source in fronts(name, text) {
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            assert!(source.known().unwrap().property_value().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {text} !important;")
            );
            let values = completed(&source);
            assert!(
                matches!(values.items()[0].value(),CssContributionValueRef::Global(v) if v==keyword)
            );
            assert!(values.items()[0].replacement_components().is_none());
        }
    }
}
fn pending_inputs(name: &str) -> Vec<&'static str> {
    let mut inputs = vec!["var(--hyphen)", "env(hyphen)", "attr(data-hyphen *)"];
    if matches!(
        name,
        "hyphenate-limit-zone" | "hyphenate-limit-chars" | "hyphenate-limit-lines"
    ) {
        inputs.extend([
            "calc(var(--hyphen))",
            "calc(env(hyphen))",
            "calc(attr(data-hyphen *))",
        ]);
    }
    if name == "hyphenate-limit-chars" {
        inputs.push("8 var(--before) auto");
    }
    inputs
}
fn pending(name: &str) {
    for input in pending_inputs(name) {
        for source in fronts(name, input) {
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("one pending occurrence")
            };
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {input} !important;")
            );
            for text in [
                "var(--again)",
                "env(again)",
                "attr(data-again *)",
                "calc(var(--again))",
                r"\76 ar(--again)",
                "var(--again)/*",
                "env(again",
                "attr(data-again *",
            ] {
                for _ in 0..2 {
                    assert!(matches!(
                        handle
                            .reenter(parse_component_values(text).unwrap())
                            .unwrap_err()
                            .kind(),
                        CssExpansionErrorKind::ResidualSubstitution
                    ));
                }
            }
            for text in negatives(name).iter().copied().chain([
                "initial auto",
                "auto!important",
                "auto;color:red",
            ]) {
                let replacement = parse_component_values(text).unwrap();
                let snapshot = replacement.clone();
                for _ in 0..2 {
                    let error = handle.reenter(replacement.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("strict replacement")
                    };
                    assert!(matches!(
                        error.kind(),
                        CssPropertyValueErrorKind::Grammar(_)
                    ));
                }
                assert_eq!(replacement, snapshot);
            }
            for &(text, canonical) in positives(name) {
                let replacement = parse_component_values(text).unwrap();
                let snapshot = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("ordinary replacement")
                };
                assert_eq!(values.items().len(), 1);
                let item = &values.items()[0];
                assert_eq!(item.property(), property(name));
                assert_source(item, &source);
                assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                assert_eq!(item.replacement_components(), Some(&replacement));
                for (actual, supplied) in item
                    .replacement_components()
                    .unwrap()
                    .items()
                    .iter()
                    .zip(replacement.items())
                {
                    assert_eq!(actual.origin(), supplied.origin());
                    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                        (actual.origin(), supplied.origin())
                    else {
                        panic!("replacement origin")
                    };
                    assert!(actual.source().same_snapshot(supplied.source()));
                    assert_eq!(actual.source().as_str(), text);
                }
                let ordinary = checked_components(name, replacement.clone(), true).unwrap();
                assert_eq!(
                    ordinary.to_specified_css().unwrap(),
                    format!("{name}: {canonical} !important;")
                );
                // Equal references here derive from exactly the same supplied
                // component snapshot, not unrelated parsed origins.
                assert_eq!(
                    item.ordinary_value(),
                    completed(&ordinary).items()[0].ordinary_value()
                );
                assert_eq!(
                    ordinary
                        .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                            0,
                            usize::MAX,
                            usize::MAX
                        ))
                        .unwrap_err()
                        .kind(),
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                );
                assert!(handle.reenter(replacement.clone()).is_ok());
                assert_eq!(replacement, snapshot);
            }
            for (text, keyword) in GLOBALS {
                let replacement = parse_component_values(text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("global replacement")
                };
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], &source);
                assert!(
                    matches!(values.items()[0].value(),CssContributionValueRef::Global(v) if v==keyword)
                );
                assert_eq!(
                    values.items()[0].replacement_components(),
                    Some(&replacement)
                );
            }
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(source, before);
        }
    }
}
fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("actual public origin mapping for original EOF")
}
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("original implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end(), at.span().start());
}
fn closure_inputs(name: &str) -> Vec<String> {
    let mut inputs: Vec<_> = positives(name)
        .iter()
        .map(|(text, _)| format!("{text}/*"))
        .collect();
    inputs.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
    inputs.extend(
        [
            "var(--hyphen)/*",
            "env(hyphen)/*",
            "attr(data-hyphen *)/*",
            "var(--hyphen",
            "env(hyphen",
            "attr(data-hyphen *",
        ]
        .map(str::to_owned),
    );
    match name {
        "hyphenate-character" => inputs.extend(["\"😀", "\""].map(str::to_owned)),
        "hyphenate-limit-zone" => inputs.push("calc(1px + 2em".into()),
        "hyphenate-limit-chars" | "hyphenate-limit-lines" => inputs.push("calc(3 / 2".into()),
        _ => {}
    }
    inputs
}
fn closure(name: &str) {
    for text in closure_inputs(name) {
        let components = parse_component_values(&text).unwrap();
        let before = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            assert_closure(
                &checked_components(name, components.clone(), grammar).unwrap_err(),
                &origin,
                &text,
            );
        }
        assert_eq!(components, before);
    }
    for &(text, canonical) in positives(name) {
        accepted(name, &format!("{text}/**/"), canonical);
    }
    for (text, _) in GLOBALS {
        checked(name, &format!("{text}/**/"), false);
        checked(name, &format!("{text}/**/"), true);
    }
}
macro_rules! property_contract {
    ($module:ident,$name:literal) => {
        mod $module {
            use super::*;
            #[test]
            fn initial_is_exact_inherited_singleton_ordinary_metadata() {
                intrinsic($name);
            }
            #[test]
            fn selected_ordinary_values_have_canonical_single_terminal_lifecycle() {
                ordinary($name);
            }
            #[test]
            fn invalid_grammar_and_global_mixtures_drop_only_the_owned_declaration() {
                invalid_grammar($name);
            }
            #[test]
            fn five_whole_globals_preserve_source_and_importance() {
                globals($name);
            }
            #[test]
            fn var_env_attr_reentry_retries_and_preserves_original_and_replacement() {
                pending($name);
            }
            #[test]
            fn checked_fronts_reject_original_implicit_closure_and_admit_complete_controls() {
                closure($name);
            }
        }
    };
}
property_contract!(hyphens, "hyphens");
property_contract!(character, "hyphenate-character");
property_contract!(zone, "hyphenate-limit-zone");
property_contract!(chars, "hyphenate-limit-chars");
property_contract!(lines, "hyphenate-limit-lines");
property_contract!(last, "hyphenate-limit-last");

#[test]
fn arbitrary_integer_magnitude_signed_zero_and_chars_authored_arity_are_preserved() {
    let huge = format!("1{}", "0".repeat(999));
    for name in ["hyphenate-limit-chars", "hyphenate-limit-lines"] {
        accepted(name, &huge, &huge);
        accepted(name, &format!("+000{huge}"), &huge);
        invalid(name, &format!("-{huge}"));
        accepted(name, "-0000", "0");
    }
    for (text, canonical) in [
        (huge.clone(), huge.clone()),
        (format!("{huge} auto"), format!("{huge} auto")),
        (format!("{huge} 2 3"), format!("{huge} 2 3")),
    ] {
        accepted("hyphenate-limit-chars", &text, &canonical);
    }
    for text in ["8", "8 auto", "8 auto auto", "8 2", "8 2 2"] {
        accepted("hyphenate-limit-chars", text, text);
    }
}
#[test]
fn every_selected_integer_function_uses_shared_math_without_computed_clamping_or_rounding() {
    for (text, canonical) in [
        ("calc(1 + 2)", "calc(3)"),
        ("min(-2, 3)", "calc(-2)"),
        ("max(-2, 3)", "calc(3)"),
        ("clamp(-2, 1, 3)", "calc(1)"),
        ("round(2.5, 1)", "calc(3)"),
        ("mod(-18, 5)", "calc(2)"),
        ("rem(-18, 5)", "calc(-3)"),
        ("sin(90deg)", "calc(1)"),
        ("cos(0)", "calc(1)"),
        ("tan(0turn)", "calc(0)"),
        ("calc(asin(1) / 90deg)", "calc(1)"),
        ("calc(acos(1) / 1deg)", "calc(0)"),
        ("calc(atan(infinity) / 90deg)", "calc(1)"),
        ("calc(atan2(1, 0) / 90deg)", "calc(1)"),
        ("pow(2, 3)", "calc(8)"),
        ("sqrt(4)", "calc(2)"),
        ("hypot(3, 4)", "calc(5)"),
        ("log(1, 2)", "calc(0)"),
        ("exp(0)", "calc(1)"),
        ("abs(-2)", "calc(2)"),
        ("sign(-2)", "calc(-1)"),
        ("calc(3 / 2)", "calc(1.5)"),
        ("calc(-3 / 2)", "calc(-1.5)"),
        ("calc(infinity)", "calc(infinity)"),
        ("calc(-infinity)", "calc(-infinity)"),
        ("calc(NaN)", "calc(NaN)"),
        ("sqrt(-1)", "calc(NaN)"),
    ] {
        for name in ["hyphenate-limit-chars", "hyphenate-limit-lines"] {
            accepted(name, text, canonical);
        }
    }
}
#[test]
fn retained_string_components_preserve_decoded_payload_utf8_offsets_and_source_snapshots() {
    for (text, decoded, canonical) in [
        (r#""""#, "", r#""""#),
        (r#"" ""#, " ", r#"" ""#),
        (r#""😀᐀""#, "😀᐀", r#""😀᐀""#),
        (r#""\2010 ""#, "‐", r#""‐""#),
        (r#""auto""#, "auto", r#""auto""#),
        (r#""a\\b""#, "a\\b", r#""a\\b""#),
    ] {
        for source in fronts("hyphenate-character", text) {
            let component = source
                .value_components()
                .items()
                .iter()
                .find(|v| {
                    matches!(
                        v.view(),
                        CssComponentValueRef::Token(CssValueTokenRef::String(_))
                    )
                })
                .unwrap();
            assert!(
                matches!(component.view(),CssComponentValueRef::Token(CssValueTokenRef::String(value)) if value==decoded)
            );
            let CssValueOrigin::Parsed(origin) = component.origin() else {
                panic!("authored string origin")
            };
            let original = origin.source().as_str();
            let start = origin.span().start().byte_offset().value();
            let end = origin.span().end().byte_offset().value();
            assert_eq!(&original[start..end], text);
            if let Some(value) = source.parsed_value() {
                assert!(origin.source().same_snapshot(value.source()));
            }
            let before = source.clone();
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("hyphenate-character: {canonical} !important;")
            );
            let values = completed(&source);
            assert_source(&values.items()[0], &source);
            assert_eq!(source, before);
        }
    }
}
#[test]
fn existing_programmatic_components_admit_six_roles_without_forging_parsed_origins() {
    let fixtures = [
        (
            "hyphens",
            vec![CssComponentValue::try_ident("manual").unwrap()],
            "manual",
        ),
        (
            "hyphenate-character",
            vec![CssComponentValue::try_string("😀").unwrap()],
            r#""😀""#,
        ),
        (
            "hyphenate-limit-zone",
            vec![CssComponentValue::try_dimension("-2", "px").unwrap()],
            "-2px",
        ),
        (
            "hyphenate-limit-chars",
            vec![
                CssComponentValue::try_number("8").unwrap(),
                CssComponentValue::try_number("2").unwrap(),
                CssComponentValue::try_ident("auto").unwrap(),
            ],
            "8 2 auto",
        ),
        (
            "hyphenate-limit-lines",
            vec![CssComponentValue::try_number("-0").unwrap()],
            "0",
        ),
        (
            "hyphenate-limit-last",
            vec![CssComponentValue::try_ident("page").unwrap()],
            "page",
        ),
    ];
    for (name, items, canonical) in fixtures {
        let components = CssComponentValues::try_new(items).unwrap();
        let before = components.clone();
        for grammar in [false, true] {
            let source = checked_components(name, components.clone(), grammar).unwrap();
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|v| v.origin() == &CssValueOrigin::Programmatic)
            );
            assert_eq!(source.value_components(), &components);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {canonical} !important;")
            );
            assert!(source.position().is_none());
            completed(&source);
        }
        assert_eq!(components, before);
    }
}
#[test]
fn recovered_replacements_keep_original_eof_and_residual_priority_then_allow_retry() {
    for name in FAMILY {
        let source = checked(name, "var(--hyphen)", false);
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for text in closure_inputs(name) {
            let components = parse_component_values(&text).unwrap();
            let origin = implicit_origin(&components);
            for _ in 0..2 {
                let error = handle.reenter(components.clone()).unwrap_err();
                if text.contains("var(") || text.contains("env(") || text.contains("attr(") {
                    assert!(matches!(
                        error.kind(),
                        CssExpansionErrorKind::ResidualSubstitution
                    ));
                } else {
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("original closure failure")
                    };
                    assert_closure(error, &origin, &text);
                }
            }
        }
        let replacement = parse_component_values(initial(name)).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("successful retry")
        };
        assert_eq!(values.items().len(), 1);
        assert_source(&values.items()[0], &source);
        assert_eq!(
            values.items()[0].replacement_components(),
            Some(&replacement)
        );
        assert_eq!(source, before);
    }
}
#[test]
fn browser_recovery_retains_diagnostics_and_one_normalized_original_occurrence() {
    for name in FAMILY {
        let mut texts = vec![
            format!("{}/*", initial(name)),
            "initial/*".into(),
            "var(--hyphen".into(),
        ];
        if name == "hyphenate-character" {
            texts.push("\"😀".into());
        }
        if name == "hyphenate-limit-zone" {
            texts.push("calc(-2px - 3%".into());
        }
        if matches!(name, "hyphenate-limit-chars" | "hyphenate-limit-lines") {
            texts.push("calc(3 / 2".into());
        }
        for text in texts {
            let css = format!(".a{{{name}:{text}");
            let report = parse_sheet(&css);
            assert!(!report.is_clean());
            assert!(validate_sheet(&css).is_err());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            let before = report.clone();
            let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
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
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().known().unwrap().property(), property(name));
            assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), 1);
                    assert_source(&values.items()[0], item.source());
                }
                _ => panic!("one retained terminal or pending handle"),
            }
            let error = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded {
                    resource: CssNormalizationResource::Contributions,
                    limit: 0
                }
            );
            assert_eq!(error.declaration_order(), Some(0));
            assert!(error.declaration().unwrap().same_occurrence(item.source()));
            assert_eq!(report, before);
            assert!(normalize_report_with_limits(&report, exact).is_ok());
        }
    }
}
#[test]
fn normalization_preserves_all_six_order_importance_context_pending_and_exact_limits() {
    let css = "@media screen{.a{hyphens:auto!important;hyphenate-character:\"😀\";hyphenate-limit-zone:-25%;hyphenate-limit-chars:8 2;hyphenate-limit-lines:var(--lines);hyphenate-limit-last:page;hyphens:unset!important;color:red}}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 8, 8).unwrap();
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
    let names = [
        "hyphens",
        "hyphenate-character",
        "hyphenate-limit-zone",
        "hyphenate-limit-chars",
        "hyphenate-limit-lines",
        "hyphenate-limit-last",
        "hyphens",
        "color",
    ];
    assert_eq!(declarations.len(), 8);
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            names[index]
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 6 {
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
                assert_eq!(index, 4);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
                if index == 6 {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                    ));
                } else {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Ordinary(_)
                    ));
                }
            }
            _ => panic!("one terminal or one pending occurrence"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 8, 7).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 7, 8).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_report_with_limits(&report, limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 7 }
            );
            assert_eq!(error.declaration_order(), Some(7));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[7].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn canonical_declarations_have_exact_leaf_math_and_byte_limits_with_atomic_retry() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    for (name, text, canonical, input, projection) in [
        ("hyphens", "manual", "manual", 1, 1),
        ("hyphenate-character", r#""😀""#, r#""😀""#, 1, 1),
        ("hyphenate-character", r#""""#, r#""""#, 1, 1),
        ("hyphenate-limit-zone", "-25%", "-25%", 1, 1),
        ("hyphenate-limit-zone", "calc(1px + 2px)", "calc(3px)", 4, 3),
        ("hyphenate-limit-chars", "8", "8", 1, 1),
        ("hyphenate-limit-chars", "8 auto", "8 auto", 2, 2),
        ("hyphenate-limit-chars", "8 auto 3", "8 auto 3", 3, 3),
        ("hyphenate-limit-lines", "no-limit", "no-limit", 1, 1),
        ("hyphenate-limit-lines", "42", "42", 1, 1),
        ("hyphenate-limit-last", "spread", "spread", 1, 1),
    ] {
        let source = checked(name, text, true);
        let before = source.clone();
        let expected = format!("{name}: {canonical} !important;");
        let exact = L::new(input + 2, projection + 2, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                L::new(input + 1, projection + 2, expected.len()),
                K::InputNodeLimit,
            ),
            (
                L::new(input + 2, projection + 1, expected.len()),
                K::ProjectionNodeLimit,
            ),
            (
                L::new(input + 2, projection + 2, expected.len() - 1),
                K::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
    let huge = format!("1{}", "0".repeat(999));
    let source = checked("hyphenate-limit-lines", &huge, true);
    let expected = format!("hyphenate-limit-lines: {huge} !important;");
    assert_eq!(
        source
            .to_specified_css_with_limits(L::new(3, 3, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        source
            .to_specified_css_with_limits(L::new(3, 3, expected.len() - 1))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(source.to_specified_css().unwrap(), expected);
}
#[test]
fn sibling_sheet_output_shares_exact_work_and_utf8_bytes_without_partial_results() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    let report = parse_sheet(".a{hyphens:manual!important}.b{hyphenate-limit-chars:8 auto 3}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { hyphens: manual !important; }\n.b { hyphenate-limit-chars: 8 auto 3; }";
    // Sheet1 + each style rule1, selector list1, simple class1, declaration
    // list1; declarations cost3 and5 (two carriers plus one/three leaves).
    let exact = L::new(17, 17, expected.len());
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    for (limits, kind) in [
        (L::new(16, 17, expected.len()), K::InputNodeLimit),
        (L::new(17, 16, expected.len()), K::ProjectionNodeLimit),
        (L::new(17, 17, expected.len() - 1), K::ByteLimit),
    ] {
        assert!(
            sheet
                .rules()
                .iter()
                .all(|rule| rule.to_specified_css_with_limits(limits).is_ok())
        );
        for _ in 0..2 {
            let error = sheet.to_specified_css_with_limits(limits).unwrap_err();
            assert_eq!(
                error.kind(),
                CssSpecifiedRuleSerializationErrorKind::Resource(kind)
            );
            assert_eq!(error.rule_index(), Some(1));
            assert_eq!(sheet, &before);
        }
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn existing_color_and_overflow_wrap_controls_exercise_the_same_public_lifecycle() {
    accepted("color", "red", "red");
    accepted("overflow-wrap", "anywhere", "anywhere");
    let css = ".a{color:red;overflow-wrap:anywhere}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let normalized = normalize_report_with_limits(
        &report,
        CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap(),
    )
    .unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    assert_eq!(
        normalized
            .syntax()
            .items()
            .iter()
            .filter(|v| matches!(v, CssNormalizedItem::Declaration(_)))
            .count(),
        2
    );
}
#[test]
fn existing_exact_integer_and_signed_length_owners_are_controls_for_new_admission() {
    let huge = format!("1{}", "0".repeat(999));
    let integer =
        CssIntegerLiteral::try_from_component(CssComponentValue::try_number(&huge).unwrap())
            .unwrap();
    assert_eq!(
        CssIntegerValue::Literal(integer)
            .serialize_specified()
            .unwrap(),
        huge
    );
    let zero = CssIntegerLiteral::try_from_component(CssComponentValue::try_number("-0").unwrap())
        .unwrap();
    assert!(zero.is_zero());
    assert!(!zero.is_negative());
    for text in ["1.0", "1e0"] {
        assert!(
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .is_err()
        );
    }
    let calculation =
        CssIntegerCalculation::try_from_components(parse_component_values("calc(3 / 2)").unwrap())
            .unwrap();
    assert_eq!(
        CssIntegerValue::Calculation(calculation)
            .serialize_specified()
            .unwrap(),
        "calc(1.5)"
    );
    let length = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_token("-25%").unwrap(),
    )
    .unwrap();
    assert_eq!(length.serialize_specified().unwrap(), "-25%");
}
#[test]
fn existing_closure_origin_and_color_provider_prices_are_independent_controls() {
    for text in ["manual/*", "\"😀", "calc(3 / 2"] {
        let components = parse_component_values(text).unwrap();
        let origin = implicit_origin(&components);
        let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
            panic!("original EOF")
        };
        assert!(opening.source().same_snapshot(at.source()));
        assert_eq!(at.source().as_str(), text);
        assert_eq!(at.span().start().byte_offset().value(), text.len());
        assert_eq!(at.span().start(), at.span().end());
    }
    let source = checked("color", "red", true);
    let expected = "color: red !important;";
    assert_eq!(
        source
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                3,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        source
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                3,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
