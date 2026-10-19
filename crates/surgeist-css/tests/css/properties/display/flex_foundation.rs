#![forbid(unsafe_code)]

//! Flexbox 1 §7.1–7.2 defines three noninherited flex terminals; Sizing 3/4
//! supplies the referenced width grammar. Values 5 §10 and the pinned WPT
//! calc-size parsing cases make `content` a flex-basis-only calc-size basis.

use surgeist_css::*;

const MEMBERS: [&str; 3] = ["flex-grow", "flex-shrink", "flex-basis"];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("missing grammar: {name}"))
}

fn direct(name: &str, value: &str) -> Result<CssDeclaration, CssPropertyValueParseError> {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Normal,
    )
}

fn declaration(value: &str) -> CssDeclaration {
    let source = format!("flex:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one flex declaration: {source}")
    };
    declaration.clone()
}

fn expanded(value: &str) -> Vec<CssLonghandContribution> {
    let source = declaration(value);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("flex:{value} expands to three longhands")
    };
    assert_eq!(values.items().len(), 3, "flex:{value}");
    for (item, name) in values.items().iter().zip(MEMBERS) {
        assert_eq!(item.property(), grammar(name).target_property());
        assert!(item.source().same_occurrence(&source), "{name}");
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
    values.items().to_vec()
}

#[test]
fn flex_metadata_has_three_noninherited_terminals_with_intrinsic_initials() {
    let CssPropertyKindRef::Shorthand(metadata) = grammar("flex").metadata().unwrap().kind() else {
        panic!("flex shorthand metadata")
    };
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|property| property.known_property().canonical_name())
            .collect::<Vec<_>>(),
        MEMBERS
    );
    assert!(metadata.reset_only_members().is_empty());
    for name in MEMBERS {
        let CssPropertyKindRef::Longhand(meta) = grammar(name).metadata().unwrap().kind() else {
            panic!("{name} longhand metadata")
        };
        assert!(!meta.inherited_by_default(), "{name}");
        let initial_metadata = meta.initial_value();
        let CssInitialValueRef::Value(initial_value) = initial_metadata.view() else {
            panic!("{name} ordinary initial")
        };
        assert_eq!(
            initial_value.property().known_property(),
            grammar(name).target_property()
        );
    }
}

#[test]
fn flex_basis_accepts_width_values_plus_content_and_contextual_calc_size() {
    for value in [
        "auto",
        "content",
        "0",
        "25%",
        "10px",
        "stretch",
        "contain",
        "min-content",
        "max-content",
        "fit-content",
        "fit-content(12px)",
        "calc-size(min-content, size + 1px)",
        "calc-size(content, size)",
        "calc-size(calc-size(content, size), size + 1px)",
    ] {
        assert!(direct("flex-basis", value).is_ok(), "flex-basis:{value}");
        let report = parse_style_attribute(&format!("flex-basis:{value}"));
        assert!(
            report.is_clean(),
            "flex-basis:{value}: {:?}",
            report.diagnostics()
        );
    }
    for name in [
        "width",
        "height",
        "min-width",
        "min-height",
        "max-width",
        "max-height",
    ] {
        for value in [
            "content",
            "calc-size(content, size)",
            "calc-size(calc-size(content, size), size + 1px)",
        ] {
            assert!(direct(name, value).is_err(), "{name}:{value}");
        }
    }
    for value in ["none", "-1px", "-25%"] {
        assert!(direct("flex-basis", value).is_err(), "flex-basis:{value}");
    }
}

#[test]
fn flex_factors_admit_nonnegative_numbers_and_symbolic_number_math() {
    for name in ["flex-grow", "flex-shrink"] {
        for value in [
            "0",
            "1",
            "1.5",
            "16777217",
            "1e100",
            "1e-100",
            "calc(1 + 2)",
            "min(1, 2)",
        ] {
            assert!(direct(name, value).is_ok(), "{name}:{value}");
        }
        for value in ["-1", "-1e-100", "1%", "1px", "auto"] {
            assert!(direct(name, value).is_err(), "{name}:{value}");
        }
    }
}

#[test]
fn flex_factors_preserve_exact_literals_at_every_magnitude() {
    for name in ["flex-grow", "flex-shrink"] {
        for text in ["1", "16777217", "1e-100"] {
            let declaration = direct(name, text).unwrap();
            let factor = match declaration.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::FlexGrow(value) => value.factor(),
                CssKnownPropertyValueRef::FlexShrink(value) => value.factor(),
                _ => panic!("factor wrapper"),
            };
            let component = factor.literal_component().expect("ordinary factor");
            assert!(matches!(component.origin(), CssValueOrigin::Parsed(_)));
            assert!(
                matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation() == text)
            );
        }
    }
}

#[test]
fn flex_shorthand_accepts_each_grammar_form_and_rejects_reordered_factors() {
    for value in [
        "none",
        "auto",
        "initial",
        "1",
        "0",
        "0px",
        "content",
        "25%",
        "2 3",
        "2 3 10px",
        "10px 2 3",
        "10px 2",
        "2 10px",
        "1 auto",
        "auto 1 2",
        "1 2 0",
        "calc(1 + 2) 0 content",
        "calc-size(content, size) 2 3",
    ] {
        let source = format!("flex:{value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(direct("flex", value).is_ok(), "{source}");
    }
    for value in [
        "2 10px 3",
        "2 content 3",
        "0 1 2",
        "-1 1 0px",
        "1 2 3",
        "none 1",
        "1 -1 0px",
    ] {
        let source = format!("color:red;flex:{value};color:blue");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert!(direct("flex", value).is_err(), "flex:{value}");
    }
}

#[test]
fn flex_expansion_uses_specified_zero_and_keyword_presets() {
    for value in [
        "1", "0", "0px", "content", "10px 2 3", "2 3 10px", "none", "auto",
    ] {
        let items = expanded(value);
        assert!(items.iter().all(|item| item.ordinary_value().is_some()));
    }
    // Flexbox 1 §7.1 specifies a unitless 0 for an omitted basis. Compare it
    // to an independently parsed longhand, not to the UA's `0%` shortcut.
    let items = expanded("1");
    let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
        expand_declaration(&direct("flex-basis", "0").unwrap()).unwrap()
    else {
        panic!("direct basis zero expands")
    };
    assert_eq!(
        items[2].ordinary_value(),
        expected.items()[0].ordinary_value()
    );
    let items = expanded("none");
    let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
        expand_declaration(&direct("flex-basis", "auto").unwrap()).unwrap()
    else {
        panic!("direct basis auto expands")
    };
    assert_eq!(
        items[2].ordinary_value(),
        expected.items()[0].ordinary_value()
    );
}

#[test]
fn flex_css_wide_and_pending_values_cover_all_three_terminals() {
    let source = declaration("inherit");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("inherit flex expands")
    };
    assert_eq!(values.items().len(), 3);
    for (item, name) in values.items().iter().zip(MEMBERS) {
        assert_eq!(item.property(), grammar(name).target_property());
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
        );
        assert!(item.source().same_occurrence(&source));
    }

    let pending_source = declaration("var(--flex)");
    let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
        panic!("flex substitution remains pending")
    };
    assert!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .is_err()
    );
    assert!(
        pending
            .reenter(parse_component_values("2 content 3").unwrap())
            .is_err()
    );
    let CssContributions::Longhands(values) = pending
        .reenter(parse_component_values("content 2 3").unwrap())
        .unwrap()
    else {
        panic!("valid flex substitution expands")
    };
    assert_eq!(values.items().len(), 3);
    for item in values.items() {
        assert!(item.source().same_occurrence(&pending_source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_some());
    }
}
