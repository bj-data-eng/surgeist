#![forbid(unsafe_code)]
//! Existing-public-API authored whitespace controls contract.
//! Text4 WD20260814 #propdef-word-space-transform (374–422) and #propdef-tab-size
//! (1472–1494), Values4 WD20240312 §2.2, typed math §10 and range §10.12,
//! Syntax3 CRD20211224 and existing checked original-closure/resource contracts.
//! Independent word grammar has five states, mandatory base and optional flag;
//! canonical base→auto-phrase follows grammar. Tab retains Number versus Length,
//! Number-first literal zero policy, exact literal range and deferred math range.
//! Product pricing: emitted word keyword1/1; transparent numeric child; declaration
//! adds2. Shared math/sorting/six-place rounding policy remains unchanged.
//! No absent typed symbols/stubs/private helpers or Debug-string oracles.
//! New typed discriminators/constructors/primitive providers/output suppression
//! receive functional implementation tests. Generic boundaries cover observable
//! authored admission, intrinsic expansion, provenance and canonical output.
//! Phrase/language/separator and tab-stop execution remain downstream; #53 owns
//! white-space-collapse/trim and is outside this two-property contract.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;
const FAMILY: [&str; 2] = ["word-space-transform", "tab-size"];
const WORD: [&str; 5] = [
    "none",
    "space",
    "ideographic-space",
    "space auto-phrase",
    "ideographic-space auto-phrase",
];
const TAB: [&str; 6] = ["8", "0", "0px", "calc(-1)", "calc(-1px)", "calc(2em + 1px)"];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn values(name: &str) -> &'static [&'static str] {
    match name {
        "word-space-transform" => &WORD,
        "tab-size" => &TAB,
        _ => panic!("selected grammar"),
    }
}
fn initial_text(name: &str) -> &'static str {
    if name == "tab-size" { "8" } else { "none" }
}
fn pending_inputs(name: &str) -> Vec<&'static str> {
    let mut inputs = vec!["var(--wrap)", "env(wrap)", "attr(data-wrap *)"];
    if name == "tab-size" {
        inputs.extend([
            "calc(var(--tab))",
            "calc(env(tab))",
            "calc(attr(data-tab *))",
        ]);
    } else {
        inputs.push("space var(--phrase)");
    }
    inputs
}
fn property(name: &str) -> P {
    P::from_name(name).unwrap_or_else(|| panic!("authored property unavailable: {name}"))
}
fn parsed(name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    source.clone()
}
fn checked_components(
    name: &str,
    value: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let p = property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), value, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            value,
            CssImportance::Important,
        )
    }
}
fn checked(name: &str, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = checked_components(name, components.clone(), grammar).unwrap();
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
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
    let item = &values.items()[0];
    assert_eq!(item.property(), source.known().unwrap().property());
    assert_source(item, source);
    values
}
fn accepted(name: &str, input: &str, canonical: &str) {
    for source in fronts(name, input) {
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
fn invalid(name: &str, value: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{value};color:blue");
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
    let components = parse_component_values(value).unwrap();
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
        parse_property_value_text(value, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(components, before);
}

#[test]
fn five_word_states_and_unordered_flag_spellings_have_canonical_single_terminals() {
    for name in FAMILY {
        let source = parsed(name, initial_text(name));
        assert_eq!(source.known().unwrap().property(), property(name));
        assert_eq!(
            P::from_name(&name.to_ascii_uppercase()),
            Some(property(name))
        );
        assert_eq!(
            CssPropertyGrammar::from_name(name),
            Some(property(name).grammar())
        );
    }
    for text in WORD {
        accepted("word-space-transform", text, text);
    }
    for (input, expected) in [
        ("auto-phrase space", "space auto-phrase"),
        (
            "auto-phrase ideographic-space",
            "ideographic-space auto-phrase",
        ),
    ] {
        accepted("word-space-transform", input, expected);
    }
}
#[test]
fn tab_literals_preserve_number_length_zero_and_exact_nonnegative_range() {
    for (input, expected) in [
        ("0", "0"),
        ("-0", "0"),
        ("-0e999", "0"),
        ("0px", "0px"),
        ("-0px", "0px"),
        ("8", "8"),
        ("1.250", "1.25"),
        ("1e3", "1000"),
        ("1.0000000000000001", "1"),
        ("1e-999", "0"),
        ("1e-999px", "0px"),
        ("2PX", "2px"),
        ("1.25em", "1.25em"),
        ("0.0000005", "0.000001"),
        ("0.0000004px", "0px"),
        ("calc(-1)", "calc(-1)"),
        ("calc(-1px)", "calc(-1px)"),
        ("calc(1px - 3px)", "calc(-2px)"),
        ("calc(1 - 3)", "calc(-2)"),
        ("calc(1px / 1px)", "calc(1)"),
        ("calc(1% / 1%)", "calc(1)"),
        ("calc(3px + 2em)", "calc(2em + 3px)"),
    ] {
        accepted("tab-size", input, expected);
    }
}
#[test]
fn every_selected_shared_math_function_can_compose_to_length_without_early_range_clamping() {
    // Exact identities and elementary function values, independently derived.
    for (input, expected) in [
        ("calc(1px + 2px)", "calc(3px)"),
        ("min(-2px, 3px)", "calc(-2px)"),
        ("max(-2px, 3px)", "calc(3px)"),
        ("clamp(-2px, 1px, 3px)", "calc(1px)"),
        ("round(2.5px, 1px)", "calc(3px)"),
        ("mod(-18px, 5px)", "calc(2px)"),
        ("rem(-18px, 5px)", "calc(-3px)"),
        ("calc(sin(90deg) * 1px)", "calc(1px)"),
        ("calc(cos(0) * 1px)", "calc(1px)"),
        ("calc(tan(0turn) * 1px)", "calc(0px)"),
        ("calc(asin(1) / 90deg * 1px)", "calc(1px)"),
        ("calc(acos(1) / 1deg * 1px)", "calc(0px)"),
        ("calc(atan(infinity) / 90deg * 1px)", "calc(1px)"),
        ("calc(atan2(1, 0) / 90deg * 1px)", "calc(1px)"),
        ("calc(pow(2, 3) * 1px)", "calc(8px)"),
        ("calc(sqrt(4) * 1px)", "calc(2px)"),
        ("hypot(3px, 4px)", "calc(5px)"),
        ("calc(log(1, 2) * 1px)", "calc(0px)"),
        ("calc(exp(0) * 1px)", "calc(1px)"),
        ("abs(-2px)", "calc(2px)"),
        ("calc(sign(-2px) * 1px)", "calc(-1px)"),
        ("min(-2, 3)", "calc(-2)"),
        ("max(-2, 3)", "calc(3)"),
        ("hypot(3, 4)", "calc(5)"),
        ("sin(90deg)", "calc(1)"),
        ("calc(asin(1) / 90deg)", "calc(1)"),
        ("calc(0)", "calc(0)"),
        ("calc(0px)", "calc(0px)"),
        ("calc(infinity)", "calc(infinity)"),
        ("calc(-infinity)", "calc(-infinity)"),
        ("calc(NaN)", "calc(NaN)"),
        ("sqrt(-1)", "calc(NaN)"),
        ("log(0)", "calc(-infinity)"),
        ("calc(sqrt(-1) * 1px)", "calc(NaN * 1px)"),
        ("calc(log(0) * 1px)", "calc(-infinity * 1px)"),
    ] {
        accepted("tab-size", input, expected);
    }
}
#[test]
fn mandatory_word_base_exclusivity_and_duplicate_roles_reject_without_losing_color_siblings() {
    for text in [
        "",
        "auto-phrase",
        "none auto-phrase",
        "none space",
        "none ideographic-space",
        "space ideographic-space",
        "space space",
        "ideographic-space ideographic-space",
        "space auto-phrase auto-phrase",
        "auto-phrase auto-phrase space",
        "auto",
        "normal",
        "space,auto-phrase",
        "1",
        "1px",
        "25%",
        "calc(1)",
        "[space]",
        "\"space\"",
        "inherit space",
        "space unset",
    ] {
        invalid("word-space-transform", text);
    }
}
#[test]
fn tab_negative_literals_percentage_roots_and_dimensionally_invalid_math_reject_atomically() {
    for text in [
        "",
        "-1",
        "-1e-999",
        "-1px",
        "-1e-999px",
        "1%",
        "0%",
        "1s",
        "1fr",
        "none",
        "normal",
        "infinite",
        "8 4",
        "1px 2px",
        "8,4",
        "[8]",
        "\"8\"",
        "calc(1 + 1px)",
        "calc(0 + 1px)",
        "calc(1px + 1s)",
        "calc(1px + 1%)",
        "calc(1%)",
        "sin(1px)",
        "asin(1px)",
        "pow(1px, 2)",
        "round(1px, 0)",
        "calc-size(any, 1px)",
        "initial 8",
        "8 inherit",
    ] {
        invalid("tab-size", text);
    }
}
#[test]
fn decoded_keyword_case_escapes_and_dimension_units_keep_source_and_canonical_output() {
    accepted(
        "word-space-transform",
        r"\73 pace AUTO-PHRASE",
        "space auto-phrase",
    );
    accepted(
        "word-space-transform",
        "AUTO-PHRASE/**/IDEOGRAPHIC-SPACE",
        "ideographic-space auto-phrase",
    );
    accepted("tab-size", r"2\70 x", "2px");
    accepted("tab-size", "/**/CALC(3PX + 2EM)/**/", "calc(2em + 3px)");
    let report = parse_style_attribute(r"\74 ab-size:8!important");
    assert!(report.is_clean());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        property("tab-size")
    );
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "tab-size: 8 !important;"
    );
}
#[test]
fn programmatic_existing_component_construction_preserves_origins_and_importance() {
    let states = [
        (
            "word-space-transform",
            vec![CssComponentValue::try_ident("none").unwrap()],
            "none",
        ),
        (
            "word-space-transform",
            vec![
                CssComponentValue::try_ident("space").unwrap(),
                CssComponentValue::try_ident("auto-phrase").unwrap(),
            ],
            "space auto-phrase",
        ),
        (
            "tab-size",
            vec![CssComponentValue::try_number("0").unwrap()],
            "0",
        ),
        (
            "tab-size",
            vec![CssComponentValue::try_dimension("0", "px").unwrap()],
            "0px",
        ),
        (
            "tab-size",
            vec![CssComponentValue::try_number("8").unwrap()],
            "8",
        ),
    ];
    for (name, items, expected) in states {
        let components = CssComponentValues::try_new(items).unwrap();
        let before = components.clone();
        for grammar in [false, true] {
            let source = checked_components(name, components.clone(), grammar).unwrap();
            assert_eq!(source.value_components(), &components);
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|v| v.origin() == &CssValueOrigin::Programmatic)
            );
            assert_eq!(source.importance(), CssImportance::Important);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {expected} !important;")
            );
            assert!(source.position().is_none());
            completed(&source);
        }
        assert_eq!(components, before);
    }
}
#[test]
fn none_and_number_eight_initials_are_inherited_ordinary_longhands() {
    for name in FAMILY {
        let p = property(name);
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("one longhand, no reset members")
        };
        assert_eq!(metadata.property().known_property(), p);
        assert!(metadata.inherited_by_default());
        let initial = metadata.initial_value();
        assert_eq!(initial.property().known_property(), p);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary semantic initial")
        };
        let component = if name == "tab-size" {
            CssComponentValue::try_number("8").unwrap()
        } else {
            CssComponentValue::try_ident("none").unwrap()
        };
        let source = checked_components(
            name,
            CssComponentValues::try_new(vec![component]).unwrap(),
            false,
        )
        .unwrap();
        let ordinary = completed(&source);
        assert_eq!(ordinary.items()[0].ordinary_value(), Some(initial));
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: {} !important;", initial_text(name))
        );
    }
}
#[test]
fn all_css_wide_values_are_symbolic_single_terminals_preserving_occurrence_and_importance() {
    for name in FAMILY {
        for (text, keyword) in GLOBALS {
            for source in fronts(name, text) {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                assert!(source.known().unwrap().property_value().is_none());
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {text} !important;")
                );
                let values = completed(&source);
                let item = &values.items()[0];
                assert!(matches!(item.value(),CssContributionValueRef::Global(v) if v == keyword));
                assert!(item.replacement_components().is_none());
            }
        }
    }
}
#[test]
fn var_env_attr_reentry_admits_every_ordinary_state_and_keeps_original_and_replacement_snapshots() {
    for name in FAMILY {
        for pending in pending_inputs(name) {
            for source in fronts(name, pending) {
                let before = source.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("pending handle")
                };
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(handle.source().importance(), CssImportance::Important);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {pending} !important;")
                );
                for text in values(name) {
                    let replacement = parse_component_values(&text.to_ascii_uppercase()).unwrap();
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
                        assert_eq!(actual.source().as_str(), text.to_ascii_uppercase());
                    }
                    let ordinary = checked_components(name, replacement.clone(), true).unwrap();
                    assert_eq!(
                        ordinary.to_specified_css().unwrap(),
                        format!("{name}: {text} !important;")
                    );
                    let ordinary_values = completed(&ordinary);
                    assert_eq!(
                        item.ordinary_value(),
                        ordinary_values.items()[0].ordinary_value()
                    );
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
                    let item = &values.items()[0];
                    assert_source(item, &source);
                    assert!(
                        matches!(item.value(),CssContributionValueRef::Global(v) if v == keyword)
                    );
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
                assert_eq!(source, before);
            }
        }
    }
}
#[test]
fn invalid_or_residual_replacements_fail_atomically_and_the_same_handle_can_retry() {
    for name in FAMILY {
        for pending in pending_inputs(name) {
            let source = checked(name, pending, true);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            for text in [
                "var(--again)",
                "env(again)",
                "attr(data-again *)",
                "auto var(--again)",
                "calc(var(--again))",
                "calc(env(again))",
                "calc(attr(data-again *))",
                r"\76 ar(--again)",
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
            for text in [
                "auto",
                "8px 1",
                "initial auto",
                "8!important",
                "8;color:red",
            ] {
                for _ in 0..2 {
                    let error = handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("strict replacement")
                    };
                    assert!(matches!(
                        error.kind(),
                        CssPropertyValueErrorKind::Grammar(_)
                    ));
                }
            }
            let replacement = parse_component_values(initial_text(name)).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("successful reusable retry")
            };
            assert_eq!(values.items().len(), 1);
            assert_source(&values.items()[0], &source);
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
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
        .expect("public original implicit closing origin")
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
        panic!("implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}
#[test]
fn both_checked_fronts_reject_original_ordinary_global_pending_and_math_closures() {
    for name in FAMILY {
        let mut texts: Vec<String> = values(name)
            .iter()
            .map(|text| format!("{text}/*"))
            .collect();
        texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
        texts.extend(
            [
                "var(--wrap)/*",
                "env(wrap)/*",
                "attr(data-wrap *)/*",
                "var(--wrap",
                "env(wrap",
                "attr(data-wrap *",
            ]
            .map(str::to_owned),
        );
        if name == "tab-size" {
            texts.push("calc(1px + 2em".to_owned());
        }
        for text in texts {
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
    }
}
#[test]
fn replacement_closure_is_strict_and_residual_priority_and_complete_comment_controls_hold() {
    for name in FAMILY {
        for pending in pending_inputs(name) {
            let source = checked(name, pending, false);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let mut texts: Vec<String> = values(name)
                .iter()
                .map(|text| format!("{text}/*"))
                .collect();
            texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
            if name == "tab-size" {
                texts.push("calc(1px + 2em".to_owned());
            }
            for text in texts {
                let components = parse_component_values(&text).unwrap();
                let origin = implicit_origin(&components);
                for _ in 0..2 {
                    let error = handle.reenter(components.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("original replacement closure")
                    };
                    assert_closure(error, &origin, &text);
                }
            }
            for text in [
                "var(--again",
                "env(again",
                "attr(data-again *",
                "var(--again)/*",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            for text in [
                format!("{} /**/", initial_text(name)),
                "initial/**/".to_owned(),
                "var(--wrap)/**/".to_owned(),
                "env(wrap)/**/".to_owned(),
                "attr(data-wrap *)/**/".to_owned(),
            ] {
                checked(name, &text, false);
                checked(name, &text, true);
            }
            for text in values(name) {
                let closed = format!("{text}/**/");
                checked(name, &closed, false);
                checked(name, &closed, true);
                assert!(
                    handle
                        .reenter(parse_component_values(&closed).unwrap())
                        .is_ok()
                );
            }
            assert!(
                handle
                    .reenter(
                        parse_component_values(&format!("{} /**/", initial_text(name))).unwrap()
                    )
                    .is_ok()
            );
        }
    }
}
#[test]
fn checked_value_annotations_map_to_original_tokens_and_authored_importance_is_valid() {
    for name in FAMILY {
        let text = format!("/*😀*/{}!important", initial_text(name));
        let components = parse_component_values(&text).unwrap();
        let serialized = components.serialize().unwrap();
        let origin = serialized
            .origin_at(serialized.as_css().find('!').unwrap())
            .unwrap();
        for grammar in [false, true] {
            let error = checked_components(name, components.clone(), grammar).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            assert_eq!(error.origin(), origin);
        }
        for report in [
            parse_property_value_text(
                &text,
                CssPropertyNameRef::Known(property(name)),
                CssImportance::Important,
            ),
            parse_property_value_text_for_grammar(
                &text,
                property(name).grammar(),
                CssImportance::Important,
            ),
        ] {
            assert!(report.syntax().is_none());
            assert!(!report.is_clean());
        }
        accepted(name, initial_text(name), initial_text(name));
    }
}
#[test]
fn browser_eof_recovery_retains_diagnostics_and_normalized_original_occurrences() {
    for name in FAMILY {
        let mut texts: Vec<String> = [
            format!("{}/*", initial_text(name)),
            "var(--wrap".to_owned(),
            "env(wrap".to_owned(),
            "attr(data-wrap *".to_owned(),
        ]
        .to_vec();
        texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
        if name == "tab-size" {
            texts.push("calc(1px + 2em".to_owned());
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
                    .any(|v| v.action() == CssRecoveryAction::RetainWithImplicitClosure)
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
                panic!("retained recovered declaration")
            };
            assert_eq!(item.order(), 0);
            assert_eq!(item.source().known().unwrap().property(), property(name));
            assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), 1);
                    assert_source(&values.items()[0], item.source());
                }
                _ => panic!("retained intrinsic lifecycle"),
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
fn normalization_preserves_occurrence_order_contexts_and_exact_terminal_budgets() {
    let report = parse_sheet(
        "@media screen{.a{word-space-transform:space auto-phrase!important;tab-size:0;tab-size:0px;word-space-transform:var(--space);tab-size:unset!important;word-space-transform:none;tab-size:8;color:red}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 8, 8).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    let names = [
        "word-space-transform",
        "tab-size",
        "tab-size",
        "word-space-transform",
        "tab-size",
        "word-space-transform",
        "tab-size",
        "color",
    ];
    assert_eq!(declarations.len(), names.len());
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property().canonical_name(),
            names[index]
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 || index == 4 {
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
                assert_eq!(index, 3);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
                assert_eq!(
                    values.items()[0].property(),
                    item.source().known().unwrap().property()
                );
                if index == 4 {
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
            _ => panic!("single intrinsic terminal/handle"),
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
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
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
fn existing_scalar_and_composed_controls_execute_without_new_property_lookup() {
    for (input, expected) in [("0", "0"), ("-0", "0"), ("8", "8"), ("1e-999", "0")] {
        let number = CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number(input).unwrap(),
        )
        .unwrap();
        assert_eq!(number.serialize_specified().unwrap(), expected);
    }
    for input in ["-1", "-1e-999"] {
        assert!(
            CssSpecifiedNonNegativeNumber::try_from_component(
                CssComponentValue::try_number(input).unwrap()
            )
            .is_err()
        );
    }
    let length = CssSpecifiedNonNegativeLength::try_from_component(
        CssComponentValue::try_dimension("0", "px").unwrap(),
    )
    .unwrap();
    assert_eq!(length.serialize_specified().unwrap(), "0px");
    for (name, input, expected, count) in [
        (
            "flow-tolerance",
            "calc(-2px - 3%)",
            "flow-tolerance: calc(-3% - 2px);",
            1,
        ),
        (
            "flex-flow",
            "wrap row-reverse",
            "flex-flow: row-reverse wrap;",
            2,
        ),
        ("grid-template", "none", "grid-template: none;", 3),
        ("item-flow", "row", "item-flow: row auto normal normal;", 4),
        ("color", "red", "color: red;", 1),
    ] {
        let report = parse_style_attribute(&format!("{name}:{input}"));
        assert!(report.is_clean());
        for source in report.syntax().iter() {
            assert_eq!(source.to_specified_css().unwrap(), expected);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(source).unwrap()
            else {
                panic!("supported control")
            };
            assert_eq!(values.items().len(), count);
            for item in values.items() {
                assert_source(item, source);
            }
        }
    }
}

#[test]
fn canonical_declared_values_share_exact_literal_keyword_and_numeric_work_prices() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    // Existing shared math contract: calc(1px + 2em) child4input/5projection;
    // new transparent Tab delegates it, then declaration/name add2.
    for (name, input, canonical, input_nodes, projection_nodes) in [
        ("word-space-transform", "none", "none", 3, 3),
        ("word-space-transform", "space", "space", 3, 3),
        (
            "word-space-transform",
            "auto-phrase space",
            "space auto-phrase",
            4,
            4,
        ),
        (
            "word-space-transform",
            "auto-phrase ideographic-space",
            "ideographic-space auto-phrase",
            4,
            4,
        ),
        ("tab-size", "0", "0", 3, 3),
        ("tab-size", "0px", "0px", 3, 3),
        ("tab-size", "8", "8", 3, 3),
        ("tab-size", "calc(1px + 2em)", "calc(2em + 1px)", 6, 7),
        ("word-space-transform", "initial", "initial", 3, 3),
        ("tab-size", "revert-layer", "revert-layer", 3, 3),
    ] {
        for source in [parsed(name, input), checked(name, input, true)] {
            let before = source.clone();
            let expected = format!("{name}: {canonical} !important;");
            let exact = Limits::new(input_nodes, projection_nodes, expected.len());
            assert_eq!(
                source.to_specified_css_with_limits(exact).unwrap(),
                expected
            );
            for (limits, kind) in [
                (
                    Limits::new(input_nodes - 1, projection_nodes, expected.len()),
                    Kind::InputNodeLimit,
                ),
                (
                    Limits::new(input_nodes, projection_nodes - 1, expected.len()),
                    Kind::ProjectionNodeLimit,
                ),
                (
                    Limits::new(input_nodes, projection_nodes, expected.len() - 1),
                    Kind::ByteLimit,
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
    }
    let digits = format!("1{}", "0".repeat(999));
    for (input, output) in [
        ("1e999".to_owned(), digits.clone()),
        ("1e999px".to_owned(), format!("{digits}px")),
    ] {
        let source = checked("tab-size", &input, true);
        let expected = format!("tab-size: {output} !important;");
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(3, 3, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(3, 3, expected.len() - 1))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(source.to_specified_css().unwrap(), expected);
    }
}
#[test]
fn sheet_siblings_share_final_canonical_bytes_with_atomic_retry_at_the_public_sheet_boundary() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        ".a{word-space-transform:auto-phrase space!important}.b{tab-size:calc(1px + 2em)}",
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { word-space-transform: space auto-phrase !important; }\n.b { tab-size: calc(2em + 1px); }";
    // Whole-sheet graph work remains with its existing aggregate owners; only
    // independently determined sibling byte accounting is constrained here.
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
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
