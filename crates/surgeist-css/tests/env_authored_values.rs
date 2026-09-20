#![forbid(unsafe_code)]
//! Authored environment substitution preserves token identity and boundary policy.
use surgeist_css::*;

fn display(values: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
        values,
        CssImportance::Important,
    )
    .unwrap()
}

#[test]
fn programmatic_functions_preserve_mixed_origins_and_separators() {
    let name = parse_component_values("Future").unwrap().items()[0].clone();
    let index = CssComponentValue::try_token("2").unwrap();
    // Separate tokens must not become the single environment name Future2.
    let arguments = CssComponentValues::try_new(vec![name.clone(), index.clone()]).unwrap();
    let function = CssComponentValue::try_function("env", arguments).unwrap();
    let values = CssComponentValues::try_new(vec![function]).unwrap();
    assert_eq!(values.serialize().unwrap().as_css(), "env(Future/**/2)");
    let declaration = display(values.clone());
    assert_eq!(declaration.value_components(), &values);
    assert_eq!(declaration.position(), None);
    let CssComponentValueRef::Function(function) = declaration.value_components().items()[0].view()
    else {
        panic!("environment function")
    };
    assert_eq!(function.values().items(), &[name, index]);
    assert!(matches!(
        function.values().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        function.values().items()[1].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(
        declaration.value_components().items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(
        declaration
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        "env(Future/**/2)"
    );
}

#[test]
fn decoded_names_and_nested_functions_keep_authored_case_and_comments() {
    for text in [
        r"e\6ev(Future)",
        r"env(\66 uture/**/+0002,/**/ )",
        "env(--Future)",
        "env(foo, f([a;b!c], {d:e}))",
    ] {
        let values = parse_component_values(text).unwrap();
        let declaration = display(values.clone());
        assert_eq!(declaration.value_components(), &values);
        assert_eq!(
            declaration
                .known()
                .unwrap()
                .substitution_dependent()
                .unwrap()
                .as_css(),
            text
        );
        assert!(matches!(
            expand_declaration(&declaration).unwrap(),
            CssExpansion::Pending(_)
        ));
    }
}

#[test]
fn property_substitution_does_not_admit_env_as_size_or_scroll_operands() {
    for text in [
        "(width:env(foo))",
        "(aspect-ratio:env(foo))",
        "scroll-state(snapped:env(foo))",
        "scroll-state(scrollable:env(foo))",
    ] {
        let condition =
            CssContainerCondition::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let CssContainerConditionKind::GeneralEnclosed(opaque) = condition.kind() else {
            panic!("env must not add query-domain admission: {text}")
        };
        assert_eq!(opaque.authored(), Some(text));
    }
    let style = CssContainerCondition::try_from_components(
        parse_component_values("style(--x:env(foo))").unwrap(),
    )
    .unwrap();
    assert!(matches!(style.kind(), CssContainerConditionKind::Style(_)));
}

#[test]
fn style_env_components_obey_explicit_enclosing_resource_limits() {
    let values = parse_component_values("style(--x:env(foo))").unwrap();
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(1, usize::MAX, usize::MAX).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let Err(CssContainerConstructionError::Component(error)) =
            CssContainerCondition::try_from_components_with_limits(values.clone(), limits)
        else {
            panic!("resource failure must not become opaque success")
        };
        assert_eq!(error.kind(), kind);
    }
    let accepted = CssContainerCondition::try_from_components(values.clone()).unwrap();
    assert!(matches!(
        accepted.kind(),
        CssContainerConditionKind::Style(_)
    ));
    assert_eq!(values.serialize().unwrap().as_css(), "style(--x:env(foo))");
}

#[test]
fn recovered_neighbor_and_pending_normalization_keep_occurrence_order() {
    let source = ".a{display:env(first);display:env(foo -1);display:env(second)!important}";
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
        .zip(["env(first)", "env(second)"])
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
    assert!(
        normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(0, 1, 1, 2).unwrap()
        )
        .is_err()
    );
}

#[test]
fn env_catalog_keeps_required_source_and_descriptor_remainder_distinct() {
    let feature = feature_metadata("required.value.environment-substitution").unwrap();
    assert_eq!(feature.kind(), CssFeatureKind::Value);
    assert_eq!(feature.status(), CssSupportStatus::Partial);
    assert_eq!(feature.source().id().as_str(), "D-ENV1");
    assert_eq!(feature.source().tier(), CssSpecificationTier::LaterStandard);
    assert_eq!(
        feature.source().url(),
        Some("https://www.w3.org/TR/2025/WD-css-env-1-20250923/")
    );
    assert_eq!(
        specification_source("D-ENV1").copied(),
        Some(feature.source())
    );
    assert_eq!(
        feature.unsupported_remainder(),
        Some(
            "The required font-palette descriptor consumer remains unimplemented. Environment lookup and substitution execution belong to style; this record does not select the complete Env1 module."
        )
    );
}
