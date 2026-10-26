//! Authored environment substitution preserves token identity and boundary policy.
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
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
    let arguments =
        CssComponentValues::try_new(vec![name.clone(), index.clone()], Default::default()).unwrap();
    let function = CssComponentValue::try_function("env", arguments).unwrap();
    let values = CssComponentValues::try_new(vec![function], Default::default()).unwrap();
    assert_eq!(
        values
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "env(Future/**/2)"
    );
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
        let condition = CssCondition::<CssContainerLeaf>::try_from_components(
            parse_component_values(text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let CssConditionKind::Leaf(CssContainerLeaf::GeneralEnclosed(opaque)) = condition.kind()
        else {
            panic!("env must not add query-domain admission: {text}")
        };
        assert_eq!(opaque.authored(), Some(text));
    }
    let style = CssCondition::<CssContainerLeaf>::try_from_components(
        parse_component_values("style(--x:env(foo))").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        style.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Style(_))
    ));
}

#[test]
fn style_env_components_obey_explicit_enclosing_resource_limits() {
    let values = parse_component_values("style(--x:env(foo))").unwrap();
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(1, usize::MAX, usize::MAX).unwrap(),
            ComponentFailure::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
            ComponentFailure::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
            ComponentFailure::ByteLimit,
        ),
    ] {
        let Err(outer) =
            CssCondition::<CssContainerLeaf>::try_from_components(values.clone(), limits)
        else {
            panic!("resource failure must not become opaque success")
        };
        let Some(error) = outer.component_error() else {
            panic!("resource failure must not become opaque success")
        };
        assert_eq!(component_failure(error), kind);
    }
    let accepted =
        CssCondition::<CssContainerLeaf>::try_from_components(values.clone(), Default::default())
            .unwrap();
    assert!(matches!(
        accepted.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Style(_))
    ));
    assert_eq!(
        values
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "style(--x:env(foo))"
    );
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
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
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
    // The second retained declaration exceeds a one-declaration budget.
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 1, 2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Declarations, 1))
    );
    assert_eq!(
        error
            .declaration()
            .and_then(|declaration| declaration.known()?.substitution_dependent())
            .map(|value| value.as_css().to_owned())
            .as_deref(),
        Some("env(second)")
    );
}

#[test]
fn env_catalog_keeps_required_source_and_remaining_contexts_distinct() {
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
            "Other Env1 contexts and environment lookup/substitution execution remain outside this authored subset; this record does not select the complete Env1 module."
        )
    );
}
