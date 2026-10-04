#![forbid(unsafe_code)]

//! Fonts 4 WD 2026-09-07 §9.1 and the frozen WebKit palette-mix-valid.html
//! provide independent authored grammar and specified-output expectations.
//! The shared accepted Color 5 omitted-weight reconciliation also applies.
//! Functional tests for new APIs, separate from the executable lifecycle RED.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("font-palette:{text}!important"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn palette(source: &CssDeclaration) -> &CssFontPalette {
    let CssKnownPropertyValueRef::FontPalette(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary font palette")
    };
    value.palette()
}

fn mix(value: &CssFontPalette) -> &CssFontPaletteMix {
    let CssFontPalette::Mix(value) = value else {
        panic!("authored mix")
    };
    value
}

fn literal(text: &str) -> CssColorMixWeight {
    let components = parse_component_values(text).unwrap();
    CssColorMixWeight::literal(
        CssColorMixPercentage::try_from_component(components.items()[0].clone()).unwrap(),
    )
}

fn assert_text(authored: &str, expected: &str) {
    let parsed = declaration(authored);
    let components = parse_component_values(authored).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::FontPalette),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &components);
    for source in [&parsed, &checked] {
        let before = source.clone();
        let value = palette(source);
        assert_eq!(value.serialize_specified().unwrap(), expected, "{authored}");
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(source, &before);
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            authored
        );
    }
    // Canonical output is admitted and stable without comparing source origins.
    assert_eq!(
        palette(&declaration(expected))
            .serialize_specified()
            .unwrap(),
        expected
    );
}

#[test]
fn typed_keywords_and_names_retain_their_distinct_authored_meanings() {
    assert_eq!(palette(&declaration("NORMAL")), &CssFontPalette::Normal);
    assert_eq!(palette(&declaration("light")), &CssFontPalette::Light);
    assert_eq!(palette(&declaration("dark")), &CssFontPalette::Dark);
    for (text, name) in [
        ("--Theme", "--Theme"),
        (r"--\54 heme", "--Theme"),
        ("--", "--"),
    ] {
        let source = declaration(text);
        let CssFontPalette::Named(value) = palette(&source) else {
            panic!("named palette")
        };
        assert_eq!(value.as_str(), name);
        assert_text(text, name);
    }
    assert_ne!(
        palette(&declaration("--Theme")),
        palette(&declaration("--theme"))
    );
    for (value, expected) in [
        (CssFontPalette::Normal, "normal"),
        (CssFontPalette::Light, "light"),
        (CssFontPalette::Dark, "dark"),
        (
            CssFontPalette::Named(CssFontPaletteName::try_new("--A B").unwrap()),
            r"--A\ B",
        ),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn checked_mixes_preserve_order_duplicates_weights_and_omitted_method() {
    assert_eq!(
        CssFontPaletteMix::try_new(None, vec![]).unwrap_err(),
        CssFontPaletteMixConstructionError::EmptyComponents
    );
    let weight = literal("25%");
    let named = CssFontPalette::Named(CssFontPaletteName::try_new("--Theme").unwrap());
    let value = CssFontPaletteMix::try_new(
        None,
        vec![
            CssFontPaletteMixComponent::new(named.clone(), Some(weight.clone())),
            CssFontPaletteMixComponent::new(CssFontPalette::Dark, None),
            CssFontPaletteMixComponent::new(named.clone(), None),
        ],
    )
    .unwrap();
    assert!(value.interpolation().is_none());
    assert_eq!(value.components().len(), 3);
    assert_eq!(value.components()[0].palette(), &named);
    assert_eq!(value.components()[1].palette(), &CssFontPalette::Dark);
    assert_eq!(value.components()[2].palette(), &named);
    assert_eq!(value.components()[0].weight(), Some(&weight));
    assert!(value.components()[1].weight().is_none());
    assert_eq!(
        CssFontPalette::Mix(Box::new(value))
            .serialize_specified()
            .unwrap(),
        "palette-mix(--Theme 25%, dark 37.5%, --Theme 37.5%)"
    );
}

#[test]
fn parsed_components_expose_ordered_palettes_and_original_weight_origins() {
    let text = "font-palette:palette-mix(25% --Theme,dark 50%,--Theme)";
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let value = mix(palette(&report.syntax()[0]));
    assert_eq!(value.components().len(), 3);
    for (component, expected) in value
        .components()
        .iter()
        .zip(["--Theme", "dark", "--Theme"])
    {
        match component.palette() {
            CssFontPalette::Named(name) => assert_eq!(name.as_str(), expected),
            CssFontPalette::Dark => assert_eq!(expected, "dark"),
            _ => panic!("original ordered palette"),
        }
    }
    for (component, expected) in value.components()[..2].iter().zip(["25%", "50%"]) {
        let literal = component
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal();
        let CssValueOrigin::Parsed(origin) = literal.origin() else {
            panic!("parsed percentage")
        };
        let start = text.find(expected).unwrap();
        assert_eq!(origin.source().as_str(), text);
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + expected.len()
        );
    }
    assert!(value.components()[2].weight().is_none());
}

#[test]
fn explicit_default_and_nondefault_interpolation_keep_authored_method_identity() {
    let omitted = declaration("palette-mix(light,dark)");
    assert!(mix(palette(&omitted)).interpolation().is_none());
    let explicit = declaration("palette-mix(in oklab,light,dark)");
    assert_eq!(
        mix(palette(&explicit))
            .interpolation()
            .unwrap()
            .predefined()
            .unwrap()
            .space(),
        CssColorInterpolationSpace::Oklab
    );
    assert_text(
        "palette-mix(in oklab,light,dark)",
        "palette-mix(light, dark)",
    );
    assert_text(
        "palette-mix(in xyz,light 10%,dark)",
        "palette-mix(in xyz-d65, light 10%, dark 90%)",
    );
    assert_text(
        "palette-mix(in oklch shorter hue, light, dark)",
        "palette-mix(in oklch, light, dark)",
    );
    assert_text(
        "palette-mix(in oklch longer hue, light, dark)",
        "palette-mix(in oklch longer hue, light, dark)",
    );
    assert_text(
        r"palette-mix(in --\50 rofile, light, dark)",
        "palette-mix(in --Profile, light, dark)",
    );
    let custom = CssColorInterpolation::custom(CssColorProfileName::try_new("--P").unwrap());
    let value = CssFontPaletteMix::try_new(
        Some(custom.clone()),
        vec![CssFontPaletteMixComponent::new(CssFontPalette::Dark, None)],
    )
    .unwrap();
    assert_eq!(value.interpolation(), Some(&custom));
    assert_eq!(
        CssFontPalette::Mix(Box::new(value))
            .serialize_specified()
            .unwrap(),
        "palette-mix(in --P, dark)"
    );
}

#[test]
fn declared_weight_output_follows_exact_shared_omission_and_fill_rules() {
    for (authored, expected) in [
        (
            "palette-mix(30% light, dark)",
            "palette-mix(light 30%, dark 70%)",
        ),
        (
            "palette-mix(light 50%, dark 50%)",
            "palette-mix(light, dark)",
        ),
        (
            "palette-mix(light 75%, dark 75%)",
            "palette-mix(light 75%, dark 75%)",
        ),
        (
            "palette-mix(light 0%, dark 0%)",
            "palette-mix(light 0%, dark 0%)",
        ),
        (
            "palette-mix(dark, light, --x)",
            "palette-mix(dark, light, --x)",
        ),
        (
            "palette-mix(dark 50%, light, --x)",
            "palette-mix(dark 50%, light 25%, --x 25%)",
        ),
        // Reuse the accepted bounded specified-sum policy; never invent a negative fill.
        (
            "palette-mix(dark 75%, light 75%, --x)",
            "palette-mix(dark 75%, light 75%, --x 0%)",
        ),
        (
            "palette-mix(dark 33.333333%, light, --x)",
            "palette-mix(dark 33.333333%, light 33.333334%, --x 33.333334%)",
        ),
        ("palette-mix(dark 100%)", "palette-mix(dark)"),
        ("palette-mix(dark 50%)", "palette-mix(dark 50%)"),
        ("palette-mix(dark 0%)", "palette-mix(dark 0%)"),
    ] {
        assert_text(authored, expected);
    }
}

#[test]
fn calculated_weights_keep_their_graph_and_leave_omitted_weights_unknown() {
    for (authored, expected) in [
        (
            "palette-mix(light calc(10%), dark)",
            "palette-mix(light calc(10%), dark)",
        ),
        (
            "palette-mix(light calc(-10%), dark)",
            "palette-mix(light calc(-10%), dark)",
        ),
        (
            "palette-mix(light calc(150%), dark)",
            "palette-mix(light calc(150%), dark)",
        ),
        (
            "palette-mix(light calc(50%), dark 50%)",
            "palette-mix(light calc(50%), dark 50%)",
        ),
    ] {
        assert_text(authored, expected);
    }
    let graph = parse_component_values("calc(100% - 20%)").unwrap();
    let root = CssPercentageCalculation::try_from_components(graph.clone()).unwrap();
    let weight = CssColorMixWeight::try_calculation(root.clone()).unwrap();
    let value = CssFontPaletteMix::try_new(
        None,
        vec![
            CssFontPaletteMixComponent::new(CssFontPalette::Light, Some(weight)),
            CssFontPaletteMixComponent::new(CssFontPalette::Dark, None),
        ],
    )
    .unwrap();
    let retained = value.components()[0]
        .weight()
        .unwrap()
        .calculation()
        .unwrap();
    assert_eq!(retained, &root);
    assert_eq!(retained.components(), &graph);
    assert!(value.components()[1].weight().is_none());
    let source = CssFontPalette::Mix(Box::new(value));
    assert_eq!(
        source.serialize_specified().unwrap(),
        "palette-mix(light calc(80%), dark)"
    );
}

#[test]
fn nested_and_singleton_functions_are_serialized_without_computed_simplification() {
    assert_text(
        "palette-mix(palette-mix(in srgb, light 30%, normal) 20%, dark)",
        "palette-mix(palette-mix(in srgb, light 30%, normal 70%) 20%, dark 80%)",
    );
    assert_text("palette-mix(in srgb, dark)", "palette-mix(in srgb, dark)");
    assert_text(
        "palette-mix(light 30%, light)",
        "palette-mix(light 30%, light 70%)",
    );
    let source = declaration("palette-mix(palette-mix(dark), --x)");
    assert!(matches!(
        mix(palette(&source)).components()[0].palette(),
        CssFontPalette::Mix(_)
    ));
}

#[test]
fn checked_palette_and_math_graphs_share_the_complete_256_level_ceiling() {
    let mut value = CssFontPalette::Dark;
    for _ in 0..256 {
        value = CssFontPalette::Mix(Box::new(
            CssFontPaletteMix::try_new(None, vec![CssFontPaletteMixComponent::new(value, None)])
                .unwrap(),
        ));
    }
    assert!(value.serialize_specified().is_ok());
    assert_eq!(
        CssFontPaletteMix::try_new(None, vec![CssFontPaletteMixComponent::new(value, None),])
            .unwrap_err(),
        CssFontPaletteMixConstructionError::NestingLimit
    );
    let text = format!("{}10%{}", "calc(".repeat(256), ")".repeat(256));
    let graph = parse_component_values(&text).unwrap();
    let calculation = CssPercentageCalculation::try_from_components(graph).unwrap();
    let weight = CssColorMixWeight::try_calculation(calculation).unwrap();
    assert_eq!(
        CssFontPaletteMix::try_new(
            None,
            vec![CssFontPaletteMixComponent::new(
                CssFontPalette::Light,
                Some(weight)
            ),]
        )
        .unwrap_err(),
        CssFontPaletteMixConstructionError::NestingLimit
    );
}

#[test]
fn specified_limits_are_cumulative_atomic_and_exact_for_simple_palette_leaves() {
    for (value, expected) in [
        (CssFontPalette::Normal, "normal"),
        (
            CssFontPalette::Named(CssFontPaletteName::try_new("--Theme").unwrap()),
            "--Theme",
        ),
    ] {
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (Limits::new(0, 1, expected.len()), Kind::InputNodeLimit),
            (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(1, 1, expected.len() - 1), Kind::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, before);
            assert_eq!(value.serialize_specified().unwrap(), expected);
        }
    }
    let source =
        declaration("palette-mix(in oklch longer hue, palette-mix(light 30%, dark), --Theme)");
    let value = palette(&source);
    let before = source.clone();
    let expected = "palette-mix(in oklch longer hue, palette-mix(light 30%, dark 70%), --Theme)";
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(1, usize::MAX, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(usize::MAX, 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(usize::MAX, usize::MAX, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(source, before);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn strict_checked_admission_and_reentry_reject_implicit_closures() {
    let recovered = parse_component_values("palette-mix(dark").unwrap();
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::FontPalette),
            recovered.clone(),
            CssImportance::Normal
        )
        .is_err()
    );
    let source = declaration("var(--palette)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    assert!(matches!(
        pending.reenter(recovered).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let replacement =
        parse_component_values("palette-mix(in --Profile, --Theme calc(10%), dark)").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("palette")
    };
    let CssLonghandValueRef::FontPalette(value) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("typed contribution")
    };
    assert_eq!(
        value.serialize_specified().unwrap(),
        "palette-mix(in --Profile, --Theme calc(10%), dark)"
    );
    assert!(values.items()[0].source().same_occurrence(&source));
    assert_eq!(
        values.items()[0].replacement_components(),
        Some(&replacement)
    );
}

#[test]
fn shared_argument_failures_keep_the_palette_property_diagnostic_context() {
    for value in [
        "palette-mix(light 101%, dark)",
        "palette-mix(light -1%, dark)",
        "palette-mix(in srgb longer hue, light, dark)",
    ] {
        let report = parse_style_attribute(&format!("font-palette:{value};color:red"));
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(report.diagnostics().len(), 1);
        let ErrorKind::InvalidPropertyValue(detail) = report.diagnostics()[0].error().kind() else {
            panic!("palette grammar failure: {:?}", report.diagnostics())
        };
        assert_eq!(detail.property(), CssKnownProperty::FontPalette);
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::FontPalette),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(
            matches!(error.kind(), CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail))
            if detail.property() == CssKnownProperty::FontPalette)
        );
    }
}

#[test]
fn palette_support_metadata_names_the_exact_pinned_property_source() {
    let metadata = property_support_metadata("FONT-PALETTE").unwrap();
    assert_eq!(metadata.property(), CssKnownProperty::FontPalette);
    assert_eq!(
        metadata.feature().id().as_str(),
        "official.property.font-palette"
    );
    assert_eq!(
        metadata.feature().source().id().as_str(),
        "I-FONTS4-20260907"
    );
    assert_eq!(
        metadata.feature().source().url(),
        Some("https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/")
    );
    assert_eq!(metadata.feature().production(), "#propdef-font-palette");
    assert_eq!(metadata.feature().status(), CssSupportStatus::Complete);
    assert_eq!(metadata.feature().supported_subset(), None);
    assert_eq!(metadata.feature().unsupported_remainder(), None);
}
