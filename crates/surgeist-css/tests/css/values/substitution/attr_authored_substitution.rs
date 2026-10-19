#![forbid(unsafe_code)]
//! Authored `attr()` substitution follows the pinned CSS Values 5 grammar.
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#funcdef-attr
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#component-function-commas
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#typedef-syntax

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

fn checked(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(property),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    declaration
}

fn assert_pending(declaration: &CssDeclaration, property: CssKnownProperty, text: &str) {
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), property);
    assert!(known.property_value().is_none());
    assert!(known.global().is_none());
    assert_eq!(known.substitution_dependent().unwrap().as_css(), text);
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssExpansion::Pending(pending) = expand_declaration(declaration).unwrap() else {
        panic!("whole declaration remains pending")
    };
    assert!(pending.source().same_occurrence(declaration));
}

#[test]
fn valid_attr_syntax_defers_entire_known_value_at_both_entry_points() {
    // One CSS string containing another CSS string containing <length>.
    let recursively_quoted = format!("attr(foo {:?})", format!("{:?}", "<length>"));
    let values = [
        "attr(foo)".to_owned(),
        "attr(ns|foo)".to_owned(),
        "a\\74 tr(foo)".to_owned(),
        "attr(foo *)".to_owned(),
        "attr(foo <length>)".to_owned(),
        "attr(foo <number> | auto)".to_owned(),
        "attr(foo auto)".to_owned(),
        "attr(foo <transform-list>)".to_owned(),
        "attr(foo <length>+)".to_owned(),
        "attr(foo <length>#)".to_owned(),
        "attr(foo \"<length>\")".to_owned(),
        recursively_quoted,
        "attr(foo, initial)".to_owned(),
        "attr(foo, {a,b})".to_owned(),
        "attr(foo, )".to_owned(),
        "block attr(foo)".to_owned(),
        "f([attr(foo)])".to_owned(),
    ];
    for text in values {
        for declaration in [
            parsed("display", &text),
            checked(CssKnownProperty::Display, &text),
        ] {
            assert_pending(&declaration, CssKnownProperty::Display, &text);
        }
    }
}

#[test]
fn malformed_attr_does_not_defer_or_survive_recovery() {
    for text in [
        "attr()",
        "attr(foo,)",
        "attr(foo,/**/)",
        "attr(foo, {})",
        "attr(foo, a,b)",
        "attr(foo, a {})",
        "attr(|)",
        "attr(ns|)",
        "attr(foo <unknown>)",
        "attr(foo <number px>)",
        "attr(foo < length>)",
        "attr(foo <length >)",
        "attr(foo <length> +)",
        "attr(foo <transform-list>+)",
        "attr(foo) attr()",
        "attr(foo, attr())",
    ] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(text).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "{text}"
        );
        let report = parse_style_attribute(&format!("display:{text};display:block"));
        assert!(!report.is_clean(), "{text}");
        assert_eq!(report.diagnostics().len(), 1, "{text}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        let [sibling] = report.syntax().as_slice() else {
            panic!("valid sibling survives: {text}")
        };
        assert!(sibling.known().unwrap().property_value().is_some());
    }
}

#[test]
fn independent_valid_variable_or_environment_can_defer_malformed_attr() {
    for text in [
        "var(--x) attr()",
        "env(foo) attr(foo,)",
        "attr(foo, var(--x)) attr()",
    ] {
        for declaration in [
            parsed("display", text),
            checked(CssKnownProperty::Display, text),
        ] {
            assert_pending(&declaration, CssKnownProperty::Display, text);
        }
    }
}

#[test]
fn custom_property_tokens_keep_their_existing_permissive_policy() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for text in ["attr()", "attr(foo,)", "attr(foo <number px>)"] {
        let components = parse_component_values(text).unwrap();
        let declaration = parse_property_value(
            CssPropertyNameRef::Custom(&name),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(declaration.value_components(), &components);
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
}

#[test]
fn attr_remains_residual_at_any_depth_before_replacement_grammar() {
    let source = parsed("display", "var(--display)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("source pending")
    };
    for text in ["attr(foo)", "attr()", "f([ATTR(foo,)])", "block attr(foo)"] {
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
        panic!("replacement completes")
    };
    let [value] = values.items() else {
        panic!("one contribution")
    };
    assert_eq!(value.property(), CssKnownProperty::Display);
    assert!(value.source().same_occurrence(&source));
    assert_eq!(value.replacement_components(), Some(&replacement));
}

#[test]
fn shorthand_attr_reentry_preserves_original_occurrence_and_importance() {
    let source = parsed("margin-block", "attr(data-size <length>)");
    assert_pending(
        &source,
        CssKnownProperty::MarginBlock,
        "attr(data-size <length>)",
    );
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending shorthand")
    };
    let replacement = parse_component_values("1px 2px").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("two margin longhands")
    };
    assert_eq!(values.items().len(), 2);
    for (value, property) in values.items().iter().zip([
        CssKnownProperty::MarginBlockStart,
        CssKnownProperty::MarginBlockEnd,
    ]) {
        assert_eq!(value.property(), property);
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&replacement));
    }
}

#[test]
fn checked_attr_function_keeps_mixed_component_origins_without_combining_tokens() {
    let parsed_name = parse_component_values("foo").unwrap().items()[0].clone();
    let keyword_syntax = CssComponentValue::try_ident("bar").unwrap();
    let arguments =
        CssComponentValues::try_new(vec![parsed_name.clone(), keyword_syntax.clone()]).unwrap();
    let function = CssComponentValue::try_function("attr", arguments).unwrap();
    assert!(matches!(function.origin(), CssValueOrigin::Programmatic));
    let components = CssComponentValues::try_new(vec![function]).unwrap();
    assert_eq!(components.serialize().unwrap().as_css(), "attr(foo/**/bar)");
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    assert!(
        declaration
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    assert!(matches!(
        declaration.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let CssComponentValueRef::Function(function) = declaration.value_components().items()[0].view()
    else {
        panic!("authored attr function")
    };
    assert_eq!(function.values().items(), &[parsed_name, keyword_syntax]);
    assert!(matches!(
        function.values().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        function.values().items()[1].origin(),
        &CssValueOrigin::Programmatic
    );
}

#[test]
fn pending_attr_normalization_retains_recovered_order_and_importance() {
    let source = ".a{display:attr(first);display:attr(foo,);display:attr(second)!important}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(source).is_err());
    let sheet = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap(),
    )
    .unwrap();
    let declarations: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    for (index, (declaration, text)) in declarations
        .iter()
        .zip(["attr(first)", "attr(second)"])
        .enumerate()
    {
        assert_eq!(declaration.order(), index);
        let CssExpansion::Pending(pending) = declaration.expansion() else {
            panic!("pending contribution")
        };
        assert!(pending.source().same_occurrence(declaration.source()));
        assert_eq!(
            pending
                .source()
                .known()
                .unwrap()
                .substitution_dependent()
                .unwrap()
                .as_css(),
            text
        );
        assert_eq!(
            pending.source().importance(),
            if index == 0 {
                CssImportance::Normal
            } else {
                CssImportance::Important
            }
        );
    }
}

#[test]
fn property_attr_deferral_does_not_admit_query_or_descriptor_operands() {
    let query = CssContainerCondition::try_from_components(
        parse_component_values("(width:attr(foo))").unwrap(),
    )
    .unwrap();
    let CssContainerConditionKind::GeneralEnclosed(opaque) = query.kind() else {
        panic!("width query retains its own grammar")
    };
    assert_eq!(opaque.authored(), Some("(width:attr(foo))"));

    let descriptor =
        parse_font_face_descriptor_value("attr(foo)", CssFontFaceDescriptorKind::FontStyle);
    assert!(descriptor.syntax().is_none());
    assert!(!descriptor.is_clean());
}

#[test]
fn ordinary_and_existing_variable_values_remain_controls() {
    assert!(
        parsed("display", "block")
            .known()
            .unwrap()
            .property_value()
            .is_some()
    );
    assert_pending(
        &parsed("display", "var(--display)"),
        CssKnownProperty::Display,
        "var(--display)",
    );
    assert_pending(
        &parsed("display", "env(foo)"),
        CssKnownProperty::Display,
        "env(foo)",
    );
}
