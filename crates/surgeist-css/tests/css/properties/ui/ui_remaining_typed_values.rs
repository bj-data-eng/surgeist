#![forbid(unsafe_code)]
//! New typed APIs, tested functionally with implementation from UI4/Images4 and CSSOM.
use surgeist_css::*;

fn declaration(property: CssKnownProperty, text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap()
}
fn caret(text: &str) -> CssCaret {
    let declaration = declaration(CssKnownProperty::Caret, text);
    let CssKnownPropertyValueRef::Caret(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("caret")
    };
    value.caret().clone()
}
fn interest(text: &str) -> CssInterestDelay {
    let declaration = declaration(CssKnownProperty::InterestDelay, text);
    let CssKnownPropertyValueRef::InterestDelay(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("interest")
    };
    value.delay().clone()
}
fn time(number: &str, unit: CssTimeUnit) -> CssInterestDelayValue {
    CssInterestDelayValue::Time(CssTimeValue::from_literal(
        CssTimeLiteral::try_new(number, unit).unwrap(),
    ))
}
fn exact_limits(
    emit: impl Fn(
        CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError>,
    expected: &str,
    nodes: usize,
) {
    assert_eq!(
        emit(CssSpecifiedValueSerializationLimits::new(
            nodes,
            nodes,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(emit(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(
        emit(CssSpecifiedValueSerializationLimits::new(
            nodes,
            nodes,
            expected.len()
        ))
        .unwrap(),
        expected
    );
}
#[test]
fn checked_caret_preserves_roles_while_auto_projection_omits_initials() {
    assert!(CssCaret::try_new(None, None, None).is_none());
    for (text, expected) in [
        ("auto manual block", "manual block"),
        ("red auto auto", "red"),
        ("manual auto auto", "manual"),
        ("block auto auto", "block"),
        ("auto auto", "auto"),
        ("auto auto auto", "auto"),
    ] {
        let value = caret(text);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let reparsed = caret(expected);
        assert_eq!(reparsed.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
    }
    let value = CssCaret::try_new(
        Some(CssCaretColor::Auto),
        Some(CssCaretAnimation::Manual),
        Some(CssCaretShape::Block),
    )
    .unwrap();
    exact_limits(
        |limits| value.serialize_specified_with_limits(limits),
        "manual block",
        4,
    );
    assert!(matches!(value.color(), Some(CssCaretColor::Auto)));
    assert_eq!(value.animation(), Some(CssCaretAnimation::Manual));
    assert_eq!(value.shape(), Some(CssCaretShape::Block));
    let all = CssCaret::try_new(
        Some(CssCaretColor::Auto),
        Some(CssCaretAnimation::Auto),
        Some(CssCaretShape::Auto),
    )
    .unwrap();
    exact_limits(
        |limits| all.serialize_specified_with_limits(limits),
        "auto",
        4,
    );
}
#[test]
fn typed_keyword_providers_reparse_each_distinct_ui_identity() {
    assert_eq!(CssCaretAnimation::default(), CssCaretAnimation::Auto);
    assert_eq!(CssCaretShape::default(), CssCaretShape::Auto);
    assert_eq!(CssInteractivity::default(), CssInteractivity::Auto);
    assert_eq!(CssAppearance::default(), CssAppearance::None);
    for (value, text) in [
        (CssCaretAnimation::Auto, "auto"),
        (CssCaretAnimation::Manual, "manual"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), text);
        let d = declaration(CssKnownProperty::CaretAnimation, text);
        let CssKnownPropertyValueRef::CaretAnimation(v) =
            d.known().unwrap().property_value().unwrap()
        else {
            panic!("animation")
        };
        assert_eq!(*v.animation(), value);
    }
    for (value, text) in [
        (CssCaretShape::Auto, "auto"),
        (CssCaretShape::Bar, "bar"),
        (CssCaretShape::Block, "block"),
        (CssCaretShape::Underscore, "underscore"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), text);
        let d = declaration(CssKnownProperty::CaretShape, text);
        let CssKnownPropertyValueRef::CaretShape(v) = d.known().unwrap().property_value().unwrap()
        else {
            panic!("shape")
        };
        assert_eq!(*v.shape(), value);
    }
    for (value, text) in [
        (CssInteractivity::Auto, "auto"),
        (CssInteractivity::Inert, "inert"),
    ] {
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            text,
            1,
        );
        let d = declaration(CssKnownProperty::Interactivity, text);
        let CssKnownPropertyValueRef::Interactivity(v) =
            d.known().unwrap().property_value().unwrap()
        else {
            panic!("interactivity")
        };
        assert_eq!(*v.interactivity(), value);
    }
    for (value, text) in [
        (CssAppearance::None, "none"),
        (CssAppearance::Auto, "auto"),
        (CssAppearance::Base, "base"),
        (CssAppearance::BaseSelect, "base-select"),
        (CssAppearance::Searchfield, "searchfield"),
        (CssAppearance::Textarea, "textarea"),
        (CssAppearance::Checkbox, "checkbox"),
        (CssAppearance::Radio, "radio"),
        (CssAppearance::Menulist, "menulist"),
        (CssAppearance::Listbox, "listbox"),
        (CssAppearance::Meter, "meter"),
        (CssAppearance::ProgressBar, "progress-bar"),
        (CssAppearance::Button, "button"),
        (CssAppearance::Textfield, "textfield"),
        (CssAppearance::MenulistButton, "menulist-button"),
    ] {
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            text,
            1,
        );
        let d = declaration(CssKnownProperty::Appearance, text);
        let CssKnownPropertyValueRef::Appearance(v) = d.known().unwrap().property_value().unwrap()
        else {
            panic!("appearance")
        };
        assert_eq!(*v.appearance(), value);
    }
    assert!(matches!(CssAccentColor::default(), CssAccentColor::Auto));
    let accent = CssAccentColor::Color(Box::new(CssColor::transparent()));
    assert!(accent.color().is_some());
    exact_limits(
        |limits| accent.serialize_specified_with_limits(limits),
        "transparent",
        1,
    );
}
#[test]
fn interest_omission_uses_exact_signed_time_proof_and_preserves_second_presence() {
    for (text, expected) in [
        ("normal normal", "normal"),
        ("1s 1000ms", "1s"),
        ("-1s -1000ms", "-1s"),
        ("calc(1s + 2s) calc(1s + 2s)", "calc(3s)"),
        ("0.0000001s 0.0000002s", "0s 0s"),
        ("calc(1s + 1s) 2s", "calc(2s) 2s"),
        ("normal 0s", "normal 0s"),
    ] {
        let value = interest(text);
        let before = value.clone();
        assert!(value.authored_end().is_some());
        assert_eq!(value.serialize_specified().unwrap(), expected);
        if text != "0.0000001s 0.0000002s" {
            assert_eq!(interest(expected).serialize_specified().unwrap(), expected);
        } else {
            // Rounded text cannot reconstitute the original distinction; omission proof
            // is made before formatting, so both retained original inputs are emitted.
            assert!(interest(expected).authored_end().is_some());
        }
        assert_eq!(value, before);
    }
    let value = CssInterestDelay::try_new(
        time("-1", CssTimeUnit::Seconds),
        Some(time("-1000", CssTimeUnit::Milliseconds)),
    )
    .unwrap();
    exact_limits(
        |limits| value.serialize_specified_with_limits(limits),
        "-1s",
        3,
    );
    assert!(value.authored_end().is_some());
    let repeated = CssInterestDelay::try_new(time("-1", CssTimeUnit::Seconds), None).unwrap();
    assert!(repeated.authored_end().is_none());
    assert_eq!(repeated.start(), repeated.end());
    exact_limits(
        |limits| repeated.serialize_specified_with_limits(limits),
        "-1s",
        2,
    );
    assert!(
        CssDuration::try_new(CssTimeValue::from_literal(
            CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap()
        ))
        .is_err()
    );
}
#[test]
fn navigation_checked_models_preserve_original_ids_strings_and_exclude_legacy_targets() {
    for bad in ["#123", "#", "Next", ".Next", "[id=Next]"] {
        if let Ok(component) = CssComponentValue::try_token(bad) {
            assert!(CssNavigationId::try_from_component(component).is_err());
        }
    }
    let component = parse_component_values(r"#N\65 xt").unwrap().items()[0].clone();
    let original = component.origin().clone();
    let id = CssNavigationId::try_from_component(component.clone()).unwrap();
    assert_eq!(id.as_str(), "Next");
    assert_eq!(id.origin(), &original);
    assert_eq!(id.component(), &component);
    assert_eq!(CssNavigationId::try_new("Next").unwrap().as_str(), "Next");
    for bad in ["_root", "_parent", "_Frame"] {
        assert!(CssNavigationTargetName::try_new(bad).is_err());
    }
    for name in ["", "Frame", "frame"] {
        let target = CssNavigationTargetName::try_new(name).unwrap();
        assert_eq!(target.as_str(), name);
        let navigation = CssNavigation::Id(Box::new(
            CssNavigationReference::try_new(id.clone(), Some(CssNavigationTarget::Name(target)))
                .unwrap(),
        ));
        let text = navigation.serialize_specified().unwrap();
        let d = declaration(CssKnownProperty::NavUp, &text);
        let CssKnownPropertyValueRef::NavUp(v) = d.known().unwrap().property_value().unwrap()
        else {
            panic!("nav")
        };
        assert_eq!(v.navigation().reference().unwrap().id().as_str(), "Next");
        let Some(CssNavigationTarget::Name(target)) = v.navigation().reference().unwrap().target()
        else {
            panic!("name")
        };
        assert_eq!(target.as_str(), name);
    }
}
#[test]
fn legacy_navigation_has_typed_diagnostic_original_string_and_reusable_report_paths() {
    let value = r#"#N\65 xt "\5f ROOT""#;
    let property = CssKnownProperty::NavUp;
    for report in [
        parse_property_value_text(
            value,
            CssPropertyNameRef::Known(property),
            CssImportance::Normal,
        ),
        parse_property_value_text_for_grammar(value, property.grammar(), CssImportance::Normal),
        parse_declaration(&format!("nav-up:{value}")),
    ] {
        assert!(!report.is_clean());
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainLegacyNavigationTarget
        );
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::LegacyNavigationTarget
        );
        let ErrorKind::LegacyNavigationTarget(detail) = diagnostic.error().kind() else {
            panic!("typed diagnostic")
        };
        assert_eq!(detail.target(), "_ROOT");
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::NavUp(v) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("nav")
        };
        let reference = v.navigation().reference().unwrap();
        let Some(CssNavigationTarget::LegacyName(target)) = reference.target() else {
            panic!("legacy")
        };
        assert_eq!(target.as_str(), "_ROOT");
        assert_eq!(target.origin(), target.component().origin());
        assert!(
            CssNavigationReference::try_new(reference.id().clone(), reference.target().cloned())
                .is_none()
        );
        assert!(CssNavigationTargetName::try_from_component(target.component().clone()).is_err());
    }
    let source = format!("@keyframes a{{from{{nav-up:{value};nav-up:#Other root}}}}");
    let report = parse_sheet(&source);
    let before = report.clone();
    assert!(validate_sheet(&source).is_err());
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RetainLegacyNavigationTarget)
        .unwrap();
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::LegacyNavigationTarget
    );
    let CssRule::Keyframes(rule) = &report.syntax().rules()[0] else {
        panic!("keyframes")
    };
    let declarations = rule.blocks()[0].declarations().as_slice();
    assert_eq!(declarations.len(), 2);
    assert_eq!(
        declarations[0].position().byte_offset().value(),
        source.find("nav-up").unwrap()
    );
    for (declaration, expected) in declarations.iter().zip([value, "#Other root"]) {
        let CssKnownPropertyValueRef::NavUp(v) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("keyframe nav")
        };
        assert_eq!(v.as_css(), expected);
        let id = v.navigation().reference().unwrap().id();
        let CssValueOrigin::Parsed(origin) = id.origin() else {
            panic!("parsed ID")
        };
        assert_eq!(origin.source().as_str(), source);
    }
    assert!(
        report
            .syntax()
            .to_specified_css()
            .unwrap()
            .contains(r#"nav-up: #Next "_ROOT";"#)
    );
    assert_eq!(report, before);
    assert!(validate_sheet("@keyframes a{from{nav-up:#Next root;nav-up:#Other current}}").is_ok());
    let valid = parse_style_attribute("nav-up:#Next root;nav-down:#Next \"\"");
    assert!(valid.is_clean());
    assert!(conformance_exclusion("excluded.X-UI4.property.ime-mode").is_some());
    assert!(CssKnownProperty::from_name("ime-mode").is_none());
}
#[test]
fn stripes_checked_construction_preserves_optional_thickness_and_exact_origins() {
    let color = parse_component_values("ReD").unwrap().items()[0].clone();
    let origin = color.origin().clone();
    let thickness = CssStripeThickness::Flex(
        CssSpecifiedFlex::try_from_component(
            CssComponentValue::try_dimension("1.0", "FR").unwrap(),
        )
        .unwrap(),
    );
    let stripe = CssStripe::try_new(color.clone(), Some(thickness)).unwrap();
    assert_eq!(stripe.color_origin(), &origin);
    assert_eq!(stripe.color_component(), &color);
    assert!(stripe.thickness().is_some());
    let stripes = CssStripes::try_new(vec![stripe]).unwrap();
    let before = stripes.clone();
    exact_limits(
        |limits| stripes.serialize_specified_with_limits(limits),
        "stripes(red)",
        4,
    );
    assert_eq!(stripes, before);
    assert!(stripes.original_component().is_none());
    assert!(matches!(
        CssStripes::try_new(vec![]),
        Err(CssImage1DConstructionError::EmptyStripes)
    ));
    let image = CssImage1D::try_from_components(
        parse_component_values("stripes(2px ReD, blue 100%)").unwrap(),
    )
    .unwrap();
    assert_eq!(
        image.serialize_specified().unwrap(),
        "stripes(red 2px, blue 100%)"
    );
    assert_eq!(image.stripes().stripes().len(), 2);
    assert!(image.stripes().original_component().is_some());
    let outline = CssOutlineColor::Image1D(Box::new(image.clone()));
    assert!(outline.color().is_none());
    assert_eq!(outline.image_1d(), Some(&image));
    assert_eq!(
        outline.serialize_specified().unwrap(),
        image.serialize_specified().unwrap()
    );
}
#[test]
fn stripe_thickness_uses_exact_ranges_signed_flex_and_existing_grid_boundary() {
    for text in [
        "-1%",
        "100.00000000000000000001%",
        "1e100000%",
        "-1px",
        "1s",
    ] {
        assert!(
            CssStripeLengthPercentage::try_from_component(
                CssComponentValue::try_token(text).unwrap()
            )
            .is_err()
        );
    }
    for text in ["0%", "100%", "-0px", "2em"] {
        let value = CssStripeLengthPercentage::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap();
        assert!(value.serialize_specified().is_ok());
    }
    let negative = CssComponentValue::try_dimension("-1", "fr").unwrap();
    assert_eq!(
        CssSpecifiedFlex::try_from_component(negative.clone())
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "-1fr"
    );
    assert!(CssSpecifiedNonNegativeFlex::try_from_component(negative).is_err());
    for (source, expected) in [
        ("stripes(red -1fr)", "stripes(red -1fr)"),
        ("stripes(red calc(2fr - 3fr))", "stripes(red calc(-1fr))"),
        (
            "stripes(red calc(1px + 1em))",
            "stripes(red calc(1em + 1px))",
        ),
    ] {
        let image =
            CssImage1D::try_from_components(parse_component_values(source).unwrap()).unwrap();
        assert_eq!(image.serialize_specified().unwrap(), expected);
        assert_eq!(
            CssImage1D::try_from_components(parse_component_values(expected).unwrap())
                .unwrap()
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
}
#[test]
fn recovered_stripe_closure_is_retained_only_by_browser_and_checked_reentry_rejects_it() {
    let report = parse_style_attribute("outline-color:stripes(red calc(1fr + 2fr)");
    assert!(!report.is_clean());
    let declaration = &report.syntax()[0];
    let CssKnownPropertyValueRef::OutlineColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("outline")
    };
    let image = value.value().image_1d().unwrap();
    let original = image.stripes().original_component().unwrap();
    let components = CssComponentValues::try_new(vec![original.clone()]).unwrap();
    assert!(matches!(
        CssImage1D::try_from_components(components.clone()),
        Err(CssImage1DConstructionError::RecoveredComponent(_))
    ));
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::OutlineColor),
            components,
            CssImportance::Normal
        )
        .is_err()
    );
    assert!(CssStripes::try_new(image.stripes().stripes().to_vec()).is_ok());
}
#[test]
fn typed_stripe_errors_use_original_component_origins_and_aggregate_depth() {
    let source = "/*lead*/stripes(red 101%)";
    let error =
        CssImage1D::try_from_components(parse_component_values(source).unwrap()).unwrap_err();
    let CssImage1DConstructionError::Grammar(error) = error else {
        panic!("grammar error")
    };
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
        panic!("original responsible component")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("101%").unwrap()
    );
    let nested = |depth: usize| {
        format!(
            "{}red{}",
            "light-dark(blue,".repeat(depth),
            ")".repeat(depth)
        )
    };
    let color = parse_component_values(&nested(255)).unwrap().items()[0].clone();
    let stripe = CssStripe::try_new(color, None).unwrap();
    assert!(CssStripes::try_new(vec![stripe]).is_ok());
    let source = format!("stripes({})", nested(255));
    let image = CssImage1D::try_from_components(parse_component_values(&source).unwrap()).unwrap();
    assert_eq!(image.stripes().stripes().len(), 1);
    let CssValueOrigin::Parsed(origin) = image.stripes().origin() else {
        panic!("original stripes origin")
    };
    assert_eq!(origin.source().as_str(), source);
    let color = parse_component_values(&nested(256)).unwrap().items()[0].clone();
    assert!(matches!(
        CssStripe::try_new(color, None),
        Err(CssImage1DConstructionError::NestingLimit)
    ));
}
#[test]
fn signed_flex_checked_calculation_reuses_numeric_owner_and_rejects_recovered_graph() {
    let calculation =
        CssFlexCalculation::try_from_components(parse_component_values("calc(2fr - 3fr)").unwrap())
            .unwrap();
    let value = CssSpecifiedFlex::try_from_calculation(calculation).unwrap();
    assert!(value.calculation().is_some());
    assert_eq!(value.serialize_specified().unwrap(), "calc(-1fr)");
    let other = CssSpecifiedFlex::try_from_calculation(
        CssFlexCalculation::try_from_components(
            parse_component_values("/*other origin*/calc(2fr - 3fr)").unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_ne!(value.origin(), other.origin());
    assert_eq!(
        CssStripeThickness::Flex(value),
        CssStripeThickness::Flex(other)
    );
    let report = parse_style_attribute("outline-color:stripes(red calc(2fr - 3fr");
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::OutlineColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("outline")
    };
    let Some(CssStripeThickness::Flex(flex)) =
        value.value().image_1d().unwrap().stripes().stripes()[0].thickness()
    else {
        panic!("flex")
    };
    let error =
        CssSpecifiedFlex::try_from_calculation(flex.calculation().unwrap().clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    assert!(error.origin().is_some());
}
#[test]
fn interest_checked_construction_rejects_recovered_time_from_either_slot() {
    let report = parse_style_attribute("interest-delay-start:calc(1s + 2s");
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::InterestDelayStart(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("delay")
    };
    let recovered = value.delay().clone();
    for (start, end) in [
        (recovered.clone(), None),
        (CssInterestDelayValue::Normal, Some(recovered)),
    ] {
        assert_eq!(
            CssInterestDelay::try_new(start, end).unwrap_err().kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
    }
}
