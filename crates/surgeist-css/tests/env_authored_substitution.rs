#![forbid(unsafe_code)]
//! Env1/Variables1 family-qualified pending values retain authored syntax.
//! https://www.w3.org/TR/2025/WD-css-env-1-20250923/#env-function
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#syntax
use surgeist_css::*;

fn parsed(property: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{property}:{text}!important"));
    assert!(
        report.is_clean(),
        "{property}:{text}: {:?}",
        report.diagnostics()
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    declaration.clone()
}
fn checked(text: &str) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    declaration
}
fn pending_value(declaration: &CssDeclaration, text: &str) {
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::Display);
    assert!(known.property_value().is_none());
    assert!(known.global().is_none());
    assert_eq!(known.substitution_dependent().unwrap().as_css(), text);
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssExpansion::Pending(pending) = expand_declaration(declaration).unwrap() else {
        panic!("pending")
    };
    assert!(pending.source().same_occurrence(declaration));
}

#[test]
fn parsed_env_values_defer_known_property_grammar_without_lookup() {
    for text in [
        "env(unknown)",
        "ENV(safe-area-inset-top)",
        "env(auto)",
        "env(none)",
        "env(span)",
        "f([env(Future 0 1)])",
        "block env(unknown)",
    ] {
        let declaration = parsed("display", text);
        pending_value(&declaration, text);
        assert!(matches!(
            declaration.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
    }
}

#[test]
fn checked_env_indices_keep_exact_integer_and_authored_math_domains() {
    for text in [
        "env(viewport-segment-width 0 1)",
        "env(future -0 +0002)",
        "env(future 999999999999999999999999999999999999999)",
        "env(future calc(2.5))",
        "env(future calc(-1))",
        "env(future calc(infinity))",
        "env(future calc(NaN))",
        "env(future calc(1px / 1em))",
    ] {
        pending_value(&checked(text), text);
    }
}

#[test]
fn env_fallback_presence_preserves_whitespace_and_full_remaining_stream() {
    for text in [
        "env(foo)",
        "env(foo, )",
        "env(foo,/**/ )",
        "env(foo,,)",
        "env(foo, red, blue)",
        "env(foo, var(--x,))",
    ] {
        pending_value(&parsed("display", text), text);
        pending_value(&checked(text), text);
    }
    for text in ["env(foo,)", "env(foo,/**/)"] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "{text}"
        );
    }
}

#[test]
fn qualifying_env_family_defers_other_family_mismatches() {
    for text in ["var(env(foo))", "env(foo, var(x))", "env(foo) var(x)"] {
        pending_value(&parsed("display", text), text);
        pending_value(&checked(text), text);
    }
}

#[test]
fn qualifying_var_family_keeps_its_existing_whole_value_deferral() {
    for text in [
        "var(--x,)",
        "env(var(--name))",
        "env(foo var(--index))",
        "env(foo calc(var(--index)))",
        "env() var(--x)",
        "var(--x, env())",
        "var(--x, env(foo,))",
    ] {
        pending_value(&parsed("display", text), text);
        pending_value(&checked(text), text);
    }
}

#[test]
fn a_same_family_mismatch_cannot_be_rescued_by_its_inner_valid_occurrence() {
    for text in [
        "env(env(foo))",
        "env(foo env(bar))",
        "env(foo calc(env(bar)))",
        "env(foo, env())",
        "var(var(--name))",
        "var(--x) var(x)",
    ] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "{text}"
        );
        let report = parse_style_attribute(&format!("display:{text};display:block"));
        assert!(!report.is_clean(), "{text}");
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(report.syntax().len(), 1);
        assert!(
            report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .is_some()
        );
    }
}

#[test]
fn nonqualifying_env_headers_and_index_types_do_not_escape_display_grammar() {
    for text in [
        "env()",
        "env(initial)",
        "env(DEFAULT)",
        "env(revert-layer)",
        "env(foo -1)",
        "env(foo -999999999999999999999999999999999)",
        "env(foo 1.0)",
        "env(foo 1e0)",
        "env(foo 1px)",
        "env(foo 1%)",
        "env(foo calc(1px))",
        "env(foo calc(1%))",
    ] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "{text}"
        );
    }
}

#[test]
fn permissive_custom_and_style_values_retain_env_mismatches_as_raw_tokens() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for text in [
        "env()",
        "env(foo,)",
        "env(foo,/**/)",
        "env(var(--name))",
        "var(--ok,env())",
    ] {
        let components = parse_component_values(text).unwrap();
        let constructed = parse_property_value(
            CssPropertyNameRef::Custom(&name),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(constructed.value_components(), &components);
        for declaration in [parsed("--x", text), constructed] {
            assert!(declaration.known().is_none());
            assert_eq!(
                declaration
                    .custom()
                    .unwrap()
                    .value()
                    .value()
                    .unwrap()
                    .as_css(),
                text
            );
        }
        let condition = CssContainerCondition::try_from_components(
            parse_component_values(&format!("style(--x:{text})")).unwrap(),
        )
        .unwrap();
        let CssContainerConditionKind::Style(query) = condition.kind() else {
            panic!("style query")
        };
        let CssContainerStyleQueryKind::Feature(CssContainerStyleFeature::Plain { value, .. }) =
            query.kind()
        else {
            panic!("plain style value")
        };
        assert_eq!(value.serialize().unwrap().as_css(), text);
    }
}

#[test]
fn permissive_contexts_keep_the_existing_malformed_var_restriction() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for text in ["var(x)", "var(var(--name))", "env(foo,var(x))"] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Custom(&name),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "{text}"
        );
        assert!(
            !parse_style_attribute(&format!("--x:{text}")).is_clean(),
            "{text}"
        );
        // Conditional5 query-in-parens retains a failed style grammar as general-enclosed.
        let source = format!("style(--x:{text})");
        let condition =
            CssContainerCondition::try_from_components(parse_component_values(&source).unwrap())
                .unwrap();
        let CssContainerConditionKind::GeneralEnclosed(opaque) = condition.kind() else {
            panic!("malformed var must not become a typed Style: {text}")
        };
        assert_eq!(opaque.authored(), Some(source.as_str()));
    }
}

#[test]
fn env_in_replacement_is_residual_before_any_replacement_grammar_check() {
    let source = parsed("display", "var(--display)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("existing var pending")
    };
    for text in ["env(foo)", "env()", "f([ENV(foo,)])", "var(x)"] {
        assert_eq!(
            pending
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution,
            "{text}"
        );
    }
    let replacement = parse_component_values("block").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("longhands")
    };
    let [value] = values.items() else {
        panic!("one display")
    };
    assert_eq!(value.property(), CssKnownProperty::Display);
    assert!(value.source().same_occurrence(&source));
    assert_eq!(value.replacement_components(), Some(&replacement));
    assert_eq!(value.source().importance(), CssImportance::Important);
}

#[test]
fn shorthand_env_pending_reenters_with_one_original_occurrence() {
    let source = parsed("margin", "env(future, 1px 2px)");
    assert_eq!(
        source
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        "env(future, 1px 2px)"
    );
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending margin")
    };
    assert!(pending.source().same_occurrence(&source));
    let replacement = parse_component_values("1px 2px").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("margin longhands")
    };
    let expected = [
        CssKnownProperty::MarginTop,
        CssKnownProperty::MarginRight,
        CssKnownProperty::MarginBottom,
        CssKnownProperty::MarginLeft,
    ];
    assert_eq!(values.items().len(), expected.len());
    for (value, property) in values.items().iter().zip(expected) {
        assert_eq!(value.property(), property);
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&replacement));
        assert!(value.ordinary_value().is_some());
    }
}
