use crate::css::support::{checked_color as color, declaration_color};
use surgeist_color::{Color, ColorSpace, PredefinedColorSpace};
use surgeist_css::*;

/// A finite numerical color from plain test coordinates.
fn numerical_color(space: ColorSpace, channels: [Option<f64>; 3], alpha: Option<f64>) -> Color {
    Color::new(
        space,
        channels.map(|channel| channel.map(|value| surgeist_color::Finite::new(value).unwrap())),
        alpha.map(|value| surgeist_color::UnitInterval::new(value).unwrap()),
    )
}

fn facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(41, 7),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    )
}
fn limits() -> CssAuthoredColorOrdinaryLimits {
    CssAuthoredColorOrdinaryLimits {
        algorithm_records: 100_000,
        arithmetic_records: 10_000,
        evaluation_records: 10_000,
        path_steps: 100,
        actions: 30_000_000,
    }
}
#[test]
fn literal_checked_values_and_exact_expressions_resolve_through_the_public_front_door() {
    let facts = facts();
    for (text, channels, alpha) in [
        ("red", [1.0, 0.0, 0.0], 1.0),
        (
            "#1234",
            [17.0 / 255.0, 34.0 / 255.0, 51.0 / 255.0],
            68.0 / 255.0,
        ),
        ("rgb(round(UP, 5.1, 2) 0 0)", [6.0 / 255.0, 0.0, 0.0], 1.0),
        (
            "rgb(clamp(none, 5, none) 0 0)",
            [5.0 / 255.0, 0.0, 0.0],
            1.0,
        ),
        (
            "rgb(calc(2 + 3 * 4) 0 255 / .5)",
            [14.0 / 255.0, 0.0, 1.0],
            0.5,
        ),
        (
            "light-dark(rgb(255 0 0), currentcolor)",
            [1.0, 0.0, 0.0],
            1.0,
        ),
        ("rgb(from red r g b / .5)", [1.0, 0.0, 0.0], 0.5),
        ("color-mix(in srgb, red 25%, blue)", [0.25, 0.0, 0.75], 1.0),
    ] {
        let root = color(text);
        let result = resolve_authored_color(&root, &facts, facts.identity(), limits())
            .unwrap_or_else(|error| panic!("{text}: {error:?}"));
        assert!(std::ptr::eq(result.authored(), &root));
        assert!(std::ptr::eq(result.facts(), &facts));
        let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
            panic!("{text}: standard failure")
        };
        assert_eq!(
            value.space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb),
            "{text}"
        );
        assert_eq!(value.channels(), channels.map(Some), "{text}");
        assert_eq!(value.alpha(), Some(alpha), "{text}");
    }
    assert_eq!(
        Color::from_srgba8([255, 0, 0, 255]).channels(),
        [Some(1.0), Some(0.0), Some(0.0)]
    );
}

#[test]
fn checked_branch_families_and_unreached_nonfinite_expressions_keep_literal_results() {
    let id = CssColorFactIdentity::new(41, 7);
    let palette = [CssColorPaletteEntry::new(
        CssSystemColor::CanvasText,
        id,
        Color::from_srgba8([255, 0, 0, 255]),
    )];
    let facts = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Current(Color::from_srgba8([0, 255, 0, 255])),
        CssColorSchemeFact::Light,
        &palette,
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    let rgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
    for (text, space, channels, alpha) in [
        ("currentcolor", rgb, [0.0, 1.0, 0.0], 1.0),
        ("transparent", rgb, [0.0, 0.0, 0.0], 0.0),
        ("CanvasText", rgb, [1.0, 0.0, 0.0], 1.0),
        ("hsl(0 100% 50%)", rgb, [1.0, 0.0, 0.0], 1.0),
        ("hwb(0 0% 0%)", rgb, [1.0, 0.0, 0.0], 1.0),
        ("lab(50 0 0)", ColorSpace::Lab, [50.0, 0.0, 0.0], 1.0),
        ("lch(50 0 0)", ColorSpace::Lch, [50.0, 0.0, 0.0], 1.0),
        ("oklab(.5 0 0)", ColorSpace::Oklab, [0.5, 0.0, 0.0], 1.0),
        ("oklch(.5 0 0)", ColorSpace::Oklch, [0.5, 0.0, 0.0], 1.0),
        ("color(srgb 1 0 0)", rgb, [1.0, 0.0, 0.0], 1.0),
        ("alpha(from red / .5)", rgb, [1.0, 0.0, 0.0], 0.5),
        ("contrast-color(white)", rgb, [0.0, 0.0, 0.0], 1.0),
        ("device-cmyk(0 1 1 0)", rgb, [1.0, 0.0, 0.0], 1.0),
        (
            "light-dark(red,lab(50 calc(infinity) 0))",
            rgb,
            [1.0, 0.0, 0.0],
            1.0,
        ),
    ] {
        let root = color(text);
        let result = resolve_authored_color(&root, &facts, id, limits())
            .unwrap_or_else(|e| panic!("{text}: {e:?}"));
        let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
            panic!("{text}: standard failure")
        };
        assert_eq!(value.space(), space, "{text}");
        assert_eq!(value.channels(), channels.map(Some), "{text}");
        assert_eq!(value.alpha(), Some(alpha), "{text}");
    }
}

#[test]
fn custom_profile_payloads_are_contiguous_and_keep_their_original_channel_order() {
    let id = CssColorFactIdentity::new(41, 7);
    let resource = CssColorFactIdentity::new(8, 9);
    let profile = CssColorProfileName::try_new("--p").unwrap();
    let intent = CssColorProfileRenderingIntent::RelativeColorimetric;
    let names = [CssColorProfileComponentName::try_new("x").unwrap()];
    let bindings = [CssColorProfileBinding::new(
        &profile,
        resource,
        CssColorProfileAvailability::Loaded,
        intent,
        5,
        &names,
    )];
    let inputs = [0.1, 0.2, 0.3, 0.4, 0.5];
    let target = ColorSpace::Predefined(PredefinedColorSpace::XyzD50);
    let converted = numerical_color(target, [Some(0.2), Some(0.3), Some(0.4)], Some(1.0));
    let conversions = [CssColorProfileConversion::Resolved(
        CssColorProfileConversionRecord::new(resource, &inputs, intent, target, converted),
    )];
    let facts = CssColorFacts::new(
        id,
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &bindings,
        &conversions,
        CssDeviceColorFact::Unknown,
    );
    for text in [
        "color(--p .1 .2 .3 .4 .5 / .5)",
        "color(from red --p .1 .2 .3 .4 .5 / .5)",
    ] {
        let root = color(text);
        let result = resolve_authored_color(&root, &facts, id, limits())
            .unwrap_or_else(|e| panic!("{text}: {e:?}"));
        let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
            panic!("used")
        };
        assert_eq!(value.space(), target);
        assert_eq!(value.channels(), [Some(0.2), Some(0.3), Some(0.4)]);
        assert_eq!(value.alpha(), Some(0.5));
    }
}

#[test]
fn completed_standard_failure_owns_its_branch_path_and_take_does_not_clone_it() {
    let facts = facts();
    let root = color("light-dark(color-mix(in srgb,red calc(-1%),blue),blue)");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::StandardFailure(failure) = result.outcome() else {
        panic!("a reached calculated negative weight is standard invalidity")
    };
    assert_eq!(failure.kind, CssColorStandardFailureKind::InvalidComponents);
    assert_eq!(
        failure.path.steps(),
        [CssColorPathStep::First, CssColorPathStep::MixWeight(0)]
    );
    assert!(failure.path.capacity() >= 2);
    let Some(CssValueOrigin::Parsed(origin)) = &failure.origin else {
        panic!("actual calculation opener")
    };
    assert_eq!(
        origin.source().as_str(),
        "light-dark(color-mix(in srgb,red calc(-1%),blue),blue)"
    );
    assert_eq!(origin.span().start().byte_offset().value(), 33);
}

#[test]
fn initial_foreign_facts_and_ordinary_denials_keep_their_original_classes() {
    let facts = facts();
    let root = color("red");
    let mut denied = limits();
    denied.actions = 0;
    assert!(matches!(
        resolve_authored_color(&root, &facts, CssColorFactIdentity::new(0, 0), denied),
        Err(CssColorOrdinaryError::Operation(CssColorError::Facts(
            CssColorFactError::Foreign
        )))
    ));
    assert!(matches!(
        resolve_authored_color(&root, &facts, facts.identity(), denied),
        Err(CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::Actions,
            required: 1,
            allowed: 0
        })
    ));
    let root = color("rgb(1 2 3)");
    denied = limits();
    denied.path_steps = 0;
    assert!(matches!(
        resolve_authored_color(&root, &facts, facts.identity(), denied),
        Err(CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::PathSteps,
            required: 1,
            allowed: 0
        })
    ));
    denied = limits();
    denied.algorithm_records = 0;
    assert!(matches!(
        resolve_authored_color(&root, &facts, facts.identity(), denied),
        Err(CssColorOrdinaryError::Limit {
            resource: CssColorOrdinaryResource::AlgorithmRecords,
            required: 1,
            allowed: 0
        })
    ));
}

#[test]
fn original_programmatic_and_parsed_scalars_keep_spelling_and_resolve_unsigned_zero() {
    // Values 4 (WD-20240312): signed zeros cannot be written in CSS; 0, +0 and -0
    // all produce the unsigned zero. The authored spelling is still retained.
    let facts = facts();
    let number = |text: &str| {
        CssColorComponent::Number(
            CssColorNumberLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
                .unwrap(),
        )
    };
    let root = CssColor::from_lab(
        CssLabColor::try_new(number("50"), number("-0"), number("1e-999"), None).unwrap(),
    );
    let CssColorComponent::Number(a) = root.lab_value().unwrap().a() else {
        panic!("number")
    };
    assert_eq!(a.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(a.numeric().representation(), "-0");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("used")
    };
    assert_eq!(value.channels()[1].unwrap().to_bits(), 0.0f64.to_bits());
    assert_eq!(value.channels()[2], Some(0.0));
    let root = color("lab(50 -0 1e-999)");
    let CssColorComponent::Number(a) = root.lab_value().unwrap().a() else {
        panic!("number")
    };
    let CssValueOrigin::Parsed(origin) = a.origin() else {
        panic!("original parsed origin")
    };
    assert_eq!(origin.source().as_str(), "lab(50 -0 1e-999)");
    let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("used")
    };
    assert_eq!(value.channels()[1].unwrap().to_bits(), 0.0f64.to_bits());
    assert_eq!(a.numeric().representation(), "-0");
}

#[test]
fn programmatic_and_parsed_calculated_failures_keep_their_actual_origins() {
    let facts = facts();
    let function = CssComponentValue::try_function(
        "calc",
        CssComponentValues::try_new(
            vec![CssComponentValue::try_token("-1%").unwrap()],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let programmatic = CssPercentageCalculation::try_from_components(
        CssComponentValues::try_new(vec![function], Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let recovered = CssPercentageCalculation::try_from_components(
        parse_component_values("calc(-1%)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let CssComponentValueRef::Function(function) = recovered.components().items()[0].view() else {
        panic!("function")
    };
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::Parsed(_)
    ));
    for (calculation, origin_kind) in [(programmatic, 0), (recovered, 1)] {
        let weight = CssColorMixWeight::try_calculation(calculation).unwrap();
        let root = CssColor::from_color_mix(
            CssColorMix::try_new(
                None,
                vec![
                    CssColorMixComponent::new(
                        CssColor::from_named(CssNamedColor::try_new("red").unwrap()),
                        Some(weight),
                    ),
                    CssColorMixComponent::new(CssColor::transparent(), None),
                ],
            )
            .unwrap(),
        );
        let original_origin = root.color_mix_value().unwrap().components()[0]
            .weight()
            .unwrap()
            .calculation()
            .unwrap()
            .origin();
        let result = resolve_authored_color(&root, &facts, facts.identity(), limits()).unwrap();
        let CssAuthoredColorOutcome::StandardFailure(failure) = result.outcome() else {
            panic!("negative weight")
        };
        assert_eq!(failure.path.steps(), [CssColorPathStep::MixWeight(0)]);
        assert_eq!(failure.kind, CssColorStandardFailureKind::InvalidComponents);
        match (&failure.origin, origin_kind) {
            (Some(CssValueOrigin::Programmatic), 0) => {}
            (Some(CssValueOrigin::Parsed(origin)), 1) => {
                assert_eq!(origin.source().as_str(), "calc(-1%)");
                let CssValueOrigin::Parsed(original) = original_origin else {
                    panic!("original parsed")
                };
                assert!(
                    std::ptr::eq(origin.source().as_str(), original.source().as_str()),
                    "the actual source backing is shared, not reconstructed"
                );
            }
            actual => panic!("wrong original origin: {actual:?}"),
        }
    }
}

#[test]
fn parser_produced_eof_recovery_remains_resolvable_and_retains_implicit_closures() {
    let text = "color: light-dark(red,rgb(0 0 calc(2 + 3";
    let report = parse_declaration_list_text(text);
    assert!(!report.is_clean());
    assert_eq!(report.syntax().len(), 1);
    let root = declaration_color(&report.syntax()[0]);
    let CssColorComponent::NumberCalculation(calculation) = &root
        .light_dark_value()
        .unwrap()
        .dark()
        .rgb_value()
        .unwrap()
        .channels()[2]
    else {
        panic!("calculation")
    };
    let CssComponentValueRef::Function(function) = calculation.components().items()[0].view()
    else {
        panic!("function")
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("actual EOF closure")
    };
    assert_eq!(opening.source().as_str(), text);
    assert_eq!(at.source().as_str(), text);
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(41, 7),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Dark,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    let result = resolve_authored_color(root, &facts, facts.identity(), limits()).unwrap();
    let CssAuthoredColorOutcome::Used(value) = result.outcome() else {
        panic!("recovered checked syntax remains valid")
    };
    assert_eq!(value.channels(), [Some(0.0), Some(0.0), Some(5.0 / 255.0)]);
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    let source = &text[7..];
    let parsed = parse_used_color(
        CssColorInput::Utf8(source.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits {
            input_units: source.len(),
            algorithm_records: 100_000,
            arithmetic_records: 10_000,
            evaluation_records: 10_000,
            actions: 30_000_000,
        },
    )
    .unwrap();
    let CssColorOrdinaryOutcome::Used(source_value) = parsed.outcome() else {
        panic!("canonical source EOF recovery")
    };
    assert_eq!(
        source_value.channels(),
        [Some(0.0), Some(0.0), Some(5.0 / 255.0)]
    );
}
