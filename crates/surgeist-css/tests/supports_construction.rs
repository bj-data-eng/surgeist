#![forbid(unsafe_code)]
//! Checked supports retains authored tokens and their original provenance while
//! applying supports grammar independently of property support or evaluation.
use surgeist_css::{
    CssBlockKind, CssCalculationExpressionRef, CssComponentValue, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssGlobalKeyword,
    CssImportance, CssKnownPropertyValueRef, CssLength, CssNamespaceConstraint,
    CssNamespaceContext, CssOpacityValue, CssRule, CssSelector, CssSerializedOrigin,
    CssSupportsCondition, CssSupportsConditionKind, CssSupportsConstructionError,
    CssSupportsDeclaration, CssValueOrigin, CssValueTokenRef, parse_component_values, parse_sheet,
};

fn values(items: Vec<CssComponentValue>) -> CssComponentValues {
    CssComponentValues::try_new(items).unwrap()
}

fn token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).unwrap()
}

fn condition(source: &str) -> CssSupportsCondition {
    CssSupportsCondition::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

fn declaration(source: &str) -> CssSupportsDeclaration {
    CssSupportsDeclaration::try_from_components(parse_component_values(source).unwrap()).unwrap()
}

fn same_parsed_origin(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected)
    else {
        panic!("original parsed token origins")
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}

#[test]
fn programmatic_width_keeps_exact_spelling_and_has_no_source_coordinates() {
    let original = values(vec![
        CssComponentValue::try_ident("width").unwrap(),
        token(":"),
        CssComponentValue::try_dimension("+001.5", "PX").unwrap(),
    ]);
    let declaration = CssSupportsDeclaration::try_from_components(original.clone()).unwrap();
    assert_eq!(declaration.components(), original.items());
    assert_eq!(declaration.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(declaration.position(), None);
    assert_eq!(declaration.authored(), None);
    assert_eq!(declaration.property(), "width");
    assert_eq!(declaration.importance(), CssImportance::Normal);
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        declaration.value_components()[0].view()
    else {
        panic!("exact dimension")
    };
    assert_eq!(number.representation(), "+001.5");
    assert_eq!(unit, "PX");
    let CssKnownPropertyValueRef::Width(width) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed width")
    };
    assert!(matches!(width.i01_subset(), Some(CssLength::Px(value)) if value.value() == 1.5));
    assert_eq!(declaration.serialize().unwrap().as_css(), "width:+001.5PX");
    let wrapped = CssComponentValue::try_block(CssBlockKind::Parenthesis, original).unwrap();
    let condition = CssSupportsCondition::try_from_components(
        values(vec![wrapped]),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(condition.position(), None);
    assert_eq!(condition.serialize().unwrap().as_css(), "(width:+001.5PX)");
}

#[test]
fn mixed_source_declaration_preserves_each_original_snapshot_inside_a_new_wrapper() {
    let property = parse_component_values("/* 😀 */\nwidth")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let numeric = parse_component_values("/* λ */\r\n+001.5PX")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let property_origin = property.origin().clone();
    let numeric_origin = numeric.origin().clone();
    let original = values(vec![property, token(":"), numeric]);
    let wrapped = CssComponentValue::try_block(CssBlockKind::Parenthesis, original).unwrap();
    let condition = CssSupportsCondition::try_from_components(
        values(vec![wrapped]),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(condition.position(), None);
    let CssSupportsConditionKind::Declaration(declaration) = condition.kind() else {
        panic!("declaration")
    };
    same_parsed_origin(declaration.property_component().origin(), &property_origin);
    same_parsed_origin(declaration.value_components()[0].origin(), &numeric_origin);
    assert_eq!(declaration.authored(), None);
    let output = condition.serialize().unwrap();
    assert_eq!(output.as_css(), "(width:+001.5PX)");
    for (offset, expected) in [(1, &property_origin), (7, &numeric_origin)] {
        let Some(CssSerializedOrigin::Token(actual)) = output.origin_at(offset) else {
            panic!("original output token")
        };
        same_parsed_origin(actual, expected);
    }
}

#[test]
fn construction_from_parsed_components_does_not_claim_an_authored_declaration_slice() {
    let source = "WIDth/**/: +001.5PX";
    let constructed = declaration(source);
    assert_eq!(constructed.authored(), None);
    assert_eq!(constructed.serialize().unwrap().as_css(), source);
    let sheet = parse_sheet(&format!("/*😀*/\n@supports ({source}) {{}}"));
    assert!(sheet.is_clean(), "{sheet:?}");
    let CssRule::Supports(rule) = &sheet.syntax().rules()[0] else {
        panic!("supports")
    };
    let CssSupportsConditionKind::Declaration(parsed) = rule.condition().kind() else {
        panic!("declaration")
    };
    assert_eq!(parsed.authored(), Some(source));
    assert_eq!(parsed.serialize().unwrap().as_css(), source);
    assert!(parsed.position().is_some());
}

#[test]
fn identical_numeric_spellings_from_distinct_snapshots_keep_distinct_leaf_identity() {
    let left = parse_component_values("1").unwrap().items()[0].clone();
    let right = parse_component_values("1").unwrap().items()[0].clone();
    let left_origin = left.origin().clone();
    let right_origin = right.origin().clone();
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (&left_origin, &right_origin)
    else {
        panic!("parsed leaves")
    };
    assert!(!a.source().same_snapshot(b.source()));
    let calc = CssComponentValue::try_function(
        "calc",
        values(vec![left, token(" "), token("+"), token(" "), right]),
    )
    .unwrap();
    let declaration = CssSupportsDeclaration::try_from_components(values(vec![
        token("opacity"),
        token(":"),
        calc,
    ]))
    .unwrap();
    let CssKnownPropertyValueRef::Opacity(opacity) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("opacity")
    };
    let CssOpacityValue::Calculation(calculation) = opacity.value() else {
        panic!("calculation")
    };
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("calc")
    };
    let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
        panic!("sum")
    };
    for (index, expected) in [(0, &left_origin), (1, &right_origin)] {
        let CssCalculationExpressionRef::Value(value) = sum.term(index).unwrap().expression()
        else {
            panic!("literal")
        };
        assert_eq!(value.literal().representation(), "1");
        same_parsed_origin(value.literal().origin(), expected);
    }
    assert_eq!(
        sum.term(1).unwrap().operator_origin(),
        Some(&CssValueOrigin::Programmatic)
    );
    assert_eq!(
        declaration.serialize().unwrap().as_css(),
        "opacity:calc(1 + 1)"
    );
}

#[test]
fn supports_declaration_validity_is_independent_of_known_property_value_support() {
    for source in ["width:", "future:yes", "width:red", "--x:"] {
        let declaration = declaration(source);
        assert!(declaration.known().is_none(), "{source}");
        assert_eq!(declaration.serialize().unwrap().as_css(), source);
        assert!(matches!(
            condition(&format!("({source})")).kind(),
            CssSupportsConditionKind::Declaration(_)
        ));
    }
    assert_eq!(
        declaration("width:inherit").known().unwrap().global(),
        Some(CssGlobalKeyword::Inherit)
    );
    assert!(
        declaration("width:var(--size)")
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
}

#[test]
fn terminal_importance_is_excluded_from_the_value_but_preserved_in_serialization() {
    let source = "width:1px !/**/ImPoRtAnT/**/";
    let declaration = declaration(source);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.serialize().unwrap().as_css(), source);
    let output = values(declaration.value_components().to_vec())
        .serialize()
        .unwrap();
    assert_eq!(output.as_css(), "1px ");
    for source in [
        "width:1px!important extra",
        "width:1px;",
        "width 1px",
        "width:!",
        "",
    ] {
        assert!(
            matches!(
                CssSupportsDeclaration::try_from_components(
                    parse_component_values(source).unwrap()
                ),
                Err(CssSupportsConstructionError::InvalidDeclarationGrammar { .. })
            ),
            "{source:?}"
        );
    }
}

#[test]
fn malformed_root_operators_are_rejected_while_general_enclosed_remains_opaque() {
    for source in [
        "width:1px",
        "(width:1px) and",
        "not not (width:1px)",
        "(width:1px) and (height:1px) or (color:red)",
    ] {
        assert!(
            matches!(
                CssSupportsCondition::try_from_components(
                    parse_component_values(source).unwrap(),
                    &CssNamespaceContext::default()
                ),
                Err(CssSupportsConstructionError::InvalidConditionGrammar { .. })
            ),
            "{source}"
        );
    }
    for source in ["(future stuff)", "selector(svg|a)", "selector(a,b)"] {
        assert!(
            matches!(
                condition(source).kind(),
                CssSupportsConditionKind::GeneralEnclosed(_)
            ),
            "{source}"
        );
    }
    let function =
        CssComponentValue::try_function("Future", values(vec![token("1"), token("e2")])).unwrap();
    let condition = CssSupportsCondition::try_from_components(
        values(vec![function.clone()]),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let CssSupportsConditionKind::GeneralEnclosed(opaque) = condition.kind() else {
        panic!("opaque")
    };
    assert_eq!(opaque.component(), &function);
    assert_eq!(condition.serialize().unwrap().as_css(), "Future(1/**/e2)");
}

#[test]
fn supplied_namespace_context_selects_the_typed_selector_without_resolving_its_constraint() {
    let sheet = parse_sheet("@namespace svg 'urn:svg'; @namespace 'urn:default';");
    assert!(sheet.is_clean());
    let context = CssNamespaceContext::from_sheet(sheet.syntax());
    for (source, named) in [("selector(svg|a)", true), ("selector(a)", false)] {
        let condition = CssSupportsCondition::try_from_components(
            parse_component_values(source).unwrap(),
            &context,
        )
        .unwrap();
        let CssSupportsConditionKind::Selector(CssSelector::Compound(selector)) = condition.kind()
        else {
            panic!("typed namespace selector")
        };
        let constraint = selector.type_selector().unwrap().namespace();
        if named {
            assert!(
                matches!(constraint, CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg")
            );
        } else {
            assert_eq!(constraint, &CssNamespaceConstraint::Default);
        }
        assert_eq!(condition.serialize().unwrap().as_css(), source);
    }
}

#[test]
fn grouping_and_cloned_interior_children_serialize_independently_of_unrelated_siblings() {
    let child = values(vec![
        CssComponentValue::try_block(
            CssBlockKind::Parenthesis,
            values(vec![token("width"), token(":"), token("1px")]),
        )
        .unwrap(),
    ]);
    let standalone =
        CssSupportsCondition::try_from_components(child.clone(), &CssNamespaceContext::default())
            .unwrap();
    let mut siblings = child.items().to_vec();
    siblings.extend([
        token(" "),
        token("and"),
        token(" "),
        CssComponentValue::try_function("future", values(vec![])).unwrap(),
    ]);
    let parent = CssSupportsCondition::try_from_components(
        values(siblings),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    let CssSupportsConditionKind::And(list) = parent.kind() else {
        panic!("and")
    };
    let cloned = list.conditions()[0].clone();
    assert_eq!(cloned, standalone);
    drop(parent);
    assert_eq!(cloned.serialize().unwrap().as_css(), "(width:1px)");
    for source in [
        "(((width:1px)))",
        "not ((width:1px) or (height:2px))",
        "((width:1px) and (height:2px)) or future()",
    ] {
        assert_eq!(condition(source).serialize().unwrap().as_css(), source);
    }
}

#[test]
fn recovery_and_explicit_resource_limits_report_the_original_component_origin() {
    let recovered = parse_component_values("Future(").unwrap();
    let original_origin = recovered.items()[0].origin().clone();
    let error =
        CssSupportsCondition::try_from_components(recovered, &CssNamespaceContext::default())
            .unwrap_err();
    assert!(matches!(
        error,
        CssSupportsConstructionError::RecoveredInput { .. }
    ));
    let CssValueOrigin::ImplicitClosure { opening, at } = error.origin() else {
        panic!("honest EOF recovery origin")
    };
    let CssValueOrigin::Parsed(expected) = &original_origin else {
        panic!("parsed opener")
    };
    assert!(opening.source().same_snapshot(expected.source()));
    assert_eq!(opening.span(), expected.span());
    assert!(at.source().same_snapshot(expected.source()));
    assert_eq!(at.span().start(), at.span().end());
    assert_eq!(at.span().start().byte_offset().value(), "Future(".len());
    assert!(matches!(
        CssSupportsDeclaration::try_from_components(
            parse_component_values("width:calc(1px").unwrap()
        ),
        Err(CssSupportsConstructionError::RecoveredInput { .. })
    ));
    let components = parse_component_values("future()").unwrap();
    let expected_origin = components.items()[0].origin().clone();
    let CssComponentValueRef::Function(function) = components.items()[0].view() else {
        panic!("function")
    };
    let closing_origin = function.closing_origin().clone();
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 7).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error = CssSupportsCondition::try_from_components_with_limits(
            components.clone(),
            &CssNamespaceContext::default(),
            limits,
        )
        .unwrap_err();
        let CssSupportsConstructionError::Component(error) = error else {
            panic!("component limit")
        };
        assert_eq!(error.kind(), expected);
        // Seven bytes admit `future(`; the eighth byte is its original closer.
        let origin = if expected == CssComponentValueErrorKind::ByteLimit {
            &closing_origin
        } else {
            &expected_origin
        };
        same_parsed_origin(error.origin(), origin);
    }
    let exact = CssComponentValueLimits::try_new(1, 1, 8).unwrap();
    assert_eq!(
        CssSupportsCondition::try_from_components_with_limits(
            components,
            &CssNamespaceContext::default(),
            exact
        )
        .unwrap()
        .serialize()
        .unwrap()
        .as_css(),
        "future()"
    );
}

#[test]
fn grammar_error_after_programmatic_tokens_identifies_the_original_parsed_offender() {
    let offender = parse_component_values("/*😀*/\nextra")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let origin = offender.origin().clone();
    let error = CssSupportsCondition::try_from_components(
        values(vec![
            CssComponentValue::try_function("future", values(vec![])).unwrap(),
            token(" "),
            offender,
        ]),
        &CssNamespaceContext::default(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CssSupportsConstructionError::InvalidConditionGrammar { .. }
    ));
    same_parsed_origin(error.origin(), &origin);
}

#[test]
fn nested_math_preserves_function_and_literal_origins_from_the_original_value_tree() {
    let parsed = parse_component_values("/*😀*/\ncalc(calc(1e99999 / 2))").unwrap();
    let outer = parsed.items().last().unwrap().clone();
    let CssComponentValueRef::Function(function) = outer.view() else {
        panic!("outer calc")
    };
    let inner = &function.values().items()[0];
    let CssComponentValueRef::Function(function) = inner.view() else {
        panic!("inner calc")
    };
    let leaf = &function.values().items()[0];
    let origins = [
        outer.origin().clone(),
        inner.origin().clone(),
        leaf.origin().clone(),
    ];
    let declaration = CssSupportsDeclaration::try_from_components(values(vec![
        token("opacity"),
        token(":"),
        outer,
    ]))
    .unwrap();
    let CssKnownPropertyValueRef::Opacity(opacity) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("opacity")
    };
    let CssOpacityValue::Calculation(calculation) = opacity.value() else {
        panic!("calculation")
    };
    let mut expression = calculation.expression();
    for origin in &origins[..2] {
        same_parsed_origin(expression.origin(), origin);
        let CssCalculationExpressionRef::NestedCalc(inner) = expression else {
            panic!("retained calc")
        };
        expression = inner.operand();
    }
    let CssCalculationExpressionRef::Product(product) = expression else {
        panic!("division")
    };
    let CssCalculationExpressionRef::Value(value) = product.factor(0).unwrap().expression() else {
        panic!("exact leaf")
    };
    assert_eq!(value.literal().representation(), "1e99999");
    same_parsed_origin(value.literal().origin(), &origins[2]);
}

#[test]
fn declaration_limits_count_original_components_and_exact_lexical_output() {
    let components = values(vec![token("width"), token(":"), token("1px")]);
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(256, 2, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 8).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error =
            CssSupportsDeclaration::try_from_components_with_limits(components.clone(), limits)
                .unwrap_err();
        let CssSupportsConstructionError::Component(error) = error else {
            panic!("component limit")
        };
        assert_eq!(error.kind(), expected);
        assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    }
    let exact = CssComponentValueLimits::try_new(0, 3, 9).unwrap();
    assert_eq!(
        CssSupportsDeclaration::try_from_components_with_limits(components, exact)
            .unwrap()
            .serialize()
            .unwrap()
            .as_css(),
        "width:1px"
    );
}

#[test]
fn parsed_eof_recovery_keeps_the_known_numeric_view_that_checked_construction_rejects() {
    use surgeist_css::{CssCalcLength, CssRecoveryAction};
    // Ordinary parsing retains EOF closure and uses recovered-syntax numeric
    // admission, as it does for an ordinary declaration's calc(1px at EOF.
    // Import's bare supports declaration is retained without a rule block;
    // a blockless @supports rule would be rejected independently of recovery.
    let source = "/*😀*/@import 'x' supports(width:calc(1px";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    let CssRule::Import(rule) = &report.syntax().rules()[0] else {
        panic!("retained import rule")
    };
    let condition = rule.supports().unwrap().condition();
    let CssSupportsConditionKind::Declaration(declaration) = condition.kind() else {
        panic!("retained declaration")
    };
    let CssKnownPropertyValueRef::Width(width) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("known width remains available")
    };
    let CssLength::Calc(CssCalcLength::Typed(calculation)) = width.i01_subset().unwrap() else {
        panic!("exact recovered calculation")
    };
    assert_eq!(calculation.serialize().unwrap().as_css(), "calc(1px)");
    let CssCalculationExpressionRef::NestedCalc(calc) = calculation.expression() else {
        panic!("calc")
    };
    let CssCalculationExpressionRef::Value(value) = calc.operand() else {
        panic!("length leaf")
    };
    assert_eq!(value.literal().representation(), "1");
    assert_eq!(value.literal().unit(), Some("px"));
    let CssValueOrigin::Parsed(origin) = value.literal().origin() else {
        panic!("authored leaf")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("1px").unwrap()
    );
    assert_eq!(origin.span().start().line().value(), 0);
    assert_eq!(
        origin.span().start().column().value(),
        u32::try_from(source[..source.find("1px").unwrap()].encode_utf16().count()).unwrap()
    );
    assert_eq!(condition.serialize().unwrap().as_css(), "(width:calc(1px))");
    assert!(matches!(
        CssSupportsCondition::try_from_components(
            values(condition.components().to_vec()),
            &CssNamespaceContext::default()
        ),
        Err(CssSupportsConstructionError::RecoveredInput { .. })
    ));
}

#[test]
fn authored_legacy_property_alias_keeps_its_own_grammar_in_parsed_and_constructed_supports() {
    use surgeist_css::{CssKnownProperty, CssPropertyGrammar};
    // The legacy angle grammar maps to TextOrientation while retaining its
    // alias grammar identity; 0deg is not the canonical property's grammar.
    let grammar = CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap();
    for value in ["0deg", "inherit", "var(--orientation)"] {
        let source = format!("glyph-orientation-vertical:{value}");
        let constructed = declaration(&source);
        let report = parse_sheet(&format!("@supports ({source}) {{}}"));
        assert!(report.is_clean(), "{report:?}");
        let CssRule::Supports(rule) = &report.syntax().rules()[0] else {
            panic!("supports")
        };
        let CssSupportsConditionKind::Declaration(parsed) = rule.condition().kind() else {
            panic!("declaration")
        };
        for declaration in [&constructed, parsed.as_ref()] {
            let known = declaration
                .known()
                .expect("authored alias grammar admits the value");
            assert_eq!(known.property(), CssKnownProperty::TextOrientation);
            assert_eq!(known.grammar(), grammar);
            assert_eq!(declaration.serialize().unwrap().as_css(), source);
            match value {
                "inherit" => assert_eq!(known.global(), Some(CssGlobalKeyword::Inherit)),
                "var(--orientation)" => assert!(known.substitution_dependent().is_some()),
                _ => assert!(known.property_value().is_some()),
            }
        }
    }
}
